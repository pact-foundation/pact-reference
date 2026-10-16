use os_info::Type;

fn main() {
    let info = os_info::get();
    if info.os_type() == Type::Macos {
      // Remove hardcoded path to avoid need to use install_name_tool.
      // Drop file into a well-known path such as /usr/local/lib and it can be automatically discovered
      println!("cargo:rustc-cdylib-link-arg=-Wl,-install_name,libpact_ffi.dylib");
    }
}
