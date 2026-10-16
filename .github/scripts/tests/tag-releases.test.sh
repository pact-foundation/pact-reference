#!/usr/bin/env bash
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script="$here/../tag-releases.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

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
echo PASS
