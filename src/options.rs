use syntax::ast::*;
use rustc::lint::{Context, LintArray, LintPass};
use rustc::middle::ty;
use syntax::codemap::Spanned;
use misc::walk_ty;
use utils::match_def_path;

declare_lint! { 
	pub OPTION_AND_THEN_SOME, Warn,
	"Warn on uses of '_.and_then(..)' where the contained closure is \
	 guaranteed to return Some(_)"
}

#[derive(Copy,Clone)]
pub struct Options;

impl LintPass for Options {
	fn get_lints(&self) -> LintArray {
		lint_array!(OPTION_AND_THEN_SOME)
	}
	
	fn check_expr(&mut self, cx: &Context, expr: &Expr) {
		if let ExprMethodCall(ref ident, _, ref args) = expr.node {
			if ident.node.as_str() == "and_then" && args.len() == 2 &&
					is_option(cx, &args[0]) && 
					is_expr_some(cx, &args[1]) {
				cx.span_lint(OPTION_AND_THEN_SOME, expr.span,
					"Consider using _.map(_) instead of _.and_then(_) \
					 if the argument only ever returns Some(_)")
			}
		}
	}
}

fn is_option(cx: &Context, expr: &Expr) -> bool {
	let ty = &walk_ty(&ty::expr_ty(cx.tcx, expr));
	if let ty::ty_enum(def_id, _) = ty.sty {
		match_def_path(cx, def_id, &["core", "option", "Option"])
	} else { false }
}

fn match_segments(path: &Path, segments: &[&str]) -> bool {
	path.segments.iter().rev().zip(segments.iter().rev()).all(
		|(a,b)| a.identifier.as_str() == *b)
}

fn is_block_some(cx: &Context, block: &Block) -> bool {
	block.stmts.iter().all(|stmt| is_statement_some(cx, stmt)) &&
		block.expr.as_ref().map_or(true, 
			|expr| is_expr_some(cx, &*expr))
}

fn is_statement_some(cx: &Context, stmt: &Stmt) -> bool {
	match stmt.node {
		StmtDecl(ref decl, _) => {
			if let DeclLocal(ref local) = decl.node {
				local.init.as_ref().map_or(true, 
					|expr| is_expr_not_ret_none(cx, &*expr))
			} else { true }
		},
		StmtExpr(ref expr, _) | StmtSemi(ref expr, _) => 
			is_expr_not_ret_none(cx, &*expr),
		StmtMac(_, _) => true // abort when matching on macros
	}
}

fn is_expr_not_ret_none(cx: &Context, expr: &Expr) -> bool {
	if let ExprRet(ref ret) = expr.node {
		ret.as_ref().map_or(false, |e| is_expr_some(cx, &*e))
	} else { true }
}

fn is_expr_some(cx: &Context, expr: &Expr) -> bool {
	match expr.node {
		ExprPath(_, ref path) =>
			match_segments(path, &["core", "option", "Option", "Some"]),
		ExprCall(ref path, ref args) => is_expr_some(cx, path) && 
			args.iter().by_ref().all(|e| is_expr_not_ret_none(cx, &*e)),
		ExprBlock(ref block) | ExprClosure(_, _, ref block) => 
			is_block_some(cx, block),
		ExprIf(_, ref block, ref else_expr) =>
			is_block_some(cx, block) && else_expr.as_ref().map_or(false, 
				|e| is_expr_some(cx, &*e)),
		ExprRet(ref ret) => 
			ret.as_ref().map_or(false, |e| is_expr_some(cx, &*e)),
		_ => {
			cx.sess().note(&format!("is_expr_some: no match: {:?}",
				expr));
			false
		}
	}
}
