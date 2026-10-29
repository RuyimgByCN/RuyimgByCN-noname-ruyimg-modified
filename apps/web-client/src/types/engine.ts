/** Shared engine / observer types (mirrors engine-api JSON). */

export type PlayerId = number;
export type CardId = number;

export interface SkillView {
  id: number;
  def: number;
  /** 原生窄壳按技能名投影（选将后刷新可见）；旧线可能不提供。 */
  name?: string;
  enabled: boolean;
  use_count: number;
  active: boolean;
  view_as: boolean;
  locked: boolean;
}

export interface SeatView {
  player: number;
  character_def: number;
  seat: number;
  hp: number;
  max_hp: number;
  alive: boolean;
  turned_over: boolean;
  hand_count: number;
  identity: "Lord" | "Loyalist" | "Rebel" | "Renegade" | null;
  skills: SkillView[];
}

export interface CardView {
  id: number;
  def: number | null;
  suit: "Spade" | "Heart" | "Club" | "Diamond" | null;
  rank: number | null;
  owner: number | null;
  zone: "DrawPile" | "DiscardPile" | "Hand" | "Equip" | "Judge" | "Processing" | "Removed";
  kind:
    | "Sha"
    | "Tao"
    | "Shan"
    | "Jiu"
    | "ShaFire"
    | "ShaThunder"
    | "Weapon"
    | "Armor"
    | "HorseOff"
    | "HorseDef"
    | "TrickWuzhong"
    | "TrickGuohe"
    | "TrickShunshou"
    | "TrickJuedou"
    | "TrickNanman"
    | "TrickWanjian"
    | "TrickTaoyuan"
    | "TrickWugu"
    | "TrickLe"
    | "TrickBingliang"
    | "TrickShandian"
    | "TrickJiedao"
    | "TrickTiesuo"
    | "TrickHuogong"
    | "TrickWuxie"
    | "Other";
}

export interface ObserverSnapshot {
  revision: number;
  state_hash: number[] | string;
  observer: unknown;
  round: number;
  current_player: number;
  phase: "Prepare" | "Judge" | "Draw" | "Play" | "Discard" | "End" | "Idle";
  seats: SeatView[];
  known_cards: CardView[];
  outcome: unknown;
}

export interface ChoiceRequest {
  choice_id: number;
  observer: number;
  options?: { token: number; label_key: string }[];
  min_count?: number;
  max_count?: number;
  allow_cancel?: boolean;
  revision?: number;
  kind?: string;
}

export interface EngineEnvelope {
  result: unknown;
  events: unknown[];
  choice: ChoiceRequest | null;
  view: ObserverSnapshot;
  legal_actions: LegalAction[];
  hash_hex: string;
  revision: number;
  finished: boolean;
}

export type LegalAction =
  | "EndPhase"
  | { PlayCard: { card: number; targets: number[] } }
  | {
      ActivateSkill: {
        skill: number;
        targets: number[];
        card_targets: number[];
      };
    }
  | {
      UseSkillCard: {
        skill: number;
        card_kind: number;
        targets: number[];
        equip_slot: number;
      };
    }
  | {
      ResolveChoice: {
        choice_id: number;
        selections: number[];
        cancelled: boolean;
      };
    };

export interface ReplayMetadata {
  total: number;
  manifest_hash: string;
  chain_verified: boolean;
}

export interface ReplayFrame {
  step: number;
  total: number;
  view: ObserverSnapshot;
  hash_hex: string;
  revision: number;
  finished: boolean;
  verified: boolean;
}

export type ReplayPrivacy = "full" | "player" | "public";

export interface ContentAssetRef {
  namespace: string;
  package_id: string;
  asset_id: string;
}

export interface ContentCatalogCharacter {
  namespace: string;
  package_id: string;
  definition: {
    id: number;
    name_key: string;
    max_hp: number;
    initial_hp?: number;
    skills: number[];
    sex?: string;
    group?: string;
    is_lord?: boolean;
    tags?: string[];
    sort_key?: string;
  };
  portrait_asset: ContentAssetRef | null;
  legacy_skin_assets: ContentAssetRef[];
  skin_ids: number[];
}

export type ContentSkinPresentation =
  | { kind: "static"; asset: ContentAssetRef }
  | {
      kind: "spine";
      bundle: ContentAssetRef;
      actions: Record<string, string>;
      layout: Record<string, number>;
      dependencies: ContentAssetRef[];
    }
  | { kind: "substitute"; rule: string; asset: ContentAssetRef };

export interface ContentCatalogSkin {
  namespace: string;
  package_id: string;
  id: number;
  character_id: number;
  name_key: string;
  presentation: ContentSkinPresentation;
  voice_assets: Record<string, ContentAssetRef>;
}

export interface ContentCatalogAlias {
  namespace: string;
  package_id: string;
  character_id: number;
  source_character_id: number;
  voice_mappings: Record<string, string>;
}

export interface ContentCatalogPage {
  content_lock_hash: string;
  offset: number;
  limit: number;
  total: number;
  characters: ContentCatalogCharacter[];
  skins: ContentCatalogSkin[];
  aliases: ContentCatalogAlias[];
  translations: Record<string, Record<string, string>>;
}

export interface ContentBundle {
  content_lock: unknown;
  packages: string[];
}

/** NC05 选将候选的一项（`general_candidates_json` 同形）。 */
export interface GeneralCandidate {
  /** 目录内武将定义 id；候选主键，提交仍按 `name`。 */
  def: number;
  name: string;
  skills: string[];
}

/** NC05 选将候选面（worker `generalCandidates` 回包）。 */
export interface GeneralCandidatesPayload {
  count: number;
  candidates: GeneralCandidate[];
}

/** NC05 选将结果（`assign_general` 返回面）。 */
export interface GeneralAssignment {
  seat: number;
  name: string;
  /** 装配**不推进** revision：状态变化只能看 `state_hash`。 */
  revision: number;
  state_hash: string;
}

/** NC05 选将回包：装配结果 + 刷新后的信封（新技能集可见）。 */
export interface ChooseGeneralPayload {
  assignment: GeneralAssignment;
  envelope: EngineEnvelope;
}

export type WorkerRequest =
  | {
      id: number;
      type: "init";
      bundle?: ContentBundle;
      packsJson?: string;
      /** 表现层性能采样（`perfProbe.ts`）：开启 Worker 分段计时；缺省关闭。 */
      perf?: boolean;
      /**
       * PA07 缓解原型（`latency=sab` URL 门控）：回包 SAB 镜像缓冲。
       * 需跨源隔离（COOP/COEP）；缺省不传＝纯 postMessage。
       */
      latencySab?: SharedArrayBuffer;
    }
  | {
      id: number;
      type: "start";
      players: number;
      seed: number;
      observer: number;
      packsJson?: string;
      /** 对局模式：`"identity"` = 身份局（NC09 身份局端到端）；缺省原型模式。 */
      mode?: "identity";
    }
  | { id: number; type: "endPhase"; actor?: number }
  | { id: number; type: "playCard"; actor: number; card: number; targets: number[] }
  | {
      id: number;
      type: "activateSkill";
      actor: number;
      skill: number;
      targets: number[];
      cardTargets: number[];
    }
  | {
      id: number;
      type: "useSkillCard";
      actor: number;
      skill: number;
      cardKind: number;
      targets: number[];
      equipSlot: number;
    }
  | {
      id: number;
      type: "resolveChoice";
      actor: number;
      choiceId: number;
      selections: number[];
      cancelled: boolean;
    }
  | { id: number; type: "autoStep" }
  | { id: number; type: "observer" }
  // NC05 选将：候选来自内核目录；选将按名字装配到座位（未知武将/越界显式报错）。
  | { id: number; type: "generalCandidates" }
  | { id: number; type: "chooseGeneral"; seat: number; name: string }
  | { id: number; type: "save" }
  | { id: number; type: "restore"; hex: string; packsJson?: string }
  // GT-014 导入期深层校验（裁决第 12 项）：单个安装载荷交 Worker 内的 Rust
  // 校验器（包/模块 logic_hash、rule-vm 装载期 verify、宿主 ABI 窗口）裁决，
  // 只校验不建局；不需要已初始化的对局句柄。
  | { id: number; type: "validatePackage"; packJson: string }
  | { id: number; type: "loadReplay"; json: string }
  | {
      id: number;
      type: "seekReplay";
      step: number;
      privacy: ReplayPrivacy;
      player: number;
    }
  | { id: number; type: "status" };

export type WorkerResponse =
  | {
      id: number;
      ok: true;
      data:
        | EngineEnvelope
        | ObserverSnapshot
        | ReplayMetadata
        | ReplayFrame
        | StatusPayload
        | GeneralCandidatesPayload
        | ChooseGeneralPayload
        | string;
    }
  | { id: number; ok: false; error: string };

export interface StatusPayload {
  revision: number;
  finished: boolean;
  hash: string;
  choice: string;
}
