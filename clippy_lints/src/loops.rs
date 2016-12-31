use reexport::*;
use rustc::hir::*;
use rustc::hir::def::Def;
use rustc::hir::def_id::DefId;
use rustc::hir::intravisit::{Visitor, walk_expr, walk_block, walk_decl};
use rustc::hir::map::Node::NodeBlock;
use rustc::lint::*;
use rustc::middle::const_val::ConstVal;
use rustc::middle::region::CodeExtent;
use rustc::ty;
use rustc_const_eval::EvalHint::ExprTypeChecked;
use rustc_const_eval::eval_const_expr_partial;
use std::collections::HashMap;
use syntax::ast;
use utils::sugg;

use utils::{snippet, span_lint, get_parent_expr, match_trait_method, match_type, multispan_sugg, in_external_macro,
            span_help_and_lint, is_integer_literal, get_enclosing_block, span_lint_and_then, higher,
            walk_ptrs_ty};
use utils::paths;

/// **What it does:** Checks for looping over the range of `0..len` of some
/// collection just to get the values by index.
///
/// **Why is this bad?** Just iterating the collection itself makes the intent
/// more clear and is probably faster.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// for i in 0..vec.len() {
///     println!("{}", vec[i]);
/// }
/// ```
declare_lint! {
    pub NEEDLESS_RANGE_LOOP,
    Warn,
    "for-looping over a range of indices where an iterator over items would do"
}

/// **What it does:** Checks for loops on `x.iter()` where `&x` will do, and
/// suggests the latter.
///
/// **Why is this bad?** Readability.
///
/// **Known problems:** False negatives. We currently only warn on some known
/// types.
///
/// **Example:**
/// ```rust
/// // with `y` a `Vec` or slice:
/// for x in y.iter() { .. }
/// ```
declare_lint! {
    pub EXPLICIT_ITER_LOOP,
    Warn,
    "for-looping over `_.iter()` or `_.iter_mut()` when `&_` or `&mut _` would do"
}

/// **What it does:** Checks for loops on `x.next()`.
///
/// **Why is this bad?** `next()` returns either `Some(value)` if there was a
/// value, or `None` otherwise. The insidious thing is that `Option<_>`
/// implements `IntoIterator`, so that possibly one value will be iterated,
/// leading to some hard to find bugs. No one will want to write such code
/// [except to win an Underhanded Rust
/// Contest](https://www.reddit.com/r/rust/comments/3hb0wm/underhanded_rust_contest/cu5yuhr).
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// for x in y.next() { .. }
/// ```
declare_lint! {
    pub ITER_NEXT_LOOP,
    Warn,
    "for-looping over `_.next()` which is probably not intended"
}

/// **What it does:** Checks for `for` loops over `Option` values.
///
/// **Why is this bad?** Readability. This is more clearly expressed as an `if let`.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// for x in option { .. }
/// ```
///
/// This should be
/// ```rust
/// if let Some(x) = option { .. }
/// ```
declare_lint! {
    pub FOR_LOOP_OVER_OPTION,
    Warn,
    "for-looping over an `Option`, which is more clearly expressed as an `if let`"
}

/// **What it does:** Checks for `for` loops over `Result` values.
///
/// **Why is this bad?** Readability. This is more clearly expressed as an `if let`.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// for x in result { .. }
/// ```
///
/// This should be
/// ```rust
/// if let Ok(x) = result { .. }
/// ```
declare_lint! {
    pub FOR_LOOP_OVER_RESULT,
    Warn,
    "for-looping over a `Result`, which is more clearly expressed as an `if let`"
}

/// **What it does:** Detects `loop + match` combinations that are easier
/// written as a `while let` loop.
///
/// **Why is this bad?** The `while let` loop is usually shorter and more readable.
///
/// **Known problems:** Sometimes the wrong binding is displayed (#383).
///
/// **Example:**
/// ```rust
/// loop {
///     let x = match y {
///         Some(x) => x,
///         None => break,
///     }
///     // .. do something with x
/// }
/// // is easier written as
/// while let Some(x) = y {
///     // .. do something with x
/// }
/// ```
declare_lint! {
    pub WHILE_LET_LOOP,
    Warn,
    "`loop { if let { ... } else break }`, which can be written as a `while let` loop"
}

/// **What it does:** Checks for using `collect()` on an iterator without using
/// the result.
///
/// **Why is this bad?** It is more idiomatic to use a `for` loop over the
/// iterator instead.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// vec.iter().map(|x| /* some operation returning () */).collect::<Vec<_>>();
/// ```
declare_lint! {
    pub UNUSED_COLLECT,
    Warn,
    "`collect()`ing an iterator without using the result; this is usually better \
     written as a for loop"
}

/// **What it does:** Checks for loops over ranges `x..y` where both `x` and `y`
/// are constant and `x` is greater or equal to `y`, unless the range is
/// reversed or has a negative `.step_by(_)`.
///
/// **Why is it bad?** Such loops will either be skipped or loop until
/// wrap-around (in debug code, this may `panic!()`). Both options are probably
/// not intended.
///
/// **Known problems:** The lint cannot catch loops over dynamically defined
/// ranges. Doing this would require simulating all possible inputs and code
/// paths through the program, which would be complex and error-prone.
///
/// **Example:**
/// ```rust
/// for x in 5..10-5 { .. } // oops, stray `-`
/// ```
declare_lint! {
    pub REVERSE_RANGE_LOOP,
    Warn,
    "iteration over an empty range, such as `10..0` or `5..5`"
}

/// **What it does:** Checks `for` loops over slices with an explicit counter
/// and suggests the use of `.enumerate()`.
///
/// **Why is it bad?** Not only is the version using `.enumerate()` more
/// readable, the compiler is able to remove bounds checks which can lead to
/// faster code in some instances.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// for i in 0..v.len() { foo(v[i]);
/// for i in 0..v.len() { bar(i, v[i]); }
/// ```
declare_lint! {
    pub EXPLICIT_COUNTER_LOOP,
    Warn,
    "for-looping with an explicit counter when `_.enumerate()` would do"
}

/// **What it does:** Checks for empty `loop` expressions.
///
/// **Why is this bad?** Those busy loops burn CPU cycles without doing
/// anything. Think of the environment and either block on something or at least
/// make the thread sleep for some microseconds.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// loop {}
/// ```
declare_lint! {
    pub EMPTY_LOOP,
    Warn,
    "empty `loop {}`, which should block or sleep"
}

/// **What it does:** Checks for `while let` expressions on iterators.
///
/// **Why is this bad?** Readability. A simple `for` loop is shorter and conveys
/// the intent better.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// while let Some(val) = iter() { .. }
/// ```
declare_lint! {
    pub WHILE_LET_ON_ITERATOR,
    Warn,
    "using a while-let loop instead of a for loop on an iterator"
}

/// **What it does:** Checks for iterating a map (`HashMap` or `BTreeMap`) and
/// ignoring either the keys or values.
///
/// **Why is this bad?** Readability. There are `keys` and `values` methods that
/// can be used to express that don't need the values or keys.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// for (k, _) in &map { .. }
/// ```
///
/// could be replaced by
///
/// ```rust
/// for k in map.keys() { .. }
/// ```
declare_lint! {
    pub FOR_KV_MAP,
    Warn,
    "looping on a map using `iter` when `keys` or `values` would do"
}

#[derive(Copy, Clone)]
pub struct Pass;

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(NEEDLESS_RANGE_LOOP,
                    EXPLICIT_ITER_LOOP,
                    ITER_NEXT_LOOP,
                    FOR_LOOP_OVER_RESULT,
                    FOR_LOOP_OVER_OPTION,
                    WHILE_LET_LOOP,
                    UNUSED_COLLECT,
                    REVERSE_RANGE_LOOP,
                    EXPLICIT_COUNTER_LOOP,
                    EMPTY_LOOP,
                    WHILE_LET_ON_ITERATOR,
                    FOR_KV_MAP)
    }
}

impl LateLintPass for Pass {
    fn check_expr(&mut self, cx: &LateContext, expr: &Expr) {
        if let Some((pat, arg, body)) = higher::for_loop(expr) {
            check_for_loop(cx, pat, arg, body, expr);
        }
        // check for `loop { if let {} else break }` that could be `while let`
        // (also matches an explicit "match" instead of "if let")
        // (even if the "match" or "if let" is used for declaration)
        if let ExprLoop(ref block, _) = expr.node {
            // also check for empty `loop {}` statements
            if block.stmts.is_empty() && block.expr.is_none() {
                span_lint(cx,
                          EMPTY_LOOP,
                          expr.span,
                          "empty `loop {}` detected. You may want to either use `panic!()` or add \
                           `std::thread::sleep(..);` to the loop body.");
            }

            // extract the expression from the first statement (if any) in a block
            let inner_stmt_expr = extract_expr_from_first_stmt(block);
            // or extract the first expression (if any) from the block
            if let Some(inner) = inner_stmt_expr.or_else(|| extract_first_expr(block)) {
                if let ExprMatch(ref matchexpr, ref arms, ref source) = inner.node {
                    // ensure "if let" compatible match structure
                    match *source {
                        MatchSource::Normal |
                        MatchSource::IfLetDesugar { .. } => {
                            if arms.len() == 2 &&
                               arms[0].pats.len() == 1 && arms[0].guard.is_none() &&
                               arms[1].pats.len() == 1 && arms[1].guard.is_none() &&
                               is_break_expr(&arms[1].body) {
                                if in_external_macro(cx, expr.span) {
                                    return;
                                }

                                // NOTE: we used to make build a body here instead of using
                                // ellipsis, this was removed because:
                                // 1) it was ugly with big bodies;
                                // 2) it was not indented properly;
                                // 3) it wasn’t very smart (see #675).
                                span_lint_and_then(cx,
                                                   WHILE_LET_LOOP,
                                                   expr.span,
                                                   "this loop could be written as a `while let` loop",
                                                   |db| {
                                                       let sug = format!("while let {} = {} {{ .. }}",
                                                                         snippet(cx, arms[0].pats[0].span, ".."),
                                                                         snippet(cx, matchexpr.span, ".."));
                                                       db.span_suggestion(expr.span, "try", sug);
                                                   });
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
        if let ExprMatch(ref match_expr, ref arms, MatchSource::WhileLetDesugar) = expr.node {
            let pat = &arms[0].pats[0].node;
            if let (&PatKind::TupleStruct(ref path, ref pat_args, _),
                    &ExprMethodCall(method_name, _, ref method_args)) = (pat, &match_expr.node) {
                let iter_expr = &method_args[0];
                if let Some(lhs_constructor) = path.segments.last() {
                    if method_name.node.as_str() == "next" &&
                       match_trait_method(cx, match_expr, &paths::ITERATOR) &&
                       lhs_constructor.name.as_str() == "Some" &&
                       !is_iterator_used_after_while_let(cx, iter_expr) {
                        let iterator = snippet(cx, method_args[0].span, "_");
                        let loop_var = snippet(cx, pat_args[0].span, "_");
                        span_lint_and_then(cx,
                                           WHILE_LET_ON_ITERATOR,
                                           expr.span,
                                           "this loop could be written as a `for` loop",
                                           |db| {
                        db.span_suggestion(expr.span,
                                           "try",
                                           format!("for {} in {} {{ .. }}", loop_var, iterator));
                        });
                    }
                }
            }
        }
    }

    fn check_stmt(&mut self, cx: &LateContext, stmt: &Stmt) {
        if let StmtSemi(ref expr, _) = stmt.node {
            if let ExprMethodCall(ref method, _, ref args) = expr.node {
                if args.len() == 1 && method.node.as_str() == "collect" &&
                   match_trait_method(cx, expr, &paths::ITERATOR) {
                    span_lint(cx,
                              UNUSED_COLLECT,
                              expr.span,
                              "you are collect()ing an iterator and throwing away the result. \
                               Consider using an explicit for loop to exhaust the iterator");
                }
            }
        }
    }
}

fn check_for_loop(cx: &LateContext, pat: &Pat, arg: &Expr, body: &Expr, expr: &Expr) {
    check_for_loop_range(cx, pat, arg, body, expr);
    check_for_loop_reverse_range(cx, arg, expr);
    check_for_loop_arg(cx, pat, arg, expr);
    check_for_loop_explicit_counter(cx, arg, body, expr);
    check_for_loop_over_map_kv(cx, pat, arg, body, expr);
}

/// Check for looping over a range and then indexing a sequence with it.
/// The iteratee must be a range literal.
fn check_for_loop_range(cx: &LateContext, pat: &Pat, arg: &Expr, body: &Expr, expr: &Expr) {
    if let Some(higher::Range { start: Some(start), ref end, limits }) = higher::range(arg) {
        // the var must be a single name
        if let PatKind::Binding(_, ref ident, _) = pat.node {
            let mut visitor = VarVisitor {
                cx: cx,
                var: cx.tcx.expect_def(pat.id).def_id(),
                indexed: HashMap::new(),
                nonindex: false,
            };
            walk_expr(&mut visitor, body);

            // linting condition: we only indexed one variable
            if visitor.indexed.len() == 1 {
                let (indexed, indexed_extent) = visitor.indexed
                                                       .into_iter()
                                                       .next()
                                                       .unwrap_or_else(|| unreachable!() /* len == 1 */);

                // ensure that the indexed variable was declared before the loop, see #601
                if let Some(indexed_extent) = indexed_extent {
                    let pat_extent = cx.tcx.region_maps.var_scope(pat.id);
                    if cx.tcx.region_maps.is_subscope_of(indexed_extent, pat_extent) {
                        return;
                    }
                }

                let starts_at_zero = is_integer_literal(start, 0);

                let skip = if starts_at_zero {
                    "".to_owned()
                } else {
                    format!(".skip({})", snippet(cx, start.span, ".."))
                };

                let take = if let Some(end) = *end {
                    if is_len_call(end, &indexed) {
                        "".to_owned()
                    } else {
                        match limits {
                            ast::RangeLimits::Closed => {
                                let end = sugg::Sugg::hir(cx, end, "<count>");
                                format!(".take({})", end + sugg::ONE)
                            }
                            ast::RangeLimits::HalfOpen => {
                                format!(".take({})", snippet(cx, end.span, ".."))
                            }
                        }
                    }
                } else {
                    "".to_owned()
                };

                if visitor.nonindex {
                    span_lint_and_then(cx,
                                       NEEDLESS_RANGE_LOOP,
                                       expr.span,
                                       &format!("the loop variable `{}` is used to index `{}`", ident.node, indexed),
                                       |db| {
                        multispan_sugg(db, "consider using an iterator".to_string(), &[
                            (pat.span, &format!("({}, <item>)", ident.node)),
                            (arg.span, &format!("{}.iter().enumerate(){}{}", indexed, take, skip)),
                        ]);
                    });
                } else {
                    let repl = if starts_at_zero && take.is_empty() {
                        format!("&{}", indexed)
                    } else {
                        format!("{}.iter(){}{}", indexed, take, skip)
                    };

                    span_lint_and_then(cx,
                                       NEEDLESS_RANGE_LOOP,
                                       expr.span,
                                       &format!("the loop variable `{}` is only used to index `{}`.", ident.node, indexed),
                                       |db| {
                        multispan_sugg(db, "consider using an iterator".to_string(), &[
                            (pat.span, "<item>"),
                            (arg.span, &repl),
                        ]);
                    });
                }
            }
        }
    }
}

fn is_len_call(expr: &Expr, var: &Name) -> bool {
    if_let_chain! {[
        let ExprMethodCall(method, _, ref len_args) = expr.node,
        len_args.len() == 1,
        method.node.as_str() == "len",
        let ExprPath(_, ref path) = len_args[0].node,
        path.segments.len() == 1,
        &path.segments[0].name == var
    ], {
        return true;
    }}

    false
}

fn check_for_loop_reverse_range(cx: &LateContext, arg: &Expr, expr: &Expr) {
    // if this for loop is iterating over a two-sided range...
    if let Some(higher::Range { start: Some(start), end: Some(end), limits }) = higher::range(arg) {
        // ...and both sides are compile-time constant integers...
        if let Ok(start_idx) = eval_const_expr_partial(cx.tcx, start, ExprTypeChecked, None) {
            if let Ok(end_idx) = eval_const_expr_partial(cx.tcx, end, ExprTypeChecked, None) {
                // ...and the start index is greater than the end index,
                // this loop will never run. This is often confusing for developers
                // who think that this will iterate from the larger value to the
                // smaller value.
                let (sup, eq) = match (start_idx, end_idx) {
                    (ConstVal::Integral(start_idx), ConstVal::Integral(end_idx)) => {
                        (start_idx > end_idx, start_idx == end_idx)
                    }
                    _ => (false, false),
                };

                if sup {
                    let start_snippet = snippet(cx, start.span, "_");
                    let end_snippet = snippet(cx, end.span, "_");
                    let dots = if limits == ast::RangeLimits::Closed {
                        "..."
                    } else {
                        ".."
                    };

                    span_lint_and_then(cx,
                                       REVERSE_RANGE_LOOP,
                                       expr.span,
                                       "this range is empty so this for loop will never run",
                                       |db| {
                                           db.span_suggestion(arg.span,
                                                              "consider using the following if \
                                                               you are attempting to iterate \
                                                               over this range in reverse",
                                                              format!("({end}{dots}{start}).rev()",
                                                                      end=end_snippet,
                                                                      dots=dots,
                                                                      start=start_snippet));
                                       });
                } else if eq && limits != ast::RangeLimits::Closed {
                    // if they are equal, it's also problematic - this loop
                    // will never run.
                    span_lint(cx,
                              REVERSE_RANGE_LOOP,
                              expr.span,
                              "this range is empty so this for loop will never run");
                }
            }
        }
    }
}

fn check_for_loop_arg(cx: &LateContext, pat: &Pat, arg: &Expr, expr: &Expr) {
    let mut next_loop_linted = false; // whether or not ITER_NEXT_LOOP lint was used
    if let ExprMethodCall(ref method, _, ref args) = arg.node {
        // just the receiver, no arguments
        if args.len() == 1 {
            let method_name = method.node;
            // check for looping over x.iter() or x.iter_mut(), could use &x or &mut x
            if method_name.as_str() == "iter" || method_name.as_str() == "iter_mut" {
                if is_ref_iterable_type(cx, &args[0]) {
                    let object = snippet(cx, args[0].span, "_");
                    span_lint(cx,
                              EXPLICIT_ITER_LOOP,
                              expr.span,
                              &format!("it is more idiomatic to loop over `&{}{}` instead of `{}.{}()`",
                                       if method_name.as_str() == "iter_mut" {
                                           "mut "
                                       } else {
                                           ""
                                       },
                                       object,
                                       object,
                                       method_name));
                }
            } else if method_name.as_str() == "next" && match_trait_method(cx, arg, &paths::ITERATOR) {
                span_lint(cx,
                          ITER_NEXT_LOOP,
                          expr.span,
                          "you are iterating over `Iterator::next()` which is an Option; this will compile but is \
                           probably not what you want");
                next_loop_linted = true;
            }
        }
    }
    if !next_loop_linted {
        check_arg_type(cx, pat, arg);
    }
}

/// Check for `for` loops over `Option`s and `Results`
fn check_arg_type(cx: &LateContext, pat: &Pat, arg: &Expr) {
    let ty = cx.tcx.expr_ty(arg);
    if match_type(cx, ty, &paths::OPTION) {
        span_help_and_lint(cx,
                           FOR_LOOP_OVER_OPTION,
                           arg.span,
                           &format!("for loop over `{0}`, which is an `Option`. This is more readably written as an \
                                     `if let` statement.",
                                    snippet(cx, arg.span, "_")),
                           &format!("consider replacing `for {0} in {1}` with `if let Some({0}) = {1}`",
                                    snippet(cx, pat.span, "_"),
                                    snippet(cx, arg.span, "_")));
    } else if match_type(cx, ty, &paths::RESULT) {
        span_help_and_lint(cx,
                           FOR_LOOP_OVER_RESULT,
                           arg.span,
                           &format!("for loop over `{0}`, which is a `Result`. This is more readably written as an \
                                     `if let` statement.",
                                    snippet(cx, arg.span, "_")),
                           &format!("consider replacing `for {0} in {1}` with `if let Ok({0}) = {1}`",
                                    snippet(cx, pat.span, "_"),
                                    snippet(cx, arg.span, "_")));
    }
}

fn check_for_loop_explicit_counter(cx: &LateContext, arg: &Expr, body: &Expr, expr: &Expr) {
    // Look for variables that are incremented once per loop iteration.
    let mut visitor = IncrementVisitor {
        cx: cx,
        states: HashMap::new(),
        depth: 0,
        done: false,
    };
    walk_expr(&mut visitor, body);

    // For each candidate, check the parent block to see if
    // it's initialized to zero at the start of the loop.
    let map = &cx.tcx.map;
    let parent_scope = map.get_enclosing_scope(expr.id).and_then(|id| map.get_enclosing_scope(id));
    if let Some(parent_id) = parent_scope {
        if let NodeBlock(block) = map.get(parent_id) {
            for (id, _) in visitor.states.iter().filter(|&(_, v)| *v == VarState::IncrOnce) {
                let mut visitor2 = InitializeVisitor {
                    cx: cx,
                    end_expr: expr,
                    var_id: *id,
                    state: VarState::IncrOnce,
                    name: None,
                    depth: 0,
                    past_loop: false,
                };
                walk_block(&mut visitor2, block);

                if visitor2.state == VarState::Warn {
                    if let Some(name) = visitor2.name {
                        span_lint(cx,
                                  EXPLICIT_COUNTER_LOOP,
                                  expr.span,
                                  &format!("the variable `{0}` is used as a loop counter. Consider using `for ({0}, \
                                            item) in {1}.enumerate()` or similar iterators",
                                           name,
                                           snippet(cx, arg.span, "_")));
                    }
                }
            }
        }
    }
}

/// Check for the `FOR_KV_MAP` lint.
fn check_for_loop_over_map_kv(cx: &LateContext, pat: &Pat, arg: &Expr, body: &Expr, expr: &Expr) {
    let pat_span = pat.span;

    if let PatKind::Tuple(ref pat, _) = pat.node {
        if pat.len() == 2 {
            let (new_pat_span, kind) = match (&pat[0].node, &pat[1].node) {
                (key, _) if pat_is_wild(key, body) => (pat[1].span, "value"),
                (_, value) if pat_is_wild(value, body) => (pat[0].span, "key"),
                _ => return,
            };

            let (arg_span, arg) = match arg.node {
                ExprAddrOf(MutImmutable, ref expr) => (arg.span, &**expr),
                ExprAddrOf(MutMutable, _) => return, // for _ in &mut _, there is no {values,keys}_mut method
                _ => (arg.span, arg),
            };

            let ty = walk_ptrs_ty(cx.tcx.expr_ty(arg));
            if match_type(cx, ty, &paths::HASHMAP) || match_type(cx, ty, &paths::BTREEMAP) {
                span_lint_and_then(cx,
                                   FOR_KV_MAP,
                                   expr.span,
                                   &format!("you seem to want to iterate on a map's {}s", kind),
                                   |db| {
                    let map = sugg::Sugg::hir(cx, arg, "map");
                    multispan_sugg(db, "use the corresponding method".into(), &[
                        (pat_span, &snippet(cx, new_pat_span, kind)),
                        (arg_span, &format!("{}.{}s()", map.maybe_par(), kind)),
                    ]);
                });
            }
        }
    }

}

/// Return true if the pattern is a `PatWild` or an ident prefixed with `'_'`.
fn pat_is_wild(pat: &PatKind, body: &Expr) -> bool {
    match *pat {
        PatKind::Wild => true,
        PatKind::Binding(_, ident, None) if ident.node.as_str().starts_with('_') => {
            let mut visitor = UsedVisitor {
                var: ident.node,
                used: false,
            };
            walk_expr(&mut visitor, body);
            !visitor.used
        }
        _ => false,
    }
}

struct UsedVisitor {
    var: ast::Name, // var to look for
    used: bool, // has the var been used otherwise?
}

impl<'a> Visitor<'a> for UsedVisitor {
    fn visit_expr(&mut self, expr: &Expr) {
        if let ExprPath(None, ref path) = expr.node {
            if path.segments.len() == 1 && path.segments[0].name == self.var {
                self.used = true;
                return;
            }
        }

        walk_expr(self, expr);
    }
}

struct VarVisitor<'v, 't: 'v> {
    cx: &'v LateContext<'v, 't>, // context reference
    var: DefId, // var name to look for as index
    indexed: HashMap<Name, Option<CodeExtent>>, // indexed variables, the extent is None for global
    nonindex: bool, // has the var been used otherwise?
}

impl<'v, 't> Visitor<'v> for VarVisitor<'v, 't> {
    fn visit_expr(&mut self, expr: &'v Expr) {
        if let ExprPath(None, ref path) = expr.node {
            if path.segments.len() == 1 && self.cx.tcx.expect_def(expr.id).def_id() == self.var {
                // we are referencing our variable! now check if it's as an index
                if_let_chain! {[
                    let Some(parexpr) = get_parent_expr(self.cx, expr),
                    let ExprIndex(ref seqexpr, _) = parexpr.node,
                    let ExprPath(None, ref seqvar) = seqexpr.node,
                    seqvar.segments.len() == 1
                ], {
                    let def_map = self.cx.tcx.def_map.borrow();
                    if let Some(def) = def_map.get(&seqexpr.id) {
                        match def.base_def {
                            Def::Local(..) | Def::Upvar(..) => {
                                let extent = self.cx.tcx.region_maps.var_scope(def.base_def.var_id());
                                self.indexed.insert(seqvar.segments[0].name, Some(extent));
                                return;  // no need to walk further
                            }
                            Def::Static(..) | Def::Const(..) => {
                                self.indexed.insert(seqvar.segments[0].name, None);
                                return;  // no need to walk further
                            }
                            _ => (),
                        }
                    }
                }}
                // we are not indexing anything, record that
                self.nonindex = true;
                return;
            }
        }
        walk_expr(self, expr);
    }
}

fn is_iterator_used_after_while_let(cx: &LateContext, iter_expr: &Expr) -> bool {
    let def_id = match var_def_id(cx, iter_expr) {
        Some(id) => id,
        None => return false,
    };
    let mut visitor = VarUsedAfterLoopVisitor {
        cx: cx,
        def_id: def_id,
        iter_expr_id: iter_expr.id,
        past_while_let: false,
        var_used_after_while_let: false,
    };
    if let Some(enclosing_block) = get_enclosing_block(cx, def_id) {
        walk_block(&mut visitor, enclosing_block);
    }
    visitor.var_used_after_while_let
}

struct VarUsedAfterLoopVisitor<'v, 't: 'v> {
    cx: &'v LateContext<'v, 't>,
    def_id: NodeId,
    iter_expr_id: NodeId,
    past_while_let: bool,
    var_used_after_while_let: bool,
}

impl<'v, 't> Visitor<'v> for VarUsedAfterLoopVisitor<'v, 't> {
    fn visit_expr(&mut self, expr: &'v Expr) {
        if self.past_while_let {
            if Some(self.def_id) == var_def_id(self.cx, expr) {
                self.var_used_after_while_let = true;
            }
        } else if self.iter_expr_id == expr.id {
            self.past_while_let = true;
        }
        walk_expr(self, expr);
    }
}


/// Return true if the type of expr is one that provides `IntoIterator` impls
/// for `&T` and `&mut T`, such as `Vec`.
#[cfg_attr(rustfmt, rustfmt_skip)]
fn is_ref_iterable_type(cx: &LateContext, e: &Expr) -> bool {
    // no walk_ptrs_ty: calling iter() on a reference can make sense because it
    // will allow further borrows afterwards
    let ty = cx.tcx.expr_ty(e);
    is_iterable_array(ty) ||
    match_type(cx, ty, &paths::VEC) ||
    match_type(cx, ty, &paths::LINKED_LIST) ||
    match_type(cx, ty, &paths::HASHMAP) ||
    match_type(cx, ty, &paths::HASHSET) ||
    match_type(cx, ty, &paths::VEC_DEQUE) ||
    match_type(cx, ty, &paths::BINARY_HEAP) ||
    match_type(cx, ty, &paths::BTREEMAP) ||
    match_type(cx, ty, &paths::BTREESET)
}

fn is_iterable_array(ty: ty::Ty) -> bool {
    // IntoIterator is currently only implemented for array sizes <= 32 in rustc
    match ty.sty {
        ty::TyArray(_, 0...32) => true,
        _ => false,
    }
}

/// If a block begins with a statement (possibly a `let` binding) and has an expression, return it.
fn extract_expr_from_first_stmt(block: &Block) -> Option<&Expr> {
    if block.stmts.is_empty() {
        return None;
    }
    if let StmtDecl(ref decl, _) = block.stmts[0].node {
        if let DeclLocal(ref local) = decl.node {
            if let Some(ref expr) = local.init {
                Some(expr)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// If a block begins with an expression (with or without semicolon), return it.
fn extract_first_expr(block: &Block) -> Option<&Expr> {
    match block.expr {
        Some(ref expr) if block.stmts.is_empty() => Some(expr),
        None if !block.stmts.is_empty() => {
            match block.stmts[0].node {
                StmtExpr(ref expr, _) | StmtSemi(ref expr, _) => Some(expr),
                StmtDecl(..) => None,
            }
        }
        _ => None,
    }
}

/// Return true if expr contains a single break expr (maybe within a block).
fn is_break_expr(expr: &Expr) -> bool {
    match expr.node {
        ExprBreak(None) => true,
        ExprBlock(ref b) => {
            match extract_first_expr(b) {
                Some(subexpr) => is_break_expr(subexpr),
                None => false,
            }
        }
        _ => false,
    }
}

// To trigger the EXPLICIT_COUNTER_LOOP lint, a variable must be
// incremented exactly once in the loop body, and initialized to zero
// at the start of the loop.
#[derive(PartialEq)]
enum VarState {
    Initial, // Not examined yet
    IncrOnce, // Incremented exactly once, may be a loop counter
    Declared, // Declared but not (yet) initialized to zero
    Warn,
    DontWarn,
}

/// Scan a for loop for variables that are incremented exactly once.
struct IncrementVisitor<'v, 't: 'v> {
    cx: &'v LateContext<'v, 't>, // context reference
    states: HashMap<NodeId, VarState>, // incremented variables
    depth: u32, // depth of conditional expressions
    done: bool,
}

impl<'v, 't> Visitor<'v> for IncrementVisitor<'v, 't> {
    fn visit_expr(&mut self, expr: &'v Expr) {
        if self.done {
            return;
        }

        // If node is a variable
        if let Some(def_id) = var_def_id(self.cx, expr) {
            if let Some(parent) = get_parent_expr(self.cx, expr) {
                let state = self.states.entry(def_id).or_insert(VarState::Initial);

                match parent.node {
                    ExprAssignOp(op, ref lhs, ref rhs) => {
                        if lhs.id == expr.id {
                            if op.node == BiAdd && is_integer_literal(rhs, 1) {
                                *state = match *state {
                                    VarState::Initial if self.depth == 0 => VarState::IncrOnce,
                                    _ => VarState::DontWarn,
                                };
                            } else {
                                // Assigned some other value
                                *state = VarState::DontWarn;
                            }
                        }
                    }
                    ExprAssign(ref lhs, _) if lhs.id == expr.id => *state = VarState::DontWarn,
                    ExprAddrOf(mutability, _) if mutability == MutMutable => *state = VarState::DontWarn,
                    _ => (),
                }
            }
        } else if is_loop(expr) {
            self.states.clear();
            self.done = true;
            return;
        } else if is_conditional(expr) {
            self.depth += 1;
            walk_expr(self, expr);
            self.depth -= 1;
            return;
        }
        walk_expr(self, expr);
    }
}

/// Check whether a variable is initialized to zero at the start of a loop.
struct InitializeVisitor<'v, 't: 'v> {
    cx: &'v LateContext<'v, 't>, // context reference
    end_expr: &'v Expr, // the for loop. Stop scanning here.
    var_id: NodeId,
    state: VarState,
    name: Option<Name>,
    depth: u32, // depth of conditional expressions
    past_loop: bool,
}

impl<'v, 't> Visitor<'v> for InitializeVisitor<'v, 't> {
    fn visit_decl(&mut self, decl: &'v Decl) {
        // Look for declarations of the variable
        if let DeclLocal(ref local) = decl.node {
            if local.pat.id == self.var_id {
                if let PatKind::Binding(_, ref ident, _) = local.pat.node {
                    self.name = Some(ident.node);

                    self.state = if let Some(ref init) = local.init {
                        if is_integer_literal(init, 0) {
                            VarState::Warn
                        } else {
                            VarState::Declared
                        }
                    } else {
                        VarState::Declared
                    }
                }
            }
        }
        walk_decl(self, decl);
    }

    fn visit_expr(&mut self, expr: &'v Expr) {
        if self.state == VarState::DontWarn {
            return;
        }
        if expr == self.end_expr {
            self.past_loop = true;
            return;
        }
        // No need to visit expressions before the variable is
        // declared
        if self.state == VarState::IncrOnce {
            return;
        }

        // If node is the desired variable, see how it's used
        if var_def_id(self.cx, expr) == Some(self.var_id) {
            if let Some(parent) = get_parent_expr(self.cx, expr) {
                match parent.node {
                    ExprAssignOp(_, ref lhs, _) if lhs.id == expr.id => {
                        self.state = VarState::DontWarn;
                    }
                    ExprAssign(ref lhs, ref rhs) if lhs.id == expr.id => {
                        self.state = if is_integer_literal(rhs, 0) && self.depth == 0 {
                            VarState::Warn
                        } else {
                            VarState::DontWarn
                        }
                    }
                    ExprAddrOf(mutability, _) if mutability == MutMutable => self.state = VarState::DontWarn,
                    _ => (),
                }
            }

            if self.past_loop {
                self.state = VarState::DontWarn;
                return;
            }
        } else if !self.past_loop && is_loop(expr) {
            self.state = VarState::DontWarn;
            return;
        } else if is_conditional(expr) {
            self.depth += 1;
            walk_expr(self, expr);
            self.depth -= 1;
            return;
        }
        walk_expr(self, expr);
    }
}

fn var_def_id(cx: &LateContext, expr: &Expr) -> Option<NodeId> {
    if let Some(path_res) = cx.tcx.def_map.borrow().get(&expr.id) {
        if let Def::Local(_, node_id) = path_res.base_def {
            return Some(node_id);
        }
    }
    None
}

fn is_loop(expr: &Expr) -> bool {
    match expr.node {
        ExprLoop(..) | ExprWhile(..) => true,
        _ => false,
    }
}

fn is_conditional(expr: &Expr) -> bool {
    match expr.node {
        ExprIf(..) | ExprMatch(..) => true,
        _ => false,
    }
}
