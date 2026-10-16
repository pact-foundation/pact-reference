# Contributing to Pact-Rust

PRs are always welcome!

## Raising defects

Before raising an issue, make sure you have checked the open and closed issues to see if an answer is provided there.
There may also be an answer to your question on [stackoverflow](https://stackoverflow.com/questions/tagged/pact).

Please provide the following information with your issue to enable us to respond as quickly as possible.

1. The relevant versions of the packages you are using.
1. The steps to recreate your issue.
1. An executable code example where possible.

## New features / changes

1. Fork it
1. Create your feature branch (git checkout -b my-new-feature)
1. Commit your changes (git commit -am 'feat: Add some feature')
1. Push to the branch (git push origin my-new-feature)
1. Create new Pull Request

### Commit messages

We follow the [Conventional Changelog](https://github.com/bcoe/conventional-changelog-standard/blob/master/convention.md)
message conventions. Please ensure you follow the guidelines.

If you'd like to get some CLI assistance, getting setup is easy:

```shell
npm install commitizen -g
npm i -g cz-conventional-changelog
```

`git cz` to commit and commitizen will guide you.

### Building

To build the libraries in this project, you need a working Rust environment. Refer to the [Rust Guide](https://www.rust-lang.org/learn/get-started).

The build tool used is `cargo`.

```shell
cd rust
cargo build
```

This will compile all the libraries and put the generated files in `rust/target/debug`.

The root `rust-toolchain.toml` pins the Rust version. [mise](https://mise.jdx.dev) manages the rest of the dev tooling; run `mise install` from the repository root to install it.

### Testing

CI runs the tests with [nextest](https://nexte.st). nextest skips doctests, so run those separately:

```shell
cd rust
cargo nextest run --workspace
cargo test --workspace --doc
```

Plain `cargo test` also runs the whole suite, doctests included.

### Releasing

The released libraries for each module are built by a GH action that attaches the libraries to the GH release for each
crate. To release a crate, run the `release.groovy` script in the crate directory. This will guide you through the
release process for the crate. Then create a GH release using the tag and changelog created by the script.

To be able to release a component, you need to:
1. Have an account on [crates.io](crates.io). Go to that site, and select "Log in with GitHub". It will auto-create your user account.
2. Get a maintainer to invite you to the crate for the component.
3. Accept the invite on crates.io. This will allow you to release the Rust crate to crates.io.
4. 