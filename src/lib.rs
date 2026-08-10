extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields, parse_macro_input};
#[proc_macro_derive(TryFromRow)]
pub fn try_from_row_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as syn::DeriveInput);
	let name = &input.ident;
	// 获取结构体的字段
	let fields = match &input.data {
		Data::Struct(s) => match &s.fields {
			Fields::Named(fields) => &fields.named,
			_ => panic!("#[derive(TryFromRow)] Fields must be named."),
		},
		_ => panic!("#[derive(TryFromRow)] can only be used on structs"),
	};
	// 生成字段赋值语句：let field_name = row.try_get("field_name")?;
	let field_assigns = fields.iter().map(|f| {
		let field_name = f.ident.as_ref().unwrap();
		let field_name_str = field_name.to_string();
		quote! {
			#field_name : row.try_get(#field_name_str)?,
		}
	});
	// 生成 impl TryFrom<&Row> for StructName
	let expanded = quote! {
		impl TryFrom<&Row> for #name {
			type Error = tokio_postgres::Error;
			fn try_from(row: &Row) -> Result<Self, Self::Error> {
				Ok(Self {
					#(#field_assigns)*
				})
			}
		}
	};
	TokenStream::from(expanded)
}
