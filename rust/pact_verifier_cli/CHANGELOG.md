To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 1.2.0 - Add Exit on first error and Only run previously failed interactions CLI options

* ce3c5d9e - feat(pact_verifier_cli): Update the docs on the new options flag #494 (Ronald Holshausen, Fri Jun 20 11:56:43 2025 +1000)
* 4fc0136c - chore: Upgrade pact_verifier to 1.3.2 (Ronald Holshausen, Fri Jun 20 11:33:58 2025 +1000)
* 31113440 - feat(pact_verifier_cli): Implement the --last-failed CLI flag #494 (Ronald Holshausen, Fri Jun 20 11:03:44 2025 +1000)
* e30beba4 - feat(pact_verifier): Update requires for the --last-failed option #494 (Ronald Holshausen, Thu Jun 19 14:36:25 2025 +1000)
* 45005bc7 - feat(pact_verifier_cli): Rename exit-first to exit-on-first-error #494 (Ronald Holshausen, Thu Jun 19 10:53:34 2025 +1000)
* bbf9dfea - chore(pact_verifier_cli): Update the CLI test fixtures #494 (Ronald Holshausen, Wed Jun 18 16:29:48 2025 +1000)
* 1f734dd8 - feat(pact_verifier_cli): Add the exit-first and last-failed CLI flags #494 (Ronald Holshausen, Wed Jun 18 16:06:17 2025 +1000)
* 7afa3849 - chore(pact_verifier): Update crate to 2024 edition and bump minor version (Ronald Holshausen, Wed Jun 18 14:52:11 2025 +1000)
* 6e8167c4 - fix(Alpine): Updae pact_models to 1.3.3 (Ronald Holshausen, Wed Jun 11 13:23:10 2025 +1000)
* 18e53fbd - chore: Update pact_models to 1.3.2 (Ronald Holshausen, Wed Jun 11 10:16:03 2025 +1000)
* 2b4536b1 - chore: Bump pact_verifier to 1.3.0 (Ronald Holshausen, Tue May 20 16:34:00 2025 +1000)
* 7d8e8f61 - chore: Upgrade pact_models to 1.3.1 (Ronald Holshausen, Tue May 20 14:53:42 2025 +1000)
* 673dacab - chore: Fix CLI test after upgrade to dependencies (Ronald Holshausen, Tue May 20 11:06:20 2025 +1000)
* 0ed75202 - chore(pact-models): Upgrade pact models crate to Rust 2024 edition (Ronald Holshausen, Tue Mar 25 16:02:47 2025 +1100)
* cff9a7f6 - chore(pact_verifier_cli): Correct the docker build for multiplatform (Ronald Holshausen, Wed Mar 19 15:59:19 2025 +1100)
* d76d7e2d - chore(pact_verifier_cli): Update docker base to Alpine 3.21 (Ronald Holshausen, Wed Mar 19 15:30:16 2025 +1100)
* eab8d1ab - bump version to 1.1.6 (Ronald Holshausen, Wed Mar 19 15:22:15 2025 +1100)

# 1.1.5 - Bugfix Release

* e1a7c0f0 - chore(pact_verifier_cli): Update dependencies (Ronald Holshausen, Wed Mar 19 15:11:20 2025 +1100)
* df9beec3 - chore: Upgrade pact_verifier to 1.2.5 (Ronald Holshausen, Wed Mar 19 14:08:12 2025 +1100)
* d67db1a4 - chore: Update pact_models to 1.2.7 (Ronald Holshausen, Fri Mar 14 11:02:53 2025 +1100)
* 823b009a - Reapply "chore(ci): refactor release pipeline" (Ronald Holshausen, Fri Dec 20 11:08:57 2024 +1100)
* 7c09f177 - Revert "chore(ci): refactor release pipeline" (Ronald Holshausen, Fri Dec 20 09:31:37 2024 +1100)
* 5fdfd023 - chore: Upgrade all the tracing crates to the latest (Ronald Holshausen, Fri Dec 13 10:54:51 2024 +1100)
* fab32aa0 - chore: Upgrade pact_models to 1.2.6 (Ronald Holshausen, Wed Dec 11 15:28:01 2024 +1100)
* 420c654d - chore: Upgrade pact_models to 1.2.5 (Ronald Holshausen, Wed Nov 13 11:01:39 2024 +1100)
* 791db1fc - bump version to 1.1.5 (Ronald Holshausen, Tue Aug 13 15:48:54 2024 +1000)

# 1.1.4 - Bugfix Release

* fa02c30b - chore: Upgrade pact_verifier to 1.2.4 (Ronald Holshausen, Tue Aug 13 15:42:48 2024 +1000)
* f50454ff - chore: Upgrade test dependencies (Ronald Holshausen, Tue Aug 13 11:54:59 2024 +1000)
* 9a7eef75 - chore: Upgrade pact_models to 1.2.4 (Ronald Holshausen, Tue Aug 13 11:51:06 2024 +1000)
* bcf4db6d - chore: Upgrade pact_models to 1.2.3 (Ronald Holshausen, Tue Aug 6 11:06:22 2024 +1000)
* 4257ed1e - chore: Upgrade pact_verifier to 1.2.3 (Ronald Holshausen, Wed Jul 17 15:19:58 2024 +1000)
* fd9ff70a - chore: Upgrade pact_models to 1.2.2 (Ronald Holshausen, Wed Jul 17 11:15:34 2024 +1000)
* 5ba47b96 - docs(pact_verifier_cli): Update README with returning values from provider state callbacks #441 (Ronald Holshausen, Thu Jul 11 15:52:55 2024 +1000)
* f03cd0d3 - feat(pact_verifier_cli): Parse consumer version selector JSON when evaluating CLI args so errors are reported earlier (Ronald Holshausen, Wed Jul 10 12:13:11 2024 +1000)
* de31b4ea - fix(pact_verifier_cli): Allow transport base-paths to be defined from the CLI #418 (Ronald Holshausen, Mon Jul 8 17:19:37 2024 +1000)
* a5ba0cb8 - doc(pact_verifier_cli): Update Rust docs from README content #418 (Ronald Holshausen, Mon Jul 8 16:37:48 2024 +1000)
* a830e491 - doc(pact_verifier_cli): Fix grammar #418 (Ronald Holshausen, Mon Jul 8 16:27:57 2024 +1000)
* 8cc4c148 - doc(pact_verifier_cli): Update docs for verifying mixed V4 Pacts #418 (Ronald Holshausen, Mon Jul 8 16:26:00 2024 +1000)
* bab814c8 - doc(pact_verifier_cli): Fix grammar #418 (Ronald Holshausen, Mon Jul 8 15:43:21 2024 +1000)
* fbb7825d - doc(pact_verifier_cli): Add some emphasis to example values #418 (Ronald Holshausen, Mon Jul 8 15:41:25 2024 +1000)
* 4775fdfd - doc(pact_verifier_cli): Update the CLI docs to explain the provider state calls #418 (Ronald Holshausen, Mon Jul 8 15:25:23 2024 +1000)
* 37bc38f1 - bump version to 1.1.4 (Ronald Holshausen, Fri Jun 14 11:51:24 2024 +1000)

# 1.1.3 - Feature + Bugfix release

* dad482f5 - chore: Upgrade pact_verifier to 1.2.2 (Ronald Holshausen, Fri Jun 14 11:21:54 2024 +1000)
* 30998a95 - chore: Upgrade pact_models to 1.2.1 (Ronald Holshausen, Wed Jun 12 15:26:49 2024 +1000)
* 582de5c9 - refactor: update pact_ffi/pact_verifier_cli release workflows macos (Yousaf Nabi, Sat Jun 8 02:48:40 2024 +0100)
* c9443652 - refactor: update pact_ffi/pact_verifier_cli release workflows (Yousaf Nabi, Sat Jun 8 02:45:46 2024 +0100)
* 0b759dea - fix(pact_verifier_cli): strip ANSI escapes from JUnit output (Martijn Pieters, Wed May 22 15:12:45 2024 +0100)
* c68a0eea - feat: Add environment variables for most options (Martijn Pieters, Thu May 16 17:09:43 2024 +0100)
* 955bcbdc - bump version to 1.1.3 (Ronald Holshausen, Wed May 8 15:58:06 2024 +1000)

# 1.1.2 - Maintenance Release

* 5a6b004a - chore: Upgrade reqwest to 0.12.4 (Ronald Holshausen, Wed May 8 15:42:28 2024 +1000)
* 2f17632d - chore: Upgrade clap crate to 4.5.4 (Ronald Holshausen, Wed May 8 15:38:21 2024 +1000)
* edc1ad0c - chore: Upgrade pact_verifier to 1.2.1 (Ronald Holshausen, Wed May 8 15:32:36 2024 +1000)
* 694100fb - chore: Update pact_models to 1.2.0 (Ronald Holshausen, Tue Apr 23 10:51:11 2024 +1000)
* edfac7ba - chore: remove local pact_models from the other pact crates (Ronald Holshausen, Tue Apr 23 10:03:29 2024 +1000)
* 7688469a - chore(pact_verifier): Bump minor version (Ronald Holshausen, Tue Apr 16 10:41:18 2024 +1000)
* add35c94 - bump version to 1.1.2 (Ronald Holshausen, Mon Mar 18 11:34:42 2024 +1100)

# 1.1.1 - Maintenance Release

* e08dc260 - chore(pact_verifier_cli): Create multi-arch docker images (Ronald Holshausen, Mon Mar 18 11:27:24 2024 +1100)
* b674c051 - chore(pact_verifier_cli): Create macos named binary files (Ronald Holshausen, Mon Mar 18 11:21:46 2024 +1100)
* ae02dc10 - chore(ci): verifier - fix ref to build_aarch64 func post building with musl only (Yousaf Nabi, Sat Mar 16 09:58:39 2024 +0000)
* 6406e9ae - refactor(release): Rename OSX to MacOS (Ronald Holshausen, Fri Mar 15 14:28:06 2024 +1100)
* a6d68190 - chore(pact_verifier_cli): Only build musl variants of the Linux binaries (Ronald Holshausen, Fri Mar 15 14:11:57 2024 +1100)
* 7732a637 - chore(ci): refactor release pipeline (JP-Ellis, Mon Feb 26 17:54:18 2024 +1100)
* c6da65d3 - Merge branch 'release/1.1.0-docker' (Ronald Holshausen, Mon Feb 19 13:03:38 2024 +1100)
* b697c1fa - fix(pact_verifier_cli): Correct docker file not that pact_models is a relative path (Ronald Holshausen, Mon Feb 19 13:03:24 2024 +1100)
* d5cd18dd - bump version to 1.1.1 (Ronald Holshausen, Mon Feb 19 12:21:48 2024 +1100)

# 1.1.0 - Feature + Bugfix release

* 40391cb0 - chore(pact_verifier_cli): Bump minor version (Ronald Holshausen, Mon Feb 19 12:03:43 2024 +1100)
* ba9eae1e - chore(pact_verifier_cli): Upgrade dependencies (Ronald Holshausen, Mon Feb 19 11:48:13 2024 +1100)
* 82804395 - Revert "chore: Upgrade pact_verifier to 1.1.1" (Ronald Holshausen, Mon Feb 19 11:43:12 2024 +1100)
* d17dac05 - chore: Upgrade pact_verifier to 1.1.1 (Ronald Holshausen, Mon Feb 19 11:42:50 2024 +1100)
* 6975203e - chore(verifier): Bump minor version #307 (Ronald Holshausen, Mon Feb 19 06:15:47 2024 +1100)
* 41faa544 - chore: Lock clap crate to 4.4 as 4.5 requires Rust 1.75 (Ronald Holshausen, Mon Feb 12 15:16:18 2024 +1100)
* 8d197959 - chore: fixup path to aarch64-unknown-linux-musl artifact (Yousaf Nabi, Thu Feb 8 21:13:21 2024 +0000)
* 9196efe4 - feat: verifier/mock_server clis - build for musl (Yousaf Nabi, Thu Feb 8 20:36:10 2024 +0000)
* 24a26cca - chore: Update pact_models to 1.1.18 (Ronald Holshausen, Wed Feb 7 10:53:22 2024 +1100)
* 73578350 - chore: use local pact_models (JP-Ellis, Tue Feb 6 10:51:09 2024 +1100)
* f49e5d5a - bump version to 1.0.4 (Ronald Holshausen, Sat Jan 20 21:40:34 2024 +1100)

# 1.0.3 - Bugfix Release

* 8f1ea28d - chore: Upgrade dependencies (Ronald Holshausen, Sat Jan 20 21:35:41 2024 +1100)
* 8d3f146a - chore: Upgrade pact_verifier to 1.0.5 (Ronald Holshausen, Sat Jan 20 21:31:51 2024 +1100)
* 6fa097a5 - Merge branch 'master' into ci/cargo_clean (Ronald Holshausen, Thu Jan 25 10:22:25 2024 +1100)
* 36390097 - Merge pull request #371 from YOU54F/chore/renable_verifier_aarch64_linux (Ronald Holshausen, Thu Jan 25 09:59:53 2024 +1100)
* 00402ba4 - chore: migrate target/artifacts to release_artifacts to avoid cargo clean loss (Yousaf Nabi, Wed Jan 24 15:26:18 2024 +0000)
* 22331310 - ci: cargo clean prior to cross build (Yousaf Nabi, Wed Jan 24 15:24:10 2024 +0000)
* 6e422105 - chore: renable aarch64 linux verifier cli (Yousaf Nabi, Wed Jan 24 12:07:04 2024 +0000)
* c412829f - feat: build aarch64-pc-windows-msvc target (ffi/mock_server/verifier) (Yousaf Nabi, Wed Jan 24 12:01:50 2024 +0000)
* a2ba1cd5 - fix: pin cross to 0.2.5 for glibc 2.23 (Yousaf Nabi, Wed Jan 24 11:54:22 2024 +0000)
* b735df9d - chore: Upgrade pact_models to 1.1.17 (Ronald Holshausen, Sat Jan 20 13:54:03 2024 +1100)
* 944613df - fix: regression - upgrade pact_models to 1.1.16 #359 (Ronald Holshausen, Fri Jan 19 14:52:36 2024 +1100)
* 403c0af1 - chore: Upgrade pact_models to 1.1.14 #355 (Ronald Holshausen, Tue Jan 16 10:31:12 2024 +1100)
* dfd13760 - chore: Upgrade pact_models to 1.1.13 #355 (Ronald Holshausen, Tue Jan 16 07:42:33 2024 +1100)
* 2e7cbf6a - bump version to 1.0.3 (Ronald Holshausen, Wed Dec 20 15:14:24 2023 +1100)

# 1.0.2 - Maintenance Release

* 75a5d1ab - chore: Upgrade all dependencies (Ronald Holshausen, Wed Dec 20 15:05:43 2023 +1100)
* 2ff2d873 - chore: Update readme with --webhook-callback-url option #350 (Ronald Holshausen, Wed Dec 20 15:01:14 2023 +1100)
* 39a14e97 - chore: Upgrade pact_verifier to 1.0.4 (Ronald Holshausen, Wed Dec 20 14:46:57 2023 +1100)
* 7a170fa5 - feat(verifier): Add option to only verify pact from a webhook URL #350 (Ronald Holshausen, Wed Dec 20 11:54:48 2023 +1100)
* 826758a6 - chore: Upgrade pact_models to 1.1.12 (Ronald Holshausen, Mon Nov 13 17:25:21 2023 +1100)
* c7dbcd88 - chore(pact_verifier_cli): Disable aarch64-unknown-linux-gnu target from release build (Ronald Holshausen, Tue Aug 29 11:34:23 2023 +1000)
* 90345760 - chore(pact_verifier_cli): Upgrade docker image to use Alpine 3.18 (Ronald Holshausen, Tue Aug 29 10:57:52 2023 +1000)
* 417cb806 - bump version to 1.0.2 (Ronald Holshausen, Tue Aug 29 10:39:04 2023 +1000)
* 72ac6f86 - update changelog for release 1.0.1 (Ronald Holshausen, Tue Aug 29 10:37:32 2023 +1000)
* c924b9ce - chore: Upgrade pact_verifier to 1.0.3 (Ronald Holshausen, Tue Aug 29 10:04:18 2023 +1000)
* e4da3e42 - chore: Upgrade pact_models to 1.1.11 (Ronald Holshausen, Mon Aug 7 13:59:34 2023 +1000)
* 24ed7835 - chore: Upgrade pact-models to 1.1.10 (Ronald Holshausen, Fri Aug 4 16:11:24 2023 +1000)
* 04af3923 - chore: Upgrade pact_verifier to 1.0.2 (Ronald Holshausen, Thu Jul 27 15:17:57 2023 +1000)
* 4a01919a - chore: Upgrade pact_models to 1.1.9 (Ronald Holshausen, Thu Jul 27 10:24:00 2023 +1000)
* 64a862ee - chore:(pact_verifier_cli): correct linux release script (Ronald Holshausen, Wed Jul 12 13:49:34 2023 +1000)
* 87017434 - bump version to 1.0.1 (Ronald Holshausen, Wed Jul 12 12:07:52 2023 +1000)

# 1.0.1 - Maintenance Release

* c924b9ce - chore: Upgrade pact_verifier to 1.0.3 (Ronald Holshausen, Tue Aug 29 10:04:18 2023 +1000)
* e4da3e42 - chore: Upgrade pact_models to 1.1.11 (Ronald Holshausen, Mon Aug 7 13:59:34 2023 +1000)
* 24ed7835 - chore: Upgrade pact-models to 1.1.10 (Ronald Holshausen, Fri Aug 4 16:11:24 2023 +1000)
* 04af3923 - chore: Upgrade pact_verifier to 1.0.2 (Ronald Holshausen, Thu Jul 27 15:17:57 2023 +1000)
* 4a01919a - chore: Upgrade pact_models to 1.1.9 (Ronald Holshausen, Thu Jul 27 10:24:00 2023 +1000)
* 64a862ee - chore:(pact_verifier_cli): correct linux release script (Ronald Holshausen, Wed Jul 12 13:49:34 2023 +1000)
* 87017434 - bump version to 1.0.1 (Ronald Holshausen, Wed Jul 12 12:07:52 2023 +1000)

# 1.0.0 - 1.0.0 Release

* b9e034b2 - feat: Add crate feature for JUnit report output (Ronald Holshausen, Wed Jul 12 11:50:08 2023 +1000)
* 7a25a819 - chore: Bump pact_verifier_cli version to 1.0.0 (Ronald Holshausen, Wed Jul 12 11:36:21 2023 +1000)
* 63c1a8e6 - chore: Upgrade pact_verifier to 1.0.1 (Ronald Holshausen, Wed Jul 12 11:27:36 2023 +1000)
* 1deca59a - chore: Upgrade pact_models to 1.1.8 (Ronald Holshausen, Mon Jul 10 16:15:43 2023 +1000)
* 2662cdfc - chore: Upgrade pact_models to 1.1.7 (Ronald Holshausen, Thu Jul 6 10:27:25 2023 +1000)
* e95ae4d0 - chore: Upgrade pact_models to 1.1.6 (Ronald Holshausen, Thu Jun 22 15:40:55 2023 +1000)
* bc68ed7f - chore: Upgrade pact_models to 1.1.4 (Ronald Holshausen, Thu Jun 1 10:22:38 2023 +1000)
* 397c837f - chore: Upgrade pact_models to 1.1.3 (fixes MockServerURL generator) (Ronald Holshausen, Mon May 29 15:12:22 2023 +1000)
* ca1e37fb - chore: Upgrade pact_verifier to 1.0.0 (Ronald Holshausen, Tue May 23 15:16:08 2023 +1000)
* ac2e24da - chore: Use "Minimum version, with restricted compatibility range" for all Pact crate versions (Ronald Holshausen, Tue May 23 11:46:52 2023 +1000)
* 261ecf47 - fix: Add RefUnwindSafe trait bound to all Pact and Interaction uses (Ronald Holshausen, Mon May 15 13:59:31 2023 +1000)
* 91cceabf - bump version to 0.10.7 (Ronald Holshausen, Tue Apr 18 15:26:13 2023 +1000)

# 0.10.6 - Bugfix Release

* 8cfc3d79 - chore: Upgrade pact_verifier to 0.15.3 (Ronald Holshausen, Tue Apr 18 15:22:26 2023 +1000)
* 6c14abfd - chore: Upgrade pact_models to 1.0.13 (Ronald Holshausen, Tue Apr 18 13:00:01 2023 +1000)
* 10bf1a48 - chore: Upgrade pact_models to 1.0.12 (fixes generators hash function) (Ronald Holshausen, Mon Apr 17 10:31:09 2023 +1000)
* 84b9d9e9 - fix: Upgrade pact models to 1.0.11 (fixes generated key for V4 Pacts) (Ronald Holshausen, Fri Apr 14 17:10:58 2023 +1000)
* 669f7812 - chore: Upgrade pact_models to 1.0.10 (Ronald Holshausen, Thu Apr 13 15:32:34 2023 +1000)
* f35fff9a - bump version to 0.10.6 (Ronald Holshausen, Wed Apr 5 10:44:52 2023 +1000)

# 0.10.5 - Maintenance Release

* 6c9351d3 - chore: Update dependencies (Ronald Holshausen, Wed Apr 5 10:30:10 2023 +1000)
* 5981539a - chore: Add trycmd tests (Ronald Holshausen, Wed Apr 5 10:24:28 2023 +1000)
* 4f62ee5d - chore: Upgrade pact_verifier to 0.15.2 (Ronald Holshausen, Wed Apr 5 10:16:07 2023 +1000)
* 40329d43 - bump version to 0.10.5 (Ronald Holshausen, Wed Mar 15 15:29:58 2023 +1100)

# 0.10.4 - Bugfix Release

* 8310b09c - chore: Upgrade pact_verifier to 0.15.1 (Ronald Holshausen, Wed Mar 15 15:25:31 2023 +1100)
* e96bc54e - fix: Upgrade pact_models to 1.0.9 (fixes issues with headers) (Ronald Holshausen, Wed Mar 15 14:31:00 2023 +1100)
* f7e0b669 - chore: Upgrade pact_models to 1.0.8 (Ronald Holshausen, Wed Mar 15 12:19:22 2023 +1100)
* f63b7b66 - bump version to 0.10.4 (Ronald Holshausen, Thu Mar 2 13:03:18 2023 +1100)

# 0.10.3 - Add option to generate a JUnit XML report file for the verification #257

* 9629c351 - chore: dump minor version of pact_verifier as some signatures have changed (Ronald Holshausen, Thu Mar 2 12:10:35 2023 +1100)
* c9333f94 - feat: add option to generate JUnit XML report format for consumption by CI servers #257 (Ronald Holshausen, Thu Mar 2 10:48:56 2023 +1100)
* 46297622 - feat: add verification timing to the verifier output (Ronald Holshausen, Mon Feb 27 16:11:18 2023 +1100)
* cad9208a - bump version to 0.10.3 (Ronald Holshausen, Thu Feb 16 14:20:45 2023 +1100)

# 0.10.2 - Bugfix Release

* c368c651 - fix: Pass any custom header values on to the plugin verification call (Ronald Holshausen, Thu Feb 16 13:52:03 2023 +1100)
* a62bb14f - chore: correct changelog (Ronald Holshausen, Fri Feb 10 14:16:35 2023 +1100)
* 88f263d0 - bump version to 0.10.2 (Ronald Holshausen, Fri Feb 10 14:11:17 2023 +1100)

# 0.10.1 - Maintenance Release (supports message metadata)

* fa45296c - chore: Update pact_verifier to 0.13.21 (Ronald Holshausen, Fri Feb 10 13:37:48 2023 +1100)
* e6bf4aad - bump version to 0.10.1 (Ronald Holshausen, Wed Jan 11 16:12:48 2023 +1100)

# 0.10.0 - Bugfix Release

* fbc4dbe1 - chore: Upgrade pact_verifier to 0.13.20 (Ronald Holshausen, Wed Jan 11 16:06:21 2023 +1100)
* 7d84d941 - chore: Upgrade pact_models to 1.0.4 (Ronald Holshausen, Wed Jan 11 14:33:13 2023 +1100)
* 61450a63 - chore: bump minor version (Ronald Holshausen, Tue Jan 10 15:51:15 2023 +1100)
* a4fc8dc7 - chore: re-organise the CLI options into groups (Ronald Holshausen, Tue Jan 10 15:49:58 2023 +1100)
* 14e3b8ff - feat: Add short option for custom headers (Ronald Holshausen, Tue Jan 10 15:36:25 2023 +1100)
* f49462fd - chore: Upgrade Clap to V4 (Ronald Holshausen, Tue Jan 10 15:32:19 2023 +1100)
* 9694c694 - chore: Resolve all Clap deprecation warnings in preperation to upgrade to Clap 4 (Ronald Holshausen, Tue Jan 10 14:06:38 2023 +1100)
* 53a622ad - feat: Add options for compact and human formats to the verifier CLI log output (Ronald Holshausen, Mon Jan 9 18:07:19 2023 +1100)
* 1bdb1054 - chore: Upgrade pact_models to 1.0.3 #239 (Ronald Holshausen, Thu Dec 22 15:37:53 2022 +1100)
* 2d7367fc - bump version to 0.9.21 (Ronald Holshausen, Mon Dec 19 16:44:00 2022 +1100)

# 0.9.20 - Add user-agent header + Support generators in plugins

* c55a7758 - chore: Upgrade pact_verifier to 0.13.19 (Ronald Holshausen, Mon Dec 19 16:20:24 2022 +1100)
* 46254545 - chore: Upgrade pact_verifier to 0.13.18 (Ronald Holshausen, Wed Dec 14 17:15:22 2022 +1100)
* e7a1b9f2 - chore: Upgrade pact_matching to 1.0 and plugin driver to 0.2 (Ronald Holshausen, Fri Dec 9 17:29:33 2022 +1100)
* f410fe35 - bump version to 0.9.20 (Ronald Holshausen, Mon Nov 28 15:18:13 2022 +1100)

# 0.9.19 - Bugfix Release

* 2f0ada6b - chore: Upgrade pact_verifier to 0.13.16 (Ronald Holshausen, Mon Nov 28 15:08:47 2022 +1100)
* c9721fd5 - chore: Upgrade pact_models to 1.0.1 and pact-plugin-driver to 0.1.16 (Ronald Holshausen, Mon Nov 28 14:10:53 2022 +1100)
* 4411d487 - chore: Upgrade docker container to latest Alpine (Ronald Holshausen, Mon Nov 7 14:47:36 2022 +1100)
* e2fa8a27 - bump version to 0.9.19 (Ronald Holshausen, Mon Nov 7 14:37:28 2022 +1100)

# 0.9.18 - Maintenance Release

* 6c58858e - chore: Upgrade dependencies (Ronald Holshausen, Mon Nov 7 14:33:24 2022 +1100)
* f43e7851 - chore: Upgrade pact_verifier to 0.13.15 (Ronald Holshausen, Mon Nov 7 14:13:26 2022 +1100)
* 577824e7 - fix: Upgrade pact_models to 1.0 and pact-plugin-driver to 0.1.15 to fix cyclic dependency issue (Ronald Holshausen, Mon Nov 7 11:14:20 2022 +1100)
* e1f985ad - chore: Upgrade pact_models to 0.4.6 and pact-plugin-driver to 0.1.14 (Ronald Holshausen, Fri Nov 4 16:38:36 2022 +1100)
* cb984375 - bump version to 0.9.18 (Ronald Holshausen, Wed Sep 28 11:02:59 2022 +1000)

# 0.9.17 - Bugfix Release

* b7bb9cd1 - chore: Upgrade pact_verifier crate to 0.13.14 (Ronald Holshausen, Wed Sep 28 10:34:48 2022 +1000)
* b626002c - fix(pact_verifier_cli): stop using deprecated clap::parser::matches::arg_matches::ArgMatches::values_of_lossy (Jerry Wang, Sat Sep 24 01:36:05 2022 -0700)
* ac4fe73f - chore: fix to release scripts (Ronald Holshausen, Wed Sep 7 10:51:01 2022 +1000)
* 83b5ee79 - bump version to 0.9.17 (Ronald Holshausen, Wed Sep 7 09:56:33 2022 +1000)

# 0.9.16 - Bugfix Release

* cdb555f8 - fix: Upgrade pact_verifier to 0.13.13 (Ronald Holshausen, Wed Sep 7 09:53:05 2022 +1000)
* 2e7563d1 - bump version to 0.9.16 (Ronald Holshausen, Wed Aug 31 16:19:21 2022 +1000)

# 0.9.15 - Maintenance Release

* 5c1d4293 - chore: Upgrade pact_verifier crate to 0.13.12 (Ronald Holshausen, Wed Aug 31 16:09:16 2022 +1000)
* 8663cd3f - feat: add ignore-no-pacts-error to the verifier CLI #213 (Ronald Holshausen, Wed Aug 31 15:19:31 2022 +1000)
* f8db90d2 - fix: Upgrade pact_models to 0.4.5 - fixes FFI bug with generators for request paths (Ronald Holshausen, Fri Aug 26 11:44:08 2022 +1000)
* 43be2e83 - feat: do not output an error if no_pacts_is_error is false and no pacts were found to verify #213 (Ronald Holshausen, Fri Aug 19 16:49:19 2022 +1000)
* 5e52d685 - chore: Upgrade pact_verifier to 0.13.11 (Ronald Holshausen, Thu Aug 18 16:33:19 2022 +1000)
* 32a70382 - chore: Upgrade pact_models (0.4.4), plugin driver (0.1.10), tracing and tracing core crates (Ronald Holshausen, Thu Aug 18 14:41:52 2022 +1000)
* 25f396ae - chore: add missing SHA files and OSX aarch64 binary to the Verifier release build #160 (Ronald Holshausen, Mon Aug 15 16:54:03 2022 +1000)
* b462bd2e - bump version to 0.9.15 (Ronald Holshausen, Mon Aug 15 16:23:45 2022 +1000)

# 0.9.14 - Support aarch64 Linux binary

* e3bef155 - feat: Add ARM64 (aarch64) linux targets to the release build #160 (Ronald Holshausen, Mon Aug 15 16:13:22 2022 +1000)
* bcea9444 - chore: docker build requires protobuf dev files (Ronald Holshausen, Wed Aug 10 13:44:41 2022 +1000)
* 6105b7a5 - bump version to 0.9.14 (Ronald Holshausen, Wed Aug 10 13:22:20 2022 +1000)

# 0.9.13 - add CLI options to provide different ports when there are different transports

* ac58f50f - chore: update readme (Ronald Holshausen, Wed Aug 10 13:18:15 2022 +1000)
* 78ff94e2 - chore: cleanup some deprecation warnings (Ronald Holshausen, Wed Aug 10 13:07:51 2022 +1000)
* 3324c1b3 - chore: Upgrade pact_verifier to 0.13.10 (Ronald Holshausen, Wed Aug 10 13:02:17 2022 +1000)
* a3fe5e7f - chore: Update pact models to 0.4.2 (Ronald Holshausen, Wed Aug 10 10:10:41 2022 +1000)
* 3a1449cb - feat: use the configured transport when provided (Ronald Holshausen, Wed Aug 3 13:20:17 2022 +1000)
* 8cc29482 - feat: add CLI options to provide different ports when there are different transports (Ronald Holshausen, Wed Aug 3 11:53:31 2022 +1000)
* 6117aa50 - chore: upgrade clap crate to 3.x (Ronald Holshausen, Mon Aug 1 14:46:56 2022 +1000)
* 5f487571 - bump version to 0.9.13 (Ronald Holshausen, Wed Jul 20 13:21:27 2022 +1000)

# 0.9.12 - add --no-color option to verfier CLI

* 701c93a6 - Merge pull request #204 from pact-foundation/snyk-fix-e3dc7fb516c9ab76c1050c65ab20c6fb (Ronald Holshausen, Tue Jul 19 22:47:31 2022 -0400)
* 4530dbde - feat: add --no-color option to verfier CLI #203 (Ronald Holshausen, Wed Jul 20 12:45:20 2022 +1000)
* 2b808db7 - chore: Update pact_verifier to 0.13.9 (Ronald Holshausen, Wed Jul 20 12:44:24 2022 +1000)
* 05e6399d - fix(pact_verifier_cli): log entries were being duplicated (Ronald Holshausen, Wed Jul 20 10:45:13 2022 +1000)
* c7f68871 - fix: rust/pact_verifier_cli/Dockerfile to reduce vulnerabilities (snyk-bot, Wed Jul 6 23:50:14 2022 +0000)
* 6d5d830f - bump version to 0.9.12 (Ronald Holshausen, Tue Jun 7 12:17:10 2022 +1000)

# 0.9.11 - Bug fixes + Support publishing results from webhook calls

* 731477f8 - chore: prep for release (Ronald Holshausen, Tue Jun 7 11:19:13 2022 +1000)
* b3f98a2c - chore: Upgrade pact_verifier to 0.13.8 (Ronald Holshausen, Tue Jun 7 11:07:24 2022 +1000)
* 18118e82 - feat: add retries to the provider state change calls #197 (Ronald Holshausen, Tue Jun 7 09:10:23 2022 +1000)
* 6cae9b09 - fix: State change descriptions were not being displayed along with the interaction description (Ronald Holshausen, Mon Jun 6 17:09:44 2022 +1000)
* 61fc3771 - chore: Upgrade pact_verifier to 0.13.7 (Ronald Holshausen, Mon May 30 12:21:12 2022 +1000)
* f8471bb7 - chore: switch from log crate to tracing crate (Ronald Holshausen, Fri May 13 13:47:18 2022 +1000)
* ee9d6bab - chore: Upgrade pact_verifier to 0.13.6 (Ronald Holshausen, Wed May 11 17:40:15 2022 +1000)
* 020b5715 - chore: upgrade pact_models to 0.4.1 (Ronald Holshausen, Wed May 11 11:36:57 2022 +1000)
* e1f4d4d9 - bump version to 0.9.11 (Ronald Holshausen, Wed Apr 27 16:09:01 2022 +1000)

# 0.9.10 - Supports verification via plugins

* 8d58ea34 - fix: lock the pact crate versions so that updates do not break CLI install #189 (Ronald Holshausen, Wed Apr 27 16:01:54 2022 +1000)
* 14a010a9 - chore: Upgrade pact_verifier to 0.13.5 (Ronald Holshausen, Wed Apr 27 15:21:15 2022 +1000)
* cdf72b05 - feat: forward provider details to plugin when verifying (Ronald Holshausen, Fri Apr 22 14:12:34 2022 +1000)
* 2395143a - feat: forward verification to plugin for transports provided by the plugin (Ronald Holshausen, Fri Apr 22 12:02:05 2022 +1000)
* 05c83b67 - chore: switch verifier over to tracing crate (Ronald Holshausen, Wed Apr 20 11:34:16 2022 +1000)
* 75145a60 - chore: setup tracing for verifier CLI (Ronald Holshausen, Tue Apr 19 17:20:18 2022 +1000)
* 763488c4 - refactor: rename scheme parameter to transport (Ronald Holshausen, Tue Apr 19 17:03:30 2022 +1000)
* 8815ec0e - bump version to 0.9.10 (Ronald Holshausen, Wed Apr 13 16:12:36 2022 +1000)

# 0.9.9 - Bugfix Release

* 136c8a82 - chore: Upgrade pact_verifier to 0.13.4 (Ronald Holshausen, Wed Apr 13 16:06:02 2022 +1000)
* 49640c5f - chore: minor update to release scripts (Ronald Holshausen, Wed Apr 13 15:32:46 2022 +1000)
* d043f6c7 - chore: upgrade pact_models to 0.3.3 (Ronald Holshausen, Wed Apr 13 15:24:33 2022 +1000)
* 73ae0ef0 - fix: Upgrade reqwest to 0.11.10 to resolve #156 (Ronald Holshausen, Wed Apr 13 13:31:55 2022 +1000)
* 776265ee - chore: bump pact_verifier to 0.13.3 (Ronald Holshausen, Thu Mar 24 15:05:01 2022 +1100)
* 345b0011 - feat: support mock servers provided from plugins (Ronald Holshausen, Mon Mar 21 15:59:46 2022 +1100)
* f709528d - fix: rust/pact_verifier_cli/Dockerfile to reduce vulnerabilities (snyk-bot, Thu Mar 17 22:19:46 2022 +0000)
* a09fade9 - bump version to 0.9.9 (Ronald Holshausen, Fri Mar 4 15:22:53 2022 +1100)

# 0.9.8 - Custom headers + Date-Time expression parser

* b6433500 - chore: upgrade pact_verifier to 0.13.2 (Ronald Holshausen, Fri Mar 4 14:49:18 2022 +1100)
* 8e864502 - chore: update all dependencies (Ronald Holshausen, Fri Mar 4 13:29:59 2022 +1100)
* 79324802 - feat: add support for custom headers via the verifier CLI #182 (Ronald Holshausen, Mon Feb 28 15:22:47 2022 +1100)
* 74bd4531 - feat: add support for custom headers with the verifier FFI calls #182 (Ronald Holshausen, Mon Feb 28 13:58:46 2022 +1100)
* eda9fc19 - chore: build verifier docker image using Rust base image (Ronald Holshausen, Mon Jan 31 13:46:58 2022 +1100)
* 4a17ea36 - bump version to 0.9.8 (Ronald Holshausen, Mon Jan 31 13:05:31 2022 +1100)

# 0.9.7 - Bugfixes + added JSON report option

* 4b2556fa - chore: update readme/docs with new json option (Ronald Holshausen, Mon Jan 31 11:32:02 2022 +1100)
* 5ecf70a7 - feat: enable ANSI console output on Windows (Ronald Holshausen, Mon Jan 31 11:02:03 2022 +1100)
* d0fa29dc - feat: add json output to the verifier CLI (Ronald Holshausen, Fri Jan 28 15:21:17 2022 +1100)
* bf152233 - feat: Capture all the results from the verification process (Ronald Holshausen, Fri Jan 28 11:28:38 2022 +1100)
* 5f148cdd - feat: capture all the output from the verifier (Ronald Holshausen, Thu Jan 27 16:08:02 2022 +1100)
* f5aa34ea - Merge pull request #175 from pact-foundation/feat/fix-provider-timeout-value-validation (Ronald Holshausen, Thu Jan 27 13:41:56 2022 +1100)
* 0ef3fb98 - fix: provider request timeout should be > 16bit integers. Fixes https://github.com/pact-foundation/pact-js/issues/761 (Matt Fellows, Wed Jan 26 22:12:35 2022 +1100)
* 8bee40b0 - feat(ffi)!: Separate verification and publishing options (Adam Rodger, Tue Jan 25 16:31:29 2022 +0000)
* d1bdd132 - chore: use docker builder image that supports Rust 2021 (Ronald Holshausen, Tue Jan 25 11:46:54 2022 +1100)
* 60afcc60 - bump version to 0.9.7 (Ronald Holshausen, Tue Jan 25 10:59:29 2022 +1100)

# 0.9.6 - Maintenance Release

* 0c200ea5 - chore: Upgrade pact verifier crate to 0.12.4 (Ronald Holshausen, Mon Jan 17 17:07:18 2022 +1100)
* 4f1ecff2 - chore: Upgrade pact-models to 0.2.7 (Ronald Holshausen, Mon Jan 17 10:53:26 2022 +1100)
* c2089645 - fix: log crate version must be fixed across all crates (including plugin driver) (Ronald Holshausen, Fri Jan 14 16:10:50 2022 +1100)
* 5479a634 - chore: Update pact_models (0.2.4) and pact-plugin-driver (0.0.14) (Ronald Holshausen, Thu Dec 23 12:57:02 2021 +1100)
* c4ff44a2 - bump version to 0.9.6 (Matt Fellows, Fri Dec 17 16:22:36 2021 +1100)
* 0110fac3 - update changelog for release 0.9.5 (Matt Fellows, Fri Dec 17 16:16:53 2021 +1100)

# 0.9.5 - Bugfix Release


# 0.9.5 - Bugfix Release

* c97f5d1a - fix: shutdown the tokio reactor correctly when there is an error (Ronald Holshausen, Wed Dec 15 16:28:37 2021 +1100)
* 7c31d981 - bump version to 0.9.5 (Ronald Holshausen, Wed Dec 15 15:59:05 2021 +1100)

# 0.9.4 - Bugfix Release

* 00a00461 - fix: add a small delay at the end of validation to allow async tasks to finish (Ronald Holshausen, Wed Dec 15 15:37:30 2021 +1100)
* d26fa4c5 - bump version to 0.9.4 (Ronald Holshausen, Wed Dec 15 13:56:30 2021 +1100)

# 0.9.3 - Add metrics for provider verification

* f8042d6b - feat: add metrics event for provider verification (Ronald Holshausen, Tue Dec 14 17:29:44 2021 +1100)
* 01171ccb - bump version to 0.9.3 (Ronald Holshausen, Thu Dec 2 12:32:36 2021 +1100)

# 0.9.2 - Bugfix Release

* 491e9259 - chore(pact_verifier_cli): upgrade to latest models crate (Ronald Holshausen, Thu Dec 2 12:22:11 2021 +1100)
* 51a147df - chore: fix docker file (Ronald Holshausen, Tue Nov 16 13:56:24 2021 +1100)
* 2780c93b - bump version to 0.9.2 (Ronald Holshausen, Tue Nov 16 13:18:59 2021 +1100)

# 0.9.1 - Fix for branches and consumer version selectors

* 5d974c4a - chore: update to latest models and plugin driver crates (Ronald Holshausen, Tue Nov 16 11:56:53 2021 +1100)
* df23ba3d - fix: allow multiple consumer version selectors (Matt Fellows, Mon Nov 15 14:28:04 2021 +1100)
* 0af18303 - fix: add missing provider-branch to verifier CLI (Ronald Holshausen, Mon Nov 8 11:40:05 2021 +1100)
* 2db1e1bb - bump version to 0.9.1 (Ronald Holshausen, Thu Nov 4 16:44:12 2021 +1100)

# 0.9.0 - Pact V4 release

* 8d05ddcc - chore: remove beta version from verifier cli (Ronald Holshausen, Thu Nov 4 16:25:02 2021 +1100)
* 400a1231 - chore: drop beta from pact_verifier version (Ronald Holshausen, Thu Nov 4 15:56:22 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* a561f883 - chore: use the non-beta models crate (Ronald Holshausen, Thu Oct 21 18:10:27 2021 +1100)
* 0c72c80e - chore: fixes after merging from master (Ronald Holshausen, Wed Oct 20 14:46:54 2021 +1100)
* ec265d83 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Oct 20 14:40:37 2021 +1100)
* 87944c79 - bump version to 0.9.0-beta.1 (Ronald Holshausen, Tue Oct 19 18:25:48 2021 +1100)
* 1ce39437 - docs: update verifier CLI docs with consumer version selectors (Matt Fellows, Tue Oct 12 13:26:20 2021 +1100)

# 0.9.0-beta.0 - Pact Plugins Support

* 1aa21870 - chore: update readme with details on plugins (Ronald Holshausen, Tue Oct 19 18:12:51 2021 +1100)
* 5bbdbcfa - refactor: move the CLI functions back from the FFI crate (Ronald Holshausen, Tue Oct 19 18:03:29 2021 +1100)
* e98a91fe - chore: update to latest verifier lib (Ronald Holshausen, Tue Oct 19 17:42:07 2021 +1100)
* 918e5beb - fix: update to latest models and plugin driver crates (Ronald Holshausen, Tue Oct 19 17:09:48 2021 +1100)
* 6f20282d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 28 14:51:34 2021 +1000)
* f14a02b2 - bump version to 0.8.9 (Ronald Holshausen, Tue Sep 28 14:20:41 2021 +1000)
* 75e13fd8 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Aug 23 10:33:45 2021 +1000)
* dfe3cd42 - chore: bump minor version of Pact verifier libs (Ronald Holshausen, Mon Aug 9 15:10:47 2021 +1000)

# 0.8.8 - support native TLS certs

* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 4458a677 - bump version to 0.8.8 (Ronald Holshausen, Sun Aug 22 16:03:00 2021 +1000)

# 0.8.7 - Bugfix Release

* 38ccd5f6 - bump version to 0.8.7 (Ronald Holshausen, Wed Jul 21 13:38:53 2021 +1000)

# 0.8.6 - Bugfix Release

* b3a6f193 - chore: rename header PACT_MESSAGE_METADATA -> Pact-Message-Metadata (Matt Fellows, Tue Jul 13 11:32:25 2021 +1000)
* 0d5ec68a - feat: copied verfier_ffi crate to pact_ffi (Ronald Holshausen, Sat Jul 10 16:54:28 2021 +1000)
* ac9a657d - chore: updated verifier readme about base64 encoded headers (Matt Fellows, Tue Jul 6 09:17:58 2021 +1000)
* a835e684 - feat: support message metadata in verifications (Matt Fellows, Sun Jul 4 21:02:35 2021 +1000)
* e8d6d844 - fix: pact_verifier_cli was printing the version from the FFI crate (Ronald Holshausen, Sat Jun 5 14:43:38 2021 +1000)
* 2f678213 - feat: initial prototype of a pact file verifier (Ronald Holshausen, Thu Jun 3 14:56:16 2021 +1000)
* 913b7b17 - chore: correct CLI docker release files (Ronald Holshausen, Tue Jun 1 11:25:28 2021 +1000)
* 47046ef5 - bump version to 0.8.6 (Ronald Holshausen, Sun May 30 18:52:34 2021 +1000)

# 0.8.5 - V4 features + updated Tokio to 1.0

* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* 9eb107a - Revert "Revert "chore: bump version to 0.0.1"" (Ronald Holshausen, Tue Jan 5 17:25:37 2021 +1100)
* 4b4d4a8 - Revert "chore: bump version to 0.0.1" (Ronald Holshausen, Tue Jan 5 17:11:54 2021 +1100)
* 0a210bb - chore: bump version to 0.0.1 (Ronald Holshausen, Tue Jan 5 16:57:47 2021 +1100)
* 2ebeef9 - fix: pact_verifier_cli needs to use Tokio 0.2 (Ronald Holshausen, Tue Jan 5 16:24:29 2021 +1100)
* d9f0e8b - refactor: split pact_verifier ffi functions into seperate crate (Ronald Holshausen, Tue Jan 5 16:17:46 2021 +1100)
* c9e0694 - Revert "Revert "bump version to 0.8.5"" (Ronald Holshausen, Tue Jan 5 15:37:25 2021 +1100)
* 1a4b9a5 - chore: correct the pact_verifier_cli windows release script (Ronald Holshausen, Tue Jan 5 15:36:58 2021 +1100)

# 0.8.4 - TLS support + FFI support

* 41096dc - chore: update release scripts for pact_verifier_cli DLLs (Ronald Holshausen, Tue Jan 5 14:34:55 2021 +1100)
* ef76f38 - chore: cleanup compiler warnings (Ronald Holshausen, Tue Jan 5 10:10:39 2021 +1100)
* 484b747 - fix: verify interaction was blocking the thread (Ronald Holshausen, Mon Jan 4 17:12:38 2021 +1100)
* 4c4eb85 - chore: bump minor version of pact_verifier crate due to breaking changes (Ronald Holshausen, Mon Jan 4 15:48:41 2021 +1100)
* b583540 - Merge branch 'master' into feat/allow-invalid-certs-during-verification (Matt Fellows, Fri Jan 1 14:22:10 2021 +1100)
* 6cec6c7 - feat: allow https scheme and ability to disable ssl verification (Matt Fellows, Thu Dec 31 12:10:57 2020 +1100)
* 79f62ce - Merge branch 'master' into feat/add-verifier-ffi (Matt Fellows, Wed Dec 30 23:21:12 2020 +1100)
* 8aeb567 - wip: minor updates to get FFI interface working (Matt Fellows, Tue Dec 1 19:12:53 2020 +1100)
* c71c78d - wip: add verifier FFI bindings (Matt Fellows, Tue Dec 1 07:04:48 2020 +1100)
* a480e76 - bump version to 0.8.4 (Matt Fellows, Tue Nov 24 11:06:22 2020 +1100)

# 0.8.3 - Bugfix Release

* 280c066 - bump version to 0.8.3 (Matt Fellows, Wed Nov 11 13:30:12 2020 +1100)

# 0.8.2 - Support Pacts for Verification API

* 087fee2 - docs: update verifier docs with new pacts for verification properties (Matt Fellows, Wed Nov 11 10:16:57 2020 +1100)
* e7f729d - wip: further cleanup, and obfuscate auth details (Matt Fellows, Tue Nov 10 13:56:02 2020 +1100)
* ada3667 - wip: cleanup verifier args (Matt Fellows, Tue Nov 10 08:13:01 2020 +1100)
* 80f4e98 - wip: refactor BrokerWithDynamicConfiguration into a struct enum for better readability (Matt Fellows, Mon Nov 9 22:40:24 2020 +1100)
* 60c1671 - wip: thread verification context into pact fetching/verification, add env vars to clap args (Matt Fellows, Sun Nov 8 13:25:17 2020 +1100)
* 60eb190 - wip: map tags to consumer version selectors (Matt Fellows, Sat Nov 7 23:35:36 2020 +1100)
* 6612a3a - wip: basic wiring in of the pacts for verification endpoint (Matt Fellows, Sat Nov 7 21:39:25 2020 +1100)
* 33864a5 - bump version to 0.8.2 (Ronald Holshausen, Fri Oct 16 12:40:37 2020 +1100)

# 0.8.1 - arrayContains matcher + text/xml content type

* 7fbc731 - chore: bump minor version of matching lib (Ronald Holshausen, Fri Oct 9 10:42:33 2020 +1100)
* c2fda15 - chore: update readme on verifying message pacts (Ronald Holshausen, Tue Sep 15 11:13:16 2020 +1000)
* 0dbcda9 - bump version to 0.8.1 (Ronald Holshausen, Mon Sep 14 17:34:25 2020 +1000)

# 0.8.0 - Supports verifying Message Pacts

* ef5f88c - chore: bump minor version of the pact_verifier crate (Ronald Holshausen, Mon Sep 14 17:13:45 2020 +1000)
* 2d44ffd - chore: bump minor version of the matching crate (Ronald Holshausen, Mon Sep 14 12:06:37 2020 +1000)
* fb6c19c - refactor: allow verifier to handle different types of interactions (Ronald Holshausen, Mon Sep 14 10:41:13 2020 +1000)
* 814c416 - refactor: added a trait for interactions, renamed Interaction to RequestResponseInteraction (Ronald Holshausen, Sun Sep 13 17:09:41 2020 +1000)
* 77c8c8d - bump version to 0.7.2 (Ronald Holshausen, Sun Aug 23 17:19:24 2020 +1000)

# 0.7.1 - implemented provider state generator

* b186ce9 - chore: update all dependent crates (Ronald Holshausen, Sun Aug 23 16:49:00 2020 +1000)
* 61ca3d7 - chore: update matching crate to latest (Ronald Holshausen, Sun Aug 23 16:37:58 2020 +1000)
* ed207a7 - chore: updated readmes for docs site (Ronald Holshausen, Sun Jun 28 10:04:09 2020 +1000)

# 0.7.0 - Updated XML Matching

* 62b0bda - chore: update to latest matching library (Ronald Holshausen, Wed Jun 24 12:17:04 2020 +1000)
* bea787c - chore: bump matching crate version to 0.6.0 (Ronald Holshausen, Sat May 23 17:56:04 2020 +1000)
* 76250b5 - chore: correct some clippy warnings (Ronald Holshausen, Wed Apr 29 17:53:40 2020 +1000)
* 43de9c3 - chore: update matching library to latest (Ronald Holshausen, Fri Apr 24 10:20:55 2020 +1000)
* bd10d00 - Avoid deprecated Error::description in favor of Display trait (Caleb Stepanian, Mon Mar 30 16:49:13 2020 -0400)
* 1cf0199 - refactor: moved state change code to a handler (Ronald Holshausen, Wed Mar 11 14:37:07 2020 +1100)
* 70e6648 - chore: converted verifier to use Reqwest (Ronald Holshausen, Mon Mar 9 12:20:14 2020 +1100)
* fe74376 - feat: implemented publishing provider tags with verification results #57 (Ronald Holshausen, Sun Mar 8 18:37:21 2020 +1100)
* a6e0c16 - Fix RequestFilterExecutor w/ verify_provider (Andrew Lilley Brinker, Mon Mar 2 11:43:59 2020 -0800)
* d944a60 - chore: added callback executors so test code can called during verification (Ronald Holshausen, Sun Feb 23 18:43:49 2020 +1100)
* f238ca1 - Make pact_verifier_cli actually runnable by using tokio::main (Audun Halland, Sun Jan 19 10:12:17 2020 +0100)
* 70a33dd - chore: bump minor version of pact_verifier (Ronald Holshausen, Sun Jan 19 11:51:36 2020 +1100)
* cb4c560 - Upgrade tokio to 0.2.9 (Audun Halland, Fri Jan 10 00:13:02 2020 +0100)
* deaf4b3 - pact_verifier_cli: Increase type length limit for big generated future type (Audun Halland, Tue Dec 17 01:53:24 2019 +0100)
* 87d787f - pact_verifier_cli: Block on async function from pact_verifier (Audun Halland, Thu Dec 12 11:15:44 2019 +0100)
* c168d0b - pact_verifier_cli: Remove extern crate from main.rs (Audun Halland, Sun Nov 17 23:25:17 2019 +0100)
* 713cd6a - Explicit edition 2018 in Cargo.toml files (Audun Halland, Sat Nov 16 23:55:37 2019 +0100)
* 9f3ad74 - fix: docker build now requires libclang system library (Ronald Holshausen, Fri Sep 27 17:14:05 2019 +1000)
* 834a60b - bump version to 0.6.2 (Ronald Holshausen, Fri Sep 27 15:37:03 2019 +1000)

# 0.6.1 - Bugfix + Oniguruma crate for regex matching

* e32350e - chore: use the latest matching lib (Ronald Holshausen, Fri Sep 27 15:22:12 2019 +1000)
* 0cc03db - bump version to 0.6.1 (Ronald Holshausen, Sun Sep 22 18:13:48 2019 +1000)

# 0.6.0 - Publishing verification results

* 0e1da1b - chore: bump minor version (Ronald Holshausen, Sun Sep 22 17:59:51 2019 +1000)
* 2e07d77 - chore: set the version of the pact matching crate (Ronald Holshausen, Sun Sep 22 17:24:02 2019 +1000)
* 1110b47 - feat: implemented publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 13:53:27 2019 +1000)
* 7b5a404 - bump version to 0.5.2 (Ronald Holshausen, Sat Aug 24 13:00:10 2019 +1000)

# 0.5.1 - Use reqwest for better HTTP/S support, support headers with multiple values

* f79b033 - chore: update terminal support in release scripts (Ronald Holshausen, Sat Aug 24 12:25:28 2019 +1000)
* b8019ba - chore: bump the version of the matching lib (Ronald Holshausen, Sat Aug 24 12:22:35 2019 +1000)
* dac8ae1 - feat: support authentication when fetching pacts from a pact broker (Ronald Holshausen, Sun Aug 11 13:57:29 2019 +1000)
* e007763 - feat: support bearer tokens when fetching pacts from URLs (Ronald Holshausen, Sun Aug 11 13:21:17 2019 +1000)
* f947d43 - chore: upgrade the logging crates (Ronald Holshausen, Sun Aug 11 12:05:14 2019 +1000)
* 0dd10e6 - fix: docker release script (Ronald Holshausen, Sat Jul 27 18:02:11 2019 +1000)
* aa336e6 - bump version to 0.5.1 (Ronald Holshausen, Sat Jul 27 17:48:41 2019 +1000)

# 0.5.0 - Upgrade to non-blocking Hyper 0.12

* d842100 - chore: bump component versions to 0.5.0 (Ronald Holshausen, Sat Jul 27 15:44:51 2019 +1000)
* 47ab6d0 - Upgrade tokio to 0.1.22 everywhere (Audun Halland, Mon Jul 22 23:47:09 2019 +0200)
* 2f8a997 - Compile everything (except the commented-out tests) (Audun Halland, Thu May 2 00:41:56 2019 +0200)
* f8fa0d8 - chore: Bump pact matchig version to 0.5.0 (Ronald Holshausen, Sat Jan 5 19:25:53 2019 +1100)
* 3c33294 - fix: Only print errors in the CLI to STDERR #28 (Ronald Holshausen, Sun Apr 8 15:57:56 2018 +1000)
* 386ab52 - fix: corrected the release scripts to check for a version parameter (Ronald Holshausen, Sun Apr 8 13:44:57 2018 +1000)
* 6c2d6c8 - chore: added docker release scripts for the CLIs (Ronald Holshausen, Sun Apr 8 13:44:18 2018 +1000)
* 9d24b7e - fix: corrected the docker build for the verifier cli #14 (Ronald Holshausen, Sun Apr 8 13:39:29 2018 +1000)
* 4b8fb64 - fix: verification CLI was reporting incorrect pact specification version (Ronald Holshausen, Sun Apr 8 13:12:45 2018 +1000)
* fb8ecf5 - bump version to 0.4.1 (Ronald Holshausen, Sat Apr 7 15:23:33 2018 +1000)

# 0.4.0 - First V3 specification release

* 6597141 - WIP - start of implementation of applying generators to the bodies (Ronald Holshausen, Sun Mar 4 17:01:11 2018 +1100)
* f63f339 - replaced use of try macro with ? (Ronald Holshausen, Tue Nov 7 16:31:39 2017 +1100)
* 7fef36b - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sat Nov 4 12:49:07 2017 +1100)
* 5c05f18 - Added docker file for pact verifier (Ronald Holshausen, Fri Nov 3 16:20:02 2017 +1100)
* 6a0548c - Correct release scripts (Ronald Holshausen, Fri Nov 3 15:51:52 2017 +1100)
* 9f20613 - bump version to 0.3.1 (Ronald Holshausen, Fri Nov 3 15:51:27 2017 +1100)
* 91a5673 - Correct the release script (Ronald Holshausen, Fri Nov 3 15:42:48 2017 +1100)
* 00dc75a - Bump version to 0.4.0 (Ronald Holshausen, Sun Oct 22 10:46:48 2017 +1100)
* 184127a - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sun Oct 22 10:32:31 2017 +1100)
* e82ee08 - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Mon Oct 16 09:24:11 2017 +1100)
* 64ff667 - Upgraded the mock server implemenation to use Hyper 0.11.2 (Ronald Holshausen, Wed Sep 6 12:56:47 2017 +1000)
* e5a93f3 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Aug 20 09:53:48 2017 +1000)
* 639ac22 - fixes after merge in from master (Ronald Holshausen, Sun Oct 23 10:45:54 2016 +1100)
* 49e45f7 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Oct 23 10:10:40 2016 +1100)
* 539eb48 - updated all the readmes and cargo manefests for v3 (Ronald Holshausen, Tue Jul 19 15:46:18 2016 +1000)

# 0.3.0 - Backported matching rules from V3 branch

* b2ad496 - Updated the verifier cli dep modules (Ronald Holshausen, Fri Nov 3 15:14:57 2017 +1100)
* ac94388 - Tests are now all passing #20 (Ronald Holshausen, Thu Oct 19 15:14:25 2017 +1100)
* c983c63 - Bump versions to 0.3.0 (Ronald Holshausen, Wed Oct 18 13:54:46 2017 +1100)
* 06e92e5 - Refer to local libs using version+paths (Eric Kidd, Tue Oct 3 06:22:23 2017 -0400)
* 7afd258 - Update all the cargo manifest versions and commit the cargo lock files (Ronald Holshausen, Wed May 17 10:37:44 2017 +1000)
* be8c299 - Cleanup unused BTreeMap usages and use remote pact dependencies (Anthony Damtsis, Mon May 15 17:09:14 2017 +1000)
* a59fb98 - Migrate remaining pact modules over to serde (Anthony Damtsis, Mon May 15 16:59:04 2017 +1000)
* d5e6ce0 - bump version to 0.2.1 (Ronald Holshausen, Sun Oct 9 17:20:25 2016 +1100)

# 0.2.0 - V2 specification implementation

* 38027f8 - updated the pact_verifier_cli to V2 (Ronald Holshausen, Sun Oct 9 17:12:35 2016 +1100)
* 770010a - update projects to use the published pact matching lib (Ronald Holshausen, Sun Oct 9 16:25:15 2016 +1100)
* 574e072 - upadte versions for V2 branch and fix an issue with loading JSON bodies encoded as a string (Ronald Holshausen, Sun Oct 9 15:31:57 2016 +1100)
* b0bebb7 - bump version to 0.1.1 (Ronald Holshausen, Sun Oct 9 11:27:41 2016 +1100)

# 0.1.0 - V1.1 specification implementation

* ea1ef54 - Updated dependencies and version for release of pact_verifier_cli (Ronald Holshausen, Sun Oct 9 10:56:34 2016 +1100)
* 1f3f3f1 - correct the versions of the inter-dependent projects as they were causing the build to fail (Ronald Holshausen, Sat Oct 8 17:41:57 2016 +1100)
* a46dabb - update all references to V1 spec after merge (Ronald Holshausen, Sat Oct 8 16:20:51 2016 +1100)
* b6df52f - bump version to 0.0.1 (Ronald Holshausen, Tue Sep 27 22:27:26 2016 +1000)

# 0.0.0 - First Release
