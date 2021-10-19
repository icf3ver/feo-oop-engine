use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Scriptable)]
pub fn scriptable_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_scriptable_macro(&ast)
}

fn impl_scriptable_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    // todo: don't assume var names

    let gen = quote! {
        impl Scriptable for #name {
            fn spawn_script_core(&mut self, this: Arc<RwLock<dyn GameObject>>, spawner: Spawner){
                let this_ptr = Arc::into_raw(this).cast::<RwLock<Self>>();
                let this = unsafe {Arc::from_raw(this_ptr)};
        
                let engine_globals = spawner.engine_globals.clone(); // put somewhere else
        
                if let Some(_) = self.script.clone(){
                    if self.script.clone().unwrap().has_started {
                        let local_self = this.clone();
                        let local_script = self.script.clone().unwrap();
                        spawner.spawn(async move {
                            (*local_script.frame).call((local_self, engine_globals)).await
                        });
                    } else {
                        let s = self.script.as_mut();
                        s.unwrap().has_started = true;
                        let local_self = this.clone();
                        let local_script = self.script.clone().unwrap();
                        spawner.spawn(async move {
                            (*local_script.start).call((local_self, engine_globals)).await
                        });
                    }
                }
                
                self.children.clone().into_iter().for_each(|game_object| {
                    let game_object_template = game_object.clone();
                    game_object_template.write().unwrap().spawn_script_core(game_object, spawner.clone());
                });
            }
        
            fn spawn_script_handler<'a>(&mut self, this: Arc<RwLock<dyn GameObject>>, spawner: Spawner, event: Event<'static, UserEvent<Arc<dyn Any + Send + Sync>>>){ // DID removed box continue
                let this_ptr = Arc::into_raw(this).cast::<RwLock<Self>>();
                let this = unsafe {Arc::from_raw(this_ptr)};
        
                let engine_globals = spawner.engine_globals.clone(); // put somewhere else
        
                if let Some(local_script) = self.script.clone() {
                    if self.script.clone().unwrap().has_started {
                        let local_self = this.clone();
                        let local_event = event.clone();
                        if let Some(event_handler) = local_script.event_handler{
                            spawner.spawn(async move {
                                (*event_handler).call((local_self, engine_globals, local_event)).await
                            });
                        }
                    }
                }
                
                self.children.clone().into_iter().for_each(|game_object| {
                    game_object.clone().write().unwrap().spawn_script_handler(game_object, spawner.clone(), event.clone());
                });
            }
        
            fn get_globals(&self) -> Result<Box<dyn Global>, &'static str> {
                match self.script.clone() {
                    Some(s) => {
                        match s.globals {
                            Some(g) => Ok(g),
                            None => Err("No globals found in script.")
                        }
                    },
                    None => Err("No script found."),
                }
            }
        
            fn set_globals(&mut self, globals: Box<dyn Global>) -> Result<(), &'static str>{
                match self.script.as_mut() {
                    Some(s) => Ok(s.globals = Some(globals)),
                    None => Err("No script found.")
                }
            }
        }
    };
    gen.into()
}