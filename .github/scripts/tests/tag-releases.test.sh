#!/usr/bin/env bash
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script="$here/../tag-releases.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# A local crates.io index; the test crates' names exist on the real one.
mkdir -p "$work/index/1"
export CRATES_INDEX_URL="file://$work/index"

git init -q --bare "$work/remote.git"
git init -q "$work/repo"
cd "$work/repo"
git config user.name test
git config user.email test@example.test
mkdir -p ws/a/src ws/b/src
printf '[workspace]\nmembers = ["a", "b"]\nresolver = "2"\n' > ws/Cargo.toml
for c in a b; do
    printf '[package]\nname = "%s"\nversion = "1.0.0"\nedition = "2021"\n' "$c" > "ws/$c/Cargo.toml"
    touch "ws/$c/src/lib.rs"
done
git add -A && git commit -qm init
git remote add origin "$work/remote.git"
git tag a-v1.0.0
git push -q origin HEAD:refs/heads/master a-v1.0.0

out=$("$script" ws/Cargo.toml origin)
[[ "$out" == "b-v1.0.0" ]] || { echo "FAIL first run: got <$out>"; exit 1; }
git -C "$work/remote.git" rev-parse -q --verify refs/tags/b-v1.0.0 >/dev/null || { echo "FAIL b-v1.0.0 not pushed"; exit 1; }
[[ $(git cat-file -t b-v1.0.0) == tag ]] || { echo "FAIL b-v1.0.0 not annotated"; exit 1; }

out=$("$script" ws/Cargo.toml origin)
[[ -z "$out" ]] || { echo "FAIL second run should tag nothing: got <$out>"; exit 1; }

# A run can start after later commits land (GitHub drops pending runs in
# a concurrency group). The tag still marks the commit that set the version.
sed -i.bak 's/^version = "1.0.0"/version = "1.1.0"/' ws/b/Cargo.toml && rm ws/b/Cargo.toml.bak
git commit -qam "chore(b): release v1.1.0"
release=$(git rev-parse HEAD)
echo later > later.txt && git add later.txt && git commit -qm "docs: later change"
out=$("$script" ws/Cargo.toml origin)
[[ "$out" == "b-v1.1.0" ]] || { echo "FAIL bump run: got <$out>"; exit 1; }
[[ $(git rev-parse 'b-v1.1.0^{commit}') == "$release" ]] || { echo "FAIL b-v1.1.0 is not on the release commit"; exit 1; }

# A release PR merged with a merge commit: the tag goes on the merge.
git switch -qc release/a
sed -i.bak 's/^version = "1.0.0"/version = "1.0.1"/' ws/a/Cargo.toml && rm ws/a/Cargo.toml.bak
git commit -qam "chore(a): release v1.0.1"
git switch -q -
echo more > more.txt && git add more.txt && git commit -qm "docs: concurrent change"
git merge -q --no-ff -m "Merge release/a" release/a
merge=$(git rev-parse HEAD)
out=$("$script" ws/Cargo.toml origin)
[[ "$out" == "a-v1.0.1" ]] || { echo "FAIL merge run: got <$out>"; exit 1; }
[[ $(git rev-parse 'a-v1.0.1^{commit}') == "$merge" ]] || { echo "FAIL a-v1.0.1 is not on the merge commit"; exit 1; }

# A version already on crates.io without a tag (a migration tag that was
# never pushed) is not tagged: the tag would land on the wrong commit and
# start a release of that version.
echo '{"name":"b","vers":"1.2.0"}' > "$work/index/1/b"
sed -i.bak 's/^version = "1.1.0"/version = "1.2.0"/' ws/b/Cargo.toml && rm ws/b/Cargo.toml.bak
git commit -qam "chore(b): version already published"
out=$("$script" ws/Cargo.toml origin 2>&1)
git rev-parse -q --verify refs/tags/b-v1.2.0 >/dev/null && { echo "FAIL b-v1.2.0 was tagged"; exit 1; }
grep -q '::warning::.*b 1.2.0' <<<"$out" || { echo "FAIL no warning: <$out>"; exit 1; }
echo PASS
