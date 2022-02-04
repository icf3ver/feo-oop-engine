use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_child_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Child for #name {
            fn get_parent(&self) -> ParentWrapper { self.parent.clone() }
            unsafe fn set_parent(&mut self, parent: ParentWrapper) { self.parent = parent; }
        }
    };
    gen.into()
}