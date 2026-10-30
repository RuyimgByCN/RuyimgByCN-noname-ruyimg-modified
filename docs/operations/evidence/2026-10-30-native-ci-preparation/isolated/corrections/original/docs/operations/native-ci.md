# Native GitHub Actions entry

The native source root contains `.github/workflows/ci.yml` and `Cargo.toml`.
A push or pull request runs the workflow on every branch. Publishing this tree
as a new branch does not change the repository's default branch.

The workflow checks out this root at `$GITHUB_WORKSPACE/noname-next`.
`prepare-original-content` checks out `libnoname/noname` at the adopted commit
`1a548c95acdf8ac1ef44d74473403eda2ac86cf0`, with `apps/core` in the sparse
checkout, at `$GITHUB_WORKSPACE/archive/upstream/noname`. Both checkout paths
are inside the workspace. This sibling layout matches the source paths in the
adopted manifest and catalog; no provenance fields are rewritten.

The shared action installs the web client's existing dependencies with pnpm 9
and `--frozen-lockfile`, then runs:

```sh
node scripts/sync-original-content.mjs --resources-only
node scripts/sync-original-content.mjs --resources-only --verify-only
git diff --exit-code -- content/original
```

This restores ignored portraits/audio from the real source files. It does not
regenerate the tracked legacy mirror, catalog, ported packages, or rules. The
final diff check rejects changes to tracked original content. Gate 4 still
runs the full original-content verification, plus ordering and resource tests.
No Liuli checkout or optional skin source is configured.

| Job | Platforms | Checks or output |
| --- | --- | --- |
| build | macOS, Windows, Ubuntu | Workspace build and library tests |
| gates | Ubuntu | Source size/layering, workspace tests, strict native targets, conformance, Native/WASM hash, original-content verification |
| native-web | Ubuntu | Fresh native and legacy WASM, native regressions, real Worker smoke, frontend tests/typecheck/build |
| desktop | macOS, Windows, Ubuntu | Tauri installers; uploads `installer-${matrix.os}` artifacts from `noname-next/target/release/bundle/**` |
| perf | macOS, Ubuntu | Existing tier list and server-baseline report |

There are ten matrix job executions. The workflow requests only `contents: read`
for its token. Public source checkout and package installation require no new
repository secret. The existing optional `TAURI_SIGNING_PRIVATE_KEY` reference
is retained; the current Tauri configuration does not enable updater artifact
signing. Certificate signing and notarization are not configured. Installers
are workflow artifacts, not GitHub releases.

A local Gate 4 pass does not prove checkout, tool installation, cross-platform
builds, packaging, or artifact upload on GitHub runners. Record an actual run
URL and its individual job results after an explicitly authorized branch push.
