
# Pact-Reference Supported Platforms / Architectures

## Legend

- ✅ : Builds with cross and officially supported
- 🧪 : Builds with cross and unofficially released for testing
- 🚧 : Cross image available, doesn't build
- ❌ : doesn't build, (or no cross image available)
- ❓ : Untested
- 👷🏽 : Locally built and tested

## Pact-Reference Target Matrix

| target | notes | tier | cross | pact_ffi | pact_verifier_cli | pact_mock_server_cli
| ---- | ---- | --- | ---- | --- | --- | --- |
| x86_64-pc-windows-msvc| 64-bit MSVC (Windows 7+) | 1 |  ✅ | ✅ | ✅ | ✅ |
| x86_64-apple-darwin |64-bit macOS (10.7+, Lion+)| 1 | ✅ | ✅ | ✅ | ✅ |
| x86_64-unknown-linux-gnu |64-bit Linux (kernel 3.2+, glibc 2.17+)| 1 | ✅ | ✅ | ✅ | ✅ |
| aarch64-unknown-linux-gnu |ARM64 Linux (kernel 4.1, glibc 2.17+)| 1 | ✅ | ✅ | ✅ | ✅ |
| aarch64-apple-darwin |ARM64 macOS (11.0+, Big Sur+)| 2 | ❌ | ✅ |  ✅ |   ✅|
| x86_64-unknown-linux-musl |64-bit Linux with MUSL| 2 | ✅ | ✅ | 🚧 | 🚧 |

## Unofficial Pact-Reference Target Matrix

target lists taken from

- Rust Platform support list <https://doc.rust-lang.org/nightly/rustc/platform-support.html>
- Cross <https://github.com/cross-rs/cross>
- Cargo-xWin <https://github.com/rust-cross/cargo-xwin>

| target | notes | tier | cross | pact_ffi | pact_verifier_cli | pact_mock_server_cli
| ---- | ---- | --- | ---- | --- | --- | --- |
| aarch64-unknown-linux-gnu |ARM64 Linux (kernel 4.1, glibc 2.17+)| 1 | ✅ | ✅ | ✅ | ✅ |
| i686-pc-windows-gnu| 32-bit MinGW (Windows 7+) | 1 |🧪 | 🧪 | 🧪 | 🧪 |
| i686-pc-windows-msvc |32-bit MSVC (Windows 7+) | 1 | 🧪 | 🧪 | 🧪 | 🧪 |
| i686-unknown-linux-gnu| 32-bit Linux (kernel 3.2+, glibc 2.17+)| 1 | 🧪 | 🧪 | 🧪 | 🧪 |
| x86_64-apple-darwin |64-bit macOS (10.7+, Lion+)| 1 | ✅ | ✅ | ✅ | ✅ |
| x86_64-pc-windows-gnu |64-bit MinGW (Windows 7+) | 1 | 🧪 | 🧪 | 🧪 | 🧪 |
| x86_64-pc-windows-msvc| 64-bit MSVC (Windows 7+) | 1 |  ✅ | ✅ | ✅ | ✅ |
| x86_64-unknown-linux-gnu |64-bit Linux (kernel 3.2+, glibc 2.17+)| 1 | ✅ | ✅ | ✅ | ✅ |
| ---- | ---- | --- | ---- | --- | --- | --- |
| aarch64-apple-darwin |ARM64 macOS (11.0+, Big Sur+)| 2 | ❌ | ✅ |  ✅ |   ✅|
| aarch64-pc-windows-msvc| ARM64 Windows MSVC| 2 | 🧪 | 🧪 | 🧪 | 🧪 |
| aarch64-unknown-linux-musl |ARM64 Linux with MUSL| 2 | 🧪 | 🧪 | 🧪 | 🧪 |
| arm-unknown-linux-gnueabi |ARMv6 Linux (kernel 3.2, glibc 2.17)| 2 | 🧪 | 🧪 | 🧪 | 🧪 |
| arm-unknown-linux-gnueabihf |ARMv6 Linux, hardfloat (kernel 3.2, glibc 2.17)| 2 | 🧪 | 🧪 | 🧪 | 🧪 |
| armv7-unknown-linux-gnueabihf |ARMv7 Linux, hardfloat (kernel 3.2, glibc 2.17)| 2 | 🧪 | 🧪 | 🧪 | 🧪 |
| mips-unknown-linux-gnu| MIPS Linux (kernel 4.4, glibc 2.23)| 2|❌ |  ❓ | ❓  |  ❓ |
| mips64-unknown-linux-gnuabi64| MIPS64 Linux, n64 ABI (kernel 4.4, glibc 2.23)| 2|❌ |  ❓ | ❓  |  ❓ |
| mips64el-unknown-linux-gnuabi64| MIPS64 (LE) Linux, n64 ABI (kernel 4.4, glibc 2.23)| 2|❌ |  ❓ | ❓  |  ❓ |
| mipsel-unknown-linux-gnu |MIPS (LE) Linux (kernel 4.4, glibc 2.23)| 2|❌ |  ❓ | ❓  |  ❓ |
| powerpc-unknown-linux-gnu| PowerPC Linux (kernel 3.2, glibc 2.17)| 2 |🚧  | 🚧  | 🚧   | 🚧  |
| powerpc64-unknown-linux-gnu| PPC64 Linux (kernel 3.2, glibc 2.17)| 2 |🚧  | 🚧  | 🚧   | 🚧  |
| powerpc64le-unknown-linux-gnu |PPC64LE Linux (kernel 3.10, glibc 2.17)| 2 |🚧  | 🚧  | 🚧   | 🚧  |
| riscv64gc-unknown-linux-gnu| RISC-V Linux (kernel 4.20, glibc 2.29)| 2 | 🚧  | 🚧  | 🚧   | 🚧  |
| s390x-unknown-linux-gnu| S390x Linux (kernel 3.2, glibc 2.17)| 2 | 🚧  | 🚧  | 🚧   | 🚧  |
| x86_64-unknown-freebsd| 64-bit FreeBSD| 2 |🧪 | 🧪 | 🧪 | 🧪 |
| x86_64-unknown-illumos| illumos| 2 |🚧  | 🚧  | 🚧   | 🚧  |
| x86_64-unknown-linux-musl |64-bit Linux with MUSL| 2 | ✅ | ✅ | 🚧 | 🚧 |
| x86_64-unknown-netbsd| NetBSD/amd64| 2 |🧪 | 🧪 | 🧪 | 🧪 |
| ---- | ---- | --- | ---- | --- | --- | --- |
| aarch64-apple-ios | ARM64 iOS| 2*| 🧪 | 🧪 | 🧪 | 🧪 |
| aarch64-apple-ios-sim | Apple iOS Simulator on ARM64| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| aarch64-fuchsia | Alias for aarch64-unknown-fuchsia| 2*|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-fuchsia | ARM64 Fuchsia| 2*|❌ |  ❓ | ❓  |  ❓ |
| aarch64-linux-android | ARM64 Android| 2*|🚧  | 🚧  | 🚧   | 🚧  |
| aarch64-unknown-none-softfloat | Bare ARM64, softfloat| 2*|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-none | Bare ARM64, hardfloat| 2*|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-uefi | ARM64 UEFI| 2*|❌ |  ❓ | ❓  |  ❓ |
| arm-linux-androideabi | ARMv7 Android| 2*| 🧪 | 🧪 | 🧪 | 🧪 |
| arm-unknown-linux-musleabi | ARMv6 Linux with MUSL| 2*| 🧪 | 🧪 | 🧪 | 🧪 |
| arm-unknown-linux-musleabihf | ARMv6 Linux with MUSL, hardfloat| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| armebv7r-none-eabi | Bare ARMv7-R, Big Endian| 2*|❌ |  ❓ | ❓  |  ❓ |
| armebv7r-none-eabihf | Bare ARMv7-R, Big Endian, hardfloat| 2*|❌ |  ❓ | ❓  |  ❓ |
| armv5te-unknown-linux-gnueabi | ARMv5TE Linux (kernel 4.4, glibc 2.23)| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| armv5te-unknown-linux-musleabi | ARMv5TE Linux with MUSL| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| armv7-linux-androideabi | ARMv7a Android| 2*| 🧪 | 🧪 | 🧪 | 🧪 |
| armv7-unknown-linux-gnueabi | ARMv7 Linux (kernel 4.15, glibc 2.27)| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| armv7-unknown-linux-musleabi | ARMv7 Linux with MUSL| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| armv7-unknown-linux-musleabihf | ARMv7 Linux with MUSL, hardfloat| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| armv7a-none-eabi | Bare ARMv7-A| 2*|❌ |  ❓ | ❓  |  ❓ |
| armv7r-none-eabi | Bare ARMv7-R| 2*|❌ |  ❓ | ❓  |  ❓ |
| armv7r-none-eabihf | Bare ARMv7-R, hardfloat| 2*|❌ |  ❓ | ❓  |  ❓ |
| asmjs-unknown-emscripten | asm.js via Emscripten| 2*|🚧  | 🚧  | 🚧   | 🚧  |
| i586-pc-windows-msvc | 32-bit Windows w/o SSE| 2*|❌ |  ❓ | ❓  |  ❓ |
| i586-unknown-linux-gnu | 32-bit Linux w/o SSE (kernel 3.2, glibc 2.17)| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| i586-unknown-linux-musl | 32-bit Linux w/o SSE, MUSL| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| i686-linux-android | 32-bit x86 Android| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| i686-unknown-freebsd | 32-bit FreeBSD| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| i686-unknown-linux-musl | 32-bit Linux with MUSL| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| i686-unknown-uefi | 32-bit UEFIV| 2*|❌ |  ❓ | ❓  |  ❓ |
| mips-unknown-linux-musl | MIPS Linux with MUSL| 2*|🚧  | 🚧  | 🚧   | 🚧  |
| mips64-unknown-linux-muslabi64 | MIPS64 Linux, n64 ABI, MUSL| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| mips64el-unknown-linux-muslabi64 | MIPS64 (LE) Linux, n64 ABI, MUSL| 2*|🚧  | 🚧  | 🚧   | 🚧  |
| mipsel-unknown-linux-musl | MIPS (LE) Linux with MUSL| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| nvptx64-nvidia-cuda | --emit=asm generates PTX code that runs on NVIDIA GPUs| 2*|❌ |  ❓ | ❓  |  ❓ |
| riscv32i-unknown-none-elf | Bare RISC-V (RV32I ISA)| 2*|❌ |  ❓ | ❓  |  ❓ |
| riscv32imac-unknown-none-elf | Bare RISC-V (RV32IMAC ISA)| 2*|❌ |  ❓ | ❓  |  ❓ |
| riscv32imc-unknown-none-elf | Bare RISC-V (RV32IMC ISA)| 2*|❌ |  ❓ | ❓  |  ❓ |
| riscv64gc-unknown-none-elf | Bare RISC-V (RV64IMAFDC ISA)| 2*|❌ |  ❓ | ❓  |  ❓ |
| riscv64imac-unknown-none-elf | Bare RISC-V (RV64IMAC ISA)| 2*|❌ |  ❓ | ❓  |  ❓ |
| sparc64-unknown-linux-gnu | SPARC Linux (kernel 4.4, glibc 2.23)| 2*|🚧  | 🚧  | 🚧   | 🚧  |
| sparcv9-sun-solaris | SPARC Solaris 10/11, illumos| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| thumbv6m-none-eabi | Bare Cortex-M0, M0+, M1| 2*|🚧  | 🚧  | 🚧   | 🚧  |
| thumbv7em-none-eabi | Bare Cortex-M4, M7| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| thumbv7em-none-eabihf | Bare Cortex-M4F, M7F, FPU, hardfloat| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| thumbv7m-none-eabi | Bare Cortex-M3| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| thumbv7neon-linux-androideabi | Thumb2-mode ARMv7a Android with NEON| 2*| 🧪 | 🧪 | 🧪 | 🧪 |
| thumbv7neon-unknown-linux-gnueabihf | Thumb2-mode ARMv7a Linux with NEON (kernel 4.4, glibc | 2.23)| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| thumbv8m.base-none-eabi | ARMv8-M Baseline| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| thumbv8m.main-none-eabi | ARMv8-M Mainline| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| thumbv8m.main-none-eabihf | ARMv8-M Mainline, hardfloat| 2*| 🚧  | 🚧  | 🚧   | 🚧  |
| wasm32-unknown-emscripten | WebAssembly via Emscripten| 2*|🚧  | 🚧  | 🚧   | 🚧  |
| wasm32-unknown-unknown | WebAssembly| 2*|❌ |  ❓ | ❓  |  ❓ |
| wasm32-wasi | WebAssembly with WASI| 2*|❌ |  ❓ | ❓  |  ❓ |
| x86_64-apple-ios | 64-bit x86 iOS| 2*|🧪 | 🧪 | 🧪 | 🧪 |
| x86_64-fortanix-unknown-sgx | Fortanix ABI for 64-bit Intel SGX| 2*|❌ |  ❓ | ❓  |  ❓ |
| x86_64-fuchsia | Alias for x86_64-unknown-fuchsia| 2*|❌ |  ❓ | ❓  |  ❓ |
| x86_64-unknown-fuchsia | 64-bit Fuchsia| 2*|❌ |  ❓ | ❓  |  ❓ |
| x86_64-linux-android | 64-bit x86 Android| 2*|🚧  | 🚧  | 🚧   | 🚧  |
| x86_64-pc-solaris | 64-bit Solaris 10/11, illumos| 2*|❌ |  ❓ | ❓  |  ❓ |
| x86_64-unknown-linux-gnux32 | 64-bit Linux (x32 ABI) (kernel 4.15, glibc 2.27)| 2*|❌ |  ❓ | ❓  |  ❓ |
| x86_64-unknown-none | Freestanding/bare-metal x86_64, softfloat| 2*|❌ |  ❓ | ❓  |  ❓ |
| x86_64-unknown-redox | Redox OS| 2*|❌ |  ❓ | ❓  |  ❓ |
| x86_64-unknown-uefi | 64-bit UEFI| 2*|❌ |  ❓ | ❓  |  ❓ |
| ---- | ---- | --- | ---- | --- | --- | --- |
| aarch64-apple-ios-macabi |  Apple Catalyst on ARM64|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-apple-tvos |  ARM64 tvOS|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-apple-watchos-sim |  ARM64 Apple WatchOS Simulator|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-kmc-solid_asp3 |  ARM64 SOLID with TOPPERS/ASP3|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-nintendo-switch-freestanding |  ARM64 Nintendo Switch, Horizon|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-pc-windows-gnullvm | | |3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-linux-ohos |  ARM64 OpenHarmony|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-nto-qnx710 |  ARM64 QNX Neutrino 7.1 RTOS|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-freebsd | | ARM64 FreeBSD|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-hermit |  ARM64 HermitCore|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-linux-gnu_ilp32 | | ARM64 Linux (ILP32 ABI)|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-netbsd | | |3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-openbsd | | ARM64 OpenBSD|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-unknown-redox |  ARM64 Redox OS|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-uwp-windows-msvc |  |3|❌ |  ❓ | ❓  |  ❓ |
| aarch64-wrs-vxworks |  |3|❌ |  ❓ | ❓  |  ❓ |
| aarch64_be-unknown-linux-gnu_ilp32 | | ARM64 Linux (big-endian, ILP32 ABI)|3|❌ |  ❓ | ❓  |  ❓ |
| aarch64_be-unknown-linux-gnu | | ARM64 Linux (big-endian)|3|❌ |  ❓ | ❓  |  ❓ |
| arm64_32-apple-watchos |  ARM Apple WatchOS 64-bit with 32-bit pointers|3|❌ |  ❓ | ❓  |  ❓ |
| armeb-unknown-linux-gnueabi | | ARM BE8 the default ARM big-endian architecture since | ARMv6.|3|❌ |  ❓ | ❓  |  ❓ |
| armv4t-none-eabi |  ARMv4T A32|3|❌ |  ❓ | ❓  |  ❓ |
| armv4t-unknown-linux-gnueabi |  |3|❌ |  ❓ | ❓  |  ❓ |
| armv5te-none-eabi |  ARMv5TE A32|3|❌ |  ❓ | ❓  |  ❓ |
| armv5te-unknown-linux-uclibceabi |  ARMv5TE Linux with uClibc|3|❌ |  ❓ | ❓  |  ❓ |
| armv6-unknown-freebsd | | ARMv6 FreeBSD|3|❌ |  ❓ | ❓  |  ❓ |
| armv6-unknown-netbsd-eabihf |  |3|❌ |  ❓ | ❓  |  ❓ |
| armv6k-nintendo-3ds |  ARMv6K Nintendo 3DS, Horizon (Requires devkitARM toolchain)|3|❌ |  ❓ | ❓  |  ❓ |
| armv7-apple-ios |  ARMv7 iOS, Cortex-a8|3|❌ |  ❓ | ❓  |  ❓ |
| armv7-sony-vita-newlibeabihf |  ARM Cortex-A9 Sony PlayStation Vita (requires VITASDK | toolchain)|3|❌ |  ❓ | ❓  |  ❓ |
| armv7-unknown-linux-ohos |  ARMv7 OpenHarmony|3|❌ |  ❓ | ❓  |  ❓ |
| armv7-unknown-linux-uclibceabi | | ARMv7 Linux with uClibc, softfloat|3|❌ |  ❓ | ❓  |  ❓ |
| armv7-unknown-linux-uclibceabihf | | ARMv7 Linux with uClibc, hardfloat|3|❌ |  ❓ | ❓  |  ❓ |
| armv7-unknown-freebsd | | ARMv7 FreeBSD|3|❌ |  ❓ | ❓  |  ❓ |
| armv7-unknown-netbsd-eabihf | | |3|❌ |  ❓ | ❓  |  ❓ |
| armv7-wrs-vxworks-eabihf |  |3|❌ |  ❓ | ❓  |  ❓ |
| armv7a-kmc-solid_asp3-eabi |  ARM SOLID with TOPPERS/ASP3|3|❌ |  ❓ | ❓  |  ❓ |
| armv7a-kmc-solid_asp3-eabihf |  ARM SOLID with TOPPERS/ASP3, hardfloat|3|❌ |  ❓ | ❓  |  ❓ |
| armv7a-none-eabihf |  ARM Cortex-A, hardfloat|3|❌ |  ❓ | ❓  |  ❓ |
| armv7k-apple-watchos |  ARM Apple WatchOS|3|❌ |  ❓ | ❓  |  ❓ |
| armv7s-apple-ios |  |3|❌ |  ❓ | ❓  |  ❓ |
| avr-unknown-gnu-atmega328 |  AVR. Requires -Z build-std=core|3|❌ |  ❓ | ❓  |  ❓ |
| bpfeb-unknown-none |  BPF (big endian)|3|❌ |  ❓ | ❓  |  ❓ |
| bpfel-unknown-none |  BPF (little endian)|3|❌ |  ❓ | ❓  |  ❓ |
| hexagon-unknown-linux-musl |  |3|❌ |  ❓ | ❓  |  ❓ |
| i386-apple-ios |  32-bit x86 iOS|3|❌ |  ❓ | ❓  |  ❓ |
| i586-pc-nto-qnx700 |  32-bit x86 QNX Neutrino 7.0 RTOS|3|❌ |  ❓ | ❓  |  ❓ |
| i686-apple-darwin | | 32-bit macOS (10.7+, Lion+)|3|❌ |  ❓ | ❓  |  ❓ |
| i686-unknown-haiku | | 32-bit Haiku|3|❌ |  ❓ | ❓  |  ❓ |
| i686-unknown-netbsd | | NetBSD/i386 with SSE2|3|❌ |  ❓ | ❓  |  ❓ |
| i686-unknown-openbsd | | 32-bit OpenBSD|3|❌ |  ❓ | ❓  |  ❓ |
| i686-uwp-windows-gnu |  |3|❌ |  ❓ | ❓  |  ❓ |
| i686-uwp-windows-msvc |  |3|❌ |  ❓ | ❓  |  ❓ |
| i686-wrs-vxworks |  |3|❌ |  ❓ | ❓  |  ❓ |
| loongarch64-unknown-linux-gnu |  LoongArch64 Linux (LP64D ABI)|3|❌ |  ❓ | ❓  |  ❓ |
| m68k-unknown-linux-gnu |  Motorola 680x0 Linux|3|❌ |  ❓ | ❓  |  ❓ |
| mips-unknown-linux-uclibc |  MIPS Linux with uClibc|3|❌ |  ❓ | ❓  |  ❓ |
| mips64-openwrt-linux-musl |  MIPS64 for OpenWrt Linux MUSL|3|❌ |  ❓ | ❓  |  ❓ |
| mipsel-sony-psp |  MIPS (LE) Sony PlayStation Portable (PSP)|3|❌ |  ❓ | ❓  |  ❓ |
| mipsel-sony-psx |  MIPS (LE) Sony PlayStation 1 (PSX)|3|❌ |  ❓ | ❓  |  ❓ |
| mipsel-unknown-linux-uclibc |  MIPS (LE) Linux with uClibc|3|❌ |  ❓ | ❓  |  ❓ |
| mipsel-unknown-none |  Bare MIPS (LE) softfloat|3|❌ |  ❓ | ❓  |  ❓ |
| mipsisa32r6-unknown-linux-gnu |  |3|❌ |  ❓ | ❓  |  ❓ |
| mipsisa32r6el-unknown-linux-gnu |  |3|❌ |  ❓ | ❓  |  ❓ |
| mipsisa64r6-unknown-linux-gnuabi64 |  |3|❌ |  ❓ | ❓  |  ❓ |
| mipsisa64r6el-unknown-linux-gnuabi64 |  |3|❌ |  ❓ | ❓  |  ❓ |
| msp430-none-elf |  16-bit MSP430 microcontrollers|3|❌ |  ❓ | ❓  |  ❓ |
| powerpc-unknown-linux-gnuspe |  PowerPC SPE Linux|3|❌ |  ❓ | ❓  |  ❓ |
| powerpc-unknown-linux-musl |  |3|❌ |  ❓ | ❓  |  ❓ |
| powerpc-unknown-netbsd | | |3|❌ |  ❓ | ❓  |  ❓ |
| powerpc-unknown-openbsd |  |3|❌ |  ❓ | ❓  |  ❓ |
| powerpc-wrs-vxworks-spe |  |3|❌ |  ❓ | ❓  |  ❓ |
| powerpc-wrs-vxworks |  |3|❌ |  ❓ | ❓  |  ❓ |
| powerpc64-unknown-freebsd | | PPC64 FreeBSD (ELFv1 and ELFv2)|3|❌ |  ❓ | ❓  |  ❓ |
| powerpc64le-unknown-freebsd   PPC64LE FreeBSD|3|❌ |  ❓ | ❓  |  ❓ |
| powerpc-unknown-freebsd   PowerPC FreeBSD|3|❌ |  ❓ | ❓  |  ❓ |
| powerpc64-unknown-linux-musl |  |3|❌ |  ❓ | ❓  |  ❓ |
| powerpc64-wrs-vxworks |  |3|❌ |  ❓ | ❓  |  ❓ |
| powerpc64le-unknown-linux-musl |  |3|❌ |  ❓ | ❓  |  ❓ |
| powerpc64-unknown-openbsd | | OpenBSD/powerpc64|3|❌ |  ❓ | ❓  |  ❓ |
| powerpc64-ibm-aix |  64-bit AIX (7.2 and newer)|3|❌ |  ❓ | ❓  |  ❓ |
| riscv32gc-unknown-linux-gnu   RISC-V Linux (kernel 5.4, glibc 2.33)|3|❌ |  ❓ | ❓  |  ❓ |
| riscv32gc-unknown-linux-musl   RISC-V Linux (kernel 5.4, musl + RISCV32 support | patches)|3|❌ |  ❓ | ❓  |  ❓ |
| riscv32im-unknown-none-elf |  Bare RISC-V (RV32IM ISA)|3|❌ |  ❓ | ❓  |  ❓ |
| riscv32imac-unknown-xous-elf |  RISC-V Xous (RV32IMAC ISA)|3|❌ |  ❓ | ❓  |  ❓ |
| riscv32imc-esp-espidf |  RISC-V ESP-IDF|3|❌ |  ❓ | ❓  |  ❓ |
| riscv64gc-unknown-freebsd   RISC-V FreeBSD|3|❌ |  ❓ | ❓  |  ❓ |
| riscv64gc-unknown-fuchsia   RISC-V Fuchsia|3|❌ |  ❓ | ❓  |  ❓ |
| riscv64gc-unknown-linux-musl   RISC-V Linux (kernel 4.20, musl 1.2.0)|3|❌ |  ❓ | ❓  |  ❓ |
| riscv64gc-unknown-openbsd | | OpenBSD/riscv64|3|❌ |  ❓ | ❓  |  ❓ |
| s390x-unknown-linux-musl   S390x Linux (kernel 3.2, MUSL)|3|❌ |  ❓ | ❓  |  ❓ |
| sparc-unknown-linux-gnu |  32-bit SPARC Linux|3|❌ |  ❓ | ❓  |  ❓ |
| sparc64-unknown-netbsd | | NetBSD/sparc64|3|❌ |  ❓ | ❓  |  ❓ |
| sparc64-unknown-openbsd | | OpenBSD/sparc64|3|❌ |  ❓ | ❓  |  ❓ |
| thumbv4t-none-eabi |  ARMv4T T32|3|❌ |  ❓ | ❓  |  ❓ |
| thumbv5te-none-eabi |  ARMv5TE T32|3|❌ |  ❓ | ❓  |  ❓ |
| thumbv7a-pc-windows-msvc |  |3|❌ |  ❓ | ❓  |  ❓ |
| thumbv7a-uwp-windows-msvc |  |3|❌ |  ❓ | ❓  |  ❓ |
| thumbv7neon-unknown-linux-musleabihf |  Thumb2-mode ARMv7a Linux with NEON, MUSL|3|❌ |  ❓ | ❓  |  ❓ |
| wasm64-unknown-unknown |  WebAssembly|3|❌ |  ❓ | ❓  |  ❓ |
| x86_64-apple-ios-macabi |  Apple Catalyst on x86_64|3|❌ |  ❓ | ❓  |  ❓ |
| x86_64-apple-tvos |  x86 64-bit tvOS|3|❌ |  ❓ | ❓  |  ❓ |
| x86_64-apple-watchos-sim |  x86 64-bit Apple WatchOS simulator|3|❌ |  ❓ | ❓  |  ❓ |
| x86_64-pc-nto-qnx710 |  x86 64-bit QNX Neutrino 7.1 RTOS|3|❌ |  ❓ | ❓  |  ❓ |
| x86_64-pc-windows-gnullvm | | |3| ❌ |  ❓ | ❓  |  ❓ |
| x86_64-sun-solaris |  Deprecated target for 64-bit Solaris 10/11, illumos|3| 🚧  | 🚧  | 🚧   | 🚧  |
| x86_64-unknown-dragonfly | | 64-bit DragonFlyBSD|3| 🚧  | 🚧  | 🚧   | 🚧  |
| x86_64-unknown-haiku | | 64-bit Haiku|3|❌ |  ❓ | ❓  |  ❓ |
| x86_64-unknown-hermit |  HermitCore|3| ❌ |  ❓ | ❓  |  ❓ |
| x86_64-unknown-l4re-uclibc |  |3| ❌ |  ❓ | ❓  |  ❓ |
| x86_64-unknown-openbsd | | 64-bit OpenBSD|❌ |  ❓ | ❓  |  ❓ |
| x86_64-uwp-windows-gnu |  |3| ❌ |  ❓ | ❓  |  ❓ |
| x86_64-uwp-windows-msvc |  |3| ❌ |  ❓ | ❓  |  ❓ |
| x86_64-wrs-vxworks |  |3| ❌ |  ❓ | ❓  |  ❓ |
| x86_64h-apple-darwin | | macOS with late-gen Intel (at least Haswell)|3| ❌ |  ❓ | ❓  |  ❓ |
