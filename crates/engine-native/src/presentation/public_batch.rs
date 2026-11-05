//! Shared volatile committed-range lifecycle and public presentation batch projector
//! (`I02` Decision 30, `proposal-v3 §§3–4`).
//!
//! Invariants:
//! - One private volatile `PresentationRange` on [`Engine`], holding only indices into
//!   `Engine.log` and the committed `revision`.
//! - No public raw-span or caller-chosen start getter exists (`public_presentation_from`
//!   is intentionally absent).
//! - `Engine::release_committed_presentation()` moves `Sealed` -> `Released`.
//! - `Engine::take_public_presentation()` consumes only `Released` once and transitions
//!   to `Idle`; a second take returns `None`.
//! - `unavailable` and `status` depend strictly on public static build/content capabilities
//!   and legitimate public occurrences, never on hidden/secret activations.

use engine_api::{
    BoundAudioAsset, FrozenOwnerKey, OriginalAudioCategory, PresentationAssetRef,
    PresentationDefinitionKind, PresentationDigestAlgorithm, PresentationDomain,
    PresentationLocalId, PresentationNature, PresentationStatus, PresentationUnavailableReason,
    PublicOccurrence, PublicPresentationBatch, PublicPresentationEvent, QualifiedDefinitionKey,
    UnavailablePresentationScope, MAX_PRESENTATION_BYTES, MAX_PRESENTATION_OCCURRENCES,
};

use crate::catalog::Catalog;
use crate::engine::Engine;
use crate::machine::LogEvent;

/// Private volatile committed-range state on [`Engine`] (`proposal-v3 §4`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PresentationRange {
    Idle,
    Held {
        start: usize,
        restored_prefix_unknown: bool,
    },
    Sealed {
        start: usize,
        end: usize,
        revision: u64,
    },
    Released {
        start: usize,
        end: usize,
        revision: u64,
    },
}

impl Default for PresentationRange {
    fn default() -> Self {
        Self::Idle
    }
}

impl Engine {
    /// Retire any unread `Sealed` or `Released` presentation range before a new command
    /// or baseline operation (`observer`/`start`/`status`/`chooseGeneral`/`restore`),
    /// while preserving an active open-segment `Held` range.
    pub fn retire_public_presentation(&mut self) {
        if matches!(
            self.presentation_range,
            PresentationRange::Sealed { .. } | PresentationRange::Released { .. }
        ) {
            self.presentation_range = PresentationRange::Idle;
        }
    }

    /// Clear staged `Sealed` / `Released` publication on a rollback checkpoint restore,
    /// retaining only a checkpoint-consistent `Held` range if `segment_open` is true.
    pub fn clear_staged_presentation_on_rollback(&mut self) {
        match self.presentation_range {
            PresentationRange::Held {
                start,
                restored_prefix_unknown,
            } if self.segment_open => {
                self.presentation_range = PresentationRange::Held {
                    start: start.min(self.log.len()),
                    restored_prefix_unknown,
                };
            }
            _ => {
                self.presentation_range = PresentationRange::Idle;
            }
        }
    }

    pub(crate) fn begin_presentation_hold(&mut self, log_mark: usize) {
        if self.simulation {
            self.presentation_range = PresentationRange::Idle;
            return;
        }
        self.presentation_range = PresentationRange::Held {
            start: log_mark.min(self.log.len()),
            restored_prefix_unknown: false,
        };
    }

    pub(crate) fn ensure_presentation_hold(&mut self, fallback_start: usize) {
        if self.simulation {
            self.presentation_range = PresentationRange::Idle;
            return;
        }
        if !matches!(self.presentation_range, PresentationRange::Held { .. }) {
            self.presentation_range = PresentationRange::Held {
                start: fallback_start.min(self.log.len()),
                restored_prefix_unknown: false,
            };
        }
    }

    pub(crate) fn discard_presentation_range(&mut self) {
        self.presentation_range = PresentationRange::Idle;
    }

    pub(crate) fn seal_presentation_range(&mut self, fallback_start: usize) {
        if self.simulation {
            self.presentation_range = PresentationRange::Idle;
            return;
        }
        match self.presentation_range {
            PresentationRange::Held {
                start: _,
                restored_prefix_unknown: true,
            } => {
                // Inherited open segment from archive restore: suppress once on closure.
                self.presentation_range = PresentationRange::Idle;
            }
            PresentationRange::Held {
                start,
                restored_prefix_unknown: false,
            } => {
                let end = self.log.len();
                let start = start.min(end);
                self.presentation_range = PresentationRange::Sealed {
                    start,
                    end,
                    revision: self.state.core.revision,
                };
            }
            _ => {
                let end = self.log.len();
                let start = fallback_start.min(end);
                self.presentation_range = PresentationRange::Sealed {
                    start,
                    end,
                    revision: self.state.core.revision,
                };
            }
        }
    }

    /// Release the range sealed by the just-committed command (`Sealed` -> `Released`).
    /// Never releases `Held` or `Idle`.
    pub fn release_committed_presentation(&mut self) {
        if self.simulation {
            self.presentation_range = PresentationRange::Idle;
            return;
        }
        if let PresentationRange::Sealed {
            start,
            end,
            revision,
        } = self.presentation_range
        {
            if revision == self.state.core.revision && start <= end && end <= self.log.len() {
                self.presentation_range = PresentationRange::Released {
                    start,
                    end,
                    revision,
                };
            } else {
                self.presentation_range = PresentationRange::Idle;
            }
        }
    }

    /// Consume the single `Released` presentation range and project it into a
    /// [`PublicPresentationBatch`]. Returns `None` if no range is `Released` or if
    /// already consumed.
    pub fn take_public_presentation(&mut self) -> Option<PublicPresentationBatch> {
        if self.simulation {
            self.presentation_range = PresentationRange::Idle;
            return None;
        }
        let PresentationRange::Released {
            start,
            end,
            revision,
        } = self.presentation_range
        else {
            return None;
        };
        self.presentation_range = PresentationRange::Idle;
        if revision != self.state.core.revision || start > end || end > self.log.len() {
            return None;
        }
        Some(project_public_batch(
            &self.log[start..end],
            &self.catalog,
            revision,
        ))
    }
}

fn push_scope(
    unavailable: &mut Vec<UnavailablePresentationScope>,
    domain: PresentationDomain,
    reason: PresentationUnavailableReason,
) {
    let scope = UnavailablePresentationScope { domain, reason };
    if !unavailable.contains(&scope) {
        unavailable.push(scope);
    }
}

/// Compute public static build/content capability gaps from `catalog` (`proposal-v3 §3`).
pub(crate) fn compute_static_unavailable_scopes(
    catalog: &Catalog,
) -> Vec<UnavailablePresentationScope> {
    let mut unavailable = Vec::new();

    let builtin_generals_bound = catalog
        .standard_generals
        .iter()
        .filter(|g| g.original_source.is_some())
        .all(|g| g.presentation_binding.is_some());
    let builtin_skills_bound = catalog
        .skills
        .iter()
        .filter(|s| s.original_source.is_some())
        .all(|s| s.presentation_binding.is_some());
    let builtin_cards_bound = catalog
        .cards
        .iter()
        .filter(|c| c.original_source.is_some())
        .all(|c| c.presentation_binding.is_some());

    if !builtin_generals_bound {
        push_scope(
            &mut unavailable,
            PresentationDomain::VoiceIdentity,
            PresentationUnavailableReason::VerifiedBindingMissing,
        );
    }
    if !builtin_skills_bound {
        push_scope(
            &mut unavailable,
            PresentationDomain::SkillActivation,
            PresentationUnavailableReason::VerifiedBindingMissing,
        );
    }
    if !builtin_cards_bound {
        push_scope(
            &mut unavailable,
            PresentationDomain::CardUse,
            PresentationUnavailableReason::VerifiedBindingMissing,
        );
        push_scope(
            &mut unavailable,
            PresentationDomain::CardResponse,
            PresentationUnavailableReason::VerifiedBindingMissing,
        );
    }

    if catalog
        .extension_generals
        .iter()
        .any(|g| g.presentation_binding.is_none())
    {
        push_scope(
            &mut unavailable,
            PresentationDomain::VoiceIdentity,
            PresentationUnavailableReason::VerifiedBindingMissing,
        );
    }

    // Any extension skill (within `extension_skill_range`, excluding R05 internal helpers)
    // that lacks a verified presentation binding registers a static capability gap.
    if catalog
        .extension_skill_range
        .clone()
        .filter(|&def_id| !catalog.is_r05_helper_def(def_id as u16))
        .any(|def_id| {
            catalog
                .skills
                .get(def_id)
                .is_some_and(|s| s.presentation_binding.is_none())
        })
    {
        push_scope(
            &mut unavailable,
            PresentationDomain::SkillActivation,
            PresentationUnavailableReason::VerifiedBindingMissing,
        );
    }

    unavailable
}

pub(crate) fn project_public_batch(
    slice: &[LogEvent],
    catalog: &Catalog,
    revision: u64,
) -> PublicPresentationBatch {
    let unavailable = compute_static_unavailable_scopes(catalog);
    let mut occurrences = Vec::new();
    let mut overflow = false;

    for ev in slice {
        let Some(pub_ev) = ev.public_presentation() else {
            continue;
        };
        if occurrences.len() >= MAX_PRESENTATION_OCCURRENCES {
            overflow = true;
            break;
        }
        let ordinal = occurrences.len() as u32;
        occurrences.push(PublicOccurrence {
            ordinal,
            event: pub_ev.clone(),
        });
    }

    let base_status = if unavailable.is_empty() {
        PresentationStatus::Complete
    } else {
        PresentationStatus::Incomplete
    };

    if overflow {
        return PublicPresentationBatch {
            revision,
            status: PresentationStatus::Overflow,
            unavailable,
            occurrences: Vec::new(),
        };
    }

    let candidate = PublicPresentationBatch {
        revision,
        status: base_status,
        unavailable: unavailable.clone(),
        occurrences,
    };

    if estimate_batch_json_bytes(&candidate) > MAX_PRESENTATION_BYTES {
        return PublicPresentationBatch {
            revision,
            status: PresentationStatus::Overflow,
            unavailable,
            occurrences: Vec::new(),
        };
    }

    candidate
}

fn decimal_digits_u64(mut n: u64) -> usize {
    if n == 0 {
        return 1;
    }
    let mut d = 0;
    while n > 0 {
        d += 1;
        n /= 10;
    }
    d
}

fn json_str_bytes(s: &str) -> usize {
    let mut extra = 0usize;
    for b in s.bytes() {
        match b {
            b'"' | b'\\' | b'\n' | b'\r' | b'\t' | 0x08 | 0x0c => extra += 1,
            0x00..=0x1f => extra += 5,
            _ => {}
        }
    }
    2 + s.len() + extra
}

fn owner_json_bytes(owner: &FrozenOwnerKey) -> usize {
    match owner {
        FrozenOwnerKey::Package {
            namespace,
            package_id,
            version,
            logic_hash,
            content_hash,
        } => {
            // {"kind":"package","namespace":..,"package_id":..,"version":..,"logic_hash":..,"content_hash":..}
            82 + json_str_bytes(namespace)
                + json_str_bytes(package_id)
                + json_str_bytes(version)
                + json_str_bytes(logic_hash)
                + json_str_bytes(content_hash)
        }
        FrozenOwnerKey::Original {
            repository,
            adopted_revision,
            catalog_sha256,
            asset_manifest_sha256,
        } => {
            // {"kind":"original","repository":..,"adopted_revision":..,"catalog_sha256":..,"asset_manifest_sha256":..}
            96 + json_str_bytes(repository)
                + json_str_bytes(adopted_revision)
                + json_str_bytes(catalog_sha256)
                + json_str_bytes(asset_manifest_sha256)
        }
    }
}

fn qualified_def_json_bytes(def: &QualifiedDefinitionKey) -> usize {
    let kind_len = match def.kind {
        PresentationDefinitionKind::General => 9, // "general"
        PresentationDefinitionKind::Skill => 7,   // "skill"
        PresentationDefinitionKind::Card => 6,    // "card"
    };
    let local_len = match &def.local_id {
        PresentationLocalId::Package(id) => decimal_digits_u64(u64::from(*id)),
        PresentationLocalId::Original(orig) => {
            // {"pack":..,"raw_symbol":..}
            23 + json_str_bytes(&orig.pack) + json_str_bytes(&orig.raw_symbol)
        }
    };
    // {"owner":..,"kind":..,"local_id":..}
    31 + owner_json_bytes(&def.owner) + kind_len + local_len
}

fn asset_json_bytes(a: &BoundAudioAsset) -> usize {
    let ref_len = match &a.reference {
        PresentationAssetRef::Package {
            namespace,
            package_id,
            asset_id,
        } => {
            // {"kind":"package","namespace":..,"package_id":..,"asset_id":..}
            53 + json_str_bytes(namespace)
                + json_str_bytes(package_id)
                + json_str_bytes(asset_id)
        }
        PresentationAssetRef::Original {
            path,
            source_category,
            source_path,
        } => {
            let cat_len = match source_category {
                OriginalAudioCategory::Core => 6,   // "core"
                OriginalAudioCategory::Compat => 8, // "compat"
            };
            // {"kind":"original","path":..,"source_category":..,"source_path":..}
            59 + json_str_bytes(path) + cat_len + json_str_bytes(source_path)
        }
    };
    let alg_len = match a.digest_algorithm {
        PresentationDigestAlgorithm::Blake3 => 8, // "blake3"
        PresentationDigestAlgorithm::Sha256 => 8, // "sha256"
    };
    // {"source":..,"reference":..,"digest_algorithm":..,"digest":..,"size":..}
    61 + owner_json_bytes(&a.source)
        + ref_len
        + alg_len
        + json_str_bytes(&a.digest)
        + decimal_digits_u64(a.size)
}

fn nature_json_bytes(n: PresentationNature) -> usize {
    match n {
        PresentationNature::Normal => 8,  // "normal"
        PresentationNature::Fire => 6,    // "fire"
        PresentationNature::Thunder => 9, // "thunder"
    }
}

/// Exact serialized JSON byte length of `batch` (matching `serde_json::to_string`).
pub(crate) fn estimate_batch_json_bytes(batch: &PublicPresentationBatch) -> usize {
    let status_len = match batch.status {
        PresentationStatus::Complete => 10,   // "complete"
        PresentationStatus::Incomplete => 12, // "incomplete"
        PresentationStatus::Overflow => 10,   // "overflow"
    };
    let mut unavail_len = 2usize; // []
    for (idx, u) in batch.unavailable.iter().enumerate() {
        if idx > 0 {
            unavail_len += 1;
        }
        let dom_len = match u.domain {
            PresentationDomain::CardUse => 10,
            PresentationDomain::CardResponse => 15,
            PresentationDomain::SkillActivation => 18,
            PresentationDomain::Damage => 8,
            PresentationDomain::Recovery => 10,
            PresentationDomain::Death => 7,
            PresentationDomain::VoiceIdentity => 16,
        };
        let reason_len = match u.reason {
            PresentationUnavailableReason::SourceNoAudioDiscriminator => 30,
            PresentationUnavailableReason::CommitBoundaryUnproved => 26,
            PresentationUnavailableReason::VerifiedBindingMissing => 26,
            PresentationUnavailableReason::PublicBindingUnavailable => 28,
        };
        // {"domain":..,"reason":..}
        unavail_len += 21 + dom_len + reason_len;
    }

    let mut occ_len = 2usize; // []
    for (idx, occ) in batch.occurrences.iter().enumerate() {
        if idx > 0 {
            occ_len += 1;
        }
        let voice_bytes = |v: &Option<engine_api::PublicVoice>| match v {
            None => 4,
            Some(pv) => {
                let form_len = pv.form.as_deref().map_or(4, json_str_bytes);
                let mut assets_len = 2usize;
                for (ai, a) in pv.assets.iter().enumerate() {
                    if ai > 0 {
                        assets_len += 1;
                    }
                    assets_len += asset_json_bytes(a);
                }
                // {"role":..,"form":..,"assets":..}
                28 + qualified_def_json_bytes(&pv.role) + form_len + assets_len
            }
        };
        let targets_bytes = |ts: &Option<Vec<engine_api::PlayerId>>| match ts {
            None => 4,
            Some(list) => {
                let mut l = 2usize;
                for (ti, p) in list.iter().enumerate() {
                    if ti > 0 {
                        l += 1;
                    }
                    l += decimal_digits_u64(u64::from(p.raw()));
                }
                l
            }
        };
        let effective_bytes = |e: &engine_api::EffectivePresentationCard| {
            // {"kind":..,"key":..,"nature":..}
            27 + json_str_bytes(&e.kind) + json_str_bytes(&e.key) + nature_json_bytes(e.nature)
        };
        let ev_len = match &occ.event {
            PublicPresentationEvent::CardUsed {
                actor,
                targets,
                effective,
                voice,
            } => {
                // {"kind":"card_used","actor":..,"targets":..,"effective":..,"voice":..}
                62 + decimal_digits_u64(u64::from(actor.raw()))
                    + targets_bytes(targets)
                    + effective_bytes(effective)
                    + voice_bytes(voice)
            }
            PublicPresentationEvent::CardResponded {
                actor,
                effective,
                voice,
            } => {
                // {"kind":"card_responded","actor":..,"effective":..,"voice":..}
                56 + decimal_digits_u64(u64::from(actor.raw()))
                    + effective_bytes(effective)
                    + voice_bytes(voice)
            }
            PublicPresentationEvent::SkillActivated {
                actor,
                skill,
                targets,
                voice,
            } => {
                // {"kind":"skill_activated","actor":..,"skill":{"definition":..},"targets":..,"voice":..}
                79 + decimal_digits_u64(u64::from(actor.raw()))
                    + qualified_def_json_bytes(&skill.definition)
                    + targets_bytes(targets)
                    + voice_bytes(voice)
            }
            PublicPresentationEvent::DamageDealt {
                source,
                target,
                amount,
                nature,
            } => {
                let src_len = source.map_or(4, |p| decimal_digits_u64(u64::from(p.raw())));
                // {"kind":"damage_dealt","source":..,"target":..,"amount":..,"nature":..}
                63 + src_len
                    + decimal_digits_u64(u64::from(target.raw()))
                    + decimal_digits_u64(u64::from(*amount))
                    + nature_json_bytes(*nature)
            }
            PublicPresentationEvent::DamagePrevented {
                source,
                target,
                nature,
            } => {
                let src_len = source.map_or(4, |p| decimal_digits_u64(u64::from(p.raw())));
                // {"kind":"damage_prevented","source":..,"target":..,"nature":..}
                57 + src_len
                    + decimal_digits_u64(u64::from(target.raw()))
                    + nature_json_bytes(*nature)
            }
            PublicPresentationEvent::HpRecovered { actor, amount } => {
                // {"kind":"hp_recovered","actor":..,"amount":..}
                42 + decimal_digits_u64(u64::from(actor.raw()))
                    + decimal_digits_u64(u64::from(*amount))
            }
            PublicPresentationEvent::PlayerDied {
                actor,
                killer,
                voice,
            } => {
                let killer_len = killer.map_or(4, |p| decimal_digits_u64(u64::from(p.raw())));
                // {"kind":"player_died","actor":..,"killer":..,"voice":..}
                49 + decimal_digits_u64(u64::from(actor.raw())) + killer_len + voice_bytes(voice)
            }
        };
        // {"ordinal":..,"event":..}
        occ_len += 21 + decimal_digits_u64(u64::from(occ.ordinal)) + ev_len;
    }

    // {"revision":..,"status":..,"unavailable":..,"occurrences":..}
    53 + decimal_digits_u64(batch.revision) + status_len + unavail_len + occ_len
}
