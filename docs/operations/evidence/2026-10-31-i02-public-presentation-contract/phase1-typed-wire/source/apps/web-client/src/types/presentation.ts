/** Frozen leaf data; producer disclosure and resource membership are separate responsibilities. */
export type FrozenOwnerKey =
  | { kind: "package"; namespace: string; package_id: string; version: string; logic_hash: string; content_hash: string }
  | { kind: "original"; repository: string; adopted_revision: string; catalog_sha256: string; asset_manifest_sha256: string };
export type QualifiedDefinitionKey =
  | { owner: Extract<FrozenOwnerKey, {kind: "package"}>; kind: "general" | "skill" | "card"; local_id: number }
  | { owner: Extract<FrozenOwnerKey, {kind: "original"}>; kind: "general" | "skill" | "card"; local_id: {pack: string; raw_symbol: string} };
export type BoundAudioAsset = {
  source: FrozenOwnerKey;
  reference: {kind: "package"; namespace: string; package_id: string; asset_id: string}
    | {kind: "original"; path: string; source_category: "core" | "compat"; source_path: string};
  digest_algorithm: "sha256" | "blake3"; digest: string; size: number;
};
export type PublicVoice = { role: QualifiedDefinitionKey; form: string | null; assets: BoundAudioAsset[] };
export type PresentationNature = "normal" | "fire" | "thunder";
export type EffectivePresentationCard = {kind: string; key: string; nature: PresentationNature};
export type PublicPresentationEvent =
  | {kind: "card_used"; actor: number; targets: number[] | null; effective: EffectivePresentationCard; voice: PublicVoice | null}
  | {kind: "card_responded"; actor: number; effective: EffectivePresentationCard; voice: PublicVoice | null}
  | {kind: "skill_activated"; actor: number; skill: {definition: QualifiedDefinitionKey}; targets: number[] | null; voice: PublicVoice | null}
  | {kind: "damage_dealt"; source: number | null; target: number; amount: number; nature: PresentationNature}
  | {kind: "damage_prevented"; source: number | null; target: number; nature: PresentationNature}
  | {kind: "hp_recovered"; actor: number; amount: number}
  | {kind: "player_died"; actor: number; killer: number | null; voice: PublicVoice | null};
export type PresentationDomain = "card_use" | "card_response" | "skill_activation" | "damage" | "recovery" | "death" | "voice_identity";
export type PublicPresentationBatch = {
  revision: number; status: "complete" | "incomplete" | "overflow";
  unavailable: {domain: PresentationDomain; reason: "source_no_audio_discriminator" | "commit_boundary_unproved" | "verified_binding_missing" | "public_binding_unavailable"}[];
  occurrences: {ordinal: number; event: PublicPresentationEvent}[];
};
