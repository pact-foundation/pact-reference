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
        pub extern fn $name($($arg: $arg_ty),*) -> $ret {
            use $crate::log::TARGET;
            use $crate::error::catch_panic;

            ::log::trace!(target: TARGET, "{}::{}", module_path!(), stringify!($name));

            $(
                ::log::trace!(target: TARGET, "@param {} = {:?}", stringify!($arg), $arg);
            )*

            let output = catch_panic(|| Ok($body)).unwrap_or($fail);

            ::log::trace!(target: TARGET, "@return {:?}", output);

            output
        }
    };

    ($(#[$doc:meta])* fn $name:ident($($arg:ident: $arg_ty:ty),*) $body:block ) => {
        $crate::ffi_fn!($(#[$doc])* fn $name($($arg: $arg_ty),*) -> () $body {});
    };
}
