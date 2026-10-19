# Releasing

Each crate releases independently through its own pull request.

## Release pull requests

On every push to `master`, the Release PR workflow keeps one draft pull request per crate with unreleased changes, on branch `release/<crate>`. Each one bumps the crate's version and adds a changelog section generated from the conventional commits since its last release.

To release a crate:

1. Review the draft `release/<crate>` pull request. Edit the version or changelog on its branch if needed. Once anyone pushes to it, the bot stops regenerating it; close it to get a fresh one.
2. Mark it ready for review. The full test suite runs. For `pact_ffi` and `pact_verifier_cli`, the release build for all eight targets runs too.
3. Merge it.

A release pull request cannot merge while it requires a workspace crate version that is not yet released; release that crate first.

## After merge

1. The merge commit is tagged `<crate>-v<version>`.
2. The tag starts the release workflow:
   1. `pact_ffi` and `pact_verifier_cli` build with dist.
   2. The crate is published to crates.io through trusted publishing.
   3. A GitHub release is created with the changelog section and any artifacts.
3. After a `pact_verifier_cli` release, the `pactfoundation/pact-verifier` image is pushed to Docker Hub and GHCR.

## Checking a release gate build

Before merging a `pact_ffi` or `pact_verifier_cli` release pull request, check the artifacts its release gate built. This needs `gh`, LLVM (`llvm-dlltool`, `llvm-readobj`) and `readelf`.

```sh
gh run download <run id> -p 'artifacts-*' -D gate
mkdir -p gate-flat
find gate -type f \( -name '*.tar.xz' -o -name '*.zip' -o -name 'pact*.h' \) -exec cp {} gate-flat/ \;
LEGACY_ARCHIVES=gate-flat .github/scripts/legacy-artifacts.sh <crate>-v<version> out
```

The script fails if a Linux library links the wrong C library, or needs a glibc newer than 2.28.

## Legacy asset names

Each `pact_ffi` release is mirrored to a `libpact_ffi-v<version>` release that carries the pre-dist asset names. Each `pact_verifier_cli` release also carries `pact-verifier-<os>-<arch>.gz`.

These names are published until the legacy cut-off date set in `.github/scripts/legacy-artifacts.sh`. After that date, delete the legacy sections of this file, along with the other files that script lists.

## Recovering a release

Each step of a release can also be run by hand. Run these commands from the repository root, with tags fetched (`git fetch --tags origin`) and the tools from `mise.toml` installed (`mise install`). Each step names the credentials it needs.

### Re-run failed jobs

Most failures are transient, or are fixed outside the repository, e.g. a missing trusted publisher. Re-run only the failed jobs of the tag's release workflow:

```sh
gh run rerun <run id> --failed
```

Publishing skips a version already on crates.io. Don't re-run the whole workflow: release creation fails once the GitHub release exists.

### Release again after a failed release

A tag whose release can never succeed, e.g. one that needs a code fix, blocks its crate. While the tagged version is missing from crates.io, the Release PR workflow skips that crate. Deleting the tag doesn't help: the next push to `master` would tag the same version again.

Release the next version instead, from a release pull request opened by hand. It needs push access to the repository.

```sh
git switch -c release/<crate> origin/master
cd rust
release-plz set-version <crate>@<next version> --config release-plz.toml
cd ..
git commit -am "chore(<crate>): release v<next version>"
git push --force origin release/<crate>
gh pr create --draft --base master --head release/<crate> \
  --title "chore(<crate>): release v<next version>" \
  --body-file <(.github/scripts/changelog-section.sh rust/<crate>/CHANGELOG.md <next version>)
```

`set-version` must run from `rust/`, where the changelog paths in `release-plz.toml` resolve. It bumps the version in the crate's `Cargo.toml` and in `Cargo.lock`, and retitles the newest changelog section, which is the failed release's section. Add the fix's changelog entry to that section by hand. The bot never regenerates a release pull request that has commits from people. The failed version's tag stays, and no release ever ships under it.

If the Release PR workflow can't run at all, e.g. while the bot App is unavailable, open the pull request the same way, with `release-plz update -p <crate> --manifest-path rust/Cargo.toml --config rust/release-plz.toml` in place of `set-version`. It computes the version and the changelog from the commits since the last release.

### Tag a merged release

The Release PR workflow tags each version with no tag. If its `tag` job fails, re-run it, or tag by hand. This needs push access to tags.

```sh
.github/scripts/tag-releases.sh rust/Cargo.toml origin
```

The script tags the commit that set each untagged version, and refuses a version that is already on crates.io. A tag pushed with your own credentials starts the release workflow, as the bot's does.

### Publish to crates.io

Trusted publishing works only from the release workflows. To publish by hand, you need an owner's crates.io API token with the `publish-update` scope.

```sh
git switch --detach <crate>-v<version>
cargo login
cargo publish --manifest-path rust/Cargo.toml -p <crate> --locked
```

Then re-run the failed jobs of the tag's release workflow. Its publish job skips the version, and the GitHub release follows.

### Create a library crate's GitHub release

For `pact_models`, `pact_matching`, `pact_consumer` and `pact_verifier`, the GitHub release carries only the changelog section. This needs write access to releases.

```sh
.github/scripts/changelog-section.sh rust/<crate>/CHANGELOG.md <version> > notes.md
gh release create <crate>-v<version> --verify-tag --title "<crate> <version>" --notes-file notes.md
```

The `pact_ffi` and `pact_verifier_cli` releases carry the archives built by the release workflow, and only that workflow builds every target. Re-run its failed jobs.

### Publish legacy asset names

If the release workflow's legacy job failed, re-run it. When the fix is a change to the legacy scripts, run the Legacy artifacts workflow with the release tag instead. It runs the scripts from `master`, while a re-run uses the tagged commit's.

```sh
gh workflow run legacy-artifacts.yml -f tag=<crate>-v<version>
```

To build and upload them locally instead, you need write access to releases, LLVM and `readelf`:

```sh
GITHUB_REPOSITORY=pact-foundation/pact-reference .github/scripts/legacy-artifacts.sh <crate>-v<version> out

# pact_ffi: a separate release, never marked latest.
gh release create libpact_ffi-v<version> --target "$(git rev-parse 'pact_ffi-v<version>^{commit}')" \
  --latest=false --title "pact_ffi <version> (legacy asset names)" \
  --notes "Legacy-named assets for pact_ffi-v<version>." out/*

# pact_verifier_cli: the release's own assets.
gh release upload pact_verifier_cli-v<version> --clobber out/*
```

### Publish the container image

Run the Publish containers workflow with the version:

```sh
gh workflow run publish-containers.yml -f version=<version>
```

To push locally instead, log in to Docker Hub with push access to `pactfoundation/pact-verifier`, and to GHCR with a token that has the `write:packages` scope. The image downloads the musl binaries from the `pact_verifier_cli-v<version>` release, so that release must exist. Building both platforms needs a buildx builder that supports them, e.g. one made with `docker buildx create --use`.

```sh
docker login
docker login ghcr.io
docker buildx build rust/pact_verifier_cli \
  --platform linux/amd64,linux/arm64 \
  --build-arg VERSION=<version> \
  --tag pactfoundation/pact-verifier:<version> \
  --tag pactfoundation/pact-verifier:latest \
  --tag ghcr.io/pact-foundation/pact-verifier:<version> \
  --tag ghcr.io/pact-foundation/pact-verifier:latest \
  --push
```
