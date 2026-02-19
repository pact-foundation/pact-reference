To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 1.3.4 - Maintenance Release

* ea571fbe - chore(pact_verifier): Update dependencies (Ronald Holshausen, Thu Feb 19 10:44:44 2026 +1100)
* 0c9a0b27 - chore: Update pact_matching to 2.0.3 (Ronald Holshausen, Thu Feb 19 10:16:12 2026 +1100)
* 650a2582 - chore: Update pact_models to 1.3.8 (Ronald Holshausen, Wed Feb 18 12:19:37 2026 +1100)
* 485b3ac7 - chore: Fix for flaky test failing in CI (Ronald Holshausen, Wed Feb 18 09:26:45 2026 +1100)
* 2afc3c85 - feat: Add option to disable verifier following redirects #513 (Ronald Holshausen, Tue Feb 17 12:11:50 2026 +1100)
* b59751b2 - chore: Upgrade pact_matching to 2.0.2 (Ronald Holshausen, Wed Sep 17 10:34:34 2025 +1000)
* eaaee9de - chore: Upgrade pact_models to 1.3.7 (Ronald Holshausen, Tue Sep 16 13:57:37 2025 +1000)
* 4d7f186c - bump version to 1.3.4 (Ronald Holshausen, Wed Aug 20 09:38:11 2025 +1000)

# 1.3.3 - Maintenance Release

* a77b53c8 - chore: Update pact_matching to 2.0.1 (Ronald Holshausen, Fri Aug 1 09:16:28 2025 +1000)
* c480357e - Chore: Update pact_matching to 2.0.0 (Ronald Holshausen, Thu Jul 31 15:40:59 2025 +1000)
* e2e69295 - refactor(v2-matching-engine): Merged matchers.rs and matchingrules.rs; deprecated Matches replaced with DoMatch trait (Ronald Holshausen, Thu Jul 31 13:18:27 2025 +1000)
* ff964960 - chore: Update pact_models to 1.3.6 (Ronald Holshausen, Wed Jul 30 09:50:09 2025 +1000)
* 1cb9a5ac - chore: Update pact_models to 1.3.5 (Ronald Holshausen, Mon Jul 28 10:44:30 2025 +1000)
* cfc5e44d - chore(v2-matching-engine): Fix after refactor of match_response function (Ronald Holshausen, Fri Jul 25 10:29:09 2025 +1000)
* 24bc59a7 - bump version to 1.3.3 (Ronald Holshausen, Fri Jun 20 11:21:42 2025 +1000)

# 1.3.2 - Support exit-on-first-error and last-failed options

* 31113440 - feat(pact_verifier_cli): Implement the --last-failed CLI flag #494 (Ronald Holshausen, Fri Jun 20 11:03:44 2025 +1000)
* a8c50d56 - feat(pact_verifier): Adds details of the verified interactions to the JSON output #494 (Ronald Holshausen, Thu Jun 19 14:51:50 2025 +1000)
* a54a7d55 - feat(pact_verifier): Correct warning when exit-on-first-error is set #494 (Ronald Holshausen, Thu Jun 19 10:56:38 2025 +1000)
* 74e34bec - feat(pact_verifier): Skip any unverified interactions if exit-first is set and there is a failure #494 (Ronald Holshausen, Thu Jun 19 10:47:18 2025 +1000)
* f02641ac - feat(pact_verifier): Skip any unverified Pact files if exit-first is set and there is a failure #494 (Ronald Holshausen, Wed Jun 18 16:27:27 2025 +1000)
* 1f734dd8 - feat(pact_verifier_cli): Add the exit-first and last-failed CLI flags #494 (Ronald Holshausen, Wed Jun 18 16:06:17 2025 +1000)
* 68d55bf9 - chore(pact_verifier): Update dependencies (Ronald Holshausen, Wed Jun 18 14:44:43 2025 +1000)
* 1f0f1c4f - bump version to 1.3.2 (Ronald Holshausen, Thu Jun 12 09:25:22 2025 +1000)

# 1.3.1 - Maintenance Release

* 5371ee6f - chore(pact_verifier): Update pact_consumer to 1.4.1 (Ronald Holshausen, Wed Jun 11 16:29:07 2025 +1000)
* 3f883a8d - fix(Alpine): Update pact_matching to 2.0.0-beta.3 (Ronald Holshausen, Wed Jun 11 13:46:00 2025 +1000)
* 6e8167c4 - fix(Alpine): Updae pact_models to 1.3.3 (Ronald Holshausen, Wed Jun 11 13:23:10 2025 +1000)
* 588f1fe2 - chore: Update pact_matching to 2.0.0-beta.2 (Ronald Holshausen, Wed Jun 11 11:24:09 2025 +1000)
* 18e53fbd - chore: Update pact_models to 1.3.2 (Ronald Holshausen, Wed Jun 11 10:16:03 2025 +1000)
* e2027e97 - bump version to 1.3.1 (Ronald Holshausen, Wed May 21 15:07:33 2025 +1000)

# 1.3.0 - Update to Rust 2024 and pact_matching 2.0.0-beta

* 71b1b7ed - chore(pact_consumer): Update pact_mock_server to 2.1.0 (Ronald Holshausen, Wed May 21 14:44:21 2025 +1000)
* 0649f61f - chore: Update pact_matching to 2.0.0-beta.1 (Ronald Holshausen, Wed May 21 12:04:10 2025 +1000)
* 64b0df33 - Revert "update changelog for release 1.3.0" (Ronald Holshausen, Wed May 21 10:32:58 2025 +1000)
* e4e5f0ed - update changelog for release 1.3.0 (Ronald Holshausen, Wed May 21 10:30:06 2025 +1000)
* 2b4536b1 - chore: Bump pact_verifier to 1.3.0 (Ronald Holshausen, Tue May 20 16:34:00 2025 +1000)
* b0d763f0 - refactor: Add builder for HALClient (Ronald Holshausen, Tue May 20 16:15:40 2025 +1000)
* 05c12d82 - chore(pact_verifier): Update to Rust 2024 edition (Ronald Holshausen, Tue May 20 15:02:09 2025 +1000)
* 7d8e8f61 - chore: Upgrade pact_models to 1.3.1 (Ronald Holshausen, Tue May 20 14:53:42 2025 +1000)
* 0383b9ac - chore: Remove the use of the deprecated pact_matching::logging::LOG_ID (Ronald Holshausen, Wed Mar 26 15:52:47 2025 +1100)
* 12ca5283 - chore: Upgrade pact-plugin-driver to 0.7.4 (Ronald Holshausen, Tue Mar 25 16:26:48 2025 +1100)
* 0ed75202 - chore(pact-models): Upgrade pact models crate to Rust 2024 edition (Ronald Holshausen, Tue Mar 25 16:02:47 2025 +1100)
* 79b52714 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Thu Mar 20 09:43:30 2025 +1100)
* bd94810b - bump version to 1.2.6 (Ronald Holshausen, Wed Mar 19 14:02:45 2025 +1100)
* b07d7aec - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Wed Mar 19 11:22:43 2025 +1100)
* ad9b555b - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Fri Mar 14 11:42:13 2025 +1100)
* c8f165b7 - fix: correct the build after last change (Ronald Holshausen, Thu Mar 6 09:52:03 2025 +1100)
* 7b94a211 - Merge tag 'master' into feat/matching-engine-v2 (Ronald Holshausen, Thu Feb 20 15:16:16 2025 +1100)
* 5db8db23 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Wed Nov 13 14:16:31 2024 +1100)
* 11cd9dc0 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Fri Oct 25 10:01:15 2024 +1100)
* 02e2ae13 - chore: Bump pact_matching to 2.0.0-beta.0 (Ronald Holshausen, Tue Jul 30 11:40:20 2024 +1000)

# 1.2.5 - Bugfix Release

* 5ad52f84 - chore(pact_verifier): Update dependencies (Ronald Holshausen, Wed Mar 19 13:47:29 2025 +1100)
* e32f0dbd - fix: Upgrade pact-plugin-driver to 0.7.3 (Ronald Holshausen, Wed Mar 19 11:20:39 2025 +1100)
* 164d6bb8 - chore(pact_verifier): change debug statement (Ronald Holshausen, Wed Mar 19 11:20:16 2025 +1100)
* d67db1a4 - chore: Update pact_models to 1.2.7 (Ronald Holshausen, Fri Mar 14 11:02:53 2025 +1100)
* cb060659 - chore: add small timeout to test (Ronald Holshausen, Tue Feb 4 09:53:11 2025 +1100)
* 8fb0bcb9 - chore: Upgrade pact_matching to 1.2.9 (Ronald Holshausen, Fri Jan 31 09:44:14 2025 +1100)
* 648ab1c9 - chore: Upgrade pact_matching to 1.2.8 (Ronald Holshausen, Thu Dec 19 16:14:10 2024 +1100)
* 5fdfd023 - chore: Upgrade all the tracing crates to the latest (Ronald Holshausen, Fri Dec 13 10:54:51 2024 +1100)
* fab32aa0 - chore: Upgrade pact_models to 1.2.6 (Ronald Holshausen, Wed Dec 11 15:28:01 2024 +1100)
* 44d9dc1e - fix(plugins): Upgrade pact-plugin-driver to 0.7.2, fixes #473 (Ronald Holshausen, Fri Nov 29 15:48:40 2024 +1100)
* 7735a276 - Merge branch 'fix/verifier_publish_results_ruby_thin_server' of github.com:YOU54F/pact-reference into YOU54F-fix/verifier_publish_results_ruby_thin_server (Ronald Holshausen, Thu Nov 28 09:36:09 2024 +1100)
* 420c654d - chore: Upgrade pact_models to 1.2.5 (Ronald Holshausen, Wed Nov 13 11:01:39 2024 +1100)
* abf14232 - fix(verifier): add Accept-Charset=utf8 header when post/put (Yousaf Nabi, Tue Nov 12 23:57:56 2024 +0000)
* c1a00dd8 - chore: Upgrade pact-plugin-driver to 0.7.1 (Ronald Holshausen, Tue Sep 3 15:47:22 2024 +1000)
* dc441709 - chore: Upgrade pact_matching to 1.2.7 (Ronald Holshausen, Tue Aug 27 14:57:12 2024 +1000)
* b33fa126 - bump version to 1.2.5 (Ronald Holshausen, Tue Aug 13 15:39:33 2024 +1000)

# 1.2.4 - Bugfix Release

* 978677cc - chore: Upgrade pact_matching to 1.2.6 (Ronald Holshausen, Tue Aug 13 15:27:49 2024 +1000)
* 9a7eef75 - chore: Upgrade pact_models to 1.2.4 (Ronald Holshausen, Tue Aug 13 11:51:06 2024 +1000)
* ae7d2a9c - fix(pact_verifier): Plugin feature was not enabled in the verifier (Ronald Holshausen, Tue Aug 13 11:13:51 2024 +1000)
* 80b7bb58 - fix(pact_verifier): Provider state values should be passed to any plugins under a providerState key (Ronald Holshausen, Wed Aug 7 16:06:54 2024 +1000)
* bcf4db6d - chore: Upgrade pact_models to 1.2.3 (Ronald Holshausen, Tue Aug 6 11:06:22 2024 +1000)
* 111a18cd - Feat(pact_consumer): Upgrade pact_mock_server to the new 2.0.0 version (Ronald Holshausen, Thu Jul 18 17:12:55 2024 +1000)
* 56c953b4 - chore: Upgrade pact_consumer to 1.2.3 (Ronald Holshausen, Wed Jul 17 15:28:49 2024 +1000)
* 25fd9c56 - chore: Update release script (Ronald Holshausen, Wed Jul 17 15:12:37 2024 +1000)
* c8d297f2 - bump version to 1.2.4 (Ronald Holshausen, Wed Jul 17 15:12:16 2024 +1000)

# 1.2.3 - Bugfix Release

* bb9fc1be - chore: Upgrade pact_matching to 1.2.5 (Ronald Holshausen, Wed Jul 17 14:34:04 2024 +1000)
* fd9ff70a - chore: Upgrade pact_models to 1.2.2 (Ronald Holshausen, Wed Jul 17 11:15:34 2024 +1000)
* e8b62f98 - chore: Upgrade pact-plugin-driver to 0.7.0 (Ronald Holshausen, Wed Jul 17 10:45:34 2024 +1000)
* f03cd0d3 - feat(pact_verifier_cli): Parse consumer version selector JSON when evaluating CLI args so errors are reported earlier (Ronald Holshausen, Wed Jul 10 12:13:11 2024 +1000)
* efa1295a - feat(pact_verifier): Allow provider state generator to fall back to the provider state parameters #441 (Ronald Holshausen, Tue Jul 9 17:27:09 2024 +1000)
* 42e2ba68 - fix(pact_verifier): Take context paths into account when fetching the root HAL resource #420 (Ronald Holshausen, Tue Jul 9 11:37:04 2024 +1000)
* efc54d26 - fix(FFI): update_provider_info was not setting the transport scheme correctly (Ronald Holshausen, Mon Jun 24 11:35:57 2024 +1000)
* 0e2e1b8a - chore: Cleanup compiler deprecation warnings (Ronald Holshausen, Fri Jun 14 11:25:56 2024 +1000)
* 570b1425 - bump version to 1.2.3 (Ronald Holshausen, Fri Jun 14 11:17:36 2024 +1000)

# 1.2.2 - Feature Release

* 7f741c7e - chore(pact_verifier): Update dependencies (Ronald Holshausen, Fri Jun 14 10:50:11 2024 +1000)
* c32e07c8 - chore: Upgrade pact_consumer to 1.2.2 (Ronald Holshausen, Fri Jun 14 10:41:31 2024 +1000)
* 29c3b5f0 - chore: Upgrade pact_matching to 1.2.4 (Ronald Holshausen, Fri Jun 14 10:18:26 2024 +1000)
* 30998a95 - chore: Upgrade pact_models to 1.2.1 (Ronald Holshausen, Wed Jun 12 15:26:49 2024 +1000)
* af347c25 - feat(verifier): support fallback_branch consumer version selector (Yousaf Nabi, Sun Jun 9 21:32:25 2024 +0100)
* a811b8a4 - chore: Update pact_matching to 1.2.3 (Ronald Holshausen, Thu May 30 15:57:32 2024 +1000)
* e44458a8 - chore(pact_verifier): Exclude parameters from the span which are included in the previous span (Ronald Holshausen, Mon May 13 14:15:18 2024 +1000)
* fe92d88e - bump version to 1.2.2 (Ronald Holshausen, Wed May 8 15:13:26 2024 +1000)

# 1.2.1 - Maintenance Release

* c3cae83e - chore: Upgrade pact_consumer to 1.2.2 (Ronald Holshausen, Wed May 8 14:46:19 2024 +1000)
* fa2b1d09 - chore: Upgrade pact_matching to 1.2.2 (Ronald Holshausen, Tue May 7 10:50:08 2024 +1000)
* 694100fb - chore: Update pact_models to 1.2.0 (Ronald Holshausen, Tue Apr 23 10:51:11 2024 +1000)
* edfac7ba - chore: remove local pact_models from the other pact crates (Ronald Holshausen, Tue Apr 23 10:03:29 2024 +1000)
* c3128a6d - feat: Support optional query parameter values (where there is only a name) (Ronald Holshausen, Mon Apr 22 10:36:05 2024 +1000)
* 758f4c03 - chore: Update pact_matching to 1.2.1 (Ronald Holshausen, Tue Apr 16 16:29:38 2024 +1000)
* 05c8d536 - bump version to 1.2.1 (Ronald Holshausen, Tue Apr 16 15:37:53 2024 +1000)

# 1.2.0 - Maintenance Release

* dffbc8e7 - chore(pact_consumer): Bump minor version (Ronald Holshausen, Tue Apr 16 11:49:14 2024 +1000)
* d625bded - Revert "update changelog for release 1.2.0" (Ronald Holshausen, Tue Apr 16 11:31:14 2024 +1000)
* 118c7b18 - update changelog for release 1.2.0 (Ronald Holshausen, Tue Apr 16 11:22:46 2024 +1000)
* bc35a287 - chore(pact_verifier): Update pact-plugin-driver to 0.6.0 (Ronald Holshausen, Tue Apr 16 10:58:31 2024 +1000)
* 0d4a72f5 - chore(pact_verifier): Update dependencies (Ronald Holshausen, Tue Apr 16 10:53:45 2024 +1000)
* 7688469a - chore(pact_verifier): Bump minor version (Ronald Holshausen, Tue Apr 16 10:41:18 2024 +1000)
* d6125b75 - chore(pact_matching): Bump minor version (Ronald Holshausen, Tue Apr 16 10:16:44 2024 +1000)
* 1664bcbc - bump version to 1.1.1 (Ronald Holshausen, Mon Feb 19 11:34:09 2024 +1100)

# 1.1.0 - Maintenance Release

* 04959d7d - feat(verifier): Include interaction description in verification results #292 (Ronald Holshausen, Mon Feb 19 07:39:21 2024 +1100)
* 2b5148b9 - feat(verifier): Add the client language to the verification results if set #307 (Ronald Holshausen, Mon Feb 19 06:44:24 2024 +1100)
* 6975203e - chore(verifier): Bump minor version #307 (Ronald Holshausen, Mon Feb 19 06:15:47 2024 +1100)
* 762f68e9 - feat(verifier): Add the verifier version to the published results #307 (Ronald Holshausen, Mon Feb 19 06:05:57 2024 +1100)
* 167c4450 - Revert "feat(verifier): Add the verifier version to the published results #307" (Ronald Holshausen, Mon Feb 19 05:59:22 2024 +1100)
* 6962fcee - feat(verifier): Add the verifier version to the published results #307 (Ronald Holshausen, Mon Feb 19 05:31:23 2024 +1100)
* 208eb753 - chore: Add test with POST with body and no content type #386 (Ronald Holshausen, Sun Feb 18 09:45:08 2024 +1100)
* a52e0ee9 - chore: Upgrade pact_matching to 1.1.10 (Ronald Holshausen, Wed Feb 7 13:20:45 2024 +1100)
* 24a26cca - chore: Update pact_models to 1.1.18 (Ronald Holshausen, Wed Feb 7 10:53:22 2024 +1100)
* 73578350 - chore: use local pact_models (JP-Ellis, Tue Feb 6 10:51:09 2024 +1100)
* 310cb604 - feat: Print provider name in 'no pacts found' message to help debugging (tien.xuan.vo, Wed Feb 7 00:21:44 2024 +0700)
* 95cbe5a9 - fix: Upgrade pact-plugin-driver to 0.5.1 (Ronald Holshausen, Wed Jan 31 19:56:04 2024 +1100)
* 1c3208e4 - chore: Upgrade pact_consumer to 1.1.1 (Ronald Holshausen, Sat Jan 20 22:04:39 2024 +1100)
* 86cb58dc - bump version to 1.0.6 (Ronald Holshausen, Sat Jan 20 20:37:10 2024 +1100)

# 1.0.5 - Bugfix Release

* bff1d573 - chore: Upgrade dependencies (Ronald Holshausen, Sat Jan 20 20:23:10 2024 +1100)
* e552bdce - chore: Upgrade pact_matching to 1.1.9 (Ronald Holshausen, Sat Jan 20 15:13:13 2024 +1100)
* 7b087acf - chore: Upgrade pact-plugin-driver to 0.5.0 (Ronald Holshausen, Sat Jan 20 14:49:21 2024 +1100)
* b735df9d - chore: Upgrade pact_models to 1.1.17 (Ronald Holshausen, Sat Jan 20 13:54:03 2024 +1100)
* 1a4bcd27 - chore: Upgrade pact_matching to 1.1.8 (Ronald Holshausen, Fri Jan 19 18:24:54 2024 +1100)
* 944613df - fix: regression - upgrade pact_models to 1.1.16 #359 (Ronald Holshausen, Fri Jan 19 14:52:36 2024 +1100)
* 403c0af1 - chore: Upgrade pact_models to 1.1.14 #355 (Ronald Holshausen, Tue Jan 16 10:31:12 2024 +1100)
* dfd13760 - chore: Upgrade pact_models to 1.1.13 #355 (Ronald Holshausen, Tue Jan 16 07:42:33 2024 +1100)
* ab3f9a56 - fix(verifier): default error formatter does not display the inner error #351 (Ronald Holshausen, Thu Dec 21 15:18:31 2023 +1100)
* aa07cdd5 - bump version to 1.0.5 (Ronald Holshausen, Wed Dec 20 14:39:08 2023 +1100)

# 1.0.4 - Maintenance Release

* f8c09785 - Revert "update changelog for release 1.0.4" (Ronald Holshausen, Wed Dec 20 14:26:46 2023 +1100)
* 71cfff46 - update changelog for release 1.0.4 (Ronald Holshausen, Wed Dec 20 14:25:12 2023 +1100)
* 8cdca5a4 - chore(pact_verifier): Upgrade all dependencies (Ronald Holshausen, Wed Dec 20 13:58:47 2023 +1100)
* 7a170fa5 - feat(verifier): Add option to only verify pact from a webhook URL #350 (Ronald Holshausen, Wed Dec 20 11:54:48 2023 +1100)
* 691f4422 - chore(pact_consumer): Bump minor version (Ronald Holshausen, Fri Dec 15 15:45:37 2023 +1100)
* 713e4098 - chore: Upgrade pact-plugin-driver to 0.4.6 (Ronald Holshausen, Thu Dec 14 17:04:59 2023 +1100)
* 3f0ae7f1 - chore: Upgrade pact_matching to 1.1.7 (Ronald Holshausen, Tue Nov 14 03:10:25 2023 +1100)
* 826758a6 - chore: Upgrade pact_models to 1.1.12 (Ronald Holshausen, Mon Nov 13 17:25:21 2023 +1100)
* 04bad264 - chore: Upgrade pact_matching to 1.1.6 (Ronald Holshausen, Fri Sep 22 11:03:38 2023 +1000)
* 2bac8b7a - bump version to 1.0.4 (Ronald Holshausen, Tue Aug 29 10:00:03 2023 +1000)

# 1.0.3 - Bugfix Release

* 3ec99c41 - chore: Upgrade pact_matching to 1.1.5 (Ronald Holshausen, Fri Aug 18 15:40:02 2023 +1000)
* 40009089 - Merge pull request #308 from tienvx/fix-wrong-plugin-version (Ronald Holshausen, Fri Aug 18 14:01:37 2023 +1000)
* 87e49674 - chore: update message (Beth Skurrie, Tue Aug 15 09:31:39 2023 +1000)
* a2706eee - chore: update 'could not load pacts' message to 'no pacts found' (Beth Skurrie, Tue Aug 15 09:26:44 2023 +1000)
* 6df8ce82 - fix(pact_verifier): Fix missing PATCH version in plugin's version (tienvx, Mon Aug 7 18:14:51 2023 +0700)
* e4da3e42 - chore: Upgrade pact_models to 1.1.11 (Ronald Holshausen, Mon Aug 7 13:59:34 2023 +1000)
* 24ed7835 - chore: Upgrade pact-models to 1.1.10 (Ronald Holshausen, Fri Aug 4 16:11:24 2023 +1000)
* e191e072 - bump version to 1.0.3 (Ronald Holshausen, Thu Jul 27 15:03:17 2023 +1000)

# 1.0.2 - Bugfix Release

* 8f88192e - chore: Upgrade pact_matching to 1.1.4 (Ronald Holshausen, Thu Jul 27 14:35:27 2023 +1000)
* 4a01919a - chore: Upgrade pact_models to 1.1.9 (Ronald Holshausen, Thu Jul 27 10:24:00 2023 +1000)
* 7e1a102e - bump version to 1.0.2 (Ronald Holshausen, Wed Jul 12 11:20:41 2023 +1000)

# 1.0.1 - Bugfix Release

* c2aad1ac - chore: Add support for datetime, xml, multipart and plugins crate features (Ronald Holshausen, Wed Jul 12 11:15:37 2023 +1000)
* 348eb3f3 - chore: Upgrade pact_matcing to 1.1.3 (Ronald Holshausen, Tue Jul 11 11:38:26 2023 +1000)
* f2ae77ba - chore: Upgrade pact-plugin-driver to 0.4.5 (Ronald Holshausen, Mon Jul 10 17:15:20 2023 +1000)
* b18b9dff - chore: Upgrade pact_matching to 1.1.2 (Ronald Holshausen, Mon Jul 10 16:42:27 2023 +1000)
* 1deca59a - chore: Upgrade pact_models to 1.1.8 (Ronald Holshausen, Mon Jul 10 16:15:43 2023 +1000)
* 2662cdfc - chore: Upgrade pact_models to 1.1.7 (Ronald Holshausen, Thu Jul 6 10:27:25 2023 +1000)
* 95753e29 - fix: Correct the use of matching rules on repeated header values (Ronald Holshausen, Wed Jun 28 11:57:35 2023 +1000)
* e95ae4d0 - chore: Upgrade pact_models to 1.1.6 (Ronald Holshausen, Thu Jun 22 15:40:55 2023 +1000)
* 39bb7ff1 - chore(verifier): cleanup some log entries (Ronald Holshausen, Mon Jun 5 15:14:19 2023 +1000)
* bc68ed7f - chore: Upgrade pact_models to 1.1.4 (Ronald Holshausen, Thu Jun 1 10:22:38 2023 +1000)
* 693418fb - fix: Provider state teardown calls were not being invoked when there are no provider states (Ronald Holshausen, Wed May 31 13:39:49 2023 +1000)
* 61bd331a - fix: correct test after changes for compatibility suite (Ronald Holshausen, Tue May 30 12:26:07 2023 +1000)
* 8b0ecd8b - fix: Correct verifier error logging and handling optional JSON fields (Ronald Holshausen, Tue May 30 11:51:41 2023 +1000)
* 397c837f - chore: Upgrade pact_models to 1.1.3 (fixes MockServerURL generator) (Ronald Holshausen, Mon May 29 15:12:22 2023 +1000)
* 2211094e - feat: Implemented the initial provider verification campatibility suite steps (Ronald Holshausen, Mon May 29 09:31:09 2023 +1000)
* c2d925e9 - chore: Upgrade pact_consumer to 1.0.0 (Ronald Holshausen, Tue May 23 16:05:12 2023 +1000)
* c3a88262 - bump version to 1.0.1 (Ronald Holshausen, Tue May 23 15:23:22 2023 +1000)

# 1.0.0 - Bugfixes + Update Pact models to 1.1 (breaking change)

* ca1e37fb - chore: Upgrade pact_verifier to 1.0.0 (Ronald Holshausen, Tue May 23 15:16:08 2023 +1000)
* 71b38a87 - fix: Add Custom Header option not replacing already existing headers #275 (Ronald Holshausen, Tue May 23 15:05:20 2023 +1000)
* 8f27f9bd - chore: Upgrade pact-plugin-driver to 0.4.4 (Ronald Holshausen, Tue May 23 11:55:23 2023 +1000)
* ac2e24da - chore: Use "Minimum version, with restricted compatibility range" for all Pact crate versions (Ronald Holshausen, Tue May 23 11:46:52 2023 +1000)
* 6df4670c - chore: Upgrade pact_matching to 1.1.1 (Ronald Holshausen, Tue May 23 11:32:51 2023 +1000)
* 54887690 - chore: Bump pact_matching to 1.1 (Ronald Holshausen, Tue May 23 11:13:14 2023 +1000)
* 64d6a75c - feat: implemented initial compatibility tests (Ronald Holshausen, Wed May 17 12:14:36 2023 +1000)
* 261ecf47 - fix: Add RefUnwindSafe trait bound to all Pact and Interaction uses (Ronald Holshausen, Mon May 15 13:59:31 2023 +1000)
* 46628a8b - chore: correct failing tests after changing validation error handling #273 (Ronald Holshausen, Mon May 8 16:47:24 2023 +1000)
* 59946c3f - fix: hanlde validation errors from Pactbroker correctly #273 (Ronald Holshausen, Mon May 8 15:40:11 2023 +1000)
* 00301196 - bump version to 0.15.4 (Ronald Holshausen, Tue Apr 18 15:18:36 2023 +1000)

# 0.15.3 - Bugfix Release

* 0bcba082 - chore: Upgrade pact_matching to 1.0.8 (Ronald Holshausen, Tue Apr 18 13:14:38 2023 +1000)
* 6c14abfd - chore: Upgrade pact_models to 1.0.13 (Ronald Holshausen, Tue Apr 18 13:00:01 2023 +1000)
* ce16d43f - chore: Upgrade pact-plugin-driver to 0.4.2 (supports auto-installing known plugins) (Ronald Holshausen, Tue Apr 18 11:49:52 2023 +1000)
* 10bf1a48 - chore: Upgrade pact_models to 1.0.12 (fixes generators hash function) (Ronald Holshausen, Mon Apr 17 10:31:09 2023 +1000)
* 84b9d9e9 - fix: Upgrade pact models to 1.0.11 (fixes generated key for V4 Pacts) (Ronald Holshausen, Fri Apr 14 17:10:58 2023 +1000)
* 669f7812 - chore: Upgrade pact_models to 1.0.10 (Ronald Holshausen, Thu Apr 13 15:32:34 2023 +1000)
* 779a59f0 - fix: Upgrade pact-plugin-driver to 0.4.1 (fixes an issue introduced in 0.4.0 with shared channels to plugins) (Ronald Holshausen, Wed Apr 5 17:01:18 2023 +1000)
* b4c09bee - bump version to 0.15.3 (Ronald Holshausen, Wed Apr 5 10:02:37 2023 +1000)

# 0.15.2 - Maintenance Release

* 126cf462 - chore: Upgrade pact_matching to 1.0.7 (Ronald Holshausen, Tue Apr 4 15:12:28 2023 +1000)
* 6f0c4b2f - feat: Upgrade pact-plugin-driver to 0.4.0 which uses a shared gRPC channel to each plugin (Ronald Holshausen, Tue Apr 4 14:32:36 2023 +1000)
* 63be53b2 - fix: allow the pact builders to set the overwrite flag (Ronald Holshausen, Mon Apr 3 14:53:36 2023 +1000)
* c6b66a28 - chore: Test was failing with dates where day of the month has one digit (Ronald Holshausen, Mon Apr 3 11:55:45 2023 +1000)
* 2e3feacd - bump version to 0.15.2 (Ronald Holshausen, Wed Mar 15 15:16:42 2023 +1100)

# 0.15.1 - Bugfix Release

* 11c701b4 - fix: Upgrade pact_matching to 1.0.6 (fixes some issues with matching HTTP headers) (Ronald Holshausen, Wed Mar 15 14:54:54 2023 +1100)
* 7e72df6e - chore: add a verification test with a Last-Modified header #259 (Ronald Holshausen, Wed Mar 15 14:33:59 2023 +1100)
* e96bc54e - fix: Upgrade pact_models to 1.0.9 (fixes issues with headers) (Ronald Holshausen, Wed Mar 15 14:31:00 2023 +1100)
* b479f233 - fix: do not split header values for headers like Date, Last-Modified, etc. #259 (Ronald Holshausen, Wed Mar 15 12:33:09 2023 +1100)
* f7e0b669 - chore: Upgrade pact_models to 1.0.8 (Ronald Holshausen, Wed Mar 15 12:19:22 2023 +1100)
* 57728a01 - chore: update pact-plugin-driver to 0.3.3 (Ronald Holshausen, Tue Mar 14 17:19:20 2023 +1100)
* b87ea333 - bump version to 0.15.1 (Ronald Holshausen, Thu Mar 2 12:50:29 2023 +1100)

# 0.15.0 - Supports JUnit XML report files

* 9629c351 - chore: dump minor version of pact_verifier as some signatures have changed (Ronald Holshausen, Thu Mar 2 12:10:35 2023 +1100)
* c9333f94 - feat: add option to generate JUnit XML report format for consumption by CI servers #257 (Ronald Holshausen, Thu Mar 2 10:48:56 2023 +1100)
* 46297622 - feat: add verification timing to the verifier output (Ronald Holshausen, Mon Feb 27 16:11:18 2023 +1100)
* 1355837e - refactor: add timeing to load pact operations (Ronald Holshausen, Mon Feb 27 14:10:29 2023 +1100)
* 50ecb585 - bump version to 0.14.1 (Ronald Holshausen, Thu Feb 16 14:14:18 2023 +1100)

# 0.14.0 - Bugfix Release

* c368c651 - fix: Pass any custom header values on to the plugin verification call (Ronald Holshausen, Thu Feb 16 13:52:03 2023 +1100)
* 0676047e - chore: Upgrade pact-plugin-driver to 0.3.2 (Ronald Holshausen, Thu Feb 16 12:09:46 2023 +1100)
* 8ad10cc2 - bump version to 0.13.22 (Ronald Holshausen, Fri Feb 10 13:27:34 2023 +1100)

# 0.13.21 - Bugfix Release

* 0a248af1 - fix: Verification output comes from the plugin, so do not display any when a plugin is used (Ronald Holshausen, Fri Feb 10 12:46:13 2023 +1100)
* 1e7331f1 - fix: Upgrade plugin driver to 0.3.1 (Ronald Holshausen, Wed Feb 8 13:28:07 2023 +1100)
* 0f4178e5 - chore: Upgrade pact_matching to 1.0.4 (Ronald Holshausen, Mon Feb 6 15:40:43 2023 +1100)
* 0b70060f - chore: Upgrade pact-plugin-driver and base64 crates (supports message metadata) (Ronald Holshausen, Mon Feb 6 14:56:29 2023 +1100)
* 6e41c5ee - bump version to 0.13.21 (Ronald Holshausen, Wed Jan 11 15:42:41 2023 +1100)

# 0.13.20 - Bugfix Release

* ee69e75c - chore: Upgrade base64 crate (Ronald Holshausen, Wed Jan 11 15:38:21 2023 +1100)
* c1b22f1c - chore: Upgrade pact_matching to 1.0.3 (Ronald Holshausen, Wed Jan 11 15:19:29 2023 +1100)
* 7d84d941 - chore: Upgrade pact_models to 1.0.4 (Ronald Holshausen, Wed Jan 11 14:33:13 2023 +1100)
* a8abf5df - chore: log spans at trace level to reduce the log entry size at other log levels #243 (Ronald Holshausen, Tue Jan 10 09:00:52 2023 +1100)
* 4409441b - fix: Matching rules are not being applied correctly to message metadata #245 (Ronald Holshausen, Mon Jan 9 13:43:41 2023 +1100)
* 34a67cb9 - fix: when loading pacts from a dir, filter by the provider name #233 (Ronald Holshausen, Wed Jan 4 18:12:28 2023 +1100)
* 8216ec76 - feat: always execute provider_states callbacks even when no state is defined (Ronald Holshausen, Tue Jan 3 14:40:45 2023 +1100)
* 4dbf3026 - chore: Add a verification test for message pact #239 (Ronald Holshausen, Thu Dec 22 15:38:58 2022 +1100)
* 1bdb1054 - chore: Upgrade pact_models to 1.0.3 #239 (Ronald Holshausen, Thu Dec 22 15:37:53 2022 +1100)
* 3aecb702 - chore: require tracing-subscriber for tests for crates that use pact_models #239 (Ronald Holshausen, Thu Dec 22 14:37:01 2022 +1100)
* 24429923 - bump version to 0.13.20 (Ronald Holshausen, Mon Dec 19 16:16:05 2022 +1100)

# 0.13.19 - Add user-agent header + Support generators in plugins

* e827f591 - chore: Upgrade pact_matching to 1.0.2 (Ronald Holshausen, Mon Dec 19 15:30:14 2022 +1100)
* 8bb2534d - feat: add user-agent header to the HALClient (andrisak, Sat Dec 17 12:42:05 2022 +0100)
* 5fbb0d6a - feat: Upgrade plugin driver to 0.2.2 (supports passing a test context to support generators) (Ronald Holshausen, Fri Dec 16 16:38:03 2022 +1100)
* 1ab47c6f - chore: Upgrade Tokio to latest (Ronald Holshausen, Fri Dec 16 16:31:31 2022 +1100)
* 46868427 - bump version to 0.13.19 (Ronald Holshausen, Wed Dec 14 17:12:51 2022 +1100)

# 0.13.18 - Maintenance Release

* fb2f4204 - chore: Upgrade pact_matching to 1.0.1 (Ronald Holshausen, Wed Dec 14 17:03:31 2022 +1100)
* 8be00f0c - chore: Upgrade pact-plugin-driver to 0.2.1 (Ronald Holshausen, Wed Dec 14 14:55:32 2022 +1100)
* c5e3431c - bump version to 0.13.18 (Ronald Holshausen, Mon Dec 12 11:13:47 2022 +1100)

# 0.13.17 - Bugfix + Support plugins generating interaction content

* e7a1b9f2 - chore: Upgrade pact_matching to 1.0 and plugin driver to 0.2 (Ronald Holshausen, Fri Dec 9 17:29:33 2022 +1100)
* 9fc56580 - fix: add tests for PUT and POST requests #220 (Ronald Holshausen, Thu Dec 1 11:42:31 2022 +1100)
* 77a7c8ba - fix: make HAL client fetch and fetch link functions support brokers hosted with context paths #220 (Ronald Holshausen, Wed Nov 30 18:20:23 2022 +1100)
* 3d85ba0b - bump version to 0.13.17 (Ronald Holshausen, Mon Nov 28 15:02:29 2022 +1100)

# 0.13.16 - Bugfix Release

* 2802fffd - chore: Upgrade pact_matching to 0.12.15 (Ronald Holshausen, Mon Nov 28 14:29:43 2022 +1100)
* c9721fd5 - chore: Upgrade pact_models to 1.0.1 and pact-plugin-driver to 0.1.16 (Ronald Holshausen, Mon Nov 28 14:10:53 2022 +1100)
* c12d9a61 - fix: Verification results across multiple pacts accumulate, publishing invalid results #231 (Ronald Holshausen, Mon Nov 28 12:00:38 2022 +1100)
* 33a784a0 - fix: add test for publish verification result issue #231 (Ronald Holshausen, Mon Nov 28 11:04:38 2022 +1100)
* 35202f6b - bump version to 0.13.16 (Ronald Holshausen, Mon Nov 7 14:05:55 2022 +1100)

# 0.13.15 - Maintenance Release

* 5d88913a - chore: Upgrade all dependencies (Ronald Holshausen, Mon Nov 7 14:01:55 2022 +1100)
* 8ec8fe9b - chore: Upgrade pact_consumer to 0.10.0 (Ronald Holshausen, Mon Nov 7 13:15:23 2022 +1100)
* 123060e3 - chore: Upgrade pact_matching to 0.12.14 (Ronald Holshausen, Mon Nov 7 11:34:36 2022 +1100)
* 577824e7 - fix: Upgrade pact_models to 1.0 and pact-plugin-driver to 0.1.15 to fix cyclic dependency issue (Ronald Holshausen, Mon Nov 7 11:14:20 2022 +1100)
* e1f985ad - chore: Upgrade pact_models to 0.4.6 and pact-plugin-driver to 0.1.14 (Ronald Holshausen, Fri Nov 4 16:38:36 2022 +1100)
* 965a1c41 - fix: Upgrade plugin driver to 0.1.13 (fixes issue loading plugin when there are multiple versions for the same plugin) (Ronald Holshausen, Wed Oct 5 17:29:37 2022 +1100)
* 4298db15 - bump version to 0.13.15 (Ronald Holshausen, Wed Sep 28 10:32:17 2022 +1000)

# 0.13.14 - Maintenance Release

* 02d9e2cb - chore: Upgrade pact matching crate to 0.12.12 (Ronald Holshausen, Wed Sep 28 10:11:11 2022 +1000)
* b8d263f7 - fix(verifier): fix typos in the implementation of Display on the PactSource enum (Jerry Wang, Wed Sep 21 22:32:56 2022 -0700)
* 60b2b642 - chore: Upgrade pact-plugin-driver to 0.1.12 (Ronald Holshausen, Mon Sep 12 17:44:13 2022 +1000)
* ac4fe73f - chore: fix to release scripts (Ronald Holshausen, Wed Sep 7 10:51:01 2022 +1000)
* c13c3f42 - bump version to 0.13.14 (Ronald Holshausen, Wed Sep 7 09:50:47 2022 +1000)

# 0.13.13 - Bugfix Release

* 7f51bdc6 - fix: publishing provider branch was broken when invoked via a webhook call (Ronald Holshausen, Tue Sep 6 18:15:16 2022 +1000)
* da163c42 - bump version to 0.13.13 (Ronald Holshausen, Wed Aug 31 16:06:30 2022 +1000)

# 0.13.12 - Bugfix Release

* 8663cd3f - feat: add ignore-no-pacts-error to the verifier CLI #213 (Ronald Holshausen, Wed Aug 31 15:19:31 2022 +1000)
* f8db90d2 - fix: Upgrade pact_models to 0.4.5 - fixes FFI bug with generators for request paths (Ronald Holshausen, Fri Aug 26 11:44:08 2022 +1000)
* 8ca3303b - feat: add a test to check for error result with IO error #213 (Ronald Holshausen, Fri Aug 19 16:53:17 2022 +1000)
* 43be2e83 - feat: do not output an error if no_pacts_is_error is false and no pacts were found to verify #213 (Ronald Holshausen, Fri Aug 19 16:49:19 2022 +1000)
* 786f002d - bump version to 0.13.12 (Ronald Holshausen, Thu Aug 18 16:31:02 2022 +1000)

# 0.13.11 - Maintenance Release

* d6eaf9cf - chore: Upgrade pact_consumer to 0.9.6 (Ronald Holshausen, Thu Aug 18 16:14:44 2022 +1000)
* 1b1c77e6 - chore: cleanup some compiler warnings (Ronald Holshausen, Thu Aug 18 16:06:43 2022 +1000)
* 1d5fb787 - chore: Upgrade pact_matching to 0.12.11 (Ronald Holshausen, Thu Aug 18 15:07:23 2022 +1000)
* 32a70382 - chore: Upgrade pact_models (0.4.4), plugin driver (0.1.10), tracing and tracing core crates (Ronald Holshausen, Thu Aug 18 14:41:52 2022 +1000)
* 8056d7e9 - fix: get verify_provider_async to wait on the metric call (Ronald Holshausen, Thu Aug 11 16:16:18 2022 +1000)
* 6c932b32 - bump version to 0.13.11 (Ronald Holshausen, Wed Aug 10 12:54:55 2022 +1000)

# 0.13.10 - Support multiple protocol transports with the verifier

* 4c8ee7f9 - chore: cleanup some deprecation warnings (Ronald Holshausen, Wed Aug 10 12:49:55 2022 +1000)
* 7b6a919b - chore: Upgrade pact_matching crate to 0.12.10 (Ronald Holshausen, Wed Aug 10 12:37:11 2022 +1000)
* 33b04eee - chore: cleanup some deprecation warnings (Ronald Holshausen, Wed Aug 10 10:34:58 2022 +1000)
* 195ad07b - chore: Updated dependant crates (uuid, simplelog) (Ronald Holshausen, Wed Aug 10 10:22:07 2022 +1000)
* 49232caa - chore: Update pact plugin driver to 0.1.9 (Ronald Holshausen, Wed Aug 10 10:14:42 2022 +1000)
* a3fe5e7f - chore: Update pact models to 0.4.2 (Ronald Holshausen, Wed Aug 10 10:10:41 2022 +1000)
* 4587a430 - fix: results for sync messages were not being displayed (Ronald Holshausen, Wed Aug 3 16:14:32 2022 +1000)
* ce11a619 - feat: allow sensible defaults for interaction transports (Ronald Holshausen, Wed Aug 3 13:58:50 2022 +1000)
* 3a1449cb - feat: use the configured transport when provided (Ronald Holshausen, Wed Aug 3 13:20:17 2022 +1000)
* 8cc29482 - feat: add CLI options to provide different ports when there are different transports (Ronald Holshausen, Wed Aug 3 11:53:31 2022 +1000)
* 7673269a - bump version to 0.13.10 (Ronald Holshausen, Wed Jul 20 12:39:25 2022 +1000)

# 0.13.9 - Add option to disable color output

* 40f7bdc4 - feat: add verification option to disable ANSI escape codes in output #203 (Ronald Holshausen, Wed Jul 20 12:18:12 2022 +1000)
* 9a6c846f - chore: Upgrade pact_matching to 0.12.9 (Ronald Holshausen, Fri Jun 10 15:46:07 2022 +1000)
* a73c75ec - bump version to 0.13.9 (Ronald Holshausen, Tue Jun 7 10:58:54 2022 +1000)

# 0.13.8 - Support publishing results from webhook calls

* 18118e82 - feat: add retries to the provider state change calls #197 (Ronald Holshausen, Tue Jun 7 09:10:23 2022 +1000)
* 6cae9b09 - fix: State change descriptions were not being displayed along with the interaction description (Ronald Holshausen, Mon Jun 6 17:09:44 2022 +1000)
* 1972a74a - feat: Detect Pactbroker responses from the URL content #199 (Ronald Holshausen, Mon Jun 6 14:48:06 2022 +1000)
* 27f6cd32 - refactor: convert fetch_pact to use anyhow::Result #199 (Ronald Holshausen, Mon Jun 6 12:13:19 2022 +1000)
* e671e3dd - chore: Upgrade pact_consumer to 0.9.5 (Ronald Holshausen, Mon May 30 12:30:00 2022 +1000)
* 3088d117 - bump version to 0.13.8 (Ronald Holshausen, Mon May 30 12:16:57 2022 +1000)

# 0.13.7 - Maintenance Release

* bcddbcfb - chore: Upgrade pact_matching to 0.12.8 (Ronald Holshausen, Mon May 30 11:52:26 2022 +1000)
* 80256458 - chore: Upgrade pact-plugin-driver to 0.1.8 (Ronald Holshausen, Mon May 30 11:36:54 2022 +1000)
* 873f0c93 - fix(ffi): resources were not freed correctly when the mock server is provided by a plugin (Ronald Holshausen, Mon May 30 11:05:20 2022 +1000)
* f80f6638 - chore: Upgrade pact_consumer to 0.9.4 (Ronald Holshausen, Mon May 23 14:41:57 2022 +1000)
* d9b9fe72 - chore: Upgrade pact-plugin-driver to 0.1.7 (Ronald Holshausen, Fri May 20 15:56:23 2022 +1000)
* f76ddd8e - feat: allow BrokerWithDynamicConfiguration to publish results (kageru, Tue May 17 11:42:14 2022 +0200)
* 1d06f19f - chore: Upgrade pact_consumer to 0.9.3 (Ronald Holshausen, Wed May 11 18:03:10 2022 +1000)
* b7c5cbec - bump version to 0.13.7 (Ronald Holshausen, Wed May 11 17:38:48 2022 +1000)

# 0.13.6 - Maintenance Release

* db94b1aa - chore: update readme (Ronald Holshausen, Wed May 11 17:30:50 2022 +1000)
* 072996b4 - chore: switch from logging crate to tracing crate (Ronald Holshausen, Wed May 11 17:22:14 2022 +1000)
* ba2252dd - chore: Upgrade crate dependencies (Ronald Holshausen, Wed May 11 17:17:07 2022 +1000)
* 08f28e4a - chore: Upgrade pact_matching to 0.12.7 (Ronald Holshausen, Wed May 11 15:57:36 2022 +1000)
* 37bfc5de - chore: Upgrade pact-plugin-driver to 0.1.6 (Ronald Holshausen, Wed May 11 11:56:23 2022 +1000)
* 020b5715 - chore: upgrade pact_models to 0.4.1 (Ronald Holshausen, Wed May 11 11:36:57 2022 +1000)
* 79005d0e - bump version to 0.13.6 (Ronald Holshausen, Wed Apr 27 15:17:32 2022 +1000)

# 0.13.5 - Supports verification via plugins

* bcae77b4 - chore: upgrade pact_matching to 0.12.6 (Ronald Holshausen, Wed Apr 27 14:29:26 2022 +1000)
* bc4f04df - feat: deal with verification output from plugins (Ronald Holshausen, Wed Apr 27 11:17:25 2022 +1000)
* dba7252e - chore: Upgrade pact-plugin-driver to 0.1.5 (Ronald Holshausen, Tue Apr 26 13:56:22 2022 +1000)
* 688e49e7 - chore: Upgrade pact-plugin-driver to 0.1.4 (Ronald Holshausen, Fri Apr 22 14:47:01 2022 +1000)
* cdf72b05 - feat: forward provider details to plugin when verifying (Ronald Holshausen, Fri Apr 22 14:12:34 2022 +1000)
* 2395143a - feat: forward verification to plugin for transports provided by the plugin (Ronald Holshausen, Fri Apr 22 12:02:05 2022 +1000)
* 05c83b67 - chore: switch verifier over to tracing crate (Ronald Holshausen, Wed Apr 20 11:34:16 2022 +1000)
* 75145a60 - chore: setup tracing for verifier CLI (Ronald Holshausen, Tue Apr 19 17:20:18 2022 +1000)
* 96a9985a - chore: Upgrade pact_consumer to 0.9.1 (Ronald Holshausen, Wed Apr 13 16:20:14 2022 +1000)
* 36dcfcbc - bump version to 0.13.5 (Ronald Holshausen, Wed Apr 13 16:02:02 2022 +1000)

# 0.13.4 - Bugfix Release

* 0df06dd2 - chore: Upgrade pact_matching to 0.12.5 (Ronald Holshausen, Wed Apr 13 15:38:49 2022 +1000)
* 49640c5f - chore: minor update to release scripts (Ronald Holshausen, Wed Apr 13 15:32:46 2022 +1000)
* d043f6c7 - chore: upgrade pact_models to 0.3.3 (Ronald Holshausen, Wed Apr 13 15:24:33 2022 +1000)
* eee09ba6 - chore: Upgrade pact-plugin-driver to 0.1.3 (Ronald Holshausen, Wed Apr 13 14:07:36 2022 +1000)
* 73ae0ef0 - fix: Upgrade reqwest to 0.11.10 to resolve #156 (Ronald Holshausen, Wed Apr 13 13:31:55 2022 +1000)
* ffeca2e2 - chore: update to the latest plugin driver (Ronald Holshausen, Wed Apr 13 13:08:25 2022 +1000)
* e93c5574 - fix: when loading plugins for Pact files, only take minor + major version into account (Ronald Holshausen, Thu Mar 24 16:50:00 2022 +1100)
* 86409c98 - Revert "chore: disable Pact tests to resolve cyclic dependency issue" (Ronald Holshausen, Thu Mar 24 14:53:22 2022 +1100)
* 8f1e9506 - chore: cleanup (Ronald Holshausen, Thu Mar 24 14:49:33 2022 +1100)
* fd515bda - bump version to 0.13.4 (Ronald Holshausen, Thu Mar 24 14:48:13 2022 +1100)

# 0.13.3 - Maintenance Release

* f1f6c980 - chore: disable Pact tests to resolve cyclic dependency issue (Ronald Holshausen, Thu Mar 24 14:26:10 2022 +1100)
* d06faac5 - update changelog for release 0.13.3 (Ronald Holshausen, Thu Mar 24 14:16:41 2022 +1100)
* 89027c87 - chore: update pact_matching (0.12.4) and pact_mock_server (0.8.8) (Ronald Holshausen, Thu Mar 24 14:09:45 2022 +1100)
* 9baf03a9 - chore: use the published version of the plugin driver (Ronald Holshausen, Thu Mar 24 13:36:01 2022 +1100)
* 345b0011 - feat: support mock servers provided from plugins (Ronald Holshausen, Mon Mar 21 15:59:46 2022 +1100)
* efb5f12b - refactor: Split ValidatingMockServer into a trait and implementation (Ronald Holshausen, Tue Mar 15 17:07:15 2022 +1100)
* e10841d7 - chore: bump consumer crate to 0.9.0 (Ronald Holshausen, Tue Mar 15 14:16:47 2022 +1100)
* 31c0c368 - chore: update pact_consumer (Ronald Holshausen, Fri Mar 4 15:08:41 2022 +1100)
* 3eed67e8 - bump version to 0.13.3 (Ronald Holshausen, Fri Mar 4 14:39:55 2022 +1100)

# 0.13.2 - Maintenance Release

* 8894fdfd - chore: update pact_matching to 0.12.3 (Ronald Holshausen, Fri Mar 4 14:09:17 2022 +1100)
* 8e864502 - chore: update all dependencies (Ronald Holshausen, Fri Mar 4 13:29:59 2022 +1100)
* f52c3625 - feat: add for custom headers to the HTTP client used by the verifier #182 (Ronald Holshausen, Mon Feb 28 14:38:00 2022 +1100)
* 74bd4531 - feat: add support for custom headers with the verifier FFI calls #182 (Ronald Holshausen, Mon Feb 28 13:58:46 2022 +1100)
* 6634953f - bump version to 0.13.2 (Ronald Holshausen, Thu Feb 3 13:39:59 2022 +1100)

# 0.13.1 - Updated req/res logging for FFI calls

* 4f47ff65 - feat: log request/response bodies at debug level (Ronald Holshausen, Thu Feb 3 13:14:35 2022 +1100)
* 3423354a - chore: update schema for verification results JSON (Ronald Holshausen, Tue Feb 1 12:21:21 2022 +1100)
* 4a407202 - chore: add draft schema for verification results JSON (Ronald Holshausen, Tue Feb 1 11:17:43 2022 +1100)
* d093f7d7 - bump version to 0.13.1 (Ronald Holshausen, Mon Jan 31 11:52:34 2022 +1100)

# 0.13.0 - Capture output/results from the verification process

* d0fa29dc - feat: add json output to the verifier CLI (Ronald Holshausen, Fri Jan 28 15:21:17 2022 +1100)
* bf152233 - feat: Capture all the results from the verification process (Ronald Holshausen, Fri Jan 28 11:28:38 2022 +1100)
* 5f148cdd - feat: capture all the output from the verifier (Ronald Holshausen, Thu Jan 27 16:08:02 2022 +1100)
* 8bee40b0 - feat(ffi)!: Separate verification and publishing options (Adam Rodger, Tue Jan 25 16:31:29 2022 +0000)
* 042d580e - chore: Upgrade pact consumer to 0.8.5 (Ronald Holshausen, Mon Jan 17 17:13:55 2022 +1100)
* 9e2388de - bump version to 0.12.5 (Ronald Holshausen, Mon Jan 17 17:05:04 2022 +1100)

# 0.12.4 - Bugfix Release

* 5e4c68ef - chore: update pact matching to 0.12.2 (Ronald Holshausen, Mon Jan 17 16:29:21 2022 +1100)
* 80b241c5 - chore: Upgrade plugin driver crate to 0.0.17 (Ronald Holshausen, Mon Jan 17 11:22:48 2022 +1100)
* 4f1ecff2 - chore: Upgrade pact-models to 0.2.7 (Ronald Holshausen, Mon Jan 17 10:53:26 2022 +1100)
* c2089645 - fix: log crate version must be fixed across all crates (including plugin driver) (Ronald Holshausen, Fri Jan 14 16:10:50 2022 +1100)
* 924ddb60 - bump version to 0.12.4 (Ronald Holshausen, Tue Jan 4 13:08:32 2022 +1100)

# 0.12.3 - Maintenance Release

* e5961b33 - chore: Update pact_matching 0.12.1, pact_models 0.2.6 (Ronald Holshausen, Tue Jan 4 12:59:44 2022 +1100)
* d670585a - chore: Update plugin driver to 0.0.16 (Ronald Holshausen, Tue Jan 4 09:37:21 2022 +1100)
* 549788cd - bump version to 0.12.3 (Ronald Holshausen, Fri Dec 31 15:35:25 2021 +1100)

# 0.12.2 - Maintenance Release

* 9c2810ad - chore: Upgrade pact-plugin-driver to 0.0.15 (Ronald Holshausen, Fri Dec 31 15:12:56 2021 +1100)
* 2bd44009 - bump version to 0.12.2 (Ronald Holshausen, Thu Dec 30 15:29:33 2021 +1100)

# 0.12.1 - Maintenance Release

* 0a6e7d9d - refactor: Convert MatchingContext to a trait and use DocPath instead of string slices (Ronald Holshausen, Wed Dec 29 14:24:39 2021 +1100)
* 52bc1735 - chore: update pact_matching crate to 0.11.5 (Ronald Holshausen, Thu Dec 23 13:12:08 2021 +1100)
* 5479a634 - chore: Update pact_models (0.2.4) and pact-plugin-driver (0.0.14) (Ronald Holshausen, Thu Dec 23 12:57:02 2021 +1100)
* fc0a8360 - chore: update pact_matching to 0.11.4 (Ronald Holshausen, Mon Dec 20 12:19:36 2021 +1100)
* 8911d5b0 - chore: update to latest plugin driver crate (metrics fixes) (Ronald Holshausen, Mon Dec 20 12:11:35 2021 +1100)
* 9d4cc4ef - bump version to 0.12.1 (Ronald Holshausen, Wed Dec 15 13:34:36 2021 +1100)

# 0.12.0 - Bugfix + add metrics for validation

* f8042d6b - feat: add metrics event for provider verification (Ronald Holshausen, Tue Dec 14 17:29:44 2021 +1100)
* 4f1ba7d9 - chore: update to the latest plugin driver (Ronald Holshausen, Tue Dec 14 13:55:02 2021 +1100)
* 6466545f - fix(verifier): provider state executor teardown function does not need to be async (Ronald Holshausen, Tue Dec 7 11:14:12 2021 +1100)
* 1768141e - fix(verifier test): missing addition of teardown impl (Mike Geeves, Mon Dec 6 13:12:45 2021 +0000)
* 5f782d67 - fix(verifier): the state_change_teardown option didn't appear to actually be used (Mike Geeves, Mon Dec 6 11:46:58 2021 +0000)
* 04dd9ab2 - bump version to 0.11.3 (Ronald Holshausen, Thu Dec 2 12:15:29 2021 +1100)

# 0.11.2 - Bugfix Release

* 9f7e22dc - Revert "update changelog for release 0.11.2" (Ronald Holshausen, Thu Dec 2 11:46:17 2021 +1100)
* 707f8f98 - update changelog for release 0.11.2 (Ronald Holshausen, Thu Dec 2 11:45:02 2021 +1100)
* 59b49c80 - chore: upgrade to latest models and plugins crate (Ronald Holshausen, Thu Dec 2 11:42:06 2021 +1100)
* f4fdba3c - fix: Templated values in HAL links need to be URL encoded #166 (Ronald Holshausen, Thu Dec 2 11:22:15 2021 +1100)
* 29605ab0 - fix: support specifying matching_branch in verifications (Matt Fellows, Wed Nov 17 18:59:36 2021 +1100)
* 260deb70 - fix: support specifying matching_branch in verifications (Matt Fellows, Wed Nov 17 17:47:37 2021 +1100)
* c45faa2c - feat: support specifying matching_branch in verifications. Fixes #158 (Matt Fellows, Wed Nov 17 17:36:49 2021 +1100)
* fc5be202 - fix: update to latest driver crate (Ronald Holshausen, Tue Nov 16 16:19:02 2021 +1100)
* 1be76c50 - bump version to 0.11.2 (Ronald Holshausen, Tue Nov 16 12:26:51 2021 +1100)

# 0.11.1 - Update to latest models and plugin driver crates

* 5d974c4a - chore: update to latest models and plugin driver crates (Ronald Holshausen, Tue Nov 16 11:56:53 2021 +1100)
* 6dfec56a - chore: drop beta from pact_consumer version (Ronald Holshausen, Thu Nov 4 16:08:47 2021 +1100)
* 41fc4380 - bump version to 0.11.1 (Ronald Holshausen, Thu Nov 4 16:06:08 2021 +1100)

# 0.11.0 - Pact V4 release

* 400a1231 - chore: drop beta from pact_verifier version (Ronald Holshausen, Thu Nov 4 15:56:22 2021 +1100)
* bd2bd0ec - chore: drop beta from pact_matching version (Ronald Holshausen, Wed Nov 3 13:28:35 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* a561f883 - chore: use the non-beta models crate (Ronald Holshausen, Thu Oct 21 18:10:27 2021 +1100)
* ec265d83 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Oct 20 14:40:37 2021 +1100)
* 630e8f9c - bump version to 0.11.0-beta.3 (Ronald Holshausen, Tue Oct 19 17:40:52 2021 +1100)
* d171edfd - feat: support provider branches (Matt Fellows, Wed Sep 29 22:47:21 2021 +1000)

# 0.11.0-beta.2 - Bugfix Release

* 1677501d - refactor: moved consumer version selector functions from pact_ffi crate (Ronald Holshausen, Tue Oct 19 17:31:13 2021 +1100)
* 918e5beb - fix: update to latest models and plugin driver crates (Ronald Holshausen, Tue Oct 19 17:09:48 2021 +1100)
* 1539050c - bump version to 0.11.0-beta.2 (Ronald Holshausen, Tue Oct 19 11:44:42 2021 +1100)

# 0.11.0-beta.1 - Plugin support with verifying pacts

* 3819522d - chore: update to the latest matching and mock server crates (Ronald Holshausen, Tue Oct 19 11:34:18 2021 +1100)
* aa434ba3 - chore: update to latest driver crate (Ronald Holshausen, Tue Oct 19 11:09:46 2021 +1100)
* bfa04370 - fix: display the error message when the verification can not be run due to an error (Ronald Holshausen, Tue Oct 19 11:09:21 2021 +1100)
* df386c8a - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Oct 18 13:41:36 2021 +1100)
* 2b4b7cc3 - feat(plugins): Support matching synchronous request/response messages (Ronald Holshausen, Fri Oct 15 16:01:50 2021 +1100)
* 9bbbb52e - chore: bump pact matching crate version (Ronald Holshausen, Tue Oct 12 16:24:01 2021 +1100)
* 1eb37c13 - chore: use the published version of the models crate (Ronald Holshausen, Thu Oct 7 10:49:11 2021 +1100)
* 2c47023c - chore: pin plugin driver version to 0.0.3 (Ronald Holshausen, Wed Oct 6 11:21:07 2021 +1100)
* 288e2168 - chore: use the published version of the plugin driver lib (Ronald Holshausen, Tue Oct 5 15:36:06 2021 +1100)
* 5525b039 - feat(plugins): cleaned up the verfier output (Ronald Holshausen, Thu Sep 30 16:19:15 2021 +1000)
* 6f20282d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 28 14:51:34 2021 +1000)
* b3732c0b - bump version to 0.10.14 (Ronald Holshausen, Tue Sep 28 13:56:56 2021 +1000)
* 4fd7a429 - update changelog for release 0.10.13 (Ronald Holshausen, Tue Sep 28 13:52:33 2021 +1000)
* 42be9eb8 - feat: add FFI functions to extract logs from a verifcation run (Ronald Holshausen, Tue Sep 28 12:48:40 2021 +1000)
* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* ee3212a8 - refactor(plugins): do not expose the catalogue statics, but rather a function to initialise it (Ronald Holshausen, Tue Sep 14 15:13:12 2021 +1000)
* b71dcabf - refactor(plugins): rename ContentTypeOverride -> ContentTypeHint (Ronald Holshausen, Tue Sep 14 15:08:52 2021 +1000)
* 9c7af69a - bump version to 0.11.0-beta.1 (Ronald Holshausen, Mon Sep 13 12:14:46 2021 +1000)

# 0.10.13 - support native TLS certs

* 42be9eb8 - feat: add FFI functions to extract logs from a verifcation run (Ronald Holshausen, Tue Sep 28 12:48:40 2021 +1000)
* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 05f4c3de - feat: add verifier ffi function set verification options (tienvx, Wed Sep 8 23:48:13 2021 +0700)
* 5ac0d219 - bump version to 0.10.13 (Ronald Holshausen, Wed Sep 8 10:32:49 2021 +1000)

# 0.11.0-beta.0 - Support for plugins when verifying pacts

* f55440c6 - chore: Bump verifier lib version to 0.11.0-beta.0 (Ronald Holshausen, Mon Sep 13 12:04:19 2021 +1000)
* 03ebe632 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Sep 13 12:01:13 2021 +1000)
* fd6f8f40 - chore: Bump pact_mock_server version to 0.8.0-beta.0 (Ronald Holshausen, Mon Sep 13 11:46:11 2021 +1000)
* 05f4c3de - feat: add verifier ffi function set verification options (tienvx, Wed Sep 8 23:48:13 2021 +0700)
* 716809f6 - chore: Get CI build passing (Ronald Holshausen, Fri Sep 10 14:55:46 2021 +1000)
* 5ac0d219 - bump version to 0.10.13 (Ronald Holshausen, Wed Sep 8 10:32:49 2021 +1000)
* ceb1c35f - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 7 10:07:45 2021 +1000)
* b77498c8 - chore: fix tests after updating plugin API (Ronald Holshausen, Fri Sep 3 16:48:18 2021 +1000)
* e8ae81b3 - refactor: matching req/res with plugins requires data from the pact and interaction (Ronald Holshausen, Thu Sep 2 11:57:50 2021 +1000)
* b9aa7ecb - feat(Plugins): allow plugins to override text/binary format of the interaction content (Ronald Holshausen, Mon Aug 30 10:48:04 2021 +1000)
* eb34b011 - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Aug 23 15:48:55 2021 +1000)
* 0c5cede2 - chore: bump models crate to 0.2 (Ronald Holshausen, Mon Aug 23 12:56:14 2021 +1000)
* 75e13fd8 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Aug 23 10:33:45 2021 +1000)
* e3a2660f - chore: fix tests after updating test builders to be async (Ronald Holshausen, Fri Aug 20 12:41:10 2021 +1000)
* b75fea5d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Aug 18 12:27:41 2021 +1000)
* 5a235414 - feat(plugins): order the matching results as plugins mau return them in any order (Ronald Holshausen, Fri Aug 13 17:18:46 2021 +1000)
* 2662241e - feat(plugins): Call out to plugins when comparing content owned by the plugin during verification (Ronald Holshausen, Fri Aug 13 14:29:30 2021 +1000)
* 60869969 - feat(plugins): Add core features to the plugin catalogue (Ronald Holshausen, Thu Aug 12 13:00:41 2021 +1000)
* bdfc6f02 - feat(plugins): Load required plugins when verifying a V4 pact (Ronald Holshausen, Wed Aug 11 14:23:54 2021 +1000)
* dfe3cd42 - chore: bump minor version of Pact verifier libs (Ronald Holshausen, Mon Aug 9 15:10:47 2021 +1000)

# 0.10.12 - Maintenance Release

* 9e582360 - chore: add verifier ffi function update provider state (tienvx, Sun Aug 29 22:20:28 2021 +0700)
* 46135a16 - chore: add verifier FFI functions for directory, URL and Pact broker sources (Ronald Holshausen, Tue Aug 24 10:14:46 2021 +1000)
* e340d2f1 - bump version to 0.10.12 (Ronald Holshausen, Sun Aug 22 15:37:43 2021 +1000)

# 0.10.11 - Bugfix Release

* 0e62fe40 - chore: set regex version to 1 (Ronald Holshausen, Sun Aug 22 15:30:12 2021 +1000)
* c274ca1a - fix: use the pacts for verification endpoint if the conusmer selectors are specified #133 (Ronald Holshausen, Sun Aug 22 11:51:22 2021 +1000)
* f56b52b2 - bump version to 0.10.11 (Ronald Holshausen, Tue Aug 17 10:48:38 2021 +1000)

# 0.10.10 - Bugfix Release

* b5a7b779 - feat: support new selectors (Matt Fellows, Mon Aug 9 13:27:33 2021 +1000)
* 8bcd1c7e - fix: min/max type matchers must not apply the limits when cascading (Ronald Holshausen, Sun Aug 8 15:50:40 2021 +1000)
* 9baa714d - chore: bump minor version of matching crate (Ronald Holshausen, Fri Jul 23 14:03:20 2021 +1000)
* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* 20f01695 - refactor: Make many JSON parsing functions fallible (Caleb Stepanian, Wed Jul 21 18:04:45 2021 -0400)
* 3dccf866 - refacfor: moved the pact structs to the models crate (Ronald Holshausen, Sun Jul 18 16:58:14 2021 +1000)
* e8046d84 - refactor: moved interaction structs to the models crate (Ronald Holshausen, Sun Jul 18 14:36:03 2021 +1000)
* b3a6f193 - chore: rename header PACT_MESSAGE_METADATA -> Pact-Message-Metadata (Matt Fellows, Tue Jul 13 11:32:25 2021 +1000)
* 0591fc47 - bump version to 0.10.10 (Ronald Holshausen, Sun Jul 11 17:31:00 2021 +1000)

# 0.10.9 - Moved structs to models crate + bugfixes and enhancements

* e2e10241 - refactor: moved Request and Response structs to the models crate (Ronald Holshausen, Wed Jul 7 18:09:36 2021 +1000)
* 9e8b01d7 - refactor: move HttpPart struct to models crate (Ronald Holshausen, Wed Jul 7 15:59:34 2021 +1000)
* 10e8ef87 - refactor: moved http_utils to the models crate (Ronald Holshausen, Wed Jul 7 14:34:20 2021 +1000)
* a935fbd6 - chore: tests for extract_headers (Matt Fellows, Tue Jul 6 10:54:39 2021 +1000)
* 33f9a823 - feat: support complex data structures in message metadata (Matt Fellows, Mon Jul 5 23:38:52 2021 +1000)
* a835e684 - feat: support message metadata in verifications (Matt Fellows, Sun Jul 4 21:02:35 2021 +1000)
* 01ff9877 - refactor: moved matching rules and generators to models crate (Ronald Holshausen, Sun Jul 4 17:17:30 2021 +1000)
* c3c22ea8 - Revert "refactor: moved matching rules and generators to models crate (part 1)" (Ronald Holshausen, Wed Jun 23 14:37:46 2021 +1000)
* 53bb86c4 - Merge branch 'release-verifier' (Ronald Holshausen, Wed Jun 23 13:59:59 2021 +1000)
* 7d69ec97 - bump version to 0.10.9 (Ronald Holshausen, Wed Jun 23 13:19:38 2021 +1000)
* d3406650 - refactor: moved matching rules and generators to models crate (part 1) (Ronald Holshausen, Wed Jun 23 12:58:30 2021 +1000)

# 0.10.8 - Refactor + Bugfixes

* 84f01d31 - chore: cleanup pedning output (Ronald Holshausen, Fri Jun 11 16:28:28 2021 +1000)
* e4927337 - chore: cleanup unused vars (Ronald Holshausen, Fri Jun 11 16:20:36 2021 +1000)
* dde8a4f6 - feat(V4): support pending interactions in the verifier (Ronald Holshausen, Fri Jun 11 16:09:29 2021 +1000)
* db75a42a - refactor: seperate displaying errors from gathering results in the verifier (Ronald Holshausen, Fri Jun 11 14:35:40 2021 +1000)
* 5c670814 - refactor: move expression_parser to pact_models crate (Ronald Holshausen, Fri Jun 11 10:51:51 2021 +1000)
* e9930740 - fix: state change URLs should not end with a slash #110 (Ronald Holshausen, Sat Jun 5 15:48:48 2021 +1000)
* 6a14ac35 - chore: add verifier test for attributes with special chars in the name (Ronald Holshausen, Wed Jun 2 15:20:00 2021 +1000)
* b4e26844 - fix: reqwest is dyn linked to openssl by default, which causes a SIGSEGV on alpine linux (Ronald Holshausen, Tue Jun 1 14:21:31 2021 +1000)
* 68f8f84e - chore: skip failing tests in alpine to get the build going (Ronald Holshausen, Tue Jun 1 13:47:20 2021 +1000)
* c690f751 - test: extract_headers function, specially with comma separated values (Artur Neumann, Mon May 31 12:59:28 2021 +0545)
* 0812d57d - Revert "update changelog for release 0.10.8" (Ronald Holshausen, Sun May 30 18:45:54 2021 +1000)
* 205b6621 - update changelog for release 0.10.8 (Ronald Holshausen, Sun May 30 18:44:14 2021 +1000)
* 4a079c64 - bump version to 0.10.8 (Ronald Holshausen, Sun May 30 18:25:27 2021 +1000)

# 0.10.7 - V4 featues + bugfixes

* 905118e - Merge pull request #109 from tonynguyenit18/fix/unmatched-expected-and-response-headers-with-multiple-value (Ronald Holshausen, Sun May 30 10:19:51 2021 +1000)
* eef6b08 - fix: correct headers attribute with multiple values might not be matched (Tony Nguyen, Sat May 29 20:55:35 2021 +0700)
* 44e7eb4 - chore: cleanup deprecation warnings (Ronald Holshausen, Sat May 29 17:55:04 2021 +1000)
* a7b81af - chore: fix clippy violation (Ronald Holshausen, Sat May 29 17:29:06 2021 +1000)
* 7022625 - refactor: move provider state models to the pact models crate (Ronald Holshausen, Sat May 29 17:18:48 2021 +1000)
* 73a53b8 - feat(V4): add an HTTP status code matcher (Ronald Holshausen, Fri May 28 18:40:11 2021 +1000)
* 62a653c - chore: remove unused imports (Matt Fellows, Thu May 27 23:40:27 2021 +1000)
* af6721a - feat: rename callback_timeout to request_timeout, and support timeouts for all http requests during verification (Matt Fellows, Thu May 27 09:04:05 2021 +1000)
* 4224088 - chore: add shasums to all release artifacts (Matt Fellows, Wed May 5 15:18:31 2021 +1000)
* b84420d - chore: add a verification test for matching values (Ronald Holshausen, Sun May 2 14:30:55 2021 +1000)
* 735c9e7 - chore: bump pact_matching to 0.9 (Ronald Holshausen, Sun Apr 25 13:50:18 2021 +1000)
* fb373b4 - chore: bump version to 0.0.2 (Ronald Holshausen, Sun Apr 25 13:40:52 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* 3dd610a - refactor: move structs and code dealing with bodies to a seperate package (Ronald Holshausen, Sun Apr 25 11:20:47 2021 +1000)
* a725ab1 - feat(V4): added synchronous request/response message formats (Ronald Holshausen, Sat Apr 24 16:05:12 2021 +1000)
* 80b7148 - feat(V4): Updated consumer DSL to set comments + mock server initial support for V4 pacts (Ronald Holshausen, Fri Apr 23 17:58:10 2021 +1000)
* 04d810b - feat(V4): display comments when verifying an interaction (Ronald Holshausen, Fri Apr 23 11:48:25 2021 +1000)
* b4bffdb - chore: correct missing changelog (Ronald Holshausen, Fri Apr 23 10:48:18 2021 +1000)
* 4bcd94f - refactor: moved OptionalBody and content types to pact models crate (Ronald Holshausen, Thu Apr 22 14:01:56 2021 +1000)
* 80812d0 - refactor: move Consumer and Provider structs to models crate (Ronald Holshausen, Thu Apr 22 13:11:03 2021 +1000)
* 220fb5e - refactor: move the PactSpecification enum to the pact_models crate (Ronald Holshausen, Thu Apr 22 11:18:26 2021 +1000)
* 2a55838 - chore: fix some Rust 2021 lint warnings (Ronald Holshausen, Wed Apr 21 16:46:47 2021 +1000)
* 9ad1474 - Merge branch 'master' of https://github.com/pact-foundation/pact-reference (Matt Fellows, Sun Apr 11 22:14:30 2021 +1000)
* a0f6a1d - refactor: Use Anyhow instead of `io::Result` (Caleb Stepanian, Wed Apr 7 16:17:35 2021 -0400)
* dcd6bed - bump version to 0.8.16 (Matt Fellows, Wed Apr 7 14:09:37 2021 +1000)

# 0.10.6 - Bugfix Release

* 63fcf49 - feat: enable consumer code to use the new Value matcher (Matt Fellows, Wed Apr 7 14:01:00 +1000)

# 0.10.5 - Bugfix Release

* 32ba4b1 - chore: update pact_matching to latest (Matt Fellows, Wed Apr 7 13:12:36 2021 +1000)
* fdae684 - update changelog for release 0.10.5 (Matt Fellows, Wed Apr 7 12:29:58 2021 +1000)
* 31e5c9c - chore: update pact_matching dependency for pact_verifier (Matt Fellows, Wed Apr 7 12:21:27 2021 +1000)
* 7cded70 - update changelog for release 0.10.5 (Matt Fellows, Wed Apr 7 12:10:43 2021 +1000)
* 89240d8 - Merge pull request #95 from pact-foundation/fix/params-missing-on-provider-state-change (Ronald Holshausen, Sun Mar 14 17:20:01 2021 +1100)
* 17682dc - fix: add missing params to provider state change executor (Matt Fellows, Sat Mar 13 08:37:46 2021 +1100)
* 656201c - feat: add exponental deplay the pact broker client retries #94 (Ronald Holshausen, Sun Mar 14 14:16:57 2021 +1100)
* e38634e - feat: add retry to the pact broker client post and put #94 (Ronald Holshausen, Sun Mar 14 14:12:26 2021 +1100)
* 8541751 - feat: add retry to the pact broker client fetch #94 (Ronald Holshausen, Sun Mar 14 13:04:20 2021 +1100)
* 4fe65fb - feat(V4): Update matching code to use matchingRules.content for V4 messages (Ronald Holshausen, Mon Mar 8 12:10:31 2021 +1100)
* 4dc5373 - bump version to 0.10.5 (Ronald Holshausen, Wed Feb 10 15:54:50 2021 +1100)

# 0.10.5 - pw

* 31e5c9c - chore: update pact_matching dependency for pact_verifier (Matt Fellows, Wed Apr 7 12:21:27 2021 +1000)
* 7cded70 - update changelog for release 0.10.5 (Matt Fellows, Wed Apr 7 12:10:43 2021 +1000)
* 89240d8 - Merge pull request #95 from pact-foundation/fix/params-missing-on-provider-state-change (Ronald Holshausen, Sun Mar 14 17:20:01 2021 +1100)
* 17682dc - fix: add missing params to provider state change executor (Matt Fellows, Sat Mar 13 08:37:46 2021 +1100)
* 656201c - feat: add exponental deplay the pact broker client retries #94 (Ronald Holshausen, Sun Mar 14 14:16:57 2021 +1100)
* e38634e - feat: add retry to the pact broker client post and put #94 (Ronald Holshausen, Sun Mar 14 14:12:26 2021 +1100)
* 8541751 - feat: add retry to the pact broker client fetch #94 (Ronald Holshausen, Sun Mar 14 13:04:20 2021 +1100)
* 4fe65fb - feat(V4): Update matching code to use matchingRules.content for V4 messages (Ronald Holshausen, Mon Mar 8 12:10:31 2021 +1100)
* 4dc5373 - bump version to 0.10.5 (Ronald Holshausen, Wed Feb 10 15:54:50 2021 +1100)

# 0.10.5 - Bugfix Release

* 89240d8 - Merge pull request #95 from pact-foundation/fix/params-missing-on-provider-state-change (Ronald Holshausen, Sun Mar 14 17:20:01 2021 +1100)
* 17682dc - fix: add missing params to provider state change executor (Matt Fellows, Sat Mar 13 08:37:46 2021 +1100)
* 656201c - feat: add exponental deplay the pact broker client retries #94 (Ronald Holshausen, Sun Mar 14 14:16:57 2021 +1100)
* e38634e - feat: add retry to the pact broker client post and put #94 (Ronald Holshausen, Sun Mar 14 14:12:26 2021 +1100)
* 8541751 - feat: add retry to the pact broker client fetch #94 (Ronald Holshausen, Sun Mar 14 13:04:20 2021 +1100)
* 4fe65fb - feat(V4): Update matching code to use matchingRules.content for V4 messages (Ronald Holshausen, Mon Mar 8 12:10:31 2021 +1100)
* 4dc5373 - bump version to 0.10.5 (Ronald Holshausen, Wed Feb 10 15:54:50 2021 +1100)

# 0.10.4 - add final newline to verifier output

* 8c2152e - fix: add final newline to verifier output (Jest will overwrite it with the test name) (Ronald Holshausen, Tue Feb 9 14:15:19 2021 +1100)
* 0a2aad9 - chore: correct release script (Ronald Holshausen, Mon Feb 8 16:14:20 2021 +1100)
* f952467 - bump version to 0.10.4 (Ronald Holshausen, Mon Feb 8 16:04:33 2021 +1100)

# 0.10.3 - Fixes + add callback timeout option for verifcation callbacks

* 49a3cf2 - refactor: use bytes crate instead of vector of bytes for body content (Ronald Holshausen, Sun Feb 7 14:43:40 2021 +1100)
* 4afa86a - fix: add callback timeout option for verifcation callbacks (Ronald Holshausen, Sat Feb 6 12:27:32 2021 +1100)
* 74bd53f - fix: include test results for successful interactions when publishing verification results #92 (Ronald Holshausen, Mon Feb 1 11:24:33 2021 +1100)
* a27ce14 - fix: in callback executors, pass self by value to avoid lifetime issues (Ronald Holshausen, Tue Jan 26 18:41:06 2021 +1100)
* dccd16f - chore: wrap verifier callbacks in Arc<Self> so they can be called across threads (Ronald Holshausen, Tue Jan 26 16:24:09 2021 +1100)
* e5b1f93 - fix: clippy error (Ronald Holshausen, Mon Jan 25 10:26:58 2021 +1100)
* e10047a - bump version to 0.10.3 (Ronald Holshausen, Mon Jan 25 10:20:40 2021 +1100)

# 0.10.2 - made pact broker module public so it can be used by other crates

* c8f7091 - feat: made pact broker module public so it can be used by other crates (Ronald Holshausen, Sun Jan 24 18:24:30 2021 +1100)
* fb4e996 - bump version to 0.10.2 (Ronald Holshausen, Mon Jan 11 10:28:35 2021 +1100)

# 0.10.1 - Updated dependencies

* 1ac3548 - chore: upgrade env_logger to 0.8 (Audun Halland, Sat Jan 9 09:50:27 2021 +0100)
* 9a8a63f - chore: upgrade quickcheck (Audun Halland, Sat Jan 9 08:46:51 2021 +0100)
* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* b79e3a1 - bump version to 0.10.1 (Ronald Holshausen, Tue Jan 5 14:24:47 2021 +1100)

# 0.10.0 - TLS support via FFI + non-blocking verify interaction

* 39c3816 - fix: using `clone` on a double-reference (Ronald Holshausen, Mon Jan 4 17:32:50 2021 +1100)
* 484b747 - fix: verify interaction was blocking the thread (Ronald Holshausen, Mon Jan 4 17:12:38 2021 +1100)
* 4c4eb85 - chore: bump minor version of pact_verifier crate due to breaking changes (Ronald Holshausen, Mon Jan 4 15:48:41 2021 +1100)
* b583540 - Merge branch 'master' into feat/allow-invalid-certs-during-verification (Matt Fellows, Fri Jan 1 14:22:10 2021 +1100)
* 6cec6c7 - feat: allow https scheme and ability to disable ssl verification (Matt Fellows, Thu Dec 31 12:10:57 2020 +1100)
* ed410bd - bump version to 0.9.6 (Ronald Holshausen, Thu Dec 31 15:14:30 2020 +1100)

# 0.9.5 - Supports generators associated with array contains matcher variants

* 144b6aa - chore: upgrade dependencies to latest (Ronald Holshausen, Thu Dec 31 14:58:09 2020 +1100)
* 09513de - feat: add verifiedBy to the verified results (Ronald Holshausen, Tue Dec 29 12:05:07 2020 +1100)
* 12c42c3 - bump version to 0.9.5 (Matt Fellows, Mon Nov 23 07:44:42 2020 +1100)
* 71a5847 - chore: update rust deps (Matt Fellows, Sun Nov 22 23:59:29 2020 +1100)

# 0.9.4 - Bugfix Release

* 52aa549 - chore: improve mismatch output + notices for pacts for verification (Matt Fellows, Sun Nov 22 23:23:15 2020 +1100)
* d481bc1 - fix: pacts for verification unmarshal fails if 'pending' attr is not returned in response (Matt Fellows, Sun Nov 22 22:31:31 2020 +1100)
* 5058a2d - feat: include the mockserver URL and port in the verification context (Ronald Holshausen, Fri Nov 20 16:43:10 2020 +1100)
* a752d6c - bump version to 0.9.4 (Ronald Holshausen, Tue Nov 17 16:58:25 2020 +1100)

# 0.9.3 - Support provider state injected values

* 850282d - fix: times with millisecond precision less 3 caused chronos to panic (Ronald Holshausen, Tue Nov 17 16:29:47 2020 +1100)
* 13ce2f2 - fix: introduce GeneratorTestMode and restrict provider state generator to the provider side (Ronald Holshausen, Mon Nov 16 15:00:01 2020 +1100)

# 0.9.2 - Support Pacts for Verification API

* bbd5364 - test: add negative test case for pacts for verification api (Matt Fellows, Wed Nov 11 08:42:47 2020 +1100)
* b3cca0d - test: add basic pact test for pacts for verification feature (Matt Fellows, Wed Nov 11 00:30:45 2020 +1100)
* e7f729d - wip: further cleanup, and obfuscate auth details (Matt Fellows, Tue Nov 10 13:56:02 2020 +1100)
* ada3667 - wip: cleanup verifier args (Matt Fellows, Tue Nov 10 08:13:01 2020 +1100)
* db0088e - wip: cleanup pacts for verification hal_client clones (Matt Fellows, Mon Nov 9 22:50:51 2020 +1100)
* 80f4e98 - wip: refactor BrokerWithDynamicConfiguration into a struct enum for better readability (Matt Fellows, Mon Nov 9 22:40:24 2020 +1100)
* 93e9161 - wip: working pending pacts with notices (Matt Fellows, Sun Nov 8 14:51:41 2020 +1100)
* 60c1671 - wip: thread verification context into pact fetching/verification, add env vars to clap args (Matt Fellows, Sun Nov 8 13:25:17 2020 +1100)
* 60eb190 - wip: map tags to consumer version selectors (Matt Fellows, Sat Nov 7 23:35:36 2020 +1100)
* 6612a3a - wip: basic wiring in of the pacts for verification endpoint (Matt Fellows, Sat Nov 7 21:39:25 2020 +1100)
* 5e0e470 - chore: bump minor version of pact_consumer crate (Ronald Holshausen, Fri Oct 16 13:22:12 2020 +1100)
* 3a93fd8 - bump version to 0.9.2 (Ronald Holshausen, Fri Oct 16 12:18:50 2020 +1100)

# 0.9.1 - arrayContains matcher + text/xml content type

* 4ef2db6 - Merge branch 'feat/v4-spec' (Ronald Holshausen, Thu Oct 15 17:02:44 2020 +1100)
* 2fb0c6e - fix: fix the build after refactoring the pact write function (Ronald Holshausen, Wed Oct 14 11:07:57 2020 +1100)
* 7fbc731 - chore: bump minor version of matching lib (Ronald Holshausen, Fri Oct 9 10:42:33 2020 +1100)
* 3e943b1 - fix: set content-type header in message request (Marco Dallagiacoma, Thu Oct 1 23:58:14 2020 +0200)
* 29ba743 - feat: add a mock server config struct (Ronald Holshausen, Thu Sep 24 10:30:59 2020 +1000)
* 0b03551 - bump version to 0.9.1 (Ronald Holshausen, Mon Sep 14 17:21:57 2020 +1000)

# 0.9.0 - Verifying Message Pacts

* ef5f88c - chore: bump minor version of the pact_verifier crate (Ronald Holshausen, Mon Sep 14 17:13:45 2020 +1000)
* 865327d - feat: handle comparing content types correctly (Ronald Holshausen, Mon Sep 14 16:37:11 2020 +1000)
* 258cb96 - feat: cleaned up the error display a bit (Ronald Holshausen, Mon Sep 14 16:05:37 2020 +1000)
* ebee1c0 - feat: implemented matching for message metadata (Ronald Holshausen, Mon Sep 14 15:31:18 2020 +1000)
* 6cba6ad - feat: implemented basic message verification with the verifier cli (Ronald Holshausen, Mon Sep 14 13:48:27 2020 +1000)
* 2d44ffd - chore: bump minor version of the matching crate (Ronald Holshausen, Mon Sep 14 12:06:37 2020 +1000)
* fb6c19c - refactor: allow verifier to handle different types of interactions (Ronald Holshausen, Mon Sep 14 10:41:13 2020 +1000)
* 7baf074 - fix: correct clippy error (Ronald Holshausen, Sun Sep 13 18:41:25 2020 +1000)
* 814c416 - refactor: added a trait for interactions, renamed Interaction to RequestResponseInteraction (Ronald Holshausen, Sun Sep 13 17:09:41 2020 +1000)
* a05bcbb - refactor: renamed Pact to RequestResponsePact (Ronald Holshausen, Sun Sep 13 12:45:34 2020 +1000)
* 19290e8 - bump version to 0.8.4 (Ronald Holshausen, Sun Aug 23 16:58:25 2020 +1000)

# 0.8.3 - implemented provider state generator

* b186ce9 - chore: update all dependent crates (Ronald Holshausen, Sun Aug 23 16:49:00 2020 +1000)
* 61ca3d7 - chore: update matching crate to latest (Ronald Holshausen, Sun Aug 23 16:37:58 2020 +1000)
* d5d3679 - feat: return the values from the state change call so they can be used by the generators (Ronald Holshausen, Sun Aug 23 15:40:41 2020 +1000)
* 76f73c6 - feat: implemented provider state generator (Ronald Holshausen, Sun Aug 23 13:29:55 2020 +1000)
* b242eb1 - refactor: changed the remaining uses of the old content type methods (Ronald Holshausen, Sun Jun 28 17:11:51 2020 +1000)
* ed207a7 - chore: updated readmes for docs site (Ronald Holshausen, Sun Jun 28 10:04:09 2020 +1000)
* 8cdcad0 - bump version to 0.8.3 (Ronald Holshausen, Wed Jun 24 11:46:03 2020 +1000)

# 0.8.2 - Updated XML Matching

* 8cf70cc - chore: update to latest matching crate (Ronald Holshausen, Wed Jun 24 11:37:49 2020 +1000)
* a15edea - chore: try set the content type on the body if known (Ronald Holshausen, Tue Jun 23 16:53:32 2020 +1000)
* 875d426 - chore: switch to Rust TLS so we dont have to link to openssl libs (Ronald Holshausen, Sun May 31 09:57:41 2020 +1000)
* df5796f - bump version to 0.8.2 (Ronald Holshausen, Sun May 24 14:02:11 2020 +1000)

# 0.8.1 - Bugfixes + update matching crate to 0.6.0

* bea787c - chore: bump matching crate version to 0.6.0 (Ronald Holshausen, Sat May 23 17:56:04 2020 +1000)
* 61ab50f - fix: date/time matchers fallback to the old key (Ronald Holshausen, Fri May 15 11:27:27 2020 +1000)
* 754a483 - chore: updated itertools to latest (Ronald Holshausen, Wed May 6 15:49:27 2020 +1000)
* 7616ccb - fix: broken tests after handling multiple header values (Ronald Holshausen, Tue May 5 15:45:27 2020 +1000)
* 76250b5 - chore: correct some clippy warnings (Ronald Holshausen, Wed Apr 29 17:53:40 2020 +1000)
* 43de9c3 - chore: update matching library to latest (Ronald Holshausen, Fri Apr 24 10:20:55 2020 +1000)
* c0b67bf - Use err.to_string() rather than format!("{}", err) (Caleb Stepanian, Tue Mar 31 13:27:27 2020 -0400)
* bd10d00 - Avoid deprecated Error::description in favor of Display trait (Caleb Stepanian, Mon Mar 30 16:49:13 2020 -0400)
* c04c0af - bump version to 0.8.1 (Ronald Holshausen, Fri Mar 13 10:06:29 2020 +1100)

# 0.8.0 - Added callback handlers + Bugfixes

* 2920364 - fix: date and time matchers with JSON (Ronald Holshausen, Thu Mar 12 16:07:05 2020 +1100)
* 126b463 - fix: provider state handlers must be synchronous so they are executed for the actual request (Ronald Holshausen, Thu Mar 12 14:16:03 2020 +1100)
* 0e8bfad - fix: allow the HTTP client to be optional in the provider state executor (Ronald Holshausen, Wed Mar 11 14:47:37 2020 +1100)
* 1cf0199 - refactor: moved state change code to a handler (Ronald Holshausen, Wed Mar 11 14:37:07 2020 +1100)
* 70e6648 - chore: converted verifier to use Reqwest (Ronald Holshausen, Mon Mar 9 12:20:14 2020 +1100)
* fe74376 - feat: implemented publishing provider tags with verification results #57 (Ronald Holshausen, Sun Mar 8 18:37:21 2020 +1100)
* b769753 - chore: remove unused import from provider_client (Matt Fellows, Tue Mar 3 12:14:27 2020 +1100)
* c2b7334 - Fixed broken tests using `VerificationOptions`. (Andrew Lilley Brinker, Mon Mar 2 12:16:45 2020 -0800)
* d198d7d - Make `NullRequestFilterExecutor` unconstructable. (Andrew Lilley Brinker, Mon Mar 2 11:59:16 2020 -0800)
* a6e0c16 - Fix RequestFilterExecutor w/ verify_provider (Andrew Lilley Brinker, Mon Mar 2 11:43:59 2020 -0800)
* d944a60 - chore: added callback executors so test code can called during verification (Ronald Holshausen, Sun Feb 23 18:43:49 2020 +1100)
* 639c1fd - bump version to 0.7.1 (Ronald Holshausen, Sun Jan 19 12:03:44 2020 +1100)

# 0.7.0 - Convert to async/await

* 70a33dd - chore: bump minor version of pact_verifier (Ronald Holshausen, Sun Jan 19 11:51:36 2020 +1100)
* 9d3ad57 - chore: bump minor version of pact consumer crate (Ronald Holshausen, Sun Jan 19 11:40:27 2020 +1100)
* cb4c560 - Upgrade tokio to 0.2.9 (Audun Halland, Fri Jan 10 00:13:02 2020 +0100)
* e8034bf - Remove mock server async spawning. (Audun Halland, Thu Jan 9 21:59:56 2020 +0100)
* 9dec41b - Upgrade reqwest to 0.10 (Audun Halland, Tue Dec 31 07:22:36 2019 +0100)
* d24c434 - pact_verifier/pact_broker: Avoid completely unnecessary clones (Audun Halland, Tue Dec 17 02:54:45 2019 +0100)
* cd1046d - pact_verifier: Actually implement HAL client using async reqwest (Audun Halland, Tue Dec 17 01:42:57 2019 +0100)
* d395d2d - pact_verifier: Upgrade reqwest to latest git alpha (Audun Halland, Tue Dec 17 00:57:16 2019 +0100)
* 8019d6d - pact_verifier: Async mock server shutdown (Audun Halland, Thu Dec 12 21:45:16 2019 +0100)
* 3074059 - Refactor ValidatingMockServer into a trait, with two implementations (Audun Halland, Thu Dec 12 15:58:50 2019 +0100)
* fe72f92 - Temporarily solve a problem where a spawned server prevents the test runtime from terminating (Audun Halland, Thu Dec 12 14:14:02 2019 +0100)
* 23a652d - pact_verifier: Implement hyper requests for provider/state change (Audun Halland, Thu Dec 12 11:46:50 2019 +0100)
* 30b1935 - pact_verifier tests: Change to spawned mock server (Audun Halland, Thu Dec 12 11:22:49 2019 +0100)
* bceb44d - pact_verifier: convert pact broker tests to async (Audun Halland, Thu Dec 12 11:04:53 2019 +0100)
* a8866e8 - pact_verifier: Into async/await, part 1 (Audun Halland, Thu Dec 12 10:43:38 2019 +0100)
* 95e46e5 - pact_verifier: Remove extern crate from lib.rs (Audun Halland, Sun Nov 17 23:22:13 2019 +0100)
* 713cd6a - Explicit edition 2018 in Cargo.toml files (Audun Halland, Sat Nov 16 23:55:37 2019 +0100)
* 924452f - 2018 edition autofix "cargo fix --edition" (Audun Halland, Sat Nov 16 22:27:42 2019 +0100)
* d566d23 - bump version to 0.6.2 (Ronald Holshausen, Fri Sep 27 15:17:24 2019 +1000)

# 0.6.1 - Bugfix + Oniguruma crate for regex matching

* 173bf22 - chore: use the matching lib with the Oniguruma crate #46 (Ronald Holshausen, Fri Sep 27 15:02:03 2019 +1000)
* defe890 - fix: switch to the Oniguruma crate for regex matching #46 (Ronald Holshausen, Fri Sep 27 14:35:16 2019 +1000)
* 665bbd8 - fix: return a failure if any pact verification fails #47 (Ronald Holshausen, Fri Sep 27 12:07:01 2019 +1000)
* 48f998d - bump version to 0.6.1 (Ronald Holshausen, Sun Sep 22 17:56:20 2019 +1000)
* 0c5d6c2 - fix: pact_consumer should be a dev dependency (Ronald Holshausen, Sun Sep 22 17:48:35 2019 +1000)

# 0.6.0 - Publishing verification results

* 2e07d77 - chore: set the version of the pact matching crate (Ronald Holshausen, Sun Sep 22 17:24:02 2019 +1000)
* eef3d97 - feat: added some tests for publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 16:44:52 2019 +1000)
* 1110b47 - feat: implemented publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 13:53:27 2019 +1000)
* cb30a2f - feat: added the ProviderStateGenerator as a generator type (Ronald Holshausen, Sun Sep 8 16:29:46 2019 +1000)
* 1e17ca8 - bump version to 0.5.2 (Ronald Holshausen, Sat Aug 24 12:39:55 2019 +1000)

# 0.5.1 - Use reqwest for better HTTP/S support, support headers with multiple values

* f79b033 - chore: update terminal support in release scripts (Ronald Holshausen, Sat Aug 24 12:25:28 2019 +1000)
* b8019ba - chore: bump the version of the matching lib (Ronald Holshausen, Sat Aug 24 12:22:35 2019 +1000)
* dac8ae1 - feat: support authentication when fetching pacts from a pact broker (Ronald Holshausen, Sun Aug 11 13:57:29 2019 +1000)
* e007763 - feat: support bearer tokens when fetching pacts from URLs (Ronald Holshausen, Sun Aug 11 13:21:17 2019 +1000)
* 4378110 - Merge pull request #42 from audunhalland/reqwest (Ronald Holshausen, Sun Aug 11 09:32:30 2019 +1000)
* 75c9b3a - Fix hal+json matching (Audun Halland, Sat Aug 10 14:30:17 2019 +0200)
* f0c0d07 - feat: support headers with multiple values (Ronald Holshausen, Sat Aug 10 17:01:10 2019 +1000)
* 9310f78 - Error messages are a bit different using reqwest: Fix tests (Audun Halland, Mon Jul 29 01:48:03 2019 +0200)
* 58b8c3c - Remove unused import (Audun Halland, Sun Jul 28 18:34:20 2019 +0200)
* 9fd6458 - Print errors using Display trait (Audun Halland, Sun Jul 28 18:33:47 2019 +0200)
* 19f11f7 - Avoid unnecessary clone (Audun Halland, Sun Jul 28 16:39:12 2019 +0200)
* 8717cdd - Fix for json_content_type with charset (Audun Halland, Sun Jul 28 16:17:37 2019 +0200)
* aa1b714 - Switch pact_broker/HAL client to use reqwest instead of hyper directly (Audun Halland, Sun Jul 28 15:48:31 2019 +0200)
* 8b9648c - bump version to 0.5.1 (Ronald Holshausen, Sat Jul 27 17:29:57 2019 +1000)

# 0.5.0 - Upgrade to non-blocking Hyper 0.12

* 89e58cc - chore: update release script (Ronald Holshausen, Sat Jul 27 17:10:06 2019 +1000)
* d842100 - chore: bump component versions to 0.5.0 (Ronald Holshausen, Sat Jul 27 15:44:51 2019 +1000)
* 47ab6d0 - Upgrade tokio to 0.1.22 everywhere (Audun Halland, Mon Jul 22 23:47:09 2019 +0200)
* 4df2797 - Rename API function again (Audun Halland, Mon Jul 22 23:38:11 2019 +0200)
* 7f7dcb0 - Don't expose tokio Runtime inside the libraries (Audun Halland, Mon Jul 22 02:18:52 2019 +0200)
* 16cc6b6 - Run pact_verifier tests in async mode + pact write lock (Audun Halland, Sun May 12 04:05:08 2019 +0200)
* fd1296f - Use Runtime explicitly in tests (Audun Halland, Thu May 2 23:48:50 2019 +0200)
* e2a544c - Fix another warning (Audun Halland, Thu May 2 22:07:10 2019 +0200)
* f831a3f - Fix a couple of warnings (Audun Halland, Thu May 2 22:06:13 2019 +0200)
* ac1c678 - Don't use tokio runtime in provider_client. Only expose futures. (Audun Halland, Thu May 2 21:58:47 2019 +0200)
* 684c292 - Improve provider client errors (Audun Halland, Thu May 2 21:52:37 2019 +0200)
* b5accd6 - Move a function (Audun Halland, Thu May 2 18:32:35 2019 +0200)
* c4d98cb - Fix all tests (Audun Halland, Thu May 2 17:32:31 2019 +0200)
* 4831483 - A join_urls function (Audun Halland, Thu May 2 10:56:46 2019 +0200)
* 1b443a5 - Remove unused test commits (Audun Halland, Thu May 2 08:05:25 2019 +0200)
* 5d8c6fa - Uncomment and compile all tests (Audun Halland, Thu May 2 01:19:28 2019 +0200)
* 2f8a997 - Compile everything (except the commented-out tests) (Audun Halland, Thu May 2 00:41:56 2019 +0200)
* fb3a859 - Temporary fixes; temporarily comment out some tests until code compiles (Audun Halland, Tue Apr 30 12:52:42 2019 +0200)
* f2ae258 - Convert provider_client to async hyper (Audun Halland, Tue Apr 30 02:21:17 2019 +0200)
* 84f4969 - Add tokio Runtime param to pact_verifier lib (Audun Halland, Sat Apr 27 23:58:38 2019 +0200)
* c060f29 - Fix all compile errors in provider_client.rs (Audun Halland, Sat Apr 27 23:50:43 2019 +0200)
* 61c5481 - Work on making the state change async (Audun Halland, Sat Apr 27 22:02:35 2019 +0200)
* 692577b - More work on futures (Audun Halland, Sat Apr 27 21:53:27 2019 +0200)
* a32ec67 - Hyper 0.12: Work in progress (Audun Halland, Sat Apr 27 18:15:50 2019 +0200)
* f8fa0d8 - chore: Bump pact matchig version to 0.5.0 (Ronald Holshausen, Sat Jan 5 19:25:53 2019 +1100)
* 386ab52 - fix: corrected the release scripts to check for a version parameter (Ronald Holshausen, Sun Apr 8 13:44:57 2018 +1000)
* b5e0666 - bump version to 0.4.1 (Ronald Holshausen, Sat Apr 7 15:02:43 2018 +1000)

# 0.4.0 - First V3 specification release

* f63f339 - replaced use of try macro with ? (Ronald Holshausen, Tue Nov 7 16:31:39 2017 +1100)
* c4d424b - Wired in the generated request/response into the mock server and verifier (Ronald Holshausen, Tue Nov 7 16:27:01 2017 +1100)
* 13558d6 - Basic generators working (Ronald Holshausen, Tue Nov 7 10:56:55 2017 +1100)
* 7fef36b - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sat Nov 4 12:49:07 2017 +1100)
* 5c8b79b - Correct the changelog and linux release script (Ronald Holshausen, Fri Nov 3 15:12:39 2017 +1100)
* 9575ee8 - bump version to 0.3.1 (Ronald Holshausen, Fri Nov 3 15:03:20 2017 +1100)
* fbe35d8 - Compiling after merge from v2-spec (Ronald Holshausen, Sun Oct 22 11:39:46 2017 +1100)
* 00dc75a - Bump version to 0.4.0 (Ronald Holshausen, Sun Oct 22 10:46:48 2017 +1100)
* e82ee08 - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Mon Oct 16 09:24:11 2017 +1100)
* 64ff667 - Upgraded the mock server implemenation to use Hyper 0.11.2 (Ronald Holshausen, Wed Sep 6 12:56:47 2017 +1000)
* e5a93f3 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Aug 20 09:53:48 2017 +1000)
* fafb23a - update the verifier to support the new V3 format matchers (Ronald Holshausen, Sun Nov 13 16:49:29 2016 +1100)
* 8765729 - Updated the verifier to handle provider state parameters (Ronald Holshausen, Sun Oct 23 12:10:12 2016 +1100)
* 8797c6c - First successful build after merge from master (Ronald Holshausen, Sun Oct 23 11:59:55 2016 +1100)
* 639ac22 - fixes after merge in from master (Ronald Holshausen, Sun Oct 23 10:45:54 2016 +1100)
* 49e45f7 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Oct 23 10:10:40 2016 +1100)
* 9d286b0 - add rlib crate type back (Ronald Holshausen, Wed Aug 24 21:13:51 2016 +1000)
* 5a7a65e - Merge branch 'master' into v3-spec (Ronald Holshausen, Wed Aug 24 21:02:23 2016 +1000)
* 539eb48 - updated all the readmes and cargo manefests for v3 (Ronald Holshausen, Tue Jul 19 15:46:18 2016 +1000)

# 0.3.0 - Backported matching rules from V3 branch

* 3c09f5b - Update the dependent modules for the verifier (Ronald Holshausen, Fri Nov 3 14:42:09 2017 +1100)
* 8c50392 - update changelog for release 0.3.0 (Ronald Holshausen, Fri Nov 3 14:27:40 2017 +1100)
* 24e3f73 - Converted OptionalBody::Present to take a Vec<u8> #19 (Ronald Holshausen, Sun Oct 22 18:04:46 2017 +1100)
* d990729 - Some code cleanup #20 (Ronald Holshausen, Wed Oct 18 16:32:37 2017 +1100)
* c983c63 - Bump versions to 0.3.0 (Ronald Holshausen, Wed Oct 18 13:54:46 2017 +1100)
* da9cfda - Implement new, experimental syntax (API BREAKAGE) (Eric Kidd, Sun Oct 8 13:33:09 2017 -0400)
* 06e92e5 - Refer to local libs using version+paths (Eric Kidd, Tue Oct 3 06:22:23 2017 -0400)
* 7afd258 - Update all the cargo manifest versions and commit the cargo lock files (Ronald Holshausen, Wed May 17 10:37:44 2017 +1000)
* 665aea1 - make release script executable (Ronald Holshausen, Wed May 17 10:30:31 2017 +1000)
* 17d6e98 - bump version to 0.2.2 (Ronald Holshausen, Wed May 17 10:23:34 2017 +1000)


# 0.2.1 - Replace rustc_serialize with serde_json

* a1f78f9 - Move linux specific bits out of the release script (Ronald Holshausen, Wed May 17 10:18:37 2017 +1000)
* efe4ca7 - Cleanup unused imports and unreachable pattern warning messages (Anthony Damtsis, Tue May 16 10:31:29 2017 +1000)
* be8c299 - Cleanup unused BTreeMap usages and use remote pact dependencies (Anthony Damtsis, Mon May 15 17:09:14 2017 +1000)
* a59fb98 - Migrate remaining pact modules over to serde (Anthony Damtsis, Mon May 15 16:59:04 2017 +1000)
* 3ca29d6 - bump version to 0.2.1 (Ronald Holshausen, Sun Oct 9 17:06:35 2016 +1100)

# 0.2.0 - V2 specification implementation

* 91f5315 - update the references to the spec in the verifier library to V2 (Ronald Holshausen, Sun Oct 9 16:59:45 2016 +1100)
* e2f88b8 - update the verifier library to use the published consumer library (Ronald Holshausen, Sun Oct 9 16:57:34 2016 +1100)
* 770010a - update projects to use the published pact matching lib (Ronald Holshausen, Sun Oct 9 16:25:15 2016 +1100)
* 574e072 - upadte versions for V2 branch and fix an issue with loading JSON bodies encoded as a string (Ronald Holshausen, Sun Oct 9 15:31:57 2016 +1100)
* dabe425 - bump version to 0.1.1 (Ronald Holshausen, Sun Oct 9 10:40:39 2016 +1100)

# 0.1.0 - V1.1 specification implementation

* 7b66941 - Update the deps for pact verifier library (Ronald Holshausen, Sun Oct 9 10:32:47 2016 +1100)
* 1f3f3f1 - correct the versions of the inter-dependent projects as they were causing the build to fail (Ronald Holshausen, Sat Oct 8 17:41:57 2016 +1100)
* a46dabb - update all references to V1 spec after merge (Ronald Holshausen, Sat Oct 8 16:20:51 2016 +1100)
* 1246784 - correct the verifier library release script (Ronald Holshausen, Tue Sep 27 20:57:13 2016 +1000)
* f0ce08a - bump version to 0.0.1 (Ronald Holshausen, Tue Sep 27 20:43:34 2016 +1000)

# 0.0.0 - First Release
