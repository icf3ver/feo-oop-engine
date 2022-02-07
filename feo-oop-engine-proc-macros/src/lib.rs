//! [![github]](https://github.com/littleTitan/feo-oop-engine-proc-macros)&ensp;[![crates-io]](https://crates.io/crates/feo-oop-engine-proc-macros)&ensp;[![docs-rs]](crate)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! Procedural macros for [feo-oop-engine](https://github.com/littletitan/feo-oop-engine)
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