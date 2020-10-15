use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{Data, DataEnum};

///
/// Генерация метода для поиска конкретного элемента enum в коллекции
///
#[proc_macro_derive(EnumMatchPredicates)]
pub fn enum_match_predicates(input: TokenStream) -> TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();
	let name = &ast.ident;
	let enum_data: DataEnum = match ast.data {
		Data::Enum(d) => d,
		_ => panic!("Only for enum"),
	};
	let variants = enum_data.variants.iter();
	let variant_structs = variants.map(|v| {
		let var_id = &v.ident;
		let name_field = Ident::new(&format!("predicate_{}", var_id), Span::call_site());
		let fields = v.fields.clone().into_token_stream();
		quote! {
			impl #name {
				pub fn #name_field(header: &Self)->Option<&#fields> {
				match header {
						Self::#var_id(#fields) => Option::Some(&#fields),
						_ => Option::None
					}
				}
			}
		}
	});
	
	
	let gen = quote! {
		#(#variant_structs)*
	};
	gen.into()
}
