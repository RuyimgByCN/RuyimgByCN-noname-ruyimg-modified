# Native GitHub Actions entry

The native source root contains `.github/workflows/ci.yml` and `Cargo.toml`.
A push or pull request runs the workflow on every branch. Publishing this tree
as a new branch does not change the repository's default branch.

The workflow checks out this root at `$GITHUB_WORKSPACE/noname-next`.
`prepare-original-content` checks out `libnoname/noname` at the adopted commit
`1a548c95acdf8ac1ef44d74473403eda2ac86cf0`, with `apps/core` in the sparse
checkout, at `$GITHUB_WORKSPACE/archive/upstream/noname`. Both checkout paths
are inside the workspace. This sibling layout matches the source paths in the
adopted manifest and catalog; no provenance fields are rewritten. Because
checkout creates an origin URL without `.git`, the action binds that existing
origin to the adopted `https://github.com/libnoname/noname.git` URL. This changes
only the new CI checkout's metadata, preserving its commit and clean source.

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
No optional skin source is configured.

Gates and native-web also request `integration-sources`. This checks out twelve
specific Liuli text files from `Helasisy1984/Noname-Liuli-Modified` at immutable
commit `8485dbb29ff16f9b258a766b1a17ef75f883aade`, under the sibling
`archive/upstream/noname-liuli/noname/...` paths used by registered tests.
Non-cone sparse patterns select only those files, without skin assets.

The existing Longxue test requires the already adopted local source variant:
the immutable HEAD lacks its `jsjiami` marker. The action applies the adjacent
`liuli-longxue.patch` to this new checkout only. SHA-256 assertions require the
exact original and adopted bytes before and after applying it. The other six
canonical local modifications are preserved outside CI; this text subset does
not claim their equivalence to the immutable HEAD. Of those six, only Epic
Cards is read directly by these tests; both versions satisfy its five source
predicates. Test content hashes refer to tracked converted packages, which are
not regenerated. No game rule or upstream version is changed by this restore.

Gates installs the same locked wasm-pack 0.15.0 as native-web before Gate 3.
This selects build-wasm's wasm-pack path, avoiding its existing fallback CLI
0.2.100 with the lockfile's wasm-bindgen 0.2.126. The build scripts and codegen
settings are unchanged.

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

Publishing a commit pushes its entire tracked tree and any reachable history
objects missing from the remote. A path list cannot filter that push. Existing
tracked web dist files and raw evidence remain part of the publication scope,
even when their paths also match ignore rules. Before seeking permission,
refresh the final accepted commit's `git ls-tree` list and history/object audit.
The baseline object's uncompressed byte count is not network transfer size.
Do not rewrite history or create an orphan publication branch as a shortcut.

A local Gate 4 pass and small input fixtures do not prove tool installation,
cross-platform builds, packaging, or artifact upload on GitHub runners. Record
an actual run URL and its individual job results after an explicitly authorized
branch push. Repository/API permissions are not publication authorization.
