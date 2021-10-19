use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Named)]
pub fn named_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_named_macro(&ast)
}

fn impl_named_macro(ast: &syn::DeriveInput) -> TokenStream {
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