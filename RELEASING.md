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
diff .github/scripts/legacy-expected-ffi.txt <(ls out | sort)  # or legacy-expected-cli.txt
```

The script fails if a Linux library links the wrong C library, or needs a glibc newer than 2.28. The `diff` prints nothing when the legacy asset names match.

## Legacy asset names

Until 2027-06-30, each `pact_ffi` release is mirrored to a `libpact_ffi-v<version>` release that carries the pre-dist asset names. Each `pact_verifier_cli` release also carries `pact-verifier-<os>-<arch>.gz`.

To backfill these for an existing release, run the Legacy artifacts workflow manually with the release tag.

## Re-running a failed release

Re-run the failed jobs of the tag's release workflow. A version already on crates.io is skipped.
