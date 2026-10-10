//! Dependency-ordered active presentation catalog.

use crate::RuntimeError;
use content_schema::{
    ContentPackage, PackageCharacterEntry, PackageSkinLayout, PackageSkinPresentation,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Default number of characters returned by one active-catalog page.
pub const DEFAULT_CATALOG_PAGE_LIMIT: u32 = 64;
/// Hard upper bound for one active-catalog page.
pub const MAX_CATALOG_PAGE_LIMIT: u32 = 200;

/// Unambiguous reference to an asset owned by one active package.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActiveAssetRef {
    /// Package namespace.
    pub namespace: String,
    /// Package id.
    pub package_id: String,
    /// Package-local asset id.
    pub asset_id: String,
}

/// One character in the active presentation catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveCharacterEntry {
    /// Namespace of the package that owns the roster definition.
    pub namespace: String,
    /// Package id that owns the roster definition.
    pub package_id: String,
    /// Rules/UI character definition.
    pub definition: PackageCharacterEntry,
    /// Qualified form of the legacy `portrait_asset` field.
    pub portrait_asset: Option<ActiveAssetRef>,
    /// Qualified forms of the legacy `skin_assets` field.
    pub legacy_skin_assets: Vec<ActiveAssetRef>,
    /// Direct and aliased stable skin ids available to this character.
    pub skin_ids: Vec<u32>,
}

/// Active, fully-qualified skin rendering model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActiveSkinPresentation {
    /// Ordinary selectable still image.
    Static {
        /// Qualified image asset.
        asset: ActiveAssetRef,
    },
    /// Spine bundle and its declarative renderer inputs.
    Spine {
        /// Qualified Spine bundle asset.
        bundle: ActiveAssetRef,
        /// Logical action key → Spine animation name.
        actions: BTreeMap<String, String>,
        /// Fixed-point safe layout.
        layout: PackageSkinLayout,
        /// Qualified bundle dependencies.
        dependencies: Vec<ActiveAssetRef>,
    },
    /// Rule-selected still-image substitute.
    Substitute {
        /// Safe host-defined rule.
        rule: String,
        /// Qualified image asset.
        asset: ActiveAssetRef,
    },
}

/// One active skin overlay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSkinEntry {
    /// Namespace of the package contributing this skin.
    pub namespace: String,
    /// Package id contributing this skin.
    pub package_id: String,
    /// Globally stable skin id.
    pub id: u32,
    /// Character receiving the direct overlay.
    pub character_id: u32,
    /// Localization key for the display name.
    pub name_key: String,
    /// Fully-qualified rendering model.
    pub presentation: ActiveSkinPresentation,
    /// Logical voice key → qualified audio asset.
    pub voice_assets: BTreeMap<String, ActiveAssetRef>,
}

/// One active character skin-sharing alias.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSkinAliasEntry {
    /// Namespace of the package contributing this alias.
    pub namespace: String,
    /// Package id contributing this alias.
    pub package_id: String,
    /// Character receiving shared skins.
    pub character_id: u32,
    /// Character whose skins are shared.
    pub source_character_id: u32,
    /// Receiving logical voice key → source logical voice key.
    pub voice_mappings: BTreeMap<String, String>,
}

/// Characters, translations, and cross-package skin overlays from the active package set.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveCatalog {
    /// Stable character id → character entry.
    pub characters: BTreeMap<u32, ActiveCharacterEntry>,
    /// Locale → translation key → translated text.
    pub translations: BTreeMap<String, BTreeMap<String, String>>,
    /// Stable skin id → skin entry.
    pub skins: BTreeMap<u32, ActiveSkinEntry>,
    /// Receiving character id → skin-sharing alias.
    pub aliases: BTreeMap<u32, ActiveSkinAliasEntry>,
}

/// Bounded reader-facing page of the active presentation catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveCatalogPage {
    /// Canonical active content-lock hash.
    pub content_lock_hash: String,
    /// Effective character offset.
    pub offset: u32,
    /// Effective page limit.
    pub limit: u32,
    /// Total active character count.
    pub total: u32,
    /// Character entries in this page.
    pub characters: Vec<ActiveCharacterEntry>,
    /// Concrete skins referenced by page characters.
    pub skins: Vec<ActiveSkinEntry>,
    /// Aliases whose receiving character is in this page.
    pub aliases: Vec<ActiveSkinAliasEntry>,
    /// Page-local translations for character and skin name keys.
    pub translations: BTreeMap<String, BTreeMap<String, String>>,
}

impl ActiveCatalog {
    /// Aggregate packages already ordered with dependencies before dependents.
    pub fn from_dependency_ordered_packages(
        packages: &[ContentPackage],
    ) -> Result<Self, RuntimeError> {
        let mut catalog = Self::default();
        let mut direct_skin_ids: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();

        for package in packages {
            package.validate().map_err(RuntimeError::Package)?;
            let namespace = package.manifest.namespace.as_str();
            let package_id = package.manifest.package_id.as_str();

            for character in &package.character_index {
                let entry = ActiveCharacterEntry {
                    namespace: namespace.into(),
                    package_id: package_id.into(),
                    definition: character.clone(),
                    portrait_asset: character
                        .portrait_asset
                        .as_deref()
                        .map(|asset_id| active_asset_ref(package, asset_id)),
                    legacy_skin_assets: character
                        .skin_assets
                        .iter()
                        .map(|asset_id| active_asset_ref(package, asset_id))
                        .collect(),
                    skin_ids: Vec::new(),
                };
                if catalog.characters.insert(character.id, entry).is_some() {
                    return Err(RuntimeError::Invalid(format!(
                        "duplicate active character id {}",
                        character.id
                    )));
                }
            }

            for (locale, entries) in &package.translations {
                let active = catalog.translations.entry(locale.clone()).or_default();
                for (key, value) in entries {
                    active.insert(key.clone(), value.clone());
                }
            }

            for skin in &package.skin_index.skins {
                if !catalog.characters.contains_key(&skin.character_id) {
                    return Err(RuntimeError::Invalid(format!(
                        "skin {} targets missing character {}",
                        skin.id, skin.character_id
                    )));
                }
                let presentation = match &skin.presentation {
                    PackageSkinPresentation::Static { asset } => ActiveSkinPresentation::Static {
                        asset: active_asset_ref(package, asset),
                    },
                    PackageSkinPresentation::Spine {
                        bundle,
                        actions,
                        layout,
                        dependencies,
                    } => ActiveSkinPresentation::Spine {
                        bundle: active_asset_ref(package, bundle),
                        actions: actions.clone(),
                        layout: layout.clone(),
                        dependencies: dependencies
                            .iter()
                            .map(|asset_id| active_asset_ref(package, asset_id))
                            .collect(),
                    },
                    PackageSkinPresentation::Substitute { rule, asset } => {
                        ActiveSkinPresentation::Substitute {
                            rule: rule.clone(),
                            asset: active_asset_ref(package, asset),
                        }
                    }
                };
                let entry = ActiveSkinEntry {
                    namespace: namespace.into(),
                    package_id: package_id.into(),
                    id: skin.id,
                    character_id: skin.character_id,
                    name_key: skin.name_key.clone(),
                    presentation,
                    voice_assets: skin
                        .voice_assets
                        .iter()
                        .map(|(key, asset_id)| (key.clone(), active_asset_ref(package, asset_id)))
                        .collect(),
                };
                if catalog.skins.insert(skin.id, entry).is_some() {
                    return Err(RuntimeError::Invalid(format!(
                        "duplicate active skin id {}",
                        skin.id
                    )));
                }
                direct_skin_ids
                    .entry(skin.character_id)
                    .or_default()
                    .insert(skin.id);
            }

            for alias in &package.skin_index.aliases {
                if !catalog.characters.contains_key(&alias.character_id) {
                    return Err(RuntimeError::Invalid(format!(
                        "skin alias targets missing character {}",
                        alias.character_id
                    )));
                }
                if !catalog.characters.contains_key(&alias.source_character_id) {
                    return Err(RuntimeError::Invalid(format!(
                        "skin alias references missing source character {}",
                        alias.source_character_id
                    )));
                }
                let entry = ActiveSkinAliasEntry {
                    namespace: namespace.into(),
                    package_id: package_id.into(),
                    character_id: alias.character_id,
                    source_character_id: alias.source_character_id,
                    voice_mappings: alias.voice_mappings.clone(),
                };
                if catalog.aliases.insert(alias.character_id, entry).is_some() {
                    return Err(RuntimeError::Invalid(format!(
                        "duplicate active skin alias for character {}",
                        alias.character_id
                    )));
                }
            }
        }

        let mut resolved = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        for character_id in catalog.characters.keys().copied() {
            resolve_skin_ids(
                character_id,
                &direct_skin_ids,
                &catalog.aliases,
                &mut resolved,
                &mut visiting,
            )?;
        }
        for (character_id, skin_ids) in resolved {
            catalog
                .characters
                .get_mut(&character_id)
                .expect("resolved character exists")
                .skin_ids = skin_ids.into_iter().collect();
        }

        Ok(catalog)
    }

    /// Look up a translated value.
    pub fn translation(&self, locale: &str, key: &str) -> Option<&str> {
        self.translations
            .get(locale)
            .and_then(|entries| entries.get(key))
            .map(String::as_str)
    }

    /// Iterate the concrete skins visible to one character, including shared aliases.
    pub fn skins_for_character(&self, character_id: u32) -> impl Iterator<Item = &ActiveSkinEntry> {
        self.characters
            .get(&character_id)
            .into_iter()
            .flat_map(|character| character.skin_ids.iter())
            .filter_map(|skin_id| self.skins.get(skin_id))
    }

    /// Build one bounded page; omitted values use offset zero and limit 64.
    pub fn page(
        &self,
        content_lock_hash: impl Into<String>,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> ActiveCatalogPage {
        let total = u32::try_from(self.characters.len()).unwrap_or(u32::MAX);
        let offset = offset.unwrap_or(0).min(total);
        let limit = limit
            .unwrap_or(DEFAULT_CATALOG_PAGE_LIMIT)
            .clamp(1, MAX_CATALOG_PAGE_LIMIT);
        let characters: Vec<_> = self
            .characters
            .values()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();
        let character_ids: BTreeSet<_> = characters
            .iter()
            .map(|character| character.definition.id)
            .collect();
        let skin_ids: BTreeSet<_> = characters
            .iter()
            .flat_map(|character| character.skin_ids.iter().copied())
            .collect();
        let skins: Vec<_> = skin_ids
            .iter()
            .filter_map(|skin_id| self.skins.get(skin_id).cloned())
            .collect();
        let aliases = character_ids
            .iter()
            .filter_map(|character_id| self.aliases.get(character_id).cloned())
            .collect();
        let name_keys: BTreeSet<_> = characters
            .iter()
            .map(|character| character.definition.name_key.as_str())
            .chain(skins.iter().map(|skin| skin.name_key.as_str()))
            .collect();
        let translations = self
            .translations
            .iter()
            .filter_map(|(locale, entries)| {
                let page_entries: BTreeMap<_, _> = name_keys
                    .iter()
                    .filter_map(|key| {
                        entries
                            .get(*key)
                            .map(|value| ((*key).to_owned(), value.clone()))
                    })
                    .collect();
                (!page_entries.is_empty()).then(|| (locale.clone(), page_entries))
            })
            .collect();

        ActiveCatalogPage {
            content_lock_hash: content_lock_hash.into(),
            offset,
            limit,
            total,
            characters,
            skins,
            aliases,
            translations,
        }
    }
}

fn active_asset_ref(package: &ContentPackage, asset_id: &str) -> ActiveAssetRef {
    ActiveAssetRef {
        namespace: package.manifest.namespace.clone(),
        package_id: package.manifest.package_id.clone(),
        asset_id: asset_id.into(),
    }
}

fn resolve_skin_ids(
    character_id: u32,
    direct: &BTreeMap<u32, BTreeSet<u32>>,
    aliases: &BTreeMap<u32, ActiveSkinAliasEntry>,
    resolved: &mut BTreeMap<u32, BTreeSet<u32>>,
    visiting: &mut BTreeSet<u32>,
) -> Result<BTreeSet<u32>, RuntimeError> {
    if let Some(skins) = resolved.get(&character_id) {
        return Ok(skins.clone());
    }
    if !visiting.insert(character_id) {
        return Err(RuntimeError::Invalid(format!(
            "skin alias cycle at character {character_id}"
        )));
    }
    let mut skins = direct.get(&character_id).cloned().unwrap_or_default();
    if let Some(alias) = aliases.get(&character_id) {
        skins.extend(resolve_skin_ids(
            alias.source_character_id,
            direct,
            aliases,
            resolved,
            visiting,
        )?);
    }
    visiting.remove(&character_id);
    resolved.insert(character_id, skins.clone());
    Ok(skins)
}
