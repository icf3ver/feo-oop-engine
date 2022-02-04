//! Procedural macros for [feo-oop-engine](https://github.com/littletitan/feo-oop-engine)
//! 
#[cfg(feature = "child")] mod child_macro_derive;
#[cfg(feature = "drawable")] mod drawable_macro_derive;
#[cfg(feature = "gameobject")] mod gameobject_macro_derive;
#[cfg(feature = "global")] mod global_macro_derive;
#[cfg(feature = "named")] mod named_macro_derive;
#[cfg(feature = "parent")] mod parent_macro_derive;
#[cfg(feature = "scriptable")] mod scriptable_macro_derive;

use proc_macro::TokenStream;

/// <span class="stab portability" title="This is supported on crate feature `child` only"><code>
/// feature = child
/// </code></span>
#[cfg(feature = "child")]
#[proc_macro_derive(Child)]
pub fn child_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    child_macro_derive::impl_child_macro(&ast)
}

/// <span class="stab portability" title="This is supported on crate feature `drawable` only"><code>
/// feature = drawable
/// </code></span>
#[cfg(feature = "drawable")]
#[proc_macro_derive(Drawable, attributes(light))]
pub fn drawable_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    drawable_macro_derive::impl_drawable_macro(&ast)
}

/// <span class="stab portability" title="This is supported on crate feature `gameobject` only"><code>
/// feature = gameobject
/// </code></span>
#[cfg(feature = "gameobject")]
#[proc_macro_derive(GameObject, attributes(camera))]
pub fn gameobject_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    gameobject_macro_derive::impl_gameobject_macro(&ast)
}

/// <span class="stab portability" title="This is supported on crate feature `global` or `default` only"><code>
/// feature = global or default
/// </code></span>
#[cfg(feature = "global")]
#[proc_macro_derive(Global)]
pub fn global_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    global_macro_derive::impl_global_macro(&ast)
}

/// <span class="stab portability" title="This is supported on crate feature `named` only"><code>
/// feature = named
/// </code></span>
#[cfg(feature = "named")]
#[proc_macro_derive(Named)]
pub fn named_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    named_macro_derive::impl_named_macro(&ast)
}


/// <span class="stab portability" title="This is supported on crate feature `parent` only"><code>
/// feature = parent
/// </code></span>
#[cfg(feature = "parent")]
#[proc_macro_derive(Parent)]
pub fn parent_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    parent_macro_derive::impl_parent_macro(&ast)
}

/// <span class="stab portability" title="This is supported on crate feature `scriptable` only"><code>
/// feature = scriptable
/// </code></span>
#[cfg(feature = "scriptable")]
#[proc_macro_derive(Scriptable)]
pub fn scriptable_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    scriptable_macro_derive::impl_scriptable_macro(&ast)
}