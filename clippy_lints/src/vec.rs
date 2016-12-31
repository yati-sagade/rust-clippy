use rustc::hir::*;
use rustc::lint::*;
use rustc::ty;
use rustc_const_eval::EvalHint::ExprTypeChecked;
use rustc_const_eval::eval_const_expr_partial;
use syntax::codemap::Span;
use utils::{higher, is_copy, snippet, span_lint_and_then};

/// **What it does:** Checks for usage of `&vec![..]` when using `&[..]` would
/// be possible.
///
/// **Why is this bad?** This is less efficient.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust,ignore
/// foo(&vec![1, 2])
/// ```
declare_lint! {
    pub USELESS_VEC,
    Warn,
    "useless `vec!`"
}

#[derive(Copy, Clone, Debug)]
pub struct Pass;

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(USELESS_VEC)
    }
}

impl LateLintPass for Pass {
    fn check_expr(&mut self, cx: &LateContext, expr: &Expr) {
        // search for `&vec![_]` expressions where the adjusted type is `&[_]`
        if_let_chain!{[
            let ty::TypeVariants::TyRef(_, ref ty) = cx.tcx.expr_ty_adjusted(expr).sty,
            let ty::TypeVariants::TySlice(..) = ty.ty.sty,
            let ExprAddrOf(_, ref addressee) = expr.node,
            let Some(vec_args) = higher::vec_macro(cx, addressee),
        ], {
            check_vec_macro(cx, &vec_args, expr.span);
        }}

        // search for `for _ in vec![…]`
        if_let_chain!{[
            let Some((_, arg, _)) = higher::for_loop(expr),
            let Some(vec_args) = higher::vec_macro(cx, arg),
            is_copy(cx, vec_type(cx.tcx.expr_ty_adjusted(arg)), cx.tcx.map.get_parent(expr.id)),
        ], {
            // report the error around the `vec!` not inside `<std macros>:`
            let span = cx.sess().codemap().source_callsite(arg.span);
            check_vec_macro(cx, &vec_args, span);
        }}
    }
}

fn check_vec_macro(cx: &LateContext, vec_args: &higher::VecArgs, span: Span) {
    let snippet = match *vec_args {
        higher::VecArgs::Repeat(elem, len) => {
            if eval_const_expr_partial(cx.tcx, len, ExprTypeChecked, None).is_ok() {
                format!("&[{}; {}]", snippet(cx, elem.span, "elem"), snippet(cx, len.span, "len")).into()
            } else {
                return;
            }
        }
        higher::VecArgs::Vec(args) => {
            if let Some(last) = args.iter().last() {
                let span = Span {
                    lo: args[0].span.lo,
                    hi: last.span.hi,
                    expn_id: args[0].span.expn_id,
                };

                format!("&[{}]", snippet(cx, span, "..")).into()
            } else {
                "&[]".into()
            }
        }
    };

    span_lint_and_then(cx, USELESS_VEC, span, "useless use of `vec!`", |db| {
        db.span_suggestion(span, "you can use a slice directly", snippet);
    });
}

/// Return the item type of the vector (ie. the `T` in `Vec<T>`).
fn vec_type(ty: ty::Ty) -> ty::Ty {
    if let ty::TyStruct(_, substs) = ty.sty {
        substs.type_at(0)
    } else {
        panic!("The type of `vec!` is a not a struct?");
    }
}
