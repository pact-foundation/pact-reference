To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 2.0.3 - Bugfix Release

* 650a2582 - chore: Update pact_models to 1.3.8 (Ronald Holshausen, Wed Feb 18 12:19:37 2026 +1100)
* 2e7fe4a5 - fix: only split known multi-value HTTP headers on commas (Matt Fellows, Sat Feb 14 21:55:07 2026 +1100)
* fa52876a - chore: Fix compatability tests after update for wasm32-wasip2 target (Ronald Holshausen, Wed Sep 17 12:03:27 2025 +1000)
* 44b28cfc - bump version to 2.0.3 (Ronald Holshausen, Wed Sep 17 10:31:10 2025 +1000)

# 2.0.2 - Support the wasm32-wasip2 target

* b95b30dd - chore(pact_matching): Support compiling with wasm32-wasip2 target (Ronald Holshausen, Wed Sep 17 10:14:50 2025 +1000)
* eaaee9de - chore: Upgrade pact_models to 1.3.7 (Ronald Holshausen, Tue Sep 16 13:57:37 2025 +1000)
* 4ce86e68 - chore(pact_matching): Update deps (Ronald Holshausen, Tue Sep 16 13:55:25 2025 +1000)
* 0bb51981 - chore(pact_matching): Update test deps (Ronald Holshausen, Tue Sep 16 13:50:49 2025 +1000)
* e2d2ca33 - fix: compatibility-suite was failing after upgrading pact_matching to 2.0.1 (Ronald Holshausen, Wed Aug 20 10:32:37 2025 +1000)
* 805777f7 - bump version to 2.0.2 (Ronald Holshausen, Fri Aug 1 09:01:58 2025 +1000)

# 2.0.1 - Bugfix Release

* d1b455f5 - fix(pact_matching): Add missing XML feature gates (Ronald Holshausen, Thu Jul 31 15:32:16 2025 +1000)
* 167b05ac - bump version to 2.0.1 (Ronald Holshausen, Thu Jul 31 15:11:30 2025 +1000)

# 2.0.0 - 2.0 Release

* 72be9b0b - chore(pact_matching): remove beta pre-release version (Ronald Holshausen, Thu Jul 31 13:57:50 2025 +1000)
* dcdc4825 - refactor(v2-matching-engine): Record the elapsed time for executing a plan (Ronald Holshausen, Thu Jul 31 13:36:42 2025 +1000)
* e2e69295 - refactor(v2-matching-engine): Merged matchers.rs and matchingrules.rs; deprecated Matches replaced with DoMatch trait (Ronald Holshausen, Thu Jul 31 13:18:27 2025 +1000)
* c5d6f66c - chore(v2-matching-engine): Fix infinite recursion with matching rule applied to an empty body (Ronald Holshausen, Wed Jul 30 10:31:24 2025 +1000)
* ff964960 - chore: Update pact_models to 1.3.6 (Ronald Holshausen, Wed Jul 30 09:50:09 2025 +1000)
* 7d4f5819 - chore(v2-matching-engine): Support root matching rules for bodies (Ronald Holshausen, Tue Jul 29 16:09:29 2025 +1000)
* 1cb9a5ac - chore: Update pact_models to 1.3.5 (Ronald Holshausen, Mon Jul 28 10:44:30 2025 +1000)
* 3cac171b - chore: Ignore invalid spec tests (Ronald Holshausen, Mon Jul 28 09:58:47 2025 +1000)
* e3d23beb - chore(v2-matching-engine): Support XML namespaces with attributes (Ronald Holshausen, Fri Jul 25 15:38:20 2025 +1000)
* 8a32316e - chore(v2-matching-engine): Support for-each over an array at the root of JSON document (Ronald Holshausen, Fri Jul 25 11:24:47 2025 +1000)
* 5adada17 - chore(v2-matching-engine): Only compare text nodes if there is no type matcher for an XML element (Ronald Holshausen, Fri Jul 25 11:17:55 2025 +1000)
* d6966d96 - chore(v2-matching-engine): Also check the child elements if there is a type matcher on an XML element (Ronald Holshausen, Fri Jul 25 10:58:26 2025 +1000)
* 6aa960bc - chore(v2-matching-engine): Fix invalid matcher path in spec test (Ronald Holshausen, Fri Jul 25 10:15:49 2025 +1000)
* 5ca57cea - chore(v2-matching-engine): Fixed deeply nested JSON arrays (Ronald Holshausen, Fri Jul 25 10:02:50 2025 +1000)
* 64672a25 - chore(v2-matching-engine): Add response matching (Ronald Holshausen, Fri Jul 25 09:39:51 2025 +1000)
* 05e1225f - chore(v2-matching-engine): Fix for JSON with type matching rules (Ronald Holshausen, Thu Jul 24 10:54:21 2025 +1000)
* b6cba6d8 - chore(v2-matching-engine): Fix for XML with type matching rules (Ronald Holshausen, Thu Jul 24 10:36:05 2025 +1000)
* 80ca2b0d - chore(v2-matching-engine): Refactored engine/bodies into seperate JSON and XML modules (Ronald Holshausen, Thu Jul 24 10:00:49 2025 +1000)
* f95f7e35 - chore(v2-matching-engine): Fixes ported over from Pact-JVM (Ronald Holshausen, Wed Jul 23 16:31:50 2025 +1000)
* 2dc425d0 - chore: Add PACT_ prefix to all V2 engine env vars (Ronald Holshausen, Wed Jul 23 10:50:41 2025 +1000)
* 684a885a - chore: Remove snailquote as it is GPL licensed #493 (Ronald Holshausen, Thu Jun 12 14:41:47 2025 +1000)
* 7921641e - bump version to 2.0.0-beta.4 (Ronald Holshausen, Wed Jun 11 13:44:12 2025 +1000)

# 2.0.0-beta.3 - Fix for Alpine

* 2b74599f - chore: Correct release script (Ronald Holshausen, Wed Jun 11 13:40:16 2025 +1000)
* 6e8167c4 - fix(Alpine): Updae pact_models to 1.3.3 (Ronald Holshausen, Wed Jun 11 13:23:10 2025 +1000)
* 1d18d3f5 - fix: Onig crate needs default-features = false so it compiles on Alpine (Ronald Holshausen, Wed Jun 11 12:35:06 2025 +1000)
* 97d1d413 - bump version to 2.0.0-beta.3 (Ronald Holshausen, Wed Jun 11 11:18:14 2025 +1000)

# 2.0.0-beta.2 - Maintenance Release

* 31010256 - chore(pact_matching): Update dependencies (Ronald Holshausen, Wed Jun 11 10:29:42 2025 +1000)
* 18e53fbd - chore: Update pact_models to 1.3.2 (Ronald Holshausen, Wed Jun 11 10:16:03 2025 +1000)
* 57a6fb5c - chore(pact_matching): Fix typo and added test for str_form (Ronald Holshausen, Wed Jun 11 10:02:54 2025 +1000)
* 82d078d9 - fix: Upgrade onig to 6.5.1 as the Match Whole String for is_match has been released (Ronald Holshausen, Wed Jun 11 10:00:52 2025 +1000)
* cb729b65 - Merge pull request #487 from tienvx/use-new-xml-generator (Ronald Holshausen, Mon May 26 11:08:07 2025 +1000)
* d1fb1f35 - bump version to 2.0.0-beta.2 (Ronald Holshausen, Wed May 21 12:00:21 2025 +1000)
* e798b1cf - feat: Use new XML generator (tien.xuan.vo, Tue Feb 11 08:07:43 2025 +0700)

# 2.0.0-beta.1 - Bugfix Release

* 7c10b6af - fix(pact_matching): All XML code must be behind the xml feature gate (Ronald Holshausen, Wed May 21 11:26:09 2025 +1000)
* 7d8e8f61 - chore: Upgrade pact_models to 1.3.1 (Ronald Holshausen, Tue May 20 14:53:42 2025 +1000)
* 0383b9ac - chore: Remove the use of the deprecated pact_matching::logging::LOG_ID (Ronald Holshausen, Wed Mar 26 15:52:47 2025 +1100)
* 045df74e - chore: bump version to v2.0.0-beta.1 (Ronald Holshausen, Tue Mar 25 17:09:26 2025 +1100)

# 2.0.0-beta.0 - V2 matching engine for requests

* 90c5db04 - chore: Update readme with V2 engine details (Ronald Holshausen, Tue Mar 25 17:03:47 2025 +1100)
* a7f218f1 - fix: kiss_xml should be required with the XML feature (Ronald Holshausen, Tue Mar 25 16:52:31 2025 +1100)
* 880f1cca - chore(pact_matching): Upgrade crate to Rust 2024 edition (Ronald Holshausen, Tue Mar 25 16:41:21 2025 +1100)
* 12ca5283 - chore: Upgrade pact-plugin-driver to 0.7.4 (Ronald Holshausen, Tue Mar 25 16:26:48 2025 +1100)
* 0ed75202 - chore(pact-models): Upgrade pact models crate to Rust 2024 edition (Ronald Holshausen, Tue Mar 25 16:02:47 2025 +1100)
* ec67cd76 - feat: Handle min/max type matchers in XML elements (Ronald Holshausen, Tue Mar 25 11:53:17 2025 +1100)
* a8eeed06 - feat: Handle type matchers in XML elements (Ronald Holshausen, Tue Mar 25 11:11:12 2025 +1100)
* b43503a4 - feat: Handle multiple matching rules (Ronald Holshausen, Tue Mar 25 09:16:58 2025 +1100)
* 974249cc - feat: Handle matching rules on attributes and text nodes (Ronald Holshausen, Mon Mar 24 15:40:24 2025 +1100)
* d0331624 - feat: Handle incorrect number of XML child elements (Ronald Holshausen, Mon Mar 24 12:01:01 2025 +1100)
* e6784418 - feat: Simple XML cases passing (Ronald Holshausen, Mon Mar 24 10:41:04 2025 +1100)
* fb7dcdc3 - refactor: Move all the execution functions into an interpreter module (Ronald Holshausen, Fri Mar 21 10:53:29 2025 +1100)
* b07d7aec - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Wed Mar 19 11:22:43 2025 +1100)
* e32f0dbd - fix: Upgrade pact-plugin-driver to 0.7.3 (Ronald Holshausen, Wed Mar 19 11:20:39 2025 +1100)
* f565e9cf - feat: Implemented basic XML matching (Ronald Holshausen, Tue Mar 18 16:39:11 2025 +1100)
* ad9b555b - Merge remote-tracking branch 'origin/master' into feat/matching-engine-v2 (Ronald Holshausen, Fri Mar 14 11:42:13 2025 +1100)
* d67db1a4 - chore: Update pact_models to 1.2.7 (Ronald Holshausen, Fri Mar 14 11:02:53 2025 +1100)
* 8cb4964c - fix: Correct for situation where a more specific matcher overrides a more general one (Ronald Holshausen, Fri Mar 14 09:56:46 2025 +1100)
* 8e460bec - feat: tee and for-each should short-curcuit on error (Ronald Holshausen, Thu Mar 13 17:05:26 2025 +1100)
* b2a3d6ba - feat: Handle Min/Max type matchers with arrays (Ronald Holshausen, Thu Mar 13 12:14:13 2025 +1100)
* 8c130bc2 - feat: Fix propogation of type matchers to children (Ronald Holshausen, Wed Mar 12 10:20:38 2025 +1100)
* 82c7897f - feat: support matching rules with arrays (Ronald Holshausen, Tue Mar 11 15:11:58 2025 +1100)
* f0373f56 - fix: Need to handle object keys that are numeric (Ronald Holshausen, Tue Mar 11 14:08:10 2025 +1100)
* 0e328ed2 - fix: remove duplicate container nodes and add matching rules for array items (Ronald Holshausen, Tue Mar 11 13:57:18 2025 +1100)
* a7a447d9 - feat: header spec tests passing (Ronald Holshausen, Tue Mar 11 12:08:32 2025 +1100)
* c57885d1 - feat: changes required to get spec tests passing (Ronald Holshausen, Tue Mar 11 11:27:19 2025 +1100)
* b954e216 - feat: Add support for matching rules with body elements (Ronald Holshausen, Thu Mar 6 16:25:06 2025 +1100)
* e4b887ba - feat: Add coloured output to the plan summary (Ronald Holshausen, Thu Mar 6 10:53:19 2025 +1100)
* da1f846c - feat: Add config from environment variables (Ronald Holshausen, Thu Mar 6 10:40:22 2025 +1100)
* 9b2d588e - feat: Convert an executed plan into the previous result structure (Ronald Holshausen, Wed Mar 5 17:10:07 2025 +1100)
* 012550f4 - feat: Added generation of a summary for the executed plan (Ronald Holshausen, Wed Mar 5 10:12:05 2025 +1100)
* c865022f - feat: Add annotation nodes for headers (Ronald Holshausen, Tue Mar 4 10:21:23 2025 +1100)
* 46784c89 - feat: Add annotation nodes for query parameters (Ronald Holshausen, Tue Mar 4 09:53:58 2025 +1100)
* 7ced4908 - feat: Add annotation nodes for method and path (Ronald Holshausen, Mon Mar 3 16:04:50 2025 +1100)
* 187bb2c9 - feat: Handle parameterised header values (Ronald Holshausen, Fri Feb 28 16:20:32 2025 +1100)
* e9ae798a - feat: Header names need to be lower-cased when comparing them (Ronald Holshausen, Thu Feb 27 12:20:12 2025 +1100)
* 1c7d67b6 - feat: Support matching headers (Ronald Holshausen, Thu Feb 27 10:33:59 2025 +1100)
* af436bfd - feat: Support min/max matching rules with query parameters (Ronald Holshausen, Wed Feb 26 11:52:40 2025 +1100)
* 3db5c6a8 - feat: Support matching rules with query parameters (Ronald Holshausen, Wed Feb 26 11:05:43 2025 +1100)
* c3a75930 - feat: Implement plan execution for simple query strings (Ronald Holshausen, Tue Feb 25 16:40:43 2025 +1100)
* 37ff9ab4 - feat: Added support for all the matching rules (Ronald Holshausen, Mon Feb 24 14:35:20 2025 +1100)
* 6641ec27 - chore: Add existance checks when matching array values (Ronald Holshausen, Fri Feb 21 17:33:49 2025 +1100)
* a3a66ece - refactor: Extract functions from execute_action to reduce the function length (Ronald Holshausen, Fri Feb 21 16:39:27 2025 +1100)
* 2f7216aa - chore: Update actions to lazy evaluate their arguments (Ronald Holshausen, Fri Feb 21 16:22:33 2025 +1100)
* df810b79 - chore: Implement plan execution for simple JSON objects (Ronald Holshausen, Fri Feb 21 10:15:11 2025 +1100)
* d91f479e - chore: Implement plan execution for simple JSON arrays (Ronald Holshausen, Thu Feb 20 15:23:14 2025 +1100)
* 7b94a211 - Merge tag 'master' into feat/matching-engine-v2 (Ronald Holshausen, Thu Feb 20 15:16:16 2025 +1100)
* dc864ccc - chore: Implement plan execution for simple JSON plans (Ronald Holshausen, Wed Feb 19 15:32:00 2025 +1100)
* cb060659 - chore: add small timeout to test (Ronald Holshausen, Tue Feb 4 09:53:11 2025 +1100)
* d86ea2c8 - fix: Content type matcher was not being applied when dealing with multi-part bodies #482 (Ronald Holshausen, Mon Feb 3 17:38:49 2025 +1100)
* 8fb0bcb9 - chore: Upgrade pact_matching to 1.2.9 (Ronald Holshausen, Fri Jan 31 09:44:14 2025 +1100)
* 72fb8305 - bump version to 1.2.10 (Ronald Holshausen, Fri Jan 31 09:25:29 2025 +1100)
* cb57c9ad - chore: Handle simple JSON arrays (Ronald Holshausen, Mon Dec 9 17:03:19 2024 +1100)
* 8ed227e0 - chore: fix builder tests (Ronald Holshausen, Mon Dec 9 14:10:10 2024 +1100)
* 36b46a78 - chore: Added pipeline node and apply action (Ronald Holshausen, Mon Dec 9 14:08:23 2024 +1100)
* 5db8db23 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Wed Nov 13 14:16:31 2024 +1100)
* 3c9ee0c3 - chore: correct the return value for the if action (Ronald Holshausen, Tue Oct 29 10:07:36 2024 +1100)
* 5bb2b5ae - feat: Got basic request matching test to pass (Ronald Holshausen, Mon Oct 28 17:17:11 2024 +1100)
* 3746accc - chore: Add doc comments for the engine types (Ronald Holshausen, Fri Oct 25 12:07:55 2024 +1100)
* 11cd9dc0 - Merge branch 'master' into feat/matching-engine-v2 (Ronald Holshausen, Fri Oct 25 10:01:15 2024 +1100)
* b7ea5d93 - feat: Add plan steps for request path, query parameters and headers (Ronald Holshausen, Thu Aug 1 10:38:58 2024 +1000)
* a8dfe571 - feat: Add plan steps for request method (Ronald Holshausen, Wed Jul 31 10:38:53 2024 +1000)
* 035a3561 - feat: Prototype request matching plan (Ronald Holshausen, Tue Jul 30 16:58:34 2024 +1000)
* 02e2ae13 - chore: Bump pact_matching to 2.0.0-beta.0 (Ronald Holshausen, Tue Jul 30 11:40:20 2024 +1000)

# 1.2.9 - Bugfix Release

* e98a2183 - fix: Do not apply include matcher at the collection level #485 (Ronald Holshausen, Wed Jan 29 17:09:04 2025 +1100)
* 096deedc - fix: when comparing JSON with number matchers, don't covert numbers in string form to numbers #484 (Ronald Holshausen, Wed Jan 29 16:52:06 2025 +1100)
* d93dfb3f - fix: Do not apply date/time matchers at the collection level #483 (Ronald Holshausen, Wed Jan 29 14:34:31 2025 +1100)
* 5002bace - bump version to 1.2.9 (Ronald Holshausen, Thu Dec 19 15:37:28 2024 +1100)

# 1.2.8 - Bugfix Release

* c1bf98ef - Merge pull request #479 from tienvx/apply-generators-for-form-urlencoded (Ronald Holshausen, Thu Dec 19 15:04:20 2024 +1100)
* 5fdfd023 - chore: Upgrade all the tracing crates to the latest (Ronald Holshausen, Fri Dec 13 10:54:51 2024 +1100)
* b1cb2d5f - feat(pact_matching): Apply generators for form urlencoded (tien.xuan.vo, Wed Dec 11 14:29:49 2024 +0700)
* fab32aa0 - chore: Upgrade pact_models to 1.2.6 (Ronald Holshausen, Wed Dec 11 15:28:01 2024 +1100)
* 44d9dc1e - fix(plugins): Upgrade pact-plugin-driver to 0.7.2, fixes #473 (Ronald Holshausen, Fri Nov 29 15:48:40 2024 +1100)
* 9c3bfb6f - Merge pull request #463 from tienvx/fix-cant-match-array-of-numbers (Ronald Holshausen, Wed Nov 13 11:04:32 2024 +1100)
* 420c654d - chore: Upgrade pact_models to 1.2.5 (Ronald Holshausen, Wed Nov 13 11:01:39 2024 +1100)
* 303073cf - docs: update obsolete links [ci skip] (Yousaf Nabi, Fri Sep 6 13:01:21 2024 +0100)
* 61c68b8c - test: Test values using number, integer, decimal matchers (tien.xuan.vo, Fri Sep 6 15:55:39 2024 +0700)
* 5d38945a - fix: Fix can't match array of numbers (tien.xuan.vo, Fri Sep 6 11:13:43 2024 +0700)
* effa4c76 - chore: Update deprecation message on in-mem support loggin functions in pact_matching crate (Ronald Holshausen, Thu Sep 5 10:30:12 2024 +1000)
* 18770107 - chore: Add deprecation notice to the functions that use LOG_ID (Ronald Holshausen, Wed Sep 4 15:44:03 2024 +1000)
* c1a00dd8 - chore: Upgrade pact-plugin-driver to 0.7.1 (Ronald Holshausen, Tue Sep 3 15:47:22 2024 +1000)
* 08288e32 - bump version to 1.2.8 (Ronald Holshausen, Tue Aug 27 14:48:23 2024 +1000)

# 1.2.7 - Bugfix Release

* e6f8ae46 - fix(plugins): Plugin config needs to have seperate values for HTTP request and response parts (Ronald Holshausen, Tue Aug 27 14:32:37 2024 +1000)
* 9f6a8c35 - bump version to 1.2.7 (Ronald Holshausen, Tue Aug 13 15:21:05 2024 +1000)

# 1.2.6 - Maintenance Release

* ab836e18 - chore(pact_matching): Update dependencies (Ronald Holshausen, Tue Aug 13 15:01:16 2024 +1000)
* f50454ff - chore: Upgrade test dependencies (Ronald Holshausen, Tue Aug 13 11:54:59 2024 +1000)
* 9a7eef75 - chore: Upgrade pact_models to 1.2.4 (Ronald Holshausen, Tue Aug 13 11:51:06 2024 +1000)
* 05c27197 - chore(pact_matching): Update trace level instrumentation (Ronald Holshausen, Tue Aug 13 11:14:40 2024 +1000)
* bcf4db6d - chore: Upgrade pact_models to 1.2.3 (Ronald Holshausen, Tue Aug 6 11:06:22 2024 +1000)
* c6df3fcc - bump version to 1.2.6 (Ronald Holshausen, Wed Jul 17 11:56:14 2024 +1000)

# 1.2.5 - Bugfix Release

* fd9ff70a - chore: Upgrade pact_models to 1.2.2 (Ronald Holshausen, Wed Jul 17 11:15:34 2024 +1000)
* e8b62f98 - chore: Upgrade pact-plugin-driver to 0.7.0 (Ronald Holshausen, Wed Jul 17 10:45:34 2024 +1000)
* efa1295a - feat(pact_verifier): Allow provider state generator to fall back to the provider state parameters #441 (Ronald Holshausen, Tue Jul 9 17:27:09 2024 +1000)
* 42e2ba68 - fix(pact_verifier): Take context paths into account when fetching the root HAL resource #420 (Ronald Holshausen, Tue Jul 9 11:37:04 2024 +1000)
* da869ddb - fix: reduce log level for matching to debug (Matt Fellows, Fri Jul 5 00:13:50 2024 +1000)
* 2163476d - bump version to 1.2.5 (Ronald Holshausen, Fri Jun 14 10:07:48 2024 +1000)

# 1.2.4 - Maintenance Release

* 30998a95 - chore: Upgrade pact_models to 1.2.1 (Ronald Holshausen, Wed Jun 12 15:26:49 2024 +1000)
* 37d65469 - chore: add some debug logs + fix compiler warning (Ronald Holshausen, Tue Jun 11 15:12:26 2024 +1000)
* a811b8a4 - chore: Update pact_matching to 1.2.3 (Ronald Holshausen, Thu May 30 15:57:32 2024 +1000)
* 67c75a09 - bump version to 1.2.4 (Ronald Holshausen, Thu May 30 15:47:12 2024 +1000)

# 1.2.3 - Supports content type detection on Windows

* e0ffa61f - chore: refactor matching logic and add debug logs if text/plain detected, before we match (Yousaf Nabi, Wed May 29 20:42:15 2024 +0100)
* 72c6e572 - chore: preserve original behaviour for text/plain content-type header with application/json body (Yousaf Nabi, Wed May 29 06:09:12 2024 +0100)
* 57b16b89 - chore: only check with tree_magic if inferred_content_type is empty (Yousaf Nabi, Wed May 29 04:17:39 2024 +0100)
* 60944777 - chore: inferred_content_type default is empty string if no match (Yousaf Nabi, Wed May 29 04:03:36 2024 +0100)
* b0fd7e2f - chore: refactor to preserve existing error messages (Yousaf Nabi, Wed May 29 03:31:11 2024 +0100)
* 968be075 - feat: update content_type_matching logic for binary payloads (Yousaf Nabi, Tue May 28 23:25:07 2024 +0100)
* 88f71944 - bump version to 1.2.3 (Ronald Holshausen, Tue May 7 10:05:48 2024 +1000)

# 1.2.2 - Maintenance Release

* 694100fb - chore: Update pact_models to 1.2.0 (Ronald Holshausen, Tue Apr 23 10:51:11 2024 +1000)
* edfac7ba - chore: remove local pact_models from the other pact crates (Ronald Holshausen, Tue Apr 23 10:03:29 2024 +1000)
* c3128a6d - feat: Support optional query parameter values (where there is only a name) (Ronald Holshausen, Mon Apr 22 10:36:05 2024 +1000)
* facb2c45 - bump version to 1.2.2 (Ronald Holshausen, Tue Apr 16 16:26:37 2024 +1000)

# 1.2.1 - Maintenance Release

* 3706f6ef - chore: Add From implementation for CommonMismatch (Ronald Holshausen, Tue Apr 16 16:13:34 2024 +1000)
* eb4d9a59 - bump version to 1.2.1 (Ronald Holshausen, Tue Apr 16 10:21:53 2024 +1000)

# 1.2.0 - Maintenance Release

* d6125b75 - chore(pact_matching): Bump minor version (Ronald Holshausen, Tue Apr 16 10:16:44 2024 +1000)
* 7900fe5a - chore(pact_matching): Update dependencies (Ronald Holshausen, Tue Apr 16 09:55:36 2024 +1000)
* 411b3314 - chore(pact_matching): Update pact-plugin-driver to 0.6.0 (Ronald Holshausen, Tue Apr 16 09:50:54 2024 +1000)
* 0ba022b1 - chore(pact_matching): Update dependencies (Ronald Holshausen, Tue Apr 16 09:41:00 2024 +1000)
* 74f43bdf - chore(pact_models): clean up some deprecation warnings (Ronald Holshausen, Wed Mar 20 15:34:27 2024 +1100)
* 8985576e - chore: Lock tree_magic_mimi to 3.0 as 3.1 fails to compile with MUSL (Ronald Holshausen, Fri Mar 15 14:52:03 2024 +1100)
* fe508268 - Merge pull request #396 from tienvx/match-headers-with-list-matching-rules (Ronald Holshausen, Wed Feb 28 14:01:34 2024 +1100)
* f117dcd2 - feat: Match headers with ArrayContains and EachValue matchers (tien.xuan.vo, Tue Feb 27 16:07:35 2024 +0700)
* a567af54 - test: Test match json body with EachValue matcher (tien.xuan.vo, Wed Feb 28 08:21:48 2024 +0700)
* 1c8a6f06 - feat: Match query with ArrayContains and EachValue matchers (tien.xuan.vo, Tue Feb 27 15:22:34 2024 +0700)
* b2e43564 - refactor: Replace Mismatch by CommonMismatch (tien.xuan.vo, Tue Feb 27 00:45:47 2024 +0700)
* 8be1bf32 - chore: clean up trace level instrumentation on matcher_is_defined function (Ronald Holshausen, Wed Feb 7 15:37:20 2024 +1100)
* 49cdc78a - Merge pull request #382 from tienvx/improve-content-type-match-error-message (Ronald Holshausen, Wed Feb 7 14:35:22 2024 +1100)
* 20742d98 - feat: Rephrase the content type matching error message (tien.xuan.vo, Wed Feb 7 09:58:13 2024 +0700)
* 7552d617 - bump version to 1.1.11 (Ronald Holshausen, Wed Feb 7 13:15:52 2024 +1100)

# 1.1.10 - Bugfix Release

* a8efa111 - fix: was missing an implementation of MinType and MaxType matchers for maps (Ronald Holshausen, Wed Feb 7 12:27:59 2024 +1100)
* 24a26cca - chore: Update pact_models to 1.1.18 (Ronald Holshausen, Wed Feb 7 10:53:22 2024 +1100)
* 73578350 - chore: use local pact_models (JP-Ellis, Tue Feb 6 10:51:09 2024 +1100)
* 95cbe5a9 - fix: Upgrade pact-plugin-driver to 0.5.1 (Ronald Holshausen, Wed Jan 31 19:56:04 2024 +1100)
* c7cb72c5 - Merge pull request #365 from tienvx/test-values-matcher-json-body (Ronald Holshausen, Thu Jan 25 10:36:22 2024 +1100)
* d5e58424 - test(pact_matching): Test Values matcher with json body (tien.xuan.vo, Mon Jan 22 23:36:44 2024 +0700)
* 482da6f0 - bump version to 1.1.10 (Ronald Holshausen, Sat Jan 20 15:10:39 2024 +1100)

# 1.1.9 - Maintenance Release

* 7b087acf - chore: Upgrade pact-plugin-driver to 0.5.0 (Ronald Holshausen, Sat Jan 20 14:49:21 2024 +1100)
* b735df9d - chore: Upgrade pact_models to 1.1.17 (Ronald Holshausen, Sat Jan 20 13:54:03 2024 +1100)
* 3ffd77ce - bump version to 1.1.9 (Ronald Holshausen, Fri Jan 19 18:16:32 2024 +1100)

# 1.1.8 - Bugfix Release

* f9792697 - Merge pull request #364 from tienvx/allow-matching-strings-with-values-matcher (Ronald Holshausen, Mon Jan 22 10:51:13 2024 +1100)
* 045ffbb6 - Merge pull request #363 from tienvx/test-content-type-matcher (Ronald Holshausen, Mon Jan 22 10:45:34 2024 +1100)
* f4891c03 - fix(pact_matching): Allow matching strings using Values matcher (tien.xuan.vo, Mon Jan 22 00:57:34 2024 +0700)
* 37e498e9 - test(pact_matching): Test more cases for ContentType matcher (tien.xuan.vo, Sat Jan 20 17:39:07 2024 +0700)
* ce421520 - chore: add a test for generators on query parameters with square brackets #359 (Ronald Holshausen, Fri Jan 19 17:38:42 2024 +1100)
* 9d60c8b0 - chore: add a test for matching rules on query parameters with square brackets #359 (Ronald Holshausen, Fri Jan 19 17:17:29 2024 +1100)
* 944613df - fix: regression - upgrade pact_models to 1.1.16 #359 (Ronald Holshausen, Fri Jan 19 14:52:36 2024 +1100)
* 403c0af1 - chore: Upgrade pact_models to 1.1.14 #355 (Ronald Holshausen, Tue Jan 16 10:31:12 2024 +1100)
* dfd13760 - chore: Upgrade pact_models to 1.1.13 #355 (Ronald Holshausen, Tue Jan 16 07:42:33 2024 +1100)
* 713e4098 - chore: Upgrade pact-plugin-driver to 0.4.6 (Ronald Holshausen, Thu Dec 14 17:04:59 2023 +1100)
* b7789ae1 - bump version to 1.1.8 (Ronald Holshausen, Tue Nov 14 02:45:34 2023 +1100)

# 1.1.7 - Maintenance Release

* 826758a6 - chore: Upgrade pact_models to 1.1.12 (Ronald Holshausen, Mon Nov 13 17:25:21 2023 +1100)
* 5402b6af - chore: Add a test for using Values matcher with headers #331 (Ronald Holshausen, Wed Oct 25 04:12:07 2023 +1100)
* dc0ae305 - chore: Add missing doc comments to the public modules (Ronald Holshausen, Wed Oct 25 04:11:34 2023 +1100)
* 985cb590 - chore: header module is not public (Ronald Holshausen, Mon Oct 23 03:07:28 2023 +1100)
* bfcd4d94 - bump version to 1.1.7 (Ronald Holshausen, Fri Sep 22 10:58:19 2023 +1000)

# 1.1.6 - Bugfix Release

* 63b7cf9d - fix: Allow matching string values using content type matching rule (tien.xuan.vo, Mon Sep 11 18:32:24 2023 +0700)
* fa661d30 - bump version to 1.1.6 (Ronald Holshausen, Fri Aug 18 15:35:06 2023 +1000)

# 1.1.5 - Maintenance Release

* ea98f458 - feat(compatibility-suite): Implemented V4 HTTP scenarios (Ronald Holshausen, Tue Aug 15 15:09:24 2023 +1000)
* f168617d - chore(compatibility-suite): Update suite and correct V3 steps (Ronald Holshausen, Fri Aug 11 11:38:09 2023 +1000)
* 5b91796b - fix(pact_matching): Correct the mismatch error with content-type header when there are multiple values #305 (Ronald Holshausen, Tue Aug 8 10:39:43 2023 +1000)
* e4da3e42 - chore: Upgrade pact_models to 1.1.11 (Ronald Holshausen, Mon Aug 7 13:59:34 2023 +1000)
* 24ed7835 - chore: Upgrade pact-models to 1.1.10 (Ronald Holshausen, Fri Aug 4 16:11:24 2023 +1000)
* 1299a622 - fix(pact_matching): Support generators for V3 Message interactions (Ronald Holshausen, Wed Aug 2 16:43:51 2023 +1000)
* a5ce79d4 - bump version to 1.1.5 (Ronald Holshausen, Thu Jul 27 10:51:04 2023 +1000)

# 1.1.4 - Bugfix Release

* ac885b89 - fix: Date/Time matchers should fall back to ISO 8601 formats if no format string is provided (Ronald Holshausen, Thu Jul 27 10:42:17 2023 +1000)
* 4a01919a - chore: Upgrade pact_models to 1.1.9 (Ronald Holshausen, Thu Jul 27 10:24:00 2023 +1000)
* 8a22a66a - fix: correct equality error message to match compatibility-suite (Ronald Holshausen, Wed Jul 26 14:30:48 2023 +1000)
* 2e45e223 - fix: Update matching error messages to be in line with the compatibility-suite (Ronald Holshausen, Tue Jul 25 17:42:03 2023 +1000)
* 78bc8e23 - fix(pact_matching): Generators should add headers and query parameters if the key does not exist (Ronald Holshausen, Tue Jul 25 15:22:24 2023 +1000)
* 0459d40c - fix(pact_matching): Number of keys were still be compared when an EachKeys matcher is defined #301 (Ronald Holshausen, Mon Jul 17 13:37:05 2023 +1000)
* 4f448b1f - fix(pact_matching): EachValue matcher was not applying the associated matching rules correctly #299 (Ronald Holshausen, Wed Jul 12 16:28:16 2023 +1000)
* 3cd0417c - chore: Add crate features to readme and module docs (Ronald Holshausen, Tue Jul 11 14:24:40 2023 +1000)
* 53867be5 - bump version to 1.1.4 (Ronald Holshausen, Tue Jul 11 11:33:25 2023 +1000)

# 1.1.3 - Add crate features to turn-off some behaviour and dependencies

* f6024b1d - chore: Add support for plugin feature (Ronald Holshausen, Tue Jul 11 11:21:24 2023 +1000)
* 27441992 - chore: Add support for multipart feature (Ronald Holshausen, Tue Jul 11 10:38:23 2023 +1000)
* 71ef0f49 - chore: Add support for datetime and xml features (Ronald Holshausen, Tue Jul 11 10:17:28 2023 +1000)
* f2ae77ba - chore: Upgrade pact-plugin-driver to 0.4.5 (Ronald Holshausen, Mon Jul 10 17:15:20 2023 +1000)
* a5e04a3d - bump version to 1.1.3 (Ronald Holshausen, Mon Jul 10 16:36:04 2023 +1000)

# 1.1.2 - Bugfix Release

* 1deca59a - chore: Upgrade pact_models to 1.1.8 (Ronald Holshausen, Mon Jul 10 16:15:43 2023 +1000)
* 913e9a27 - feat(multipart): Support the content-transfer-encoding header (Ronald Holshausen, Mon Jul 10 11:20:04 2023 +1000)
* a6049684 - refactor: Update MIME multipart parser to a supported lib + make implementation inline with Pact-JVM (Ronald Holshausen, Fri Jul 7 15:42:02 2023 +1000)
* 692120aa - feat: Add support for matching application/x-www-form-urlencoded encoded bodies (Ronald Holshausen, Thu Jul 6 12:41:59 2023 +1000)
* 67e8ad8c - refactor: use DocPath.to_lower_case from pact_models (Ronald Holshausen, Thu Jul 6 10:46:33 2023 +1000)
* 2662cdfc - chore: Upgrade pact_models to 1.1.7 (Ronald Holshausen, Thu Jul 6 10:27:25 2023 +1000)
* 72818ea7 - feat: Update to latest compatibility-suite and correct some error messages (Ronald Holshausen, Wed Jul 5 15:29:03 2023 +1000)
* 445ea1ee - fix: Header matching rules should be looked up in a case-insenstive way (Ronald Holshausen, Wed Jun 28 15:21:32 2023 +1000)
* 95753e29 - fix: Correct the use of matching rules on repeated header values (Ronald Holshausen, Wed Jun 28 11:57:35 2023 +1000)
* 6a7e504e - fix: Correct the use of matching rules on repeated query parameters (Ronald Holshausen, Wed Jun 28 11:56:07 2023 +1000)
* d5df06ac - fix: Regex matcher was incorrectly being applied to lists (Ronald Holshausen, Wed Jun 28 11:52:45 2023 +1000)
* f88adb2a - fix: Correct the format of matching errors on JSON (Ronald Holshausen, Wed Jun 28 11:51:10 2023 +1000)
* a3c5db59 - bump version to 1.1.2 (Ronald Holshausen, Thu Jun 22 15:56:55 2023 +1000)

# 1.1.1 - Bugfix Release

* e95ae4d0 - chore: Upgrade pact_models to 1.1.6 (Ronald Holshausen, Thu Jun 22 15:40:55 2023 +1000)
* af498c73 - fix(pact_matching): EachValue matcher was applying the associated rule to the list and not the items in the list (Ronald Holshausen, Thu Jun 22 12:29:32 2023 +1000)
* bc68ed7f - chore: Upgrade pact_models to 1.1.4 (Ronald Holshausen, Thu Jun 1 10:22:38 2023 +1000)
* 37673fac - fix: correct tests after upgrading pact_models (Ronald Holshausen, Mon May 29 15:13:44 2023 +1000)
* 397c837f - chore: Upgrade pact_models to 1.1.3 (fixes MockServerURL generator) (Ronald Holshausen, Mon May 29 15:12:22 2023 +1000)
* 8f27f9bd - chore: Upgrade pact-plugin-driver to 0.4.4 (Ronald Holshausen, Tue May 23 11:55:23 2023 +1000)
* ac2e24da - chore: Use "Minimum version, with restricted compatibility range" for all Pact crate versions (Ronald Holshausen, Tue May 23 11:46:52 2023 +1000)
* 67cf4a06 - bump version to 1.1.1 (Ronald Holshausen, Tue May 23 11:30:39 2023 +1000)

# 1.1.0 - Update Pact models to 1.1 (breaking change)

* 54887690 - chore: Bump pact_matching to 1.1 (Ronald Holshausen, Tue May 23 11:13:14 2023 +1000)
* f72f8191 - feat: Implemented the remaining V1 HTTP consumer compatability suite feature (Ronald Holshausen, Thu May 18 14:12:40 2023 +1000)
* 261ecf47 - fix: Add RefUnwindSafe trait bound to all Pact and Interaction uses (Ronald Holshausen, Mon May 15 13:59:31 2023 +1000)
* 40df44ba - bump version to 1.0.9 (Ronald Holshausen, Tue Apr 18 13:03:24 2023 +1000)

# 1.0.8 - Bugfix Release

* 6c14abfd - chore: Upgrade pact_models to 1.0.13 (Ronald Holshausen, Tue Apr 18 13:00:01 2023 +1000)
* ce16d43f - chore: Upgrade pact-plugin-driver to 0.4.2 (supports auto-installing known plugins) (Ronald Holshausen, Tue Apr 18 11:49:52 2023 +1000)
* 10bf1a48 - chore: Upgrade pact_models to 1.0.12 (fixes generators hash function) (Ronald Holshausen, Mon Apr 17 10:31:09 2023 +1000)
* 84b9d9e9 - fix: Upgrade pact models to 1.0.11 (fixes generated key for V4 Pacts) (Ronald Holshausen, Fri Apr 14 17:10:58 2023 +1000)
* 669f7812 - chore: Upgrade pact_models to 1.0.10 (Ronald Holshausen, Thu Apr 13 15:32:34 2023 +1000)
* 779a59f0 - fix: Upgrade pact-plugin-driver to 0.4.1 (fixes an issue introduced in 0.4.0 with shared channels to plugins) (Ronald Holshausen, Wed Apr 5 17:01:18 2023 +1000)
* 126cf462 - chore: Upgrade pact_matching to 1.0.7 (Ronald Holshausen, Tue Apr 4 15:12:28 2023 +1000)
* f816170b - bump version to 1.0.8 (Ronald Holshausen, Tue Apr 4 15:06:25 2023 +1000)

# 1.0.7 - Maintenance Release

* eb0b7fdf - chore: Update dependencies (Ronald Holshausen, Tue Apr 4 14:46:30 2023 +1000)
* 6f0c4b2f - feat: Upgrade pact-plugin-driver to 0.4.0 which uses a shared gRPC channel to each plugin (Ronald Holshausen, Tue Apr 4 14:32:36 2023 +1000)
* c6b66a28 - chore: Test was failing with dates where day of the month has one digit (Ronald Holshausen, Mon Apr 3 11:55:45 2023 +1000)
* 8ecf1a68 - bump version to 1.0.7 (Ronald Holshausen, Wed Mar 15 14:49:40 2023 +1100)

# 1.0.6 - Bugfix Release

* e96bc54e - fix: Upgrade pact_models to 1.0.9 (fixes issues with headers) (Ronald Holshausen, Wed Mar 15 14:31:00 2023 +1100)
* f7e0b669 - chore: Upgrade pact_models to 1.0.8 (Ronald Holshausen, Wed Mar 15 12:19:22 2023 +1100)
* 57728a01 - chore: update pact-plugin-driver to 0.3.3 (Ronald Holshausen, Tue Mar 14 17:19:20 2023 +1100)
* c559bc3d - fix: header matching was incorrectly stripping whitespace around commas #259 (Ronald Holshausen, Tue Mar 14 15:36:34 2023 +1100)
* 0676047e - chore: Upgrade pact-plugin-driver to 0.3.2 (Ronald Holshausen, Thu Feb 16 12:09:46 2023 +1100)
* 46e0a7df - chore: correct doc comment (Ronald Holshausen, Wed Feb 8 14:35:38 2023 +1100)
* 14b1b240 - bump version to 1.0.6 (Ronald Holshausen, Wed Feb 8 13:50:37 2023 +1100)

# 1.0.5 - Maintenance Release

* 1e7331f1 - fix: Upgrade plugin driver to 0.3.1 (Ronald Holshausen, Wed Feb 8 13:28:07 2023 +1100)
* e9978a00 - bump version to 1.0.5 (Ronald Holshausen, Mon Feb 6 15:32:35 2023 +1100)

# 1.0.4 - Support matching rules for message metadata

* 0b70060f - chore: Upgrade pact-plugin-driver and base64 crates (supports message metadata) (Ronald Holshausen, Mon Feb 6 14:56:29 2023 +1100)
* 1acacffd - bump version to 1.0.4 (Ronald Holshausen, Wed Jan 11 14:55:00 2023 +1100)

# 1.0.3 - Bugfix Release

* 7d84d941 - chore: Upgrade pact_models to 1.0.4 (Ronald Holshausen, Wed Jan 11 14:33:13 2023 +1100)
* a8abf5df - chore: log spans at trace level to reduce the log entry size at other log levels #243 (Ronald Holshausen, Tue Jan 10 09:00:52 2023 +1100)
* 2c8467ed - fix: Header matching rules with an index were not being applied #238 (Ronald Holshausen, Mon Jan 9 16:45:51 2023 +1100)
* 4409441b - fix: Matching rules are not being applied correctly to message metadata #245 (Ronald Holshausen, Mon Jan 9 13:43:41 2023 +1100)
* 4f786ff4 - fix: support header values that are not well formed #228 (Ronald Holshausen, Wed Jan 4 11:05:45 2023 +1100)
* 1bdb1054 - chore: Upgrade pact_models to 1.0.3 #239 (Ronald Holshausen, Thu Dec 22 15:37:53 2022 +1100)
* 3aecb702 - chore: require tracing-subscriber for tests for crates that use pact_models #239 (Ronald Holshausen, Thu Dec 22 14:37:01 2022 +1100)
* 9c5bc31d - bump version to 1.0.3 (Ronald Holshausen, Mon Dec 19 15:25:33 2022 +1100)

# 1.0.2 - Maintenance Release

* 5fbb0d6a - feat: Upgrade plugin driver to 0.2.2 (supports passing a test context to support generators) (Ronald Holshausen, Fri Dec 16 16:38:03 2022 +1100)
* 1ab47c6f - chore: Upgrade Tokio to latest (Ronald Holshausen, Fri Dec 16 16:31:31 2022 +1100)
* e749dbad - bump version to 1.0.2 (Ronald Holshausen, Wed Dec 14 16:59:54 2022 +1100)

# 1.0.1 - Bugfix Release

* 8be00f0c - chore: Upgrade pact-plugin-driver to 0.2.1 (Ronald Holshausen, Wed Dec 14 14:55:32 2022 +1100)
* e91ad622 - fix: Interaction builder was not copying plugin config data to the Pact metadata (Ronald Holshausen, Mon Dec 12 13:59:36 2022 +1100)
* b258c94f - bump version to 1.0.1 (Ronald Holshausen, Fri Dec 9 17:56:35 2022 +1100)

# 1.0.0 - Support plugins generating interaction content

* 1744ddc2 - feat: Support plugins generating interaction content (Ronald Holshausen, Fri Dec 9 17:24:04 2022 +1100)
* bf2eca32 - bump version to 0.12.16 (Ronald Holshausen, Mon Nov 28 14:20:13 2022 +1100)

# 0.12.15 - Maintenance Release

* c9721fd5 - chore: Upgrade pact_models to 1.0.1 and pact-plugin-driver to 0.1.16 (Ronald Holshausen, Mon Nov 28 14:10:53 2022 +1100)
* 123060e3 - chore: Upgrade pact_matching to 0.12.14 (Ronald Holshausen, Mon Nov 7 11:34:36 2022 +1100)
* adaa1b02 - bump version to 0.12.15 (Ronald Holshausen, Mon Nov 7 11:22:16 2022 +1100)

# 0.12.14 - Maintenance Release

* 577824e7 - fix: Upgrade pact_models to 1.0 and pact-plugin-driver to 0.1.15 to fix cyclic dependency issue (Ronald Holshausen, Mon Nov 7 11:14:20 2022 +1100)
* 972e27dd - bump version to 0.12.14 (Ronald Holshausen, Fri Nov 4 16:48:51 2022 +1100)

# 0.12.13 - Bugfix Release

* e1f985ad - chore: Upgrade pact_models to 0.4.6 and pact-plugin-driver to 0.1.14 (Ronald Holshausen, Fri Nov 4 16:38:36 2022 +1100)
* 6ad00a5d - fix: Update onig to latest master to fix  Regex Matcher Fails On Valid Inputs #214 (Ronald Holshausen, Fri Nov 4 15:23:50 2022 +1100)
* 83d14ce1 - fix: when comparing content types, check the base type if the actual content type has a suffix #224 (Ronald Holshausen, Fri Nov 4 14:22:47 2022 +1100)
* 965a1c41 - fix: Upgrade plugin driver to 0.1.13 (fixes issue loading plugin when there are multiple versions for the same plugin) (Ronald Holshausen, Wed Oct 5 17:29:37 2022 +1100)
* 244849d7 - bump version to 0.12.13 (Ronald Holshausen, Wed Sep 28 10:04:47 2022 +1000)

# 0.12.12 - Maintenance Release

* b8be05c1 - fix(FFI): Use a star for the path with values matcher #216 (Ronald Holshausen, Tue Sep 27 17:50:32 2022 +1000)
* 60b2b642 - chore: Upgrade pact-plugin-driver to 0.1.12 (Ronald Holshausen, Mon Sep 12 17:44:13 2022 +1000)
* fcab3016 - chore: Upgrade pact-plugin-driver to 0.1.11 (Ronald Holshausen, Thu Sep 8 11:28:52 2022 +1000)
* ac4fe73f - chore: fix to release scripts (Ronald Holshausen, Wed Sep 7 10:51:01 2022 +1000)
* f8db90d2 - fix: Upgrade pact_models to 0.4.5 - fixes FFI bug with generators for request paths (Ronald Holshausen, Fri Aug 26 11:44:08 2022 +1000)
* 4524bcf9 - bump version to 0.12.12 (Ronald Holshausen, Thu Aug 18 15:01:27 2022 +1000)

# 0.12.11 - Maintenance Release

* 72f9f75d - chore: clean some deprecation warnings (Ronald Holshausen, Thu Aug 18 14:58:35 2022 +1000)
* 32a70382 - chore: Upgrade pact_models (0.4.4), plugin driver (0.1.10), tracing and tracing core crates (Ronald Holshausen, Thu Aug 18 14:41:52 2022 +1000)
* 11d162a8 - chore: disable content type check tests ion windows (Ronald Holshausen, Wed Aug 17 17:08:14 2022 +1000)
* 65d05149 - fix: content type matcher was not being applied if content type was not octet_stream #171 (Ronald Holshausen, Wed Aug 17 16:32:43 2022 +1000)
* 6c5c90ee - refactor: split metrics into sync and async functions (Ronald Holshausen, Thu Aug 11 14:55:32 2022 +1000)
* f5e4c3a7 - bump version to 0.12.11 (Ronald Holshausen, Wed Aug 10 12:33:21 2022 +1000)

# 0.12.10 - Maintenance Release

* 33b04eee - chore: cleanup some deprecation warnings (Ronald Holshausen, Wed Aug 10 10:34:58 2022 +1000)
* 195ad07b - chore: Updated dependant crates (uuid, simplelog) (Ronald Holshausen, Wed Aug 10 10:22:07 2022 +1000)
* 49232caa - chore: Update pact plugin driver to 0.1.9 (Ronald Holshausen, Wed Aug 10 10:14:42 2022 +1000)
* a3fe5e7f - chore: Update pact models to 0.4.2 (Ronald Holshausen, Wed Aug 10 10:10:41 2022 +1000)
* 3d73e3c2 - Removed dependency on time v0.1 (Daan Oosterveld, Wed Jul 6 15:56:29 2022 +0200)
* 40f7bdc4 - feat: add verification option to disable ANSI escape codes in output #203 (Ronald Holshausen, Wed Jul 20 12:18:12 2022 +1000)
* 4de924d0 - bump version to 0.12.10 (Ronald Holshausen, Fri Jun 10 15:35:55 2022 +1000)

# 0.12.9 - Bugfix Release

* 0e3db9df - fix: comparing query paraneters where actual has less values but there is a type matcher (Ronald Holshausen, Fri Jun 10 15:17:45 2022 +1000)
* 4e9d8374 - fix: min/max type matchers were not being applied to query parameters (Ronald Holshausen, Fri Jun 10 14:17:41 2022 +1000)
* 09e63637 - bump version to 0.12.9 (Ronald Holshausen, Mon May 30 11:42:13 2022 +1000)

# 0.12.8 - Maintenance Release

* 80256458 - chore: Upgrade pact-plugin-driver to 0.1.8 (Ronald Holshausen, Mon May 30 11:36:54 2022 +1000)
* d9b9fe72 - chore: Upgrade pact-plugin-driver to 0.1.7 (Ronald Holshausen, Fri May 20 15:56:23 2022 +1000)
* 191cd687 - bump version to 0.12.8 (Ronald Holshausen, Wed May 11 15:55:02 2022 +1000)

# 0.12.7 - Maintenance Release

* 6bf1e9aa - chore: switch from logging crate to tracing crate (Ronald Holshausen, Wed May 11 14:57:37 2022 +1000)
* e8e5cb5a - chore: Upgrade dependencies (Ronald Holshausen, Wed May 11 12:57:29 2022 +1000)
* 37bfc5de - chore: Upgrade pact-plugin-driver to 0.1.6 (Ronald Holshausen, Wed May 11 11:56:23 2022 +1000)
* 020b5715 - chore: upgrade pact_models to 0.4.1 (Ronald Holshausen, Wed May 11 11:36:57 2022 +1000)
* 45e1d194 - bump version to 0.12.7 (Ronald Holshausen, Wed Apr 27 14:21:29 2022 +1000)

# 0.12.6 - Maintenance Release

* dba7252e - chore: Upgrade pact-plugin-driver to 0.1.5 (Ronald Holshausen, Tue Apr 26 13:56:22 2022 +1000)
* 688e49e7 - chore: Upgrade pact-plugin-driver to 0.1.4 (Ronald Holshausen, Fri Apr 22 14:47:01 2022 +1000)
* cdf72b05 - feat: forward provider details to plugin when verifying (Ronald Holshausen, Fri Apr 22 14:12:34 2022 +1000)
* 2395143a - feat: forward verification to plugin for transports provided by the plugin (Ronald Holshausen, Fri Apr 22 12:02:05 2022 +1000)
* 49640c5f - chore: minor update to release scripts (Ronald Holshausen, Wed Apr 13 15:32:46 2022 +1000)
* dce0ec9d - bump version to 0.12.6 (Ronald Holshausen, Wed Apr 13 15:30:39 2022 +1000)

# 0.12.5 - Maintenance Release

* d043f6c7 - chore: upgrade pact_models to 0.3.3 (Ronald Holshausen, Wed Apr 13 15:24:33 2022 +1000)
* eee09ba6 - chore: Upgrade pact-plugin-driver to 0.1.3 (Ronald Holshausen, Wed Apr 13 14:07:36 2022 +1000)
* 73ae0ef0 - fix: Upgrade reqwest to 0.11.10 to resolve #156 (Ronald Holshausen, Wed Apr 13 13:31:55 2022 +1000)
* ffeca2e2 - chore: update to the latest plugin driver (Ronald Holshausen, Wed Apr 13 13:08:25 2022 +1000)
* 6f85c8bc - bump version to 0.12.5 (Ronald Holshausen, Thu Mar 24 13:56:30 2022 +1100)

# 0.12.4 - Maintenance Release

* 9baf03a9 - chore: use the published version of the plugin driver (Ronald Holshausen, Thu Mar 24 13:36:01 2022 +1100)
* 345b0011 - feat: support mock servers provided from plugins (Ronald Holshausen, Mon Mar 21 15:59:46 2022 +1100)
* 68207eb6 - bump version to 0.12.4 (Ronald Holshausen, Fri Mar 4 14:00:31 2022 +1100)

# 0.12.3 - Maintenance Release

* fbcec27a - chore: Upgrade pact-models to 0.3.0 (Ronald Holshausen, Fri Mar 4 12:23:49 2022 +1100)
* 5f148cdd - feat: capture all the output from the verifier (Ronald Holshausen, Thu Jan 27 16:08:02 2022 +1100)
* 43754e6d - fix: PACT_DO_NOT_TRACK should be upper case (Ronald Holshausen, Thu Jan 27 14:34:13 2022 +1100)
* 5e4c68ef - chore: update pact matching to 0.12.2 (Ronald Holshausen, Mon Jan 17 16:29:21 2022 +1100)
* 9bb56cfe - bump version to 0.12.3 (Ronald Holshausen, Mon Jan 17 11:29:51 2022 +1100)

# 0.12.2 - Bugfix Release

* 80b241c5 - chore: Upgrade plugin driver crate to 0.0.17 (Ronald Holshausen, Mon Jan 17 11:22:48 2022 +1100)
* 4f1ecff2 - chore: Upgrade pact-models to 0.2.7 (Ronald Holshausen, Mon Jan 17 10:53:26 2022 +1100)
* c2089645 - fix: log crate version must be fixed across all crates (including plugin driver) (Ronald Holshausen, Fri Jan 14 16:10:50 2022 +1100)
* d670585a - chore: Update plugin driver to 0.0.16 (Ronald Holshausen, Tue Jan 4 09:37:21 2022 +1100)
* 9c2810ad - chore: Upgrade pact-plugin-driver to 0.0.15 (Ronald Holshausen, Fri Dec 31 15:12:56 2021 +1100)
* f8e718ac - bump version to 0.12.2 (Ronald Holshausen, Fri Dec 31 10:05:17 2021 +1100)

# 0.12.1 - Bugfix Release

* dfa9f614 - fix: Values matcher should not be applied to a slice like Equality (Ronald Holshausen, Thu Dec 30 16:23:41 2021 +1100)
* 3576c857 - chore: fix compiler warning (Ronald Holshausen, Thu Dec 30 14:24:58 2021 +1100)
* aab26798 - bump version to 0.12.1 (Ronald Holshausen, Thu Dec 30 14:22:19 2021 +1100)

# 0.12.0 - Support for matching Protobuf payloads

* 1a01d111 - fix: correct the matching logic with lists and eachkey/eachvalue matchers (Ronald Holshausen, Thu Dec 30 13:34:21 2021 +1100)
* 28f562e2 - fix: Each key matching was not implemented correctly (Ronald Holshausen, Wed Dec 29 17:20:05 2021 +1100)
* 07e2a3b6 - fix: Values matchers must not cascade (Ronald Holshausen, Wed Dec 29 16:36:57 2021 +1100)
* 60764855 - fix: missing import (Ronald Holshausen, Wed Dec 29 15:50:36 2021 +1100)
* cd6fe27a - fix: map matching logic was not including the EachValue matcher (Ronald Holshausen, Wed Dec 29 15:47:09 2021 +1100)
* 0a6e7d9d - refactor: Convert MatchingContext to a trait and use DocPath instead of string slices (Ronald Holshausen, Wed Dec 29 14:24:39 2021 +1100)
* 41b406aa - fix: shared mime-info db not available on Windows (Ronald Holshausen, Wed Dec 29 10:13:43 2021 +1100)
* a0c9d203 - fix: detect common text types when comparing content type (Ronald Holshausen, Fri Dec 24 16:19:43 2021 +1100)
* ede663ec - fix: add matching implementations for Vec<u8> and &Vec<u8> (Ronald Holshausen, Fri Dec 24 15:24:48 2021 +1100)
* 85a4ae53 - chore: Make pact_matching::matchingrules public so it can be used outside of the crate (Ronald Holshausen, Thu Dec 23 16:38:54 2021 +1100)
* 8089b542 - bump version to 0.11.6 (Ronald Holshausen, Thu Dec 23 13:04:14 2021 +1100)

# 0.11.5 - Maintenance Release

* 5479a634 - chore: Update pact_models (0.2.4) and pact-plugin-driver (0.0.14) (Ronald Holshausen, Thu Dec 23 12:57:02 2021 +1100)
* 226795ee - bump version to 0.11.5 (Ronald Holshausen, Mon Dec 20 12:19:00 2021 +1100)

# 0.11.4 - Bugfix Release

* 8911d5b0 - chore: update to latest plugin driver crate (metrics fixes) (Ronald Holshausen, Mon Dec 20 12:11:35 2021 +1100)
* 25d8cd9b - fix(metrics): swap uid for cid (Matt Fellows, Fri Dec 17 15:48:42 2021 +1100)
* ef29bc17 - chore: make json::compare_json public so it can be used by other crates (Ronald Holshausen, Fri Dec 17 13:55:22 2021 +1100)
* d4a46381 - bump version to 0.11.4 (Ronald Holshausen, Wed Dec 15 10:06:34 2021 +1100)

# 0.11.3 - Bugfix Release

* 48d061ef - feat: add metrics publishing to matching crate (Ronald Holshausen, Tue Dec 14 16:19:59 2021 +1100)
* 4f1ba7d9 - chore: update to the latest plugin driver (Ronald Holshausen, Tue Dec 14 13:55:02 2021 +1100)
* ecb6afbe - bump version to 0.11.3 (Ronald Holshausen, Thu Dec 2 11:58:27 2021 +1100)

# 0.11.2 - Upgrade to latest models and plugins crates

* c1eca940 - chore(pact_matching): upgrade to latest models and plugins crate (Ronald Holshausen, Thu Dec 2 11:49:32 2021 +1100)
* 2db6a46f - refactor: test_env_log has been replaced with test_log (Ronald Holshausen, Tue Nov 23 16:15:02 2021 +1100)
* fc5be202 - fix: update to latest driver crate (Ronald Holshausen, Tue Nov 16 16:19:02 2021 +1100)
* 718cdbcf - bump version to 0.11.2 (Ronald Holshausen, Tue Nov 16 12:10:26 2021 +1100)

# 0.11.1 - Updated to latest models crate

* 5d974c4a - chore: update to latest models and plugin driver crates (Ronald Holshausen, Tue Nov 16 11:56:53 2021 +1100)
* 1e76c400 - chore: correct readme (Ronald Holshausen, Wed Nov 3 17:09:56 2021 +1100)
* b8da65c2 - bump version to 0.11.1 (Ronald Holshausen, Wed Nov 3 17:07:06 2021 +1100)

# 0.11.0 - Pact V4 release

* e62cefdc - chore: fix clippy warnings (Ronald Holshausen, Wed Nov 3 16:52:24 2021 +1100)
* bd2bd0ec - chore: drop beta from pact_matching version (Ronald Holshausen, Wed Nov 3 13:28:35 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* a561f883 - chore: use the non-beta models crate (Ronald Holshausen, Thu Oct 21 18:10:27 2021 +1100)
* 9f674ca3 - bump version to 0.11.0-beta.5 (Ronald Holshausen, Tue Oct 19 17:16:10 2021 +1100)

# 0.11.0-beta.4 - Bugfix Release

* 918e5beb - fix: update to latest models and plugin driver crates (Ronald Holshausen, Tue Oct 19 17:09:48 2021 +1100)
* 8f8ad35a - bump version to 0.11.0-beta.4 (Ronald Holshausen, Tue Oct 19 11:17:01 2021 +1100)

# 0.11.0-beta.3 - Support matching synchronous request/response messages

* aa434ba3 - chore: update to latest driver crate (Ronald Holshausen, Tue Oct 19 11:09:46 2021 +1100)
* df386c8a - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Oct 18 13:41:36 2021 +1100)
* 2b4b7cc3 - feat(plugins): Support matching synchronous request/response messages (Ronald Holshausen, Fri Oct 15 16:01:50 2021 +1100)
* db3c6268 - bump version to 0.11.0-beta.3 (Ronald Holshausen, Tue Oct 12 16:17:18 2021 +1100)

# 0.11.0-beta.2 - synchronous messages with plugins

* d0bfb8a8 - feat: Support consumer tests with synchronous messages (Ronald Holshausen, Tue Oct 12 15:51:08 2021 +1100)
* 48d662a8 - chore: add docs about the matching rule definition language (Ronald Holshausen, Thu Oct 7 13:29:16 2021 +1100)
* 605aa158 - chore: Update the matching readme with the new V4 matchers (Ronald Holshausen, Thu Oct 7 10:49:48 2021 +1100)
* de5bbeef - bump version to 0.11.0-beta.2 (Ronald Holshausen, Wed Oct 6 11:30:22 2021 +1100)

# 0.11.0-beta.1 - Plugin support (driver version 0.0.3)

* 2c47023c - chore: pin plugin driver version to 0.0.3 (Ronald Holshausen, Wed Oct 6 11:21:07 2021 +1100)
* 288e2168 - chore: use the published version of the plugin driver lib (Ronald Holshausen, Tue Oct 5 15:36:06 2021 +1100)
* 57ba661a - chore: fix tests after removing Deserialize, Serialize from Message (Ronald Holshausen, Tue Oct 5 14:55:58 2021 +1100)
* 5525b039 - feat(plugins): cleaned up the verfier output (Ronald Holshausen, Thu Sep 30 16:19:15 2021 +1000)
* ee3212a8 - refactor(plugins): do not expose the catalogue statics, but rather a function to initialise it (Ronald Holshausen, Tue Sep 14 15:13:12 2021 +1000)
* b71dcabf - refactor(plugins): rename ContentTypeOverride -> ContentTypeHint (Ronald Holshausen, Tue Sep 14 15:08:52 2021 +1000)
* 7eb10081 - bump version to 0.11.0-beta.1 (Ronald Holshausen, Mon Sep 13 11:03:28 2021 +1000)

# 0.11.0-beta.0 - Plugin support + nontEmpty and semver matchers

* 716809f6 - chore: Get CI build passing (Ronald Holshausen, Fri Sep 10 14:55:46 2021 +1000)
* ceb1c35f - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 7 10:07:45 2021 +1000)
* 6c0070ba - bump version to 0.10.4 (Ronald Holshausen, Sat Sep 4 15:39:07 2021 +1000)
* b77498c8 - chore: fix tests after updating plugin API (Ronald Holshausen, Fri Sep 3 16:48:18 2021 +1000)
* c0bdd359 - fix: PluginData configuration is optional (Ronald Holshausen, Thu Sep 2 15:37:01 2021 +1000)
* e8ae81b3 - refactor: matching req/res with plugins requires data from the pact and interaction (Ronald Holshausen, Thu Sep 2 11:57:50 2021 +1000)
* 474b803e - feat(V4): added nontEmpty and semver matchers (Ronald Holshausen, Tue Aug 31 11:58:18 2021 +1000)
* b9aa7ecb - feat(Plugins): allow plugins to override text/binary format of the interaction content (Ronald Holshausen, Mon Aug 30 10:48:04 2021 +1000)
* eb34b011 - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Aug 23 15:48:55 2021 +1000)
* 0c5cede2 - chore: bump models crate to 0.2 (Ronald Holshausen, Mon Aug 23 12:56:14 2021 +1000)
* 75e13fd8 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Aug 23 10:33:45 2021 +1000)
* e3a2660f - chore: fix tests after updating test builders to be async (Ronald Holshausen, Fri Aug 20 12:41:10 2021 +1000)
* 779f099c - feat(plugins): Got generators from plugin working (Ronald Holshausen, Thu Aug 19 17:20:47 2021 +1000)
* b75fea5d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Aug 18 12:27:41 2021 +1000)
* 5a235414 - feat(plugins): order the matching results as plugins mau return them in any order (Ronald Holshausen, Fri Aug 13 17:18:46 2021 +1000)
* 2662241e - feat(plugins): Call out to plugins when comparing content owned by the plugin during verification (Ronald Holshausen, Fri Aug 13 14:29:30 2021 +1000)
* 60869969 - feat(plugins): Add core features to the plugin catalogue (Ronald Holshausen, Thu Aug 12 13:00:41 2021 +1000)

# 0.10.3 - Upgrade treemagic => upgrade nom => upgrade memchr

* 42de0a5d - chore: skip tests requiring mime db in Windows (Ronald Holshausen, Sat Sep 4 14:46:32 2021 +1000)
* 4e97e806 - chore: do not include tree_magic_db to keep the project license MIT (Ronald Holshausen, Sat Sep 4 14:15:42 2021 +1000)
* 3cf339ae - chore: upgrade tree_magic_mini to 3.0.1 (Audun Halland, Tue Aug 31 11:08:48 2021 +0200)
* cfdc475c - bump version to 0.10.3 (Ronald Holshausen, Sun Aug 22 15:22:12 2021 +1000)

# 0.10.2 - upgrade nom to 7.0

* 21be6bce - chore: upgrade nom to 7.0 #128 (Ronald Holshausen, Sun Aug 22 11:56:33 2021 +1000)
* e07a1b84 - bump version to 0.10.2 (Ronald Holshausen, Tue Aug 17 10:22:37 2021 +1000)

# 0.10.1 - Bugfix Release

* 8bcd1c7e - fix: min/max type matchers must not apply the limits when cascading (Ronald Holshausen, Sun Aug 8 15:50:40 2021 +1000)
* 0b347204 - chore: update spec test cases (Ronald Holshausen, Sun Aug 8 14:29:43 2021 +1000)
* 6124ed0b - refactor: Introduce DocPath struct for path expressions (Caleb Stepanian, Thu Jul 29 12:27:32 2021 -0400)
* 26f642e7 - bump version to 0.10.1 (Ronald Holshausen, Fri Jul 23 14:15:58 2021 +1000)

# 0.10.0 - Final version after extracting models

* 9baa714d - chore: bump minor version of matching crate (Ronald Holshausen, Fri Jul 23 14:03:20 2021 +1000)
* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* 458fdd15 - refactor: Move path expression functions into path_exp module (Caleb Stepanian, Mon Jul 19 14:22:02 2021 -0400)
* 3dccf866 - refacfor: moved the pact structs to the models crate (Ronald Holshausen, Sun Jul 18 16:58:14 2021 +1000)
* e8046d84 - refactor: moved interaction structs to the models crate (Ronald Holshausen, Sun Jul 18 14:36:03 2021 +1000)
* 31873ee3 - feat: added validation of provider state JSON (Ronald Holshausen, Wed Jul 14 15:44:20 2021 +1000)
* 82a2d5df - feat: added verification of req/res interaction (Ronald Holshausen, Mon Jul 12 16:57:04 2021 +1000)
* 331211da - bump version to 0.9.5 (Ronald Holshausen, Sun Jul 11 16:53:33 2021 +1000)

# 0.9.4 - Moved structs to models crate + bugfixes and enhancements

* e2151800 - feat: support generating UUIDs with different formats #121 (Ronald Holshausen, Sun Jul 11 12:36:23 2021 +1000)
* e2e10241 - refactor: moved Request and Response structs to the models crate (Ronald Holshausen, Wed Jul 7 18:09:36 2021 +1000)
* 2c3c6ac0 - refactor: moved the header, body and query functions to the model crate (Ronald Holshausen, Wed Jul 7 16:37:28 2021 +1000)
* 9e8b01d7 - refactor: move HttpPart struct to models crate (Ronald Holshausen, Wed Jul 7 15:59:34 2021 +1000)
* 10e8ef87 - refactor: moved http_utils to the models crate (Ronald Holshausen, Wed Jul 7 14:34:20 2021 +1000)
* ed73b98a - chore: fix compiler warnings (Ronald Holshausen, Wed Jul 7 13:54:53 2021 +1000)
* 1282378d - chore: use json_to_string on metadata for parsing content-type (Matt Fellows, Tue Jul 6 14:30:03 2021 +1000)
* 33f9a823 - feat: support complex data structures in message metadata (Matt Fellows, Mon Jul 5 23:38:52 2021 +1000)
* a835e684 - feat: support message metadata in verifications (Matt Fellows, Sun Jul 4 21:02:35 2021 +1000)
* 01ff9877 - refactor: moved matching rules and generators to models crate (Ronald Holshausen, Sun Jul 4 17:17:30 2021 +1000)
* 357b2390 - refactor: move path expressions to models crate (Ronald Holshausen, Sun Jul 4 15:31:36 2021 +1000)
* 80e3c4e7 - fix: retain the data type for simple expressions #116 (Ronald Holshausen, Sun Jul 4 13:02:43 2021 +1000)
* e21db699 - fix: Keep the original value when injecting from a provider state value so data type is retained #116 (Ronald Holshausen, Sat Jul 3 18:01:34 2021 +1000)
* c3c22ea8 - Revert "refactor: moved matching rules and generators to models crate (part 1)" (Ronald Holshausen, Wed Jun 23 14:37:46 2021 +1000)
* d3406650 - refactor: moved matching rules and generators to models crate (part 1) (Ronald Holshausen, Wed Jun 23 12:58:30 2021 +1000)
* 9b7ad27d - refactor: moved xml_utils to models crate (Ronald Holshausen, Tue Jun 22 16:30:06 2021 +1000)
* 4db98181 - refactor: move file_utils to the models crate (Ronald Holshausen, Tue Jun 22 16:06:02 2021 +1000)
* 56da2e11 - bump version to 0.9.4 (Ronald Holshausen, Tue Jun 22 15:22:29 2021 +1000)

# 0.9.3 - Refactor + Bugfixes

* 4fe8383c - chore: upgrade nom to 6.2.0 to resolve lexical-core compiler error (Ronald Holshausen, Tue Jun 22 14:41:16 2021 +1000)
* bbc638be - feat(pact file verification): verify consumer and provider sections (Ronald Holshausen, Fri Jun 18 16:52:15 2021 +1000)
* a7c071bc - feat(pact-file-validation): implemented validation of the metadata section (Ronald Holshausen, Wed Jun 16 09:17:28 2021 +1000)
* 0652139e - feat(file-validator): check for additional properties in the root (Ronald Holshausen, Mon Jun 14 16:45:20 2021 +1000)
* 00b65dcf - chore: rename pact_file_verifier -> pact_cli (Ronald Holshausen, Mon Jun 14 14:08:24 2021 +1000)
* db75a42a - refactor: seperate displaying errors from gathering results in the verifier (Ronald Holshausen, Fri Jun 11 14:35:40 2021 +1000)
* 6198538d - refactor: move time_utils to pact_models crate (Ronald Holshausen, Fri Jun 11 12:58:26 2021 +1000)
* 5c670814 - refactor: move expression_parser to pact_models crate (Ronald Holshausen, Fri Jun 11 10:51:51 2021 +1000)
* 457aa5fc - fix(V4): Status code matcher was not converted to JSON correctly (Ronald Holshausen, Sun Jun 6 12:53:37 2021 +1000)
* 696ffb6a - chore: fix failing test #113 (Ronald Holshausen, Sat Jun 5 15:13:41 2021 +1000)
* a44cbbee - fix: verifier was returning a mismatch when the expected body is empty #113 (Ronald Holshausen, Sat Jun 5 15:07:22 2021 +1000)
* 4e328d93 - feat: implement verification for RequestResponsePact, Consumer, Provider (Ronald Holshausen, Thu Jun 3 16:59:23 2021 +1000)
* 2f678213 - feat: initial prototype of a pact file verifier (Ronald Holshausen, Thu Jun 3 14:56:16 2021 +1000)
* 4038e611 - chore: add missing matches_with implementation (Ronald Holshausen, Tue Jun 1 15:36:33 2021 +1000)
* 68f8f84e - chore: skip failing tests in alpine to get the build going (Ronald Holshausen, Tue Jun 1 13:47:20 2021 +1000)
* 6d058529 - chore: fix some clippy warnings (Ronald Holshausen, Mon May 31 16:42:57 2021 +1000)
* 17beef62 - feat: support accumulating log entries per running mock server (Ronald Holshausen, Mon May 31 15:09:20 2021 +1000)
* e27ce896 - bump version to 0.9.3 (Ronald Holshausen, Sun May 30 10:45:06 2021 +1000)

# 0.9.2 - Bugfixes + V4 changes (Status code matcher + Pending flag)

* 44e7eb4 - chore: cleanup deprecation warnings (Ronald Holshausen, Sat May 29 17:55:04 2021 +1000)
* f24bbbc - refactor: decouple matching rule logic from matching rule model (Ronald Holshausen, Sat May 29 17:44:12 2021 +1000)
* a7b81af - chore: fix clippy violation (Ronald Holshausen, Sat May 29 17:29:06 2021 +1000)
* 7022625 - refactor: move provider state models to the pact models crate (Ronald Holshausen, Sat May 29 17:18:48 2021 +1000)
* a84151c - refactor(V4): reuse the extract common message parts with synchronous messages (Ronald Holshausen, Sat May 29 17:06:04 2021 +1000)
* ef37cb9 - refactor(V4): extract common message parts into a seperate struct (Ronald Holshausen, Sat May 29 16:38:38 2021 +1000)
* 59e23f4 - fix: message pact needed matchingrules + generators (Matt Fellows, Sat May 29 15:16:32 2021 +1000)
* ebb11df - feat(V4): fixed test _ refactored types for match functions (Ronald Holshausen, Sat May 29 14:56:31 2021 +1000)
* 73a53b8 - feat(V4): add an HTTP status code matcher (Ronald Holshausen, Fri May 28 18:40:11 2021 +1000)
* 81eed06 - fix: add zip file for binary test (Matt Fellows, Thu May 27 22:55:40 2021 +1000)
* cf679bd - fix: message pact feature test (Matt Fellows, Thu May 27 21:33:24 2021 +1000)
* 84d79a1 - fix: message pact feature test (Matt Fellows, Thu May 27 21:05:38 2021 +1000)
* a7e5778 - fix: broken message test (Matt Fellows, Thu May 27 15:36:31 2021 +1000)
* db6e8b2 - feat(V4): added some tests for pending interactions (Ronald Holshausen, Thu May 27 17:08:46 2021 +1000)
* 7e4caf8 - feat(V4): added a pending flag to V4 interactions (Ronald Holshausen, Thu May 27 16:59:18 2021 +1000)
* 8e8075b - refactor: move some more structs to the models crate (Ronald Holshausen, Thu May 27 14:34:03 2021 +1000)
* 0fcb371 - chore: ignore failing message interaction tests for now (Ronald Holshausen, Thu May 27 14:32:03 2021 +1000)
* 0c9391d - Merge pull request #101 from pact-foundation/feat/ffi-consumer-message-pact (Matt Fellows, Thu May 27 12:57:58 2021 +1000)
* 810106d - Merge pull request #100 from pact-foundation/feat/more-consumer-ffi-matchers (Ronald Holshausen, Thu May 27 11:17:53 2021 +1000)
* ffbcaf5 - feat: Added header_from_provider_state and path_from_provider_state (Rob Caiger, Mon May 24 13:54:16 2021 +0100)
* 5024e17 - feat: allow messages to have binary payloads (Matt Fellows, Sat May 22 21:50:57 2021 +1000)
* 413e9a5 - feat: initial consumer FFI based message pact (Matt Fellows, Tue May 18 23:37:49 2021 +1000)
* 066d7a9 - Revert "feat: support a dummy matcher" (Matt Fellows, Mon May 17 10:47:40 2021 +1000)
* 5167cfb - fix: broken test for v2 path matcher (Matt Fellows, Sun May 16 14:29:50 2021 +1000)
* a33718a - fix: serialise v2 path matcher correctly for FFI (Matt Fellows, Sun May 16 14:10:14 2021 +1000)
* f56ae24 - feat: support a dummy matcher (Matt Fellows, Sun May 16 14:05:08 2021 +1000)
* fbd5ae3 - bump version to 0.9.2 (Ronald Holshausen, Tue May 4 10:13:02 2021 +1000)

# 0.9.1 - V4 features

* 6b2da7d - feat(V4): added a boolean matcher (Ronald Holshausen, Sun May 2 12:57:09 2021 +1000)
* e3a71a3 - bump version to 0.9.1 (Ronald Holshausen, Sun Apr 25 14:09:22 2021 +1000)

# 0.9.0 - Extracted some models to pact_models + V4 spec updates

* 735c9e7 - chore: bump pact_matching to 0.9 (Ronald Holshausen, Sun Apr 25 13:50:18 2021 +1000)
* fb373b4 - chore: bump version to 0.0.2 (Ronald Holshausen, Sun Apr 25 13:40:52 2021 +1000)
* 5ea36db - refactor: move content handling functions to pact_models crate (Ronald Holshausen, Sun Apr 25 13:12:22 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* f06690e - chore: cleanup deprecation warnings (Ronald Holshausen, Sun Apr 25 11:36:57 2021 +1000)
* 3dd610a - refactor: move structs and code dealing with bodies to a seperate package (Ronald Holshausen, Sun Apr 25 11:20:47 2021 +1000)
* a725ab1 - feat(V4): added synchronous request/response message formats (Ronald Holshausen, Sat Apr 24 16:05:12 2021 +1000)
* 80b7148 - feat(V4): Updated consumer DSL to set comments + mock server initial support for V4 pacts (Ronald Holshausen, Fri Apr 23 17:58:10 2021 +1000)
* 4264821 - feat(V4): add an optional comments to the interaction (Ronald Holshausen, Fri Apr 23 09:04:56 2021 +1000)
* 4bcd94f - refactor: moved OptionalBody and content types to pact models crate (Ronald Holshausen, Thu Apr 22 14:01:56 2021 +1000)
* 80812d0 - refactor: move Consumer and Provider structs to models crate (Ronald Holshausen, Thu Apr 22 13:11:03 2021 +1000)
* 220fb5e - refactor: move the PactSpecification enum to the pact_models crate (Ronald Holshausen, Thu Apr 22 11:18:26 2021 +1000)
* 83d3d60 - chore: bump version to 0.0.1 (Ronald Holshausen, Thu Apr 22 10:52:04 2021 +1000)
* 34e7dcd - chore: add a pact models crate (Ronald Holshausen, Thu Apr 22 10:04:40 2021 +1000)
* a0f6a1d - refactor: Use Anyhow instead of `io::Result` (Caleb Stepanian, Wed Apr 7 16:17:35 2021 -0400)

# 0.8.14 - Bugfix Release

* 75c2c1a - fix: upgrade to tree_magic_mini 2.0.0 because they pulled 1.0.0 from crates.io and now builds fail (Matt Fellows, Wed Apr 7 12:53:26 2021 +1000)
* 4f48223 - chore: add some tests for process_object (Ronald Holshausen, Fri Mar 26 15:00:32 2021 +1100)
* 41652e2 - bump version to 0.8.14 (Ronald Holshausen, Sun Mar 14 14:34:21 2021 +1100)

# 0.8.13 - V4 features (message refactor)

* b7c80e8 - chore: update specification tests (Ronald Holshausen, Mon Mar 8 15:48:34 2021 +1100)
* 4fe65fb - feat(V4): Update matching code to use matchingRules.content for V4 messages (Ronald Holshausen, Mon Mar 8 12:10:31 2021 +1100)
* 86f8140 - fix: missing $ in macro (Ronald Holshausen, Sun Mar 7 18:48:09 2021 +1100)
* 81de3d3 - feat(V4): Move message pact content matching rules from matchingRules.body to matchingRules.content (Ronald Holshausen, Sun Mar 7 17:47:27 2021 +1100)
* 127e6f2 - refactor: converted V4 interaction models from enum structs to plain structs (Ronald Holshausen, Sun Mar 7 17:17:15 2021 +1100)
* b71a13f - bump version to 0.8.13 (Ronald Holshausen, Fri Mar 5 11:07:43 2021 +1100)

# 0.8.12 - Values matcher - match values, ignoring keys

* bc84a4d - feat: implemented matching values ignoring keys (Ronald Holshausen, Fri Mar 5 10:52:01 2021 +1100)
* 16f736c - bump version to 0.8.12 (Ronald Holshausen, Wed Feb 10 15:21:46 2021 +1100)

# 0.8.11 - file locking with exp backoff

* 6f13f49 - feat: implemented non-blocking write file locking with exp backoff (Ronald Holshausen, Wed Feb 10 14:50:38 2021 +1100)
* f34629e - feat: implemented non-blockng read file locking with exp backoff (Ronald Holshausen, Wed Feb 10 14:27:31 2021 +1100)
* 7f054e8 - fix: correctly assemble UTF-8 percent encoded query parameters (Ronald Holshausen, Tue Feb 9 14:02:04 2021 +1100)
* aae3c01 - bump version to 0.8.11 (Ronald Holshausen, Mon Feb 8 15:18:29 2021 +1100)

# 0.8.10 - use a file system lock when merging pact files

* eae1b16 - chore: fix clippy errors (Ronald Holshausen, Mon Feb 8 14:57:42 2021 +1100)
* 4442949 - chore: keep the file locked for the smallest possible window (Ronald Holshausen, Mon Feb 8 14:34:22 2021 +1100)
* 9976e80 - feat: added read locks and a mutex guard to reading and writing pacts (Ronald Holshausen, Mon Feb 8 11:58:52 2021 +1100)
* 61e16ed - feat: use a file system lock when merging pact files (Ronald Holshausen, Sun Feb 7 17:00:29 2021 +1100)
* 49a3cf2 - refactor: use bytes crate instead of vector of bytes for body content (Ronald Holshausen, Sun Feb 7 14:43:40 2021 +1100)
* 48997d6 - bump version to 0.8.10 (Ronald Holshausen, Mon Jan 25 10:07:31 2021 +1100)

# 0.8.9 - Fixes + thread safe support functions

* c8f7091 - feat: made pact broker module public so it can be used by other crates (Ronald Holshausen, Sun Jan 24 18:24:30 2021 +1100)
* 53232b4 - chore: added some thread safe support functions to Interaction trait (Ronald Holshausen, Sun Jan 24 14:38:06 2021 +1100)
* 93ae06d - chore: add function to clone and wrap an interaction in a box (Ronald Holshausen, Sun Jan 24 12:20:37 2021 +1100)
* a35150b - feat: extracted the JSON -> Pact logic into a public function (Ronald Holshausen, Sun Jan 24 10:16:33 2021 +1100)
* ae95e0c - fix: apply generators to the request in the same manor as the response (Ronald Holshausen, Mon Jan 18 17:25:38 2021 +1100)
* 1c0cfba - bump version to 0.8.9 (Ronald Holshausen, Mon Jan 11 09:57:30 2021 +1100)

# 0.8.8 - Bugfixes + updated dependencies

* 56ce20a - fix: MockServerURL generator was using the incorrect field (Ronald Holshausen, Sun Jan 10 15:54:18 2021 +1100)
* 2bcf8fa - Merge pull request #88 from audunhalland/upgrade-http (Ronald Holshausen, Sun Jan 10 10:33:50 2021 +1100)
* 5e5c320 - chore: upgrade rand, rand_regex (Audun Halland, Sat Jan 9 09:33:13 2021 +0100)
* 3a28a6c - chore: upgrade regex, chrono-tz (Audun Halland, Sat Jan 9 11:12:49 2021 +0100)
* 1483fef - chore: upgrade uuid to 0.8 (Audun Halland, Sat Jan 9 11:03:30 2021 +0100)
* 9a8a63f - chore: upgrade quickcheck (Audun Halland, Sat Jan 9 08:46:51 2021 +0100)
* 245a5d6 - chore: upgrade indextree (Audun Halland, Sat Jan 9 08:35:59 2021 +0100)
* 5b60ec0 - chore: get rid of nom dupes by using tree_magic_mini (Audun Halland, Sat Jan 9 08:06:22 2021 +0100)
* 13d295a - fixing an incorrect use of sort() (Steve Cooper, Thu Jan 7 20:56:35 2021 +0000)
* 422cfb9 - request headers generated unexpected differences when the pact was serialised and deserialised. fixed by sorting header keys during comparison (Steve Cooper, Thu Jan 7 17:47:45 2021 +0000)
* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* df77b63 - bump version to 0.8.8 (Ronald Holshausen, Tue Jan 5 12:29:55 2021 +1100)

# 0.8.7 - Updated dependencies

* e78969d - refactor: Generator body handler now returns a Result (Ronald Holshausen, Mon Jan 4 15:47:28 2021 +1100)
* 7b8d74e - deps: remove httparse (Audun Halland, Sun Jan 3 04:51:32 2021 +0100)
* a7a96fc - deps(pact_matching): remove formdata, add multipart (Audun Halland, Sun Jan 3 00:19:28 2021 +0100)
* 51ff156 - chore: cleanup rustdoc warnings (Ronald Holshausen, Wed Dec 30 15:29:44 2020 +1100)
* d7d86d2 - bump version to 0.8.7 (Ronald Holshausen, Wed Dec 30 15:10:01 2020 +1100)

# 0.8.6 - support generators associated with array contains matcher variants

* c180d2c - chore: Implement Hash and PartialEq for Generator (Ronald Holshausen, Wed Dec 30 14:31:13 2020 +1100)
* 5556b32 - feat: added test for array contains as a generator (Ronald Holshausen, Wed Dec 30 13:47:31 2020 +1100)
* bb73fd7 - fix(clippy) cleaned up some clippy warnings (Ronald Holshausen, Wed Dec 30 10:32:19 2020 +1100)
* 0a70d64 - fix(clippy): using `clone` on a double-reference; this will copy the reference instead of cloning the inner type (Ronald Holshausen, Wed Dec 30 09:41:12 2020 +1100)
* 1ed95ae - feat: implemented using ArrayContains as a generator for JSON (Ronald Holshausen, Tue Dec 29 17:52:25 2020 +1100)
* f2086d8 - chore: cleanup warnings (Ronald Holshausen, Tue Dec 29 15:46:46 2020 +1100)
* f83804c - chore: upgrade crates in pact_matching (Ronald Holshausen, Tue Dec 29 15:21:50 2020 +1100)
* 9852811 - fix(clippy): you are implementing `Hash` explicitly but have derived `PartialEq` (Ronald Holshausen, Tue Dec 29 14:28:22 2020 +1100)
* 42f0a39 - fix: use Vec instead of HashSet to maintain order of matching rules on OSX (Ronald Holshausen, Tue Dec 29 13:22:57 2020 +1100)
* c8ad6d4 - fix: matchers in Pact file can have a different order on OSX (Ronald Holshausen, Tue Dec 29 12:49:19 2020 +1100)
* 09513de - feat: add verifiedBy to the verified results (Ronald Holshausen, Tue Dec 29 12:05:07 2020 +1100)
* 5e56ecb - refactor: support generators associated with array contains matcher variants (Ronald Holshausen, Tue Dec 29 11:46:56 2020 +1100)
* ab16c08 - bump version to 0.8.6 (Matt Fellows, Sun Nov 22 23:52:51 2020 +1100)

# 0.8.5 - Bugfix Release

* 5058a2d - feat: include the mockserver URL and port in the verification context (Ronald Holshausen, Fri Nov 20 16:43:10 2020 +1100)
* 09b197d - feat: add a mock server URL generator (Ronald Holshausen, Fri Nov 20 13:24:09 2020 +1100)
* 118daa1 - feat: when merging pact files, upcast to the higher spec version (Ronald Holshausen, Thu Nov 19 18:09:13 2020 +1100)
* 6995298 - fix: make application/xml equivalent to text/xml (Ronald Holshausen, Thu Nov 19 14:33:58 2020 +1100)
* 88eff15 - fix: when matching bodies, use any content type header matcher (Ronald Holshausen, Thu Nov 19 14:19:08 2020 +1100)
* 12a3c43 - bump version to 0.8.5 (Ronald Holshausen, Tue Nov 17 16:36:15 2020 +1100)

# 0.8.4 - Support provider state injected values

* 850282d - fix: times with millisecond precision less 3 caused chronos to panic (Ronald Holshausen, Tue Nov 17 16:29:47 2020 +1100)
* baf3693 - fix: when displaying diff, if actual body was empty a panic resulted (Ronald Holshausen, Tue Nov 17 16:29:12 2020 +1100)
* 13ce2f2 - fix: introduce GeneratorTestMode and restrict provider state generator to the provider side (Ronald Holshausen, Mon Nov 16 15:00:01 2020 +1100)
* 4cb3c26 - bump version to 0.8.4 (Matt Fellows, Wed Nov 11 11:08:37 2020 +1100)

# 0.8.3 - Bugfix Release

* 8dccd1a - Merge pull request #79 from pact-foundation/feat/pacts-for-verification (Ronald Holshausen, Wed Nov 11 09:43:37 2020 +1100)
* b3cca0d - test: add basic pact test for pacts for verification feature (Matt Fellows, Wed Nov 11 00:30:45 2020 +1100)
* e7f729d - wip: further cleanup, and obfuscate auth details (Matt Fellows, Tue Nov 10 13:56:02 2020 +1100)
* 80f4e98 - wip: refactor BrokerWithDynamicConfiguration into a struct enum for better readability (Matt Fellows, Mon Nov 9 22:40:24 2020 +1100)
* 60eb190 - wip: map tags to consumer version selectors (Matt Fellows, Sat Nov 7 23:35:36 2020 +1100)
* 6633575 - fix: ported matching logic fixes from Pact-JVM (Ronald Holshausen, Mon Nov 2 18:20:22 2020 +1100)
* 6cddd4c - bump version to 0.8.3 (Ronald Holshausen, Fri Oct 30 12:10:23 2020 +1100)

# 0.8.2 - Bugfix Release

* b4c4de8 - chore: upgrade to latest Onig crate (Ronald Holshausen, Wed Oct 28 09:59:36 2020 +1100)
* 3acf437 - fix: when merging pacts, it helps to use the new interations in the merged pact, not the old ones #77 (Ronald Holshausen, Sat Oct 17 18:17:57 2020 +1100)
* 2d945bf - bump version to 0.8.2 (Ronald Holshausen, Fri Oct 16 16:11:32 2020 +1100)

# 0.8.1 - Bugfix Release

* d24cfe3 - fix: matching binary data was broken after refactor (Ronald Holshausen, Fri Oct 16 16:05:26 2020 +1100)
* aa287ed - bump version to 0.8.1 (Ronald Holshausen, Fri Oct 16 11:08:30 2020 +1100)

# 0.8.0 - V4 models + arrayContains matcher

* b668d81 - chore: cleanup warnings and add missing doc comments (Ronald Holshausen, Thu Oct 15 16:57:57 2020 +1100)
* c686ce0 - fix: arrayContains matcher JSON was missing match attribute (Ronald Holshausen, Thu Oct 15 15:55:50 2020 +1100)
* f090323 - feat: updated integration JSON to handle array contains matcher (Ronald Holshausen, Thu Oct 15 15:31:47 2020 +1100)
* 7110ab1 - feat: array contains working with Siren example (Ronald Holshausen, Thu Oct 15 11:47:01 2020 +1100)
* d79beb4 - feat: basic array contains matcher working (Ronald Holshausen, Wed Oct 14 17:04:08 2020 +1100)
* 03f43d4 - feat: initail implementation of array contains matcher (Ronald Holshausen, Wed Oct 14 14:43:05 2020 +1100)
* cbc7812 - fix: clippy erros (Ronald Holshausen, Wed Oct 14 11:39:37 2020 +1100)
* a16250a - chore: update spec test cases (Ronald Holshausen, Wed Oct 14 11:25:36 2020 +1100)
* 831ba3d - fix: implement display for Interaction and Message (Ronald Holshausen, Wed Oct 14 10:09:32 2020 +1100)
* 013fbaf - feat: implemented writing pact for V4 pacts (Ronald Holshausen, Tue Oct 13 18:56:03 2020 +1100)
* d4ff696 - refactor: store the content type class with the body, not the string value (Ronald Holshausen, Tue Oct 13 17:23:25 2020 +1100)
* fa62520 - refactor: V4 message spec test cases passing (Ronald Holshausen, Tue Oct 13 16:05:57 2020 +1100)
* 9d0f05c - refactor: V4 request and response spec tests passing (Ronald Holshausen, Tue Oct 13 14:41:42 2020 +1100)
* a151bcc - fix: Charsets in headers should be compared ignoring case (Ronald Holshausen, Tue Oct 13 14:12:15 2020 +1100)
* 50aa09b - refactor: got spec tests passing; added V4 spec tests (Ronald Holshausen, Tue Oct 13 13:38:05 2020 +1100)
* 6e3ffd7 - refactor: fix remaining failing tests after refactor (Ronald Holshausen, Tue Oct 13 11:13:47 2020 +1100)
* 41126a7 - refactor: got all unit tests passing after big refactor (Ronald Holshausen, Mon Oct 12 17:32:58 2020 +1100)
* f334a4f - refactor: introduce a MatchingContext into all matching functions + delgate to matchers for collections (Ronald Holshausen, Mon Oct 12 14:06:00 2020 +1100)
* 7fbc731 - chore: bump minor version of matching lib (Ronald Holshausen, Fri Oct 9 10:42:33 2020 +1100)
* 32f112b - refactor: change matching methods to return a result (Ronald Holshausen, Fri Oct 9 10:39:01 2020 +1100)
* dd2ffa7 - feat: support text/xml as an XML content type (Ronald Holshausen, Thu Oct 8 15:49:23 2020 +1100)
* 356de26 - chore: cleanup some deprecation warnings (Ronald Holshausen, Thu Oct 8 15:41:33 2020 +1100)
* 2fdba73 - chore: cleanup some deprecation warnings (Ronald Holshausen, Thu Oct 8 15:15:45 2020 +1100)
* 172f505 - chore: cleaned up some compiler warnings (Ronald Holshausen, Thu Oct 8 15:02:49 2020 +1100)
* 02fa83d - chore: implemented the remaining unimplemented integration methods (Ronald Holshausen, Wed Oct 7 16:36:59 2020 +1100)
* ce2ab5e - chore: fix clippy errors (Ronald Holshausen, Wed Oct 7 15:44:52 2020 +1100)
* d0d7380 - feat: enabled some more tests for V4 models (Ronald Holshausen, Wed Oct 7 14:38:07 2020 +1100)
* 5d8f744 - feat: loading V4 pact tests passing (Ronald Holshausen, Wed Oct 7 13:51:13 2020 +1100)
* 511272a - feat: got V4 Synchronous/HTTP pact loading (Ronald Holshausen, Wed Oct 7 12:56:48 2020 +1100)
* 7be8de6 - feat: Implemented V4 interactions + loading from JSON (Ronald Holshausen, Tue Oct 6 17:16:40 2020 +1100)
* b2725dd - feat: added V4 interaction types (Ronald Holshausen, Tue Oct 6 12:03:03 2020 +1100)
* 7232e89 - feat: Add initial V4 models and example pact files (Ronald Holshausen, Tue Oct 6 09:13:21 2020 +1100)
* cbb6e20 - fix: generators to_json was only writing the first one for bodies, headers and queries (Ronald Holshausen, Sun Oct 4 12:52:24 2020 +1100)
* 3131237 - bump version to 0.7.2 (Ronald Holshausen, Mon Sep 28 11:54:03 2020 +1000)

# 0.7.1 - CORS pre-flight + fixes

* 7fd4dd2 - refactor: update the mock server CLI to use webmachine 0.2 and hyper 0.13 (Ronald Holshausen, Sun Sep 27 09:39:23 2020 +1000)
* 2e662a6 - feat: handle CORS pre-flight requests in the mock server (Ronald Holshausen, Wed Sep 23 17:59:32 2020 +1000)
* af4f106 - chore: cleanup some clippy warnings (Ronald Holshausen, Sun Sep 20 15:34:48 2020 +1000)
* d8ceb74 - fix: don't clone a double reference (clippy error) (Ronald Holshausen, Sun Sep 20 15:12:11 2020 +1000)
* 9c9b172 - chore: handle edge cases in random_decimal_generator (Ronald Holshausen, Sun Sep 20 15:01:06 2020 +1000)
* 042bed0 - fix: random decimal generator now includes a decimal point in the generated values (Ronald Holshausen, Sun Sep 20 11:18:28 2020 +1000)
* cd9d41c - fix: strip off anchors before generating a value from a regex (Ronald Holshausen, Fri Sep 18 15:38:38 2020 +1000)
* 9389c0a - fix: don't unwrap a result when generating random string from regex (Ronald Holshausen, Fri Sep 18 15:24:42 2020 +1000)
* a5f17a5 - fix: UUID generator should return hyphenated values (Ronald Holshausen, Thu Sep 17 10:06:52 2020 +1000)
* 91cc833 - bump version to 0.7.1 (Ronald Holshausen, Mon Sep 14 16:43:41 2020 +1000)

# 0.7.0 - Message pacts and matching messages

* 865327d - feat: handle comparing content types correctly (Ronald Holshausen, Mon Sep 14 16:37:11 2020 +1000)
* ebee1c0 - feat: implemented matching for message metadata (Ronald Holshausen, Mon Sep 14 15:31:18 2020 +1000)
* 6cba6ad - feat: implemented basic message verification with the verifier cli (Ronald Holshausen, Mon Sep 14 13:48:27 2020 +1000)
* 2d44ffd - chore: bump minor version of the matching crate (Ronald Holshausen, Mon Sep 14 12:06:37 2020 +1000)
* fb6c19c - refactor: allow verifier to handle different types of interactions (Ronald Holshausen, Mon Sep 14 10:41:13 2020 +1000)
* 814c416 - refactor: added a trait for interactions, renamed Interaction to RequestResponseInteraction (Ronald Holshausen, Sun Sep 13 17:09:41 2020 +1000)
* 08dfa39 - chore: cleanup some deprecation warnings (Ronald Holshausen, Sun Sep 13 13:10:23 2020 +1000)
* a05bcbb - refactor: renamed Pact to RequestResponsePact (Ronald Holshausen, Sun Sep 13 12:45:34 2020 +1000)
* cc42fbc - feat: add MessagePact (Pact with Messages instead of Interactions) (Caleb Stepanian, Sun Aug 16 15:25:01 2020 -0400)
* ec3193e - bump version to 0.6.6 (Ronald Holshausen, Sun Aug 23 14:07:51 2020 +1000)

# 0.6.5 - implemented provider state generator

* f2532ee - chore: remove incorrect imports (Ronald Holshausen, Sun Aug 23 14:02:07 2020 +1000)
* b130cd2 - feat: add tests for serialising Generator::ProviderStateGenerator (Ronald Holshausen, Sun Aug 23 13:53:41 2020 +1000)
* 76f73c6 - feat: implemented provider state generator (Ronald Holshausen, Sun Aug 23 13:29:55 2020 +1000)
* 30d5a75 - bump version to 0.6.5 (Ronald Holshausen, Sun Jul 26 12:05:16 2020 +1000)

# 0.6.4 - Refactor to return the most relevant response from the mock server

* da53bac - fix: return the most relevant response from the mock server #69 (Ronald Holshausen, Tue Jul 21 16:10:54 2020 +1000)
* b242eb1 - refactor: changed the remaining uses of the old content type methods (Ronald Holshausen, Sun Jun 28 17:11:51 2020 +1000)
* f531966 - refactor: update body matchers to use content type struct (Ronald Holshausen, Sun Jun 28 16:50:52 2020 +1000)
* c8913e8 - refactor: convert generators to use the content type struct (Ronald Holshausen, Sun Jun 28 13:55:43 2020 +1000)
* 5316030 - feat: added a struct for handling content types (Ronald Holshausen, Sun Jun 28 13:31:22 2020 +1000)
* dc4a1ef - chore: fix link in readme (Ronald Holshausen, Sun Jun 28 10:08:14 2020 +1000)
* 359a944 - chore: update versions in readmes (Ronald Holshausen, Sat Jun 27 13:21:24 2020 +1000)
* 876c60d - bump version to 0.6.4 (Ronald Holshausen, Wed Jun 24 10:30:33 2020 +1000)

# 0.6.3 - Updated XML Matching

* a15edea - chore: try set the content type on the body if known (Ronald Holshausen, Tue Jun 23 16:53:32 2020 +1000)
* daeaa0c - feat: update the spec test cases after implementing XML matching MkII (Ronald Holshausen, Tue Jun 23 16:09:23 2020 +1000)
* 90c175c - feat: re-implement XML matching to support elements with different children (Ronald Holshausen, Tue Jun 23 15:20:36 2020 +1000)
* 4d18e1b - bump version to 0.6.3 (Ronald Holshausen, Fri Jun 12 12:00:38 2020 +1000)
* f2c7145 - fix: correct build dependencies (Ronald Holshausen, Fri Jun 12 11:57:48 2020 +1000)

# 0.6.2 - Overhaul date/time matching

* 45fc1a0 - fix: cleanup warnings and fixed test (Ronald Holshausen, Fri Jun 12 10:51:44 2020 +1000)
* a6cbe4b - feat: support validating datetimes with timezones (Ronald Holshausen, Wed Jun 10 17:03:56 2020 +1000)
* 875d7a1 - refactor: changed date/time parsing to support Java DateTimeFormatter format (Ronald Holshausen, Tue Jun 9 17:56:30 2020 +1000)
* c1b657b - feat: make default metadata public so other language impl can access it (Ronald Holshausen, Thu Jun 4 16:02:16 2020 +1000)
* e699061 - feat: add convience methods to modify headers (Ronald Holshausen, Thu Jun 4 16:01:04 2020 +1000)
* 0d11998 - chore: switch to Rust TLS so we dont have to link to openssl libs (Ronald Holshausen, Sun May 31 09:49:55 2020 +1000)
* f94f25a - fix: intermediate date/time matcher JSON should use the format attribute (Ronald Holshausen, Wed May 27 14:19:34 2020 +1000)
* ae0af17 - bump version to 0.6.2 (Ronald Holshausen, Wed May 27 10:35:40 2020 +1000)

# 0.6.1 - Bugfix Release

* 6c65dab - feat: handle namespaces when matching XML (Ronald Holshausen, Mon May 25 16:23:20 2020 +1000)
* 67e2147 - fix: was incorrectly selecting the matching rule when weight was equal (Ronald Holshausen, Mon May 25 16:22:36 2020 +1000)
* 1e3516b - bump version to 0.6.1 (Ronald Holshausen, Sun May 24 11:49:20 2020 +1000)

# 0.6.0 - multi-part form post bodies

* ce94df9 - feat: cleaned up the logging of request matches (Ronald Holshausen, Sun May 24 11:17:08 2020 +1000)
* bea787c - chore: bump matching crate version to 0.6.0 (Ronald Holshausen, Sat May 23 17:56:04 2020 +1000)
* d0a54f7 - feat: implemented matching multi-part form post bodies (Ronald Holshausen, Sat May 23 17:49:48 2020 +1000)
* ac2903d - chore: update the specification test cases (Ronald Holshausen, Wed May 20 12:08:08 2020 +1000)
* b0f3387 - bump version to 0.5.15 (Ronald Holshausen, Fri May 15 16:27:48 2020 +1000)

# 0.5.14 - Bugfix Release

* 61ab50f - fix: date/time matchers fallback to the old key (Ronald Holshausen, Fri May 15 11:27:27 2020 +1000)
* ddacb5d - fix: FFI datetime matcher was using incorrect field (Ronald Holshausen, Wed May 13 17:58:31 2020 +1000)
* 6af29ce - fix: improve the error message when a merge conflict occurs (Ronald Holshausen, Wed May 13 10:57:25 2020 +1000)
* ddd0881 - bump version to 0.5.14 (Ronald Holshausen, Tue May 12 12:33:30 2020 +1000)

# 0.5.13 - matching of binary payloads

* 708db47 - feat: implement matching of binary payloads (application/octet-stream) (Ronald Holshausen, Fri May 8 15:52:03 2020 +1000)
* 754a483 - chore: updated itertools to latest (Ronald Holshausen, Wed May 6 15:49:27 2020 +1000)
* b6b81a3 - bump version to 0.5.13 (Ronald Holshausen, Tue May 5 16:39:23 2020 +1000)

# 0.5.12 - Bugfix Release

* d85f28c - fix: mock server matching requests with headers with multiple values (Ronald Holshausen, Tue May 5 15:23:11 2020 +1000)
* a45d0c3 - fix: FFI mismatch json should have the actual values as UTF-8 string not bytes #64 (Ronald Holshausen, Thu Apr 30 11:16:25 2020 +1000)
* 2003d7b - chore: roll back onig crate to 4.3.3 #64 (Ronald Holshausen, Thu Apr 30 09:50:48 2020 +1000)
* 76250b5 - chore: correct some clippy warnings (Ronald Holshausen, Wed Apr 29 17:53:40 2020 +1000)
* 47cc589 - chore: added clippy and fixed resulting lint errors (Ronald Holshausen, Wed Apr 29 15:32:55 2020 +1000)
* 6f24994 - bump version to 0.5.12 (Ronald Holshausen, Fri Apr 24 10:11:33 2020 +1000)

# 0.5.11 - Cleaned up logging and warnings

* 3d490ef - chore: implemented Display for Interaction (Ronald Holshausen, Wed Apr 22 13:01:45 2020 +1000)
* af8d19a - chore: cleanup warning (Ronald Holshausen, Thu Apr 16 14:37:55 2020 +1000)
* 9ff6f20 - chore: cleaned up some debug logging (Ronald Holshausen, Tue Apr 7 12:10:12 2020 +1000)
* 1ad8edd - bump version to 0.5.11 (Ronald Holshausen, Tue Apr 7 11:42:14 2020 +1000)

# 0.5.10 - Bugfix Release

* b52f095 - fix: V3 path matcher JSON format was incorrect (Ronald Holshausen, Tue Apr 7 11:14:25 2020 +1000)
* 9623183 - chore: upgraded the testing crates to latest (Ronald Holshausen, Tue Apr 7 09:40:39 2020 +1000)
* a9d512f - bump version to 0.5.10 (Ronald Holshausen, Fri Mar 13 09:39:43 2020 +1100)

# 0.5.9 - Bugfixes + Date/Time matchers with JSON

* e0f23a2 - feat: exposes time/date utils for language implementations (Ronald Holshausen, Thu Mar 12 17:01:44 2020 +1100)
* 2920364 - fix: date and time matchers with JSON (Ronald Holshausen, Thu Mar 12 16:07:05 2020 +1100)
* db74b68 - Merge pull request #61 from mitre/v3_provider_states (Ronald Holshausen, Mon Mar 9 13:37:03 2020 +1100)
* 70e6648 - chore: converted verifier to use Reqwest (Ronald Holshausen, Mon Mar 9 12:20:14 2020 +1100)
* 627c4ad - At least partially correct broken Serialize/Deserialize for Message. (Andrew Lilley Brinker, Tue Mar 3 08:06:52 2020 -0800)
* 162f52d - Fixed three broken tests. (Andrew Lilley Brinker, Tue Mar 3 07:15:44 2020 -0800)
* d87a2c3 - Made `Message` understand `providerStates`. (Andrew Lilley Brinker, Mon Mar 2 08:38:56 2020 -0800)
* d594dbb - Fix broken documentation link for provider_states. (Andrew Lilley Brinker, Mon Mar 2 08:21:40 2020 -0800)
* 6187cfa - bump version to 0.5.9 (Ronald Holshausen, Sun Jan 19 11:11:09 2020 +1100)

# 0.5.8 - Upgrade reqwest to 0.10

* 9dec41b - Upgrade reqwest to 0.10 (Audun Halland, Tue Dec 31 07:22:36 2019 +0100)
* fda11e4 - Merge remote-tracking branch 'upstream/master' into async-await (Audun Halland, Tue Dec 17 02:13:58 2019 +0100)
* d395d2d - pact_verifier: Upgrade reqwest to latest git alpha (Audun Halland, Tue Dec 17 00:57:16 2019 +0100)
* 298f217 - pact_matching: Upgrade reqwest to current alpha (Audun Halland, Tue Dec 17 00:36:33 2019 +0100)
* d28d97d - bump version to 0.5.8 (Ronald Holshausen, Sat Dec 14 16:57:02 2019 +1100)

# 0.5.7 - Bugfix Release

* a660b87 - fix: correct pact merging to remove duplicates #54 (Ronald Holshausen, Sat Dec 14 15:06:30 2019 +1100)
* 51f5a3e - Update READMEs and doc to not require any "extern crate" (Audun Halland, Sun Nov 17 23:28:21 2019 +0100)
* bc1515a - pact_matching: Upgrade lazy_static to get rid of warning msg (Audun Halland, Sun Nov 17 22:47:30 2019 +0100)
* e0bb698 - pact_matching: Remove test extern crate from lib.rs (Audun Halland, Sun Nov 17 22:43:45 2019 +0100)
* c16574d - pact_matching: Remove prod extern crate from lib.rs (Audun Halland, Sun Nov 17 22:32:35 2019 +0100)
* 85efd07 - pact_matching: use maplit::* explicitly (Audun Halland, Sun Nov 17 22:17:53 2019 +0100)
* 382f304 - pact_matching: Upgrade log to 0.4 - for scoped macro (Audun Halland, Sun Nov 17 22:12:55 2019 +0100)
* fcadd7f - pact_matching: Remove extern crate serde_json (Audun Halland, Sun Nov 17 21:57:39 2019 +0100)
* 713cd6a - Explicit edition 2018 in Cargo.toml files (Audun Halland, Sat Nov 16 23:55:37 2019 +0100)
* 924452f - 2018 edition autofix "cargo fix --edition" (Audun Halland, Sat Nov 16 22:27:42 2019 +0100)
* 8523e69 - bump version to 0.5.7 (Ronald Holshausen, Sun Nov 10 10:30:20 2019 +1100)

# 0.5.6 - Bugfix Release

* a0dc946 - fix: store matching rules in a set to avoid duplicates (Ronald Holshausen, Sun Nov 10 10:08:34 2019 +1100)
* 66c328e - feat: add colons to the allowed path characters (Ronald Holshausen, Sun Oct 27 17:13:14 2019 +1100)
* 869af94 - bump version to 0.5.6 (Ronald Holshausen, Fri Sep 27 14:57:05 2019 +1000)

# 0.5.5 - Oniguruma crate for regex matching

* defe890 - fix: switch to the Oniguruma crate for regex matching #46 (Ronald Holshausen, Fri Sep 27 14:35:16 2019 +1000)
* d5c0ac8 - chore: re-enabled time and timestamp matching tests (Ronald Holshausen, Fri Sep 27 12:49:32 2019 +1000)
* 19bf916 - bump version to 0.5.5 (Ronald Holshausen, Sun Sep 22 17:11:00 2019 +1000)

# 0.5.4 - Refactor for publishing verification results

* eef3d97 - feat: added some tests for publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 16:44:52 2019 +1000)
* 1110b47 - feat: implemented publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 13:53:27 2019 +1000)
* cb30a2f - feat: added the ProviderStateGenerator as a generator type (Ronald Holshausen, Sun Sep 8 16:29:46 2019 +1000)
* 8932ef6 - feat: support an integration format for matchers for language integration (Ronald Holshausen, Sun Aug 25 11:36:23 2019 +1000)
* 6899663 - bump version to 0.5.4 (Ronald Holshausen, Sun Aug 11 14:41:09 2019 +1000)

# 0.5.3 - support bearer tokens

* 152682e - chore: cleanup crates and warnings (Ronald Holshausen, Sun Aug 11 14:28:02 2019 +1000)
* dac8ae1 - feat: support authentication when fetching pacts from a pact broker (Ronald Holshausen, Sun Aug 11 13:57:29 2019 +1000)
* e007763 - feat: support bearer tokens when fetching pacts from URLs (Ronald Holshausen, Sun Aug 11 13:21:17 2019 +1000)
* 8009184 - bump version to 0.5.3 (Ronald Holshausen, Sun Aug 11 09:46:21 2019 +1000)

# 0.5.2 - Support headers with multiple values

* 0c5f718 - feat: support matchers on plain text bodies #43 (Ronald Holshausen, Sat Aug 10 17:54:26 2019 +1000)
* f0c0d07 - feat: support headers with multiple values (Ronald Holshausen, Sat Aug 10 17:01:10 2019 +1000)
* 699f48f - bump version to 0.5.2 (Ronald Holshausen, Sat Jun 29 19:34:44 2019 +1000)
* 0fe57d9 - fix: release script (Ronald Holshausen, Sat Jun 29 19:28:46 2019 +1000)
* 756ac9d - chore: update release script (Ronald Holshausen, Sat Jun 29 19:27:45 2019 +1000)

# 0.5.1 - Bugfix Release

* eab2d86 - chore: removed P macro (Ronald Holshausen, Sat Jun 29 18:45:12 2019 +1000)
* 91da912 - fix: correct overflow of max value for random int generator #39 (Ronald Holshausen, Sat Jun 29 18:43:56 2019 +1000)
* 4ccd09d - bump version to 0.5.1 (Ronald Holshausen, Sat Jan 5 19:42:58 2019 +1100)

# 0.5.0 - Regex, Date and Time matching and generators

* f8fa0d8 - chore: Bump pact matchig version to 0.5.0 (Ronald Holshausen, Sat Jan 5 19:25:53 2019 +1100)
* 4f471de - feat: implemented generating values from regex (Ronald Holshausen, Sat Jan 5 18:46:48 2019 +1100)
* 73bc70e - feat: implemented generators for dates and times #33 (Ronald Holshausen, Sat Jan 5 17:10:56 2019 +1100)
* e72fb9e - feat: cleanup date matching #33 (Ronald Holshausen, Sat Jan 5 14:31:50 2019 +1100)
* 8b9b043 - feat: implemeted general timezone patterns in date matching #33 (Ronald Holshausen, Sat Jan 5 14:23:21 2019 +1100)
* 45e1ee1 - feat: implemeted RFC 822 and ISO 8601 timezones in date matching #33 (Ronald Holshausen, Fri Jan 4 15:19:09 2019 +1100)
* 2978a00 - feat: implemeted time in date matching #33 (Ronald Holshausen, Fri Jan 4 14:08:06 2019 +1100)
* 5d890a5 - feat: implemeted day of week in date matching #33 (Ronald Holshausen, Fri Jan 4 13:31:59 2019 +1100)
* 33f4054 - feat: implemeted simple date matching #33 (Ronald Holshausen, Fri Jan 4 11:16:16 2019 +1100)
* ce57f17 - feat: implemented formatted display for request and response (Ronald Holshausen, Tue Jan 1 11:52:58 2019 +1100)
* 433d9c5 - fix: handle path expressions that start with an underscore (Ronald Holshausen, Tue Jan 1 10:51:43 2019 +1100)
* 009b176 - bump version to 0.4.6 (Ronald Holshausen, Sat Sep 8 14:41:35 2018 +1000)

# 0.4.5 - feat: added convenience header methods to HttpPart

* ead1af2 - feat: added convenience header methods to HttpPart (Ronald Holshausen, Sat Sep 8 14:29:59 2018 +1000)
* 129333f - bump version to 0.4.5 (Ronald Holshausen, Sat Aug 11 15:21:01 2018 +1000)

# 0.4.4 - Bugfix Release

* 97abce4 - fix: support matching rules affected by Pact-JVM defect 743 (Ronald Holshausen, Sat Aug 11 15:07:41 2018 +1000)
* f9d091e - bump version to 0.4.4 (Ronald Holshausen, Sat Jun 30 17:14:16 2018 +1000)

# 0.4.3 - Bugfix Release

* 1184203 - fix: Allow dashes in path expressions for headers like Content-Type (Ronald Holshausen, Sat Jun 30 17:03:08 2018 +1000)
* 995139b - Revert "fix: query and header paths should be escaped" (Ronald Holshausen, Sat Jun 30 16:56:05 2018 +1000)
* 74e9116 - bump version to 0.4.3 (Ronald Holshausen, Sat Jun 30 16:40:17 2018 +1000)

# 0.4.2 - Bugfix Release

* d6fbed4 - fix: query and header paths should be escaped (Ronald Holshausen, Sat Jun 30 16:22:56 2018 +1000)
* 948e620 - fix: parse the V3 keys as path expressions for query and header matchers (Ronald Holshausen, Sat Jun 30 15:22:51 2018 +1000)
* dec17b8 - doc: update readme (Ronald Holshausen, Sun May 13 14:33:20 2018 +1000)
* c3898b9 - bump version to 0.4.2 (Ronald Holshausen, Sun May 13 14:24:54 2018 +1000)

# 0.4.1 - implemented some missing matchers (include, null, integer, decimal, number)

* b060bbb - feat: implemented some missing matchers (include, null, integer, decimal, number) (Ronald Holshausen, Sun May 13 13:46:23 2018 +1000)
* 0aa161d - test: Added a test to confirm that binary bodies are persisted in base64 format #19 (Ronald Holshausen, Sun Apr 8 14:27:19 2018 +1000)
* b68c893 - fix: pact specification key in the metadata should be camelcase #3 (Ronald Holshausen, Sun Apr 8 12:05:39 2018 +1000)
* 10eb623 - bump version to 0.4.1 (Ronald Holshausen, Sat Apr 7 14:08:52 2018 +1000)

# 0.4.0 - First V3 specification release

* e5322f1 - code cleanup in prep of release (Ronald Holshausen, Sat Apr 7 13:58:55 2018 +1000)
* d90af09 - Implemented decimal, hexadecimal and string generators (Ronald Holshausen, Fri Mar 9 16:48:46 2018 +1100)
* bc077ec - Completed the implementation of applying generators to JSON bodies (Ronald Holshausen, Fri Mar 9 15:44:11 2018 +1100)
* 5e824ba - Implemented applying a generator based on a key to a JSON document (Ronald Holshausen, Fri Mar 9 15:05:08 2018 +1100)
* 35eb4d1 - Simplify updating the json document by using the Serde pointer functions (Ronald Holshausen, Mon Mar 5 14:24:47 2018 +1100)
* d688d59 - Removed use of RefCell (Ronald Holshausen, Mon Mar 5 10:44:02 2018 +1100)
* 691b14c - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Mar 4 17:10:14 2018 +1100)
* 6597141 - WIP - start of implementation of applying generators to the bodies (Ronald Holshausen, Sun Mar 4 17:01:11 2018 +1100)
* a2d3a27 - fix json matcher so root value strings and key value pairs are both valid (Samuel McKendrick, Mon Feb 12 15:29:58 2018 +0100)
* a76bb5a - Cleaned up all the compiler warnings (Ronald Holshausen, Sun Nov 19 11:29:47 2017 +1100)
* efb17a1 - When there is no content type, default to text/plain (Ronald Holshausen, Sun Nov 19 10:58:17 2017 +1100)
* ec89fcd - Implemented generators being applied to query parameters (Ronald Holshausen, Tue Nov 7 17:23:21 2017 +1100)
* c4d424b - Wired in the generated request/response into the mock server and verifier (Ronald Holshausen, Tue Nov 7 16:27:01 2017 +1100)
* 308fe4d - Implemented writing the generators to the pact JSON (Ronald Holshausen, Tue Nov 7 16:09:51 2017 +1100)
* 051ecb7 - Implemented parsing the generators from the Pact JSON (Ronald Holshausen, Tue Nov 7 14:08:58 2017 +1100)
* 13558d6 - Basic generators working (Ronald Holshausen, Tue Nov 7 10:56:55 2017 +1100)
* 7fef36b - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sat Nov 4 12:49:07 2017 +1100)
* 7fb7a34 - bump version to 0.3.2 (Ronald Holshausen, Fri Nov 3 12:12:31 2017 +1100)
* a905bed - Cleaned up some compiler warnings (Ronald Holshausen, Sun Oct 22 12:26:09 2017 +1100)
* 940a0e3 - Reverted hyper to 0.9.x (Ronald Holshausen, Sun Oct 22 12:01:17 2017 +1100)
* fbe35d8 - Compiling after merge from v2-spec (Ronald Holshausen, Sun Oct 22 11:39:46 2017 +1100)
* 00dc75a - Bump version to 0.4.0 (Ronald Holshausen, Sun Oct 22 10:46:48 2017 +1100)
* 184127a - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sun Oct 22 10:32:31 2017 +1100)
* e82ee08 - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Mon Oct 16 09:24:11 2017 +1100)
* 64ff667 - Upgraded the mock server implemenation to use Hyper 0.11.2 (Ronald Holshausen, Wed Sep 6 12:56:47 2017 +1000)
* 1d7ed25 - Upgraded all crates to the latest versions (Ronald Holshausen, Sun Aug 20 16:21:15 2017 +1000)
* 8f72bd4 - Cleaned up all imports and documentation after merge from master (Ronald Holshausen, Sun Aug 20 11:52:39 2017 +1000)
* ab667ca - pact_matching build passing after merge from master (Ronald Holshausen, Sun Aug 20 11:36:22 2017 +1000)
* 362753a - pact_matching compiling after merge from master (Ronald Holshausen, Sun Aug 20 10:55:29 2017 +1000)
* e5a93f3 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Aug 20 09:53:48 2017 +1000)
* b56d11c - add enum values for V4 spec (Ronald Holshausen, Sun Aug 20 09:15:07 2017 +1000)
* b4e0e2a - re-enabled all the specification test cases (Ronald Holshausen, Sun Nov 13 16:23:45 2016 +1100)
* 9541a96 - Implemented matching with V3 format matchers (Ronald Holshausen, Sun Nov 13 16:01:33 2016 +1100)
* 2ed6b00 - Implemented serialisation of v3 format matchers to json (Ronald Holshausen, Sat Nov 12 19:37:25 2016 +1100)
* 6f322c1 - Implemented matching rule lookup for a rule category (Ronald Holshausen, Sat Nov 12 16:10:50 2016 +1100)
* 278978b - Load the old V2 format matchers into matching rules structure (Ronald Holshausen, Mon Oct 31 17:23:56 2016 +1100)
* c7119c0 - replaced matcher collection with a matching rules struct and implemented loading V3 format rules (Ronald Holshausen, Mon Oct 31 16:41:03 2016 +1100)
* 8797c6c - First successful build after merge from master (Ronald Holshausen, Sun Oct 23 11:59:55 2016 +1100)
* 639ac22 - fixes after merge in from master (Ronald Holshausen, Sun Oct 23 10:45:54 2016 +1100)
* 7361688 - moved missing files after merge from master (Ronald Holshausen, Sun Oct 23 10:19:31 2016 +1100)
* 49e45f7 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Oct 23 10:10:40 2016 +1100)

# 0.3.1 - Converted OptionalBody::Present to take a Vec<u8>

* 24e3f73 - Converted OptionalBody::Present to take a Vec<u8> #19 (Ronald Holshausen, Sun Oct 22 18:04:46 2017 +1100)
* cd564ac - Added cargo update after to release script after bumping the version (Ronald Holshausen, Fri Oct 20 09:34:57 2017 +1100)
* c345796 - Fix the release script as docs are no longer generated by build (Ronald Holshausen, Fri Oct 20 09:22:19 2017 +1100)
* c507222 - Correct the version in the readme (Ronald Holshausen, Fri Oct 20 09:21:55 2017 +1100)
* aa5cc66 - bump version to 0.3.1 (Ronald Holshausen, Fri Oct 20 09:11:35 2017 +1100)

# 0.3.0 - Backported matching rules from V3 branch

* ac94388 - Tests are now all passing #20 (Ronald Holshausen, Thu Oct 19 15:14:25 2017 +1100)
* d990729 - Some code cleanup #20 (Ronald Holshausen, Wed Oct 18 16:32:37 2017 +1100)
* db6100e - Updated the consumer DSL to use the matching rules (compiling, but tests are failing) #20 (Ronald Holshausen, Wed Oct 18 15:48:23 2017 +1100)
* 161d69d - Added a test to confirm the min/max matchers are writing their values in the correct form #20 (Ronald Holshausen, Wed Oct 18 14:19:34 2017 +1100)
* c983c63 - Bump versions to 0.3.0 (Ronald Holshausen, Wed Oct 18 13:54:46 2017 +1100)
* 941d0de - Backported the matching rules from the V3 branch #20 (Ronald Holshausen, Mon Oct 31 16:41:03 2016 +1100)
* 01f09be - [BUG] pact_matching: Parse JSON paths with `_` (Eric Kidd, Tue Oct 10 08:55:44 2017 -0400)
* d6f867b - Replace `Term` with open-ended `Matchable` trait (Eric Kidd, Fri Oct 6 06:56:02 2017 -0400)
* 3f42e50 - Implement `JsonPattern` w/o matcher support (Eric Kidd, Wed Oct 4 13:40:09 2017 -0400)
* 06e92e5 - Refer to local libs using version+paths (Eric Kidd, Tue Oct 3 06:22:23 2017 -0400)
* 691c9e6 - Fetch test JSON paths in a more reliable fashion (Eric Kidd, Mon Oct 2 07:20:48 2017 -0400)
* 7afd258 - Update all the cargo manifest versions and commit the cargo lock files (Ronald Holshausen, Wed May 17 10:37:44 2017 +1000)
* bb278d3 - bump version to 0.2.3 (Anthony Damtsis, Tue May 16 17:09:58 2017 +1000)

# 0.2.2 - Bugfix Release

* 3399f7c - Merge pull request #13 from adamtsis/remove-deprecated-json-lib (Ronald Holshausen, Tue May 16 15:56:22 2017 +1000)
* efe4ca7 - Cleanup unused imports and unreachable pattern warning messages (Anthony Damtsis, Tue May 16 10:31:29 2017 +1000)
* a59fb98 - Migrate remaining pact modules over to serde (Anthony Damtsis, Mon May 15 16:59:04 2017 +1000)
* 142d550 - Merge pull request #12 from adamtsis/remove-deprecated-json-lib (Ronald Holshausen, Mon May 8 16:13:30 2017 +1000)
* ff1b676 - Change spec test generator template to use serde library (Anthony Damtsis, Mon May 8 12:23:28 2017 +1000)
* cdecc71 - Simplify json handling logic when running comparisons (Anthony Damtsis, Fri May 5 15:48:17 2017 +1000)
* f725ddc - Remove commented crate (Anthony Damtsis, Fri May 5 15:39:27 2017 +1000)
* bd6fa9b - Fixed remaining serialisation bugs with writing pact files (Anthony Damtsis, Fri May 5 15:27:59 2017 +1000)
* d1bd5ef - Changed type parameter to be a Hashmap when deserialising payload body (Anthony Damtsis, Fri May 5 14:09:54 2017 +1000)
* 83a8b7e - Fix incorrectly deserialised json objects in tests (Anthony Damtsis, Fri May 5 13:23:03 2017 +1000)
* 21cb633 - Compiles now - lots of test failures to work through (Anthony Damtsis, Fri May 5 13:27:36 2017 +1000)
* 1e8531b - Begun work to replace rustc_serialize - work in progress (Anthony Damtsis, Mon May 1 14:52:08 2017 +1000)
* 7982137 - Merge pull request #11 from adamtsis/camel-case-specification (Ronald Holshausen, Mon May 1 13:49:09 2017 +1000)
* 9a29085 - Supports camel case format pact specifications (Anthony Damtsis, Thu Apr 27 15:03:15 2017 +1000)
* 4dabb31 - Simplify call to HeaderMismatch destructure (Anthony Damtsis, Wed Apr 26 20:48:32 2017 +1000)
* 381a85e - Explicitly clone the borrowed header reference (Anthony Damtsis, Wed Apr 26 18:25:04 2017 +1000)
* 755ada8 - Fixed compiler warning messages (Anthony Damtsis, Wed Apr 26 18:12:55 2017 +1000)
* a2847c6 - Replace .to_string() refs with s!() macro (Anthony Damtsis, Wed Apr 26 17:28:19 2017 +1000)
* c9eff21 - Support optional header parameters for 'accept' and 'content-type' (Anthony Damtsis, Wed Apr 26 15:36:08 2017 +1000)
* 26f91a5 - Ensure mismatch for HeaderMismatch is consistent for EqualityMatcher (Anthony Damtsis, Wed Apr 26 15:32:30 2017 +1000)
* a501309 - bump version to 0.2.2 (Ronald Holshausen, Sun Oct 9 16:14:45 2016 +1100)
* 227b61b - correct the doc URL in the cargo manifest (Ronald Holshausen, Sun Oct 9 16:13:27 2016 +1100)
* 5233cfa - correct updating the documentation URL in the release script (Ronald Holshausen, Sun Oct 9 16:08:33 2016 +1100)

# 0.2.1 - Changes required for verifying V2 pacts

* 574e072 - upadte versions for V2 branch and fix an issue with loading JSON bodies encoded as a string (Ronald Holshausen, Sun Oct 9 15:31:57 2016 +1100)
* a21973a - Get the build passing after merge from V1.1 branch (Ronald Holshausen, Sun Oct 9 13:47:09 2016 +1100)
* 341607c - Merge branch 'v1.1-spec' into v2-spec (Ronald Holshausen, Sun Oct 9 12:10:12 2016 +1100)
* 797c9b9 - correct the URLs to the repos (Ronald Holshausen, Sat Oct 8 17:10:56 2016 +1100)
* b7e038e - bump version to 0.1.2 (Ronald Holshausen, Sat Oct 8 16:54:52 2016 +1100)

# 0.1.1 - Changes required for verifying V1.1 pacts

* 373f82d - regenerated the specification tests (Ronald Holshausen, Sat Oct 8 16:50:38 2016 +1100)
* 388a19f - update references (Ronald Holshausen, Sat Oct 8 16:46:11 2016 +1100)
* a46dabb - update all references to V1 spec after merge (Ronald Holshausen, Sat Oct 8 16:20:51 2016 +1100)
* 63ae7e4 - get project compiling after merge from V1 branch (Ronald Holshausen, Sat Oct 8 15:53:22 2016 +1100)
* 1d6d4f8 - Merge branch 'v1-spec' into v1.1-spec (Ronald Holshausen, Sat Oct 8 15:44:25 2016 +1100)
* 04d9e5f - update the docs for the pact consumer library (Ronald Holshausen, Mon Sep 26 23:06:19 2016 +1000)
* 7dd04e6 - update the release scripts to point the docs to docs.rs (Ronald Holshausen, Mon Sep 26 21:49:35 2016 +1000)
* d7c859c - bump version to 0.0.3 (Ronald Holshausen, Mon Sep 26 20:55:12 2016 +1000)
* 02421d5 - exclude IntelliJ files from packaging (Ronald Holshausen, Mon Sep 26 20:46:47 2016 +1000)

# 0.1.0 - V1.1 Specification Implementation

* 140526d - Implement V1.1 matching (Ronald Holshausen, Tue Jun 28 15:58:35 2016 +1000)
* 4224875 - update readmes and bump versions for V1.1 implementation (Ronald Holshausen, Tue Jun 28 15:05:39 2016 +1000)
* b5dc6d2 - added some additional pact loading tests (Ronald Holshausen, Tue Jun 28 14:35:48 2016 +1000)
* 44ec659 - in prep for supporting other spec versions, take the version into account when parsing a pact file (Ronald Holshausen, Tue Jun 28 11:40:07 2016 +1000)
* 91d6d62 - removed the v1 from the project path, will use a git branch instead (Ronald Holshausen, Mon Jun 27 22:09:32 2016 +1000)

# 0.0.2 - Fixes required for verifying pacts

* 429ef78 - Implemented handling state change requests in the pact verifier (Ronald Holshausen, Sun Sep 25 15:55:18 2016 +1000)
* cc1e359 - implemented rudimentary diff output on json bodies (Ronald Holshausen, Sun Sep 25 13:43:45 2016 +1000)
* cd367e6 - Use a regex to detect the content type to handle extended types (e.g application/hal+json) (Ronald Holshausen, Sat Sep 24 17:14:16 2016 +1000)
* 0d69675 - Implemented pact test where there are no pacts in the pact broker (Ronald Holshausen, Sun Sep 18 17:41:51 2016 +1000)
* bc3405c - implemented handling templated HAL URLs (Ronald Holshausen, Sun Sep 18 13:58:54 2016 +1000)
* c3a8a30 - renamed the pact_matching and pact_mock_server directories (Ronald Holshausen, Sun Sep 18 11:07:32 2016 +1000)

# 0.0.1 - Second Feature Release

* 25bf4d0 - added changelog (Ronald Holshausen, Sun Jun 26 15:20:23 2016 +1000)
* 4c60f07 - replace rustful with webmachine (Ronald Holshausen, Thu Jun 16 17:31:11 2016 +1000)
* 7dc4b52 - implemented merging of pact files when writing (Ronald Holshausen, Thu Jun 9 17:34:02 2016 +1000)
* 801f24c - update the github readmes to point to the published rust docs (Ronald Holshausen, Wed Jun 8 10:42:30 2016 +1000)
* ecc4018 - add example pact files for testing (Ronald Holshausen, Wed Jun 8 09:36:35 2016 +1000)
* bbf6fbb - make test not be dependent on the library version (Ronald Holshausen, Wed Jun 1 17:23:02 2016 +1000)
* 937360d - Oops, test generates a pact with the version in the metadata (Ronald Holshausen, Wed Jun 1 17:07:29 2016 +1000)
* e957983 - bump libpact_matching version (Ronald Holshausen, Wed Jun 1 17:00:41 2016 +1000)

# 0.0.0 - First Release
