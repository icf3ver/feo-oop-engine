use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(GameObject, attributes(camera))]
pub fn gameobject_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_gameobject_macro(&ast)
}

fn impl_gameobject_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let attributes = &ast.attrs;
    
    let mut dyn_camera_cast = quote!(Err(()));
    let mut dyn_light_cast = quote!(Err(()));
    attributes.iter().for_each(|attr| {
        if attr.path.is_ident("camera") { // TODO: make part of drawable
            dyn_camera_cast = quote!(
                let this_ptr = Arc::into_raw(this).cast::<RwLock<Self>>();
                let this = unsafe {Arc::from_raw(this_ptr)};
                Ok(this as Arc<RwLock<dyn Camera>>)
            );
        } else if attr.path.is_ident("light") { // already part of drawable
            dyn_light_cast = quote!(
                let this_ptr = Arc::into_raw(this).cast::<RwLock<Self>>();
                let this = unsafe {Arc::from_raw(this_ptr)};
                Ok(this as Arc<RwLock<dyn Light>>)
            );
        }
    });

    let gen = quote! {
        impl GameObject for #name {
            fn as_any(&self) -> &dyn Any { self }
            fn cast_camera_arc_rwlock(&self, this: Arc<RwLock<dyn GameObject>>) -> Result<Arc<RwLock<dyn Camera>>, ()> { #dyn_camera_cast }
            fn cast_light_arc_rwlock(&self, this: Arc<RwLock<dyn GameObject>>) -> Result<Arc<RwLock<dyn Light>>, ()> { #dyn_light_cast  }
        
            fn get_id(&self) -> ID { self.id.clone() } // TOFIX
        
            fn get_subspace(&self) -> Space{
                match self.parent.clone() {
                    ParentWrapper::GameObject(game_object) =>
                        self.subspace.join(game_object.read().unwrap().get_subspace()),
                    ParentWrapper::Scene(scene) =>
                        self.subspace.join(scene.read().unwrap().worldspace)
                }
            }
            
            fn get_inversed_subspace(&self) -> Space{
                match self.parent.clone() {
                    ParentWrapper::GameObject(game_object) =>
                        self.subspace.join_reverse(game_object.read().unwrap().get_inversed_subspace()),
                    ParentWrapper::Scene(scene) =>
                        self.subspace.join_reverse(scene.read().unwrap().worldspace.join_reverse(Space::identity()))
                }
            }
        }
    };
    gen.into()
}