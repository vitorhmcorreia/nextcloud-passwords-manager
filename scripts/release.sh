#!/usr/bin/env bash
# Bump the app version everywhere, commit, tag and push.
# Pushing the tag starts the GitHub Actions build (.github/workflows/release.yml).
#
# Usage: scripts/release.sh 0.2.0
set -euo pipefail
cd "$(dirname "$0")/.."

version="${1:-}"
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Usage: $0 X.Y.Z   (current: $(node -p 'require("./package.json").version'))" >&2
  exit 1
fi
if [[ -n "$(git status --porcelain)" ]]; then
  echo "Commit or stash your changes first." >&2
  exit 1
fi
if git rev-parse "v$version" >/dev/null 2>&1; then
  echo "Tag v$version already exists." >&2
  exit 1
fi

# The version lives in three files; keep them identical.
node -e '
  const fs = require("fs"), v = process.argv[1];
  for (const f of ["package.json", "src-tauri/tauri.conf.json"]) {
    const j = JSON.parse(fs.readFileSync(f, "utf8"));
    j.version = v;
    fs.writeFileSync(f, JSON.stringify(j, null, 2) + "\n");
  }
' "$version"
sed -i.bak -E "0,/^version = \".*\"/s//version = \"$version\"/" src-tauri/Cargo.toml && rm src-tauri/Cargo.toml.bak
(cd src-tauri && cargo update -p ncpass --quiet) # refresh Cargo.lock

git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git diff --cached --quiet || git commit -m "Release v$version"
git tag "v$version"
git push origin HEAD "v$version"

echo
echo "Pushed v$version. Follow the build with:  gh run watch"
echo "Then publish the draft at: https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/releases"
