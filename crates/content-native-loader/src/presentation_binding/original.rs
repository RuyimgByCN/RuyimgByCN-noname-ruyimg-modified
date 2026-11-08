//! Frozen original presentation verification and binding (`I02` Decision 30, `proposal-v3 §2`).
//!
//! Verifies `content/original/catalog.json` and `content/original/asset-manifest.json`
//! against build-pipeline SHA-256 pins and six-field alias/backing invariants, and
//! attaches [`DefinitionPresentationBinding`] records to built-in definitions that
//! declare `original_source`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use engine_api::{
    BoundAudioAsset, DefinitionPresentationBinding, FrozenOwnerKey, OriginalAudioCategory,
    OriginalDefinitionKey, PresentationAssetRef, PresentationDefinitionKind,
    PresentationDigestAlgorithm, PresentationLocalId, QualifiedDefinitionKey,
};
use engine_native::catalog::Catalog;
use serde_json::Value;

use super::push_unique_source;

/// Expected upstream repository recorded in `content/original/catalog.json` and `asset-manifest.json`.
pub const EXPECTED_ORIGINAL_REPOSITORY: &str = "https://github.com/libnoname/noname.git";
/// Expected upstream commit SHA-1 recorded in `content/original/catalog.json` and `asset-manifest.json`.
pub const EXPECTED_ORIGINAL_ADOPTED_REVISION: &str = "1a548c95acdf8ac1ef44d74473403eda2ac86cf0";
/// Trusted build-pipeline SHA-256 pin for `content/original/catalog.json`.
pub const EXPECTED_ORIGINAL_CATALOG_SHA256: &str =
    "2c375927fec9387a2a78e58db7d49422091fb17427c082e220db5e18ac0779fe";
/// Trusted build-pipeline SHA-256 pin for `content/original/asset-manifest.json`.
pub const EXPECTED_ORIGINAL_ASSET_MANIFEST_SHA256: &str =
    "7e5792aaefc8fbecd880581fbbea4dab5fed3c09d8a3ffe625fca65c4ffa5493";

const EMBEDDED_CATALOG_JSON: &[u8] =
    include_bytes!("../../../../content/original/catalog.json");
const EMBEDDED_ASSET_MANIFEST_JSON: &[u8] =
    include_bytes!("../../../../content/original/asset-manifest.json");

static EMBEDDED_CATALOG_LF: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
static EMBEDDED_MANIFEST_LF: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();

static EMBEDDED_VERIFIED_ORIGINAL: std::sync::OnceLock<
    Result<VerifiedOriginalPresentation, String>,
> = std::sync::OnceLock::new();

fn normalize_embedded_lf(
    raw: &'static [u8],
    cell: &'static std::sync::OnceLock<Vec<u8>>,
) -> &'static [u8] {
    if !raw.contains(&b'\r') {
        raw
    } else {
        cell.get_or_init(|| raw.iter().copied().filter(|&b| b != b'\r').collect())
            .as_slice()
    }
}

/// Canonical frozen original presentation inputs embedded from the repository's verified
/// `content/original/catalog.json` and `content/original/asset-manifest.json` (`I02 §2`).
pub fn embedded_original_presentation_input() -> OriginalPresentationInput<'static> {
    OriginalPresentationInput {
        catalog_json: normalize_embedded_lf(EMBEDDED_CATALOG_JSON, &EMBEDDED_CATALOG_LF),
        asset_manifest_json: normalize_embedded_lf(
            EMBEDDED_ASSET_MANIFEST_JSON,
            &EMBEDDED_MANIFEST_LF,
        ),
    }
}

/// Attach the embedded verified original presentation bindings onto `engine`'s catalog (`I02 §2`).
pub fn attach_embedded_original_presentation(
    engine: &mut engine_native::Engine,
) -> Result<(), String> {
    let orig = verify_original_presentation(&embedded_original_presentation_input())?;
    engine.try_with_catalog_mutation(|cat| attach_original_presentation_bindings(cat, &orig))
}

/// Raw bytes of the frozen original catalog and asset manifest supplied to the controlled factory.
#[derive(Debug, Clone, Copy)]
pub struct OriginalPresentationInput<'a> {
    /// Raw UTF-8 bytes of `content/original/catalog.json`.
    pub catalog_json: &'a [u8],
    /// Raw UTF-8 bytes of `content/original/asset-manifest.json`.
    pub asset_manifest_json: &'a [u8],
}

/// Verified original presentation tables ready to attach to a [`Catalog`].
#[derive(Debug, Clone)]
pub struct VerifiedOriginalPresentation {
    pub(crate) owner: FrozenOwnerKey,
    pub(crate) packages: BTreeSet<String>,
    pub(crate) characters: BTreeMap<(String, String), Vec<String>>,
    pub(crate) skills: BTreeSet<(String, String)>,
    pub(crate) character_voices: BTreeMap<(String, String), BTreeMap<String, Vec<BoundAudioAsset>>>,
    pub(crate) skill_voices: BTreeMap<(String, String), BTreeMap<String, Vec<BoundAudioAsset>>>,
    pub(crate) card_voices: BTreeMap<String, BTreeMap<String, Vec<BoundAudioAsset>>>,
}

impl VerifiedOriginalPresentation {
    /// Frozen owner key verified from the original catalog and asset manifest.
    pub fn owner(&self) -> &FrozenOwnerKey {
        &self.owner
    }
}

/// Pure-Rust FIPS 180-4 SHA-256 lowercase hexadecimal digest.
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(((data.len() + 9 + 63) / 64) * 64);
    padded.extend_from_slice(data);
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = String::with_capacity(64);
    for word in h {
        use std::fmt::Write;
        let _ = write!(&mut out, "{word:08x}");
    }
    out
}

fn is_safe_token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 256
        && s != "."
        && s != ".."
        && !s.chars().any(|c| c.is_control() || "/\\%?#:".contains(c))
}

fn is_safe_path(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && s.split('/').all(is_safe_token)
}

fn is_lower_hex_64(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn is_audio_extension(path: &str) -> bool {
    [
        ".mp3", ".ogg", ".wav", ".m4a", ".aac", ".flac", ".opus", ".mp4",
    ]
    .iter()
    .any(|ext| path.ends_with(ext))
}

/// Verify `OriginalPresentationInput` against the trusted build-pipeline SHA-256 pins.
pub fn verify_original_presentation(
    input: &OriginalPresentationInput<'_>,
) -> Result<VerifiedOriginalPresentation, String> {
    let embedded = embedded_original_presentation_input();
    if std::ptr::eq(input.catalog_json.as_ptr(), embedded.catalog_json.as_ptr())
        && input.catalog_json.len() == embedded.catalog_json.len()
        && std::ptr::eq(
            input.asset_manifest_json.as_ptr(),
            embedded.asset_manifest_json.as_ptr(),
        )
        && input.asset_manifest_json.len() == embedded.asset_manifest_json.len()
    {
        return EMBEDDED_VERIFIED_ORIGINAL
            .get_or_init(|| {
                verify_original_presentation_with_pins(
                    input,
                    EXPECTED_ORIGINAL_CATALOG_SHA256,
                    EXPECTED_ORIGINAL_ASSET_MANIFEST_SHA256,
                )
            })
            .clone();
    }
    verify_original_presentation_with_pins(
        input,
        EXPECTED_ORIGINAL_CATALOG_SHA256,
        EXPECTED_ORIGINAL_ASSET_MANIFEST_SHA256,
    )
}

/// Verify `OriginalPresentationInput` against explicit expected SHA-256 pins.
pub fn verify_original_presentation_with_pins(
    input: &OriginalPresentationInput<'_>,
    expected_catalog_sha256: &str,
    expected_manifest_sha256: &str,
) -> Result<VerifiedOriginalPresentation, String> {
    let catalog_sha256 = sha256_hex(input.catalog_json);
    if catalog_sha256 != expected_catalog_sha256 {
        return Err(format!(
            "original catalog.json sha256 mismatch: expected {expected_catalog_sha256}, got {catalog_sha256}"
        ));
    }
    let asset_manifest_sha256 = sha256_hex(input.asset_manifest_json);
    if asset_manifest_sha256 != expected_manifest_sha256 {
        return Err(format!(
            "original asset-manifest.json sha256 mismatch: expected {expected_manifest_sha256}, got {asset_manifest_sha256}"
        ));
    }
    let catalog: Value =
        serde_json::from_slice(input.catalog_json).map_err(|e| format!("catalog.json: {e}"))?;
    let manifest: Value = serde_json::from_slice(input.asset_manifest_json)
        .map_err(|e| format!("asset-manifest.json: {e}"))?;

    if catalog.get("schema_version").and_then(Value::as_u64) != Some(4) {
        return Err("catalog.json schema_version must be 4".into());
    }
    if manifest.get("schema_version").and_then(Value::as_u64) != Some(2) {
        return Err("asset-manifest.json schema_version must be 2".into());
    }
    let cat_core = &catalog["source"]["provenance"]["core"];
    let man_core = &manifest["sources"]["core"];
    let cat_repo = cat_core
        .get("repository")
        .and_then(Value::as_str)
        .ok_or_else(|| "catalog.json missing core repository".to_string())?;
    let cat_head = cat_core
        .get("head")
        .and_then(Value::as_str)
        .ok_or_else(|| "catalog.json missing core head".to_string())?;
    let man_repo = man_core
        .get("repository")
        .and_then(Value::as_str)
        .ok_or_else(|| "asset-manifest.json missing core repository".to_string())?;
    let man_head = man_core
        .get("head")
        .and_then(Value::as_str)
        .ok_or_else(|| "asset-manifest.json missing core head".to_string())?;
    if cat_repo != EXPECTED_ORIGINAL_REPOSITORY
        || man_repo != EXPECTED_ORIGINAL_REPOSITORY
        || cat_head != EXPECTED_ORIGINAL_ADOPTED_REVISION
        || man_head != EXPECTED_ORIGINAL_ADOPTED_REVISION
        || cat_core.get("dirty").and_then(Value::as_bool) != Some(false)
        || man_core.get("dirty").and_then(Value::as_bool) != Some(false)
    {
        return Err("original provenance core repository/head/dirty mismatch".into());
    }

    let owner = FrozenOwnerKey::Original {
        repository: cat_repo.to_string(),
        adopted_revision: cat_head.to_string(),
        catalog_sha256,
        asset_manifest_sha256,
    };

    let mut packages = BTreeSet::new();
    for pkg in catalog
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| "catalog.json packages must be an array".to_string())?
    {
        let p = pkg
            .as_str()
            .ok_or_else(|| "catalog.json packages[] must be string".to_string())?;
        if !is_safe_token(p) {
            return Err(format!("invalid package token {p} in catalog.json"));
        }
        packages.insert(p.to_string());
    }

    let mut characters = BTreeMap::new();
    for ch in catalog
        .get("characters")
        .and_then(Value::as_array)
        .ok_or_else(|| "catalog.json characters must be an array".to_string())?
    {
        let pack = ch
            .get("pack")
            .and_then(Value::as_str)
            .ok_or_else(|| "catalog.json character missing pack".to_string())?;
        let name = ch
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "catalog.json character missing name".to_string())?;
        if !is_safe_token(pack) || !is_safe_token(name) {
            return Err(format!("invalid character key {pack}:{name}"));
        }
        let mut sk_list = Vec::new();
        for sk in ch
            .get("skills")
            .and_then(Value::as_array)
            .ok_or_else(|| "catalog.json character.skills must be an array".to_string())?
        {
            if let Some(s) = sk.as_str() {
                if is_safe_token(s) {
                    sk_list.push(s.to_string());
                }
            }
        }
        if characters
            .insert((pack.to_string(), name.to_string()), sk_list)
            .is_some()
        {
            return Err(format!("duplicate character {pack}:{name} in catalog.json"));
        }
    }

    let mut skills = BTreeSet::new();
    for sk in catalog
        .get("skills")
        .and_then(Value::as_array)
        .ok_or_else(|| "catalog.json skills must be an array".to_string())?
    {
        let pack = sk
            .get("pack")
            .and_then(Value::as_str)
            .ok_or_else(|| "catalog.json skill missing pack".to_string())?;
        let name = sk
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "catalog.json skill missing name".to_string())?;
        if !is_safe_token(pack) || !is_safe_token(name) {
            return Err(format!("invalid skill key {pack}:{name}"));
        }
        skills.insert((pack.to_string(), name.to_string()));
    }

    let assets = manifest
        .get("assets")
        .and_then(Value::as_array)
        .ok_or_else(|| "asset-manifest.json assets must be an array".to_string())?;

    #[derive(Clone)]
    struct BackingRow {
        path: String,
        source_category: OriginalAudioCategory,
        source_path: String,
        sha256: String,
        size: u64,
    }

    let mut seen_paths = BTreeSet::new();
    let mut backing_map: BTreeMap<(String, String), BackingRow> = BTreeMap::new();
    let mut card_voices: BTreeMap<String, BTreeMap<String, Vec<BoundAudioAsset>>> = BTreeMap::new();

    for row in assets {
        let obj = row
            .as_object()
            .ok_or_else(|| "asset row must be an object".to_string())?;
        if obj.len() != 6 {
            return Err("asset row must have exactly 6 fields".into());
        }
        let kind = obj
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| "asset row missing kind".to_string())?;
        let path = obj
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| "asset row missing path".to_string())?;
        if !seen_paths.insert(path.to_string()) {
            return Err(format!("duplicate path {path} in asset-manifest.json"));
        }
        if kind != "audio" || path.starts_with("audio-declared/") {
            continue;
        }
        let cat_str = obj
            .get("source_category")
            .and_then(Value::as_str)
            .ok_or_else(|| "audio row missing source_category".to_string())?;
        let source_category = match cat_str {
            "core" => OriginalAudioCategory::Core,
            "compat" => OriginalAudioCategory::Compat,
            other => return Err(format!("invalid audio source_category {other}")),
        };
        let source_path = obj
            .get("source_path")
            .and_then(Value::as_str)
            .ok_or_else(|| "audio row missing source_path".to_string())?;
        let sha256 = obj
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| "audio row missing sha256".to_string())?;
        let size = obj
            .get("size")
            .and_then(Value::as_u64)
            .ok_or_else(|| "audio row missing size".to_string())?;
        let expected_path = match source_category {
            OriginalAudioCategory::Core => source_path.to_string(),
            OriginalAudioCategory::Compat => source_path.replacen("audio/", "audio-compat/", 1),
        };
        if !is_safe_path(path)
            || !is_safe_path(source_path)
            || !source_path.starts_with("audio/")
            || path != expected_path
            || !is_lower_hex_64(sha256)
            || size == 0
            || !is_audio_extension(path)
        {
            return Err(format!("invalid backing audio row {path}"));
        }
        if backing_map
            .insert(
                (cat_str.to_string(), source_path.to_string()),
                BackingRow {
                    path: path.to_string(),
                    source_category,
                    source_path: source_path.to_string(),
                    sha256: sha256.to_string(),
                    size,
                },
            )
            .is_some()
        {
            return Err(format!(
                "duplicate backing audio source {cat_str}:{source_path}"
            ));
        }
        if source_category == OriginalAudioCategory::Core {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() == 4
                && parts[0] == "audio"
                && parts[1] == "card"
                && matches!(parts[2], "male" | "female")
            {
                if let Some(stem) = parts[3].strip_suffix(".mp3") {
                    let bound = BoundAudioAsset {
                        source: owner.clone(),
                        reference: PresentationAssetRef::Original {
                            path: path.to_string(),
                            source_category,
                            source_path: source_path.to_string(),
                        },
                        digest_algorithm: PresentationDigestAlgorithm::Sha256,
                        digest: sha256.to_string(),
                        size,
                    };
                    let entry = card_voices.entry(stem.to_string()).or_default();
                    entry
                        .entry(parts[2].to_string())
                        .or_default()
                        .push(bound.clone());
                    if parts[2] == "male" {
                        entry.entry("default".to_string()).or_default().push(bound);
                    }
                }
            }
        }
    }

    // Populate elemental sha variants on the base "sha" card entry as well.
    for (suffix, elem_key) in [("sha_fire", "fire"), ("sha_thunder", "thunder")] {
        if let Some(elem_map) = card_voices.get(suffix).cloned() {
            let sha_entry = card_voices.entry("sha".to_string()).or_default();
            for gender in ["male", "female"] {
                if let Some(list) = elem_map.get(gender) {
                    sha_entry.insert(format!("{gender}:{elem_key}"), list.clone());
                }
            }
        }
    }

    let mut character_voices: BTreeMap<(String, String), BTreeMap<String, Vec<BoundAudioAsset>>> =
        BTreeMap::new();
    let mut skill_voices: BTreeMap<(String, String), BTreeMap<String, Vec<BoundAudioAsset>>> =
        BTreeMap::new();

    for row in assets {
        let obj = row.as_object().unwrap();
        let path = obj["path"].as_str().unwrap();
        if !path.starts_with("audio-declared/") {
            continue;
        }
        let kind = obj
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| "alias row missing kind".to_string())?;
        let cat_str = obj
            .get("source_category")
            .and_then(Value::as_str)
            .ok_or_else(|| "alias row missing source_category".to_string())?;
        let source_category = match cat_str {
            "core" => OriginalAudioCategory::Core,
            "compat" => OriginalAudioCategory::Compat,
            other => return Err(format!("invalid alias source_category {other}")),
        };
        let source_path = obj
            .get("source_path")
            .and_then(Value::as_str)
            .ok_or_else(|| "alias row missing source_path".to_string())?;
        let sha256 = obj
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| "alias row missing sha256".to_string())?;
        let size = obj
            .get("size")
            .and_then(Value::as_u64)
            .ok_or_else(|| "alias row missing size".to_string())?;
        if kind != "audio"
            || !is_safe_path(path)
            || !is_safe_path(source_path)
            || !source_path.starts_with("audio/")
            || !is_lower_hex_64(sha256)
            || size == 0
            || !is_audio_extension(path)
        {
            return Err(format!("invalid audio-declared row {path}"));
        }
        let backing = backing_map
            .get(&(cat_str.to_string(), source_path.to_string()))
            .ok_or_else(|| format!("missing backing for {path} -> {cat_str}:{source_path}"))?;
        if backing.source_category != source_category
            || backing.source_path != source_path
            || backing.sha256 != sha256
            || backing.size != size
            || path.rsplit('/').next() != backing.path.rsplit('/').next()
        {
            return Err(format!("backing mismatch for {path}"));
        }
        let bound = BoundAudioAsset {
            source: owner.clone(),
            reference: PresentationAssetRef::Original {
                path: path.to_string(),
                source_category,
                source_path: source_path.to_string(),
            },
            digest_algorithm: PresentationDigestAlgorithm::Sha256,
            digest: sha256.to_string(),
            size,
        };
        let parts: Vec<&str> = path.split('/').collect();
        match parts.as_slice() {
            ["audio-declared", pack, "die", char_sym, _idx, _file] => {
                let map = character_voices
                    .entry(((*pack).to_string(), (*char_sym).to_string()))
                    .or_default();
                push_unique_source(map.entry("die".to_string()).or_default(), bound.clone());
                push_unique_source(map.entry("default".to_string()).or_default(), bound);
            }
            ["audio-declared", pack, "character", char_sym, "die", variant, _idx, _file] => {
                let map = character_voices
                    .entry(((*pack).to_string(), (*char_sym).to_string()))
                    .or_default();
                if *variant == "base" {
                    push_unique_source(map.entry("die".to_string()).or_default(), bound.clone());
                    push_unique_source(map.entry("default".to_string()).or_default(), bound);
                } else if let Some(form) = variant.strip_prefix("form-") {
                    push_unique_source(map.entry(format!("die:form:{form}")).or_default(), bound);
                }
            }
            ["audio-declared", pack, "character", char_sym, "skill", skill_sym, variant, _idx, _file] =>
            {
                let char_key = ((*pack).to_string(), (*char_sym).to_string());
                if characters
                    .get(&char_key)
                    .is_some_and(|declared| declared.iter().any(|s| s == skill_sym))
                {
                    let map = character_voices.entry(char_key).or_default();
                    if *variant == "base" {
                        push_unique_source(
                            map.entry(format!("skill:{skill_sym}")).or_default(),
                            bound,
                        );
                    } else if let Some(form) = variant.strip_prefix("form-") {
                        push_unique_source(
                            map.entry(format!("skill:{skill_sym}:form:{form}"))
                                .or_default(),
                            bound,
                        );
                    }
                }
            }
            ["audio-declared", pack, "skill", skill_sym, variant, _idx, _file] => {
                let map = skill_voices
                    .entry(((*pack).to_string(), (*skill_sym).to_string()))
                    .or_default();
                if *variant == "base" {
                    push_unique_source(
                        map.entry("default".to_string()).or_default(),
                        bound.clone(),
                    );
                    push_unique_source(map.entry("activate".to_string()).or_default(), bound);
                } else if let Some(role) = variant.strip_prefix("name-") {
                    push_unique_source(map.entry(format!("role:{role}")).or_default(), bound);
                }
            }
            _ => return Err(format!("unrecognized audio-declared path shape {path}")),
        }
    }

    Ok(VerifiedOriginalPresentation {
        owner,
        packages,
        characters,
        skills,
        character_voices,
        skill_voices,
        card_voices,
    })
}

/// Attach verified original presentation bindings to all built-in definitions on `catalog`
/// that declare `original_source`. Atomically rejects before mutating `catalog` if any
/// declared `original_source` symbol is missing from the verified original catalog.
pub fn attach_original_presentation_bindings(
    catalog: &mut Catalog,
    verified: &VerifiedOriginalPresentation,
) -> Result<(), String> {
    for g in catalog
        .standard_generals
        .iter()
        .chain(catalog.boss_generals.iter())
    {
        if let Some(orig) = g.original_source {
            let key = (orig.pack.to_string(), orig.raw_symbol.to_string());
            if !verified.characters.contains_key(&key) {
                return Err(format!(
                    "built-in general {} original_source {}:{} missing from catalog.json",
                    g.name, orig.pack, orig.raw_symbol
                ));
            }
        }
    }
    for s in &catalog.skills {
        if let Some(orig) = s.original_source {
            let key = (orig.pack.to_string(), orig.raw_symbol.to_string());
            if !verified.skills.contains(&key) {
                return Err(format!(
                    "built-in skill {} original_source {}:{} missing from catalog.json",
                    s.name, orig.pack, orig.raw_symbol
                ));
            }
        }
    }
    for c in &catalog.cards {
        if let Some(orig) = c.original_source {
            if !verified.packages.contains(orig.pack) && orig.pack != "standard" {
                return Err(format!(
                    "built-in card {} original_source pack {} missing",
                    c.name, orig.pack
                ));
            }
        }
    }

    for g in catalog
        .standard_generals
        .iter_mut()
        .chain(catalog.boss_generals.iter_mut())
    {
        if let Some(orig) = g.original_source {
            let key = (orig.pack.to_string(), orig.raw_symbol.to_string());
            let voice_assets = verified
                .character_voices
                .get(&key)
                .cloned()
                .unwrap_or_default();
            g.presentation_binding = Some(Arc::new(DefinitionPresentationBinding {
                definition: QualifiedDefinitionKey {
                    owner: verified.owner.clone(),
                    kind: PresentationDefinitionKind::General,
                    local_id: PresentationLocalId::Original(OriginalDefinitionKey {
                        pack: orig.pack.to_string(),
                        raw_symbol: orig.raw_symbol.to_string(),
                    }),
                },
                voice_assets,
            }));
        }
    }
    for s in &mut catalog.skills {
        if let Some(orig) = s.original_source {
            let key = (orig.pack.to_string(), orig.raw_symbol.to_string());
            let voice_assets = verified.skill_voices.get(&key).cloned().unwrap_or_default();
            s.presentation_binding = Some(Arc::new(DefinitionPresentationBinding {
                definition: QualifiedDefinitionKey {
                    owner: verified.owner.clone(),
                    kind: PresentationDefinitionKind::Skill,
                    local_id: PresentationLocalId::Original(OriginalDefinitionKey {
                        pack: orig.pack.to_string(),
                        raw_symbol: orig.raw_symbol.to_string(),
                    }),
                },
                voice_assets,
            }));
        }
    }
    for c in &mut catalog.cards {
        if let Some(orig) = c.original_source {
            let voice_assets = verified
                .card_voices
                .get(orig.raw_symbol)
                .cloned()
                .unwrap_or_default();
            c.presentation_binding = Some(Arc::new(DefinitionPresentationBinding {
                definition: QualifiedDefinitionKey {
                    owner: verified.owner.clone(),
                    kind: PresentationDefinitionKind::Card,
                    local_id: PresentationLocalId::Original(OriginalDefinitionKey {
                        pack: orig.pack.to_string(),
                        raw_symbol: orig.raw_symbol.to_string(),
                    }),
                },
                voice_assets,
            }));
        }
    }
    Ok(())
}
