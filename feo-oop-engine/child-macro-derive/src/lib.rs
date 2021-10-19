use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Child)]
pub fn child_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_child_macro(&ast)
}

fn impl_child_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Child for #name {
            fn get_parent(&self) -> ParentWrapper { self.parent.clone() }
            unsafe fn set_parent(&mut self, parent: ParentWrapper) { self.parent = parent; }
        }
    };
    gen.into()
}