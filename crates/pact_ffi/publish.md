## Publishing the FFI libs to conan repo

You're obviously going to replace the versions below with the actual versions you're releasing, and not
just copy it verbatim and expect magical things to happen.

### For lib conan package

```
 git checkout libpact_ffi-v0.0.7
 cd conan/lib/
 conan create . pact/beta
 conan upload pact_ffi/0.0.7@pact/beta -r=pact-foundation
```

### For DLL conan package

```
 git checkout libpact_ffi-v0.0.7
 cd conan/dll/
 conan create . pact/beta
 conan upload pact_ffi_dll/0.0.7@pact/beta -r=pact-foundation
```
