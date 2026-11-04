//! Observer projection types (server-side only full state → view).

use crate::ids::{CardId, PlayerId, SeatIndex, SkillId};
use crate::outcome::MatchOutcome;
use serde::{Deserialize, Serialize};

/// Who is observing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObserverKind {
    /// Seated player.
    Player(PlayerId),
    /// Delayed or live spectator (public info only by default).
    Spectator,
    /// Referee / full hidden info.
    Referee,
    /// Replay exporter role.
    ReplayExporter {
        /// Privacy level.
        privacy: ReplayPrivacy,
    },
}

/// Replay privacy level (architecture §11.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayPrivacy {
    /// Full hidden information.
    Full,
    /// One player's authorized information.
    Player(PlayerId),
    /// Public spectate rules.
    Public,
}

/// Snapshot of what one observer may see.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverSnapshot {
    /// Revision of the authority state this view is derived from.
    pub revision: u64,
    /// Canonical hash of authority state (not of the view).
    pub state_hash: [u8; 32],
    /// Observer role.
    pub observer: ObserverKind,
    /// Current round number (public turn state).
    pub round: u32,
    /// Current turn player, or `PlayerId::NONE` between turns.
    pub current_player: PlayerId,
    /// Current public phase.
    pub phase: crate::TurnPhaseTag,
    /// Seat order (public).
    pub seats: Vec<SeatView>,
    /// Public / authorized cards known to this observer.
    pub known_cards: Vec<CardView>,
    /// Match outcome if finished.
    pub outcome: MatchOutcome,
}

/// Per-seat public (or authorized) view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatView {
    /// Player id.
    pub player: PlayerId,
    /// Stable character definition id.
    #[serde(default)]
    pub character_def: u32,
    /// Seat index.
    pub seat: SeatIndex,
    /// Current HP (public in standard rules).
    pub hp: i32,
    /// Max HP.
    pub max_hp: u32,
    /// Alive flag.
    pub alive: bool,
    /// Whether the character card is currently face-down.
    #[serde(default)]
    pub turned_over: bool,
    /// Whether the character is currently linked (chain state).
    #[serde(default)]
    pub linked: bool,
    /// Hand count (not hand contents unless authorized).
    pub hand_count: u32,
    /// Known identity if revealed / authorized; otherwise unknown.
    pub identity: Option<IdentityTag>,
    /// Public skill instances owned by this player.
    pub skills: Vec<SkillView>,
    /// Ordered per-logical-skill R05 effect slots present on this seat.
    #[serde(default)]
    pub r05_effects: Vec<R05EffectView>,
    /// Authorized primary and secondary general slots; hidden/unconfigured slots are absent.
    #[serde(default)]
    pub generals: [Option<GeneralSlotView>; 2],
}

/// Public logical skill identity (`namespace`, `package_id`, `logical_skill_def_id`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct QualifiedSkillRef {
    /// Content namespace owning the package.
    pub namespace: String,
    /// Package identifier within the namespace.
    pub package_id: String,
    /// Stable logical skill definition id within the package.
    pub logical_skill_def_id: u32,
}

/// R05 four-role effect projection for a single logical skill identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct R05EffectView {
    /// Public logical skill identity.
    pub skill: QualifiedSkillRef,
    /// Tiqi hand-limit delta (`None` when slot absent; `Some(0)` or signed when present).
    #[serde(default)]
    pub tiqi_limit: Option<i32>,
    /// Baoshu comb count (`None` when slot absent; `Some(0)` or signed when present).
    #[serde(default)]
    pub baoshu_combs: Option<i32>,
    /// Dcbihuo pending next-turn count (`None` when slot absent; `Some(0)` or signed when present).
    #[serde(default)]
    pub dcbihuo_next: Option<i32>,
    /// Dcbihuo active current-turn count (`None` when slot absent; `Some(0)` or signed when present).
    #[serde(default)]
    pub dcbihuo_current: Option<i32>,
}

/// Observer-filtered general metadata; definition zero is a valid definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralSlotView {
    /// Stable general definition id.
    pub def: u32,
    /// Public general name or localization key.
    pub name: String,
    /// Public faction name.
    pub faction: String,
    /// Whether this slot has been revealed.
    pub revealed: bool,
}

/// Public skill metadata required for client-side activation controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillView {
    /// Runtime instance id used by `ActivateSkill`.
    pub id: SkillId,
    /// Stable skill definition id.
    pub def: u32,
    /// Whether the instance is enabled.
    pub enabled: bool,
    /// Uses in the current turn window.
    pub use_count: u32,
    /// Has an active handler.
    pub active: bool,
    /// Has a view-as handler requiring a card input.
    pub view_as: bool,
    /// Declared locked/passive skill.
    pub locked: bool,
}

/// Identity mode tags (public API).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityTag {
    /// Lord.
    Lord,
    /// Loyalist.
    Loyalist,
    /// Rebel.
    Rebel,
    /// Renegade.
    Renegade,
}

/// Card view for an observer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardView {
    /// Instance id (always stable).
    pub id: CardId,
    /// Definition id if face-up / owned / authorized; none if hidden face-down.
    pub def: Option<u32>,
    /// Suit if known.
    pub suit: Option<Suit>,
    /// Rank if known.
    pub rank: Option<u8>,
    /// Zone owner player if applicable.
    pub owner: Option<PlayerId>,
    /// Public zone containing the card.
    pub zone: CardZoneTag,
    /// Public rule category of the known card.
    pub kind: CardKindTag,
}

/// Public card-zone tag. This deliberately does not expose internal `ZoneId`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardZoneTag {
    /// Shared draw pile.
    DrawPile,
    /// Shared discard pile.
    DiscardPile,
    /// Player hand.
    Hand,
    /// Player equipment area.
    Equip,
    /// Player judgment area.
    Judge,
    /// Transient processing area.
    Processing,
    /// Removed from the match.
    Removed,
    /// Public skill expansion pile.
    ///
    /// Appended so existing serialized enum ordinals remain stable.
    Expansion,
}

/// Public rule category for a known card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardKindTag {
    /// 杀.
    Sha,
    /// 桃.
    Tao,
    /// 闪.
    Shan,
    /// 酒.
    Jiu,
    /// 火杀.
    ShaFire,
    /// 雷杀.
    ShaThunder,
    /// Weapon.
    Weapon,
    /// Armor.
    Armor,
    /// Offensive horse.
    HorseOff,
    /// Defensive horse.
    HorseDef,
    /// 无中生有.
    TrickWuzhong,
    /// 过河拆桥.
    TrickGuohe,
    /// 顺手牵羊.
    TrickShunshou,
    /// 决斗.
    TrickJuedou,
    /// 南蛮入侵.
    TrickNanman,
    /// 万箭齐发.
    TrickWanjian,
    /// 桃园结义.
    TrickTaoyuan,
    /// 五谷丰登.
    TrickWugu,
    /// 乐不思蜀.
    TrickLe,
    /// 兵粮寸断.
    TrickBingliang,
    /// 闪电.
    TrickShandian,
    /// 借刀杀人.
    TrickJiedao,
    /// 铁索连环.
    TrickTiesuo,
    /// 火攻.
    TrickHuogong,
    /// 无懈可击.
    TrickWuxie,
    /// Package-defined or otherwise uncategorized card.
    Other,
    /// Treasure equipment.
    Treasure,
    /// 远交近攻.
    TrickYuanjiao,
    /// 调剂盐梅.
    TrickTiaojiyanmei,
    /// 瞒天过海.
    TrickMantianguohai,
    /// 诱敌深入.
    TrickYoudishenru,
    /// 调兵遣将.
    TrickDiaobingqianjiang,
    /// 望梅止渴.
    TrickWangmeizhike,
    /// 以逸待劳（国战）.
    TrickYiyi,
    /// 水淹七军（国战）.
    TrickShuiyanqijunx,
    /// 戮力同心（国战）.
    TrickLulitongxin,
    /// 克复中原（国战）.
    TrickKefuzhongyuan,
    /// 固国安邦（国战）.
    TrickGuguoanbang,
    /// 火烧连营（国战）.
    TrickHuoshaolianying,
    /// 联军盛宴（国战）.
    TrickLianjunshengyan,
    /// 文和乱武（国战）.
    TrickWenheluanwu,
    /// 号令天下（国战）.
    TrickHaolingtianxia,
    /// 敕令（国战）.
    TrickChiling,
    /// 出其不意.
    TrickChuqibuyi,
}

/// Card suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    /// ♠
    Spade,
    /// ♥
    Heart,
    /// ♣
    Club,
    /// ♦
    Diamond,
    /// Virtual/cardless suit. Appended to preserve existing serialized ordinals.
    None,
}

/// Incremental view update (architecture: clients consume deltas, not full authority state).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverDelta {
    /// From revision (exclusive base).
    pub from_revision: u64,
    /// To revision (inclusive).
    pub to_revision: u64,
    /// Opaque ordered ops for the client view store (typed later).
    pub ops: Vec<ViewOp>,
}

/// Single view mutation op.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewOp {
    /// Replace seat public fields.
    SeatUpdated {
        /// Seat view.
        seat: SeatView,
    },
    /// Card became known / updated.
    CardUpdated {
        /// Card view.
        card: CardView,
    },
    /// Card left observer knowledge.
    CardHidden {
        /// Card id.
        id: CardId,
    },
    /// Outcome set.
    OutcomeSet {
        /// Outcome.
        outcome: MatchOutcome,
    },
    /// Full resync required.
    ResyncRequired,
}
