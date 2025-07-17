To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 0.4.28 - Maintenance Release

* 4fc0136c - chore: Upgrade pact_verifier to 1.3.2 (Ronald Holshausen, Fri Jun 20 11:33:58 2025 +1000)
* 42b0b285 - chore(pact-ffi): Update pact_mock_server to 1.2.16 (Ronald Holshausen, Wed Jun 11 15:33:46 2025 +1000)
* 92c4db5e - chore(pact_ffi): Update pact_matching to 1.2.12 (Ronald Holshausen, Wed Jun 11 13:52:19 2025 +1000)
* 6e8167c4 - fix(Alpine): Updae pact_models to 1.3.3 (Ronald Holshausen, Wed Jun 11 13:23:10 2025 +1000)
* 1d18d3f5 - fix: Onig crate needs default-features = false so it compiles on Alpine (Ronald Holshausen, Wed Jun 11 12:35:06 2025 +1000)
* 588f1fe2 - chore: Update pact_matching to 2.0.0-beta.2 (Ronald Holshausen, Wed Jun 11 11:24:09 2025 +1000)
* 18e53fbd - chore: Update pact_models to 1.3.2 (Ronald Holshausen, Wed Jun 11 10:16:03 2025 +1000)
* 82d078d9 - fix: Upgrade onig to 6.5.1 as the Match Whole String for is_match has been released (Ronald Holshausen, Wed Jun 11 10:00:52 2025 +1000)
* 2b4536b1 - chore: Bump pact_verifier to 1.3.0 (Ronald Holshausen, Tue May 20 16:34:00 2025 +1000)
* 7d8e8f61 - chore: Upgrade pact_models to 1.3.1 (Ronald Holshausen, Tue May 20 14:53:42 2025 +1000)
* e1d1b5e3 - chore: Use the tasklocal LOG_ID from the mock server crate (Ronald Holshausen, Wed Mar 26 15:45:56 2025 +1100)
* 9afae5a5 - fix(pact-ffi): FFI needs to use the same version of pact-matching as the mock server crate (Ronald Holshausen, Wed Mar 26 11:27:01 2025 +1100)
* 369dba85 - chore: Upgrade pact-mock-server to 2.0.3/1.2.12 (Ronald Holshausen, Wed Mar 26 11:00:31 2025 +1100)
* 12ca5283 - chore: Upgrade pact-plugin-driver to 0.7.4 (Ronald Holshausen, Tue Mar 25 16:26:48 2025 +1100)
* 0ed75202 - chore(pact-models): Upgrade pact models crate to Rust 2024 edition (Ronald Holshausen, Tue Mar 25 16:02:47 2025 +1100)
* 79b52714 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Thu Mar 20 09:43:30 2025 +1100)
* 70b366f4 - bump version to 0.4.28 (Ronald Holshausen, Thu Mar 20 09:19:53 2025 +1100)
* b07d7aec - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Wed Mar 19 11:22:43 2025 +1100)
* ad9b555b - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Fri Mar 14 11:42:13 2025 +1100)
* ad9ee539 - fix: Matching rule definitions now take th expression they were parsed from (Ronald Holshausen, Tue Mar 4 09:18:51 2025 +1100)
* 7b94a211 - Merge tag 'master' into feat/matching-engine-v2 (Ronald Holshausen, Thu Feb 20 15:16:16 2025 +1100)
* 5db8db23 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Wed Nov 13 14:16:31 2024 +1100)
* 11cd9dc0 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Fri Oct 25 10:01:15 2024 +1100)
* 02e2ae13 - chore: Bump pact_matching to 2.0.0-beta.0 (Ronald Holshausen, Tue Jul 30 11:40:20 2024 +1000)

# 0.4.27 - Bugfix Release

* df9beec3 - chore: Upgrade pact_verifier to 1.2.5 (Ronald Holshausen, Wed Mar 19 14:08:12 2025 +1100)
* e32f0dbd - fix: Upgrade pact-plugin-driver to 0.7.3 (Ronald Holshausen, Wed Mar 19 11:20:39 2025 +1100)
* bdbbd587 - Merge pull request #489 from pulak777/pact-ffi-tests (Ronald Holshausen, Mon Mar 17 09:59:44 2025 +1100)
* 11dfc492 - feat(test):add tests for different date matcher scenarios in pact_ffi (Pulak Deyashi, Thu Mar 13 17:16:38 2025 +0000)
* d67db1a4 - chore: Update pact_models to 1.2.7 (Ronald Holshausen, Fri Mar 14 11:02:53 2025 +1100)
* d86ea2c8 - fix: Content type matcher was not being applied when dealing with multi-part bodies #482 (Ronald Holshausen, Mon Feb 3 17:38:49 2025 +1100)
* 8fb0bcb9 - chore: Upgrade pact_matching to 1.2.9 (Ronald Holshausen, Fri Jan 31 09:44:14 2025 +1100)
* e98a2183 - fix: Do not apply include matcher at the collection level #485 (Ronald Holshausen, Wed Jan 29 17:09:04 2025 +1100)
* 096deedc - fix: when comparing JSON with number matchers, don't covert numbers in string form to numbers #484 (Ronald Holshausen, Wed Jan 29 16:52:06 2025 +1100)
* d93dfb3f - fix: Do not apply date/time matchers at the collection level #483 (Ronald Holshausen, Wed Jan 29 14:34:31 2025 +1100)
* 283fcb12 - bump version to 0.4.27 (Ronald Holshausen, Fri Dec 20 14:19:23 2024 +1100)

# 0.4.26 - Re-release FFI

* 823b009a - Reapply "chore(ci): refactor release pipeline" (Ronald Holshausen, Fri Dec 20 11:08:57 2024 +1100)
* 9321da66 - Revert "chore(release): Rename MacOS artifacts correctly" (Ronald Holshausen, Fri Dec 20 11:07:26 2024 +1100)
* 7bc0cf52 - chore(release): Rename MacOS artifacts correctly (Ronald Holshausen, Fri Dec 20 10:03:56 2024 +1100)
* 7c09f177 - Revert "chore(ci): refactor release pipeline" (Ronald Holshausen, Fri Dec 20 09:31:37 2024 +1100)
* f4754e3d - bump version to 0.4.26 (Ronald Holshausen, Thu Dec 19 16:32:57 2024 +1100)
* 750e0f7e - update changelog for release 0.4.25 (Ronald Holshausen, Thu Dec 19 16:31:15 2024 +1100)
* 648ab1c9 - chore: Upgrade pact_matching to 1.2.8 (Ronald Holshausen, Thu Dec 19 16:14:10 2024 +1100)
* d74af0ab - Merge pull request #462 from tienvx/support-integration-json-for-form-urlencoded (Ronald Holshausen, Thu Dec 19 15:15:09 2024 +1100)
* 4aa50181 - chore(FFI): Update pact_mock_server to 1.2.11 (Ronald Holshausen, Fri Dec 13 10:57:23 2024 +1100)
* 5fdfd023 - chore: Upgrade all the tracing crates to the latest (Ronald Holshausen, Fri Dec 13 10:54:51 2024 +1100)
* 4722e2e3 - feat(FFI): Support integration json for form urlencoded (tien.xuan.vo, Thu Sep 5 01:30:55 2024 +0700)
* fab32aa0 - chore: Upgrade pact_models to 1.2.6 (Ronald Holshausen, Wed Dec 11 15:28:01 2024 +1100)
* 7792c635 - bump version to 0.4.25 (Ronald Holshausen, Fri Nov 29 16:03:37 2024 +1100)

# 0.4.25 - Support for FORM urlencoded bodies

* 648ab1c9 - chore: Upgrade pact_matching to 1.2.8 (Ronald Holshausen, Thu Dec 19 16:14:10 2024 +1100)
* d74af0ab - Merge pull request #462 from tienvx/support-integration-json-for-form-urlencoded (Ronald Holshausen, Thu Dec 19 15:15:09 2024 +1100)
* 4aa50181 - chore(FFI): Update pact_mock_server to 1.2.11 (Ronald Holshausen, Fri Dec 13 10:57:23 2024 +1100)
* 5fdfd023 - chore: Upgrade all the tracing crates to the latest (Ronald Holshausen, Fri Dec 13 10:54:51 2024 +1100)
* 4722e2e3 - feat(FFI): Support integration json for form urlencoded (tien.xuan.vo, Thu Sep 5 01:30:55 2024 +0700)
* fab32aa0 - chore: Upgrade pact_models to 1.2.6 (Ronald Holshausen, Wed Dec 11 15:28:01 2024 +1100)
* 7792c635 - bump version to 0.4.25 (Ronald Holshausen, Fri Nov 29 16:03:37 2024 +1100)

# 0.4.24 - Bugfix Release

* 44d9dc1e - fix(plugins): Upgrade pact-plugin-driver to 0.7.2, fixes #473 (Ronald Holshausen, Fri Nov 29 15:48:40 2024 +1100)
* b55407ef - fix(FFI): pactffi_with_pact_metadata overwrites existing namespace contents #466 (Ronald Holshausen, Mon Nov 18 12:22:42 2024 +1100)
* 420c654d - chore: Upgrade pact_models to 1.2.5 (Ronald Holshausen, Wed Nov 13 11:01:39 2024 +1100)
* 82e570a1 - docs: Update comment for pactffi_message_given_with_param (tien.xuan.vo, Thu Oct 31 17:18:45 2024 +0700)
* a85d9e07 - Merge pull request #470 from tienvx/add-error-handling (Ronald Holshausen, Mon Oct 28 08:58:09 2024 +1100)
* 70bf2d92 - docs: Add error handling (tien.xuan.vo, Sat Oct 26 15:49:59 2024 +0700)
* 2b5b0c1e - docs: Add known log sink type (tien.xuan.vo, Sat Oct 26 11:52:26 2024 +0700)
* f5ddf3d3 - docs: Update safety comments (tien.xuan.vo, Thu Sep 26 14:51:14 2024 +0700)
* fe07b878 - docs: add pact_ffi compat docs [skip ci] (Yousaf Nabi, Fri Sep 6 12:25:40 2024 +0100)
* f5228c52 - chore: Use the inmem logging functions in the FFI crate (Ronald Holshausen, Thu Sep 5 10:14:50 2024 +1000)
* 9d3610dd - chore: Use the pact_matcing LOG_ID in the FFI inmem logging functions (Ronald Holshausen, Thu Sep 5 10:03:30 2024 +1000)
* 16861857 - chore: Replicate setting the LOG_ID in the FFI verify functions (Ronald Holshausen, Wed Sep 4 16:04:50 2024 +1000)
* edc6a23e - chore: Copy the in-mem logging functions to the FFI crate (Ronald Holshausen, Wed Sep 4 15:53:01 2024 +1000)
* 8a5150e0 - chore: Copy the in-mem logging functions to the FFI crate (Ronald Holshausen, Wed Sep 4 15:50:51 2024 +1000)
* 8350e4d3 - bump version to 0.4.24 (Ronald Holshausen, Wed Sep 4 15:19:59 2024 +1000)

# 0.4.23 - Bugfix Release

* 3cb09550 - chore(pact_ffi): Update dependencies (Ronald Holshausen, Wed Sep 4 11:09:27 2024 +1000)
* c1a00dd8 - chore: Upgrade pact-plugin-driver to 0.7.1 (Ronald Holshausen, Tue Sep 3 15:47:22 2024 +1000)
* eb687a55 - fix(ffi): annotate body in raw body branch (JP-Ellis, Fri Aug 30 13:01:02 2024 +1000)
* dc441709 - chore: Upgrade pact_matching to 1.2.7 (Ronald Holshausen, Tue Aug 27 14:57:12 2024 +1000)
* fa02c30b - chore: Upgrade pact_verifier to 1.2.4 (Ronald Holshausen, Tue Aug 13 15:42:48 2024 +1000)
* 978677cc - chore: Upgrade pact_matching to 1.2.6 (Ronald Holshausen, Tue Aug 13 15:27:49 2024 +1000)
* f50454ff - chore: Upgrade test dependencies (Ronald Holshausen, Tue Aug 13 11:54:59 2024 +1000)
* 9a7eef75 - chore: Upgrade pact_models to 1.2.4 (Ronald Holshausen, Tue Aug 13 11:51:06 2024 +1000)
* bcf4db6d - chore: Upgrade pact_models to 1.2.3 (Ronald Holshausen, Tue Aug 6 11:06:22 2024 +1000)
* 4d7295a4 - feat: allow non-object matchers for array-contains (M. J. W-H. Campman, Sat Aug 3 13:49:18 2024 -0400)
* 033a50ab - feat: Update the matching rule expression parser to support values from provider states (Ronald Holshausen, Fri Aug 2 10:06:52 2024 +1000)
* fd1ba0c3 - bump version to 0.4.23 (Ronald Holshausen, Thu Jul 18 15:51:48 2024 +1000)

# 0.4.22 - Bugfix Release

* 4257ed1e - chore: Upgrade pact_verifier to 1.2.3 (Ronald Holshausen, Wed Jul 17 15:19:58 2024 +1000)
* 3c96c31b - chore: Upgrade pact_mock_server to 1.2.9 (Ronald Holshausen, Wed Jul 17 14:54:40 2024 +1000)
* 51c6cb05 - chore(ffi): deprecate `pactffi_with_binary_file` (JP-Ellis, Wed Jul 17 13:14:04 2024 +1000)
* bb9fc1be - chore: Upgrade pact_matching to 1.2.5 (Ronald Holshausen, Wed Jul 17 14:34:04 2024 +1000)
* 47754035 - fix(ffi): shared processing of body (JP-Ellis, Tue Jul 16 10:08:22 2024 +1000)
* fd9ff70a - chore: Upgrade pact_models to 1.2.2 (Ronald Holshausen, Wed Jul 17 11:15:34 2024 +1000)
* e8b62f98 - chore: Upgrade pact-plugin-driver to 0.7.0 (Ronald Holshausen, Wed Jul 17 10:45:34 2024 +1000)
* cccd4285 - docs(FFI): Update doc comments to indicate how to enable publishing verification results (Ronald Holshausen, Wed Jul 10 17:16:35 2024 +1000)
* f03cd0d3 - feat(pact_verifier_cli): Parse consumer version selector JSON when evaluating CLI args so errors are reported earlier (Ronald Holshausen, Wed Jul 10 12:13:11 2024 +1000)
* b52007e2 - chore: Fix race condition in tests in CI (Ronald Holshausen, Tue Jul 9 14:46:57 2024 +1000)
* fde1e2d1 - chore: Fix race condition in tests in CI (Ronald Holshausen, Tue Jul 9 14:43:10 2024 +1000)
* efc54d26 - fix(FFI): update_provider_info was not setting the transport scheme correctly (Ronald Holshausen, Mon Jun 24 11:35:57 2024 +1000)
* 45914c08 - bump version to 0.4.22 (Ronald Holshausen, Wed Jun 19 14:54:30 2024 +1000)

# 0.4.21 - Bugfix Release

* 3acc9ae5 - Merge pull request #443 from pact-foundation/feat/generate-contents (Ronald Holshausen, Wed Jun 19 14:21:50 2024 +1000)
* a18b7746 - feat: add `generate_contents` methods (JP-Ellis, Mon Jun 17 14:35:10 2024 +1000)
* 30bac100 - feat: add with_generators (JP-Ellis, Tue Jun 18 12:08:44 2024 +1000)
* 598377d4 - feat(ffi): add async message iterator (JP-Ellis, Fri Jun 14 20:11:49 2024 +1000)
* dad482f5 - chore: Upgrade pact_verifier to 1.2.2 (Ronald Holshausen, Fri Jun 14 11:21:54 2024 +1000)
* 29c3b5f0 - chore: Upgrade pact_matching to 1.2.4 (Ronald Holshausen, Fri Jun 14 10:18:26 2024 +1000)
* 30998a95 - chore: Upgrade pact_models to 1.2.1 (Ronald Holshausen, Wed Jun 12 15:26:49 2024 +1000)
* ab80cd2b - fix: merge pact duplicates with diff desc/same prov state fix: merge message / sync pacts with no provider state (Yousaf Nabi, Sat Jun 8 23:48:30 2024 +0100)
* 0998cd29 - chore(FFI): Disable test using curl as the header fails to compile (Ronald Holshausen, Wed Jun 12 11:38:21 2024 +1000)
* d7489b8d - chore(FFI): Add a test for mock server logs from C example (Ronald Holshausen, Wed Jun 12 11:21:28 2024 +1000)
* 278f56fa - fix: FFI Inmemory sink uses the global static `LOG_BUFFER` in the pact_matching crate #434 (Ronald Holshausen, Tue Jun 11 16:56:15 2024 +1000)
* b7869cdb - Merge pull request #439 from pact-foundation/feat/with-metadata (Ronald Holshausen, Tue Jun 11 16:19:07 2024 +1000)
* 19957df5 - chore: Lock tracing crates to the same version via cargo patch block (Ronald Holshausen, Tue Jun 11 16:00:04 2024 +1000)
* dd6381c1 - feat(ffi): add with_metadata (JP-Ellis, Tue Jun 11 15:26:54 2024 +1000)
* 582de5c9 - refactor: update pact_ffi/pact_verifier_cli release workflows macos (Yousaf Nabi, Sat Jun 8 02:48:40 2024 +0100)
* c9443652 - refactor: update pact_ffi/pact_verifier_cli release workflows (Yousaf Nabi, Sat Jun 8 02:45:46 2024 +0100)
* 5699cab7 - chore: Upgrade pact_mock_server to 1.2.8 (Ronald Holshausen, Thu May 30 16:53:17 2024 +1000)
* a811b8a4 - chore: Update pact_matching to 1.2.3 (Ronald Holshausen, Thu May 30 15:57:32 2024 +1000)
* 4f01a302 - chore: Fix project so build works (Ronald Holshausen, Thu May 30 12:01:44 2024 +1000)
* b0fd7e2f - chore: refactor to preserve existing error messages (Yousaf Nabi, Wed May 29 03:31:11 2024 +0100)
* 968be075 - feat: update content_type_matching logic for binary payloads (Yousaf Nabi, Tue May 28 23:25:07 2024 +0100)
* efc3dc49 - bump version to 0.4.21 (Ronald Holshausen, Thu May 9 14:03:59 2024 +1000)

# 0.4.20 - Maintenance Release

* 2c28d36b - chore(FFI): Upgrade dependencies (Ronald Holshausen, Wed May 8 16:32:55 2024 +1000)
* edc1ad0c - chore: Upgrade pact_verifier to 1.2.1 (Ronald Holshausen, Wed May 8 15:32:36 2024 +1000)
* f5ad729f - chore: Moved mock server crates to pact-foundation/pact-core-mock-server (Ronald Holshausen, Tue May 7 17:36:26 2024 +1000)
* fa2b1d09 - chore: Upgrade pact_matching to 1.2.2 (Ronald Holshausen, Tue May 7 10:50:08 2024 +1000)
* adf6379a - chore: build linux x86_64 ffi against glibc 2.23 (Yousaf Nabi, Mon Apr 29 18:25:54 2024 +0100)
* 694100fb - chore: Update pact_models to 1.2.0 (Ronald Holshausen, Tue Apr 23 10:51:11 2024 +1000)
* edfac7ba - chore: remove local pact_models from the other pact crates (Ronald Holshausen, Tue Apr 23 10:03:29 2024 +1000)
* d7952e15 - Merge pull request #411 from pact-foundation/feat/optional-query-parameter-values (Ronald Holshausen, Mon Apr 22 15:17:01 2024 +1000)
* 941560b4 - chore: Update doc comment with NULL query parameter value example (Ronald Holshausen, Mon Apr 22 15:01:08 2024 +1000)
* c3128a6d - feat: Support optional query parameter values (where there is only a name) (Ronald Holshausen, Mon Apr 22 10:36:05 2024 +1000)
* e3dc2658 - fix: downgrade JSON parsing from warn to debug in from_integration_json_v2. Fixes #408 (Matt Fellows, Wed Apr 17 13:28:14 2024 +1000)
* 758f4c03 - chore: Update pact_matching to 1.2.1 (Ronald Holshausen, Tue Apr 16 16:29:38 2024 +1000)
* bfd6eae0 - chore: Upgrade pact_mock_server to 1.2.6 (Ronald Holshausen, Tue Apr 16 15:14:00 2024 +1000)
* 7688469a - chore(pact_verifier): Bump minor version (Ronald Holshausen, Tue Apr 16 10:41:18 2024 +1000)
* d6125b75 - chore(pact_matching): Bump minor version (Ronald Holshausen, Tue Apr 16 10:16:44 2024 +1000)
* a38e93c5 - fix(ci): fix macos extensions (JP-Ellis, Thu Apr 11 13:12:30 2024 +1000)
* c2ec4610 - bump version to 0.4.20 (Ronald Holshausen, Thu Apr 11 12:05:29 2024 +1000)

# 0.4.19 - Bugfix Release

* d3c18d21 - chore(ffi): Remove unecessary mutable references #399 (Ronald Holshausen, Wed Mar 20 14:47:54 2024 +1100)
* 21406631 - feat(ffi): Update intermediate JSON to support matching rule expressions #399 (Ronald Holshausen, Wed Mar 20 11:59:46 2024 +1100)
* 09077457 - chore(ffi): Fix for failing musl build #399 (Ronald Holshausen, Tue Mar 19 14:16:55 2024 +1100)
* 85dc3e95 - feat(ffi): Update intermediate JSON to support combining matching rules #399 (Ronald Holshausen, Tue Mar 19 13:57:54 2024 +1100)
* 6d9ddb76 - Merge pull request #401 from pact-foundation/feat/add-text-comment (Ronald Holshausen, Mon Mar 18 11:05:40 2024 +1100)
* f6430bf0 - refactor(ffi): simply `set_comment` using `comments_mut` (JP-Ellis, Fri Mar 15 15:59:02 2024 +1100)
* f310e8ec - feat(ffi): add `add_text_comment` (JP-Ellis, Fri Mar 15 09:02:49 2024 +1100)
* ffffa61d - chore: Upgrade pact_mock_server to 1.2.5 (Ronald Holshausen, Fri Mar 15 15:13:04 2024 +1100)
* 6406e9ae - refactor(release): Rename OSX to MacOS (Ronald Holshausen, Fri Mar 15 14:28:06 2024 +1100)
* fca0c4a5 - fix: binary body lifetime issue (JP-Ellis, Tue Mar 12 16:36:43 2024 +1100)
* 72f83688 - Merge pull request #391 from pact-foundation/chore/fix-pactffi-release (Ronald Holshausen, Wed Feb 28 14:39:04 2024 +1100)
* e5eee74c - fix(FFI): Use random handle IDs instead of sequentual ones (Ronald Holshausen, Wed Feb 28 10:41:07 2024 +1100)
* 7732a637 - chore(ci): refactor release pipeline (JP-Ellis, Mon Feb 26 17:54:18 2024 +1100)
* 84dfc194 - bump version to 0.4.19 (Ronald Holshausen, Mon Feb 26 14:47:29 2024 +1100)
* 6bfa4a94 - update changelog for release 0.4.18 (Ronald Holshausen, Mon Feb 26 14:44:33 2024 +1100)
* 93f02c74 - feat(ffi): allow setting comments (JP-Ellis, Mon Feb 26 11:26:39 2024 +1100)
* 8ac21bdb - feat(ffi): allow toggling pending state (JP-Ellis, Mon Feb 26 10:48:32 2024 +1100)
* 0d12bee4 - feat(ffi): allow setting interaction key (JP-Ellis, Mon Feb 26 10:40:00 2024 +1100)
* 634bc589 - chore(FFI): FFI Feature tests were writing to the same Pact file and can cause race conditions (Ronald Holshausen, Wed Feb 21 11:58:37 2024 +1100)
* e902c532 - bump version to 0.4.18 (Ronald Holshausen, Wed Feb 21 11:29:01 2024 +1100)

# 0.4.18 - Maintenance Release

* 93f02c74 - feat(ffi): allow setting comments (JP-Ellis, Mon Feb 26 11:26:39 2024 +1100)
* 8ac21bdb - feat(ffi): allow toggling pending state (JP-Ellis, Mon Feb 26 10:48:32 2024 +1100)
* 0d12bee4 - feat(ffi): allow setting interaction key (JP-Ellis, Mon Feb 26 10:40:00 2024 +1100)
* 634bc589 - chore(FFI): FFI Feature tests were writing to the same Pact file and can cause race conditions (Ronald Holshausen, Wed Feb 21 11:58:37 2024 +1100)
* e902c532 - bump version to 0.4.18 (Ronald Holshausen, Wed Feb 21 11:29:01 2024 +1100)

# 0.4.17 - Bugfix Release

* 82804395 - Revert "chore: Upgrade pact_verifier to 1.1.1" (Ronald Holshausen, Mon Feb 19 11:43:12 2024 +1100)
* d17dac05 - chore: Upgrade pact_verifier to 1.1.1 (Ronald Holshausen, Mon Feb 19 11:42:50 2024 +1100)
* 6975203e - chore(verifier): Bump minor version #307 (Ronald Holshausen, Mon Feb 19 06:15:47 2024 +1100)
* b6a0e66b - chore(FFI): Add a test using pactffi_given_with_params #298 (Ronald Holshausen, Sun Feb 18 11:11:04 2024 +1100)
* c4e5ffe5 - fix(FFI): Update doc comments to include details of replacing interactions #389 (Ronald Holshausen, Sun Feb 18 08:56:33 2024 +1100)
* 5dad90cb - fix(FFI): creating a new interaction with the same description as an existiong one will now replace it #389 (Ronald Holshausen, Sun Feb 18 06:55:23 2024 +1100)
* 8b35aab4 - chore(FFI): add a test that calls write pact functions multiple types #389 (Ronald Holshausen, Sun Feb 18 06:12:02 2024 +1100)
* db3c3818 - Merge pull request #388 from YOU54F/feat/convert_musl_static_to_dynamic_so (Ronald Holshausen, Wed Feb 14 09:17:25 2024 +1100)
* df22043e - feat: convert musl static .a to .so (Yousaf Nabi, Tue Feb 13 21:51:02 2024 +0000)
* 41e9c469 - bump version to 0.4.17 (Ronald Holshausen, Mon Feb 12 17:06:33 2024 +1100)

# 0.4.16 - Bugfix Release

* 29b326e5 - fix(FFI): Metadata matching rules from plugins were being ignored (Ronald Holshausen, Thu Feb 8 18:31:05 2024 +1100)
* 4b9aeb2d - chore: correct CHANGELOG.md (Ronald Holshausen, Wed Feb 7 14:21:29 2024 +1100)
* 76debd2d - bump version to 0.4.16 (Ronald Holshausen, Wed Feb 7 14:15:28 2024 +1100)

# 0.4.15 - Bugfix Release

* b0f0151e - Merge pull request #383 from pact-foundation/feat/with-matching-rules-2 (Ronald Holshausen, Wed Feb 7 14:54:11 2024 +1100)
* 3554a333 - feat(ffi): add with_matching_rules (JP-Ellis, Tue Feb 6 12:47:42 2024 +1100)
* a52e0ee9 - chore: Upgrade pact_matching to 1.1.10 (Ronald Holshausen, Wed Feb 7 13:20:45 2024 +1100)
* 24a26cca - chore: Update pact_models to 1.1.18 (Ronald Holshausen, Wed Feb 7 10:53:22 2024 +1100)
* 73578350 - chore: use local pact_models (JP-Ellis, Tue Feb 6 10:51:09 2024 +1100)
* 95cbe5a9 - fix: Upgrade pact-plugin-driver to 0.5.1 (Ronald Holshausen, Wed Jan 31 19:56:04 2024 +1100)
* 7370d8f9 - chore: update artifact path for aarch64 musl pact_ffi (Yousaf Nabi, Thu Jan 25 12:01:20 2024 +0000)
* 0cd5fe82 - bump version to 0.4.15 (Ronald Holshausen, Sat Jan 20 22:33:27 2024 +1100)

# 0.4.14 - Bugfix Release

* 6b82b19f - chore(FFI): Update all dependencies (Ronald Holshausen, Sat Jan 20 22:24:41 2024 +1100)
* 8d3f146a - chore: Upgrade pact_verifier to 1.0.5 (Ronald Holshausen, Sat Jan 20 21:31:51 2024 +1100)
* 5d3b041e - Merge branch 'master' into allow-turn-off-log (tien.xuan.vo, Thu Jan 25 08:35:46 2024 +0700)
* b82f0a4c - tests(pact_ffi): Add tests for log_level_from_c_char (tien.xuan.vo, Thu Jan 25 08:35:03 2024 +0700)
* bbf61ba5 - docs(pact_ffi): Mention about default log level value (tien.xuan.vo, Thu Jan 25 08:34:03 2024 +0700)
* f2fa8a1a - feat(pact_ffi): Allow turn off log (tien.xuan.vo, Wed Jan 24 22:06:46 2024 +0700)
* def5685c - chore: Upgrade pact_mock_server to 1.2.4 (Ronald Holshausen, Sat Jan 20 18:56:26 2024 +1100)
* 6fa097a5 - Merge branch 'master' into ci/cargo_clean (Ronald Holshausen, Thu Jan 25 10:22:25 2024 +1100)
* 835bd21b - Merge branch 'master' into feat/musl_aarch64_ffi (Ronald Holshausen, Thu Jan 25 10:02:21 2024 +1100)
* 36390097 - Merge pull request #371 from YOU54F/chore/renable_verifier_aarch64_linux (Ronald Holshausen, Thu Jan 25 09:59:53 2024 +1100)
* 00402ba4 - chore: migrate target/artifacts to release_artifacts to avoid cargo clean loss (Yousaf Nabi, Wed Jan 24 15:26:18 2024 +0000)
* 22331310 - ci: cargo clean prior to cross build (Yousaf Nabi, Wed Jan 24 15:24:10 2024 +0000)
* c412829f - feat: build aarch64-pc-windows-msvc target (ffi/mock_server/verifier) (Yousaf Nabi, Wed Jan 24 12:01:50 2024 +0000)
* acfab971 - chore: remove cross usage from aarch64-apple-darwin (Yousaf Nabi, Wed Jan 24 12:26:43 2024 +0000)
* bd636e70 - chore: remove cross usage from aarch64-apple-darwin (Yousaf Nabi, Wed Jan 24 12:20:11 2024 +0000)
* a2ba1cd5 - fix: pin cross to 0.2.5 for glibc 2.23 (Yousaf Nabi, Wed Jan 24 11:54:22 2024 +0000)
* c74d1876 - feat: enable aarch64-unknown-linux-musl libpact_ffi.a (Yousaf Nabi, Wed Jan 24 12:10:56 2024 +0000)
* 1b847777 - fix: pin cross to 0.2.5 for glibc 2.23 (Yousaf Nabi, Wed Jan 24 11:54:22 2024 +0000)
* e552bdce - chore: Upgrade pact_matching to 1.1.9 (Ronald Holshausen, Sat Jan 20 15:13:13 2024 +1100)
* 7b087acf - chore: Upgrade pact-plugin-driver to 0.5.0 (Ronald Holshausen, Sat Jan 20 14:49:21 2024 +1100)
* b735df9d - chore: Upgrade pact_models to 1.1.17 (Ronald Holshausen, Sat Jan 20 13:54:03 2024 +1100)
* 367aa253 - fix(FFI): do not use a panic as the mechanism to detect end of iterator #344 (Ronald Holshausen, Fri Jan 19 21:38:58 2024 +1100)
* 1a4bcd27 - chore: Upgrade pact_matching to 1.1.8 (Ronald Holshausen, Fri Jan 19 18:24:54 2024 +1100)
* 37e498e9 - test(pact_matching): Test more cases for ContentType matcher (tien.xuan.vo, Sat Jan 20 17:39:07 2024 +0700)
* a990c55d - bump version to 0.4.14 (Ronald Holshausen, Fri Jan 19 15:05:21 2024 +1100)

# 0.4.13 - Bugfix Release

* 944613df - fix: regression - upgrade pact_models to 1.1.16 #359 (Ronald Holshausen, Fri Jan 19 14:52:36 2024 +1100)
* db970926 - bump version to 0.4.13 (Ronald Holshausen, Wed Jan 17 09:48:16 2024 +1100)

# 0.4.12 - Bugfix Release

* 403c0af1 - chore: Upgrade pact_models to 1.1.14 #355 (Ronald Holshausen, Tue Jan 16 10:31:12 2024 +1100)
* 9d121e5e - fix(FFI): Headers configured with provider state generator have invalid paths in Pact file #355 (Ronald Holshausen, Tue Jan 16 07:48:20 2024 +1100)
* dfd13760 - chore: Upgrade pact_models to 1.1.13 #355 (Ronald Holshausen, Tue Jan 16 07:42:33 2024 +1100)
* 39a14e97 - chore: Upgrade pact_verifier to 1.0.4 (Ronald Holshausen, Wed Dec 20 14:46:57 2023 +1100)
* 713e4098 - chore: Upgrade pact-plugin-driver to 0.4.6 (Ronald Holshausen, Thu Dec 14 17:04:59 2023 +1100)
* ca698fce - bump version to 0.4.12 (Ronald Holshausen, Mon Nov 27 19:55:59 2023 +1100)

# 0.4.11 - Bugfix Release

* 82e51872 - Merge pull request #348 from tienvx/update-trace-logs (Ronald Holshausen, Tue Nov 28 09:36:04 2023 +1100)
* 133bf14f - Merge pull request #347 from tienvx/update-comment-metadata-v2 (Ronald Holshausen, Tue Nov 28 09:35:22 2023 +1100)
* 777e19ef - chore: Update trace logs (tien.xuan.vo, Thu Nov 23 17:12:15 2023 +0700)
* c238fdd2 - docs: Add comment about example usage and safety to pactffi_message_with_metadata_v2 (tien.xuan.vo, Thu Nov 23 02:39:00 2023 +0700)
* f87c76e8 - feat: Add pactffi_response_status_v2 (tien.xuan.vo, Wed Nov 22 23:07:05 2023 +0700)
* 2983e52f - refactor: Reuse matching rules and generators processing code (tien.xuan.vo, Tue Nov 21 22:24:52 2023 +0700)
* aec900ae - feat: Add pactffi_message_with_metadata_v2 (tien.xuan.vo, Mon Nov 20 20:14:26 2023 +0700)
* 86aa29aa - docs: Update safety comment for pactffi_message_reify (tien.xuan.vo, Tue Nov 21 11:32:22 2023 +0700)
* e26bf72d - feat: Apply generators to message's contents and metadata (tien.xuan.vo, Mon Nov 20 16:26:39 2023 +0700)
* b293ddce - docs: Update safety comment for pactffi_message_metadata_iter_next (tien.xuan.vo, Tue Nov 21 11:24:27 2023 +0700)
* 3df61ab7 - feat: Apply generators to message's metadata (tien.xuan.vo, Mon Nov 20 15:08:20 2023 +0700)
* 755909b8 - Merge pull request #340 from tienvx/generate-message-contents (Ronald Holshausen, Tue Nov 21 15:19:05 2023 +1100)
* a7b81fe7 - docs: Update safety comment for pactffi_message_get_contents (tien.xuan.vo, Tue Nov 21 11:12:01 2023 +0700)
* b57a717e - feat: Apply generators to message's contents (tien.xuan.vo, Mon Nov 20 14:45:40 2023 +0700)
* 7405ccbe - Merge branch 'release/0.4.10' (Ronald Holshausen, Tue Nov 14 05:50:08 2023 +1100)
* 08786976 - bump version to 0.4.11 (Ronald Holshausen, Tue Nov 14 04:08:57 2023 +1100)

# 0.4.10 - Bugfix Release

* 3f0ae7f1 - chore: Upgrade pact_matching to 1.1.7 (Ronald Holshausen, Tue Nov 14 03:10:25 2023 +1100)
* 826758a6 - chore: Upgrade pact_models to 1.1.12 (Ronald Holshausen, Mon Nov 13 17:25:21 2023 +1100)
* a4200b07 - feat(FFI): Add with_binary_body method to set a pure binary body on an interaction #336 (Ronald Holshausen, Mon Nov 13 16:38:08 2023 +1100)
* 944f67dc - chore: add deprecation warning for new_async_message #334 (Ronald Holshausen, Mon Nov 13 15:24:11 2023 +1100)
* c172bcb5 - feat: Add functions for Pact handle -> pointer compatibility #333 (Ronald Holshausen, Mon Nov 13 14:41:51 2023 +1100)
* 7a1d6b37 - chore(FFI): Update the doc comment for multiple query parameter values with a matcher #332 (Ronald Holshausen, Wed Oct 25 08:48:35 2023 +1100)
* 3240fa91 - chore(FFI): add a test for multiple query parameter values with a matcher #332 (Ronald Holshausen, Wed Oct 25 06:55:15 2023 +1100)
* dc104c03 - fix(FFI): fix for FFI C examples #328 (Ronald Holshausen, Fri Oct 20 10:55:46 2023 +1100)
* 7b05bd66 - fix(FFI): fix for musl build #328 (Ronald Holshausen, Fri Oct 20 10:36:23 2023 +1100)
* 9a389f46 - fix(FFI): return the number of bytes written by pactffi_get_error_message #328 (Ronald Holshausen, Fri Oct 20 09:59:02 2023 +1100)
* 335d52dc - fix: Only specification v3 or higher support binary body's content type matching rule (tien.xuan.vo, Mon Oct 16 22:11:54 2023 +0700)
* c0201a77 - bump version to 0.4.10 (Ronald Holshausen, Fri Sep 22 11:25:07 2023 +1000)

# 0.4.9 - Bugfix Release

* 04bad264 - chore: Upgrade pact_matching to 1.1.6 (Ronald Holshausen, Fri Sep 22 11:03:38 2023 +1000)
* 9f8dde99 - fix: Nested array contains matchers where not having their values propogated to the outer matcher #324 (Ronald Holshausen, Fri Sep 22 10:18:56 2023 +1000)
* 0bbb66e3 - Merge pull request #323 from tienvx/ffi-with-multipart-file-v2 (Ronald Holshausen, Thu Sep 21 12:13:07 2023 +1000)
* b6cb715a - Merge pull request #319 from tienvx/match-string-value-using-content-type (Ronald Holshausen, Thu Sep 21 12:05:15 2023 +1000)
* e7b5cced - Merge pull request #321 from YOU54F/chore/enable_aarch64-unknown-linux-gnu (Ronald Holshausen, Thu Sep 21 08:31:57 2023 +1000)
* 53a08108 - feat: Add ffi function pactffi_with_multipart_file_v2 (tien.xuan.vo, Sat Sep 16 16:08:53 2023 +0700)
* b11e2808 - chore: renable aarch64-unknown-linux-gnu (Yousaf Nabi, Fri Sep 15 20:13:13 2023 +0100)
* 63b7cf9d - fix: Allow matching string values using content type matching rule (tien.xuan.vo, Mon Sep 11 18:32:24 2023 +0700)
* 89a18dab - bump version to 0.4.9 (Ronald Holshausen, Tue Aug 29 14:57:35 2023 +1000)

# 0.4.8 - Bugfix Release

* a3025b14 - chore(pact_ffi): Disable aarch64-unknown-linux-gnu target from release build (Ronald Holshausen, Tue Aug 29 14:53:10 2023 +1000)
* c924b9ce - chore: Upgrade pact_verifier to 1.0.3 (Ronald Holshausen, Tue Aug 29 10:04:18 2023 +1000)
* d592cd8b - chore: Upgrade pact_mock_server to 1.2.3 (Ronald Holshausen, Tue Aug 29 09:25:14 2023 +1000)
* 37fa901c - fix(FFI): Stupid Windows #314 (Ronald Holshausen, Mon Aug 28 15:54:08 2023 +1000)
* 7fc7bbc7 - fix(FFI): When appending parts to an existing multipart body, matching rules should still be configured for the new part #314 (Ronald Holshausen, Mon Aug 28 15:36:57 2023 +1000)
* bae6b5a9 - fix(FFI): Allow pactffi_with_multipart_file to append parts to an existing multipart body #314 (Ronald Holshausen, Mon Aug 28 15:32:25 2023 +1000)
* 3ec99c41 - chore: Upgrade pact_matching to 1.1.5 (Ronald Holshausen, Fri Aug 18 15:40:02 2023 +1000)
* 6df8ce82 - fix(pact_verifier): Fix missing PATCH version in plugin's version (tienvx, Mon Aug 7 18:14:51 2023 +0700)
* e6484f39 - fix(FFI): Check for the intermediate JSON format when setting the body contents with XML #305 (Ronald Holshausen, Mon Aug 7 16:44:47 2023 +1000)
* 66648b4c - fix(FFI): Guard against header names being passed in different case #305 (Ronald Holshausen, Mon Aug 7 16:06:11 2023 +1000)
* e4da3e42 - chore: Upgrade pact_models to 1.1.11 (Ronald Holshausen, Mon Aug 7 13:59:34 2023 +1000)
* 24ed7835 - chore: Upgrade pact-models to 1.1.10 (Ronald Holshausen, Fri Aug 4 16:11:24 2023 +1000)
* ef920ba7 - bump version to 0.4.8 (Ronald Holshausen, Thu Jul 27 15:28:46 2023 +1000)

# 0.4.7 - Bugfix Release

* 04af3923 - chore: Upgrade pact_verifier to 1.0.2 (Ronald Holshausen, Thu Jul 27 15:17:57 2023 +1000)
* 0586fcf1 - chore: Upgrade pact_mock_server to 1.2.2 (Ronald Holshausen, Thu Jul 27 14:45:02 2023 +1000)
* 8f88192e - chore: Upgrade pact_matching to 1.1.4 (Ronald Holshausen, Thu Jul 27 14:35:27 2023 +1000)
* 4a01919a - chore: Upgrade pact_models to 1.1.9 (Ronald Holshausen, Thu Jul 27 10:24:00 2023 +1000)
* 8a22a66a - fix: correct equality error message to match compatibility-suite (Ronald Holshausen, Wed Jul 26 14:30:48 2023 +1000)
* 2e45e223 - fix: Update matching error messages to be in line with the compatibility-suite (Ronald Holshausen, Tue Jul 25 17:42:03 2023 +1000)
* 0459d40c - fix(pact_matching): Number of keys were still be compared when an EachKeys matcher is defined #301 (Ronald Holshausen, Mon Jul 17 13:37:05 2023 +1000)
* e69eef09 - bump version to 0.4.7 (Ronald Holshausen, Thu Jul 13 10:26:33 2023 +1000)

# 0.4.6 - Bugfix Release

* 7527fd32 - chore: Correct release script (Ronald Holshausen, Thu Jul 13 10:23:51 2023 +1000)
* 4f448b1f - fix(pact_matching): EachValue matcher was not applying the associated matching rules correctly #299 (Ronald Holshausen, Wed Jul 12 16:28:16 2023 +1000)
* 63c1a8e6 - chore: Upgrade pact_verifier to 1.0.1 (Ronald Holshausen, Wed Jul 12 11:27:36 2023 +1000)
* e07ca36c - chore: Upgrade pact_mock_server to 1.2.1 (Ronald Holshausen, Tue Jul 11 14:47:17 2023 +1000)
* 348eb3f3 - chore: Upgrade pact_matcing to 1.1.3 (Ronald Holshausen, Tue Jul 11 11:38:26 2023 +1000)
* f2ae77ba - chore: Upgrade pact-plugin-driver to 0.4.5 (Ronald Holshausen, Mon Jul 10 17:15:20 2023 +1000)
* b18b9dff - chore: Upgrade pact_matching to 1.1.2 (Ronald Holshausen, Mon Jul 10 16:42:27 2023 +1000)
* 1deca59a - chore: Upgrade pact_models to 1.1.8 (Ronald Holshausen, Mon Jul 10 16:15:43 2023 +1000)
* bfd731b8 - fix(FFI): Deal with multi-value headers correctly #300 (Ronald Holshausen, Mon Jul 10 14:58:51 2023 +1000)
* b7b7b9c0 - fix(pact_models): MatchingRule::from_json shoud support integration format (Ronald Holshausen, Mon Jul 10 12:59:54 2023 +1000)
* 2662cdfc - chore: Upgrade pact_models to 1.1.7 (Ronald Holshausen, Thu Jul 6 10:27:25 2023 +1000)
* e95ae4d0 - chore: Upgrade pact_models to 1.1.6 (Ronald Holshausen, Thu Jun 22 15:40:55 2023 +1000)
* 244f1fdb - feat(compatibility-suite): Implemented scenarios for no provider state callback configured + request filters (Ronald Holshausen, Fri Jun 16 11:36:30 2023 +1000)
* 834f77cc - chore: Upgrade pact_mock_server to 1.2.0 (Ronald Holshausen, Wed Jun 14 15:22:11 2023 +1000)
* e58aa917 - fix: no need to wrap the Pact for a mock server in a mutex (mock server is already behind a mutex) as this can cause deadlocks #274 (Ronald Holshausen, Wed Jun 14 13:26:54 2023 +1000)
* bc68ed7f - chore: Upgrade pact_models to 1.1.4 (Ronald Holshausen, Thu Jun 1 10:22:38 2023 +1000)
* 397c837f - chore: Upgrade pact_models to 1.1.3 (fixes MockServerURL generator) (Ronald Holshausen, Mon May 29 15:12:22 2023 +1000)
* 4555d0c4 - bump version to 0.4.6 (Ronald Holshausen, Tue May 23 16:45:37 2023 +1000)

# 0.4.5 - Bugfix Release

* 6137bba1 - chore: fix deps (Ronald Holshausen, Tue May 23 16:41:20 2023 +1000)
* e7e889f0 - Revert "update changelog for release 0.4.5" (Ronald Holshausen, Tue May 23 16:40:53 2023 +1000)
* 261d34bb - update changelog for release 0.4.5 (Ronald Holshausen, Tue May 23 16:40:08 2023 +1000)
* 1dce67a5 - chore: cleanup deprecation warnings (Ronald Holshausen, Tue May 23 16:34:15 2023 +1000)
* ca1e37fb - chore: Upgrade pact_verifier to 1.0.0 (Ronald Holshausen, Tue May 23 15:16:08 2023 +1000)
* 8e9bd503 - chore: Upgrade pact_mock_server to 1.1.0 (Ronald Holshausen, Tue May 23 12:20:01 2023 +1000)
* 8f27f9bd - chore: Upgrade pact-plugin-driver to 0.4.4 (Ronald Holshausen, Tue May 23 11:55:23 2023 +1000)
* ac2e24da - chore: Use "Minimum version, with restricted compatibility range" for all Pact crate versions (Ronald Holshausen, Tue May 23 11:46:52 2023 +1000)
* 6df4670c - chore: Upgrade pact_matching to 1.1.1 (Ronald Holshausen, Tue May 23 11:32:51 2023 +1000)
* 54887690 - chore: Bump pact_matching to 1.1 (Ronald Holshausen, Tue May 23 11:13:14 2023 +1000)
* f72f8191 - feat: Implemented the remaining V1 HTTP consumer compatability suite feature (Ronald Holshausen, Thu May 18 14:12:40 2023 +1000)
* 261ecf47 - fix: Add RefUnwindSafe trait bound to all Pact and Interaction uses (Ronald Holshausen, Mon May 15 13:59:31 2023 +1000)
* feb4e5ca - chore: cleanup some deprecation warnings (Ronald Holshausen, Mon May 8 11:16:20 2023 +1000)
* 7277a355 - chore: fix test on CI (Ronald Holshausen, Tue May 2 12:03:54 2023 +1000)
* 95664129 - feat: add method pactffi_given_with_params to allow a provider state to be repeated with different values (Ronald Holshausen, Tue May 2 11:41:10 2023 +1000)
* 99e7c08a - chore: drop nightly-2022-12-01 fixed https://github.com/rust-lang/rust/issues/105886 (Yousaf Nabi, Fri Apr 21 17:26:01 2023 +0100)
* cece8369 - bump version to 0.4.5 (Ronald Holshausen, Tue Apr 18 16:53:28 2023 +1000)

# 0.4.4 - Bugfix Release

* 8cfc3d79 - chore: Upgrade pact_verifier to 0.15.3 (Ronald Holshausen, Tue Apr 18 15:22:26 2023 +1000)
* 6a71b12d - chore: Upgrade pact_mock_server to 1.0.2 (Ronald Holshausen, Tue Apr 18 13:30:21 2023 +1000)
* 0bcba082 - chore: Upgrade pact_matching to 1.0.8 (Ronald Holshausen, Tue Apr 18 13:14:38 2023 +1000)
* 6c14abfd - chore: Upgrade pact_models to 1.0.13 (Ronald Holshausen, Tue Apr 18 13:00:01 2023 +1000)
* ce16d43f - chore: Upgrade pact-plugin-driver to 0.4.2 (supports auto-installing known plugins) (Ronald Holshausen, Tue Apr 18 11:49:52 2023 +1000)
* a4192b38 - feat: add FFI function to validate a Date/Time string against a format #265 (Ronald Holshausen, Mon Apr 17 12:48:01 2023 +1000)
* ac136ed5 - doc: Update the doc comment for pactffi_given_with_param to indicate the value must be JSON #263 (Ronald Holshausen, Mon Apr 17 11:09:04 2023 +1000)
* 10bf1a48 - chore: Upgrade pact_models to 1.0.12 (fixes generators hash function) (Ronald Holshausen, Mon Apr 17 10:31:09 2023 +1000)
* 84b9d9e9 - fix: Upgrade pact models to 1.0.11 (fixes generated key for V4 Pacts) (Ronald Holshausen, Fri Apr 14 17:10:58 2023 +1000)
* 669f7812 - chore: Upgrade pact_models to 1.0.10 (Ronald Holshausen, Thu Apr 13 15:32:34 2023 +1000)
* 2d436288 - fix(FFI): Fix test failing on CI on Alpine #262 (Ronald Holshausen, Thu Apr 6 11:38:41 2023 +1000)
* b2d7ec3a - fix(FFI): Fix test failing on CI because the plugins dir does not exist #262 (Ronald Holshausen, Thu Apr 6 11:09:57 2023 +1000)
* 3e44b376 - chore: remove debug statement (Ronald Holshausen, Thu Apr 6 11:02:46 2023 +1000)
* 96ac10c1 - fix(FFI): log and capture the error when the verification fails #262 (Ronald Holshausen, Thu Apr 6 10:52:52 2023 +1000)
* 779a59f0 - fix: Upgrade pact-plugin-driver to 0.4.1 (fixes an issue introduced in 0.4.0 with shared channels to plugins) (Ronald Holshausen, Wed Apr 5 17:01:18 2023 +1000)
* 81fbfa7f - bump version to 0.4.4 (Ronald Holshausen, Wed Apr 5 14:31:56 2023 +1000)

# 0.4.3 - Bugfix Release

* 0af00359 - fix: use a shared global tokio runtime so shared plugin connections can be used (Ronald Holshausen, Wed Apr 5 14:05:11 2023 +1000)
* 33ef054f - bump version to 0.4.3 (Ronald Holshausen, Wed Apr 5 11:17:18 2023 +1000)

# 0.4.2 - Bugfix Release

* d216b5d5 - chore: Update dependencies (Ronald Holshausen, Wed Apr 5 11:10:52 2023 +1000)
* 799f886d - chore: remove use of deprecated methods (Ronald Holshausen, Wed Apr 5 11:07:22 2023 +1000)
* 4f62ee5d - chore: Upgrade pact_verifier to 0.15.2 (Ronald Holshausen, Wed Apr 5 10:16:07 2023 +1000)
* 81a9b306 - chore: Upgrade pact_mock_server to 1.0.1 (Ronald Holshausen, Tue Apr 4 15:40:20 2023 +1000)
* 126cf462 - chore: Upgrade pact_matching to 1.0.7 (Ronald Holshausen, Tue Apr 4 15:12:28 2023 +1000)
* 6f0c4b2f - feat: Upgrade pact-plugin-driver to 0.4.0 which uses a shared gRPC channel to each plugin (Ronald Holshausen, Tue Apr 4 14:32:36 2023 +1000)
* 8310b09c - chore: Upgrade pact_verifier to 0.15.1 (Ronald Holshausen, Wed Mar 15 15:25:31 2023 +1100)
* 11c701b4 - fix: Upgrade pact_matching to 1.0.6 (fixes some issues with matching HTTP headers) (Ronald Holshausen, Wed Mar 15 14:54:54 2023 +1100)
* e96bc54e - fix: Upgrade pact_models to 1.0.9 (fixes issues with headers) (Ronald Holshausen, Wed Mar 15 14:31:00 2023 +1100)
* f7e0b669 - chore: Upgrade pact_models to 1.0.8 (Ronald Holshausen, Wed Mar 15 12:19:22 2023 +1100)
* 57728a01 - chore: update pact-plugin-driver to 0.3.3 (Ronald Holshausen, Tue Mar 14 17:19:20 2023 +1100)
* 9629c351 - chore: dump minor version of pact_verifier as some signatures have changed (Ronald Holshausen, Thu Mar 2 12:10:35 2023 +1100)
* c9333f94 - feat: add option to generate JUnit XML report format for consumption by CI servers #257 (Ronald Holshausen, Thu Mar 2 10:48:56 2023 +1100)
* af786d20 - bump version to 0.4.2 (Ronald Holshausen, Thu Feb 16 14:59:56 2023 +1100)

# 0.4.1 - Maintenance Release

* c368c651 - fix: Pass any custom header values on to the plugin verification call (Ronald Holshausen, Thu Feb 16 13:52:03 2023 +1100)
* 0676047e - chore: Upgrade pact-plugin-driver to 0.3.2 (Ronald Holshausen, Thu Feb 16 12:09:46 2023 +1100)
* 2365c6ce - chore: cleanup deprecation messages (Ronald Holshausen, Thu Feb 16 10:26:09 2023 +1100)
* 7589b9b0 - chore: Bump pact_mock_server version to 1.0.0 (Ronald Holshausen, Fri Feb 10 14:43:53 2023 +1100)
* fa45296c - chore: Update pact_verifier to 0.13.21 (Ronald Holshausen, Fri Feb 10 13:37:48 2023 +1100)
* 019bd2fe - chore: Upgrade pact_matching to 1.0.5 (Ronald Holshausen, Wed Feb 8 13:53:15 2023 +1100)
* 1e7331f1 - fix: Upgrade plugin driver to 0.3.1 (Ronald Holshausen, Wed Feb 8 13:28:07 2023 +1100)
* 0f4178e5 - chore: Upgrade pact_matching to 1.0.4 (Ronald Holshausen, Mon Feb 6 15:40:43 2023 +1100)
* 0b70060f - chore: Upgrade pact-plugin-driver and base64 crates (supports message metadata) (Ronald Holshausen, Mon Feb 6 14:56:29 2023 +1100)
* a56bc056 - fix(FFI): Message metadata was not being passed on to the mock server (Ronald Holshausen, Fri Feb 3 11:30:32 2023 +1100)
* f391b4de - bump version to 0.4.1 (Ronald Holshausen, Wed Jan 11 16:57:57 2023 +1100)

# 0.4.0 - Add FFI functions for plugin authors to parse Pact JSON and get matching rules and generators + bugfixes

* fbc4dbe1 - chore: Upgrade pact_verifier to 0.13.20 (Ronald Holshausen, Wed Jan 11 16:06:21 2023 +1100)
* c1b22f1c - chore: Upgrade pact_matching to 1.0.3 (Ronald Holshausen, Wed Jan 11 15:19:29 2023 +1100)
* 7d84d941 - chore: Upgrade pact_models to 1.0.4 (Ronald Holshausen, Wed Jan 11 14:33:13 2023 +1100)
* 3760c2b3 - fix: matcher_from_integration_json in mockserver/bodies.rs doesn't support all MatchingRules #247 (Ronald Holshausen, Wed Jan 11 13:58:33 2023 +1100)
* a8abf5df - chore: log spans at trace level to reduce the log entry size at other log levels #243 (Ronald Holshausen, Tue Jan 10 09:00:52 2023 +1100)
* 9494cc7a - feat(FFI): add iterators over the interaction generators (Ronald Holshausen, Mon Jan 9 11:29:55 2023 +1100)
* 8ed492c3 - feat(FFI): add iterators over the interaction matching rules (Ronald Holshausen, Fri Jan 6 15:18:34 2023 +1100)
* f67340dc - feat(FFI): Support MessageMetadataIterator with V4 messages (Ronald Holshausen, Fri Jan 6 09:13:53 2023 +1100)
* 0e158011 - feat(FFI): add FFI functions to get the content opaque pointer (Ronald Holshausen, Thu Jan 5 16:38:55 2023 +1100)
* 9fea001c - chore(FFI): bump minor version (Ronald Holshausen, Thu Jan 5 16:36:09 2023 +1100)
* 2b82fb53 - feat(FFI): Add functions to downcast to concrete interaction types (Ronald Holshausen, Thu Jan 5 15:40:54 2023 +1100)
* bea076b3 - feat(FFI): Add iterator over interactions in Pact model (Ronald Holshausen, Thu Jan 5 14:29:44 2023 +1100)
* f190bd59 - feat(FFI): add FFI functions to get a Pact consumer and provider (Ronald Holshausen, Thu Jan 5 12:38:56 2023 +1100)
* 34a67cb9 - fix: when loading pacts from a dir, filter by the provider name #233 (Ronald Holshausen, Wed Jan 4 18:12:28 2023 +1100)
* 00c77e92 - bump version to 0.3.20 (Ronald Holshausen, Thu Dec 22 15:44:18 2022 +1100)

# 0.3.19 - Fix for V3 Message ignores the interaction ID when loaded from a Pact Broker

* 1bdb1054 - chore: Upgrade pact_models to 1.0.3 #239 (Ronald Holshausen, Thu Dec 22 15:37:53 2022 +1100)
* 09479b91 - Merge branch 'fix/release-0.3.18' (Ronald Holshausen, Mon Dec 19 18:52:51 2022 +1100)
* 68b7a37d - bump version to 0.3.19 (Ronald Holshausen, Mon Dec 19 17:24:59 2022 +1100)

# 0.3.18 - Bugfixes + Support generators in plugins

* c55a7758 - chore: Upgrade pact_verifier to 0.13.19 (Ronald Holshausen, Mon Dec 19 16:20:24 2022 +1100)
* 81e55220 - chore: Upgrade pact_mock_server to 0.9.7 (Ronald Holshausen, Mon Dec 19 16:04:55 2022 +1100)
* e827f591 - chore: Upgrade pact_matching to 1.0.2 (Ronald Holshausen, Mon Dec 19 15:30:14 2022 +1100)
* dece8df7 - Merge pull request #235 from leonasa/feat/allow-non-deafult-json-content-type (Ronald Holshausen, Mon Dec 19 13:19:29 2022 +1100)
* 86344804 - chore: fix FFI CI build (Ronald Holshausen, Mon Dec 19 12:57:42 2022 +1100)
* 61e4d69d - fix: cbindgen fails in latest nightly rust (Ronald Holshausen, Mon Dec 19 12:48:31 2022 +1100)
* 4b1ba4a2 - fix: cbindgen fails in latest nightly rust (Ronald Holshausen, Mon Dec 19 12:07:43 2022 +1100)
* 166e9f86 - chore: correct clippy error (Ronald Holshausen, Fri Dec 16 17:40:15 2022 +1100)
* 82e20053 - chore: correct clippy error (Ronald Holshausen, Fri Dec 16 17:18:34 2022 +1100)
* 5fbb0d6a - feat: Upgrade plugin driver to 0.2.2 (supports passing a test context to support generators) (Ronald Holshausen, Fri Dec 16 16:38:03 2022 +1100)
* 5251fcf5 - fix: error caused an internal mutex to be poisoned (Ronald Holshausen, Fri Dec 16 16:34:19 2022 +1100)
* 1ab47c6f - chore: Upgrade Tokio to latest (Ronald Holshausen, Fri Dec 16 16:31:31 2022 +1100)
* dbe56950 - bump version to 0.3.18 (Ronald Holshausen, Thu Dec 15 13:26:05 2022 +1100)
* c1ccb62f - feat: allow non deafult json content-type (Leonardo Santana, Mon Dec 12 15:26:22 2022 -0500)

# 0.3.17 - Bugfix Release

* ec2ed51d - fix(FFI): use a multi-threaded reactor for FFI setup_contents call to plugins (Ronald Holshausen, Thu Dec 15 12:22:24 2022 +1100)
* 46254545 - chore: Upgrade pact_verifier to 0.13.18 (Ronald Holshausen, Wed Dec 14 17:15:22 2022 +1100)
* fb2f4204 - chore: Upgrade pact_matching to 1.0.1 (Ronald Holshausen, Wed Dec 14 17:03:31 2022 +1100)
* 8be00f0c - chore: Upgrade pact-plugin-driver to 0.2.1 (Ronald Holshausen, Wed Dec 14 14:55:32 2022 +1100)
* f0387856 - bump version to 0.3.17 (Ronald Holshausen, Mon Dec 12 14:18:18 2022 +1100)

# 0.3.16 - Support for plugin authors

* 9be00044 - chore: Upgrade pact_mock_server to 0.9.6 (Ronald Holshausen, Mon Dec 12 10:06:25 2022 +1100)
* 4f366ac5 - fix(FFI): broken build after upgrading pact_models (Ronald Holshausen, Fri Dec 9 17:31:40 2022 +1100)
* e7a1b9f2 - chore: Upgrade pact_matching to 1.0 and plugin driver to 0.2 (Ronald Holshausen, Fri Dec 9 17:29:33 2022 +1100)
* cfb2c03f - chore: pactffi_matches_binary_value_test will not work on Windows (Ronald Holshausen, Thu Dec 1 16:49:29 2022 +1100)
* 6fe19b9e - feat(FFI): add functions for matching common values (Ronald Holshausen, Thu Dec 1 16:26:39 2022 +1100)
* 7756d305 - fix(FFI): Replaced the matching rule union type with 3 FFI functions which should support Go better (Ronald Holshausen, Wed Nov 30 16:10:15 2022 +1100)
* fbdcaabc - bump version to 0.3.16 (Ronald Holshausen, Mon Nov 28 16:09:44 2022 +1100)

# 0.3.15 - Bugfix + FFI functions to support plugin authors

* 2f0ada6b - chore: Upgrade pact_verifier to 0.13.16 (Ronald Holshausen, Mon Nov 28 15:08:47 2022 +1100)
* 246c0730 - chore: Upgrade pact_mock_server to 0.9.5 (Ronald Holshausen, Mon Nov 28 14:52:37 2022 +1100)
* 2802fffd - chore: Upgrade pact_matching to 0.12.15 (Ronald Holshausen, Mon Nov 28 14:29:43 2022 +1100)
* c9721fd5 - chore: Upgrade pact_models to 1.0.1 and pact-plugin-driver to 0.1.16 (Ronald Holshausen, Mon Nov 28 14:10:53 2022 +1100)
* 43a8cae1 - chore: clean up deprecation warnings (Ronald Holshausen, Mon Nov 28 13:19:31 2022 +1100)
* 2e5823a0 - feat: add custom-header to the old FFI args for implementations that have not moved to handles (Ronald Holshausen, Fri Nov 25 11:09:46 2022 +1100)
* bdab5130 - chore: cleanup compiler warnings (Ronald Holshausen, Mon Nov 14 14:57:24 2022 +1100)
* 570e33c1 - feat: add FFI function to return generator JSON (Ronald Holshausen, Mon Nov 14 14:53:11 2022 +1100)
* 72b22116 - chore: update test (Ronald Holshausen, Mon Nov 14 13:43:35 2022 +1100)
* 7a2686e0 - feat: add function to get matching rule as JSON (Ronald Holshausen, Mon Nov 14 13:35:46 2022 +1100)
* cfc565e3 - feat: add docs on the matching rule IDs (Ronald Holshausen, Mon Nov 14 12:08:26 2022 +1100)
* 18e1e113 - feat: add an iterator over the matching rules from a matching definition expression (Ronald Holshausen, Mon Nov 14 12:02:41 2022 +1100)
* e21d3454 - feat: add FFI function to parse JSON to a Pact model (Ronald Holshausen, Fri Nov 11 17:00:36 2022 +1100)
* b7c010eb - feat: add generator FFI functions (Ronald Holshausen, Fri Nov 11 14:54:39 2022 +1100)
* f7b561ee - feat: add FFI function to get the generator from a matching definition (Ronald Holshausen, Fri Nov 11 11:16:58 2022 +1100)
* 768a132b - feat: add FFI function to parse a matching definition expression (Ronald Holshausen, Thu Nov 10 18:18:39 2022 +1100)
* 2c7788fb - bump version to 0.3.15 (Ronald Holshausen, Mon Nov 7 15:11:44 2022 +1100)

# 0.3.14 - Bugfix Release

* 0ed385bf - chore: correct version in manifest file after previous release was done wrong (Ronald Holshausen, Mon Nov 7 15:06:05 2022 +1100)
* 5f75239e - Revert "update changelog for release 0.3.13" (Ronald Holshausen, Mon Nov 7 15:03:15 2022 +1100)
* a7ca2e2f - update changelog for release 0.3.13 (Ronald Holshausen, Mon Nov 7 15:02:31 2022 +1100)
* d6ea8357 - chore: Upgrade all dependencies (Ronald Holshausen, Mon Nov 7 14:58:56 2022 +1100)
* f43e7851 - chore: Upgrade pact_verifier to 0.13.15 (Ronald Holshausen, Mon Nov 7 14:13:26 2022 +1100)
* a3110bd6 - chore: Upgrade pact_mock_server to 0.9.4 (Ronald Holshausen, Mon Nov 7 11:50:05 2022 +1100)
* 123060e3 - chore: Upgrade pact_matching to 0.12.14 (Ronald Holshausen, Mon Nov 7 11:34:36 2022 +1100)
* 577824e7 - fix: Upgrade pact_models to 1.0 and pact-plugin-driver to 0.1.15 to fix cyclic dependency issue (Ronald Holshausen, Mon Nov 7 11:14:20 2022 +1100)
* e1f985ad - chore: Upgrade pact_models to 0.4.6 and pact-plugin-driver to 0.1.14 (Ronald Holshausen, Fri Nov 4 16:38:36 2022 +1100)
* 6ad00a5d - fix: Update onig to latest master to fix  Regex Matcher Fails On Valid Inputs #214 (Ronald Holshausen, Fri Nov 4 15:23:50 2022 +1100)
* d976db0c - fix: panicked at 'called  on a  value' when FFI LevelFilter == Off #226 (Ronald Holshausen, Fri Nov 4 13:47:20 2022 +1100)
* 9dad5d2a - fix: ffi.pactffi_logger_attach_sink causes seg fault if log directory doesn't exist #226 (Ronald Holshausen, Fri Nov 4 12:26:15 2022 +1100)
* 3bc69045 - chore: correct FFI function doc comments (Ronald Holshausen, Wed Nov 2 10:36:46 2022 +1100)

# 0.3.13 - Bugfix Release

* eb505b7f - Merge pull request #221 from pact-foundation/feat/multiple-transports-in-ffi (Matt Fellows, Wed Oct 12 10:11:15 2022 +1100)
* 965a1c41 - fix: Upgrade plugin driver to 0.1.13 (fixes issue loading plugin when there are multiple versions for the same plugin) (Ronald Holshausen, Wed Oct 5 17:29:37 2022 +1100)
* dda213e3 - feat(ffi): support adding transports to provider via pactffi_verifier_add_provider_transport (Matt Fellows, Wed Oct 5 10:49:07 2022 +1100)
* 2014c432 - bump version to 0.3.13 (Ronald Holshausen, Wed Sep 28 10:54:32 2022 +1000)

# 0.3.12 - Bugfix Release

* b7bb9cd1 - chore: Upgrade pact_verifier crate to 0.13.14 (Ronald Holshausen, Wed Sep 28 10:34:48 2022 +1000)
* cf913949 - chore: update the list of matcher types (Ronald Holshausen, Wed Sep 28 10:20:18 2022 +1000)
* 02d9e2cb - chore: Upgrade pact matching crate to 0.12.12 (Ronald Holshausen, Wed Sep 28 10:11:11 2022 +1000)
* c96bd173 - chore(FFI): Array contains variants need to be recursivly processed #216 (Ronald Holshausen, Wed Sep 28 09:18:50 2022 +1000)
* 6fac3c80 - chore(FFI): add deprecation notice to matcher_from_integration_json (Ronald Holshausen, Tue Sep 27 17:59:39 2022 +1000)
* b8be05c1 - fix(FFI): Use a star for the path with values matcher #216 (Ronald Holshausen, Tue Sep 27 17:50:32 2022 +1000)
* 937db6af - chore(FFI): add test with nested matching rules #216 (Ronald Holshausen, Tue Sep 27 16:34:27 2022 +1000)
* 7d906324 - chore: document the FFI integration JSON format (Ronald Holshausen, Tue Sep 27 15:38:43 2022 +1000)
* 671e58fc - chore: display the JSON when we can not parse it (Ronald Holshausen, Tue Sep 13 14:19:50 2022 +1000)
* 60b2b642 - chore: Upgrade pact-plugin-driver to 0.1.12 (Ronald Holshausen, Mon Sep 12 17:44:13 2022 +1000)
* ac4fe73f - chore: fix to release scripts (Ronald Holshausen, Wed Sep 7 10:51:01 2022 +1000)
* b6ab1785 - chore: correct cargo manifest after release (Ronald Holshausen, Wed Sep 7 10:46:44 2022 +1000)
* 333bb92a - bump version to 0.3.12 (Ronald Holshausen, Wed Sep 7 10:44:40 2022 +1000)

# 0.3.11 - Bugfix Release

* cdb555f8 - fix: Upgrade pact_verifier to 0.13.13 (Ronald Holshausen, Wed Sep 7 09:53:05 2022 +1000)
* 1c5bde33 - bump version to 0.3.11 (Ronald Holshausen, Wed Aug 31 17:05:22 2022 +1000)

# 0.3.10 - Add option to not fail verification if no Pacts are found

* 5c1d4293 - chore: Upgrade pact_verifier crate to 0.13.12 (Ronald Holshausen, Wed Aug 31 16:09:16 2022 +1000)
* c128d22b - feat(FFI): add pactffi_verifier_set_no_pacts_is_error function #213 (Ronald Holshausen, Wed Aug 31 15:37:04 2022 +1000)
* ac6e8858 - chore: fix release script (Ronald Holshausen, Fri Aug 26 20:25:33 2022 +1000)
* 592975ec - bump version to 0.3.10 (Ronald Holshausen, Fri Aug 26 15:12:33 2022 +1000)
* 9adb7592 - update changelog for release 0.3.9 (Ronald Holshausen, Fri Aug 26 15:09:34 2022 +1000)
* 508005a3 - chore: clean up some compiler warnings (Ronald Holshausen, Fri Aug 26 13:48:58 2022 +1000)
* f8db90d2 - fix: Upgrade pact_models to 0.4.5 - fixes FFI bug with generators for request paths (Ronald Holshausen, Fri Aug 26 11:44:08 2022 +1000)
* a436392a - bump version to 0.3.9 (Ronald Holshausen, Fri Aug 19 11:38:18 2022 +1000)
* a74952cb - update changelog for release 0.3.8 (Ronald Holshausen, Fri Aug 19 11:36:32 2022 +1000)
* 70d2c09a - chore: Build FFI lib with Debian stretch image #202 (Ronald Holshausen, Fri Aug 19 11:33:40 2022 +1000)
* 546d84f6 - bump version to 0.3.8 (Ronald Holshausen, Thu Aug 18 16:38:32 2022 +1000)

# 0.3.9 - Bugfix Release

* 508005a3 - chore: clean up some compiler warnings (Ronald Holshausen, Fri Aug 26 13:48:58 2022 +1000)
* f8db90d2 - fix: Upgrade pact_models to 0.4.5 - fixes FFI bug with generators for request paths (Ronald Holshausen, Fri Aug 26 11:44:08 2022 +1000)
* a436392a - bump version to 0.3.9 (Ronald Holshausen, Fri Aug 19 11:38:18 2022 +1000)
* a74952cb - update changelog for release 0.3.8 (Ronald Holshausen, Fri Aug 19 11:36:32 2022 +1000)
* 70d2c09a - chore: Build FFI lib with Debian stretch image #202 (Ronald Holshausen, Fri Aug 19 11:33:40 2022 +1000)
* 546d84f6 - bump version to 0.3.8 (Ronald Holshausen, Thu Aug 18 16:38:32 2022 +1000)

# 0.3.8 - Support FFI libs on Ubuntu 18.04

* 70d2c09a - chore: Build FFI lib with Debian stretch image #202 (Ronald Holshausen, Fri Aug 19 11:33:40 2022 +1000)
* 546d84f6 - bump version to 0.3.8 (Ronald Holshausen, Thu Aug 18 16:38:32 2022 +1000)

# 0.3.7 - Bugfix Release

* 5e52d685 - chore: Upgrade pact_verifier to 0.13.11 (Ronald Holshausen, Thu Aug 18 16:33:19 2022 +1000)
* 9d1e8e89 - chore: Upgrade pact_mock_server to 0.9.3 (Ronald Holshausen, Thu Aug 18 16:03:38 2022 +1000)
* 1d5fb787 - chore: Upgrade pact_matching to 0.12.11 (Ronald Holshausen, Thu Aug 18 15:07:23 2022 +1000)
* 32a70382 - chore: Upgrade pact_models (0.4.4), plugin driver (0.1.10), tracing and tracing core crates (Ronald Holshausen, Thu Aug 18 14:41:52 2022 +1000)
* 11d162a8 - chore: disable content type check tests ion windows (Ronald Holshausen, Wed Aug 17 17:08:14 2022 +1000)
* 10311301 - chore: disable content type check on windows (Ronald Holshausen, Wed Aug 17 16:49:19 2022 +1000)
* 65d05149 - fix: content type matcher was not being applied if content type was not octet_stream #171 (Ronald Holshausen, Wed Aug 17 16:32:43 2022 +1000)
* 3c5c45d4 - fix(FFI): pactffi_with_binary_file was incorrectly setting the response content type to application/octet-stream #171 (Ronald Holshausen, Wed Aug 17 14:46:03 2022 +1000)
* 033b5ee9 - bump version to 0.3.7 (Ronald Holshausen, Mon Aug 15 17:55:05 2022 +1000)

# 0.3.6 - Maintenance Release

* a41fe69c - chore: Upgrade pact_mock_server to 0.9.2 (Ronald Holshausen, Mon Aug 15 17:40:09 2022 +1000)
* 68ef5b4b - Revert "update changelog for release 0.3.6" (Ronald Holshausen, Mon Aug 15 17:31:28 2022 +1000)
* 00e01177 - update changelog for release 0.3.6 (Ronald Holshausen, Mon Aug 15 17:29:25 2022 +1000)
* e3bef155 - feat: Add ARM64 (aarch64) linux targets to the release build #160 (Ronald Holshausen, Mon Aug 15 16:13:22 2022 +1000)
* 78c05f29 - feat: add metric call when the mock server is shutdown via FFI function (Ronald Holshausen, Thu Aug 11 17:50:29 2022 +1000)
* 3324c1b3 - chore: Upgrade pact_verifier to 0.13.10 (Ronald Holshausen, Wed Aug 10 13:02:17 2022 +1000)
* 7b6a919b - chore: Upgrade pact_matching crate to 0.12.10 (Ronald Holshausen, Wed Aug 10 12:37:11 2022 +1000)
* 195ad07b - chore: Updated dependant crates (uuid, simplelog) (Ronald Holshausen, Wed Aug 10 10:22:07 2022 +1000)
* 49232caa - chore: Update pact plugin driver to 0.1.9 (Ronald Holshausen, Wed Aug 10 10:14:42 2022 +1000)
* a3fe5e7f - chore: Update pact models to 0.4.2 (Ronald Holshausen, Wed Aug 10 10:10:41 2022 +1000)
* 12ce4f6c - chore: missed some references to nightly-2022-04-12 (Ronald Holshausen, Mon Aug 8 13:35:26 2022 +1000)
* 63af7ce3 - chore: Fix FFI CI build - cbindgen no longer needs nightly-2022-04-12 (Ronald Holshausen, Mon Aug 8 13:20:50 2022 +1000)
* c8a63526 - chore: debug CI FFI build (Ronald Holshausen, Mon Aug 8 11:52:00 2022 +1000)
* 3a1449cb - feat: use the configured transport when provided (Ronald Holshausen, Wed Aug 3 13:20:17 2022 +1000)
* 8cc29482 - feat: add CLI options to provide different ports when there are different transports (Ronald Holshausen, Wed Aug 3 11:53:31 2022 +1000)
* 68c5444e - feat(FFI): add function to disable coloured output with the verifier (Ronald Holshausen, Mon Aug 1 14:07:28 2022 +1000)
* 4a7a935c - bump version to 0.3.6 (Ronald Holshausen, Mon Aug 1 12:12:59 2022 +1000)

# 0.3.5 - Support message interactions with FFI body functions

* 4c957894 - feat(FFI): updated doc comments for pactffi_with_multipart_file (Ronald Holshausen, Mon Aug 1 11:48:37 2022 +1000)
* 128ae7c3 - feat(FFI): update pactffi_with_binary_file function to support message interactions (Ronald Holshausen, Mon Aug 1 11:12:20 2022 +1000)
* ded4dc62 - feat(FFI): update pactffi_with_body function to support message interactions (Ronald Holshausen, Thu Jul 28 14:52:53 2022 +1000)
* 433ab442 - feat(FFI): add functions for getting/setting HTTP interaction bodies (Ronald Holshausen, Thu Jul 28 13:41:37 2022 +1000)
* e9c332bf - feat(FFI) - add FFI functions to set the req/res bodies of sync messages (Ronald Holshausen, Thu Jul 28 11:14:24 2022 +1000)
* 85207602 - fix(FFI): fixed race condition with Pact handle ids (Ronald Holshausen, Thu Jul 28 11:13:40 2022 +1000)
* 1a6eed7c - chore: add some debug logs for handle access functions (Ronald Holshausen, Wed Jul 27 15:54:02 2022 +1000)
* 8f112ad0 - feat: add FFI function to set a message contents as binary (Ronald Holshausen, Wed Jul 27 15:15:41 2022 +1000)
* cf40b7de - feat: add FFI function to set a message contents (Ronald Holshausen, Wed Jul 27 14:52:41 2022 +1000)
* 52cacd37 - chore: pinned dependency that does not compile on Alpine (Ronald Holshausen, Mon Jul 25 11:00:36 2022 +1000)
* b80064d1 - chore: correct function docs (Ronald Holshausen, Mon Jul 25 10:46:04 2022 +1000)
* eb8123aa - bump version to 0.3.5 (Ronald Holshausen, Mon Jul 25 10:21:25 2022 +1000)

# 0.3.4 - Bugfix Release

* e95d701d - fix(FFI): fix matching rule for paths #205 (Ronald Holshausen, Fri Jul 22 13:42:06 2022 +1000)
* b0fdbb6e - fix(FFI): fix matching rule for paths #205 (Ronald Holshausen, Fri Jul 22 13:34:46 2022 +1000)
* f634fa91 - fix(FFI): handle headers with multiple values correctly #205 (Ronald Holshausen, Fri Jul 22 13:12:15 2022 +1000)
* f0cde4e9 - fix(FFI): update the example in docs to use new function #205 (Ronald Holshausen, Thu Jul 21 17:28:41 2022 +1000)
* 52b70097 - fix(FFI): handle query parameters with multiple values correctly #205 (Ronald Holshausen, Thu Jul 21 17:20:48 2022 +1000)
* 2b808db7 - chore: Update pact_verifier to 0.13.9 (Ronald Holshausen, Wed Jul 20 12:44:24 2022 +1000)
* 40f7bdc4 - feat: add verification option to disable ANSI escape codes in output #203 (Ronald Holshausen, Wed Jul 20 12:18:12 2022 +1000)
* d069b3f2 - bump version to 0.3.4 (Ronald Holshausen, Fri Jul 1 10:30:30 2022 +1000)

# 0.3.3 - Bug fixes + Support publishing results from webhook calls

* 9a6c846f - chore: Upgrade pact_matching to 0.12.9 (Ronald Holshausen, Fri Jun 10 15:46:07 2022 +1000)
* b3f98a2c - chore: Upgrade pact_verifier to 0.13.8 (Ronald Holshausen, Tue Jun 7 11:07:24 2022 +1000)
* 18118e82 - feat: add retries to the provider state change calls #197 (Ronald Holshausen, Tue Jun 7 09:10:23 2022 +1000)
* 23c0c593 - bump version to 0.3.3 (Ronald Holshausen, Mon May 30 14:31:20 2022 +1000)

# 0.3.2 - Bugfix Release

* 42dcd525 - chore: Disable ANSI escape codes in logs as Pact .Net is unable to deal with them (Ronald Holshausen, Mon May 30 13:16:05 2022 +1000)
* 61fc3771 - chore: Upgrade pact_verifier to 0.13.7 (Ronald Holshausen, Mon May 30 12:21:12 2022 +1000)
* f42026d5 - chore: Upgrade pact_mock_server to 0.9.1 (Ronald Holshausen, Mon May 30 12:09:06 2022 +1000)
* bcddbcfb - chore: Upgrade pact_matching to 0.12.8 (Ronald Holshausen, Mon May 30 11:52:26 2022 +1000)
* 80256458 - chore: Upgrade pact-plugin-driver to 0.1.8 (Ronald Holshausen, Mon May 30 11:36:54 2022 +1000)
* 873f0c93 - fix(ffi): resources were not freed correctly when the mock server is provided by a plugin (Ronald Holshausen, Mon May 30 11:05:20 2022 +1000)
* e32caf8d - bump version to 0.3.2 (Ronald Holshausen, Tue May 24 15:55:23 2022 +1000)

# 0.3.1 - Bugfix Release

* 797d1cce - fix(ffi): plugin data was not merged into the Pact file correctly (Ronald Holshausen, Tue May 24 14:13:25 2022 +1000)
* a78f2a1d - fix(ffi): pactffi_create_mock_server_for_transport was returning the wrong status for invalid address (Ronald Holshausen, Mon May 23 16:56:32 2022 +1000)
* bf70164f - bump version to 0.3.1 (Ronald Holshausen, Mon May 23 15:33:59 2022 +1000)

# 0.3.0 - Support mock servers from plugins

* 5cd2ae5a - feat: add pactffi_create_mock_server_for_transport function (Ronald Holshausen, Fri May 20 16:09:36 2022 +1000)
* d9b9fe72 - chore: Upgrade pact-plugin-driver to 0.1.7 (Ronald Holshausen, Fri May 20 15:56:23 2022 +1000)
* 2b24b52b - chore(ffi): update mock server function docs (Ronald Holshausen, Tue May 17 13:45:44 2022 +1000)
* bb6f6f47 - chore(CI): fix examples with cmake on Windows (Ronald Holshausen, Tue May 17 13:22:40 2022 +1000)
* ec49b971 - chore(CI): fix examples with cmake on Windows (Ronald Holshausen, Mon May 16 18:03:43 2022 +1000)
* 0b2ac979 - chore(CI): fix examples with cmake on Windows (Ronald Holshausen, Mon May 16 17:40:38 2022 +1000)
* 2ae32295 - chore(ffi): fix examples with cmake on Windows (Ronald Holshausen, Mon May 16 17:07:25 2022 +1000)
* 6d76df16 - chore(CI): copy OSX dylib to example build dir (Ronald Holshausen, Mon May 16 16:37:33 2022 +1000)
* 888e6586 - chore: cleanup compiler warnings (Ronald Holshausen, Mon May 16 16:28:46 2022 +1000)
* 1307dde0 - fix(ffi): OSX CMake file had the wring filename (Ronald Holshausen, Mon May 16 16:04:51 2022 +1000)
* e1ddffc3 - feat(ffi): open log files in append mode (Ronald Holshausen, Mon May 16 15:31:16 2022 +1000)
* d76e417c - chore: re-enable the FFI examples in CI (Ronald Holshausen, Mon May 16 14:29:22 2022 +1000)
* b14fb2b1 - refactor: convert the FFI logging functions to setup a tracing subscriber (Ronald Holshausen, Mon May 16 14:18:22 2022 +1000)
* f8471bb7 - chore: switch from log crate to tracing crate (Ronald Holshausen, Fri May 13 13:47:18 2022 +1000)
* 1c97e1e6 - chore: Upgrade depedent crates (Ronald Holshausen, Thu May 12 11:04:34 2022 +1000)
* 1a973502 - chore: bump minor version of FFI crate (Ronald Holshausen, Thu May 12 11:01:48 2022 +1000)
* ee9d6bab - chore: Upgrade pact_verifier to 0.13.6 (Ronald Holshausen, Wed May 11 17:40:15 2022 +1000)
* f6b942da - chore: Upgrade pact_mock_server to 0.8.11 (Ronald Holshausen, Wed May 11 17:00:46 2022 +1000)
* 08f28e4a - chore: Upgrade pact_matching to 0.12.7 (Ronald Holshausen, Wed May 11 15:57:36 2022 +1000)
* 37bfc5de - chore: Upgrade pact-plugin-driver to 0.1.6 (Ronald Holshausen, Wed May 11 11:56:23 2022 +1000)
* 020b5715 - chore: upgrade pact_models to 0.4.1 (Ronald Holshausen, Wed May 11 11:36:57 2022 +1000)
* e8d62b79 - bump version to 0.2.7 (Ronald Holshausen, Wed Apr 27 16:46:23 2022 +1000)

# 0.2.6 - Maintenance Release

* 14a010a9 - chore: Upgrade pact_verifier to 0.13.5 (Ronald Holshausen, Wed Apr 27 15:21:15 2022 +1000)
* 563ae9fc - chore: Upgrade pact_mock_server to 0.8.10 (Ronald Holshausen, Wed Apr 27 15:06:50 2022 +1000)
* bcae77b4 - chore: upgrade pact_matching to 0.12.6 (Ronald Holshausen, Wed Apr 27 14:29:26 2022 +1000)
* dba7252e - chore: Upgrade pact-plugin-driver to 0.1.5 (Ronald Holshausen, Tue Apr 26 13:56:22 2022 +1000)
* 688e49e7 - chore: Upgrade pact-plugin-driver to 0.1.4 (Ronald Holshausen, Fri Apr 22 14:47:01 2022 +1000)
* cdf72b05 - feat: forward provider details to plugin when verifying (Ronald Holshausen, Fri Apr 22 14:12:34 2022 +1000)
* 2395143a - feat: forward verification to plugin for transports provided by the plugin (Ronald Holshausen, Fri Apr 22 12:02:05 2022 +1000)
* 2eeaccf4 - bump version to 0.2.6 (Ronald Holshausen, Tue Apr 19 13:59:46 2022 +1000)

# 0.2.5 - Maintenance Release

* d41e2440 - fix(ffi): correct race condition in pactffi_using_plugin (Ronald Holshausen, Wed Apr 13 16:51:36 2022 +1000)
* 136c8a82 - chore: Upgrade pact_verifier to 0.13.4 (Ronald Holshausen, Wed Apr 13 16:06:02 2022 +1000)
* 1e8ae855 - chore: Upgrade pact_mock_server to 0.8.9 (Ronald Holshausen, Wed Apr 13 15:49:03 2022 +1000)
* 0df06dd2 - chore: Upgrade pact_matching to 0.12.5 (Ronald Holshausen, Wed Apr 13 15:38:49 2022 +1000)
* d043f6c7 - chore: upgrade pact_models to 0.3.3 (Ronald Holshausen, Wed Apr 13 15:24:33 2022 +1000)
* eee09ba6 - chore: Upgrade pact-plugin-driver to 0.1.3 (Ronald Holshausen, Wed Apr 13 14:07:36 2022 +1000)
* 73ae0ef0 - fix: Upgrade reqwest to 0.11.10 to resolve #156 (Ronald Holshausen, Wed Apr 13 13:31:55 2022 +1000)
* ffeca2e2 - chore: update to the latest plugin driver (Ronald Holshausen, Wed Apr 13 13:08:25 2022 +1000)
* efaba75b - chore: Update release for FFI to use the correct nightly Rust for cbindgen (Ronald Holshausen, Wed Apr 13 10:40:34 2022 +1000)
* 610490ab - chore: trying to get the ffi cmake build working (Ronald Holshausen, Tue Apr 12 18:09:47 2022 +1000)
* e13eb80d - chore: Update ci-build.sh to use the same nightly Rust as build (Ronald Holshausen, Tue Apr 12 17:49:21 2022 +1000)
* 776265ee - chore: bump pact_verifier to 0.13.3 (Ronald Holshausen, Thu Mar 24 15:05:01 2022 +1100)
* 89027c87 - chore: update pact_matching (0.12.4) and pact_mock_server (0.8.8) (Ronald Holshausen, Thu Mar 24 14:09:45 2022 +1100)
* 9baf03a9 - chore: use the published version of the plugin driver (Ronald Holshausen, Thu Mar 24 13:36:01 2022 +1100)
* 42b1a461 - Merge branch 'master' into feat/plugin-mock-server (Ronald Holshausen, Mon Mar 21 16:01:33 2022 +1100)
* 345b0011 - feat: support mock servers provided from plugins (Ronald Holshausen, Mon Mar 21 15:59:46 2022 +1100)
* 63b63358 - bump version to 0.2.5 (Matt Fellows, Mon Mar 21 11:29:45 2022 +1100)

# 0.2.4 - Bugfix Release

* 13f7c36f - fix: xml response matching rules (Matt Fellows, Wed Mar 9 17:07:56 2022 +1100)
* c5b96ebb - chore: need musl-tools om release build (Ronald Holshausen, Fri Mar 4 17:15:43 2022 +1100)
* 01b7adb9 - bump version to 0.2.4 (Ronald Holshausen, Fri Mar 4 16:46:18 2022 +1100)
* b67292db - update changelog for release 0.2.3 (Ronald Holshausen, Fri Mar 4 16:42:52 2022 +1100)
* 16fbe7cf - feat: add musl target to the release build #185 (Ronald Holshausen, Fri Mar 4 16:23:39 2022 +1100)
* b6433500 - chore: upgrade pact_verifier to 0.13.2 (Ronald Holshausen, Fri Mar 4 14:49:18 2022 +1100)
* 5a4a8a1c - chore: update pact_mock_server to 0.8.7 (Ronald Holshausen, Fri Mar 4 14:24:23 2022 +1100)
* 8894fdfd - chore: update pact_matching to 0.12.3 (Ronald Holshausen, Fri Mar 4 14:09:17 2022 +1100)
* 8e864502 - chore: update all dependencies (Ronald Holshausen, Fri Mar 4 13:29:59 2022 +1100)
* f52c3625 - feat: add for custom headers to the HTTP client used by the verifier #182 (Ronald Holshausen, Mon Feb 28 14:38:00 2022 +1100)
* 74bd4531 - feat: add support for custom headers with the verifier FFI calls #182 (Ronald Holshausen, Mon Feb 28 13:58:46 2022 +1100)
* c6d553e0 - bump version to 0.2.3 (Ronald Holshausen, Mon Feb 14 13:45:19 2022 +1100)

# 0.2.3 - Support Custom headers + Date-Time expression parser

* 16fbe7cf - feat: add musl target to the release build #185 (Ronald Holshausen, Fri Mar 4 16:23:39 2022 +1100)
* b6433500 - chore: upgrade pact_verifier to 0.13.2 (Ronald Holshausen, Fri Mar 4 14:49:18 2022 +1100)
* 5a4a8a1c - chore: update pact_mock_server to 0.8.7 (Ronald Holshausen, Fri Mar 4 14:24:23 2022 +1100)
* 8894fdfd - chore: update pact_matching to 0.12.3 (Ronald Holshausen, Fri Mar 4 14:09:17 2022 +1100)
* 8e864502 - chore: update all dependencies (Ronald Holshausen, Fri Mar 4 13:29:59 2022 +1100)
* f52c3625 - feat: add for custom headers to the HTTP client used by the verifier #182 (Ronald Holshausen, Mon Feb 28 14:38:00 2022 +1100)
* 74bd4531 - feat: add support for custom headers with the verifier FFI calls #182 (Ronald Holshausen, Mon Feb 28 13:58:46 2022 +1100)
* c6d553e0 - bump version to 0.2.3 (Ronald Holshausen, Mon Feb 14 13:45:19 2022 +1100)

# 0.2.2 - Bugfix Release

* 76889087 - fix(pact-ffi): intermediate JSON - add test for JSON with decimal matcher #179 (Ronald Holshausen, Mon Feb 14 13:04:16 2022 +1100)
* b10453c3 - fix(pact-ffi): intermediate JSON - type matcher paths were being incorrectly allocated to children #179 (Ronald Holshausen, Mon Feb 14 12:45:43 2022 +1100)
* 1555c682 - bump version to 0.2.2 (Ronald Holshausen, Thu Feb 3 14:12:46 2022 +1100)

# 0.2.1 - add option to strip ANSI control codes from verifier output

* 506add91 - chore: bump pact_verifier version (Ronald Holshausen, Thu Feb 3 13:54:45 2022 +1100)
* cc872209 - chore: add non-windows init ansi support function (Ronald Holshausen, Thu Feb 3 13:22:51 2022 +1100)
* 7311e022 - feat(FFI): add option to strip ANSI control codes from verifier output (Ronald Holshausen, Thu Feb 3 12:29:02 2022 +1100)
* c18e1ccc - chore: ANSI support function was missing pactffi prefix (Ronald Holshausen, Thu Feb 3 11:12:45 2022 +1100)
* fbfd072f - feat(FFI): add an explicit function to enable ANSI terminal support on Windows (Ronald Holshausen, Thu Feb 3 11:11:30 2022 +1100)
* 07806b05 - bump version to 0.2.1 (Ronald Holshausen, Mon Jan 31 14:28:33 2022 +1100)

# 0.2.0 - Bugfixes + FFI functions to return the verifier output and results

* 1d95f3cf - chore: Bump minor version of Pact FFI lib (Ronald Holshausen, Mon Jan 31 13:58:42 2022 +1100)
* 739cb7b8 - chore: fix missing import on Windows (Ronald Holshausen, Mon Jan 31 11:16:55 2022 +1100)
* 5ecf70a7 - feat: enable ANSI console output on Windows (Ronald Holshausen, Mon Jan 31 11:02:03 2022 +1100)
* c676e821 - feat: add FFI functions to return the verifier output and results (Ronald Holshausen, Fri Jan 28 15:40:17 2022 +1100)
* bf152233 - feat: Capture all the results from the verification process (Ronald Holshausen, Fri Jan 28 11:28:38 2022 +1100)
* 5f148cdd - feat: capture all the output from the verifier (Ronald Holshausen, Thu Jan 27 16:08:02 2022 +1100)
* f5aa34ea - Merge pull request #175 from pact-foundation/feat/fix-provider-timeout-value-validation (Ronald Holshausen, Thu Jan 27 13:41:56 2022 +1100)
* c58a2fb7 - Merge pull request #174 from adamrodger/feat/provider-name (Ronald Holshausen, Thu Jan 27 13:39:26 2022 +1100)
* 0ef3fb98 - fix: provider request timeout should be > 16bit integers. Fixes https://github.com/pact-foundation/pact-js/issues/761 (Matt Fellows, Wed Jan 26 22:12:35 2022 +1100)
* 753c9599 - feat(ffi)!: Remove the need to repeat the provider name in verifier FFI (Adam Rodger, Wed Jan 26 10:17:23 2022 +0000)
* 8bee40b0 - feat(ffi)!: Separate verification and publishing options (Adam Rodger, Tue Jan 25 16:31:29 2022 +0000)
* bef310b2 - bump version to 0.1.7 (Ronald Holshausen, Mon Jan 17 17:20:07 2022 +1100)

# 0.1.6 - Maintenance Release

* 0c200ea5 - chore: Upgrade pact verifier crate to 0.12.4 (Ronald Holshausen, Mon Jan 17 17:07:18 2022 +1100)
* 10c9b842 - chore: Upgrade pact_mock_server to 0.8.6 (Ronald Holshausen, Mon Jan 17 16:57:31 2022 +1100)
* 5e4c68ef - chore: update pact matching to 0.12.2 (Ronald Holshausen, Mon Jan 17 16:29:21 2022 +1100)
* 80b241c5 - chore: Upgrade plugin driver crate to 0.0.17 (Ronald Holshausen, Mon Jan 17 11:22:48 2022 +1100)
* 4f1ecff2 - chore: Upgrade pact-models to 0.2.7 (Ronald Holshausen, Mon Jan 17 10:53:26 2022 +1100)
* 63ab0d2d - fix: generators in process_object (Matt Fellows, Sat Jan 15 23:21:34 2022 +1100)
* c2089645 - fix: log crate version must be fixed across all crates (including plugin driver) (Ronald Holshausen, Fri Jan 14 16:10:50 2022 +1100)
* 255d6eba - bump version to 0.1.6 (Ronald Holshausen, Tue Jan 4 10:59:38 2022 +1100)

# 0.1.5 - Maintenance Release

* 7dbdd456 - chore: update test log crate (Ronald Holshausen, Tue Jan 4 10:46:46 2022 +1100)
* 1b16e30a - chore: test-env-log has been renamed to test-log (Ronald Holshausen, Tue Jan 4 10:43:51 2022 +1100)
* 1cafd00a - fix: drop(from_raw(ptr))` if you intend to drop the `CString` (Ronald Holshausen, Tue Jan 4 10:39:16 2022 +1100)
* fe22ae3a - fix: expected opaque type, found enum `Result` (Ronald Holshausen, Tue Jan 4 10:26:22 2022 +1100)
* 213d1459 - fix: add a small delay after loading plugins via FFI to resolve a race condition (Ronald Holshausen, Tue Jan 4 09:56:33 2022 +1100)
* 9c2810ad - chore: Upgrade pact-plugin-driver to 0.0.15 (Ronald Holshausen, Fri Dec 31 15:12:56 2021 +1100)
* 0a6e7d9d - refactor: Convert MatchingContext to a trait and use DocPath instead of string slices (Ronald Holshausen, Wed Dec 29 14:24:39 2021 +1100)
* 4d088317 - chore: Update pact_mock_server crate to 0.8.4 (Ronald Holshausen, Thu Dec 23 13:24:15 2021 +1100)
* 52bc1735 - chore: update pact_matching crate to 0.11.5 (Ronald Holshausen, Thu Dec 23 13:12:08 2021 +1100)
* 5479a634 - chore: Update pact_models (0.2.4) and pact-plugin-driver (0.0.14) (Ronald Holshausen, Thu Dec 23 12:57:02 2021 +1100)
* fc0a8360 - chore: update pact_matching to 0.11.4 (Ronald Holshausen, Mon Dec 20 12:19:36 2021 +1100)
* 8911d5b0 - chore: update to latest plugin driver crate (metrics fixes) (Ronald Holshausen, Mon Dec 20 12:11:35 2021 +1100)
* 9153cc5b - bump version to 0.1.5 (Ronald Holshausen, Wed Dec 15 16:44:22 2021 +1100)

# 0.1.4 - Maintenance Release

* a1d03b95 - chore: update dependent pact crates (Ronald Holshausen, Wed Dec 15 16:34:39 2021 +1100)
* f8042d6b - feat: add metrics event for provider verification (Ronald Holshausen, Tue Dec 14 17:29:44 2021 +1100)
* 4f1ba7d9 - chore: update to the latest plugin driver (Ronald Holshausen, Tue Dec 14 13:55:02 2021 +1100)
* 2f97c25f - bump version to 0.1.4 (Ronald Holshausen, Thu Dec 2 13:21:24 2021 +1100)

# 0.1.3 - Bugfix Release

* 4184e562 - chore(pact_ffi): upgrade to latest models, matching and verifier crates (Ronald Holshausen, Thu Dec 2 13:13:37 2021 +1100)
* d43b1847 - Merge pull request #164 from tienvx/feat-filter-info (Ronald Holshausen, Fri Nov 19 11:38:41 2021 +1100)
* 41e69a22 - feat: allow set filter info (tienvx, Thu Nov 18 08:56:36 2021 +0700)
* 7c561f2a - feat: allow set consumer version selectors (tienvx, Thu Nov 18 00:12:31 2021 +0700)
* 260deb70 - fix: support specifying matching_branch in verifications (Matt Fellows, Wed Nov 17 17:47:37 2021 +1100)
* 86ea5779 - chore: fix FFI release build (Ronald Holshausen, Wed Nov 17 15:52:14 2021 +1100)
* 5480733f - bump version to 0.1.3 (Ronald Holshausen, Wed Nov 17 15:20:29 2021 +1100)

# 0.1.2 - Bugfix Release

* 631167fa - chore: update to latest mock server crate (Ronald Holshausen, Wed Nov 17 15:13:32 2021 +1100)
* 87e7f11e - chore: remove note from pactffi_write_pact_file (Ronald Holshausen, Wed Nov 17 14:55:22 2021 +1100)
* 5d4a09c6 - feat: store the pact specification version with the mock server (Ronald Holshausen, Wed Nov 17 14:46:56 2021 +1100)
* 4ccc5d02 - chore: update doc comment on pactffi_write_pact_file (Ronald Holshausen, Wed Nov 17 14:04:59 2021 +1100)
* 675506e1 - feat: add pactffi_pact_handle_write_file which knows about the spec version (Ronald Holshausen, Wed Nov 17 13:58:45 2021 +1100)
* 09f3b888 - refactor: make the pact handle types opaque (Ronald Holshausen, Wed Nov 17 13:27:06 2021 +1100)
* aff4d301 - fix: FFI always detects + stores JSON bodies as plain text (Matt Fellows, Tue Nov 16 23:02:12 2021 +1100)
* fc5be202 - fix: update to latest driver crate (Ronald Holshausen, Tue Nov 16 16:19:02 2021 +1100)
* e4a445ba - fix: race condition when shutting down plugin via FFI (Ronald Holshausen, Tue Nov 16 16:01:18 2021 +1100)
* f3c5e7c1 - bump version to 0.1.2 (Ronald Holshausen, Tue Nov 16 14:04:54 2021 +1100)

# 0.1.1 - Support V4 synchronous messages + protobuf plugin

* 5d974c4a - chore: update to latest models and plugin driver crates (Ronald Holshausen, Tue Nov 16 11:56:53 2021 +1100)
* 19beb0ea - feat(plugins): add support for synch messages via FFI (Ronald Holshausen, Tue Nov 16 10:06:07 2021 +1100)
* df23ba3d - fix: allow multiple consumer version selectors (Matt Fellows, Mon Nov 15 14:28:04 2021 +1100)
* 7c150c8b - feat(plugins): Support message tests via FFI that use plugins (Ronald Holshausen, Wed Nov 10 17:03:49 2021 +1100)
* 20643590 - feat(plugins): add plugin support to FFI functions (Ronald Holshausen, Tue Nov 9 16:06:01 2021 +1100)
* 62f7d36c - refactor: moved the message consumer FFI functions to the handles module (Ronald Holshausen, Mon Nov 8 17:42:26 2021 +1100)
* 0cb367d9 - refactor: moved the HTTP consumer FFI functions to the handles module (Ronald Holshausen, Mon Nov 8 17:25:16 2021 +1100)
* 2027537d - refactor: update FFI to use V4 models internally (Ronald Holshausen, Mon Nov 8 16:44:39 2021 +1100)
* e1ff90c7 - bump version to 0.1.1 (Ronald Holshausen, Thu Nov 4 17:23:32 2021 +1100)

# 0.1.0 - Pact V4 release

* 59e21413 - feat: Pact V4 release (Ronald Holshausen, Thu Nov 4 16:54:56 2021 +1100)
* 400a1231 - chore: drop beta from pact_verifier version (Ronald Holshausen, Thu Nov 4 15:56:22 2021 +1100)
* fc4580b8 - chore: drop beta from pact_mock_server version (Ronald Holshausen, Thu Nov 4 15:28:51 2021 +1100)
* bd2bd0ec - chore: drop beta from pact_matching version (Ronald Holshausen, Wed Nov 3 13:28:35 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* a561f883 - chore: use the non-beta models crate (Ronald Holshausen, Thu Oct 21 18:10:27 2021 +1100)
* 0c72c80e - chore: fixes after merging from master (Ronald Holshausen, Wed Oct 20 14:46:54 2021 +1100)
* ec265d83 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Oct 20 14:40:37 2021 +1100)
* 2ac17234 - chore: deprecate the old verifier functions (Ronald Holshausen, Tue Oct 19 17:42:56 2021 +1100)
* e98a91fe - chore: update to latest verifier lib (Ronald Holshausen, Tue Oct 19 17:42:07 2021 +1100)
* a3d321cb - chore: update to latest mock server crate (Ronald Holshausen, Tue Oct 19 17:28:24 2021 +1100)
* 46a404c0 - chore: update to latest pact matching crate (Ronald Holshausen, Tue Oct 19 17:20:27 2021 +1100)
* 918e5beb - fix: update to latest models and plugin driver crates (Ronald Holshausen, Tue Oct 19 17:09:48 2021 +1100)
* 7e209367 - chore: update to latest verification crate (Ronald Holshausen, Tue Oct 19 11:50:57 2021 +1100)
* 3819522d - chore: update to the latest matching and mock server crates (Ronald Holshausen, Tue Oct 19 11:34:18 2021 +1100)
* bfa04370 - fix: display the error message when the verification can not be run due to an error (Ronald Holshausen, Tue Oct 19 11:09:21 2021 +1100)
* df386c8a - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Oct 18 13:41:36 2021 +1100)
* 1dc6f543 - chore: bump pact_mock_server version (Ronald Holshausen, Tue Oct 12 16:36:51 2021 +1100)
* 9bbbb52e - chore: bump pact matching crate version (Ronald Holshausen, Tue Oct 12 16:24:01 2021 +1100)
* 35ff0993 - feat: record the version of the lib that created the pact in the metadata (Ronald Holshausen, Tue Oct 12 14:52:43 2021 +1100)
* 1eb37c13 - chore: use the published version of the models crate (Ronald Holshausen, Thu Oct 7 10:49:11 2021 +1100)
* 2e86c48d - Merge pull request #154 from pact-foundation/feat/xml-matchers (Matt Fellows, Tue Oct 5 16:39:18 2021 +1100)
* 9a2049c2 - feat: support XML bodies in FFI interface (Matt Fellows, Thu Sep 30 22:08:01 2021 +1000)
* d171edfd - feat: support provider branches (Matt Fellows, Wed Sep 29 22:47:21 2021 +1000)
* 6d23796f - feat(plugins): support each key and each value matchers (Ronald Holshausen, Wed Sep 29 11:10:46 2021 +1000)
* 6f20282d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 28 14:51:34 2021 +1000)
* a8f900ab - bump version to 0.0.4 (Ronald Holshausen, Tue Sep 28 14:09:57 2021 +1000)
* 7a3c7693 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Sep 20 13:44:53 2021 +1000)
* b71dcabf - refactor(plugins): rename ContentTypeOverride -> ContentTypeHint (Ronald Holshausen, Tue Sep 14 15:08:52 2021 +1000)
* f55440c6 - chore: Bump verifier lib version to 0.11.0-beta.0 (Ronald Holshausen, Mon Sep 13 12:04:19 2021 +1000)
* 03ebe632 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Sep 13 12:01:13 2021 +1000)
* fd6f8f40 - chore: Bump pact_mock_server version to 0.8.0-beta.0 (Ronald Holshausen, Mon Sep 13 11:46:11 2021 +1000)
* 716809f6 - chore: Get CI build passing (Ronald Holshausen, Fri Sep 10 14:55:46 2021 +1000)
* ceb1c35f - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 7 10:07:45 2021 +1000)
* e8ae81b3 - refactor: matching req/res with plugins requires data from the pact and interaction (Ronald Holshausen, Thu Sep 2 11:57:50 2021 +1000)
* b9aa7ecb - feat(Plugins): allow plugins to override text/binary format of the interaction content (Ronald Holshausen, Mon Aug 30 10:48:04 2021 +1000)
* 0c5cede2 - chore: bump models crate to 0.2 (Ronald Holshausen, Mon Aug 23 12:56:14 2021 +1000)
* 248629e1 - chore: fix build after merge from master (Ronald Holshausen, Mon Aug 23 10:42:44 2021 +1000)
* 75e13fd8 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Aug 23 10:33:45 2021 +1000)
* b75fea5d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Aug 18 12:27:41 2021 +1000)
* 2662241e - feat(plugins): Call out to plugins when comparing content owned by the plugin during verification (Ronald Holshausen, Fri Aug 13 14:29:30 2021 +1000)
* bdfc6f02 - feat(plugins): Load required plugins when verifying a V4 pact (Ronald Holshausen, Wed Aug 11 14:23:54 2021 +1000)
* dfe3cd42 - chore: bump minor version of Pact verifier libs (Ronald Holshausen, Mon Aug 9 15:10:47 2021 +1000)

# 0.0.3 - support native TLS certs + updated verifier FFI functions

* 42be9eb8 - feat: add FFI functions to extract logs from a verifcation run (Ronald Holshausen, Tue Sep 28 12:48:40 2021 +1000)
* 40cf1ab9 - chore: mark pactffi_logger_attach_sink as unsafe #148 (Ronald Holshausen, Fri Sep 24 11:36:38 2021 +1000)
* ab89152e - Merge pull request #150 from tienvx/make-state-change-url-optional (Ronald Holshausen, Tue Sep 21 09:20:54 2021 +1000)
* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 339a9504 - feat: make state change url optional (tienvx, Mon Sep 20 12:13:29 2021 +0700)
* dab70272 - feat: add verifier ffi function set consumer filters (tienvx, Tue Sep 14 23:47:14 2021 +0700)
* 36f7e477 - fix: fix missing last tag (tienvx, Tue Sep 14 23:51:02 2021 +0700)
* 4e02722e - Handle required and optional parameters (tienvx, Fri Sep 10 21:56:45 2021 +0700)
* ad73c9af - Extract function get_tags to reuse code (tienvx, Fri Sep 10 21:55:36 2021 +0700)
* 05f4c3de - feat: add verifier ffi function set verification options (tienvx, Wed Sep 8 23:48:13 2021 +0700)
* 971b980e - chore: fix clippy warnings (Ronald Holshausen, Fri Sep 10 17:31:16 2021 +1000)
* 0bb96329 - chore: fix clippy warnings (Ronald Holshausen, Fri Sep 10 17:15:17 2021 +1000)
* b8e51313 - Merge pull request #137 from tienvx/ffi-function-update-provider-state (Ronald Holshausen, Sat Sep 4 13:04:50 2021 +1000)
* 47e940a8 - test(ffi verifier): remove unused import (Mike Geeves, Tue Aug 31 10:11:37 2021 +0100)
* d5167056 - feat(ffi verifier cli): simplify duplicated conversion for default_value, env, possible_values (Mike Geeves, Mon Aug 30 21:29:34 2021 +0100)
* fd9ea9c3 - feat(ffi verifier cli): attributes long/short/help can be simplified (Mike Geeves, Mon Aug 30 21:06:23 2021 +0100)
* 9e582360 - chore: add verifier ffi function update provider state (tienvx, Sun Aug 29 22:20:28 2021 +0700)
* 55985d0a - feat(ffi verifier cli): add in support for ENVs (Mike Geeves, Fri Aug 27 15:59:56 2021 +0100)
* 4a5cdb82 - Merge branch 'master' into feat/ffi_arguments (Mike Geeves, Fri Aug 27 09:57:52 2021 +0100)
* 84957fb9 - feat(ffi verifier cli): verify we can deserialize the json from cli_args, and there are some args (Mike Geeves, Fri Aug 27 09:55:24 2021 +0100)
* 906661cb - feat(ffi verifier cli): split out flags and options (Mike Geeves, Thu Aug 26 11:45:18 2021 +0100)
* 491c23fb - feat(ffi verifier): add multiple to CLI JSON output (Mike Geeves, Wed Aug 25 15:58:00 2021 +0100)
* 46135a16 - chore: add verifier FFI functions for directory, URL and Pact broker sources (Ronald Holshausen, Tue Aug 24 10:14:46 2021 +1000)
* bbae32da - feat(ffi verify): add in default values, start looking at flags (Mike Geeves, Tue Aug 24 00:25:56 2021 +0100)
* ffcabb63 - feat(ffi verifier): add possible_values (Mike Geeves, Mon Aug 23 10:21:16 2021 +0100)
* 5a32f04d - feat(ffi verifier): bump serde version to latest (Mike Geeves, Mon Aug 23 09:55:31 2021 +0100)
* f64b0ead - feat(ffi verifier): revert unwanted changes (Mike Geeves, Mon Aug 23 09:53:31 2021 +0100)
* e8247e55 - feat(ffi verifier): merge master, fix conflicts (Mike Geeves, Mon Aug 23 09:51:24 2021 +0100)
* e557ce27 - feat(ffi verifier): move pactffi_verifier_cli_args to mod.rs, tidy, add docs (Mike Geeves, Mon Aug 23 09:45:54 2021 +0100)
* 4982bfc7 - chore: update FFI readme (Ronald Holshausen, Mon Aug 23 10:31:08 2021 +1000)
* f8d98dcb - feat(ffi verifier): added a crude method to pull out CLI arguments, and make available via FFI (Mike Geeves, Sun Aug 22 19:45:42 2021 +0100)
* 50fcd409 - chore: re-enable cmake build for pact-ffi (Ronald Holshausen, Sun Aug 22 16:20:25 2021 +1000)
* eaefe4d2 - chore: correct the conan recipes (Ronald Holshausen, Sun Aug 22 16:18:50 2021 +1000)
* 72125560 - bump version to 0.0.3 (Ronald Holshausen, Sun Aug 22 15:51:25 2021 +1000)

# 0.0.2 - Bugfix Release

* 9370327c - feat(FFI): Added initial verifier FFI prototype (Ronald Holshausen, Sun Aug 22 15:01:17 2021 +1000)
* c274ca1a - fix: use the pacts for verification endpoint if the conusmer selectors are specified #133 (Ronald Holshausen, Sun Aug 22 11:51:22 2021 +1000)
* 3215821e - chore: correct the OSX release (Ronald Holshausen, Tue Aug 17 12:47:20 2021 +1000)
* 64cf38e9 - bump version to 0.0.2 (Ronald Holshausen, Tue Aug 17 11:00:54 2021 +1000)

# 0.0.1 - M1 architecture support + Bugfixes

* a9940325 - chore: release m1 arm package for new Mac hardware (Matt Fellows, Wed Aug 11 22:57:46 2021 +1000)
* b5a7b779 - feat: support new selectors (Matt Fellows, Mon Aug 9 13:27:33 2021 +1000)
* 8bcd1c7e - fix: min/max type matchers must not apply the limits when cascading (Ronald Holshausen, Sun Aug 8 15:50:40 2021 +1000)
* 738e9961 - chore: generate both a C and C++ header (Ronald Holshausen, Sun Aug 8 14:28:11 2021 +1000)
* 6124ed0b - refactor: Introduce DocPath struct for path expressions (Caleb Stepanian, Thu Jul 29 12:27:32 2021 -0400)
* 9baa714d - chore: bump minor version of matching crate (Ronald Holshausen, Fri Jul 23 14:03:20 2021 +1000)
* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* 3dccf866 - refacfor: moved the pact structs to the models crate (Ronald Holshausen, Sun Jul 18 16:58:14 2021 +1000)
* 372cbdd1 - chore: remove CMake step from CI build (Ronald Holshausen, Sun Jul 18 15:57:30 2021 +1000)
* 996761a6 - chore: remove CMake step from CI build (Ronald Holshausen, Sun Jul 18 15:26:31 2021 +1000)
* 685a2df2 - chore: cmake is failing on CI to find cargo (Ronald Holshausen, Sun Jul 18 15:12:51 2021 +1000)
* e8046d84 - refactor: moved interaction structs to the models crate (Ronald Holshausen, Sun Jul 18 14:36:03 2021 +1000)
* 0c528ea0 - chore: update pact_ffi readme (Ronald Holshausen, Mon Jul 12 15:28:53 2021 +1000)
* 95c873ec - chore: correct conan packages (Ronald Holshausen, Mon Jul 12 11:18:44 2021 +1000)
* bd989b8f - chore: add homepage and repositry URL to FFI manifest (Ronald Holshausen, Mon Jul 12 10:25:36 2021 +1000)
* 5e725866 - chore: add conan publish to release script (Ronald Holshausen, Mon Jul 12 10:23:33 2021 +1000)
* 6d1ff318 - fix: conan packages for pact_ffi (Ronald Holshausen, Mon Jul 12 09:23:48 2021 +1000)
* d7079e43 - bump version to 0.0.1 (Ronald Holshausen, Sun Jul 11 17:47:18 2021 +1000)

# 0.0.0 - Initial Release
