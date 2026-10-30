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

The root `.gitattributes` preserves the exact bytes of
`content/original-audio-compat/provenance.json` with a path-specific `-text`
rule. The resource validator pins that file's SHA-256 as well as its parsed
bindings. See the [controlled Git checkout proof](evidence/2026-10-30-windows-provenance-checkout/README.md)
and the [777 publication's actual CI failure](evidence/2026-10-30-up37-i02-audio-compat/publication-ci/README.md).
The historical runner did not log its provenance SHA; controlled reproduction
and subsequent platform acceptance are recorded separately.

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

Perf runs both commands with `--release`, matching the historical phase17
measurement profile. The unchanged runner measures the legacy engine-core line;
profile diagnostics do not establish an engine-native performance improvement.

There are ten matrix job executions. The workflow requests only `contents: read`
for its token. Public source checkout and package installation require no new
repository secret. The existing optional `TAURI_SIGNING_PRIVATE_KEY` reference
is retained; the current Tauri configuration does not enable updater artifact
signing. Certificate signing and notarization are not configured. Installers
are workflow artifacts, not GitHub releases.

The 2026-10-30 installer correction is based on accepted source
`9dbc65530feb695688d3160dc38b6cf64ad39ecd` and first run `41053688083`.
Windows desktop failed with WiX `LGHT0311` (database codepage 1252);
Linux desktop failed with RPM name `无名杀-next`. The original raw logs are
`run41053688083-desktop-{windows,ubuntu}-raw.log` under
`docs/operations/evidence/2026-10-30-native-ci-preparation/first-publication/`
in the original publication evidence checkout. Their SHA-256 values are:

| Raw log | SHA-256 |
| --- | --- |
| Windows | `f1db45af81ad893e7c7cf82e013a7ed2929a28d7a760c971329c0405a9f78193` |
| Ubuntu | `0abac1175fde025798ad8e127ba0f7aa522adf753829a67bb17cc041dbb7045d` |

[The root configuration](../../apps/desktop-client/tauri.conf.json) retains
`无名杀 Next` and selects `bundle.windows.wix.language = "zh-CN"`.
First-CI Tauri CLI 2.12.4 / bundler 2.10.4 maps this to language 2052 and
codepage 936; WiX 3.14.1's zh-cn UI localization also sets database Codepage 936.
[The Linux configuration](../../apps/desktop-client/tauri.linux.conf.json)
automatically merges `productName = "noname-next"`, because the supported Linux
schema has no independent package-name field. This is the Linux package name,
bundle filename/resource-directory stem and desktop-file stem; the binary
remains `noname-next-desktop`. DEB and RPM use the same
[desktop template](../../apps/desktop-client/linux/main.desktop) to retain
`Name=无名杀 Next`; AppImage uses the DEB desktop generator. The window title
remains Chinese on all platforms. All default bundle targets remain enabled;
macOS identity, targets, security capabilities and signing settings are retained.

Local configuration/schema checks and the build-pipeline regressions cover
these metadata changes. Subsequent actual runs and uploaded corrected bundles
are registered in the [current progress](../architecture/native-progress.md#delivery-2026-10-24)
and the [independent alias-publication CI record](evidence/2026-10-30-up37-i02-audio-aliases/publication-ci/README.md).
The later [compat-publication CI record](evidence/2026-10-30-up37-i02-audio-compat/publication-ci/README.md)
retains its Windows resource-preparation failure separately.
The first run's successful macOS desktop artifact
`installer-macos-latest` (32,791,200 bytes), three OS build jobs and native-web
job do not establish a fully green workflow or validate these corrected bundles.

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
