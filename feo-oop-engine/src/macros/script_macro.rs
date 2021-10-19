//! Script macros
//! 
//! The script macros are in here because of an attempt to pass a dyn GameObject.


// Generalized
// #[macro_export]
// macro_rules! dyn_async {(
//     $( #[$attr:meta] )*
//     $pub:vis
//     async
//     fn $fname:ident<$lt:lifetime> ( $($args:tt)* ) $(-> $Ret:ty)?
//     {
//         $($body:tt)*
//     }
// ) => (
//     $( #[$attr] )*
//     #[allow(unused_parens)]
//     $pub
//     fn $fname<$lt> ( $($args)* ) -> ::std::pin::Pin<::std::boxed::Box<
//         dyn ::std::future::Future<Output = ($($Ret)?)>
//             + ::std::marker::Send + $lt
//     >>
//     {
//         ::std::boxed::Box::pin(async move { $($body)* })
//     }
// )}

#[macro_export]
macro_rules! start_script {
    (
        $( #[$attr:meta] )*
        $pub:vis
        async
        fn start<$lt:lifetime> ($this:tt : $this_ty:ty, $engine_globals:tt : $engine_globals_ty:ty) -> Swap
        {
            $($body:tt)*
        }
    ) => (
        $( #[$attr] )*
        #[allow(unused_parens)]
        $pub
        fn start<$lt> ($this : $this_ty, $engine_globals : $engine_globals_ty) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<Output = Swap>
                + ::std::marker::Send + $lt
        >>
        {
            ::std::boxed::Box::pin(async move { 
                $($body)*
            })
        }
    )
}

#[macro_export]
macro_rules! frame_script {
    (
        $( #[$attr:meta] )*
        $pub:vis
        async
        fn frame<$lt:lifetime> ($this:tt : $this_ty:ty , $engine_globals:tt : $engine_globals_ty:ty) -> Swap
        {
            $($body:tt)*
        }
    ) => (
        $( #[$attr] )*
        #[allow(unused_parens)]
        $pub
        fn frame<$lt> ($this : $this_ty, $engine_globals : $engine_globals_ty) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<Output = Swap>
                + ::std::marker::Send + $lt
        >>
        {
            ::std::boxed::Box::pin(async move { 
                $($body)*
            })
        }
    )
}

#[macro_export]
macro_rules! event_handler {
    (
        $( #[$attr:meta] )*
        $pub:vis
        async
        fn event_handler<$lt:lifetime> ($this:tt : $this_ty:ty , $engine_globals:tt : $engine_globals_ty:ty, $event:tt : $event_ty:ty) -> Swap
        {
            $($body:tt)*
        }
    ) => (
        $( #[$attr] )*
        #[allow(unused_parens)]
        $pub
        fn event_handler<$lt> ($this : $this_ty, $engine_globals : $engine_globals_ty, $event: $event_ty) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<Output = Swap>
                + ::std::marker::Send + $lt
        >>
        {
            ::std::boxed::Box::pin(async move { 
                $($body)*
            })
        }
    )
}