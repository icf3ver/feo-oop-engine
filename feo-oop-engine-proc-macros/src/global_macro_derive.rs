use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_global_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Global for #name {
            fn as_any(&self) -> &dyn std::any::Any{
               self
            }
        }
    };
    gen.into()
}