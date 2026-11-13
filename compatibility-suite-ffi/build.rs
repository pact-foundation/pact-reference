// Generates runtime-loaded bindings for pact.h, so one binary can run against any libpact_ffi.
// The header defaults to the one `.github/scripts/ffi-examples.sh` installs; set PACT_FFI_HEADER to use another.
use std::{env, path::PathBuf};

fn main() {
  let header = env::var("PACT_FFI_HEADER").unwrap_or_else(|_| {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../rust/pact_ffi/build/install/include/pact.h").to_string()
  });
  println!("cargo:rerun-if-env-changed=PACT_FFI_HEADER");
  println!("cargo:rerun-if-changed={header}");
  bindgen::Builder::default()
    .header(&header)
    .allowlist_function("pactffi_.*")
    .allowlist_type("PactHandle|InteractionHandle|InteractionPart|PactSpecification|VerifierHandle|MessageContents|AsynchronousMessage|SynchronousMessage|PactAsyncMessageIterator|PactSyncMessageIterator|MessageMetadataIterator|MessageMetadataPair|Generator|GeneratorCategoryIterator|GeneratorKeyValuePair|GeneratorCategory")
    .prepend_enum_name(false)
    .dynamic_library_name("PactFfi")
    .dynamic_link_require_all(false)
    .generate()
    .expect("bindgen failed on pact.h")
    .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
    .unwrap();
}
