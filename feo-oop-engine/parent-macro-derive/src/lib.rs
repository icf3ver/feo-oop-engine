use proc_macro::TokenStream;
use quote::quote;

/// default visible = true by default
///

#[proc_macro_derive(Parent)]
pub fn parent_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_parent_macro(&ast)
}

fn impl_parent_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Parent for #name {
            fn get_children(&self) -> Vec<Arc<RwLock<dyn GameObject>>> { self.children.clone() }

            fn add_child(&mut self, child: Arc<RwLock<dyn GameObject>>) { self.children.push(child); }
        
            unsafe fn remove_child(&mut self, child: Arc<RwLock<dyn GameObject>>) -> Result<(), ()> {
                let child_read = &*child.read().unwrap();
                let index = self.children.clone().iter().position(|r| &*r.read().unwrap() == child_read);
                match index {
                    Some(i) => {
                        self.children.remove(i);
                        Ok(())
                    },
                    None => Err(())
                }
            }
        
            unsafe fn replace_child(&mut self, old: Arc<RwLock<dyn GameObject>>, new: Arc<RwLock<dyn GameObject>>) -> Result<(), ()> {
                let index = self.children.clone().iter().position(|r| r.read().unwrap().get_id() == old.read().unwrap().get_id());
                match index {
                    Some(i) => {
                        let _ = mem::replace(&mut self.children[i], new);
                        Ok(())
                    },
                    None => Err(())
                }
            }
        }
    };
    gen.into()
}