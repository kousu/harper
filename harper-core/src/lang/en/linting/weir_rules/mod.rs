//! English's built-in Weir rules: every `.weir` file in this directory.
//!
//! A directory here is exposed as a single public rule; see [`crate::weir::weir_rule_list`].

use crate::weir::weir_rule_list;

include!(concat!(env!("OUT_DIR"), "/weir_rules_en.rs"));
