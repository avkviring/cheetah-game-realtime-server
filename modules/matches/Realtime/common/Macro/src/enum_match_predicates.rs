use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Ident, Span, TokenTree};
use quote::{quote, ToTokens};
use syn::{Data, DataEnum};

///
/// Генерация метода для поиска конкретного элемента enum в коллекции
///
pub fn do_enum_match_predicates(input: TokenStream) -> TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();
	let name = &ast.ident;
	let enum_data: DataEnum = match ast.data {
		Data::Enum(d) => d,
		_ => panic!("Only for enum"),
	};
	let variants = enum_data.variants.iter();
	let variant_structs = variants.map(|v| {
		let var_id = &v.ident;
		let name_field = Ident::new(&format!("predicate_{}", var_id).to_snake_case(), Span::call_site());
		let fields = v.fields.clone().into_token_stream();
		if fields.is_empty() {
			quote! {}
		} else {
			let mut f = fields.into_iter().next().unwrap();
			let f = match f {
				TokenTree::Group(ref mut g) => {
					let ident = g.stream().into_iter().next().unwrap();
					Group::new(Delimiter::None, ident.to_token_stream())
				}
				_ => {
					panic!();
				}
			};

			quote! {
				impl #name {
					pub fn #name_field(header: &Self)->Option<&#f> {
					match header {
							Self::#var_id(value) => Some(&value),
							_ => None
						}
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
