use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Global)]
pub fn global_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_global_macro(&ast)
}

fn impl_global_macro(ast: &syn::DeriveInput) -> TokenStream {
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