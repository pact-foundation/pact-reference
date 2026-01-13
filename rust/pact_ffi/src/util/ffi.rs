//! Provides a convenience macro for wrapping FFI code.

// All of this module is a macro and should not appear in the C header file
// or documentation.

#[doc(hidden)]
#[macro_export]
macro_rules! ffi_fn {
    ($(#[$doc:meta])* fn $name:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty $body:block $fail:block ) => {
        $(#[$doc])*
        #[no_mangle]
        #[allow(clippy::or_fun_call)]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub extern "C" fn $name($($arg: $arg_ty),*) -> $ret {
            use $crate::error::catch_panic;

            ::tracing::debug!("{}::{} FFI function invoked", module_path!(), stringify!($name));

            $(
                ::tracing::trace!("@param {} = {:?}", stringify!($arg), $arg);
            )*

            let output = catch_panic(|| Ok($body)).unwrap_or($fail);

            ::tracing::trace!(output = ?output, "{} FFI function completed", stringify!($name));

            output
        }
    };

    ($(#[$doc:meta])* fn $name:ident($($arg:ident: $arg_ty:ty),*) $body:block ) => {
        $crate::ffi_fn!($(#[$doc])* fn $name($($arg: $arg_ty),*) -> () $body {});
    };

    // Support async functions as well, by wrapping them in a block_on call
    ($(#[$doc:meta])* async fn $name:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty $body:block $fail:block ) => {
        $(#[$doc])*
        #[no_mangle]
        #[allow(clippy::or_fun_call)]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub extern "C" fn $name($($arg: $arg_ty),*) -> $ret {
            use $crate::error::catch_panic;

            ::tracing::debug!("{}::{} FFI function invoked", module_path!(), stringify!($name));

            $(
                ::tracing::trace!("@param {} = {:?}", stringify!($arg), $arg);
            )*

            let output = catch_panic(|| ::futures::executor::block_on(async { Ok($body) })).unwrap_or($fail);

            ::tracing::trace!(output = ?output, "{} FFI function completed", stringify!($name));

            output
        }
    };

    ($(#[$doc:meta])* async fn $name:ident($($arg:ident: $arg_ty:ty),*) $body:block ) => {
        $crate::ffi_fn!($(#[$doc])* async fn $name($($arg: $arg_ty),*) -> () $body {});
    };
}
