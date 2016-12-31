#![feature(plugin)]
#![feature(rustc_private)]
#![plugin(clippy)]

#![deny(lint_without_lint_pass)]

#[macro_use] extern crate rustc;

use rustc::lint::{LintPass, LintArray};

declare_lint! { GOOD_LINT, Warn, "good lint" }
declare_lint! { MISSING_LINT, Warn, "missing lint" }
//~^ ERROR: the lint `MISSING_LINT` is not added to any `LintPass`

pub struct Pass;

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array![GOOD_LINT]
    }
}

fn main() {
    let _ = MISSING_LINT;
}


