
# Architecture

The `pact_matching_ffi` crate is built with one goal and a couple constraints.
The goal is that it expose all of `pact_matching` for use by other languages.

The constraints are:

1. It should be non-invasive (meaning it doesn't require changes to
   `pact_matching` to, for example, use FFI-friendly types which could be
   exposed directly to C).
2. It should introduce limited performance overhead.
3. It should be easy to call from C.
4. It should be easy for other languages to wrap.
5. It should provide strong support for error handling and logging.
6. It should preserve and document any safety guarantees or requirements. 

The design of the crate is done with these goals in mind. Before getting into
how the wrapping of `pact_matching` is performed, it's worthwhile to describe
the mechanisms around it.

## Error Handling

Error handling in Rust is robust, using the `Option` and `Result` types to
indicate when data may be missing or errors may occur. Error handling in C
is less powerful, usually using some sentinel values in the return value of
a function to indicate an error has occured. There is also the `errno` API
from the C standard library, which permits reporting of a standardized set
of error codes, but that mechanism doesn't permit custom error information,
and would require us to create a mapping from all possible `pact_matching`
errors into `errno` error codes.

Instead, the error handling system in `pact_matching_ffi` is based around
1) signaling of errors through sentinel values, and 2) permitting C-side
collection of error messages through the `get_error_message` function.
This design is based on the work of Michael F. Bryan in the
[unofficial Rust FFI Guide][ffi_guide], which was based on the error handling
in `libgit2`.

In short: errors are collected from all FFI code, and stored in a
thread-local variable called `LAST_ERROR`. The `get_error_message` function
pulls the latest error message from `LAST_ERROR` and reports it to the
user. All the mechanisms in `pact_matching_ffi/src/error` exist to handle
this error collection and reporting.

## Logging

Logging is a crucial part of any application, and just because `pact_matching`
is being called from another language is no reason to render logging info
from it unavailable. For this reason, `pact_matching_ffi` provides a way for
C callers to initialize a logger and direct logging output to a location of
their choice.

First, they call `logger_init`, which begins the logging setup by creating
a log dispatcher with no sinks. Then they call `logger_attach_sink` to add
sinks, to which any logs matching the filtering level will be sent. Finally,
after all `logger_attach_sink` calls are done, they call `logger_apply` to
apply the logger and complete setup.

The remainder of the code in `pact_matching_ffi/src/log` is plumbing for this
logging setup process.

## The `ffi_fn` macro

FFI functions have to be written with some boilerplate. First, they must be
marked `#[no_mangle]` so their names are exported as-written for C to call.
Second, they must be marked `pub extern` so they're exported and picked up
by `cbindgen`, the tool we use for generating the C header which users of
`pact_matching_ffi` will need.

Then, within the functions themselves, we want to be sure that certain FFI
information is logged, and that errors are captured and reported consistently
and correctly.

For this purpose, we use the `ffi_fn` macro, found in
`pact_matching_ffi/src/util/ffi.rs`. This macro is inspired by a
[macro of the same name][hyper_macro] from the `hyper` crate's C API
(created for use of Hyper in `curl`). Our macro does considerably more work
to capture errors and ensure complete logging, but the idea is the same. The
macro also ensures panics are captured and packaged up like regular errors.
This includes panics due to allocation failure, although we don't do anything
to handle this case specifically.

One thing to note is that most functions using the macro will be written
slightly differently from normal Rust functions. If the function being
wrapped is fallible, then a second block is needed after the function body
block showing what to return in the case of any unexpected failure (usually
either a null pointer of the appropriate `const`-ness and type, or a
sentinel integer value signalling an error occurred).

Here's an example of the macro in use.

```rust
ffi_fn! {
    /// Get a mutable pointer to a newly-created default message on the heap.
    fn message_new() -> *mut Message {
        let message = Message::default();
        ptr::raw_to(message)
    } {
        ptr::null_mut_to::<Message>()
    }
}
```

For destructors, or other functions which have no return value, no second
block is required.

```rust
ffi_fn! {
    /// Destroy the `Message` being pointed to.
    fn message_delete(message: *mut Message) {
        ptr::drop_raw(message);
    }
}
```

## Opaque Pointers

The `pact_matching_ffi` crate is [_non-invasive_][non_invasive], meaning it
doesn't involve the modification of types in `pact_matching` to be exposable
over the FFI boundary (which would at minimum mean marking them `#[repr(C)]`).
Instead, we expose types in almost all cases as an "opaque pointer," meaning
C knows the _name_ of the type, but not its _contents_, and works only with
pointers to that type.

On the one hand, this is convenient, because it keeps the FFI boundary clear
and separate. On the other hand, it means that C code has to make function
calls to the Rust side rather than accessing fields directly, and that certain
optimizations may be missed because of the indirection. This can be addressed
at least in part through use of cross-language [Link Time Optimization (LTO)][lto]

## Strings

For the most part, to avoid issues with buffer sizing and fallible operations
against output parameters, the `pact_matching_ffi` crate is designed, when
returning strings, to return them as `const char *`, pointing to a
heap-allocated string buffer which must be deleted by calling the provided
`string_delete` function when finished.

One exception is returning the error message, where the user provides a buffer
and the size of that buffer, and the operation to fill the buffer with the
error message may fail. This is because error handling is expected to be very
common, so buffer reuse is ideal.

Additionally, `pact_matching_ffi` returns exclusively UTF-8-encoded strings,
and expects all strings it receives to be UTF-8 encoded.

## Buffers

Whenever `pact_matching_ffi` writes to a buffer, it zeroes out any excess
capacity in the buffer for security reasons.

[ffi_guide]: https://michael-f-bryan.github.io/rust-ffi-guide/errors/index.html
[hyper_macro]: https://github.com/hyperium/hyper/blob/master/src/ffi/macros.rs
[non_invasive]: https://www.possiblerust.com/guide/inbound-outbound-ffi#non-invasive-outbound-ffi
[lto]: http://blog.llvm.org/2019/09/closing-gap-cross-language-lto-between.html
