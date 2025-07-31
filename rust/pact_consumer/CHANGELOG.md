To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 1.4.2 - Maintenance Release

* a77b53c8 - chore: Update pact_matching to 2.0.1 (Ronald Holshausen, Fri Aug 1 09:16:28 2025 +1000)
* 52f5e243 - Chore: Update pact_mock_server to 2.1.3 (Ronald Holshausen, Thu Jul 31 15:43:54 2025 +1000)
* c480357e - Chore: Update pact_matching to 2.0.0 (Ronald Holshausen, Thu Jul 31 15:40:59 2025 +1000)
* e2e69295 - refactor(v2-matching-engine): Merged matchers.rs and matchingrules.rs; deprecated Matches replaced with DoMatch trait (Ronald Holshausen, Thu Jul 31 13:18:27 2025 +1000)
* ff964960 - chore: Update pact_models to 1.3.6 (Ronald Holshausen, Wed Jul 30 09:50:09 2025 +1000)
* 1cb9a5ac - chore: Update pact_models to 1.3.5 (Ronald Holshausen, Mon Jul 28 10:44:30 2025 +1000)
* 77c61e3c - fix(pact_consumer): Upgrade pact_mock_server to 2.1.2 to fix mock server URL in the test context (Ronald Holshausen, Thu Jun 12 11:26:11 2025 +1000)
* 5371ee6f - chore(pact_verifier): Update pact_consumer to 1.4.1 (Ronald Holshausen, Wed Jun 11 16:29:07 2025 +1000)
* 6c009917 - bump version to 1.4.2 (Ronald Holshausen, Wed Jun 11 16:25:26 2025 +1000)

# 1.4.1 - Maintenance Release

* 76530c4a - chore(pact_consumer): Update dependencies (Ronald Holshausen, Wed Jun 11 16:22:05 2025 +1000)
* 8dd1502a - chore(pact_consumer): Update pact_mock_server to 2.1.1 (Ronald Holshausen, Wed Jun 11 16:16:06 2025 +1000)
* 3f883a8d - fix(Alpine): Update pact_matching to 2.0.0-beta.3 (Ronald Holshausen, Wed Jun 11 13:46:00 2025 +1000)
* 6e8167c4 - fix(Alpine): Updae pact_models to 1.3.3 (Ronald Holshausen, Wed Jun 11 13:23:10 2025 +1000)
* 588f1fe2 - chore: Update pact_matching to 2.0.0-beta.2 (Ronald Holshausen, Wed Jun 11 11:24:09 2025 +1000)
* 18e53fbd - chore: Update pact_models to 1.3.2 (Ronald Holshausen, Wed Jun 11 10:16:03 2025 +1000)
* 372ee77c - chore: correct release script (Ronald Holshausen, Wed May 21 14:55:20 2025 +1000)
* 720bd04a - bump version to 1.4.1 (Ronald Holshausen, Wed May 21 14:54:53 2025 +1000)

# 1.4.0 - Update to Rust 2024 and pact_matching 2.0.0-beta

* 60afbe95 - chore(pact_consumer): Update crate to Rust 2024 edition (Ronald Holshausen, Wed May 21 14:49:18 2025 +1000)
* 71b1b7ed - chore(pact_consumer): Update pact_mock_server to 2.1.0 (Ronald Holshausen, Wed May 21 14:44:21 2025 +1000)
* 0649f61f - chore: Update pact_matching to 2.0.0-beta.1 (Ronald Holshausen, Wed May 21 12:04:10 2025 +1000)
* 7d8e8f61 - chore: Upgrade pact_models to 1.3.1 (Ronald Holshausen, Tue May 20 14:53:42 2025 +1000)
* 369dba85 - chore: Upgrade pact-mock-server to 2.0.3/1.2.12 (Ronald Holshausen, Wed Mar 26 11:00:31 2025 +1100)
* 12ca5283 - chore: Upgrade pact-plugin-driver to 0.7.4 (Ronald Holshausen, Tue Mar 25 16:26:48 2025 +1100)
* 0ed75202 - chore(pact-models): Upgrade pact models crate to Rust 2024 edition (Ronald Holshausen, Tue Mar 25 16:02:47 2025 +1100)
* b07d7aec - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Wed Mar 19 11:22:43 2025 +1100)
* e32f0dbd - fix: Upgrade pact-plugin-driver to 0.7.3 (Ronald Holshausen, Wed Mar 19 11:20:39 2025 +1100)
* ad9b555b - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Fri Mar 14 11:42:13 2025 +1100)
* d67db1a4 - chore: Update pact_models to 1.2.7 (Ronald Holshausen, Fri Mar 14 11:02:53 2025 +1100)
* c57885d1 - feat: changes required to get spec tests passing (Ronald Holshausen, Tue Mar 11 11:27:19 2025 +1100)
* c8f165b7 - fix: correct the build after last change (Ronald Holshausen, Thu Mar 6 09:52:03 2025 +1100)
* ad9ee539 - fix: Matching rule definitions now take th expression they were parsed from (Ronald Holshausen, Tue Mar 4 09:18:51 2025 +1100)
* 7b94a211 - Merge tag 'master' into feat/matching-engine-v2 (Ronald Holshausen, Thu Feb 20 15:16:16 2025 +1100)
* 8fb0bcb9 - chore: Upgrade pact_matching to 1.2.9 (Ronald Holshausen, Fri Jan 31 09:44:14 2025 +1100)
* 648ab1c9 - chore: Upgrade pact_matching to 1.2.8 (Ronald Holshausen, Thu Dec 19 16:14:10 2024 +1100)
* a7303e61 - bump version to 1.3.4 (Ronald Holshausen, Fri Dec 13 14:05:32 2024 +1100)
* 5db8db23 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Wed Nov 13 14:16:31 2024 +1100)
* 11cd9dc0 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Fri Oct 25 10:01:15 2024 +1100)
* 02e2ae13 - chore: Bump pact_matching to 2.0.0-beta.0 (Ronald Holshausen, Tue Jul 30 11:40:20 2024 +1000)

# 1.3.3 - Maintenance Release

* 6189c0cd - fix(pact_consumer): Pass the transport config to the plugin in the test context under the transport_config key (Ronald Holshausen, Fri Dec 13 12:00:05 2024 +1100)
* 5fdfd023 - chore: Upgrade all the tracing crates to the latest (Ronald Holshausen, Fri Dec 13 10:54:51 2024 +1100)
* fab32aa0 - chore: Upgrade pact_models to 1.2.6 (Ronald Holshausen, Wed Dec 11 15:28:01 2024 +1100)
* 93ce0e95 - bump version to 1.3.3 (Ronald Holshausen, Fri Nov 29 16:46:10 2024 +1100)

# 1.3.2 - Bugfix Release

* 44d9dc1e - fix(plugins): Upgrade pact-plugin-driver to 0.7.2, fixes #473 (Ronald Holshausen, Fri Nov 29 15:48:40 2024 +1100)
* f3918b61 - Merge pull request #464 from tienvx/fix-cant-compare-each-key-each-value-matching-rules (Ronald Holshausen, Wed Nov 13 11:07:10 2024 +1100)
* 420c654d - chore: Upgrade pact_models to 1.2.5 (Ronald Holshausen, Wed Nov 13 11:01:39 2024 +1100)
* c2793764 - test: Add test for eachKey and eachValue matching rules comparation (tien.xuan.vo, Fri Sep 6 11:40:19 2024 +0700)
* 4604729d - chore(pact_consumer): Update dependencies (Ronald Holshausen, Wed Sep 4 11:01:33 2024 +1000)
* cbd167b6 - chore(pact_consumer): Upgrade pact_mock_server to 2.0.2 (Ronald Holshausen, Wed Sep 4 10:59:27 2024 +1000)
* c1a00dd8 - chore: Upgrade pact-plugin-driver to 0.7.1 (Ronald Holshausen, Tue Sep 3 15:47:22 2024 +1000)
* 73a19100 - bump version to 1.3.2 (Ronald Holshausen, Tue Aug 27 15:40:56 2024 +1000)

# 1.3.1 - Bugfix Release

* 941e4455 - fix(pact_consumer): Plugin config needs to be stored for both the request and response part of HTTP interactions (Ronald Holshausen, Tue Aug 27 15:35:16 2024 +1000)
* dae427a7 - chore: Upgrade pact_mock_server to 2.0.1 (Ronald Holshausen, Tue Aug 27 15:23:18 2024 +1000)
* dc441709 - chore: Upgrade pact_matching to 1.2.7 (Ronald Holshausen, Tue Aug 27 14:57:12 2024 +1000)
* 978677cc - chore: Upgrade pact_matching to 1.2.6 (Ronald Holshausen, Tue Aug 13 15:27:49 2024 +1000)
* 9a7eef75 - chore: Upgrade pact_models to 1.2.4 (Ronald Holshausen, Tue Aug 13 11:51:06 2024 +1000)
* bcf4db6d - chore: Upgrade pact_models to 1.2.3 (Ronald Holshausen, Tue Aug 6 11:06:22 2024 +1000)
* 00efcb56 - chore: Update readme (Ronald Holshausen, Mon Jul 29 11:01:32 2024 +1000)
* c91bb0ca - bump version to 1.3.1 (Ronald Holshausen, Mon Jul 29 10:27:31 2024 +1000)

# 1.3.0 - Update to 2.0 Pact mock server

* d46af23e - feat(pact-consumer): Give each mock server 2 worker threads (Ronald Holshausen, Mon Jul 29 10:03:38 2024 +1000)
* 111a18cd - Feat(pact_consumer): Upgrade pact_mock_server to the new 2.0.0 version (Ronald Holshausen, Thu Jul 18 17:12:55 2024 +1000)
* 3665a921 - bump version to 1.2.4 (Ronald Holshausen, Wed Jul 17 15:25:27 2024 +1000)

# 1.2.3 - Maintenance Release

* 9659c982 - chore(pact_consumer): Update dependencies (Ronald Holshausen, Wed Jul 17 15:20:24 2024 +1000)
* 3c96c31b - chore: Upgrade pact_mock_server to 1.2.9 (Ronald Holshausen, Wed Jul 17 14:54:40 2024 +1000)
* bb9fc1be - chore: Upgrade pact_matching to 1.2.5 (Ronald Holshausen, Wed Jul 17 14:34:04 2024 +1000)
* fd9ff70a - chore: Upgrade pact_models to 1.2.2 (Ronald Holshausen, Wed Jul 17 11:15:34 2024 +1000)
* e8b62f98 - chore: Upgrade pact-plugin-driver to 0.7.0 (Ronald Holshausen, Wed Jul 17 10:45:34 2024 +1000)
* 57c0bc85 - feat(pact_consumer): Improve output of mock server errors with ANSI colour (Ronald Holshausen, Thu Jul 11 10:34:14 2024 +1000)
* dee42806 - fix(pact_consumer): Fix for attempt to subtract with overflow error (Ronald Holshausen, Wed Jul 10 15:45:23 2024 +1000)
* 2e2580f6 - feat(pact_consumer): Improve the mock server error output (Ronald Holshausen, Wed Jul 10 15:23:48 2024 +1000)
* c70c1072 - chore(pact_consumer): Update dependencies (Ronald Holshausen, Fri Jun 14 10:35:55 2024 +1000)
* 55f584bf - bump version to 1.2.3 (Ronald Holshausen, Fri Jun 14 10:30:00 2024 +1000)

# 1.2.2 - Maintenance Release

* 29c3b5f0 - chore: Upgrade pact_matching to 1.2.4 (Ronald Holshausen, Fri Jun 14 10:18:26 2024 +1000)
* 30998a95 - chore: Upgrade pact_models to 1.2.1 (Ronald Holshausen, Wed Jun 12 15:26:49 2024 +1000)
* 5699cab7 - chore: Upgrade pact_mock_server to 1.2.8 (Ronald Holshausen, Thu May 30 16:53:17 2024 +1000)
* a811b8a4 - chore: Update pact_matching to 1.2.3 (Ronald Holshausen, Thu May 30 15:57:32 2024 +1000)
* 4f01a302 - chore: Fix project so build works (Ronald Holshausen, Thu May 30 12:01:44 2024 +1000)
* 968be075 - feat: update content_type_matching logic for binary payloads (Yousaf Nabi, Tue May 28 23:25:07 2024 +0100)
* 87e82e1b - fix(rust_consumer): correct term! documentation (Martijn Pieters, Mon May 20 15:35:51 2024 +0100)
* d7d69964 - bump version to 1.2.2 (Ronald Holshausen, Wed May 8 11:42:08 2024 +1000)

# 1.2.1 - Maintenance Release

* f5ad729f - chore: Moved mock server crates to pact-foundation/pact-core-mock-server (Ronald Holshausen, Tue May 7 17:36:26 2024 +1000)
* fa2b1d09 - chore: Upgrade pact_matching to 1.2.2 (Ronald Holshausen, Tue May 7 10:50:08 2024 +1000)
* 694100fb - chore: Update pact_models to 1.2.0 (Ronald Holshausen, Tue Apr 23 10:51:11 2024 +1000)
* edfac7ba - chore: remove local pact_models from the other pact crates (Ronald Holshausen, Tue Apr 23 10:03:29 2024 +1000)
* c3128a6d - feat: Support optional query parameter values (where there is only a name) (Ronald Holshausen, Mon Apr 22 10:36:05 2024 +1000)
* 758f4c03 - chore: Update pact_matching to 1.2.1 (Ronald Holshausen, Tue Apr 16 16:29:38 2024 +1000)
* 831995b9 - bump version to 1.2.1 (Ronald Holshausen, Tue Apr 16 15:18:59 2024 +1000)

# 1.2.0 - Maintenance Release

* bfd6eae0 - chore: Upgrade pact_mock_server to 1.2.6 (Ronald Holshausen, Tue Apr 16 15:14:00 2024 +1000)
* b9c54975 - Revert "update changelog for release 1.2.0" (Ronald Holshausen, Tue Apr 16 14:39:01 2024 +1000)
* 5cc668a1 - update changelog for release 1.2.0 (Ronald Holshausen, Tue Apr 16 14:37:22 2024 +1000)
* 6234fe35 - chore(pact_consumer): Upgrade pact-plugin-driver to 0.6.0 (Ronald Holshausen, Tue Apr 16 11:58:35 2024 +1000)
* de579415 - chore(pact_consumer): Upgrade dependencies (Ronald Holshausen, Tue Apr 16 11:54:47 2024 +1000)
* dffbc8e7 - chore(pact_consumer): Bump minor version (Ronald Holshausen, Tue Apr 16 11:49:14 2024 +1000)
* d6125b75 - chore(pact_matching): Bump minor version (Ronald Holshausen, Tue Apr 16 10:16:44 2024 +1000)
* ffffa61d - chore: Upgrade pact_mock_server to 1.2.5 (Ronald Holshausen, Fri Mar 15 15:13:04 2024 +1100)
* fa2af940 - bump version to 1.1.3 (Ronald Holshausen, Mon Feb 12 14:49:15 2024 +1100)

# 1.1.2 - Bugfix Release

* e31589d7 - chore: add test for checking matching rules on message metadata (Ronald Holshausen, Sat Feb 10 10:13:01 2024 +1100)
* 6bb57df0 - fix: metadata matching rules for sync message (Stan, Fri Feb 9 00:16:51 2024 -0800)
* a52e0ee9 - chore: Upgrade pact_matching to 1.1.10 (Ronald Holshausen, Wed Feb 7 13:20:45 2024 +1100)
* 24a26cca - chore: Update pact_models to 1.1.18 (Ronald Holshausen, Wed Feb 7 10:53:22 2024 +1100)
* 73578350 - chore: use local pact_models (JP-Ellis, Tue Feb 6 10:51:09 2024 +1100)
* 95cbe5a9 - fix: Upgrade pact-plugin-driver to 0.5.1 (Ronald Holshausen, Wed Jan 31 19:56:04 2024 +1100)
* 5f5ad750 - bump version to 1.1.2 (Ronald Holshausen, Sat Jan 20 22:00:01 2024 +1100)

# 1.1.1 - Maintenace Release

* c68e00a5 - chore: Upgrade dependencies (Ronald Holshausen, Sat Jan 20 21:53:03 2024 +1100)
* def5685c - chore: Upgrade pact_mock_server to 1.2.4 (Ronald Holshausen, Sat Jan 20 18:56:26 2024 +1100)
* e552bdce - chore: Upgrade pact_matching to 1.1.9 (Ronald Holshausen, Sat Jan 20 15:13:13 2024 +1100)
* 7b087acf - chore: Upgrade pact-plugin-driver to 0.5.0 (Ronald Holshausen, Sat Jan 20 14:49:21 2024 +1100)
* b735df9d - chore: Upgrade pact_models to 1.1.17 (Ronald Holshausen, Sat Jan 20 13:54:03 2024 +1100)
* 1a4bcd27 - chore: Upgrade pact_matching to 1.1.8 (Ronald Holshausen, Fri Jan 19 18:24:54 2024 +1100)
* 944613df - fix: regression - upgrade pact_models to 1.1.16 #359 (Ronald Holshausen, Fri Jan 19 14:52:36 2024 +1100)
* 403c0af1 - chore: Upgrade pact_models to 1.1.14 #355 (Ronald Holshausen, Tue Jan 16 10:31:12 2024 +1100)
* dfd13760 - chore: Upgrade pact_models to 1.1.13 #355 (Ronald Holshausen, Tue Jan 16 07:42:33 2024 +1100)
* 41618d94 - bump version to 1.1.1 (Ronald Holshausen, Wed Dec 20 14:32:05 2023 +1100)

# 1.1.0 - Maintenance Release

* 302a5e88 - chore(pact_consumer): Cleanup TODO (Ronald Holshausen, Fri Dec 15 16:26:44 2023 +1100)
* 52dfe7d5 - chore(pact_consumer): Cleanup TODO (Ronald Holshausen, Fri Dec 15 16:10:45 2023 +1100)
* 691f4422 - chore(pact_consumer): Bump minor version (Ronald Holshausen, Fri Dec 15 15:45:37 2023 +1100)
* 713e4098 - chore: Upgrade pact-plugin-driver to 0.4.6 (Ronald Holshausen, Thu Dec 14 17:04:59 2023 +1100)
* e12536a3 - bump version to 1.0.6 (Ronald Holshausen, Tue Nov 14 03:23:24 2023 +1100)

# 1.0.5 - Maintenance Release

* f8110971 - chore: cleanup compiler warning (Ronald Holshausen, Tue Nov 14 03:18:42 2023 +1100)
* 3f0ae7f1 - chore: Upgrade pact_matching to 1.1.7 (Ronald Holshausen, Tue Nov 14 03:10:25 2023 +1100)
* 826758a6 - chore: Upgrade pact_models to 1.1.12 (Ronald Holshausen, Mon Nov 13 17:25:21 2023 +1100)
* 99462514 - chore: fix compilation issue compiling Rust without default features #326 (Ronald Holshausen, Fri Oct 6 09:27:59 2023 +1100)
* 04bad264 - chore: Upgrade pact_matching to 1.1.6 (Ronald Holshausen, Fri Sep 22 11:03:38 2023 +1000)
* 88dd9d61 - bump version to 1.0.5 (Ronald Holshausen, Tue Aug 29 14:35:03 2023 +1000)

# 1.0.4 - Bugfix Release

* d592cd8b - chore: Upgrade pact_mock_server to 1.2.3 (Ronald Holshausen, Tue Aug 29 09:25:14 2023 +1000)
* 3ec99c41 - chore: Upgrade pact_matching to 1.1.5 (Ronald Holshausen, Fri Aug 18 15:40:02 2023 +1000)
* db59c865 - fix: test test_req_res_message_client was failing on Windows with a channel error (Ronald Holshausen, Fri Aug 18 11:17:15 2023 +1000)
* 3438d4e5 - feat: Add Synchronous Messages feature (Ronald Holshausen, Fri Aug 18 09:59:21 2023 +1000)
* 92fc950e - feat(compatibility-suite): Add V4 message scenarios (Ronald Holshausen, Wed Aug 16 14:09:49 2023 +1000)
* ea98f458 - feat(compatibility-suite): Implemented V4 HTTP scenarios (Ronald Holshausen, Tue Aug 15 15:09:24 2023 +1000)
* e4da3e42 - chore: Upgrade pact_models to 1.1.11 (Ronald Holshausen, Mon Aug 7 13:59:34 2023 +1000)
* 24ed7835 - chore: Upgrade pact-models to 1.1.10 (Ronald Holshausen, Fri Aug 4 16:11:24 2023 +1000)
* ad36106a - feat(pact-consumer): Message DSL was missing support for V3 messages and provider state parameters (Ronald Holshausen, Wed Aug 2 16:45:21 2023 +1000)
* 0586fcf1 - chore: Upgrade pact_mock_server to 1.2.2 (Ronald Holshausen, Thu Jul 27 14:45:02 2023 +1000)
* 8f88192e - chore: Upgrade pact_matching to 1.1.4 (Ronald Holshausen, Thu Jul 27 14:35:27 2023 +1000)
* 4a01919a - chore: Upgrade pact_models to 1.1.9 (Ronald Holshausen, Thu Jul 27 10:24:00 2023 +1000)
* 9f31eb88 - feat(compatibility-suite): Added remaining V3 features; Got generator steps passing (Ronald Holshausen, Tue Jul 25 15:35:39 2023 +1000)
* 71fba00c - feat(pact_comsumer): Added support for using EachKey and EachValue matchers (Ronald Holshausen, Mon Jul 17 11:33:08 2023 +1000)
* a239f786 - bump version to 1.0.4 (Ronald Holshausen, Thu Jul 13 09:28:32 2023 +1000)

# 1.0.3 - Bugfix Release

* e08119ab - fix(pact-consumer): Some code for the datetime feature was not being excluded when the feature was turned off #290 (Ronald Holshausen, Thu Jul 13 09:22:24 2023 +1000)
* 4f448b1f - fix(pact_matching): EachValue matcher was not applying the associated matching rules correctly #299 (Ronald Holshausen, Wed Jul 12 16:28:16 2023 +1000)
* 7444b6af - bump version to 1.0.3 (Ronald Holshausen, Wed Jul 12 10:04:09 2023 +1000)

# 1.0.2 - Bugfixes + added crate features to disable functionality

* 6661077a - feat: Add support for disabling plugins with consumer DSL (Ronald Holshausen, Tue Jul 11 16:53:12 2023 +1000)
* b3c65285 - chore: Add support for datetime, xml, multipart. plugins and tls crate features (Ronald Holshausen, Tue Jul 11 15:39:05 2023 +1000)
* e07ca36c - chore: Upgrade pact_mock_server to 1.2.1 (Ronald Holshausen, Tue Jul 11 14:47:17 2023 +1000)
* 348eb3f3 - chore: Upgrade pact_matcing to 1.1.3 (Ronald Holshausen, Tue Jul 11 11:38:26 2023 +1000)
* f2ae77ba - chore: Upgrade pact-plugin-driver to 0.4.5 (Ronald Holshausen, Mon Jul 10 17:15:20 2023 +1000)
* b18b9dff - chore: Upgrade pact_matching to 1.1.2 (Ronald Holshausen, Mon Jul 10 16:42:27 2023 +1000)
* 1deca59a - chore: Upgrade pact_models to 1.1.8 (Ronald Holshausen, Mon Jul 10 16:15:43 2023 +1000)
* 5c725214 - feat(consumer DSL): Add methods to support using a regex with a text body #291 (Ronald Holshausen, Mon Jul 10 15:38:42 2023 +1000)
* 73033f54 - fix(consumer): Setup multi-value headers correctly #300 (Ronald Holshausen, Mon Jul 10 14:59:22 2023 +1000)
* 2662cdfc - chore: Upgrade pact_models to 1.1.7 (Ronald Holshausen, Thu Jul 6 10:27:25 2023 +1000)
* 445ea1ee - fix: Header matching rules should be looked up in a case-insenstive way (Ronald Holshausen, Wed Jun 28 15:21:32 2023 +1000)
* e95ae4d0 - chore: Upgrade pact_models to 1.1.6 (Ronald Holshausen, Thu Jun 22 15:40:55 2023 +1000)
* a8faee63 - chore: Update example in docs comments to show starting the mock server #294 (Ronald Holshausen, Thu Jun 22 09:45:49 2023 +1000)
* 244f1fdb - feat(compatibility-suite): Implemented scenarios for no provider state callback configured + request filters (Ronald Holshausen, Fri Jun 16 11:36:30 2023 +1000)
* 834f77cc - chore: Upgrade pact_mock_server to 1.2.0 (Ronald Holshausen, Wed Jun 14 15:22:11 2023 +1000)
* e58aa917 - fix: no need to wrap the Pact for a mock server in a mutex (mock server is already behind a mutex) as this can cause deadlocks #274 (Ronald Holshausen, Wed Jun 14 13:26:54 2023 +1000)
* cd536170 - feat: Add builder interface for plugins to provide DSL to construct interactions (Ronald Holshausen, Wed Jun 7 15:12:15 2023 +1000)
* 8287f375 - bump version to 1.0.2 (Ronald Holshausen, Wed Jun 7 11:51:33 2023 +1000)

# 1.0.1 - Bugfix Release

* 4fbd45cc - fix: Mock server errors were not being printed if the test was panicking due to a failed assertion #282 (Ronald Holshausen, Wed Jun 7 11:27:15 2023 +1000)
* bc68ed7f - chore: Upgrade pact_models to 1.1.4 (Ronald Holshausen, Thu Jun 1 10:22:38 2023 +1000)
* 397c837f - chore: Upgrade pact_models to 1.1.3 (fixes MockServerURL generator) (Ronald Holshausen, Mon May 29 15:12:22 2023 +1000)
* cbcb530e - bump version to 1.0.1 (Ronald Holshausen, Tue May 23 16:13:29 2023 +1000)

# 1.0.0 - Bugfixes + Update Pact models to 1.1 (breaking change)

* e9bb4c69 - chore: fix deps (Ronald Holshausen, Tue May 23 16:10:46 2023 +1000)
* 90cd909d - Revert "update changelog for release 1.0.0" (Ronald Holshausen, Tue May 23 16:10:02 2023 +1000)
* c3b91a52 - update changelog for release 1.0.0 (Ronald Holshausen, Tue May 23 16:09:10 2023 +1000)
* c2d925e9 - chore: Upgrade pact_consumer to 1.0.0 (Ronald Holshausen, Tue May 23 16:05:12 2023 +1000)
* 6aac93bb - feat: Update message builders to support adding message metadata (Ronald Holshausen, Tue May 23 16:00:19 2023 +1000)
* 8e9bd503 - chore: Upgrade pact_mock_server to 1.1.0 (Ronald Holshausen, Tue May 23 12:20:01 2023 +1000)
* 8f27f9bd - chore: Upgrade pact-plugin-driver to 0.4.4 (Ronald Holshausen, Tue May 23 11:55:23 2023 +1000)
* ac2e24da - chore: Use "Minimum version, with restricted compatibility range" for all Pact crate versions (Ronald Holshausen, Tue May 23 11:46:52 2023 +1000)
* 6df4670c - chore: Upgrade pact_matching to 1.1.1 (Ronald Holshausen, Tue May 23 11:32:51 2023 +1000)
* 54887690 - chore: Bump pact_matching to 1.1 (Ronald Holshausen, Tue May 23 11:13:14 2023 +1000)
* 261ecf47 - fix: Add RefUnwindSafe trait bound to all Pact and Interaction uses (Ronald Holshausen, Mon May 15 13:59:31 2023 +1000)
* 470b1a25 - bump version to 0.10.9 (Ronald Holshausen, Tue Apr 18 16:39:36 2023 +1000)

# 0.10.8 - Bugfix Release

* 6a71b12d - chore: Upgrade pact_mock_server to 1.0.2 (Ronald Holshausen, Tue Apr 18 13:30:21 2023 +1000)
* 0bcba082 - chore: Upgrade pact_matching to 1.0.8 (Ronald Holshausen, Tue Apr 18 13:14:38 2023 +1000)
* 6c14abfd - chore: Upgrade pact_models to 1.0.13 (Ronald Holshausen, Tue Apr 18 13:00:01 2023 +1000)
* ce16d43f - chore: Upgrade pact-plugin-driver to 0.4.2 (supports auto-installing known plugins) (Ronald Holshausen, Tue Apr 18 11:49:52 2023 +1000)
* 10bf1a48 - chore: Upgrade pact_models to 1.0.12 (fixes generators hash function) (Ronald Holshausen, Mon Apr 17 10:31:09 2023 +1000)
* b62d58ee - chore: add tests to validate the V4 generated key (Ronald Holshausen, Fri Apr 14 17:15:25 2023 +1000)
* 84b9d9e9 - fix: Upgrade pact models to 1.0.11 (fixes generated key for V4 Pacts) (Ronald Holshausen, Fri Apr 14 17:10:58 2023 +1000)
* 669f7812 - chore: Upgrade pact_models to 1.0.10 (Ronald Holshausen, Thu Apr 13 15:32:34 2023 +1000)
* 9da17e24 - bump version to 0.10.8 (Ronald Holshausen, Wed Apr 5 17:03:39 2023 +1000)

# 0.10.7 - Bugfix Release

* 779a59f0 - fix: Upgrade pact-plugin-driver to 0.4.1 (fixes an issue introduced in 0.4.0 with shared channels to plugins) (Ronald Holshausen, Wed Apr 5 17:01:18 2023 +1000)
* ddcb3ded - bump version to 0.10.7 (Ronald Holshausen, Wed Apr 5 14:49:04 2023 +1000)

# 0.10.6 - Bugfix Release

* 7bd44a0d - fix: PactBuilder drop handler was cauing plugins to be shutdown twice (Ronald Holshausen, Wed Apr 5 14:44:26 2023 +1000)
* 6aa389c9 - fix: Make using_plugin consume self so that the builder will have the same lifetime as the returned async one (Ronald Holshausen, Wed Apr 5 14:43:43 2023 +1000)
* 48284036 - chore: Upgrade dependencies (Ronald Holshausen, Wed Apr 5 14:41:33 2023 +1000)
* 81a9b306 - chore: Upgrade pact_mock_server to 1.0.1 (Ronald Holshausen, Tue Apr 4 15:40:20 2023 +1000)
* 126cf462 - chore: Upgrade pact_matching to 1.0.7 (Ronald Holshausen, Tue Apr 4 15:12:28 2023 +1000)
* 6f0c4b2f - feat: Upgrade pact-plugin-driver to 0.4.0 which uses a shared gRPC channel to each plugin (Ronald Holshausen, Tue Apr 4 14:32:36 2023 +1000)
* 63be53b2 - fix: allow the pact builders to set the overwrite flag (Ronald Holshausen, Mon Apr 3 14:53:36 2023 +1000)
* f8aea4fc - fix: request and response builders were incorrectly setting empty bodies from plugin contents (Ronald Holshausen, Mon Apr 3 14:52:29 2023 +1000)
* a31bfa41 - bump version to 0.10.6 (Ronald Holshausen, Tue Mar 21 10:23:17 2023 +1100)

# 0.10.5 - Maintenance Release

* 11c701b4 - fix: Upgrade pact_matching to 1.0.6 (fixes some issues with matching HTTP headers) (Ronald Holshausen, Wed Mar 15 14:54:54 2023 +1100)
* e96bc54e - fix: Upgrade pact_models to 1.0.9 (fixes issues with headers) (Ronald Holshausen, Wed Mar 15 14:31:00 2023 +1100)
* f7e0b669 - chore: Upgrade pact_models to 1.0.8 (Ronald Holshausen, Wed Mar 15 12:19:22 2023 +1100)
* 57728a01 - chore: update pact-plugin-driver to 0.3.3 (Ronald Holshausen, Tue Mar 14 17:19:20 2023 +1100)
* 0676047e - chore: Upgrade pact-plugin-driver to 0.3.2 (Ronald Holshausen, Thu Feb 16 12:09:46 2023 +1100)
* 7589b9b0 - chore: Bump pact_mock_server version to 1.0.0 (Ronald Holshausen, Fri Feb 10 14:43:53 2023 +1100)
* c5c66721 - bump version to 0.10.5 (Ronald Holshausen, Wed Feb 8 14:16:36 2023 +1100)

# 0.10.4 - Bugfix Release

* 019bd2fe - chore: Upgrade pact_matching to 1.0.5 (Ronald Holshausen, Wed Feb 8 13:53:15 2023 +1100)
* f6d0c35e - fix(consumer): request and response builders were using the first interaction from plugins, not the correct one (Ronald Holshausen, Wed Feb 8 13:29:02 2023 +1100)
* 1e7331f1 - fix: Upgrade plugin driver to 0.3.1 (Ronald Holshausen, Wed Feb 8 13:28:07 2023 +1100)
* 0f4178e5 - chore: Upgrade pact_matching to 1.0.4 (Ronald Holshausen, Mon Feb 6 15:40:43 2023 +1100)
* 0b70060f - chore: Upgrade pact-plugin-driver and base64 crates (supports message metadata) (Ronald Holshausen, Mon Feb 6 14:56:29 2023 +1100)
* c1b22f1c - chore: Upgrade pact_matching to 1.0.3 (Ronald Holshausen, Wed Jan 11 15:19:29 2023 +1100)
* 7d84d941 - chore: Upgrade pact_models to 1.0.4 (Ronald Holshausen, Wed Jan 11 14:33:13 2023 +1100)
* 1bdb1054 - chore: Upgrade pact_models to 1.0.3 #239 (Ronald Holshausen, Thu Dec 22 15:37:53 2022 +1100)
* 3aecb702 - chore: require tracing-subscriber for tests for crates that use pact_models #239 (Ronald Holshausen, Thu Dec 22 14:37:01 2022 +1100)
* 94cfe951 - bump version to 0.10.4 (Ronald Holshausen, Mon Dec 19 17:01:07 2022 +1100)

# 0.10.3 - Support generators in plugins

* 81e55220 - chore: Upgrade pact_mock_server to 0.9.7 (Ronald Holshausen, Mon Dec 19 16:04:55 2022 +1100)
* e827f591 - chore: Upgrade pact_matching to 1.0.2 (Ronald Holshausen, Mon Dec 19 15:30:14 2022 +1100)
* 21821045 - chore: Update mock server to start_mock_server_v2 (Ronald Holshausen, Fri Dec 16 16:49:32 2022 +1100)
* 5fbb0d6a - feat: Upgrade plugin driver to 0.2.2 (supports passing a test context to support generators) (Ronald Holshausen, Fri Dec 16 16:38:03 2022 +1100)
* 1ab47c6f - chore: Upgrade Tokio to latest (Ronald Holshausen, Fri Dec 16 16:31:31 2022 +1100)
* fb2f4204 - chore: Upgrade pact_matching to 1.0.1 (Ronald Holshausen, Wed Dec 14 17:03:31 2022 +1100)
* 8be00f0c - chore: Upgrade pact-plugin-driver to 0.2.1 (Ronald Holshausen, Wed Dec 14 14:55:32 2022 +1100)
* 7b473b2c - bump version to 0.10.3 (Ronald Holshausen, Mon Dec 12 14:27:43 2022 +1100)

# 0.10.2 - Bugfix Release

* e91ad622 - fix: Interaction builder was not copying plugin config data to the Pact metadata (Ronald Holshausen, Mon Dec 12 13:59:36 2022 +1100)
* 1fdb4176 - bump version to 0.10.2 (Ronald Holshausen, Mon Dec 12 11:42:05 2022 +1100)

# 0.10.1 - Support plugins generating interaction content

* 9be00044 - chore: Upgrade pact_mock_server to 0.9.6 (Ronald Holshausen, Mon Dec 12 10:06:25 2022 +1100)
* e7a1b9f2 - chore: Upgrade pact_matching to 1.0 and plugin driver to 0.2 (Ronald Holshausen, Fri Dec 9 17:29:33 2022 +1100)
* 2cf5d8ad - fix: Correct test after upgrading pact_models to 1.0.2 (Ronald Holshausen, Fri Dec 9 12:59:24 2022 +1100)
* 246c0730 - chore: Upgrade pact_mock_server to 0.9.5 (Ronald Holshausen, Mon Nov 28 14:52:37 2022 +1100)
* 2802fffd - chore: Upgrade pact_matching to 0.12.15 (Ronald Holshausen, Mon Nov 28 14:29:43 2022 +1100)
* c9721fd5 - chore: Upgrade pact_models to 1.0.1 and pact-plugin-driver to 0.1.16 (Ronald Holshausen, Mon Nov 28 14:10:53 2022 +1100)
* 2e5823a0 - feat: add custom-header to the old FFI args for implementations that have not moved to handles (Ronald Holshausen, Fri Nov 25 11:09:46 2022 +1100)
* c6aebcb5 - chore: add a test with two near identical interactions (Ronald Holshausen, Mon Nov 14 12:37:05 2022 +1100)
* 57de6527 - bump version to 0.10.1 (Ronald Holshausen, Mon Nov 7 13:49:06 2022 +1100)

# 0.10.0 - Bugfix Release

* 8ec8fe9b - chore: Upgrade pact_consumer to 0.10.0 (Ronald Holshausen, Mon Nov 7 13:15:23 2022 +1100)
* 248d9502 - chore: fix code blocks in readme (Ronald Holshausen, Mon Nov 7 12:58:08 2022 +1100)
* a3fa7d63 - feat: Split the builders into synch and async versions (Ronald Holshausen, Mon Nov 7 12:46:00 2022 +1100)
* 10b1aa34 - chore: Upgrade dependencies (Ronald Holshausen, Mon Nov 7 11:56:26 2022 +1100)
* a3110bd6 - chore: Upgrade pact_mock_server to 0.9.4 (Ronald Holshausen, Mon Nov 7 11:50:05 2022 +1100)
* 123060e3 - chore: Upgrade pact_matching to 0.12.14 (Ronald Holshausen, Mon Nov 7 11:34:36 2022 +1100)
* 577824e7 - fix: Upgrade pact_models to 1.0 and pact-plugin-driver to 0.1.15 to fix cyclic dependency issue (Ronald Holshausen, Mon Nov 7 11:14:20 2022 +1100)
* e1f985ad - chore: Upgrade pact_models to 0.4.6 and pact-plugin-driver to 0.1.14 (Ronald Holshausen, Fri Nov 4 16:38:36 2022 +1100)
* 6ad00a5d - fix: Update onig to latest master to fix  Regex Matcher Fails On Valid Inputs #214 (Ronald Holshausen, Fri Nov 4 15:23:50 2022 +1100)
* 965a1c41 - fix: Upgrade plugin driver to 0.1.13 (fixes issue loading plugin when there are multiple versions for the same plugin) (Ronald Holshausen, Wed Oct 5 17:29:37 2022 +1100)
* 02d9e2cb - chore: Upgrade pact matching crate to 0.12.12 (Ronald Holshausen, Wed Sep 28 10:11:11 2022 +1000)
* 60b2b642 - chore: Upgrade pact-plugin-driver to 0.1.12 (Ronald Holshausen, Mon Sep 12 17:44:13 2022 +1000)
* f3af0e5e - bump version to 0.9.8 (Ronald Holshausen, Thu Sep 8 12:16:31 2022 +1000)

# 0.9.7 - Bugfix Release

* 57a8ad7d - fix: Consumer DSL needs to increment plugin access to avoid plugin shutting down when mock server starts (Ronald Holshausen, Thu Sep 8 11:54:33 2022 +1000)
* fcab3016 - chore: Upgrade pact-plugin-driver to 0.1.11 (Ronald Holshausen, Thu Sep 8 11:28:52 2022 +1000)
* ac4fe73f - chore: fix to release scripts (Ronald Holshausen, Wed Sep 7 10:51:01 2022 +1000)
* f8db90d2 - fix: Upgrade pact_models to 0.4.5 - fixes FFI bug with generators for request paths (Ronald Holshausen, Fri Aug 26 11:44:08 2022 +1000)
* c4f1c973 - bump version to 0.9.7 (Ronald Holshausen, Thu Aug 18 16:12:40 2022 +1000)

# 0.9.6 - Maintenance Release

* 296e43ae - chore: cleanup some compiler warnings (Ronald Holshausen, Thu Aug 18 16:09:02 2022 +1000)
* 9d1e8e89 - chore: Upgrade pact_mock_server to 0.9.3 (Ronald Holshausen, Thu Aug 18 16:03:38 2022 +1000)
* 1d5fb787 - chore: Upgrade pact_matching to 0.12.11 (Ronald Holshausen, Thu Aug 18 15:07:23 2022 +1000)
* 32a70382 - chore: Upgrade pact_models (0.4.4), plugin driver (0.1.10), tracing and tracing core crates (Ronald Holshausen, Thu Aug 18 14:41:52 2022 +1000)
* a41fe69c - chore: Upgrade pact_mock_server to 0.9.2 (Ronald Holshausen, Mon Aug 15 17:40:09 2022 +1000)
* 8056d7e9 - fix: get verify_provider_async to wait on the metric call (Ronald Holshausen, Thu Aug 11 16:16:18 2022 +1000)
* 7b6a919b - chore: Upgrade pact_matching crate to 0.12.10 (Ronald Holshausen, Wed Aug 10 12:37:11 2022 +1000)
* 195ad07b - chore: Updated dependant crates (uuid, simplelog) (Ronald Holshausen, Wed Aug 10 10:22:07 2022 +1000)
* 49232caa - chore: Update pact plugin driver to 0.1.9 (Ronald Holshausen, Wed Aug 10 10:14:42 2022 +1000)
* a3fe5e7f - chore: Update pact models to 0.4.2 (Ronald Holshausen, Wed Aug 10 10:10:41 2022 +1000)
* 24186e90 - feat: allow the interaction transport to be set in consumer tests (Ronald Holshausen, Wed Aug 3 12:47:27 2022 +1000)
* 9a6c846f - chore: Upgrade pact_matching to 0.12.9 (Ronald Holshausen, Fri Jun 10 15:46:07 2022 +1000)
* e099715a - bump version to 0.9.6 (Ronald Holshausen, Mon May 30 12:27:28 2022 +1000)

# 0.9.5 - Maintenance Release

* f42026d5 - chore: Upgrade pact_mock_server to 0.9.1 (Ronald Holshausen, Mon May 30 12:09:06 2022 +1000)
* bcddbcfb - chore: Upgrade pact_matching to 0.12.8 (Ronald Holshausen, Mon May 30 11:52:26 2022 +1000)
* 80256458 - chore: Upgrade pact-plugin-driver to 0.1.8 (Ronald Holshausen, Mon May 30 11:36:54 2022 +1000)
* 08e6aa12 - bump version to 0.9.5 (Ronald Holshausen, Mon May 23 14:37:14 2022 +1000)

# 0.9.4 - Maintenance Release

* d9b9fe72 - chore: Upgrade pact-plugin-driver to 0.1.7 (Ronald Holshausen, Fri May 20 15:56:23 2022 +1000)
* 6463c5ea - bump version to 0.9.4 (Ronald Holshausen, Wed May 11 17:59:26 2022 +1000)
* ff607074 - chore: update readme (Ronald Holshausen, Wed May 11 17:56:49 2022 +1000)

# 0.9.3 - Maintenance Release

* 91d72007 - chore: switch from logging crate to tracing crate (Ronald Holshausen, Wed May 11 17:50:30 2022 +1000)
* 5c0f28fa - chore: Upgrade crate dependencies (Ronald Holshausen, Wed May 11 17:46:26 2022 +1000)
* f6b942da - chore: Upgrade pact_mock_server to 0.8.11 (Ronald Holshausen, Wed May 11 17:00:46 2022 +1000)
* 08f28e4a - chore: Upgrade pact_matching to 0.12.7 (Ronald Holshausen, Wed May 11 15:57:36 2022 +1000)
* 37bfc5de - chore: Upgrade pact-plugin-driver to 0.1.6 (Ronald Holshausen, Wed May 11 11:56:23 2022 +1000)
* 020b5715 - chore: upgrade pact_models to 0.4.1 (Ronald Holshausen, Wed May 11 11:36:57 2022 +1000)
* eebe3bc2 - bump version to 0.9.3 (Ronald Holshausen, Wed Apr 27 15:52:57 2022 +1000)

# 0.9.2 - Maintenance Release

* 563ae9fc - chore: Upgrade pact_mock_server to 0.8.10 (Ronald Holshausen, Wed Apr 27 15:06:50 2022 +1000)
* bcae77b4 - chore: upgrade pact_matching to 0.12.6 (Ronald Holshausen, Wed Apr 27 14:29:26 2022 +1000)
* dba7252e - chore: Upgrade pact-plugin-driver to 0.1.5 (Ronald Holshausen, Tue Apr 26 13:56:22 2022 +1000)
* 688e49e7 - chore: Upgrade pact-plugin-driver to 0.1.4 (Ronald Holshausen, Fri Apr 22 14:47:01 2022 +1000)
* cdf72b05 - feat: forward provider details to plugin when verifying (Ronald Holshausen, Fri Apr 22 14:12:34 2022 +1000)
* 2395143a - feat: forward verification to plugin for transports provided by the plugin (Ronald Holshausen, Fri Apr 22 12:02:05 2022 +1000)
* 6704f230 - bump version to 0.9.2 (Ronald Holshausen, Wed Apr 13 16:19:13 2022 +1000)

# 0.9.1 - Bugfix Release

* 1e8ae855 - chore: Upgrade pact_mock_server to 0.8.9 (Ronald Holshausen, Wed Apr 13 15:49:03 2022 +1000)
* 0df06dd2 - chore: Upgrade pact_matching to 0.12.5 (Ronald Holshausen, Wed Apr 13 15:38:49 2022 +1000)
* d043f6c7 - chore: upgrade pact_models to 0.3.3 (Ronald Holshausen, Wed Apr 13 15:24:33 2022 +1000)
* eee09ba6 - chore: Upgrade pact-plugin-driver to 0.1.3 (Ronald Holshausen, Wed Apr 13 14:07:36 2022 +1000)
* 73ae0ef0 - fix: Upgrade reqwest to 0.11.10 to resolve #156 (Ronald Holshausen, Wed Apr 13 13:31:55 2022 +1000)
* ffeca2e2 - chore: update to the latest plugin driver (Ronald Holshausen, Wed Apr 13 13:08:25 2022 +1000)
* 23a0f91f - bump version to 0.9.1 (Ronald Holshausen, Thu Mar 24 15:15:31 2022 +1100)

# 0.9.0 - Supports mock servers from plugins

* 89027c87 - chore: update pact_matching (0.12.4) and pact_mock_server (0.8.8) (Ronald Holshausen, Thu Mar 24 14:09:45 2022 +1100)
* 9baf03a9 - chore: use the published version of the plugin driver (Ronald Holshausen, Thu Mar 24 13:36:01 2022 +1100)
* 345b0011 - feat: support mock servers provided from plugins (Ronald Holshausen, Mon Mar 21 15:59:46 2022 +1100)
* 6772f111 - Merge branch 'master' into feat/plugin-mock-server (Ronald Holshausen, Fri Mar 18 14:55:41 2022 +1100)
* 6818367f - chore: fix build after releasing models 0.3.1 (Ronald Holshausen, Fri Mar 18 14:55:26 2022 +1100)
* efb5f12b - refactor: Split ValidatingMockServer into a trait and implementation (Ronald Holshausen, Tue Mar 15 17:07:15 2022 +1100)
* e10841d7 - chore: bump consumer crate to 0.9.0 (Ronald Holshausen, Tue Mar 15 14:16:47 2022 +1100)
* ae87a65d - bump version to 0.8.7 (Ronald Holshausen, Fri Mar 4 14:58:50 2022 +1100)

# 0.8.6 - Maintenance Release

* 5a4a8a1c - chore: update pact_mock_server to 0.8.7 (Ronald Holshausen, Fri Mar 4 14:24:23 2022 +1100)
* 8894fdfd - chore: update pact_matching to 0.12.3 (Ronald Holshausen, Fri Mar 4 14:09:17 2022 +1100)
* 8e864502 - chore: update all dependencies (Ronald Holshausen, Fri Mar 4 13:29:59 2022 +1100)
* 50c73c68 - chore: remove accidental debug statement (Ronald Holshausen, Mon Feb 14 09:04:22 2022 +1100)
* 418e81f8 - bump version to 0.8.6 (Ronald Holshausen, Mon Jan 17 17:12:24 2022 +1100)

# 0.8.5 - Bugfix Release

* 10c9b842 - chore: Upgrade pact_mock_server to 0.8.6 (Ronald Holshausen, Mon Jan 17 16:57:31 2022 +1100)
* 5e4c68ef - chore: update pact matching to 0.12.2 (Ronald Holshausen, Mon Jan 17 16:29:21 2022 +1100)
* 80b241c5 - chore: Upgrade plugin driver crate to 0.0.17 (Ronald Holshausen, Mon Jan 17 11:22:48 2022 +1100)
* 4f1ecff2 - chore: Upgrade pact-models to 0.2.7 (Ronald Holshausen, Mon Jan 17 10:53:26 2022 +1100)
* c2089645 - fix: log crate version must be fixed across all crates (including plugin driver) (Ronald Holshausen, Fri Jan 14 16:10:50 2022 +1100)
* 30aa8573 - bump version to 0.8.5 (Ronald Holshausen, Tue Jan 4 12:51:34 2022 +1100)

# 0.8.4 - Maintenance Release

* 62e89d78 - chore: update pact_mock_server to 0.8.5 (Ronald Holshausen, Tue Jan 4 12:45:42 2022 +1100)
* 8259c12a - chore: update pact_models 0.2.6, pact_matching 0.12.1, pact-plugin-driver 0.0.16 (Ronald Holshausen, Tue Jan 4 12:38:26 2022 +1100)
* a3297695 - chore: update pact_models 0.2.6, pact_matching 0.12.1, pact-plugin-driver 0.0.16 (Ronald Holshausen, Tue Jan 4 12:34:55 2022 +1100)
* 9c2810ad - chore: Upgrade pact-plugin-driver to 0.0.15 (Ronald Holshausen, Fri Dec 31 15:12:56 2021 +1100)
* 0a6e7d9d - refactor: Convert MatchingContext to a trait and use DocPath instead of string slices (Ronald Holshausen, Wed Dec 29 14:24:39 2021 +1100)
* 4e59a652 - bump version to 0.8.4 (Ronald Holshausen, Thu Dec 23 13:32:50 2021 +1100)

# 0.8.3 - Maintenance Release

* 4d088317 - chore: Update pact_mock_server crate to 0.8.4 (Ronald Holshausen, Thu Dec 23 13:24:15 2021 +1100)
* 52bc1735 - chore: update pact_matching crate to 0.11.5 (Ronald Holshausen, Thu Dec 23 13:12:08 2021 +1100)
* 5479a634 - chore: Update pact_models (0.2.4) and pact-plugin-driver (0.0.14) (Ronald Holshausen, Thu Dec 23 12:57:02 2021 +1100)
* fc0a8360 - chore: update pact_matching to 0.11.4 (Ronald Holshausen, Mon Dec 20 12:19:36 2021 +1100)
* 8911d5b0 - chore: update to latest plugin driver crate (metrics fixes) (Ronald Holshausen, Mon Dec 20 12:11:35 2021 +1100)
* 47c51f74 - bump version to 0.8.3 (Ronald Holshausen, Wed Dec 15 14:35:59 2021 +1100)

# 0.8.2 - Maintenance Release

* 8d8c7706 - chore: update models and mock server crates (Ronald Holshausen, Wed Dec 15 14:03:30 2021 +1100)
* cba3f08e - feat: add metrics events for Pact-Rust consumer tests (Ronald Holshausen, Tue Dec 14 16:20:40 2021 +1100)
* 4f1ba7d9 - chore: update to the latest plugin driver (Ronald Holshausen, Tue Dec 14 13:55:02 2021 +1100)
* fc5be202 - fix: update to latest driver crate (Ronald Holshausen, Tue Nov 16 16:19:02 2021 +1100)
* 24fc4e90 - bump version to 0.8.2 (Ronald Holshausen, Tue Nov 16 13:01:59 2021 +1100)

# 0.8.1 - Add plugin support to FFI functions

* 5d974c4a - chore: update to latest models and plugin driver crates (Ronald Holshausen, Tue Nov 16 11:56:53 2021 +1100)
* 20643590 - feat(plugins): add plugin support to FFI functions (Ronald Holshausen, Tue Nov 9 16:06:01 2021 +1100)
* 188caf6a - chore: update release script (Ronald Holshausen, Thu Nov 4 16:19:38 2021 +1100)
* c7dbfdb9 - bump version to 0.8.1 (Ronald Holshausen, Thu Nov 4 16:17:23 2021 +1100)

# 0.8.0 - Pact V4 release

* 6dfec56a - chore: drop beta from pact_consumer version (Ronald Holshausen, Thu Nov 4 16:08:47 2021 +1100)
* fc4580b8 - chore: drop beta from pact_mock_server version (Ronald Holshausen, Thu Nov 4 15:28:51 2021 +1100)
* bd2bd0ec - chore: drop beta from pact_matching version (Ronald Holshausen, Wed Nov 3 13:28:35 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* a561f883 - chore: use the non-beta models crate (Ronald Holshausen, Thu Oct 21 18:10:27 2021 +1100)
* e72d602a - chore: bump pact models to non-beta version (Ronald Holshausen, Thu Oct 21 17:47:17 2021 +1100)
* 75dd211c - feat: update readme with plugin example (Ronald Holshausen, Thu Oct 21 11:53:52 2021 +1100)
* be6c02b1 - feat: update readme with sync req/res message examples (Ronald Holshausen, Thu Oct 21 11:18:41 2021 +1100)
* e6610312 - feat: update readme with sync req/res message examples (Ronald Holshausen, Thu Oct 21 11:15:05 2021 +1100)
* 3b7aee5f - feat: update tests and docs with sync req/res message examples (Ronald Holshausen, Thu Oct 21 10:28:08 2021 +1100)
* 1427aa33 - feat: update tests and docs with message examples (Ronald Holshausen, Wed Oct 20 16:49:29 2021 +1100)
* 45511b6e - bump version to 0.8.0-beta.4 (Ronald Holshausen, Tue Oct 19 17:51:58 2021 +1100)

# 0.8.0-beta.3 - Bugfix Release

* df67b723 - fix: async message builder was not setting the pact plugin config correctly (Ronald Holshausen, Tue Oct 19 17:44:35 2021 +1100)
* 918e5beb - fix: update to latest models and plugin driver crates (Ronald Holshausen, Tue Oct 19 17:09:48 2021 +1100)
* 1fc6eb17 - bump version to 0.8.0-beta.3 (Ronald Holshausen, Tue Oct 19 11:56:35 2021 +1100)

# 0.8.0-beta.2 - Support matching synchronous request/response messages

* 3819522d - chore: update to the latest matching and mock server crates (Ronald Holshausen, Tue Oct 19 11:34:18 2021 +1100)
* aa434ba3 - chore: update to latest driver crate (Ronald Holshausen, Tue Oct 19 11:09:46 2021 +1100)
* df386c8a - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Oct 18 13:41:36 2021 +1100)
* 2b4b7cc3 - feat(plugins): Support matching synchronous request/response messages (Ronald Holshausen, Fri Oct 15 16:01:50 2021 +1100)
* c72f8b04 - bump version to 0.8.0-beta.2 (Ronald Holshausen, Tue Oct 12 16:43:50 2021 +1100)

# 0.8.0-beta.1 - Support consumer tests with synchronous messages (Protobuf)

* 1dc6f543 - chore: bump pact_mock_server version (Ronald Holshausen, Tue Oct 12 16:36:51 2021 +1100)
* 9bbbb52e - chore: bump pact matching crate version (Ronald Holshausen, Tue Oct 12 16:24:01 2021 +1100)
* d0bfb8a8 - feat: Support consumer tests with synchronous messages (Ronald Holshausen, Tue Oct 12 15:51:08 2021 +1100)
* 35ff0993 - feat: record the version of the lib that created the pact in the metadata (Ronald Holshausen, Tue Oct 12 14:52:43 2021 +1100)
* acbe66f1 - bump version to 0.8.0-beta.1 (Ronald Holshausen, Wed Oct 6 13:04:36 2021 +1100)

# 0.8.0-beta.0 - Plugin support with consumer tests

* a81659e9 - chore: update release script (Ronald Holshausen, Wed Oct 6 12:51:24 2021 +1100)
* ddc64246 - chore: use the published version of the models crate (Ronald Holshausen, Wed Oct 6 12:40:52 2021 +1100)
* 2c47023c - chore: pin plugin driver version to 0.0.3 (Ronald Holshausen, Wed Oct 6 11:21:07 2021 +1100)
* 288e2168 - chore: use the published version of the plugin driver lib (Ronald Holshausen, Tue Oct 5 15:36:06 2021 +1100)
* 9fd9e652 - feat: do no write empty comments + added consumer version to metadata (Ronald Holshausen, Thu Sep 30 17:40:56 2021 +1000)
* 6d23796f - feat(plugins): support each key and each value matchers (Ronald Holshausen, Wed Sep 29 11:10:46 2021 +1000)
* 6f20282d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 28 14:51:34 2021 +1000)
* b45a1fe0 - chore: correct changelog (Ronald Holshausen, Tue Sep 28 14:39:20 2021 +1000)
* 1c7f004a - bump version to 0.7.9 (Ronald Holshausen, Tue Sep 28 14:27:46 2021 +1000)
* 0d7b840e - update changelog for release 0.7.8 (Ronald Holshausen, Tue Sep 28 14:25:41 2021 +1000)
* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 21a7ede5 - feat(plugins): support matching protobuf embedded messages (Ronald Holshausen, Wed Sep 15 12:14:54 2021 +1000)
* 9bf9dc30 - feat(plugins): Add consumer message test builder + persist plugin data (Ronald Holshausen, Tue Sep 14 15:14:17 2021 +1000)
* ee3212a8 - refactor(plugins): do not expose the catalogue statics, but rather a function to initialise it (Ronald Holshausen, Tue Sep 14 15:13:12 2021 +1000)
* b71dcabf - refactor(plugins): rename ContentTypeOverride -> ContentTypeHint (Ronald Holshausen, Tue Sep 14 15:08:52 2021 +1000)
* 03ebe632 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Sep 13 12:01:13 2021 +1000)
* fd6f8f40 - chore: Bump pact_mock_server version to 0.8.0-beta.0 (Ronald Holshausen, Mon Sep 13 11:46:11 2021 +1000)
* 971b980e - chore: fix clippy warnings (Ronald Holshausen, Fri Sep 10 17:31:16 2021 +1000)
* 716809f6 - chore: Get CI build passing (Ronald Holshausen, Fri Sep 10 14:55:46 2021 +1000)
* 4aaaafd8 - feat(plugins): Support non-blocking mock server in consumer tests + shutting down plugins when mock servers shutdown (Ronald Holshausen, Fri Sep 10 13:20:01 2021 +1000)
* 37978075 - feat(plugins): support getting interaction markup from plugins (Ronald Holshausen, Thu Sep 9 12:09:06 2021 +1000)
* f44ecc54 - feat(plugins): make the interaction markup type explicit (Ronald Holshausen, Thu Sep 9 11:47:32 2021 +1000)
* ee498dce - feat: consumer builders need to populate the interaction config from the plugins (Ronald Holshausen, Thu Sep 9 08:49:54 2021 +1000)
* a89eca23 - feat(plugins): support storing interaction markup for interactions in Pact files (Ronald Holshausen, Wed Sep 8 16:39:43 2021 +1000)
* ceb1c35f - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 7 10:07:45 2021 +1000)
* b77498c8 - chore: fix tests after updating plugin API (Ronald Holshausen, Fri Sep 3 16:48:18 2021 +1000)
* aa3be062 - feat(plugins): update to the latest plugin interface (Ronald Holshausen, Fri Sep 3 13:06:02 2021 +1000)
* e8ae81b3 - refactor: matching req/res with plugins requires data from the pact and interaction (Ronald Holshausen, Thu Sep 2 11:57:50 2021 +1000)
* 1dd2d883 - feat(plugins): update to latest plugin driver and proto (Ronald Holshausen, Tue Aug 31 11:59:53 2021 +1000)
* b9aa7ecb - feat(Plugins): allow plugins to override text/binary format of the interaction content (Ronald Holshausen, Mon Aug 30 10:48:04 2021 +1000)
* eb34b011 - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Aug 23 15:48:55 2021 +1000)
* 0c5cede2 - chore: bump models crate to 0.2 (Ronald Holshausen, Mon Aug 23 12:56:14 2021 +1000)
* e3a2660f - chore: fix tests after updating test builders to be async (Ronald Holshausen, Fri Aug 20 12:41:10 2021 +1000)
* 8c61ae96 - feat(plugins): Support plugins with the consumer DSL interaction/response (Ronald Holshausen, Thu Aug 19 15:24:04 2021 +1000)
* b75fea5d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Aug 18 12:27:41 2021 +1000)
* 8bcd1c7e - fix: min/max type matchers must not apply the limits when cascading (Ronald Holshausen, Sun Aug 8 15:50:40 2021 +1000)
* 6124ed0b - refactor: Introduce DocPath struct for path expressions (Caleb Stepanian, Thu Jul 29 12:27:32 2021 -0400)
* 9baa714d - chore: bump minor version of matching crate (Ronald Holshausen, Fri Jul 23 14:03:20 2021 +1000)
* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* 0671221f - bump version to 0.7.8 (Ronald Holshausen, Fri Jul 23 11:00:25 2021 +1000)
* 85f08255 - update changelog for release 0.7.7 (Ronald Holshausen, Fri Jul 23 10:58:24 2021 +1000)
* 809e22fc - Revert "update changelog for release 0.7.7" (Ronald Holshausen, Fri Jul 23 10:24:58 2021 +1000)
* 759d627a - update changelog for release 0.7.7 (Ronald Holshausen, Fri Jul 23 10:22:28 2021 +1000)
* 3dccf866 - refacfor: moved the pact structs to the models crate (Ronald Holshausen, Sun Jul 18 16:58:14 2021 +1000)
* e8046d84 - refactor: moved interaction structs to the models crate (Ronald Holshausen, Sun Jul 18 14:36:03 2021 +1000)
* 084ab46b - feat: Copied pact_mockserver_ffi to pact_ffi (Ronald Holshausen, Sat Jul 10 16:24:29 2021 +1000)
* e2e10241 - refactor: moved Request and Response structs to the models crate (Ronald Holshausen, Wed Jul 7 18:09:36 2021 +1000)
* 01ff9877 - refactor: moved matching rules and generators to models crate (Ronald Holshausen, Sun Jul 4 17:17:30 2021 +1000)
* c3c22ea8 - Revert "refactor: moved matching rules and generators to models crate (part 1)" (Ronald Holshausen, Wed Jun 23 14:37:46 2021 +1000)
* d3406650 - refactor: moved matching rules and generators to models crate (part 1) (Ronald Holshausen, Wed Jun 23 12:58:30 2021 +1000)
* 6198538d - refactor: move time_utils to pact_models crate (Ronald Holshausen, Fri Jun 11 12:58:26 2021 +1000)
* 5c670814 - refactor: move expression_parser to pact_models crate (Ronald Holshausen, Fri Jun 11 10:51:51 2021 +1000)
* be604cce - feat: add date-time matcher to consumer DSL (Ronald Holshausen, Wed Jun 2 15:19:06 2021 +1000)
* b4e26844 - fix: reqwest is dyn linked to openssl by default, which causes a SIGSEGV on alpine linux (Ronald Holshausen, Tue Jun 1 14:21:31 2021 +1000)
* 68f8f84e - chore: skip failing tests in alpine to get the build going (Ronald Holshausen, Tue Jun 1 13:47:20 2021 +1000)
* c5059104 - bump version to 0.7.7 (Ronald Holshausen, Sun May 30 18:58:43 2021 +1000)

# 0.7.8 - Bugfixes + support native TLS certs

* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 971b980e - chore: fix clippy warnings (Ronald Holshausen, Fri Sep 10 17:31:16 2021 +1000)
* 8bcd1c7e - fix: min/max type matchers must not apply the limits when cascading (Ronald Holshausen, Sun Aug 8 15:50:40 2021 +1000)
* 6124ed0b - refactor: Introduce DocPath struct for path expressions (Caleb Stepanian, Thu Jul 29 12:27:32 2021 -0400)
* 9baa714d - chore: bump minor version of matching crate (Ronald Holshausen, Fri Jul 23 14:03:20 2021 +1000)
* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* 0671221f - bump version to 0.7.8 (Ronald Holshausen, Fri Jul 23 11:00:25 2021 +1000)

# 0.7.7 - Bugfix Release

* ad7d3d54 - chore: pin the dependant versions for now (Ronald Holshausen, Fri Jul 23 10:42:52 2021 +1000)
* 084ab46b - feat: Copied pact_mockserver_ffi to pact_ffi (Ronald Holshausen, Sat Jul 10 16:24:29 2021 +1000)
* e2e10241 - refactor: moved Request and Response structs to the models crate (Ronald Holshausen, Wed Jul 7 18:09:36 2021 +1000)
* 01ff9877 - refactor: moved matching rules and generators to models crate (Ronald Holshausen, Sun Jul 4 17:17:30 2021 +1000)
* c3c22ea8 - Revert "refactor: moved matching rules and generators to models crate (part 1)" (Ronald Holshausen, Wed Jun 23 14:37:46 2021 +1000)
* d3406650 - refactor: moved matching rules and generators to models crate (part 1) (Ronald Holshausen, Wed Jun 23 12:58:30 2021 +1000)
* 6198538d - refactor: move time_utils to pact_models crate (Ronald Holshausen, Fri Jun 11 12:58:26 2021 +1000)
* 5c670814 - refactor: move expression_parser to pact_models crate (Ronald Holshausen, Fri Jun 11 10:51:51 2021 +1000)
* be604cce - feat: add date-time matcher to consumer DSL (Ronald Holshausen, Wed Jun 2 15:19:06 2021 +1000)
* b4e26844 - fix: reqwest is dyn linked to openssl by default, which causes a SIGSEGV on alpine linux (Ronald Holshausen, Tue Jun 1 14:21:31 2021 +1000)
* 68f8f84e - chore: skip failing tests in alpine to get the build going (Ronald Holshausen, Tue Jun 1 13:47:20 2021 +1000)
* c5059104 - bump version to 0.7.7 (Ronald Holshausen, Sun May 30 18:58:43 2021 +1000)

# 0.7.6 - V4 features + DSL enhancements

* 7022625 - refactor: move provider state models to the pact models crate (Ronald Holshausen, Sat May 29 17:18:48 2021 +1000)
* 73a53b8 - feat(V4): add an HTTP status code matcher (Ronald Holshausen, Fri May 28 18:40:11 2021 +1000)
* 7e4caf8 - feat(V4): added a pending flag to V4 interactions (Ronald Holshausen, Thu May 27 16:59:18 2021 +1000)
* ffbcaf5 - feat: Added header_from_provider_state and path_from_provider_state (Rob Caiger, Mon May 24 13:54:16 2021 +0100)
* 735c9e7 - chore: bump pact_matching to 0.9 (Ronald Holshausen, Sun Apr 25 13:50:18 2021 +1000)
* fb373b4 - chore: bump version to 0.0.2 (Ronald Holshausen, Sun Apr 25 13:40:52 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* 3dd610a - refactor: move structs and code dealing with bodies to a seperate package (Ronald Holshausen, Sun Apr 25 11:20:47 2021 +1000)
* 80b7148 - feat(V4): Updated consumer DSL to set comments + mock server initial support for V4 pacts (Ronald Holshausen, Fri Apr 23 17:58:10 2021 +1000)
* 4bcd94f - refactor: moved OptionalBody and content types to pact models crate (Ronald Holshausen, Thu Apr 22 14:01:56 2021 +1000)
* 80812d0 - refactor: move Consumer and Provider structs to models crate (Ronald Holshausen, Thu Apr 22 13:11:03 2021 +1000)
* 5ed389b - bump version to 0.7.6 (Ronald Holshausen, Sun Mar 14 15:39:21 2021 +1100)

# 0.7.5 - mock server metrics

* 5a529fd - feat: add ability of mock server to expose metrics #94 (Ronald Holshausen, Sun Mar 14 11:41:16 2021 +1100)
* eec03d2 - bump version to 0.7.5 (Ronald Holshausen, Mon Feb 8 16:23:43 2021 +1100)

# 0.7.4 - Use a file system lock when merging pact files

* 9976e80 - feat: added read locks and a mutex guard to reading and writing pacts (Ronald Holshausen, Mon Feb 8 11:58:52 2021 +1100)
* e43fdb8 - chore: upgrade maplit, itertools (Audun Halland, Mon Jan 11 05:30:10 2021 +0100)
* 8792c29 - bump version to 0.7.4 (Ronald Holshausen, Mon Jan 11 15:11:22 2021 +1100)

# 0.7.3 - Updated dependencies

* 4a70bef - chore: upgrade expectest to 0.12 (Audun Halland, Sat Jan 9 11:29:29 2021 +0100)
* 3a28a6c - chore: upgrade regex, chrono-tz (Audun Halland, Sat Jan 9 11:12:49 2021 +0100)
* 1483fef - chore: upgrade uuid to 0.8 (Audun Halland, Sat Jan 9 11:03:30 2021 +0100)
* 1ac3548 - chore: upgrade env_logger to 0.8 (Audun Halland, Sat Jan 9 09:50:27 2021 +0100)
* 9a8a63f - chore: upgrade quickcheck (Audun Halland, Sat Jan 9 08:46:51 2021 +0100)
* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* c9a7e44 - bump version to 0.7.3 (Ronald Holshausen, Tue Jan 5 13:39:06 2021 +1100)

# 0.7.2 - Upgrade Tokio to 1.0

* ef76f38 - chore: cleanup compiler warnings (Ronald Holshausen, Tue Jan 5 10:10:39 2021 +1100)
* 4636982 - chore: update other crates to use Tokio 1.0 (Ronald Holshausen, Mon Jan 4 17:26:59 2021 +1100)
* 648a8a3 - bump version to 0.7.2 (Ronald Holshausen, Thu Dec 31 14:47:10 2020 +1100)

# 0.7.1 - support generators associated with array contains matcher variants

* beb1c03 - fix: cleanup compiler warning (Ronald Holshausen, Thu Dec 31 14:41:09 2020 +1100)
* 335e921 - chore: update pact_matching and pact_mock_server crates to latest (Ronald Holshausen, Thu Dec 31 14:39:50 2020 +1100)
* 5e56ecb - refactor: support generators associated with array contains matcher variants (Ronald Holshausen, Tue Dec 29 11:46:56 2020 +1100)
* 6182b56 - bump version to 0.7.1 (Ronald Holshausen, Fri Oct 16 13:25:59 2020 +1100)

# 0.7.0 - Update to latest matching and mock server crates

* 5e0e470 - chore: bump minor version of pact_consumer crate (Ronald Holshausen, Fri Oct 16 13:22:12 2020 +1100)
* 13976f5 - fix: failing pact_consumer build (Ronald Holshausen, Thu Oct 15 12:00:09 2020 +1100)
* 2fb0c6e - fix: fix the build after refactoring the pact write function (Ronald Holshausen, Wed Oct 14 11:07:57 2020 +1100)
* f334a4f - refactor: introduce a MatchingContext into all matching functions + delgate to matchers for collections (Ronald Holshausen, Mon Oct 12 14:06:00 2020 +1100)
* 7fbc731 - chore: bump minor version of matching lib (Ronald Holshausen, Fri Oct 9 10:42:33 2020 +1100)
* 29ba743 - feat: add a mock server config struct (Ronald Holshausen, Thu Sep 24 10:30:59 2020 +1000)
* 2d44ffd - chore: bump minor version of the matching crate (Ronald Holshausen, Mon Sep 14 12:06:37 2020 +1000)
* 814c416 - refactor: added a trait for interactions, renamed Interaction to RequestResponseInteraction (Ronald Holshausen, Sun Sep 13 17:09:41 2020 +1000)
* cc9661f - chore: cleanup some deprecation warnings (Ronald Holshausen, Sun Sep 13 13:17:08 2020 +1000)
* a05bcbb - refactor: renamed Pact to RequestResponsePact (Ronald Holshausen, Sun Sep 13 12:45:34 2020 +1000)
* 359a944 - chore: update versions in readmes (Ronald Holshausen, Sat Jun 27 13:21:24 2020 +1000)
* fc86c16 - bump version to 0.6.3 (Ronald Holshausen, Wed Jun 24 11:09:16 2020 +1000)

# 0.6.2 - Updated XML Matching

* 97d8521 - chore: update to latest matching crate (Ronald Holshausen, Wed Jun 24 11:03:24 2020 +1000)
* a15edea - chore: try set the content type on the body if known (Ronald Holshausen, Tue Jun 23 16:53:32 2020 +1000)
* 570f405 - chore: correct version in readme (Ronald Holshausen, Wed May 27 16:38:58 2020 +1000)
* a54dfd0 - bump version to 0.6.2 (Ronald Holshausen, Wed May 27 14:43:24 2020 +1000)

# 0.6.1 - Bugfix Release

* bea787c - chore: bump matching crate version to 0.6.0 (Ronald Holshausen, Sat May 23 17:56:04 2020 +1000)
* 754a483 - chore: updated itertools to latest (Ronald Holshausen, Wed May 6 15:49:27 2020 +1000)
* a45d0c3 - fix: FFI mismatch json should have the actual values as UTF-8 string not bytes #64 (Ronald Holshausen, Thu Apr 30 11:16:25 2020 +1000)
* 411f697 - chore: correct some clippy warnings (Ronald Holshausen, Wed Apr 29 16:49:36 2020 +1000)
* f84e672 - chore: update mock server library to latest (Ronald Holshausen, Fri Apr 24 11:00:34 2020 +1000)
* 43de9c3 - chore: update matching library to latest (Ronald Holshausen, Fri Apr 24 10:20:55 2020 +1000)
* 6ff9c33 - fix: ignore flakey test (Matt Fellows, Tue Mar 3 12:14:08 2020 +1100)
* 3c590fb - bump version to 0.6.1 (Ronald Holshausen, Sun Jan 19 11:46:21 2020 +1100)

# 0.6.0 - Convert to async/await

* 9d3ad57 - chore: bump minor version of pact consumer crate (Ronald Holshausen, Sun Jan 19 11:40:27 2020 +1100)
* d457221 - chore: update dependant crates to use mock server lib 0.7.0 (Ronald Holshausen, Sun Jan 19 11:31:21 2020 +1100)
* cb4c560 - Upgrade tokio to 0.2.9 (Audun Halland, Fri Jan 10 00:13:02 2020 +0100)
* e8034bf - Remove mock server async spawning. (Audun Halland, Thu Jan 9 21:59:56 2020 +0100)
* 3dec6ff - Upgrade tokio to 0.2.6 (Audun Halland, Tue Dec 31 07:40:14 2019 +0100)
* 9dec41b - Upgrade reqwest to 0.10 (Audun Halland, Tue Dec 31 07:22:36 2019 +0100)
* ec81ed2 - pact_consumer test: Use blocking reqwest (Audun Halland, Tue Dec 17 02:27:24 2019 +0100)
* fda11e4 - Merge remote-tracking branch 'upstream/master' into async-await (Audun Halland, Tue Dec 17 02:13:58 2019 +0100)
* 5573583 - Add more doc (Audun Halland, Tue Dec 17 01:56:03 2019 +0100)
* d395d2d - pact_verifier: Upgrade reqwest to latest git alpha (Audun Halland, Tue Dec 17 00:57:16 2019 +0100)
* 298f217 - pact_matching: Upgrade reqwest to current alpha (Audun Halland, Tue Dec 17 00:36:33 2019 +0100)
* c4dea34 - pact_consumer: Upgrade blocking to 2.1, upgrade reqwest to unreleased alpha (Audun Halland, Tue Dec 17 00:16:30 2019 +0100)
* 6e4f12b - bump version to 0.5.4 (Ronald Holshausen, Sat Dec 14 18:31:17 2019 +1100)
* fee6d06 - pact_consumer: Better mock server documentation (Audun Halland, Thu Dec 12 21:44:09 2019 +0100)
* 3074059 - Refactor ValidatingMockServer into a trait, with two implementations (Audun Halland, Thu Dec 12 15:58:50 2019 +0100)
* fe72f92 - Temporarily solve a problem where a spawned server prevents the test runtime from terminating (Audun Halland, Thu Dec 12 14:14:02 2019 +0100)
* 6a43f82 - Cut down tokio features to the bone (Audun Halland, Wed Dec 11 22:15:03 2019 +0100)
* d4bdcb6 - Update ValidatingMockServer interfaces for use with tokio::test (Audun Halland, Wed Dec 11 22:01:06 2019 +0100)

# 0.5.3 - Bugfix Release

* ec1a368 - chore: update lib versions (Ronald Holshausen, Sat Dec 14 18:09:26 2019 +1100)
* 19e8ced - fix: cleanup env var and set tests to not run in parallel on CI #54 (Ronald Holshausen, Sat Dec 14 16:08:56 2019 +1100)
* b5474b4 - fix: set the path to the generated pact file #54 (Ronald Holshausen, Sat Dec 14 15:46:37 2019 +1100)
* d4dd39f - fix: repeat the test 3 times #54 (Ronald Holshausen, Sat Dec 14 15:30:01 2019 +1100)
* bc044be - fix: check the size of the merged pact file #54 (Ronald Holshausen, Sat Dec 14 15:25:33 2019 +1100)
* a660b87 - fix: correct pact merging to remove duplicates #54 (Ronald Holshausen, Sat Dec 14 15:06:30 2019 +1100)
* 51f5a3e - Update READMEs and doc to not require any "extern crate" (Audun Halland, Sun Nov 17 23:28:21 2019 +0100)
* 9ba4fc1 - Fix doc uses in pact_consumer (Audun Halland, Sun Nov 17 02:43:53 2019 +0100)
* 276fa40 - 2018ize pact_consumer (Audun Halland, Sun Nov 17 00:21:59 2019 +0100)
* 346bf5e - Update pact_consumer/README (Audun Halland, Sun Nov 17 00:04:54 2019 +0100)
* 4a7d402 - Remove macro_use from documentation (Audun Halland, Sun Nov 17 00:02:58 2019 +0100)
* 713cd6a - Explicit edition 2018 in Cargo.toml files (Audun Halland, Sat Nov 16 23:55:37 2019 +0100)
* 924452f - 2018 edition autofix "cargo fix --edition" (Audun Halland, Sat Nov 16 22:27:42 2019 +0100)
* d736b8a - bump version to 0.5.3 (Ronald Holshausen, Mon Sep 30 11:02:42 2019 +1000)

# 0.5.2 - Fix dependency versions

* b5ab246 - chore: update the pact_matching and pact_mock_server to latest versions (Ronald Holshausen, Mon Sep 30 10:41:02 2019 +1000)
* eef3d97 - feat: added some tests for publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 16:44:52 2019 +1000)
* 1110b47 - feat: implemented publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 13:53:27 2019 +1000)
* 2488ab9 - Merge branch 'master' of https://github.com/pact-foundation/pact-reference (milleniumbug, Wed Sep 18 11:32:03 2019 +0200)
* 097d045 - refactor: added a mock server ffi module and bumped the mock server minor version (Ronald Holshausen, Sat Sep 7 09:39:27 2019 +1000)
* b48ee72 - Provide public API for passing in a listener address and post (milleniumbug, Thu Sep 5 15:20:37 2019 +0200)
* f79b033 - chore: update terminal support in release scripts (Ronald Holshausen, Sat Aug 24 12:25:28 2019 +1000)
* bcc75da - bump version to 0.5.2 (Ronald Holshausen, Sat Aug 24 12:20:56 2019 +1000)

# 0.5.1 - support headers with multiple values

* da1956a - chore: bump the version of the matching lib (Ronald Holshausen, Sat Aug 24 12:06:51 2019 +1000)
* f0c0d07 - feat: support headers with multiple values (Ronald Holshausen, Sat Aug 10 17:01:10 2019 +1000)
* b595eff - bump version to 0.5.1 (Ronald Holshausen, Sat Jul 27 17:22:11 2019 +1000)

# 0.5.0 - Upgrade to non-blocking Hyper 0.12

* d842100 - chore: bump component versions to 0.5.0 (Ronald Holshausen, Sat Jul 27 15:44:51 2019 +1000)
* ee8a898 - Rewrite server matches sync from mpsc queue to Arc<Mutex<Vec>>. Avoids awkward synchronization (Audun Halland, Tue Jul 23 02:10:55 2019 +0200)
* 4df2797 - Rename API function again (Audun Halland, Mon Jul 22 23:38:11 2019 +0200)
* 7f7dcb0 - Don't expose tokio Runtime inside the libraries (Audun Halland, Mon Jul 22 02:18:52 2019 +0200)
* 522e7ba - Set runtime::Builder core_threads instead of blocking_threads (Audun Halland, Sun May 12 10:36:54 2019 +0200)
* 3277301 - No point having MockServer in an Option, as shutdown signal consumption is now encapsulated (Audun Halland, Sun May 12 10:32:51 2019 +0200)
* a0dc885 - Shut down MockServer without consuming self, by putting shutdown_tx in an Option (Audun Halland, Sun May 12 10:28:27 2019 +0200)
* 39d231d - pact_consumer async support (untested) (Audun Halland, Sun May 12 03:45:05 2019 +0200)
* f8fa0d8 - chore: Bump pact matchig version to 0.5.0 (Ronald Holshausen, Sat Jan 5 19:25:53 2019 +1100)
* 1e0c65b - fix: doc tests with Into trait fail to link with Rust beta 1.27.0 (Ronald Holshausen, Sun May 13 15:26:36 2018 +1000)
* a5588dc - feat: Allow the directory pacts are written to to be overriden in consumer tests #21 (Ronald Holshausen, Sun Apr 8 15:20:38 2018 +1000)
* b83a0f6 - bump version to 0.4.1 (Ronald Holshausen, Sat Apr 7 14:45:05 2018 +1000)

# 0.4.0 - First V3 specification release

* 398edaf - Upgrade UUID library to latest (Ronald Holshausen, Sat Apr 7 12:29:58 2018 +1000)
* 6597141 - WIP - start of implementation of applying generators to the bodies (Ronald Holshausen, Sun Mar 4 17:01:11 2018 +1100)
* 7fef36b - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sat Nov 4 12:49:07 2017 +1100)
* 5a83885 - bump version to 0.3.2 (Ronald Holshausen, Fri Nov 3 14:54:22 2017 +1100)
* a905bed - Cleaned up some compiler warnings (Ronald Holshausen, Sun Oct 22 12:26:09 2017 +1100)
* 940a0e3 - Reverted hyper to 0.9.x (Ronald Holshausen, Sun Oct 22 12:01:17 2017 +1100)
* fbe35d8 - Compiling after merge from v2-spec (Ronald Holshausen, Sun Oct 22 11:39:46 2017 +1100)
* 00dc75a - Bump version to 0.4.0 (Ronald Holshausen, Sun Oct 22 10:46:48 2017 +1100)
* 184127a - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sun Oct 22 10:32:31 2017 +1100)
* e82ee08 - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Mon Oct 16 09:24:11 2017 +1100)
* 64ff667 - Upgraded the mock server implemenation to use Hyper 0.11.2 (Ronald Holshausen, Wed Sep 6 12:56:47 2017 +1000)
* e5a93f3 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Aug 20 09:53:48 2017 +1000)
* 8797c6c - First successful build after merge from master (Ronald Holshausen, Sun Oct 23 11:59:55 2016 +1100)
* 639ac22 - fixes after merge in from master (Ronald Holshausen, Sun Oct 23 10:45:54 2016 +1100)
* 49e45f7 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Oct 23 10:10:40 2016 +1100)

# 0.3.1 - Converted OptionalBody::Present to take a Vec<u8>

* 24e3f73 - Converted OptionalBody::Present to take a Vec<u8> #19 (Ronald Holshausen, Sun Oct 22 18:04:46 2017 +1100)
* 1c70982 - bump version to 0.3.1 (Ronald Holshausen, Fri Oct 20 11:46:27 2017 +1100)

# 0.3.0 - Improved Consumer DSL

* 89bebb3 - Correct the paths in the release scripts for pact_consumer (Ronald Holshausen, Fri Oct 20 11:36:05 2017 +1100)
* ac94388 - Tests are now all passing #20 (Ronald Holshausen, Thu Oct 19 15:14:25 2017 +1100)
* d990729 - Some code cleanup #20 (Ronald Holshausen, Wed Oct 18 16:32:37 2017 +1100)
* db6100e - Updated the consumer DSL to use the matching rules (compiling, but tests are failing) #20 (Ronald Holshausen, Wed Oct 18 15:48:23 2017 +1100)
* c983c63 - Bump versions to 0.3.0 (Ronald Holshausen, Wed Oct 18 13:54:46 2017 +1100)
* 44e2cf6 - Add myself to "authors" list (Eric Kidd, Wed Oct 11 11:31:08 2017 -0400)
* 1029745 - Provide more context in top-level crate docs (Eric Kidd, Wed Oct 11 11:29:30 2017 -0400)
* 28b7742 - Add a `strip_null_fields` helper (Eric Kidd, Wed Oct 11 11:21:22 2017 -0400)
* 3e3e5a7 - Change `json` helper to `json_utf8` (Eric Kidd, Wed Oct 11 10:06:15 2017 -0400)
* d53dc01 - Allow `each_like!({ "a": 1 }, min = 2)` (Eric Kidd, Wed Oct 11 09:02:07 2017 -0400)
* 8f864cb - Confirm that `^` and `$` are required (Eric Kidd, Wed Oct 11 08:50:22 2017 -0400)
* 9de566b - Rename `something_like!` and `array_like!` to match JS (Eric Kidd, Wed Oct 11 08:39:06 2017 -0400)
* 01f09be - [BUG] pact_matching: Parse JSON paths with `_` (Eric Kidd, Tue Oct 10 08:55:44 2017 -0400)
* 76b9cd7 - Add helper methods for building popular properties (Eric Kidd, Tue Oct 10 06:42:01 2017 -0400)
* f0e2522 - Add `MockServer::path` and update examples (Eric Kidd, Mon Oct 9 16:43:53 2017 -0400)
* 6d9bb6a - Add macros for `term!` and other special rules (Eric Kidd, Mon Oct 9 16:19:53 2017 -0400)
* 25ad54b - Convert builders to use `StringPattern` (Eric Kidd, Mon Oct 9 12:00:05 2017 -0400)
* 86efdc0 - Add a `get_defaulting` helper and break out utils (Eric Kidd, Mon Oct 9 11:48:22 2017 -0400)
* 12bd014 - Create a new `StringPattern` type (Eric Kidd, Mon Oct 9 11:16:31 2017 -0400)
* 137e349 - Fix outdated comment (Eric Kidd, Mon Oct 9 08:47:40 2017 -0400)
* da9cfda - Implement new, experimental syntax (API BREAKAGE) (Eric Kidd, Sun Oct 8 13:33:09 2017 -0400)
* eb5fcd6 - Fix warnings by removing unused `p-macro` (Eric Kidd, Fri Oct 6 07:56:44 2017 -0400)
* e6ad973 - Reorganize `matchables` code (Eric Kidd, Fri Oct 6 07:55:24 2017 -0400)
* d6f867b - Replace `Term` with open-ended `Matchable` trait (Eric Kidd, Fri Oct 6 06:56:02 2017 -0400)
* 23f0a26 - Create a Rust version of `Term` (Eric Kidd, Thu Oct 5 07:49:12 2017 -0400)
* 3f42e50 - Implement `JsonPattern` w/o matcher support (Eric Kidd, Wed Oct 4 13:40:09 2017 -0400)
* 182b0a4 - Add a `body_present` function that handles boilerplate (Eric Kidd, Tue Oct 3 10:42:55 2017 -0400)
* 0bd43a3 - Get rid of `hashmap!` in public APIs (Eric Kidd, Tue Oct 3 09:19:53 2017 -0400)
* 4e9f6a6 - Replace `s!` with `Into<String>` (Eric Kidd, Tue Oct 3 07:18:02 2017 -0400)
* 359f1f5 - Re-export OptionalBody (Eric Kidd, Tue Oct 3 07:17:01 2017 -0400)
* 487a0bd - pact_consumer: Move doctest to tests.rs temporarily (Eric Kidd, Tue Oct 3 06:33:54 2017 -0400)
* 06e92e5 - Refer to local libs using version+paths (Eric Kidd, Tue Oct 3 06:22:23 2017 -0400)
* 4c7c66a - Missed updating the crate versions for pact_consumer (Ronald Holshausen, Wed May 17 12:45:06 2017 +1000)
* 7afd258 - Update all the cargo manifest versions and commit the cargo lock files (Ronald Holshausen, Wed May 17 10:37:44 2017 +1000)
* be8c299 - Cleanup unused BTreeMap usages and use remote pact dependencies (Anthony Damtsis, Mon May 15 17:09:14 2017 +1000)
* a59fb98 - Migrate remaining pact modules over to serde (Anthony Damtsis, Mon May 15 16:59:04 2017 +1000)
* c988180 - bump version to 0.2.1 (Ronald Holshausen, Sun Oct 9 16:55:35 2016 +1100)

# 0.2.0 - V2 implementation

* 2eb38fc - update the consumer library versions for the V2 branch (Ronald Holshausen, Sun Oct 9 16:50:03 2016 +1100)
* e3eebbd -  update projects to use the published pact mock server library (Ronald Holshausen, Sun Oct 9 16:36:25 2016 +1100)
* 770010a - update projects to use the published pact matching lib (Ronald Holshausen, Sun Oct 9 16:25:15 2016 +1100)
* 574e072 - upadte versions for V2 branch and fix an issue with loading JSON bodies encoded as a string (Ronald Holshausen, Sun Oct 9 15:31:57 2016 +1100)
* 6d581d5 - bump version to 0.1.1 (Ronald Holshausen, Sat Oct 8 17:59:33 2016 +1100)

# 0.1.0 - V1.1 specification implementation

* dae5d42 - correct the doc link (Ronald Holshausen, Sat Oct 8 17:55:15 2016 +1100)
* 16b99b5 - V1.1 spec changes (Ronald Holshausen, Sat Oct 8 17:53:53 2016 +1100)
* 1f3f3f1 - correct the versions of the inter-dependent projects as they were causing the build to fail (Ronald Holshausen, Sat Oct 8 17:41:57 2016 +1100)
* a46dabb - update all references to V1 spec after merge (Ronald Holshausen, Sat Oct 8 16:20:51 2016 +1100)
* 548c5aa - bump version to 0.0.1 (Ronald Holshausen, Mon Sep 26 23:16:50 2016 +1000)
* d80e899 - release script needs to be executable (Ronald Holshausen, Mon Sep 26 23:14:15 2016 +1000)

# 0.0.0 - First Release
