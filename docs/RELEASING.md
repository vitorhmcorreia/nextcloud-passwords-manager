# Releasing (building installers)

## What's a tag / release?

- A **commit** is a snapshot of the code. Git gives each one an ugly id like `7c7a0d6`.
- A **tag** is a permanent, human-friendly label stuck on one commit, e.g. `v0.1.0`.
  By convention tags are version numbers: *this exact code is version 0.1.0*.
- A **GitHub Release** is a page on GitHub attached to a tag, where you can download files
  (our installers).

Our workflow (`.github/workflows/release.yml`) watches for tags that start with `v`.
When you push one, GitHub starts three machines (Linux, macOS, Windows). Each builds the
app from that tagged commit and uploads its installer to a **draft** release. A draft is
only visible to you until you press *Publish*.

## Versions

We use `MAJOR.MINOR.PATCH` ([semver](https://semver.org)):
- `0.1.0 → 0.1.1`: small fix
- `0.1.0 → 0.2.0`: new features
- `1.0.0`: when you consider it "done" for daily use

## How to release

```bash
scripts/release.sh 0.2.0
```

The script:
1. writes `0.2.0` into `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`
2. commits that as "Release v0.2.0"
3. creates the tag `v0.2.0`
4. pushes the commit and the tag, which starts the build

Then:
1. `gh run watch` (or the repo's **Actions** tab) shows progress. It takes about 10–20 min;
   macOS is the slowest.
2. Go to the repo's **Releases** page, open the draft, check the files are there, and press **Publish**.
3. Download the right file on each computer.

## Which file do I install?

| OS | File | Notes |
|---|---|---|
| Windows 10/11 | `ncpass_X.Y.Z_x64_en-US.msi` | SmartScreen may warn (unsigned): *More info → Run anyway* |
| macOS 12 | `ncpass_X.Y.Z_universal.dmg` | Drag to Applications. First launch: **right-click → Open** (unsigned) |
| Ubuntu/Debian | `ncpass_X.Y.Z_amd64.deb` | `sudo apt install ./ncpass_…deb` |
| Other Linux | `.AppImage` | `chmod +x` and run |

Each computer needs its own first-run setup (server, login, master password). The vault
file is per computer and is never synced between them. Each one syncs from Nextcloud.

## If a build fails

Open the failed run in the Actions tab and read the red step's log (or ask Claude 🙂).
Fix it, then release a new patch version (e.g. `0.2.1`). Don't reuse a tag.

## Cost note

The repo is public, so GitHub Actions minutes are free (including macOS).
