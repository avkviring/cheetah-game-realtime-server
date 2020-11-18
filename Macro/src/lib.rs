use proc_macro::TokenStream;

use crate::enum_match_predicates::do_enum_match_predicates;

mod enum_match_predicates;

#[proc_macro_derive(EnumMatchPredicates)]
pub fn enum_match_predicates(input: TokenStream) -> TokenStream {
	do_enum_match_predicates(input)
}

