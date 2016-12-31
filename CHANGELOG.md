# Change Log
All notable changes to this project will be documented in this file.

## 0.0.87 — ??
* New lints: [`builtin_type_shadow`]
* Fix FP in [`zero_prefixed_literal`] and `0b`/`Oo`

## 0.0.86 — 2016-08-28
* Rustup to *rustc 1.13.0-nightly (a23064af5 2016-08-27)*
* New lints: [`missing_docs_in_private_items`], [`zero_prefixed_literal`]

## 0.0.85 — 2016-08-19
* Fix ICE with [`useless_attribute`]
* [`useless_attribute`] ignores [`unused_imports`] on `use` statements

## 0.0.84 — 2016-08-18
* Rustup to *rustc 1.13.0-nightly (aef6971ca 2016-08-17)*

## 0.0.83 — 2016-08-17
* Rustup to *rustc 1.12.0-nightly (1bf5fa326 2016-08-16)*
* New lints: [`print_with_newline`], [`useless_attribute`]

## 0.0.82 — 2016-08-17
* Rustup to *rustc 1.12.0-nightly (197be89f3 2016-08-15)*
* New lint: [`module_inception`]

## 0.0.81 — 2016-08-14
* Rustup to *rustc 1.12.0-nightly (1deb02ea6 2016-08-12)*
* New lints: [`eval_order_dependence`], [`mixed_case_hex_literals`], [`unseparated_literal_suffix`]
* False positive fix in [`too_many_arguments`]
* Addition of functionality to [`needless_borrow`]
* Suggestions for [`clone_on_copy`]
* Bug fix in [`wrong_self_convention`]
* Doc improvements

## 0.0.80 — 2016-07-31
* Rustup to *rustc 1.12.0-nightly (1225e122f 2016-07-30)*
* New lints: [`misrefactored_assign_op`], [`serde_api_misuse`]

## 0.0.79 — 2016-07-10
* Rustup to *rustc 1.12.0-nightly (f93aaf84c 2016-07-09)*
* Major suggestions refactoring

## 0.0.78 — 2016-07-02
* Rustup to *rustc 1.11.0-nightly (01411937f 2016-07-01)*
* New lints: [`wrong_transmute`], [`double_neg`], [`filter_map`]
* For compatibility, `cargo clippy` does not defines the `clippy` feature
  introduced in 0.0.76 anymore
* [`collapsible_if`] now considers `if let`

## 0.0.77 — 2016-06-21
* Rustup to *rustc 1.11.0-nightly (5522e678b 2016-06-20)*
* New lints: [`stutter`] and [`iter_nth`]

## 0.0.76 — 2016-06-10
* Rustup to *rustc 1.11.0-nightly (7d2f75a95 2016-06-09)*
* `cargo clippy` now automatically defines the `clippy` feature
* New lint: [`not_unsafe_ptr_arg_deref`]

## 0.0.75 — 2016-06-08
* Rustup to *rustc 1.11.0-nightly (763f9234b 2016-06-06)*

## 0.0.74 — 2016-06-07
* Fix bug with `cargo-clippy` JSON parsing
* Add the `CLIPPY_DISABLE_WIKI_LINKS` environment variable to deactivate the
  “for further information visit *wiki-link*” message.

## 0.0.73 — 2016-06-05
* Fix false positives in [`useless_let_if_seq`]

## 0.0.72 — 2016-06-04
* Fix false positives in [`useless_let_if_seq`]

## 0.0.71 — 2016-05-31
* Rustup to *rustc 1.11.0-nightly (a967611d8 2016-05-30)*
* New lint: [`useless_let_if_seq`]

## 0.0.70 — 2016-05-28
* Rustup to *rustc 1.10.0-nightly (7bddce693 2016-05-27)*
* [`invalid_regex`] and [`trivial_regex`] can now warn on `RegexSet::new`,
  `RegexBuilder::new` and byte regexes

## 0.0.69 — 2016-05-20
* Rustup to *rustc 1.10.0-nightly (476fe6eef 2016-05-21)*
* [`used_underscore_binding`] has been made `Allow` temporarily

## 0.0.68 — 2016-05-17
* Rustup to *rustc 1.10.0-nightly (cd6a40017 2016-05-16)*
* New lint: [`unnecessary_operation`]

## 0.0.67 — 2016-05-12
* Rustup to *rustc 1.10.0-nightly (22ac88f1a 2016-05-11)*

## 0.0.66 — 2016-05-11
* New `cargo clippy` subcommand
* New lints: [`assign_op_pattern`], [`assign_ops`], [`needless_borrow`]

## 0.0.65 — 2016-05-08
* Rustup to *rustc 1.10.0-nightly (62e2b2fb7 2016-05-06)*
* New lints: [`float_arithmetic`], [`integer_arithmetic`]

## 0.0.64 — 2016-04-26
* Rustup to *rustc 1.10.0-nightly (645dd013a 2016-04-24)*
* New lints: [`temporary_cstring_as_ptr`], [`unsafe_removed_from_name`], and [`mem_forget`]

## 0.0.63 — 2016-04-08
* Rustup to *rustc 1.9.0-nightly (7979dd608 2016-04-07)*

## 0.0.62 — 2016-04-07
* Rustup to *rustc 1.9.0-nightly (bf5da36f1 2016-04-06)*

## 0.0.61 — 2016-04-03
* Rustup to *rustc 1.9.0-nightly (5ab11d72c 2016-04-02)*
* New lint: [`invalid_upcast_comparisons`]

## 0.0.60 — 2016-04-01
* Rustup to *rustc 1.9.0-nightly (e1195c24b 2016-03-31)*

## 0.0.59 — 2016-03-31
* Rustup to *rustc 1.9.0-nightly (30a3849f2 2016-03-30)*
* New lints: [`logic_bug`], [`nonminimal_bool`]
* Fixed: [`match_same_arms`] now ignores arms with guards
* Improved: [`useless_vec`] now warns on `for … in vec![…]`

## 0.0.58 — 2016-03-27
* Rustup to *rustc 1.9.0-nightly (d5a91e695 2016-03-26)*
* New lint: [`doc_markdown`]

## 0.0.57 — 2016-03-27
* Update to *rustc 1.9.0-nightly (a1e29daf1 2016-03-25)*
* Deprecated lints: [`str_to_string`], [`string_to_string`], [`unstable_as_slice`], [`unstable_as_mut_slice`]
* New lint: [`crosspointer_transmute`]

## 0.0.56 — 2016-03-23
* Update to *rustc 1.9.0-nightly (0dcc413e4 2016-03-22)*
* New lints: [`many_single_char_names`] and [`similar_names`]

## 0.0.55 — 2016-03-21
* Update to *rustc 1.9.0-nightly (02310fd31 2016-03-19)*

## 0.0.54 — 2016-03-16
* Update to *rustc 1.9.0-nightly (c66d2380a 2016-03-15)*

## 0.0.53 — 2016-03-15
* Add a [configuration file]

## ~~0.0.52~~

## 0.0.51 — 2016-03-13
* Add `str` to types considered by [`len_zero`]
* New lints: [`indexing_slicing`]

## 0.0.50 — 2016-03-11
* Update to *rustc 1.9.0-nightly (c9629d61c 2016-03-10)*

## 0.0.49 — 2016-03-09
* Update to *rustc 1.9.0-nightly (eabfc160f 2016-03-08)*
* New lints: [`overflow_check_conditional`], [`unused_label`], [`new_without_default`]

## 0.0.48 — 2016-03-07
* Fixed: ICE in [`needless_range_loop`] with globals

## 0.0.47 — 2016-03-07
* Update to *rustc 1.9.0-nightly (998a6720b 2016-03-07)*
* New lint: [`redundant_closure_call`]

[configuration file]: ./rust-clippy#configuration

<!-- begin autogenerated links to wiki -->
[`absurd_extreme_comparisons`]: https://github.com/Manishearth/rust-clippy/wiki#absurd_extreme_comparisons
[`almost_swapped`]: https://github.com/Manishearth/rust-clippy/wiki#almost_swapped
[`approx_constant`]: https://github.com/Manishearth/rust-clippy/wiki#approx_constant
[`assign_op_pattern`]: https://github.com/Manishearth/rust-clippy/wiki#assign_op_pattern
[`assign_ops`]: https://github.com/Manishearth/rust-clippy/wiki#assign_ops
[`bad_bit_mask`]: https://github.com/Manishearth/rust-clippy/wiki#bad_bit_mask
[`blacklisted_name`]: https://github.com/Manishearth/rust-clippy/wiki#blacklisted_name
[`block_in_if_condition_expr`]: https://github.com/Manishearth/rust-clippy/wiki#block_in_if_condition_expr
[`block_in_if_condition_stmt`]: https://github.com/Manishearth/rust-clippy/wiki#block_in_if_condition_stmt
[`bool_comparison`]: https://github.com/Manishearth/rust-clippy/wiki#bool_comparison
[`box_vec`]: https://github.com/Manishearth/rust-clippy/wiki#box_vec
[`boxed_local`]: https://github.com/Manishearth/rust-clippy/wiki#boxed_local
[`builtin_type_shadow`]: https://github.com/Manishearth/rust-clippy/wiki#builtin_type_shadow
[`cast_possible_truncation`]: https://github.com/Manishearth/rust-clippy/wiki#cast_possible_truncation
[`cast_possible_wrap`]: https://github.com/Manishearth/rust-clippy/wiki#cast_possible_wrap
[`cast_precision_loss`]: https://github.com/Manishearth/rust-clippy/wiki#cast_precision_loss
[`cast_sign_loss`]: https://github.com/Manishearth/rust-clippy/wiki#cast_sign_loss
[`char_lit_as_u8`]: https://github.com/Manishearth/rust-clippy/wiki#char_lit_as_u8
[`chars_next_cmp`]: https://github.com/Manishearth/rust-clippy/wiki#chars_next_cmp
[`clone_double_ref`]: https://github.com/Manishearth/rust-clippy/wiki#clone_double_ref
[`clone_on_copy`]: https://github.com/Manishearth/rust-clippy/wiki#clone_on_copy
[`cmp_nan`]: https://github.com/Manishearth/rust-clippy/wiki#cmp_nan
[`cmp_null`]: https://github.com/Manishearth/rust-clippy/wiki#cmp_null
[`cmp_owned`]: https://github.com/Manishearth/rust-clippy/wiki#cmp_owned
[`collapsible_if`]: https://github.com/Manishearth/rust-clippy/wiki#collapsible_if
[`crosspointer_transmute`]: https://github.com/Manishearth/rust-clippy/wiki#crosspointer_transmute
[`cyclomatic_complexity`]: https://github.com/Manishearth/rust-clippy/wiki#cyclomatic_complexity
[`deprecated_semver`]: https://github.com/Manishearth/rust-clippy/wiki#deprecated_semver
[`derive_hash_xor_eq`]: https://github.com/Manishearth/rust-clippy/wiki#derive_hash_xor_eq
[`doc_markdown`]: https://github.com/Manishearth/rust-clippy/wiki#doc_markdown
[`double_neg`]: https://github.com/Manishearth/rust-clippy/wiki#double_neg
[`drop_ref`]: https://github.com/Manishearth/rust-clippy/wiki#drop_ref
[`duplicate_underscore_argument`]: https://github.com/Manishearth/rust-clippy/wiki#duplicate_underscore_argument
[`empty_loop`]: https://github.com/Manishearth/rust-clippy/wiki#empty_loop
[`enum_clike_unportable_variant`]: https://github.com/Manishearth/rust-clippy/wiki#enum_clike_unportable_variant
[`enum_glob_use`]: https://github.com/Manishearth/rust-clippy/wiki#enum_glob_use
[`enum_variant_names`]: https://github.com/Manishearth/rust-clippy/wiki#enum_variant_names
[`eq_op`]: https://github.com/Manishearth/rust-clippy/wiki#eq_op
[`eval_order_dependence`]: https://github.com/Manishearth/rust-clippy/wiki#eval_order_dependence
[`expl_impl_clone_on_copy`]: https://github.com/Manishearth/rust-clippy/wiki#expl_impl_clone_on_copy
[`explicit_counter_loop`]: https://github.com/Manishearth/rust-clippy/wiki#explicit_counter_loop
[`explicit_iter_loop`]: https://github.com/Manishearth/rust-clippy/wiki#explicit_iter_loop
[`extend_from_slice`]: https://github.com/Manishearth/rust-clippy/wiki#extend_from_slice
[`filter_map`]: https://github.com/Manishearth/rust-clippy/wiki#filter_map
[`filter_next`]: https://github.com/Manishearth/rust-clippy/wiki#filter_next
[`float_arithmetic`]: https://github.com/Manishearth/rust-clippy/wiki#float_arithmetic
[`float_cmp`]: https://github.com/Manishearth/rust-clippy/wiki#float_cmp
[`for_kv_map`]: https://github.com/Manishearth/rust-clippy/wiki#for_kv_map
[`for_loop_over_option`]: https://github.com/Manishearth/rust-clippy/wiki#for_loop_over_option
[`for_loop_over_result`]: https://github.com/Manishearth/rust-clippy/wiki#for_loop_over_result
[`identity_op`]: https://github.com/Manishearth/rust-clippy/wiki#identity_op
[`if_not_else`]: https://github.com/Manishearth/rust-clippy/wiki#if_not_else
[`if_same_then_else`]: https://github.com/Manishearth/rust-clippy/wiki#if_same_then_else
[`ifs_same_cond`]: https://github.com/Manishearth/rust-clippy/wiki#ifs_same_cond
[`indexing_slicing`]: https://github.com/Manishearth/rust-clippy/wiki#indexing_slicing
[`ineffective_bit_mask`]: https://github.com/Manishearth/rust-clippy/wiki#ineffective_bit_mask
[`inline_always`]: https://github.com/Manishearth/rust-clippy/wiki#inline_always
[`integer_arithmetic`]: https://github.com/Manishearth/rust-clippy/wiki#integer_arithmetic
[`invalid_regex`]: https://github.com/Manishearth/rust-clippy/wiki#invalid_regex
[`invalid_upcast_comparisons`]: https://github.com/Manishearth/rust-clippy/wiki#invalid_upcast_comparisons
[`items_after_statements`]: https://github.com/Manishearth/rust-clippy/wiki#items_after_statements
[`iter_next_loop`]: https://github.com/Manishearth/rust-clippy/wiki#iter_next_loop
[`iter_nth`]: https://github.com/Manishearth/rust-clippy/wiki#iter_nth
[`len_without_is_empty`]: https://github.com/Manishearth/rust-clippy/wiki#len_without_is_empty
[`len_zero`]: https://github.com/Manishearth/rust-clippy/wiki#len_zero
[`let_and_return`]: https://github.com/Manishearth/rust-clippy/wiki#let_and_return
[`let_unit_value`]: https://github.com/Manishearth/rust-clippy/wiki#let_unit_value
[`linkedlist`]: https://github.com/Manishearth/rust-clippy/wiki#linkedlist
[`logic_bug`]: https://github.com/Manishearth/rust-clippy/wiki#logic_bug
[`manual_swap`]: https://github.com/Manishearth/rust-clippy/wiki#manual_swap
[`many_single_char_names`]: https://github.com/Manishearth/rust-clippy/wiki#many_single_char_names
[`map_clone`]: https://github.com/Manishearth/rust-clippy/wiki#map_clone
[`map_entry`]: https://github.com/Manishearth/rust-clippy/wiki#map_entry
[`match_bool`]: https://github.com/Manishearth/rust-clippy/wiki#match_bool
[`match_overlapping_arm`]: https://github.com/Manishearth/rust-clippy/wiki#match_overlapping_arm
[`match_ref_pats`]: https://github.com/Manishearth/rust-clippy/wiki#match_ref_pats
[`match_same_arms`]: https://github.com/Manishearth/rust-clippy/wiki#match_same_arms
[`mem_forget`]: https://github.com/Manishearth/rust-clippy/wiki#mem_forget
[`min_max`]: https://github.com/Manishearth/rust-clippy/wiki#min_max
[`misrefactored_assign_op`]: https://github.com/Manishearth/rust-clippy/wiki#misrefactored_assign_op
[`missing_docs_in_private_items`]: https://github.com/Manishearth/rust-clippy/wiki#missing_docs_in_private_items
[`mixed_case_hex_literals`]: https://github.com/Manishearth/rust-clippy/wiki#mixed_case_hex_literals
[`module_inception`]: https://github.com/Manishearth/rust-clippy/wiki#module_inception
[`modulo_one`]: https://github.com/Manishearth/rust-clippy/wiki#modulo_one
[`mut_mut`]: https://github.com/Manishearth/rust-clippy/wiki#mut_mut
[`mutex_atomic`]: https://github.com/Manishearth/rust-clippy/wiki#mutex_atomic
[`mutex_integer`]: https://github.com/Manishearth/rust-clippy/wiki#mutex_integer
[`needless_bool`]: https://github.com/Manishearth/rust-clippy/wiki#needless_bool
[`needless_borrow`]: https://github.com/Manishearth/rust-clippy/wiki#needless_borrow
[`needless_continue`]: https://github.com/Manishearth/rust-clippy/wiki#needless_continue
[`needless_lifetimes`]: https://github.com/Manishearth/rust-clippy/wiki#needless_lifetimes
[`needless_range_loop`]: https://github.com/Manishearth/rust-clippy/wiki#needless_range_loop
[`needless_return`]: https://github.com/Manishearth/rust-clippy/wiki#needless_return
[`needless_update`]: https://github.com/Manishearth/rust-clippy/wiki#needless_update
[`neg_multiply`]: https://github.com/Manishearth/rust-clippy/wiki#neg_multiply
[`new_ret_no_self`]: https://github.com/Manishearth/rust-clippy/wiki#new_ret_no_self
[`new_without_default`]: https://github.com/Manishearth/rust-clippy/wiki#new_without_default
[`new_without_default_derive`]: https://github.com/Manishearth/rust-clippy/wiki#new_without_default_derive
[`no_effect`]: https://github.com/Manishearth/rust-clippy/wiki#no_effect
[`non_ascii_literal`]: https://github.com/Manishearth/rust-clippy/wiki#non_ascii_literal
[`nonminimal_bool`]: https://github.com/Manishearth/rust-clippy/wiki#nonminimal_bool
[`nonsensical_open_options`]: https://github.com/Manishearth/rust-clippy/wiki#nonsensical_open_options
[`not_unsafe_ptr_arg_deref`]: https://github.com/Manishearth/rust-clippy/wiki#not_unsafe_ptr_arg_deref
[`ok_expect`]: https://github.com/Manishearth/rust-clippy/wiki#ok_expect
[`option_map_unwrap_or`]: https://github.com/Manishearth/rust-clippy/wiki#option_map_unwrap_or
[`option_map_unwrap_or_else`]: https://github.com/Manishearth/rust-clippy/wiki#option_map_unwrap_or_else
[`option_unwrap_used`]: https://github.com/Manishearth/rust-clippy/wiki#option_unwrap_used
[`or_fun_call`]: https://github.com/Manishearth/rust-clippy/wiki#or_fun_call
[`out_of_bounds_indexing`]: https://github.com/Manishearth/rust-clippy/wiki#out_of_bounds_indexing
[`overflow_check_conditional`]: https://github.com/Manishearth/rust-clippy/wiki#overflow_check_conditional
[`panic_params`]: https://github.com/Manishearth/rust-clippy/wiki#panic_params
[`precedence`]: https://github.com/Manishearth/rust-clippy/wiki#precedence
[`print_stdout`]: https://github.com/Manishearth/rust-clippy/wiki#print_stdout
[`print_with_newline`]: https://github.com/Manishearth/rust-clippy/wiki#print_with_newline
[`ptr_arg`]: https://github.com/Manishearth/rust-clippy/wiki#ptr_arg
[`range_step_by_zero`]: https://github.com/Manishearth/rust-clippy/wiki#range_step_by_zero
[`range_zip_with_len`]: https://github.com/Manishearth/rust-clippy/wiki#range_zip_with_len
[`redundant_closure`]: https://github.com/Manishearth/rust-clippy/wiki#redundant_closure
[`redundant_closure_call`]: https://github.com/Manishearth/rust-clippy/wiki#redundant_closure_call
[`redundant_pattern`]: https://github.com/Manishearth/rust-clippy/wiki#redundant_pattern
[`regex_macro`]: https://github.com/Manishearth/rust-clippy/wiki#regex_macro
[`result_unwrap_used`]: https://github.com/Manishearth/rust-clippy/wiki#result_unwrap_used
[`reverse_range_loop`]: https://github.com/Manishearth/rust-clippy/wiki#reverse_range_loop
[`search_is_some`]: https://github.com/Manishearth/rust-clippy/wiki#search_is_some
[`serde_api_misuse`]: https://github.com/Manishearth/rust-clippy/wiki#serde_api_misuse
[`shadow_reuse`]: https://github.com/Manishearth/rust-clippy/wiki#shadow_reuse
[`shadow_same`]: https://github.com/Manishearth/rust-clippy/wiki#shadow_same
[`shadow_unrelated`]: https://github.com/Manishearth/rust-clippy/wiki#shadow_unrelated
[`should_implement_trait`]: https://github.com/Manishearth/rust-clippy/wiki#should_implement_trait
[`similar_names`]: https://github.com/Manishearth/rust-clippy/wiki#similar_names
[`single_char_pattern`]: https://github.com/Manishearth/rust-clippy/wiki#single_char_pattern
[`single_match`]: https://github.com/Manishearth/rust-clippy/wiki#single_match
[`single_match_else`]: https://github.com/Manishearth/rust-clippy/wiki#single_match_else
[`str_to_string`]: https://github.com/Manishearth/rust-clippy/wiki#str_to_string
[`string_add`]: https://github.com/Manishearth/rust-clippy/wiki#string_add
[`string_add_assign`]: https://github.com/Manishearth/rust-clippy/wiki#string_add_assign
[`string_lit_as_bytes`]: https://github.com/Manishearth/rust-clippy/wiki#string_lit_as_bytes
[`string_to_string`]: https://github.com/Manishearth/rust-clippy/wiki#string_to_string
[`stutter`]: https://github.com/Manishearth/rust-clippy/wiki#stutter
[`suspicious_assignment_formatting`]: https://github.com/Manishearth/rust-clippy/wiki#suspicious_assignment_formatting
[`suspicious_else_formatting`]: https://github.com/Manishearth/rust-clippy/wiki#suspicious_else_formatting
[`temporary_assignment`]: https://github.com/Manishearth/rust-clippy/wiki#temporary_assignment
[`temporary_cstring_as_ptr`]: https://github.com/Manishearth/rust-clippy/wiki#temporary_cstring_as_ptr
[`too_many_arguments`]: https://github.com/Manishearth/rust-clippy/wiki#too_many_arguments
[`toplevel_ref_arg`]: https://github.com/Manishearth/rust-clippy/wiki#toplevel_ref_arg
[`transmute_ptr_to_ref`]: https://github.com/Manishearth/rust-clippy/wiki#transmute_ptr_to_ref
[`trivial_regex`]: https://github.com/Manishearth/rust-clippy/wiki#trivial_regex
[`type_complexity`]: https://github.com/Manishearth/rust-clippy/wiki#type_complexity
[`unicode_not_nfc`]: https://github.com/Manishearth/rust-clippy/wiki#unicode_not_nfc
[`unit_cmp`]: https://github.com/Manishearth/rust-clippy/wiki#unit_cmp
[`unnecessary_mut_passed`]: https://github.com/Manishearth/rust-clippy/wiki#unnecessary_mut_passed
[`unnecessary_operation`]: https://github.com/Manishearth/rust-clippy/wiki#unnecessary_operation
[`unneeded_field_pattern`]: https://github.com/Manishearth/rust-clippy/wiki#unneeded_field_pattern
[`unsafe_removed_from_name`]: https://github.com/Manishearth/rust-clippy/wiki#unsafe_removed_from_name
[`unseparated_literal_suffix`]: https://github.com/Manishearth/rust-clippy/wiki#unseparated_literal_suffix
[`unstable_as_mut_slice`]: https://github.com/Manishearth/rust-clippy/wiki#unstable_as_mut_slice
[`unstable_as_slice`]: https://github.com/Manishearth/rust-clippy/wiki#unstable_as_slice
[`unused_collect`]: https://github.com/Manishearth/rust-clippy/wiki#unused_collect
[`unused_label`]: https://github.com/Manishearth/rust-clippy/wiki#unused_label
[`unused_lifetimes`]: https://github.com/Manishearth/rust-clippy/wiki#unused_lifetimes
[`use_debug`]: https://github.com/Manishearth/rust-clippy/wiki#use_debug
[`used_underscore_binding`]: https://github.com/Manishearth/rust-clippy/wiki#used_underscore_binding
[`useless_attribute`]: https://github.com/Manishearth/rust-clippy/wiki#useless_attribute
[`useless_format`]: https://github.com/Manishearth/rust-clippy/wiki#useless_format
[`useless_let_if_seq`]: https://github.com/Manishearth/rust-clippy/wiki#useless_let_if_seq
[`useless_transmute`]: https://github.com/Manishearth/rust-clippy/wiki#useless_transmute
[`useless_vec`]: https://github.com/Manishearth/rust-clippy/wiki#useless_vec
[`while_let_loop`]: https://github.com/Manishearth/rust-clippy/wiki#while_let_loop
[`while_let_on_iterator`]: https://github.com/Manishearth/rust-clippy/wiki#while_let_on_iterator
[`wrong_pub_self_convention`]: https://github.com/Manishearth/rust-clippy/wiki#wrong_pub_self_convention
[`wrong_self_convention`]: https://github.com/Manishearth/rust-clippy/wiki#wrong_self_convention
[`wrong_transmute`]: https://github.com/Manishearth/rust-clippy/wiki#wrong_transmute
[`zero_divided_by_zero`]: https://github.com/Manishearth/rust-clippy/wiki#zero_divided_by_zero
[`zero_prefixed_literal`]: https://github.com/Manishearth/rust-clippy/wiki#zero_prefixed_literal
[`zero_width_space`]: https://github.com/Manishearth/rust-clippy/wiki#zero_width_space
<!-- end autogenerated links to wiki -->
