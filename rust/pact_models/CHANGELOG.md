To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 1.3.5 - Maintenance Release

* e3d23beb - chore(v2-matching-engine): Support XML namespaces with attributes (Ronald Holshausen, Fri Jul 25 15:38:20 2025 +1000)
* f95f7e35 - chore(v2-matching-engine): Fixes ported over from Pact-JVM (Ronald Holshausen, Wed Jul 23 16:31:50 2025 +1000)
* d9b2d33a - bump version to 1.3.5 (Ronald Holshausen, Thu Jun 26 09:45:42 2025 +1000)

# 1.3.4 - Bugfix Release

* 72da979e - feat: Allow atLeast and atMost first in expression (tien.xuan.vo, Wed Jun 25 01:22:58 2025 +0700)
* de9ec164 - Merge pull request #496 from tienvx/add-test-case-for-atLeast-atMost-at-beginning (Ronald Holshausen, Thu Jun 19 09:50:28 2025 +1000)
* bde2c24b - test: Add test for atLeast, atMost at the beginning of the matching expression (tien.xuan.vo, Thu Jun 19 06:11:18 2025 +0700)
* 262fbe8e - feat: Support fromProviderState generator in regex matching expression (tien.xuan.vo, Wed Jun 18 23:49:26 2025 +0700)
* 61667972 - bump version to 1.3.4 (Ronald Holshausen, Wed Jun 11 13:12:52 2025 +1000)

# 1.3.3 - Fix for Alpine

* 1d18d3f5 - fix: Onig crate needs default-features = false so it compiles on Alpine (Ronald Holshausen, Wed Jun 11 12:35:06 2025 +1000)
* 03ffd616 - bump version to 1.3.3 (Ronald Holshausen, Wed Jun 11 10:12:59 2025 +1000)

# 1.3.2 - Maintenance Release

* 82d078d9 - fix: Upgrade onig to 6.5.1 as the Match Whole String for is_match has been released (Ronald Holshausen, Wed Jun 11 10:00:52 2025 +1000)
* 6a613889 - bump version to 1.3.2 (Ronald Holshausen, Tue May 20 14:48:26 2025 +1000)

# 1.3.1 - Supports generators with XML

* 84085856 - chore(pact-models): Update dependencies (Ronald Holshausen, Tue May 20 11:40:04 2025 +1000)
* 035bfd72 - test: Test apply XML generator to elements located too deep (tien.xuan.vo, Tue Mar 18 00:53:17 2025 +0700)
* 3c0c5a09 - test: Test apply XML generator to elements in the middle (tien.xuan.vo, Tue Mar 18 00:52:51 2025 +0700)
* 25025d8a - test: Test apply XML generator without any generators (tien.xuan.vo, Tue Mar 18 00:50:56 2025 +0700)
* 9caa1d26 - perf: Improve performance of XML attribute and text generating (tien.xuan.vo, Mon Mar 17 23:54:04 2025 +0700)
* 67c9f20f - test: Test XML generator with escaping data (tien.xuan.vo, Sun Mar 16 00:57:12 2025 +0700)
* 1eead4e3 - test: Test XML generator with comment (tien.xuan.vo, Sun Mar 16 00:14:22 2025 +0700)
* b30ae400 - test: Test XML generator with unicode (tien.xuan.vo, Sat Mar 15 17:20:59 2025 +0700)
* 662950ac - fix(test): Fix wrong content type (tien.xuan.vo, Sat Mar 15 17:19:58 2025 +0700)
* 8084b802 - fix: Handle wildcard in XML generator key (tien.xuan.vo, Fri Mar 14 22:29:27 2025 +0700)
* 3c7a7b33 - feat: Implement generators for XML (tien.xuan.vo, Mon Feb 10 22:43:08 2025 +0700)
* 4eb429de - fix: kiss_xml should be required with the XML feature (Ronald Holshausen, Tue Mar 25 16:51:39 2025 +1100)
* a545c105 - chore(pact-models): Correct changelog (Ronald Holshausen, Tue Mar 25 16:14:59 2025 +1100)
* 45ac2c65 - bump version to 1.3.1 (Ronald Holshausen, Tue Mar 25 16:10:29 2025 +1100)

# 1.3.0 - Support V2 matching engine

* fd3aed0f - chore(pact-models): Upgrade pact models crate to Rust 2024 edition (Ronald Holshausen, Tue Mar 25 16:06:21 2025 +1100)
* 0ed75202 - chore(pact-models): Upgrade pact models crate to Rust 2024 edition (Ronald Holshausen, Tue Mar 25 16:02:47 2025 +1100)
* 306af77a - chore(pact-models): Update dependencies (Ronald Holshausen, Tue Mar 25 15:38:22 2025 +1100)
* ec67cd76 - feat: Handle min/max type matchers in XML elements (Ronald Holshausen, Tue Mar 25 11:53:17 2025 +1100)
* a8eeed06 - feat: Handle type matchers in XML elements (Ronald Holshausen, Tue Mar 25 11:11:12 2025 +1100)
* b43503a4 - feat: Handle multiple matching rules (Ronald Holshausen, Tue Mar 25 09:16:58 2025 +1100)
* 8664b516 - fix: resolving xml paths now filters nodes when an index is provided (Ronald Holshausen, Mon Mar 24 09:45:03 2025 +1100)
* fb7dcdc3 - refactor: Move all the execution functions into an interpreter module (Ronald Holshausen, Fri Mar 21 10:53:29 2025 +1100)
* b07d7aec - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Wed Mar 19 11:22:43 2025 +1100)
* 70bfc52d - feat: Added a function fetch XML node using a path (Ronald Holshausen, Tue Mar 18 14:10:21 2025 +1100)
* befe72ac - feat: Add functions to match a DocPath against an XML document (Ronald Holshausen, Tue Mar 18 11:06:59 2025 +1100)
* dc333aa9 - fix: Fix test wrong method (tien.xuan.vo, Tue Mar 18 00:54:48 2025 +0700)
* ad9b555b - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Fri Mar 14 11:42:13 2025 +1100)
* 63c97865 - chore: Correct failing tests after dependency update (Ronald Holshausen, Fri Mar 14 11:10:58 2025 +1100)
* 242e7b8d - bump version to 1.2.8 (Ronald Holshausen, Fri Mar 14 10:46:51 2025 +1100)

# 1.2.7 - Maintenance Release

* c6346594 - fix: join_field should only replace * and not [*] parts (Ronald Holshausen, Fri Mar 14 10:38:08 2025 +1100)
* 92394a40 - feat: enhancements backported from the V2 matching engine branch (Ronald Holshausen, Fri Mar 14 10:21:40 2025 +1100)
* a63fb583 - fix: Path weight calculation did not take [*] fragments into account (Ronald Holshausen, Fri Mar 14 09:53:14 2025 +1100)
* bcf2934b - feat: add additional conversions for docpath (Ronald Holshausen, Tue Mar 11 11:26:28 2025 +1100)
* a4ecc0b6 - refactor: Extract the DocPath resolving functions from the generators to the json_utils module (Ronald Holshausen, Thu Feb 20 14:31:59 2025 +1100)
* 5fdfd023 - chore: Upgrade all the tracing crates to the latest (Ronald Holshausen, Fri Dec 13 10:54:51 2024 +1100)
* 74df5d1d - fix: Remove duplicated timezone entries from timezone_db (Ronald Holshausen, Wed Dec 11 15:45:10 2024 +1100)
* c266f8ea - bump version to 1.2.7 (Ronald Holshausen, Wed Dec 11 15:22:25 2024 +1100)

# 1.2.6 - Maintenance Release

* 707752ad - feat: Support generators for form urlencoded (tien.xuan.vo, Mon Dec 2 10:10:43 2024 +0700)
* f3918b61 - Merge pull request #464 from tienvx/fix-cant-compare-each-key-each-value-matching-rules (Ronald Holshausen, Wed Nov 13 11:07:10 2024 +1100)
* 7465b6da - bump version to 1.2.6 (Ronald Holshausen, Wed Nov 13 10:55:04 2024 +1100)
* c2793764 - test: Add test for eachKey and eachValue matching rules comparation (tien.xuan.vo, Fri Sep 6 11:40:19 2024 +0700)
* 691ddae5 - fix: Fix can't compare 2 eachKey matching rules, 2 eachValue matching rules (tien.xuan.vo, Fri Sep 6 11:15:59 2024 +0700)

# 1.2.5 - Maintenance Release

* e485190f - chore(pact-models): Updated dependencies (Ronald Holshausen, Wed Nov 13 10:29:10 2024 +1100)
* 234cbaf0 - chore(pact-models): Upgraded rand_regex to 0.17.0 (Ronald Holshausen, Wed Nov 13 10:25:55 2024 +1100)
* 04f5d1ac - chore(pact-models): Updated dependencies (Ronald Holshausen, Wed Nov 13 09:58:06 2024 +1100)
* 0aaa3fb5 - chore(pact-models): Upgrade ariadne to 0.5.0 (Ronald Holshausen, Wed Nov 13 09:49:21 2024 +1100)
* c6aff26f - feat: Add join_index to DocPath (Ronald Holshausen, Fri Nov 1 17:03:10 2024 +1100)
* 9cc38a2f - chore: do not write empty matching rules to the Pact file (Ronald Holshausen, Tue Aug 27 11:23:43 2024 +1000)
* 60ce3bc6 - bump version to 1.2.5 (Ronald Holshausen, Tue Aug 13 11:45:30 2024 +1000)

# 1.2.4 - Bugfix Release

* 50fc022c - fix(pact_models): Provider state values may be passed in under a "providerState" key (Ronald Holshausen, Thu Aug 8 11:53:23 2024 +1000)
* 8387b9e1 - fix(pact_models): ProviderStateGenerator name was not correct (Ronald Holshausen, Tue Aug 6 11:49:19 2024 +1000)
* 42911cfd - bump version to 1.2.4 (Ronald Holshausen, Tue Aug 6 10:54:23 2024 +1000)

# 1.2.3 - Support for provider state injected values in plugins

* 033a50ab - feat: Update the matching rule expression parser to support values from provider states (Ronald Holshausen, Fri Aug 2 10:06:52 2024 +1000)
* 2ca9a9ae - chore(pact_models): Improve the matching rule expression parser error messages (Ronald Holshausen, Thu Aug 1 16:07:31 2024 +1000)
* 77814aef - fix(pact_models):  fix: when writing pact specs in V2 format, header matchers must be pluralised #449 (Ronald Holshausen, Wed Jul 31 15:08:00 2024 +1000)
* 3c96c31b - chore: Upgrade pact_mock_server to 1.2.9 (Ronald Holshausen, Wed Jul 17 14:54:40 2024 +1000)
* 57f05775 - bump version to 1.2.3 (Ronald Holshausen, Wed Jul 17 11:10:27 2024 +1000)

# 1.2.2 - Maintenance Release

* 2e2580f6 - feat(pact_consumer): Improve the mock server error output (Ronald Holshausen, Wed Jul 10 15:23:48 2024 +1000)
* efa1295a - feat(pact_verifier): Allow provider state generator to fall back to the provider state parameters #441 (Ronald Holshausen, Tue Jul 9 17:27:09 2024 +1000)
* 86704a82 - bump version to 1.2.2 (Ronald Holshausen, Wed Jun 12 15:17:57 2024 +1000)

# 1.2.1 - Bugfix Release

* ab80cd2b - fix: merge pact duplicates with diff desc/same prov state fix: merge message / sync pacts with no provider state (Yousaf Nabi, Sat Jun 8 23:48:30 2024 +0100)
* 19957df5 - chore: Lock tracing crates to the same version via cargo patch block (Ronald Holshausen, Tue Jun 11 16:00:04 2024 +1000)
* 37d65469 - chore: add some debug logs + fix compiler warning (Ronald Holshausen, Tue Jun 11 15:12:26 2024 +1000)
* b0fd7e2f - chore: refactor to preserve existing error messages (Yousaf Nabi, Wed May 29 03:31:11 2024 +0100)
* 7ca9e223 - bump version to 1.2.1 (Ronald Holshausen, Tue Apr 23 10:23:09 2024 +1000)

# 1.2.0 - Support query parameters with no values

* f20bb77d - chore(pact_models): Bump minor version (Ronald Holshausen, Tue Apr 23 10:04:56 2024 +1000)
* e12cb9f0 - Merge pull request #410 from kageru/fix-user-agent-headers (Ronald Holshausen, Tue Apr 23 09:43:11 2024 +1000)
* 4e1fca98 - feat: Add extra test cases for query generation (Ronald Holshausen, Mon Apr 22 11:28:02 2024 +1000)
* c3128a6d - feat: Support optional query parameter values (where there is only a name) (Ronald Holshausen, Mon Apr 22 10:36:05 2024 +1000)
* 6a617b71 - parse user-agent as single value headers (kageru, Wed Apr 17 14:14:27 2024 +0200)
* 74f43bdf - chore(pact_models): clean up some deprecation warnings (Ronald Holshausen, Wed Mar 20 15:34:27 2024 +1100)
* 2b59bffe - chore(pact_models): clean up some deprecation warnings (Ronald Holshausen, Tue Mar 19 10:03:38 2024 +1100)
* 712bc797 - chore: update examples in matchers to JSON test (Ronald Holshausen, Sun Feb 18 03:54:56 2024 +1100)
* 88581695 - bump version to 1.1.19 (Ronald Holshausen, Wed Feb 7 10:49:24 2024 +1100)

# 1.1.18 - Maintenance Release

* c7a086d6 - feat(models): add merge method (JP-Ellis, Tue Feb 6 12:45:59 2024 +1100)
* 7222cd27 - fix: Min and Max type matchers were not able to parse their integration form (Ronald Holshausen, Wed Feb 7 09:45:45 2024 +1100)
* 1d463aae - bump version to 1.1.18 (Ronald Holshausen, Sat Jan 20 13:45:58 2024 +1100)

# 1.1.17 - Bugfix Release

* 57e8d092 - fix: Implemented missing atLeast and atMost options with matching rule definitions (Ronald Holshausen, Sat Jan 20 13:29:30 2024 +1100)
* e9f7c06f - fix:  Regex generator panics if the regex has any anchors #311 (Ronald Holshausen, Fri Jan 19 19:33:50 2024 +1100)
* 69c1cbb6 - bump version to 1.1.17 (Ronald Holshausen, Fri Jan 19 14:46:18 2024 +1100)

# 1.1.16 - Bugfix Release

* 0f5f563f - fix: regression - prevous matching rules change broke specification tests #359 (Ronald Holshausen, Fri Jan 19 14:41:37 2024 +1100)
* a77c9eb0 - bump version to 1.1.16 (Ronald Holshausen, Fri Jan 19 14:13:25 2024 +1100)

# 1.1.15 - Bugfix Release

* 1d9034aa - fix: regression - generators must support query parameters in q[]= form #359 (Ronald Holshausen, Fri Jan 19 14:03:23 2024 +1100)
* 01903998 - fix: regression - matching rules must support query parameters in q[]= form #359 (Ronald Holshausen, Fri Jan 19 13:03:21 2024 +1100)
* 512ed87b - fix: Error where interactions failed to load where being silently ignored #359 (Ronald Holshausen, Fri Jan 19 12:49:01 2024 +1100)
* 7d615579 - docs: Fix date example value (tien.xuan.vo, Wed Jan 17 12:25:54 2024 +0700)
* ab1d496f - fix: Add set-cookie header to the list of single value headers #353 (Ronald Holshausen, Wed Jan 17 13:46:49 2024 +1100)
* 54022150 - bump version to 1.1.15 (Ronald Holshausen, Tue Jan 16 10:24:00 2024 +1100)

# 1.1.14 - Bugfix Release

* d94c7827 - fix: Support V2 matching rule header/query paths in encoded format #355 (Ronald Holshausen, Tue Jan 16 09:19:21 2024 +1100)
* e7e0c278 - bump version to 1.1.14 (Ronald Holshausen, Tue Jan 16 04:25:00 2024 +1100)

# 1.1.13 - Bugfix Release

* 6e5504cd - fix: matching rules and generators for headers could be written to JSON form incorrectly #355 (Ronald Holshausen, Tue Jan 16 03:44:20 2024 +1100)
* d8688c17 - bump version to 1.1.13 (Ronald Holshausen, Mon Nov 13 17:14:24 2023 +1100)

# 1.1.12 - Bugfix Release

* 07a1e7c2 - fix: Fallback to always generate a value to fix 'Could not generate a random TYPE from null' (tien.xuan.vo, Wed Nov 8 08:12:29 2023 +0700)
* 3af7357f - feat: Force generators works regardless of value's type (tien.xuan.vo, Fri Nov 3 09:58:17 2023 +0700)
* 10174b28 - chore: fix clippy errors (Ronald Holshausen, Fri Oct 6 09:54:49 2023 +1100)
* c5c49849 - bump version to 1.1.12 (Ronald Holshausen, Mon Aug 7 12:04:15 2023 +1000)

# 1.1.11 - Maintenance Release

* 86da370f - feat(pact_models): Add functions for dealing with different header key case with V4 req/res models (Ronald Holshausen, Mon Aug 7 11:49:22 2023 +1000)
* 2b97c737 - bump version to 1.1.11 (Ronald Holshausen, Fri Aug 4 15:43:39 2023 +1000)

# 1.1.10 - Bugfix Release

* a03fc5f0 - fix: V3 message binary content was not being base64 decoded correctly when loaded from a Pact file (Ronald Holshausen, Fri Aug 4 15:41:38 2023 +1000)
* 7e108383 - fix(pact_models): Implement add_header for V3 Message (Ronald Holshausen, Wed Aug 2 16:42:35 2023 +1000)
* 28a9df9e - bump version to 1.1.10 (Ronald Holshausen, Thu Jul 27 10:05:20 2023 +1000)

# 1.1.9 - Bugfix Release

* 2e45e223 - fix: Update matching error messages to be in line with the compatibility-suite (Ronald Holshausen, Tue Jul 25 17:42:03 2023 +1000)
* e729e348 - chore: MockServerURL generator should look for both url and href keys (Ronald Holshausen, Tue Jul 25 14:39:19 2023 +1000)
* 8a8ae8c6 - feat(pact_models): Added function to convert a DocPath into a JSON pointer (Ronald Holshausen, Tue Jul 25 11:16:59 2023 +1000)
* 1deca59a - chore: Upgrade pact_models to 1.1.8 (Ronald Holshausen, Mon Jul 10 16:15:43 2023 +1000)
* c78ea504 - bump version to 1.1.9 (Ronald Holshausen, Mon Jul 10 15:59:15 2023 +1000)

# 1.1.8 - Bugfix Release

* 4b4e7105 - fix(pact_models): MatchingRule::from_json shoud support integration format (Ronald Holshausen, Mon Jul 10 13:56:13 2023 +1000)
* a6049684 - refactor: Update MIME multipart parser to a supported lib + make implementation inline with Pact-JVM (Ronald Holshausen, Fri Jul 7 15:42:02 2023 +1000)
* fa93d6c7 - bump version to 1.1.8 (Ronald Holshausen, Thu Jul 6 10:20:20 2023 +1000)

# 1.1.7 - Bugfix Release

* 7dd89384 - feat: Add application/x-www-form-urlencoded to the known content types (Ronald Holshausen, Wed Jul 5 16:58:21 2023 +1000)
* f6ba3b2a - fix: When writing V4 format, correct the content type set on the body (Ronald Holshausen, Wed Jul 5 16:57:40 2023 +1000)
* 52d6bfab - feat: Add method to DocPath to return a lower-case copy of the path (Ronald Holshausen, Wed Jun 28 15:20:39 2023 +1000)
* 59cbaad0 - bump version to 1.1.7 (Ronald Holshausen, Thu Jun 22 15:34:04 2023 +1000)

# 1.1.6 - Bugfix Release

* 727ea824 - fix: Support string escape sequences in matching definitions #283 (Ronald Holshausen, Wed Jun 21 14:55:42 2023 +1000)
* 3cd0cbbb - chore: Upgrade Logos to 0.13.0 (Ronald Holshausen, Tue Jun 20 16:54:05 2023 +1000)
* 8d4de61c - feat(pact_models): Add path method to build the string expression from the path tokens (Ronald Holshausen, Tue Jun 20 14:37:06 2023 +1000)
* 27bc02c9 - fix(pact_models): DocPath.parent was creating incorrect paths when the parent is a * (Ronald Holshausen, Tue Jun 20 10:41:18 2023 +1000)
* b33f884d - bump version to 1.1.6 (Ronald Holshausen, Fri Jun 16 16:15:11 2023 +1000)

# 1.1.5 - Add crate features for date/time and XML support

* 161f1b82 - chore: cleanup deprecation warnings (Ronald Holshausen, Fri Jun 16 16:11:18 2023 +1000)
* c5887670 - feat(pact-models): Add crate features for date/time and XML support #290 (Ronald Holshausen, Fri Jun 16 16:00:04 2023 +1000)
* 2477729f - chore: reduce log entry to debug level as it makes no sense as a warning (Ronald Holshausen, Mon Jun 5 15:01:45 2023 +1000)
* b7d949ed - bump version to 1.1.5 (Ronald Holshausen, Thu Jun 1 10:17:41 2023 +1000)

# 1.1.4 - Bugfix Release

* 9d3205a0 - fix: Support fraction of seconds with more then 3 digits #279 (Ronald Holshausen, Wed May 31 14:40:41 2023 +1000)
* f001eba2 - bump version to 1.1.4 (Ronald Holshausen, Mon May 29 15:03:44 2023 +1000)

# 1.1.3 - Bugfix Release

* 743b1823 - fix: MockServerURL generator was fetching the incorrect field from the test context (Ronald Holshausen, Mon May 29 14:58:19 2023 +1000)
* 2eeecc4a - bump version to 1.1.3 (Ronald Holshausen, Mon May 15 13:25:58 2023 +1000)

# 1.1.2 - Removed auto-generated keys and updates all trait methods so they can be used in an FFI context

* f8f0e773 - fix: Add RefUnwindSafe trait bound to all Pact and Interaction trait methods so they can be used in an FFI context (Ronald Holshausen, Mon May 15 12:37:16 2023 +1000)
* cf55b3c5 - fix: do not auto-generate the interaction key if not set #264 (Ronald Holshausen, Mon May 8 11:10:20 2023 +1000)
* 5eed834a - feat: Update date validation to check for leap years (Ronald Holshausen, Tue May 2 14:22:10 2023 +1000)
* 1a33368f - bump version to 1.0.14 (Ronald Holshausen, Tue Apr 18 12:51:22 2023 +1000)

# 1.0.13 - Maintenance Release

* 473d3167 - chore: remove dbg statements (Ronald Holshausen, Mon Apr 17 11:52:12 2023 +1000)
* 2eed4e60 - bump version to 1.0.13 (Ronald Holshausen, Mon Apr 17 10:16:45 2023 +1000)

# 1.0.12 - Fixes hash function for generators

* a81b2df3 - chore: fix for diabled generators hash test (Ronald Holshausen, Mon Apr 17 09:25:49 2023 +1000)
* 04ddccac - chore: disable hash_test_for_generators as it is failing on CI (Ronald Holshausen, Fri Apr 14 17:37:25 2023 +1000)
* afbd35d1 - chore: switch generator tests to FxHasher as they are failing on CI (Ronald Holshausen, Fri Apr 14 17:22:22 2023 +1000)
* 848b1943 - bump version to 1.0.12 (Ronald Holshausen, Fri Apr 14 17:06:03 2023 +1000)

# 1.0.11 - Bugfix Release

* d3cd2357 - fix(V4): corrected all the hash functions for all V4 models (Ronald Holshausen, Fri Apr 14 16:51:01 2023 +1000)
* 867936d6 - fix: V4 models were not including the key in the implementation of equals (Ronald Holshausen, Fri Apr 14 11:16:34 2023 +1000)
* 67123cb0 - bump version to 1.0.11 (Ronald Holshausen, Thu Apr 13 15:27:32 2023 +1000)

# 1.0.10 - Bugfix Release

* 10239f7c - fix(V4): when generating the interaction key, treat header keys in a case-insensitive manner (Ronald Holshausen, Thu Apr 13 15:02:29 2023 +1000)
* 6f70f30a - fix: exclude headers from the pact conflict check (Ronald Holshausen, Wed Mar 15 15:23:19 2023 +1100)
* 2b5c53f6 - bump version to 1.0.10 (Ronald Holshausen, Wed Mar 15 14:23:05 2023 +1100)

# 1.0.9 - Bugfix Release

* 97684ade - fix: Correctly deal with headers when the value is a string (Ronald Holshausen, Wed Mar 15 14:12:24 2023 +1100)
* 55748ea0 - bump version to 1.0.9 (Ronald Holshausen, Wed Mar 15 12:15:43 2023 +1100)

# 1.0.8 - Bugfix Release

* 315e6aee - fix(V4): Bodies where the content attribute is NULL should be NULL bodies (Ronald Holshausen, Wed Mar 15 12:01:28 2023 +1100)
* 86d4ee08 - bump version to 1.0.8 (Ronald Holshausen, Wed Mar 15 11:32:50 2023 +1100)

# 1.0.7 - Bugfix Release

* eeb256db - fix(V4): Bodies specified as a single empty JSON string should be mapped to an empty body (Ronald Holshausen, Wed Mar 15 11:22:08 2023 +1100)
* 20d9efd1 - bump version to 1.0.7 (Ronald Holshausen, Tue Mar 14 17:10:26 2023 +1100)
* 1e51e8a9 - Revert "bump version to 1.0.7" (Ronald Holshausen, Tue Mar 14 17:09:08 2023 +1100)
* dbb750b1 - bump version to 1.0.7 (Ronald Holshausen, Tue Mar 14 16:46:19 2023 +1100)

# 1.0.6 - Maintenance Release

* fbe6ebe2 - chore: set pact_models version to 1.0.6 as 1.0.5 was yanked (Ronald Holshausen, Tue Mar 14 16:41:56 2023 +1100)
* f04a3273 - feat: add a simple header parser to pact_models #259 (Ronald Holshausen, Tue Mar 14 16:38:54 2023 +1100)
* 43bb0d50 - Bump indextree, use append_value() (Yoni Feigelson, Sun Mar 12 11:32:57 2023 +0200)
* 80ec8f1a - Revert "fix: Add UnwindSafe trait bound to all Pact and Interaction trait methods so they can be used in a FFI context" (Ronald Holshausen, Tue Feb 28 16:46:43 2023 +1100)
* 029d34a8 - Revert "update changelog for release 1.0.5" (Ronald Holshausen, Tue Feb 28 16:46:32 2023 +1100)
* 4d12d790 - Revert "bump version to 1.0.6" (Ronald Holshausen, Tue Feb 28 16:46:19 2023 +1100)
* 3b82429f - Revert "fix: Bumping minor version as the last change broke everything using the traits" (Ronald Holshausen, Tue Feb 28 16:46:09 2023 +1100)
* 36685717 - Revert "update changelog for release 1.1.0" (Ronald Holshausen, Tue Feb 28 16:45:56 2023 +1100)
* 2bdc2736 - Revert "bump version to 1.1.1" (Ronald Holshausen, Tue Feb 28 16:45:44 2023 +1100)
* 5c802419 - Revert "fix: UnwindSafe trait bound was missing on the read/write functions" (Ronald Holshausen, Tue Feb 28 16:45:31 2023 +1100)
* add61a75 - Revert "update changelog for release 1.1.1" (Ronald Holshausen, Tue Feb 28 16:45:20 2023 +1100)
* 8ad8e409 - Revert "bump version to 1.1.2" (Ronald Holshausen, Tue Feb 28 16:45:01 2023 +1100)
* 5712b511 - bump version to 1.1.2 (Ronald Holshausen, Tue Feb 28 16:30:50 2023 +1100)
* f3b03ba8 - update changelog for release 1.1.1 (Ronald Holshausen, Tue Feb 28 16:29:43 2023 +1100)
* 019a7ea1 - fix: UnwindSafe trait bound was missing on the read/write functions (Ronald Holshausen, Tue Feb 28 16:28:29 2023 +1100)
* de34e8e4 - bump version to 1.1.1 (Ronald Holshausen, Tue Feb 28 16:14:14 2023 +1100)
* fb1d9385 - update changelog for release 1.1.0 (Ronald Holshausen, Tue Feb 28 16:08:29 2023 +1100)
* 5e0307cb - fix: Bumping minor version as the last change broke everything using the traits (Ronald Holshausen, Tue Feb 28 16:06:45 2023 +1100)
* b3140d94 - bump version to 1.0.6 (Ronald Holshausen, Tue Feb 28 15:23:07 2023 +1100)
* 208e3133 - update changelog for release 1.0.5 (Ronald Holshausen, Tue Feb 28 15:20:48 2023 +1100)
* b5ab945a - fix: Add UnwindSafe trait bound to all Pact and Interaction trait methods so they can be used in a FFI context (Ronald Holshausen, Tue Feb 28 15:19:04 2023 +1100)
* 1c45b63c - fix: add a test to reflect behaviour as per V4 spec (Ronald Holshausen, Mon Jan 30 15:22:02 2023 +1100)
* ba24b0a8 - fix: correct parsing of JSON encoded bodies as per V4 spec (Ronald Holshausen, Mon Jan 30 15:19:38 2023 +1100)
* 68aad6e9 - bump version to 1.0.5 (Ronald Holshausen, Wed Jan 11 14:26:02 2023 +1100)

# 1.0.4 - Bugfix Release

* 155dae40 - fix: Support RequestResponsePact loading from V4 formatted JSON #246 (Ronald Holshausen, Wed Jan 11 12:32:01 2023 +1100)
* 4c04cb65 - fix: sort the header and query parameter keys when writing the pact #246 (Ronald Holshausen, Wed Jan 11 10:39:36 2023 +1100)
* 1cbd7fc6 - bump version to 1.0.4 (Ronald Holshausen, Thu Dec 22 15:00:32 2022 +1100)

# 1.0.3 - Bugfix Release

* 64d500b0 - fix: Message pact was not loading the IDs from the Pact Broker #239 (Ronald Holshausen, Thu Dec 22 14:24:26 2022 +1100)
* a359c22f - chore: enable generator doc test (Ronald Holshausen, Mon Dec 12 11:08:30 2022 +1100)
* 9038c04f - bump version to 1.0.3 (Ronald Holshausen, Fri Dec 9 12:28:39 2022 +1100)

# 1.0.2 - Bugfix Release

* f84adc7a - fix: Metadata was missing from the generator categories (Ronald Holshausen, Fri Dec 9 11:00:34 2022 +1100)
* f9dd7a4e - Fix: date and time matchers were not being persisted as per the spec (Ronald Holshausen, Thu Dec 1 12:25:12 2022 +1100)
* 7e5add7e - bump version to 1.0.2 (Ronald Holshausen, Mon Nov 28 13:27:43 2022 +1100)

# 1.0.1 - Maintenance Release

* 43a8cae1 - chore: clean up deprecation warnings (Ronald Holshausen, Mon Nov 28 13:19:31 2022 +1100)
* 19cc4ca3 - chore: Update dependencies (Ronald Holshausen, Mon Nov 28 12:12:18 2022 +1100)
* e21d3454 - feat: add FFI function to parse JSON to a Pact model (Ronald Holshausen, Fri Nov 11 17:00:36 2022 +1100)
* 66e84e9d - bump version to 1.0.1 (Ronald Holshausen, Mon Nov 7 10:27:49 2022 +1100)

# 1.0.0 - Maintenance Release

* f91dc00d - fix: try loosen dependencies to fix dependency cycle issue (Ronald Holshausen, Mon Nov 7 10:15:19 2022 +1100)
* 87430af9 - chore: set onig to the crate version (Ronald Holshausen, Fri Nov 4 16:41:32 2022 +1100)
* 0bd3f51b - bump version to 0.4.7 (Ronald Holshausen, Fri Nov 4 15:50:52 2022 +1100)

# 0.4.6 - Maintenance Release

* 6ad00a5d - fix: Update onig to latest master to fix  Regex Matcher Fails On Valid Inputs #214 (Ronald Holshausen, Fri Nov 4 15:23:50 2022 +1100)
* ac4fe73f - chore: fix to release scripts (Ronald Holshausen, Wed Sep 7 10:51:01 2022 +1000)
* bcdc6443 - bump version to 0.4.6 (Ronald Holshausen, Fri Aug 26 11:39:48 2022 +1000)

# 0.4.5 - Bugfix Release

* b6bba540 - fix(FFI): FFI passes matching rules and generators for paths etc. with a path of $ (Ronald Holshausen, Fri Aug 26 11:22:19 2022 +1000)
* a37d621e - chore: update the doc comments on matchingrules! macro (Ronald Holshausen, Wed Aug 24 17:25:04 2022 +1000)
* 46b7c1a2 - bump version to 0.4.5 (Ronald Holshausen, Thu Aug 18 13:35:04 2022 +1000)

# 0.4.4 - Updated dependencies

* bd53ad0d - chore: Upgrade dependencies (uuid, tracing) (Ronald Holshausen, Thu Aug 18 13:32:28 2022 +1000)
* 3e5acc2c - bump version to 0.4.4 (Ronald Holshausen, Thu Aug 18 13:05:45 2022 +1000)

# 0.4.3 - Bugfix Release

* 74a36a1b - fix: Matching rule parser was not handling decimal values correctly (Ronald Holshausen, Wed Aug 17 13:19:58 2022 +1000)
* cf4b52eb - chore: fix failing time expression tests after a dependency update (Ronald Holshausen, Mon Aug 15 17:09:04 2022 +1000)
* 9e0dbc26 - chore: clean up some deprecation warnings in Pact models (Ronald Holshausen, Wed Aug 10 11:38:45 2022 +1000)
* 91cb99a9 - bump version to 0.4.3 (Ronald Holshausen, Mon Aug 8 12:59:10 2022 +1000)

# 0.4.2 - Maintenance Release

* 13dc3b52 - Merge branch 'master' into feat/verifier-multiple-transports (Ronald Holshausen, Wed Aug 3 11:55:05 2022 +1000)
* 3d73e3c2 - Removed dependency on time v0.1 (Daan Oosterveld, Wed Jul 6 15:56:29 2022 +0200)
* 2ca2fe49 - fix: add function to display binary data in a meaningful way (Ronald Holshausen, Mon Aug 1 17:39:34 2022 +1000)
* 1972a74a - feat: Detect Pactbroker responses from the URL content #199 (Ronald Holshausen, Mon Jun 6 14:48:06 2022 +1000)
* 4da79d75 - chore: lock the tracing crate version (Ronald Holshausen, Mon May 9 17:04:29 2022 +1000)
* 137e3503 - bump version to 0.4.2 (Ronald Holshausen, Mon May 9 14:18:35 2022 +1000)

# 0.4.1 - switch pact_models to use tracing crate

* 9d30a441 - chore: switch pact_models to use tracing crate (Ronald Holshausen, Mon May 9 13:24:37 2022 +1000)
* 0dd9a176 - bump version to 0.4.1 (Ronald Holshausen, Fri Apr 22 12:07:23 2022 +1000)

# 0.4.0 - Updated V4 model interfaces

* 6de6c229 - feat: Add functions to calc unique key to V4 interaction trait (Ronald Holshausen, Fri Apr 22 12:00:57 2022 +1000)
* 7e3e2e18 - feat: add method to V4Pact to find an interaction by ID (Ronald Holshausen, Thu Apr 21 12:25:09 2022 +1000)
* 49640c5f - chore: minor update to release scripts (Ronald Holshausen, Wed Apr 13 15:32:46 2022 +1000)
* 97b49229 - bump version to 0.3.4 (Ronald Holshausen, Wed Apr 13 13:47:10 2022 +1000)

# 0.3.3 - Bugfix Release

* 73ae0ef0 - fix: Upgrade reqwest to 0.11.10 to resolve #156 (Ronald Holshausen, Wed Apr 13 13:31:55 2022 +1000)
* 42b1a461 - Merge branch 'master' into feat/plugin-mock-server (Ronald Holshausen, Mon Mar 21 16:01:33 2022 +1100)
* 345b0011 - feat: support mock servers provided from plugins (Ronald Holshausen, Mon Mar 21 15:59:46 2022 +1100)
* daa2c101 - feat: add mutable iteraction over Pact interactions (Ronald Holshausen, Fri Mar 18 16:55:34 2022 +1100)
* 0bc98834 - bump version to 0.3.3 (Ronald Holshausen, Fri Mar 18 16:08:30 2022 +1100)
* 0ca9f62b - update changelog for release 0.3.2 (Ronald Holshausen, Fri Mar 18 16:06:30 2022 +1100)
* 01ac989b - fix: was missing setter to set the transport with V4 interactions (Ronald Holshausen, Fri Mar 18 16:04:00 2022 +1100)
* a075f679 - bump version to 0.3.2 (Ronald Holshausen, Fri Mar 18 14:39:20 2022 +1100)
* e82a67fb - update changelog for release 0.3.1 (Ronald Holshausen, Fri Mar 18 14:37:03 2022 +1100)
* 7fd87eb9 - feat: store the transport with V4 interactions to support mockservers from plugins (Ronald Holshausen, Fri Mar 18 14:30:20 2022 +1100)
* 27e41386 - bump version to 0.3.1 (Ronald Holshausen, Fri Mar 4 11:32:56 2022 +1100)

# 0.3.2 - Maintenance Release

* 01ac989b - fix: was missing setter to set the transport with V4 interactions (Ronald Holshausen, Fri Mar 18 16:04:00 2022 +1100)
* a075f679 - bump version to 0.3.2 (Ronald Holshausen, Fri Mar 18 14:39:20 2022 +1100)

# 0.3.1 - Plugin Support

* 7fd87eb9 - feat: store the transport with V4 interactions to support mockservers from plugins (Ronald Holshausen, Fri Mar 18 14:30:20 2022 +1100)
* 27e41386 - bump version to 0.3.1 (Ronald Holshausen, Fri Mar 4 11:32:56 2022 +1100)

# 0.3.0 - Ported the date-time expressions from Pact-JVM

* 0aa55cfe - feat: wired the date-time expression parsers into the generators (Ronald Holshausen, Thu Mar 3 18:01:46 2022 +1100)
* 98b887f0 - feat: Implemented date-time expression parser (from Pact-JVM) (Ronald Holshausen, Thu Mar 3 16:34:42 2022 +1100)
* 318037a7 - feat: Implemented time part in date-time expressions (Ronald Holshausen, Thu Mar 3 16:05:01 2022 +1100)
* 66442251 - feat: Implement the base part of time expressions (Ronald Holshausen, Thu Mar 3 13:09:03 2022 +1100)
* b8ea7240 - feat: Implemented date expression parser (from Pact-JVM) (Ronald Holshausen, Tue Mar 1 14:46:57 2022 +1100)
* 2927e979 - feat: ported the date manipulation functions from Pact-JVM #180 (Ronald Holshausen, Thu Feb 17 16:14:52 2022 +1100)
* 12a7b78c - chore: bump minor version (Ronald Holshausen, Wed Feb 16 15:06:35 2022 +1100)
* c7d39ca6 - bump version to 0.2.8 (Ronald Holshausen, Mon Jan 17 10:47:48 2022 +1100)

# 0.2.7 - Bugfix Release

* c2089645 - fix: log crate version must be fixed across all crates (including plugin driver) (Ronald Holshausen, Fri Jan 14 16:10:50 2022 +1100)
* ff49f33a - chore: update docs on matching rule definitions (Ronald Holshausen, Wed Jan 5 15:07:26 2022 +1100)
* 7b23378f - feat: some matching rules should not cascade (Ronald Holshausen, Thu Dec 30 13:32:04 2021 +1100)
* 83c36db7 - bump version to 0.2.7 (Ronald Holshausen, Wed Dec 29 13:33:06 2021 +1100)

# 0.2.6 - Bugfix Release

* 7b2e8538 - fix: DocPath join needs to detect numeric values (Ronald Holshausen, Wed Dec 29 13:23:47 2021 +1100)
* e5fd165d - refactor: Move is_values_matcher logic to MatchingRule (Ronald Holshausen, Wed Dec 29 10:05:03 2021 +1100)
* 41a52319 - fix: values_matcher_defined should include EachValue matcher (Ronald Holshausen, Wed Dec 29 09:48:02 2021 +1100)
* b7f967e0 - fix: `match` arms have incompatible types (Ronald Holshausen, Thu Dec 23 17:48:20 2021 +1100)
* 39338c46 - fix: Some matching rules do not have associated configuration (Ronald Holshausen, Thu Dec 23 14:02:50 2021 +1100)
* deb30e92 - bump version to 0.2.6 (Ronald Holshausen, Thu Dec 23 11:57:34 2021 +1100)

# 0.2.5 - Bugfix Release

* e1e0b43e - fix: matching definition parser was incorrectly merging multiple definitions (Ronald Holshausen, Thu Dec 23 11:48:09 2021 +1100)
* 85bffe40 - bump version to 0.2.5 (Ronald Holshausen, Thu Dec 23 09:23:57 2021 +1100)

# 0.2.4 - Maintenance Release

* b5fd82e5 - feat: add method to DocPath to return the parent path (Ronald Holshausen, Wed Dec 22 18:06:43 2021 +1100)
* a3f74711 - fix: Docpath join was escaping * (Ronald Holshausen, Wed Dec 22 17:23:12 2021 +1100)
* cc0775e2 - bump version to 0.2.4 (Ronald Holshausen, Tue Dec 21 13:16:32 2021 +1100)

# 0.2.3 - Maintenance Release

* 481762f0 - feat: add function to detect if a string is a matching definition (Ronald Holshausen, Tue Dec 21 13:06:31 2021 +1100)
* 98e364b2 - chore: add docpath method to return a vector of strings (Ronald Holshausen, Thu Dec 16 17:16:36 2021 +1100)
* c707a8c0 - feat: add a method to join a value onto a doc path (Ronald Holshausen, Thu Dec 16 16:03:27 2021 +1100)
* 77892ab1 - bump version to 0.2.3 (Ronald Holshausen, Mon Nov 29 11:57:36 2021 +1100)

# 0.2.2 - Fixes to the matching rule parser

* 58039496 - chore: fix imports on expression parser (Ronald Holshausen, Mon Nov 29 11:49:49 2021 +1100)
* cb0f7df8 - Revert "update changelog for release 0.2.2" (Ronald Holshausen, Mon Nov 29 11:42:37 2021 +1100)
* 33a37c9d - update changelog for release 0.2.2 (Ronald Holshausen, Mon Nov 29 11:39:56 2021 +1100)
* 3207cb49 - feat: implement each key and aech value matching rule definitions (Ronald Holshausen, Wed Nov 24 14:28:42 2021 +1100)
* 2db6a46f - refactor: test_env_log has been replaced with test_log (Ronald Holshausen, Tue Nov 23 16:15:02 2021 +1100)
* d3234684 - feat: update matcher defintions to support references (Ronald Holshausen, Tue Nov 23 16:13:49 2021 +1100)
* 682df9e4 - feat: update matcher defintions to include the semver matcher (Ronald Holshausen, Tue Nov 23 14:24:02 2021 +1100)
* 20a275fb - feat: Improve the error message format for matching rule definitions (Ronald Holshausen, Mon Nov 22 15:21:57 2021 +1100)
* a859d0e1 - fix: make sure metadata entries are correctly encoded when downgrading a pact (Ronald Holshausen, Wed Nov 17 16:54:15 2021 +1100)
* d32ae2b1 - bump version to 0.2.2 (Ronald Holshausen, Tue Nov 16 10:38:04 2021 +1100)

# 0.2.1 - Update V4 models to support FFI + plugins

* 15b8f08f - feat: add functions to return mutable references to the V4 model trait (Ronald Holshausen, Tue Nov 16 10:03:03 2021 +1100)
* 7c150c8b - feat(plugins): Support message tests via FFI that use plugins (Ronald Holshausen, Wed Nov 10 17:03:49 2021 +1100)
* fa83806c - feat: add mutable methods to Pact model traits (Ronald Holshausen, Tue Nov 9 16:04:16 2021 +1100)
* 2027537d - refactor: update FFI to use V4 models internally (Ronald Holshausen, Mon Nov 8 16:44:39 2021 +1100)
* b42b7ad9 - chore: fix clippy warnings and errors (Ronald Holshausen, Wed Nov 3 15:45:29 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* dac05481 - bump version to 0.2.1 (Ronald Holshausen, Thu Oct 21 17:52:39 2021 +1100)

# 0.2.0 - Pact V4 + Plugins release

* e72d602a - chore: bump pact models to non-beta version (Ronald Holshausen, Thu Oct 21 17:47:17 2021 +1100)
* 3fb9258a - bump version to 0.2.0-beta.7 (Ronald Holshausen, Tue Oct 19 16:41:08 2021 +1100)

# 0.2.0-beta.6 - Bugfix Release

* 48a6be5f - fix: EachValue was outputting the wrong JSON (Ronald Holshausen, Tue Oct 19 16:35:17 2021 +1100)
* 021a65e6 - bump version to 0.2.0-beta.6 (Ronald Holshausen, Mon Oct 18 13:00:22 2021 +1100)

# 0.2.0-beta.5 - matching synchronous request/response messages

* 2b4b7cc3 - feat(plugins): Support matching synchronous request/response messages (Ronald Holshausen, Fri Oct 15 16:01:50 2021 +1100)
* de3e8296 - bump version to 0.2.0-beta.5 (Ronald Holshausen, Tue Oct 12 15:09:59 2021 +1100)

# 0.2.0-beta.4 - Enhancements for plugins

* a52db737 - feat: record the version of the lib that created the pact in the metadata (Ronald Holshausen, Tue Oct 12 14:55:42 2021 +1100)
* 35ff0993 - feat: record the version of the lib that created the pact in the metadata (Ronald Holshausen, Tue Oct 12 14:52:43 2021 +1100)
* 5f4a578e - refactor: add method to set content type of a body (Ronald Holshausen, Tue Oct 12 14:40:46 2021 +1100)
* d1016565 - refactor: renamed SynchronousMessages -> SynchronousMessage (Ronald Holshausen, Tue Oct 12 14:37:30 2021 +1100)
* 407cc2e5 - fix: notEmpty matching rule defintion should be applied to any primitive value (Ronald Holshausen, Thu Oct 7 14:08:02 2021 +1100)
* 48d662a8 - chore: add docs about the matching rule definition language (Ronald Holshausen, Thu Oct 7 13:29:16 2021 +1100)
* 780a0c97 - bump version to 0.2.0-beta.4 (Ronald Holshausen, Tue Oct 5 15:12:12 2021 +1100)

# 0.2.0-beta.3 - Fixes from master + Updated matching rule definitions

* d608a028 - chore: re-enable matching definition tests (Ronald Holshausen, Tue Oct 5 15:06:11 2021 +1100)
* 57ba661a - chore: fix tests after removing Deserialize, Serialize from Message (Ronald Holshausen, Tue Oct 5 14:55:58 2021 +1100)
* 9fd9e652 - feat: do no write empty comments + added consumer version to metadata (Ronald Holshausen, Thu Sep 30 17:40:56 2021 +1000)
* 5525b039 - feat(plugins): cleaned up the verfier output (Ronald Holshausen, Thu Sep 30 16:19:15 2021 +1000)
* 6d23796f - feat(plugins): support each key and each value matchers (Ronald Holshausen, Wed Sep 29 11:10:46 2021 +1000)
* 6f20282d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 28 14:51:34 2021 +1000)
* 1b994c8d - bump version to 0.1.5 (Ronald Holshausen, Tue Sep 28 13:25:36 2021 +1000)
* 7d46a966 - update changelog for release 0.1.4 (Ronald Holshausen, Tue Sep 28 13:23:07 2021 +1000)
* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 97ebf555 - feat(plugins): Updated matching rule definitions to include notEmpty and contentType (Ronald Holshausen, Wed Sep 15 12:30:01 2021 +1000)
* 21a7ede5 - feat(plugins): support matching protobuf embedded messages (Ronald Holshausen, Wed Sep 15 12:14:54 2021 +1000)
* 9bf9dc30 - feat(plugins): Add consumer message test builder + persist plugin data (Ronald Holshausen, Tue Sep 14 15:14:17 2021 +1000)
* b71dcabf - refactor(plugins): rename ContentTypeOverride -> ContentTypeHint (Ronald Holshausen, Tue Sep 14 15:08:52 2021 +1000)
* 03ebe632 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Sep 13 12:01:13 2021 +1000)
* 62db0e44 - chore: fix clippy warnings (Ronald Holshausen, Fri Sep 10 17:41:43 2021 +1000)
* 7cfe2883 - chore: fix clippy violation (Ronald Holshausen, Fri Sep 10 15:34:03 2021 +1000)
* 24d4b39a - bump version to 0.2.0-beta.3 (Ronald Holshausen, Fri Sep 10 14:16:03 2021 +1000)

# 0.1.4 - WASM support + native TLS certs

* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 62db0e44 - chore: fix clippy warnings (Ronald Holshausen, Fri Sep 10 17:41:43 2021 +1000)
* 067ded8f - feat: expose Pact models via WASM (Ronald Holshausen, Sun Sep 5 11:55:26 2021 +1000)
* 80509c01 - chore: add crate to support WASM (Ronald Holshausen, Sat Sep 4 17:32:12 2021 +1000)
* 7c12b03b - bump version to 0.1.4 (Ronald Holshausen, Sat Sep 4 15:30:35 2021 +1000)
* 8fe00acd - chore: correct release script (Ronald Holshausen, Sat Sep 4 15:26:17 2021 +1000)

# 0.2.0-beta.2 - Support for getting interaction markup from plugins

* 997c063f - chore: update release script (Ronald Holshausen, Fri Sep 10 14:08:52 2021 +1000)
* dc41e498 - chore: correct links in rust docs (Ronald Holshausen, Fri Sep 10 14:07:01 2021 +1000)
* 37978075 - feat(plugins): support getting interaction markup from plugins (Ronald Holshausen, Thu Sep 9 12:09:06 2021 +1000)
* f44ecc54 - feat(plugins): make the interaction markup type explicit (Ronald Holshausen, Thu Sep 9 11:47:32 2021 +1000)
* a89eca23 - feat(plugins): support storing interaction markup for interactions in Pact files (Ronald Holshausen, Wed Sep 8 16:39:43 2021 +1000)
* ceb1c35f - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 7 10:07:45 2021 +1000)
* 067ded8f - feat: expose Pact models via WASM (Ronald Holshausen, Sun Sep 5 11:55:26 2021 +1000)
* 80509c01 - chore: add crate to support WASM (Ronald Holshausen, Sat Sep 4 17:32:12 2021 +1000)
* 7c12b03b - bump version to 0.1.4 (Ronald Holshausen, Sat Sep 4 15:30:35 2021 +1000)
* 8fe00acd - chore: correct release script (Ronald Holshausen, Sat Sep 4 15:26:17 2021 +1000)
* 53d5d75a - update changelog for release 0.1.3 (Ronald Holshausen, Sat Sep 4 15:25:43 2021 +1000)
* 1dfe83fa - bump version to 0.2.0-beta.2 (Ronald Holshausen, Fri Sep 3 17:19:27 2021 +1000)
* 689a35f4 - chore: fix release script (Ronald Holshausen, Fri Sep 3 17:16:45 2021 +1000)
* 46135a16 - chore: add verifier FFI functions for directory, URL and Pact broker sources (Ronald Holshausen, Tue Aug 24 10:14:46 2021 +1000)

# 0.1.3 - Bugfix Release

* 46135a16 - chore: add verifier FFI functions for directory, URL and Pact broker sources (Ronald Holshausen, Tue Aug 24 10:14:46 2021 +1000)
* c2a9c5cc - bump version to 0.1.3 (Ronald Holshausen, Sun Aug 22 15:15:16 2021 +1000)

# 0.2.0-beta.1 - Support for plugins

* b77498c8 - chore: fix tests after updating plugin API (Ronald Holshausen, Fri Sep 3 16:48:18 2021 +1000)
* c0bdd359 - fix: PluginData configuration is optional (Ronald Holshausen, Thu Sep 2 15:37:01 2021 +1000)
* e8ae81b3 - refactor: matching req/res with plugins requires data from the pact and interaction (Ronald Holshausen, Thu Sep 2 11:57:50 2021 +1000)
* 474b803e - feat(V4): added nontEmpty and semver matchers (Ronald Holshausen, Tue Aug 31 11:58:18 2021 +1000)
* b9aa7ecb - feat(Plugins): allow plugins to override text/binary format of the interaction content (Ronald Holshausen, Mon Aug 30 10:48:04 2021 +1000)
* 7c8fae8b - chore: add additional tests for matching definition parser (Ronald Holshausen, Thu Aug 26 13:49:28 2021 +1000)
* 1a3c1959 - feat(plugins): moved the matching rule definition parser into the models crate (Ronald Holshausen, Wed Aug 25 17:31:17 2021 +1000)
* b40dab60 - feat(plugins): moved the matching rule definition parser into the models crate (Ronald Holshausen, Wed Aug 25 17:27:02 2021 +1000)
* c53dbd79 - bump version to 0.2.0-beta.1 (Ronald Holshausen, Mon Aug 23 13:04:02 2021 +1000)

# 0.2.0-beta.0 - Beta version supporting Pact plugins

* 72b9baaa - chore: update release script for beta versions (Ronald Holshausen, Mon Aug 23 12:57:49 2021 +1000)
* 0c5cede2 - chore: bump models crate to 0.2 (Ronald Holshausen, Mon Aug 23 12:56:14 2021 +1000)
* 75e13fd8 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Aug 23 10:33:45 2021 +1000)
* e3a2660f - chore: fix tests after updating test builders to be async (Ronald Holshausen, Fri Aug 20 12:41:10 2021 +1000)
* 779f099c - feat(plugins): Got generators from plugin working (Ronald Holshausen, Thu Aug 19 17:20:47 2021 +1000)
* 8c61ae96 - feat(plugins): Support plugins with the consumer DSL interaction/response (Ronald Holshausen, Thu Aug 19 15:24:04 2021 +1000)
* b75fea5d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Aug 18 12:27:41 2021 +1000)
* 5a235414 - feat(plugins): order the matching results as plugins mau return them in any order (Ronald Holshausen, Fri Aug 13 17:18:46 2021 +1000)
* 2662241e - feat(plugins): Call out to plugins when comparing content owned by the plugin during verification (Ronald Holshausen, Fri Aug 13 14:29:30 2021 +1000)
* bdfc6f02 - feat(plugins): Load required plugins when verifying a V4 pact (Ronald Holshausen, Wed Aug 11 14:23:54 2021 +1000)

# 0.1.2 - upgrade nom to 7.0

* 21be6bce - chore: upgrade nom to 7.0 #128 (Ronald Holshausen, Sun Aug 22 11:56:33 2021 +1000)
* c274ca1a - fix: use the pacts for verification endpoint if the conusmer selectors are specified #133 (Ronald Holshausen, Sun Aug 22 11:51:22 2021 +1000)

# 0.1.1 - Bugfix Release

* 8bcd1c7e - fix: min/max type matchers must not apply the limits when cascading (Ronald Holshausen, Sun Aug 8 15:50:40 2021 +1000)
* cb1beb99 - feat(plugins): make NoopVariantMatcher public (Ronald Holshausen, Sat Aug 7 14:18:29 2021 +1000)
* 33b308d8 - feat(plugins): fix after merging PR (Ronald Holshausen, Thu Aug 5 12:43:58 2021 +1000)
* 4ca3e02b - Merge pull request #129 from mitre/docpath (Ronald Holshausen, Thu Aug 5 12:16:56 2021 +1000)
* 41e66b30 - feat(plugins): updated matching rules + generators to support working with plugins (Ronald Holshausen, Thu Aug 5 11:58:56 2021 +1000)
* 6124ed0b - refactor: Introduce DocPath struct for path expressions (Caleb Stepanian, Thu Jul 29 12:27:32 2021 -0400)

# 0.1.0 - Final Version

* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* b37a4d02 - chore: add a prelude module to the models crate (Ronald Holshausen, Fri Jul 23 13:10:48 2021 +1000)
* 20f01695 - refactor: Make many JSON parsing functions fallible (Caleb Stepanian, Wed Jul 21 18:04:45 2021 -0400)
* 458fdd15 - refactor: Move path expression functions into path_exp module (Caleb Stepanian, Mon Jul 19 14:22:02 2021 -0400)
* 3dccf866 - refacfor: moved the pact structs to the models crate (Ronald Holshausen, Sun Jul 18 16:58:14 2021 +1000)
* e8046d84 - refactor: moved interaction structs to the models crate (Ronald Holshausen, Sun Jul 18 14:36:03 2021 +1000)
* 31873ee3 - feat: added validation of provider state JSON (Ronald Holshausen, Wed Jul 14 15:44:20 2021 +1000)

# 0.0.5 - Moved structs to models crate + bugfixes and enhancements

* e2151800 - feat: support generating UUIDs with different formats #121 (Ronald Holshausen, Sun Jul 11 12:36:23 2021 +1000)
* e2e10241 - refactor: moved Request and Response structs to the models crate (Ronald Holshausen, Wed Jul 7 18:09:36 2021 +1000)
* 2c3c6ac0 - refactor: moved the header, body and query functions to the model crate (Ronald Holshausen, Wed Jul 7 16:37:28 2021 +1000)
* 9e8b01d7 - refactor: move HttpPart struct to models crate (Ronald Holshausen, Wed Jul 7 15:59:34 2021 +1000)
* 10e8ef87 - refactor: moved http_utils to the models crate (Ronald Holshausen, Wed Jul 7 14:34:20 2021 +1000)
* 01ff9877 - refactor: moved matching rules and generators to models crate (Ronald Holshausen, Sun Jul 4 17:17:30 2021 +1000)
* 357b2390 - refactor: move path expressions to models crate (Ronald Holshausen, Sun Jul 4 15:31:36 2021 +1000)
* 80e3c4e7 - fix: retain the data type for simple expressions #116 (Ronald Holshausen, Sun Jul 4 13:02:43 2021 +1000)
* b1a4c8cb - fix: failing tests #116 (Ronald Holshausen, Sun Jul 4 11:28:20 2021 +1000)
* e21db699 - fix: Keep the original value when injecting from a provider state value so data type is retained #116 (Ronald Holshausen, Sat Jul 3 18:01:34 2021 +1000)
* 8b075d38 - fix: FFI function was exposing a struct from the models crate (Ronald Holshausen, Sun Jun 27 11:30:55 2021 +1000)
* c3c22ea8 - Revert "refactor: moved matching rules and generators to models crate (part 1)" (Ronald Holshausen, Wed Jun 23 14:37:46 2021 +1000)
* d3406650 - refactor: moved matching rules and generators to models crate (part 1) (Ronald Holshausen, Wed Jun 23 12:58:30 2021 +1000)
* 5babb21b - refactor: convert xml_utils to use anyhow::Result (Ronald Holshausen, Tue Jun 22 16:35:56 2021 +1000)
* 9b7ad27d - refactor: moved xml_utils to models crate (Ronald Holshausen, Tue Jun 22 16:30:06 2021 +1000)
* 193da2b9 - chore: fix compiler warnings (Ronald Holshausen, Tue Jun 22 16:06:16 2021 +1000)
* 4db98181 - refactor: move file_utils to the models crate (Ronald Holshausen, Tue Jun 22 16:06:02 2021 +1000)

# 0.0.4 - Refactor + Bugfixes

* 225fd3d8 - chore: upgrade nom to 6.2.0 to resolve lexical-core compiler error (Ronald Holshausen, Tue Jun 22 15:10:31 2021 +1000)
* c392caae - Revert "update changelog for release 0.0.4" (Ronald Holshausen, Tue Jun 22 15:08:36 2021 +1000)
* 6866204f - update changelog for release 0.0.4 (Ronald Holshausen, Tue Jun 22 15:07:17 2021 +1000)
* ed3ae59a - chore: fix for failing parse_era test on CI (Ronald Holshausen, Tue Jun 22 14:55:46 2021 +1000)
* bbc638be - feat(pact file verification): verify consumer and provider sections (Ronald Holshausen, Fri Jun 18 16:52:15 2021 +1000)
* a7c071bc - feat(pact-file-validation): implemented validation of the metadata section (Ronald Holshausen, Wed Jun 16 09:17:28 2021 +1000)
* 00b65dcf - chore: rename pact_file_verifier -> pact_cli (Ronald Holshausen, Mon Jun 14 14:08:24 2021 +1000)
* 6198538d - refactor: move time_utils to pact_models crate (Ronald Holshausen, Fri Jun 11 12:58:26 2021 +1000)
* 5c670814 - refactor: move expression_parser to pact_models crate (Ronald Holshausen, Fri Jun 11 10:51:51 2021 +1000)
* 457aa5fc - fix(V4): Status code matcher was not converted to JSON correctly (Ronald Holshausen, Sun Jun 6 12:53:37 2021 +1000)
* b0ac7141 - feat: support graphql as a JSON content type (Ronald Holshausen, Sat Jun 5 15:14:06 2021 +1000)
* a44cbbee - fix: verifier was returning a mismatch when the expected body is empty #113 (Ronald Holshausen, Sat Jun 5 15:07:22 2021 +1000)
* 4e328d93 - feat: implement verification for RequestResponsePact, Consumer, Provider (Ronald Holshausen, Thu Jun 3 16:59:23 2021 +1000)
* 2f678213 - feat: initial prototype of a pact file verifier (Ronald Holshausen, Thu Jun 3 14:56:16 2021 +1000)

# 0.0.3 - Moved provider state models

* a7b81af - chore: fix clippy violation (Ronald Holshausen, Sat May 29 17:29:06 2021 +1000)
* 7022625 - refactor: move provider state models to the pact models crate (Ronald Holshausen, Sat May 29 17:18:48 2021 +1000)
* ef37cb9 - refactor(V4): extract common message parts into a seperate struct (Ronald Holshausen, Sat May 29 16:38:38 2021 +1000)
* ebb11df - feat(V4): fixed test _ refactored types for match functions (Ronald Holshausen, Sat May 29 14:56:31 2021 +1000)
* 73a53b8 - feat(V4): add an HTTP status code matcher (Ronald Holshausen, Fri May 28 18:40:11 2021 +1000)
* 8e8075b - refactor: move some more structs to the models crate (Ronald Holshausen, Thu May 27 14:34:03 2021 +1000)

# 0.0.2 - FFI support

* 82711d6 - chore: use a feature to enable FFI representation in the core crates (Ronald Holshausen, Mon May 3 12:14:02 2021 +1000)
* 6af4d3f - feat: allow ffi bindings to set spec version (Matt Fellows, Sun May 2 22:41:41 2021 +1000)

# 0.0.1 - Refactor: moved content type and body code from pact_matching

* 5ea36db - refactor: move content handling functions to pact_models crate (Ronald Holshausen, Sun Apr 25 13:12:22 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* 3dd610a - refactor: move structs and code dealing with bodies to a seperate package (Ronald Holshausen, Sun Apr 25 11:20:47 2021 +1000)
* a725ab1 - feat(V4): added synchronous request/response message formats (Ronald Holshausen, Sat Apr 24 16:05:12 2021 +1000)
* 4bcd94f - refactor: moved OptionalBody and content types to pact models crate (Ronald Holshausen, Thu Apr 22 14:01:56 2021 +1000)
* 80812d0 - refactor: move Consumer and Provider structs to models crate (Ronald Holshausen, Thu Apr 22 13:11:03 2021 +1000)
* 220fb5e - refactor: move the PactSpecification enum to the pact_models crate (Ronald Holshausen, Thu Apr 22 11:18:26 2021 +1000)
* 83d3d60 - chore: bump version to 0.0.1 (Ronald Holshausen, Thu Apr 22 10:52:04 2021 +1000)
* 9962e0e - chore: add required metadata fields to Cargo manifest (Ronald Holshausen, Thu Apr 22 10:45:14 2021 +1000)
* 34e7dcd - chore: add a pact models crate (Ronald Holshausen, Thu Apr 22 10:04:40 2021 +1000)

# 0.0.0 - First Release
