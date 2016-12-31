use reexport::*;
use rustc::hir::*;
use rustc::hir::def_id::DefId;
use rustc::hir::map::Node;
use rustc::lint::{LintContext, LateContext, Level, Lint};
use rustc::middle::cstore;
use rustc::session::Session;
use rustc::traits::Reveal;
use rustc::traits;
use rustc::ty::subst::Subst;
use rustc::ty;
use rustc_errors;
use std::borrow::Cow;
use std::env;
use std::mem;
use std::str::FromStr;
use syntax::ast::{self, LitKind};
use syntax::codemap::{ExpnFormat, ExpnInfo, MultiSpan, Span, DUMMY_SP};
use syntax::errors::DiagnosticBuilder;
use syntax::ptr::P;

pub mod cargo;
pub mod comparisons;
pub mod conf;
pub mod constants;
mod hir;
pub mod paths;
pub mod sugg;
pub mod internal_lints;
pub use self::hir::{SpanlessEq, SpanlessHash};

pub type MethodArgs = HirVec<P<Expr>>;

/// Produce a nested chain of if-lets and ifs from the patterns:
///
///     if_let_chain! {[
///         let Some(y) = x,
///         y.len() == 2,
///         let Some(z) = y,
///     ], {
///         block
///     }}
///
/// becomes
///
///     if let Some(y) = x {
///         if y.len() == 2 {
///             if let Some(z) = y {
///                 block
///             }
///         }
///     }
#[macro_export]
macro_rules! if_let_chain {
    ([let $pat:pat = $expr:expr, $($tt:tt)+], $block:block) => {
        if let $pat = $expr {
           if_let_chain!{ [$($tt)+], $block }
        }
    };
    ([let $pat:pat = $expr:expr], $block:block) => {
        if let $pat = $expr {
           $block
        }
    };
    ([let $pat:pat = $expr:expr,], $block:block) => {
        if let $pat = $expr {
           $block
        }
    };
    ([$expr:expr, $($tt:tt)+], $block:block) => {
        if $expr {
           if_let_chain!{ [$($tt)+], $block }
        }
    };
    ([$expr:expr], $block:block) => {
        if $expr {
           $block
        }
    };
    ([$expr:expr,], $block:block) => {
        if $expr {
           $block
        }
    };
}

pub mod higher;

/// Returns true if the two spans come from differing expansions (i.e. one is from a macro and one
/// isn't).
pub fn differing_macro_contexts(lhs: Span, rhs: Span) -> bool {
    rhs.expn_id != lhs.expn_id
}
/// Returns true if this `expn_info` was expanded by any macro.
pub fn in_macro<T: LintContext>(cx: &T, span: Span) -> bool {
    cx.sess().codemap().with_expn_info(span.expn_id, |info| info.is_some())
}

/// Returns true if the macro that expanded the crate was outside of the current crate or was a
/// compiler plugin.
pub fn in_external_macro<T: LintContext>(cx: &T, span: Span) -> bool {
    /// Invokes `in_macro` with the expansion info of the given span slightly heavy, try to use
    /// this after other checks have already happened.
    fn in_macro_ext<T: LintContext>(cx: &T, opt_info: Option<&ExpnInfo>) -> bool {
        // no ExpnInfo = no macro
        opt_info.map_or(false, |info| {
            if let ExpnFormat::MacroAttribute(..) = info.callee.format {
                // these are all plugins
                return true;
            }
            // no span for the callee = external macro
            info.callee.span.map_or(true, |span| {
                // no snippet = external macro or compiler-builtin expansion
                cx.sess().codemap().span_to_snippet(span).ok().map_or(true, |code| !code.starts_with("macro_rules"))
            })
        })
    }

    cx.sess().codemap().with_expn_info(span.expn_id, |info| in_macro_ext(cx, info))
}

/// Check if a `DefId`'s path matches the given absolute type path usage.
///
/// # Examples
/// ```
/// match_def_path(cx, id, &["core", "option", "Option"])
/// ```
///
/// See also the `paths` module.
pub fn match_def_path(cx: &LateContext, def_id: DefId, path: &[&str]) -> bool {
    use syntax::parse::token;

    struct AbsolutePathBuffer {
        names: Vec<token::InternedString>,
    }

    impl ty::item_path::ItemPathBuffer for AbsolutePathBuffer {
        fn root_mode(&self) -> &ty::item_path::RootMode {
            const ABSOLUTE: &'static ty::item_path::RootMode = &ty::item_path::RootMode::Absolute;
            ABSOLUTE
        }

        fn push(&mut self, text: &str) {
            self.names.push(token::intern(text).as_str());
        }
    }

    let mut apb = AbsolutePathBuffer { names: vec![] };

    cx.tcx.push_item_path(&mut apb, def_id);

    apb.names == path
}

/// Check if type is struct or enum type with given def path.
pub fn match_type(cx: &LateContext, ty: ty::Ty, path: &[&str]) -> bool {
    match ty.sty {
        ty::TyEnum(adt, _) |
        ty::TyStruct(adt, _) => match_def_path(cx, adt.did, path),
        _ => false,
    }
}

/// Check if the method call given in `expr` belongs to given type.
pub fn match_impl_method(cx: &LateContext, expr: &Expr, path: &[&str]) -> bool {
    let method_call = ty::MethodCall::expr(expr.id);

    let trt_id = cx.tcx
                   .tables
                   .borrow()
                   .method_map
                   .get(&method_call)
                   .and_then(|callee| cx.tcx.impl_of_method(callee.def_id));
    if let Some(trt_id) = trt_id {
        match_def_path(cx, trt_id, path)
    } else {
        false
    }
}

/// Check if the method call given in `expr` belongs to given trait.
pub fn match_trait_method(cx: &LateContext, expr: &Expr, path: &[&str]) -> bool {
    let method_call = ty::MethodCall::expr(expr.id);

    let trt_id = cx.tcx
                   .tables
                   .borrow()
                   .method_map
                   .get(&method_call)
                   .and_then(|callee| cx.tcx.trait_of_item(callee.def_id));
    if let Some(trt_id) = trt_id {
        match_def_path(cx, trt_id, path)
    } else {
        false
    }
}

/// Match a `Path` against a slice of segment string literals.
///
/// # Examples
/// ```
/// match_path(path, &["std", "rt", "begin_unwind"])
/// ```
pub fn match_path(path: &Path, segments: &[&str]) -> bool {
    path.segments.iter().rev().zip(segments.iter().rev()).all(|(a, b)| a.name.as_str() == *b)
}

/// Match a `Path` against a slice of segment string literals, e.g.
///
/// # Examples
/// ```
/// match_path(path, &["std", "rt", "begin_unwind"])
/// ```
pub fn match_path_ast(path: &ast::Path, segments: &[&str]) -> bool {
    path.segments.iter().rev().zip(segments.iter().rev()).all(|(a, b)| a.identifier.name.as_str() == *b)
}

/// Get the definition associated to a path.
/// TODO: investigate if there is something more efficient for that.
pub fn path_to_def(cx: &LateContext, path: &[&str]) -> Option<cstore::DefLike> {
    let cstore = &cx.tcx.sess.cstore;

    let crates = cstore.crates();
    let krate = crates.iter().find(|&&krate| cstore.crate_name(krate) == path[0]);
    if let Some(krate) = krate {
        let mut items = cstore.crate_top_level_items(*krate);
        let mut path_it = path.iter().skip(1).peekable();

        loop {
            let segment = match path_it.next() {
                Some(segment) => segment,
                None => return None,
            };

            for item in &mem::replace(&mut items, vec![]) {
                if item.name.as_str() == *segment {
                    if path_it.peek().is_none() {
                        return Some(item.def);
                    }

                    let def_id = match item.def {
                        cstore::DefLike::DlDef(def) => def.def_id(),
                        cstore::DefLike::DlImpl(def_id) => def_id,
                        _ => panic!("Unexpected {:?}", item.def),
                    };

                    items = cstore.item_children(def_id);
                    break;
                }
            }
        }
    } else {
        None
    }
}

/// Convenience function to get the `DefId` of a trait by path.
pub fn get_trait_def_id(cx: &LateContext, path: &[&str]) -> Option<DefId> {
    let def = match path_to_def(cx, path) {
        Some(def) => def,
        None => return None,
    };

    match def {
        cstore::DlDef(def::Def::Trait(trait_id)) => Some(trait_id),
        _ => None,
    }
}

/// Check whether a type implements a trait.
/// See also `get_trait_def_id`.
pub fn implements_trait<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, ty: ty::Ty<'tcx>, trait_id: DefId,
                                  ty_params: Vec<ty::Ty<'tcx>>)
                                  -> bool {
    cx.tcx.populate_implementations_for_trait_if_necessary(trait_id);

    let ty = cx.tcx.erase_regions(&ty);
    cx.tcx.infer_ctxt(None, None, Reveal::All).enter(|infcx| {
        let obligation = cx.tcx.predicate_for_trait_def(traits::ObligationCause::dummy(),
                                                        trait_id,
                                                        0,
                                                        ty,
                                                        &ty_params);

        traits::SelectionContext::new(&infcx).evaluate_obligation_conservatively(&obligation)
    })
}

/// Match an `Expr` against a chain of methods, and return the matched `Expr`s.
///
/// For example, if `expr` represents the `.baz()` in `foo.bar().baz()`,
/// `matched_method_chain(expr, &["bar", "baz"])` will return a `Vec` containing the `Expr`s for
/// `.bar()` and `.baz()`
pub fn method_chain_args<'a>(expr: &'a Expr, methods: &[&str]) -> Option<Vec<&'a MethodArgs>> {
    let mut current = expr;
    let mut matched = Vec::with_capacity(methods.len());
    for method_name in methods.iter().rev() {
        // method chains are stored last -> first
        if let ExprMethodCall(ref name, _, ref args) = current.node {
            if name.node.as_str() == *method_name {
                matched.push(args); // build up `matched` backwards
                current = &args[0] // go to parent expression
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    matched.reverse(); // reverse `matched`, so that it is in the same order as `methods`
    Some(matched)
}


/// Get the name of the item the expression is in, if available.
pub fn get_item_name(cx: &LateContext, expr: &Expr) -> Option<Name> {
    let parent_id = cx.tcx.map.get_parent(expr.id);
    match cx.tcx.map.find(parent_id) {
        Some(Node::NodeItem(&Item { ref name, .. })) |
        Some(Node::NodeTraitItem(&TraitItem { ref name, .. })) |
        Some(Node::NodeImplItem(&ImplItem { ref name, .. })) => Some(*name),
        _ => None,
    }
}

/// Convert a span to a code snippet if available, otherwise use default.
///
/// # Example
/// ```
/// snippet(cx, expr.span, "..")
/// ```
pub fn snippet<'a, T: LintContext>(cx: &T, span: Span, default: &'a str) -> Cow<'a, str> {
    cx.sess().codemap().span_to_snippet(span).map(From::from).unwrap_or_else(|_| Cow::Borrowed(default))
}

/// Convert a span to a code snippet. Returns `None` if not available.
pub fn snippet_opt<T: LintContext>(cx: &T, span: Span) -> Option<String> {
    cx.sess().codemap().span_to_snippet(span).ok()
}

/// Convert a span (from a block) to a code snippet if available, otherwise use default.
/// This trims the code of indentation, except for the first line. Use it for blocks or block-like
/// things which need to be printed as such.
///
/// # Example
/// ```
/// snippet(cx, expr.span, "..")
/// ```
pub fn snippet_block<'a, T: LintContext>(cx: &T, span: Span, default: &'a str) -> Cow<'a, str> {
    let snip = snippet(cx, span, default);
    trim_multiline(snip, true)
}

/// Like `snippet_block`, but add braces if the expr is not an `ExprBlock`.
/// Also takes an `Option<String>` which can be put inside the braces.
pub fn expr_block<'a, T: LintContext>(cx: &T, expr: &Expr, option: Option<String>, default: &'a str) -> Cow<'a, str> {
    let code = snippet_block(cx, expr.span, default);
    let string = option.unwrap_or_default();
    if let ExprBlock(_) = expr.node {
        Cow::Owned(format!("{}{}", code, string))
    } else if string.is_empty() {
        Cow::Owned(format!("{{ {} }}", code))
    } else {
        Cow::Owned(format!("{{\n{};\n{}\n}}", code, string))
    }
}

/// Trim indentation from a multiline string with possibility of ignoring the first line.
pub fn trim_multiline(s: Cow<str>, ignore_first: bool) -> Cow<str> {
    let s_space = trim_multiline_inner(s, ignore_first, ' ');
    let s_tab = trim_multiline_inner(s_space, ignore_first, '\t');
    trim_multiline_inner(s_tab, ignore_first, ' ')
}

fn trim_multiline_inner(s: Cow<str>, ignore_first: bool, ch: char) -> Cow<str> {
    let x = s.lines()
             .skip(ignore_first as usize)
             .filter_map(|l| {
                 if l.is_empty() {
                     None
                 } else {
                     // ignore empty lines
                     Some(l.char_indices()
                           .find(|&(_, x)| x != ch)
                           .unwrap_or((l.len(), ch))
                           .0)
                 }
             })
             .min()
             .unwrap_or(0);
    if x > 0 {
        Cow::Owned(s.lines()
                    .enumerate()
                    .map(|(i, l)| {
                        if (ignore_first && i == 0) || l.is_empty() {
                            l
                        } else {
                            l.split_at(x).1
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n"))
    } else {
        s
    }
}

/// Get a parent expressions if any – this is useful to constrain a lint.
pub fn get_parent_expr<'c>(cx: &'c LateContext, e: &Expr) -> Option<&'c Expr> {
    let map = &cx.tcx.map;
    let node_id: NodeId = e.id;
    let parent_id: NodeId = map.get_parent_node(node_id);
    if node_id == parent_id {
        return None;
    }
    map.find(parent_id).and_then(|node| {
        if let Node::NodeExpr(parent) = node {
            Some(parent)
        } else {
            None
        }
    })
}

pub fn get_enclosing_block<'c>(cx: &'c LateContext, node: NodeId) -> Option<&'c Block> {
    let map = &cx.tcx.map;
    let enclosing_node = map.get_enclosing_scope(node)
                            .and_then(|enclosing_id| map.find(enclosing_id));
    if let Some(node) = enclosing_node {
        match node {
            Node::NodeBlock(block) => Some(block),
            Node::NodeItem(&Item { node: ItemFn(_, _, _, _, _, ref block), .. }) => Some(block),
            _ => None,
        }
    } else {
        None
    }
}

pub struct DiagnosticWrapper<'a>(pub DiagnosticBuilder<'a>);

impl<'a> Drop for DiagnosticWrapper<'a> {
    fn drop(&mut self) {
        self.0.emit();
    }
}

impl<'a> DiagnosticWrapper<'a> {
    fn wiki_link(&mut self, lint: &'static Lint) {
        if env::var("CLIPPY_DISABLE_WIKI_LINKS").is_err() {
            self.0.help(&format!("for further information visit https://github.com/Manishearth/rust-clippy/wiki#{}",
                               lint.name_lower()));
        }
    }
}

pub fn span_lint<T: LintContext>(cx: &T, lint: &'static Lint, sp: Span, msg: &str) {
    let mut db = DiagnosticWrapper(cx.struct_span_lint(lint, sp, msg));
    if cx.current_level(lint) != Level::Allow {
        db.wiki_link(lint);
    }
}

// FIXME: needless lifetime doesn't trigger here
pub fn span_help_and_lint<'a, T: LintContext>(cx: &'a T, lint: &'static Lint, span: Span, msg: &str, help: &str) {
    let mut db = DiagnosticWrapper(cx.struct_span_lint(lint, span, msg));
    if cx.current_level(lint) != Level::Allow {
        db.0.help(help);
        db.wiki_link(lint);
    }
}

pub fn span_note_and_lint<'a, T: LintContext>(cx: &'a T, lint: &'static Lint, span: Span, msg: &str, note_span: Span,
                                              note: &str) {
    let mut db = DiagnosticWrapper(cx.struct_span_lint(lint, span, msg));
    if cx.current_level(lint) != Level::Allow {
        if note_span == span {
            db.0.note(note);
        } else {
            db.0.span_note(note_span, note);
        }
        db.wiki_link(lint);
    }
}

pub fn span_lint_and_then<'a, T: LintContext, F>(cx: &'a T, lint: &'static Lint, sp: Span, msg: &str, f: F)
    where F: FnOnce(&mut DiagnosticBuilder<'a>)
{
    let mut db = DiagnosticWrapper(cx.struct_span_lint(lint, sp, msg));
    if cx.current_level(lint) != Level::Allow {
        f(&mut db.0);
        db.wiki_link(lint);
    }
}

/// Create a suggestion made from several `span → replacement`.
///
/// Note: in the JSON format (used by `compiletest_rs`), the help message will appear once per
/// replacement. In human-readable format though, it only appears once before the whole suggestion.
pub fn multispan_sugg(db: &mut DiagnosticBuilder, help_msg: String, sugg: &[(Span, &str)]) {
    let sugg = rustc_errors::RenderSpan::Suggestion(rustc_errors::CodeSuggestion {
        msp: MultiSpan::from_spans(sugg.iter().map(|&(span, _)| span).collect()),
        substitutes: sugg.iter().map(|&(_, subs)| subs.to_owned()).collect(),
    });

    let sub = rustc_errors::SubDiagnostic {
        level: rustc_errors::Level::Help,
        message: help_msg,
        span: MultiSpan::new(),
        render_span: Some(sugg),
    };
    db.children.push(sub);
}

/// Return the base type for references and raw pointers.
pub fn walk_ptrs_ty(ty: ty::Ty) -> ty::Ty {
    match ty.sty {
        ty::TyRef(_, ref tm) => walk_ptrs_ty(tm.ty),
        _ => ty,
    }
}

/// Return the base type for references and raw pointers, and count reference depth.
pub fn walk_ptrs_ty_depth(ty: ty::Ty) -> (ty::Ty, usize) {
    fn inner(ty: ty::Ty, depth: usize) -> (ty::Ty, usize) {
        match ty.sty {
            ty::TyRef(_, ref tm) => inner(tm.ty, depth + 1),
            _ => (ty, depth),
        }
    }
    inner(ty, 0)
}

/// Check whether the given expression is a constant literal of the given value.
pub fn is_integer_literal(expr: &Expr, value: u64) -> bool {
    // FIXME: use constant folding
    if let ExprLit(ref spanned) = expr.node {
        if let LitKind::Int(v, _) = spanned.node {
            return v == value;
        }
    }
    false
}

pub fn is_adjusted(cx: &LateContext, e: &Expr) -> bool {
    cx.tcx.tables.borrow().adjustments.get(&e.id).is_some()
}

pub struct LimitStack {
    stack: Vec<u64>,
}

impl Drop for LimitStack {
    fn drop(&mut self) {
        assert_eq!(self.stack.len(), 1);
    }
}

impl LimitStack {
    pub fn new(limit: u64) -> LimitStack {
        LimitStack { stack: vec![limit] }
    }
    pub fn limit(&self) -> u64 {
        *self.stack.last().expect("there should always be a value in the stack")
    }
    pub fn push_attrs(&mut self, sess: &Session, attrs: &[ast::Attribute], name: &'static str) {
        let stack = &mut self.stack;
        parse_attrs(sess, attrs, name, |val| stack.push(val));
    }
    pub fn pop_attrs(&mut self, sess: &Session, attrs: &[ast::Attribute], name: &'static str) {
        let stack = &mut self.stack;
        parse_attrs(sess, attrs, name, |val| assert_eq!(stack.pop(), Some(val)));
    }
}

fn parse_attrs<F: FnMut(u64)>(sess: &Session, attrs: &[ast::Attribute], name: &'static str, mut f: F) {
    for attr in attrs {
        let attr = &attr.node;
        if attr.is_sugared_doc {
            continue;
        }
        if let ast::MetaItemKind::NameValue(ref key, ref value) = attr.value.node {
            if *key == name {
                if let LitKind::Str(ref s, _) = value.node {
                    if let Ok(value) = FromStr::from_str(s) {
                        f(value)
                    } else {
                        sess.span_err(value.span, "not a number");
                    }
                } else {
                    unreachable!()
                }
            }
        }
    }
}

/// Return the pre-expansion span if is this comes from an expansion of the macro `name`.
/// See also `is_direct_expn_of`.
pub fn is_expn_of(cx: &LateContext, mut span: Span, name: &str) -> Option<Span> {
    loop {
        let span_name_span = cx.tcx
                               .sess
                               .codemap()
                               .with_expn_info(span.expn_id, |expn| expn.map(|ei| (ei.callee.name(), ei.call_site)));

        match span_name_span {
            Some((mac_name, new_span)) if mac_name.as_str() == name => return Some(new_span),
            None => return None,
            Some((_, new_span)) => span = new_span,
        }
    }
}

/// Return the pre-expansion span if is this directly comes from an expansion of the macro `name`.
/// The difference with `is_expn_of` is that in
/// ```rust,ignore
/// foo!(bar!(42));
/// ```
/// `42` is considered expanded from `foo!` and `bar!` by `is_expn_of` but only `bar!` by
/// `is_direct_expn_of`.
pub fn is_direct_expn_of(cx: &LateContext, span: Span, name: &str) -> Option<Span> {
    let span_name_span = cx.tcx
                           .sess
                           .codemap()
                           .with_expn_info(span.expn_id, |expn| expn.map(|ei| (ei.callee.name(), ei.call_site)));

    match span_name_span {
        Some((mac_name, new_span)) if mac_name.as_str() == name => Some(new_span),
        _ => None,
    }
}

/// Return the index of the character after the first camel-case component of `s`.
pub fn camel_case_until(s: &str) -> usize {
    let mut iter = s.char_indices();
    if let Some((_, first)) = iter.next() {
        if !first.is_uppercase() {
            return 0;
        }
    } else {
        return 0;
    }
    let mut up = true;
    let mut last_i = 0;
    for (i, c) in iter {
        if up {
            if c.is_lowercase() {
                up = false;
            } else {
                return last_i;
            }
        } else if c.is_uppercase() {
            up = true;
            last_i = i;
        } else if !c.is_lowercase() {
            return i;
        }
    }
    if up {
        last_i
    } else {
        s.len()
    }
}

/// Return index of the last camel-case component of `s`.
pub fn camel_case_from(s: &str) -> usize {
    let mut iter = s.char_indices().rev();
    if let Some((_, first)) = iter.next() {
        if !first.is_lowercase() {
            return s.len();
        }
    } else {
        return s.len();
    }
    let mut down = true;
    let mut last_i = s.len();
    for (i, c) in iter {
        if down {
            if c.is_uppercase() {
                down = false;
                last_i = i;
            } else if !c.is_lowercase() {
                return last_i;
            }
        } else if c.is_lowercase() {
            down = true;
        } else {
            return last_i;
        }
    }
    last_i
}

/// Convenience function to get the return type of a function
pub fn return_ty<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, fn_item: NodeId) -> ty::Ty<'tcx> {
    let parameter_env = ty::ParameterEnvironment::for_item(cx.tcx, fn_item);
    let fn_sig = cx.tcx.node_id_to_type(fn_item).fn_sig().subst(cx.tcx, parameter_env.free_substs);
    let fn_sig = cx.tcx.liberate_late_bound_regions(parameter_env.free_id_outlive, &fn_sig);
    fn_sig.output
}

/// Check if two types are the same.
// FIXME: this works correctly for lifetimes bounds (`for <'a> Foo<'a>` == `for <'b> Foo<'b>` but
// not for type parameters.
pub fn same_tys<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, a: ty::Ty<'tcx>, b: ty::Ty<'tcx>, parameter_item: NodeId) -> bool {
    let parameter_env = ty::ParameterEnvironment::for_item(cx.tcx, parameter_item);
    cx.tcx.infer_ctxt(None, Some(parameter_env), Reveal::All).enter(|infcx| {
        let new_a = a.subst(infcx.tcx, infcx.parameter_environment.free_substs);
        let new_b = b.subst(infcx.tcx, infcx.parameter_environment.free_substs);
        infcx.can_equate(&new_a, &new_b).is_ok()
    })
}

/// Return whether the given type is an `unsafe` function.
pub fn type_is_unsafe_function(ty: ty::Ty) -> bool {
    match ty.sty {
        ty::TyFnDef(_, _, f) |
        ty::TyFnPtr(f) => f.unsafety == Unsafety::Unsafe,
        _ => false,
    }
}

pub fn is_copy<'a, 'ctx>(cx: &LateContext<'a, 'ctx>, ty: ty::Ty<'ctx>, env: NodeId) -> bool {
    let env = ty::ParameterEnvironment::for_item(cx.tcx, env);
    !ty.subst(cx.tcx, env.free_substs).moves_by_default(cx.tcx.global_tcx(), &env, DUMMY_SP)
}
