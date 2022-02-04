use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_named_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Named for #name {
            fn get_name(&self) -> &str {
               self.name.as_str()
            }
        }
    };
    gen.into()
}