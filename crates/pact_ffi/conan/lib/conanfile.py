from conans import ConanFile, VisualStudioBuildEnvironment, CMake, tools

class CbindgenTestConan(ConanFile):
    name = "pact_ffi"
    version = "0.0.4"
    description = "Pact/Rust FFI bindings"
    url = "https://pactfoundation.jfrog.io/artifactory/pactfoundation-conan/"
    homepage = "https://github.com/pact-foundation/pact-reference"
    license = "MIT"
    settings = "os", "compiler", "build_type", "arch"
    no_copy_source = True
    requires = "openssl/1.1.1k"
    topics = ("pact", "consumer-driven-contracts", "contract-testing", "mock-server")

    def build(self):
        if self.settings.os == "Windows":
            url = ("https://github.com/pact-foundation/pact-reference/releases/download/libpact_ffi-v%s/pact_ffi-windows-x86_64.lib.gz"
                   % (str(self.version)))
            tools.download(url, "pact_ffi.lib.gz")
            tools.unzip("pact_ffi.lib.gz")
        elif self.settings.os == "Linux":
            url = ("https://github.com/pact-foundation/pact-reference/releases/download/libpact_ffi-v%s/libpact_ffi-linux-x86_64.a.gz"
                % (str(self.version)))
            tools.download(url, "libpact_ffi.a.gz")
            tools.unzip("libpact_ffi.a.gz")
        elif self.settings.os == "Macos":
            url = ("https://github.com/pact-foundation/pact-reference/releases/download/libpact_ffi-v%s/libpact_ffi-osx-x86_64.a.gz"
                   % (str(self.version)))
            tools.download(url, "libpact_ffi.a.gz")
            tools.unzip("libpact_ffi.a.gz")
        else:
            raise Exception("Binary does not exist for these settings")
        tools.download(("https://github.com/pact-foundation/pact-reference/releases/download/libpact_ffi-v%s/pact.h"
                % (str(self.version))), "include/pact.h")
        tools.download(("https://github.com/pact-foundation/pact-reference/releases/download/libpact_ffi-v%s/pact-cpp.h"
                        % (str(self.version))), "include/pact-cpp.h")

    def package(self):
        self.copy("libpact_ffi*.a", "lib", "")
        self.copy("pact_ffi*.lib", "lib", "")
        self.copy("*.h", "", "")

    def package_info(self):  # still very useful for package consumers
        self.cpp_info.libs = ["pact_ffi"]
