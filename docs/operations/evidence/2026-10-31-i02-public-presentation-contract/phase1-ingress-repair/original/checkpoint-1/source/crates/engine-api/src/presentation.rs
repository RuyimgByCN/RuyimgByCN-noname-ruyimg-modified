//! Optional public presentation data. Structure is not proof of producer disclosure authority.
use crate::PlayerId;
use serde::{Deserialize, Deserializer, Serialize};

/// Maximum committed public occurrences in one batch.
pub const MAX_PRESENTATION_OCCURRENCES: usize = 1024;
/// Maximum encoded batch size at transport boundaries.
pub const MAX_PRESENTATION_BYTES: usize = 64 * 1024;
/// Largest integer exactly representable by every supported JSON consumer.
pub const MAX_PRESENTATION_REVISION: u64 = 9_007_199_254_740_991;

/// An exact frozen resource owner, without granting runtime authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FrozenOwnerKey {
    /// Existing LockedPackage identity fields.
    Package {
        /// Package namespace.
        namespace: String,
        /// Package slot.
        package_id: String,
        /// Exact installed version.
        version: String,
        /// BLAKE3 semantic hash.
        logic_hash: String,
        /// BLAKE3 original package byte hash.
        content_hash: String,
    },
    /// Pins for imported original presentation inputs.
    Original {
        /// Explicit repository identifier.
        repository: String,
        /// Adopted source commit.
        adopted_revision: String,
        /// SHA256 of the frozen catalog bytes.
        catalog_sha256: String,
        /// SHA256 of the frozen manifest bytes.
        asset_manifest_sha256: String,
    },
}

/// Genuine definition category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationDefinitionKind {
    /// General definition.
    General,
    /// Skill definition.
    Skill,
    /// Card definition.
    Card,
}

/// An original source symbol, never a translated name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OriginalDefinitionKey {
    /// Source pack.
    pub pack: String,
    /// Raw source symbol.
    pub raw_symbol: String,
}

/// Package-local numeric identity or explicit original source symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PresentationLocalId {
    /// Existing SDK definition id; zero rules follow the definition category.
    Package(u32),
    /// Exact imported symbol, valid only with an original owner.
    Original(OriginalDefinitionKey),
}

/// Qualified definition identity; numeric runtime indexes are not resource bindings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualifiedDefinitionKey {
    /// Frozen owner.
    pub owner: FrozenOwnerKey,
    /// Definition category.
    pub kind: PresentationDefinitionKind,
    /// Local id whose representation must match the owner.
    pub local_id: PresentationLocalId,
}

/// Exact existing asset reference shape, selected by source category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PresentationAssetRef {
    /// Existing ActiveAssetRef fields.
    Package {
        /// Namespace owning the asset.
        namespace: String,
        /// Package owning the asset.
        package_id: String,
        /// Package-local asset id.
        asset_id: String,
    },
    /// Existing original alias/backing reference.
    Original {
        /// Canonical manifest path.
        path: String,
        /// Core or verified compatibility backing.
        source_category: OriginalAudioCategory,
        /// Source-relative backing path.
        source_path: String,
    },
}

/// Existing original audio backing categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginalAudioCategory {
    /// Canonical original payload.
    Core,
    /// Verified compatibility derivative.
    Compat,
}

/// Payload digest algorithm, not the aggregate asset hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresentationDigestAlgorithm {
    /// Original payload digest.
    Sha256,
    /// Package manifest payload digest.
    Blake3,
}

/// A frozen audio payload; membership is checked by the future verified resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundAudioAsset {
    /// Explicit frozen source (including dependency assets).
    pub source: FrozenOwnerKey,
    /// Exact resource reference.
    pub reference: PresentationAssetRef,
    /// Payload hash algorithm.
    pub digest_algorithm: PresentationDigestAlgorithm,
    /// Lowercase hexadecimal payload digest.
    pub digest: String,
    /// Positive payload byte length.
    pub size: u64,
}

/// Public occurrence-time speaker declaration, not a cryptographic certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicVoice {
    /// Qualified genuine general definition.
    pub role: QualifiedDefinitionKey,
    /// Public form, or explicit null for the base role.
    #[serde(deserialize_with = "required_nullable")]
    pub form: Option<String>,
    /// Ordered payload candidates.
    pub assets: Vec<BoundAudioAsset>,
}

/// Actual effective card identity after validation and view-as conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectivePresentationCard {
    /// Existing CardKind key, distinct from physical name and audio basename.
    pub kind: String,
    /// Committed public use name.
    pub key: String,
    /// Actual damage nature.
    pub nature: PresentationNature,
}

/// Public modeled card/damage natures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationNature {
    /// Normal.
    Normal,
    /// Fire.
    Fire,
    /// Thunder.
    Thunder,
}

/// Committed skill identity, without a prompt/source_skill shortcut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPresentationSkill {
    /// Qualified skill definition.
    pub definition: QualifiedDefinitionKey,
}

/// Seven public committed occurrence types; producers are not implemented by these DTOs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PublicPresentationEvent {
    /// An effective card was used.
    CardUsed {
        /// API player id (one based).
        actor: PlayerId,
        /// Public targets, or explicit null when unavailable.
        #[serde(deserialize_with = "required_nullable")]
        targets: Option<Vec<PlayerId>>,
        /// Effective card.
        effective: EffectivePresentationCard,
        /// Public speaker or explicit null.
        #[serde(deserialize_with = "required_nullable")]
        voice: Option<PublicVoice>,
    },
    /// An effective card was responded.
    CardResponded {
        /// API player id.
        actor: PlayerId,
        /// Effective response.
        effective: EffectivePresentationCard,
        /// Public speaker or explicit null.
        #[serde(deserialize_with = "required_nullable")]
        voice: Option<PublicVoice>,
    },
    /// A genuine skill activation committed.
    SkillActivated {
        /// API player id.
        actor: PlayerId,
        /// Genuine qualified skill.
        skill: PublicPresentationSkill,
        /// Public targets or explicit null.
        #[serde(deserialize_with = "required_nullable")]
        targets: Option<Vec<PlayerId>>,
        /// Public speaker or explicit null.
        #[serde(deserialize_with = "required_nullable")]
        voice: Option<PublicVoice>,
    },
    /// Actual positive damage dealt.
    DamageDealt {
        /// Public source or explicit null.
        #[serde(deserialize_with = "required_nullable")]
        source: Option<PlayerId>,
        /// API target id.
        target: PlayerId,
        /// Actual positive amount.
        amount: u32,
        /// Final nature.
        nature: PresentationNature,
    },
    /// Damage was prevented.
    DamagePrevented {
        /// Public source or explicit null.
        #[serde(deserialize_with = "required_nullable")]
        source: Option<PlayerId>,
        /// API target id.
        target: PlayerId,
        /// Final nature.
        nature: PresentationNature,
    },
    /// Actual positive HP recovered.
    HpRecovered {
        /// API player id.
        actor: PlayerId,
        /// Actual positive amount.
        amount: u32,
    },
    /// A genuine death committed.
    PlayerDied {
        /// API player id.
        actor: PlayerId,
        /// Public killer or explicit null.
        #[serde(deserialize_with = "required_nullable")]
        killer: Option<PlayerId>,
        /// Public speaker or explicit null.
        #[serde(deserialize_with = "required_nullable")]
        voice: Option<PublicVoice>,
    },
}

/// Stable public ordinal within a committed revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicOccurrence {
    /// Zero-based contiguous ordinal; never a private log offset.
    pub ordinal: u32,
    /// Typed public occurrence.
    pub event: PublicPresentationEvent,
}

/// Public static capability domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationDomain {
    /// Card use.
    CardUse,
    /// Response.
    CardResponse,
    /// Genuine activation.
    SkillActivation,
    /// Damage.
    Damage,
    /// Recovery.
    Recovery,
    /// Death.
    Death,
    /// Voice binding.
    VoiceIdentity,
}

/// Static public capability gap, never derived from a hidden occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationUnavailableReason {
    /// Source has no proven discriminator.
    SourceNoAudioDiscriminator,
    /// True commitment boundary is unproved.
    CommitBoundaryUnproved,
    /// Verified qualified binding is absent.
    VerifiedBindingMissing,
    /// Public binding support is absent.
    PublicBindingUnavailable,
}

/// A public static capability gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnavailablePresentationScope {
    /// Capability domain.
    pub domain: PresentationDomain,
    /// Public static reason.
    pub reason: PresentationUnavailableReason,
}

/// Completeness of public presentation metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationStatus {
    /// All supported public occurrences included.
    Complete,
    /// Public static capability/history gap.
    Incomplete,
    /// Public metadata overflow, with no partial occurrences.
    Overflow,
}

/// Optional metadata, validated atomically before transport state changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawPublicPresentationBatch")]
pub struct PublicPresentationBatch {
    /// Existing committed revision.
    pub revision: u64,
    /// Public completeness status.
    pub status: PresentationStatus,
    /// Public static capability gaps.
    pub unavailable: Vec<UnavailablePresentationScope>,
    /// Ordered public occurrences.
    pub occurrences: Vec<PublicOccurrence>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPublicPresentationBatch {
    revision: u64,
    status: PresentationStatus,
    unavailable: Vec<UnavailablePresentationScope>,
    occurrences: Vec<PublicOccurrence>,
}

fn required_nullable<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}

impl TryFrom<RawPublicPresentationBatch> for PublicPresentationBatch {
    type Error = String;
    fn try_from(raw: RawPublicPresentationBatch) -> Result<Self, Self::Error> {
        let batch = Self {
            revision: raw.revision,
            status: raw.status,
            unavailable: raw.unavailable,
            occurrences: raw.occurrences,
        };
        batch.validate()?;
        Ok(batch)
    }
}

mod validation;
