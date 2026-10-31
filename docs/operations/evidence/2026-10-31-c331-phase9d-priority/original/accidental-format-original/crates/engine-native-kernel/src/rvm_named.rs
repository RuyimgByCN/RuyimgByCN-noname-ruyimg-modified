//! T8 具名组合 filter：把 DSL 名称编译成可真可假的谓词程序。
//!
//! 未识别的技能标识成为存储键（标记 / once / 旗标），不会被吃成恒真或恒假。
//! 整段无法归入 `owner*` 域的名称返回 None，装载器继续跳过。

use crate::catalog::{CardKind, Faction, QueryCtx, Suit};
use crate::event::{DamageNature, Phase};
use crate::ids::PlayerId;
use crate::state::Role;
use crate::vocabulary::generals::Gender;
mod identity;

/// 触发帧上 filter 能看见的字段（由桥从 `EventData` 填）。
#[derive(Debug, Clone)]
pub struct FilterView {
    pub event_player: Option<PlayerId>,
    pub event_source: Option<PlayerId>,
    pub event_card: Option<u32>,
    pub event_amount: u32,
    pub phase: Option<Phase>,
    pub nature: Option<DamageNature>,
    pub targets: Vec<u8>,
    pub use_as: Option<CardKind>,
    pub from_chain: bool,
    pub batch_len: u32,
    pub moved_from: Option<PlayerId>,
    pub begin: Option<bool>,
    /// 1 伤害 2 用牌 3 阶段 4 弃牌 5 移牌 6 濒死 7 死亡 8 打出。
    pub kind: u8,
}

/// 一条可否定的原子。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    pub neg: bool,
    pub atom: Atom,
}

/// 状态谓词。每个变体能在真实局面上取到真和假。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    EvPlayer,
    EvActor,
    OwnerAlive,
    InTargets,
    UserOther,
    Wounded,
    Healthy,
    Linked,
    Unlinked,
    FaceDown,
    FaceUp,
    HandEmpty,
    HandNonEmpty,
    HandBelowMax,
    HandAboveHp,
    HandAtLeastHp,
    HandAtLeast(u8),
    HpBelow(u8),
    HpEq(u8),
    JudgeNonEmpty,
    EquipNonEmpty,
    EquipEmpty,
    HandOrEquip,
    HandOrEquipEmpty,
    OtherAlive,
    OtherHand,
    OtherWounded,
    CardSha,
    CardTrick,
    CardBasic,
    CardEquipKind,
    CardJuedou,
    CardRed,
    CardBlack,
    CardSpade,
    CardHeart,
    CardClub,
    CardDiamond,
    CardJiu,
    CardShan,
    CardNanman,
    CardWanjian,
    CardWuzhong,
    CardDelayed,
    Fire,
    Thunder,
    Damage,
    PhasePlay,
    PhaseDraw,
    PhaseDiscard,
    PhaseJudge,
    PhaseEnd,
    PhaseBegin,
    TurnDmg0,
    TurnDmg,
    TurnDmgAtLeast(u8),
    BtePos,
    SourceAlive,
    DistLe1,
    TiedHand,
    TiedHp,
    TiedEquip,
    NotUniqueHp,
    Chain,
    Direct,
    Dying,
    Death,
    DiscardEvt,
    Moved,
    Responded,
    Used,
    OutsideTurn,
    InTurn,
    Lost,
    LostAtLeast(u8),
    Batch,
    PileNonEmpty,
    PileEmpty,
    MarkPos,
    DrawNotFixed,
    Once { scope: u8 },
    FlagSet(String),
    FlagUnset(String),
    Tag(String),
    FirstHit,
    SecondHit,
    AfterFirst,
    OddHand,
    Lord,
    Female,
    Male,
    SexSame,
    GroupSame,
    Transformed,
    MinHp,
    TargetAlive,
    TargetHandNonEmpty,
    TargetHandEmpty,
    CanDiscard(u8),
    CountAtLeast(u8),
    SingleTarget,
    MultiTarget,
    MovedFromOwner,
    Gained,
    FactionIs(u8),
    ShaOrTrick,
    ShaOrJuedou,
    BasicOrTrick,
    ElementalOrTrick,
    RedOrJiu,
    JuedouOrRedSha,
}

/// OR 组，组内 AND。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedProgram {
    pub groups: Vec<Vec<Clause>>,
    /// Halted 时写入 once 的 (键, 范围)。空键表示用技能名。
    pub consume: Vec<(String, u8)>,
    pub count_hits: bool,
}

#[derive(Clone)]
enum Seed {
    Skip,
    Not,
    Avail,
    Pending,
    Below,
    Num(u8),
    AtLeast(u8),
    Once(u8),
    Atom(Atom),
}

/// 词表里不以 owner 开头、也不是 `always` 的名称。其余非 owner 名返回 None。
fn vocabulary_extra(name: &str) -> bool {
    matches!(
        name,
        "debate_owner_is_initiator"
            | "debate_owner_participated"
            | "debate_owner_opinion_differs_initiator"
            | "debate_owner_is_initiator_result_red"
            | "debate_owner_is_initiator_result_black"
            | "cards_moved_into_expansion"
            | "cards_moved_out_of_expansion"
            | "cilv_tracked_card_damage_before"
            | "cilv_tracked_card_resolved"
            | "dcsbyingmou_tracked_card_resolved"
            | "dcsbmengmou_tracked_sha_damage"
            | "dcsbmengmou_tracked_sha_resolved"
            | "dcyinjun_tracked_sha_resolved"
            | "hand_empty"
            | "from_hand"
            | "from_equip"
            | "from_hand_empty"
            | "from_field_empty"
            | "card_is_trick"
            | "once_per_turn"
            | "dddchashi_matching_card"
            | "dddchashi_pending_phase_play_end"
            | "ddd_xuyu_pending"
            | "ddd_shijian_watched_source_card_used_targeted"
            | "ddd_shijian_pending"
            | "cards_moved_discard_at_least_three"
    )
}

/// 编译具名 filter。`always` 与词表外的非 owner 名返回 None。
pub fn compile_named(name: &str) -> Option<NamedProgram> {
    if name == "always" || name.is_empty() {
        return None;
    }
    if !name.starts_with("owner") && !vocabulary_extra(name) {
        return None;
    }
    let (domains, rest) = split_domain(name);
    let rest_parts = split_top_or(&protect(&rest));
    let mut groups = Vec::new();
    for domain in &domains {
        for part in &rest_parts {
            let mut clauses = domain.clone();
            clauses.extend(parse_words(part));
            if clauses.is_empty() {
                return None;
            }
            groups.push(clauses);
        }
    }
    if groups.is_empty() {
        return None;
    }
    // owner_is_target_*：每个 OR 组都含「事件主玩家=持有者」。
    if name.starts_with("owner_is_target_") {
        for g in &mut groups {
            if !g.iter().any(|c| !c.neg && c.atom == Atom::EvPlayer) {
                g.insert(
                    0,
                    Clause {
                        neg: false,
                        atom: Atom::EvPlayer,
                    },
                );
            }
        }
    }
    let mut consume = Vec::new();
    let mut count_hits = false;
    for g in &groups {
        for c in g {
            if c.neg {
                continue;
            }
            match &c.atom {
                Atom::Once { scope } => consume.push((String::new(), *scope)),
                Atom::FlagUnset(k) => consume.push((k.clone(), 1)),
                Atom::FirstHit | Atom::SecondHit | Atom::AfterFirst => count_hits = true,
                _ => {}
            }
        }
    }
    consume.sort();
    consume.dedup();
    Some(NamedProgram {
        groups,
        consume,
        count_hits,
    })
}

pub fn eval_program(
    prog: &NamedProgram,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    skill: &str,
    view: &FilterView,
) -> bool {
    prog.groups.iter().any(|g| {
        g.iter().all(|c| {
            let v = eval_atom(&c.atom, ctx, owner, skill, view);
            if c.neg {
                !v
            } else {
                v
            }
        })
    })
}

fn split_domain(name: &str) -> (Vec<Vec<Clause>>, String) {
    const TABLE: &[(&str, &[Atom])] = &[
        (
            "owner_targeted_by_other_",
            &[Atom::InTargets, Atom::UserOther, Atom::OwnerAlive],
        ),
        (
            "owner_targeted_outside_turn_",
            &[Atom::InTargets, Atom::OutsideTurn, Atom::OwnerAlive],
        ),
        ("owner_targeted_by_", &[Atom::InTargets, Atom::OwnerAlive]),
        ("owner_is_target_", &[Atom::EvPlayer]),
        ("owner_is_source_", &[Atom::EvActor]),
        ("owner_source_", &[Atom::EvActor]),
        ("owner_alive_", &[Atom::OwnerAlive]),
        ("owner_from_", &[Atom::MovedFromOwner]),
        ("owner_gained_", &[Atom::Gained]),
        ("owner_discarded_", &[Atom::DiscardEvt]),
        ("owner_used_", &[Atom::Used]),
        ("owner_phase_", &[Atom::EvPlayer]),
        ("owner_tracked_", &[Atom::OwnerAlive]),
        ("owner_initiated_", &[Atom::EvActor]),
        ("owner_sha_", &[Atom::CardSha, Atom::EvActor]),
        ("owner_", &[Atom::OwnerAlive]),
    ];
    for (pre, atoms) in TABLE {
        if let Some(rest) = name.strip_prefix(pre) {
            if rest.starts_with("or_") {
                continue;
            }
            // source_or_target 是析取，不走单一前缀。
            if pre.ends_with("source_")
                && (rest.starts_with("or_target")
                    || rest.starts_with("or_targeted")
                    || rest.starts_with("or_within"))
            {
                continue;
            }
            let clauses = atoms
                .iter()
                .cloned()
                .map(|atom| Clause { neg: false, atom })
                .collect();
            return (vec![clauses], rest.to_string());
        }
    }
    if let Some(rest) = name.strip_prefix("owner_source_or_target_") {
        return (
            vec![
                vec![Clause {
                    neg: false,
                    atom: Atom::EvActor,
                }],
                vec![Clause {
                    neg: false,
                    atom: Atom::EvPlayer,
                }],
            ],
            rest.to_string(),
        );
    }
    if let Some(rest) = name.strip_prefix("owner_is_source_or_target_") {
        return (
            vec![
                vec![Clause {
                    neg: false,
                    atom: Atom::EvActor,
                }],
                vec![Clause {
                    neg: false,
                    atom: Atom::EvPlayer,
                }],
            ],
            rest.to_string(),
        );
    }
    if let Some(rest) = name.strip_prefix("owner_source_or_targeted_by_") {
        return (
            vec![
                vec![Clause {
                    neg: false,
                    atom: Atom::EvActor,
                }],
                vec![
                    Clause {
                        neg: false,
                        atom: Atom::InTargets,
                    },
                    Clause {
                        neg: false,
                        atom: Atom::UserOther,
                    },
                ],
            ],
            rest.to_string(),
        );
    }
    if let Some(rest) = name.strip_prefix("owner_source_or_within_") {
        return (
            vec![
                vec![Clause {
                    neg: false,
                    atom: Atom::EvActor,
                }],
                vec![
                    Clause {
                        neg: false,
                        atom: Atom::DistLe1,
                    },
                    Clause {
                        neg: false,
                        atom: Atom::OwnerAlive,
                    },
                ],
            ],
            rest.to_string(),
        );
    }
    if let Some(rest) = name.strip_prefix("owner_gained_or_lost_") {
        return (
            vec![
                vec![Clause {
                    neg: false,
                    atom: Atom::Gained,
                }],
                vec![Clause {
                    neg: false,
                    atom: Atom::Lost,
                }],
            ],
            rest.to_string(),
        );
    }
    (
        vec![vec![Clause {
            neg: false,
            atom: Atom::OwnerAlive,
        }]],
        name.trim_start_matches("owner_").to_string(),
    )
}

fn protect(rest: &str) -> String {
    const COMPOUNDS: &[&str] = &[
        "hand_or_equip",
        "armor_or_mount",
        "elemental_or_trick",
        "sha_or_ordinary_trick",
        "sha_or_juedou",
        "sha_or_trick",
        "sha_or_black",
        "sha_or_duel",
        "red_or_jiu",
        "basic_or_trick",
        "virtual_or_converted",
        "single_or_discardable",
        "wounded_or_guiming",
        "dead_or_damaged",
        "used_or_responded",
        "card_used_or_responded",
        "hp_one_or_hand_empty",
        "self_or_in_range",
        "owner_or_female",
        "owner_or_min",
        "owner_or_during",
        "current_or_turn",
        "dealt_or_received",
        "suit_or_rank",
        "face_up_or_linked",
        "juedou_or_red",
        "became_face_up_or_linked",
    ];
    let mut s = rest.to_string();
    for c in COMPOUNDS {
        if s.contains(c) {
            s = s.replace(c, &c.replace("_or_", "OR"));
        }
    }
    s
}

fn split_top_or(s: &str) -> Vec<String> {
    if !s.contains("_or_") {
        return vec![s.to_string()];
    }
    s.split("_or_")
        .map(|p| p.to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

fn parse_words(rest: &str) -> Vec<Clause> {
    let words: Vec<&str> = rest.split('_').filter(|w| !w.is_empty()).collect();
    let mut seeds = Vec::new();
    let mut i = 0;
    while i < words.len() {
        if let Some((n, s)) = take_known(&words[i..]) {
            seeds.extend(s);
            i += n;
        } else {
            let start = i;
            i += 1;
            while i < words.len() && take_known(&words[i..]).is_none() {
                i += 1;
            }
            seeds.push(Seed::Atom(Atom::Tag(words[start..i].join("_"))));
        }
    }
    finish(seeds)
}

fn take_known(words: &[&str]) -> Option<(usize, Vec<Seed>)> {
    let max = words.len().min(6);
    for n in (2..=max).rev() {
        let key = words[..n].join("_");
        if let Some(s) = phrase(&key) {
            return Some((n, s));
        }
    }
    word(words.first()?).map(|s| (1, s))
}

fn atom(a: Atom) -> Vec<Seed> {
    vec![Seed::Atom(a)]
}

fn phrase(key: &str) -> Option<Vec<Seed>> {
    Some(match key {
        "once_per_turn" | "per_turn" => vec![Seed::Once(1)],
        "once_per_round" | "per_round" => vec![Seed::Once(2)],
        "once_per_game" | "per_game" => vec![Seed::Once(3)],
        "twice_per_turn" => vec![Seed::Once(1), Seed::AtLeast(1)],
        "hand_nonempty" | "has_hand" => atom(Atom::HandNonEmpty),
        "hand_empty" => atom(Atom::HandEmpty),
        "hand_below_max_hp" | "below_max_hp" => atom(Atom::HandBelowMax),
        "hand_above_hp" => atom(Atom::HandAboveHp),
        "hand_at_least_hp" | "at_least_hp" => atom(Atom::HandAtLeastHp),
        "skill_pile_nonempty" | "originating_skill_pile" | "skill_pile" => atom(Atom::PileNonEmpty),
        "skill_pile_empty" | "piles_empty" => atom(Atom::PileEmpty),
        "play_phase" | "phase_play" => atom(Atom::PhasePlay),
        "phase_end" => atom(Atom::PhaseEnd),
        "phase_begin" | "phase_any" => atom(Atom::PhaseBegin),
        "phase_draw" | "draw_phase" => atom(Atom::PhaseDraw),
        "phase_discard" | "discard_phase" => atom(Atom::PhaseDiscard),
        "phase_judge" | "judge_phase" => atom(Atom::PhaseJudge),
        "turn_damage" => atom(Atom::TurnDmg),
        "no_turn_damage" => atom(Atom::TurnDmg0),
        "turn_damage_at_least_three" => atom(Atom::TurnDmgAtLeast(3)),
        "at_least_two" | "at_least_2" => vec![Seed::AtLeast(2)],
        "at_least_three" | "at_least_3" => vec![Seed::AtLeast(3)],
        "at_least_one" => vec![Seed::AtLeast(1)],
        "hp_below" => vec![Seed::Below],
        "hp_one" | "hp_eq_one" => atom(Atom::HpEq(1)),
        "hp_gt_one" | "max_hp_gt_one" => atom(Atom::HpBelow(255)), // rewritten: not hp==1, use Healthy inverse via Count
        "face_down" => atom(Atom::FaceDown),
        "face_up" => atom(Atom::FaceUp),
        "other_alive" => atom(Atom::OtherAlive),
        "other_hand" | "other_hand_nonempty" => atom(Atom::OtherHand),
        "other_wounded" => atom(Atom::OtherWounded),
        "source_alive" => atom(Atom::SourceAlive),
        "tied_max_hand" => atom(Atom::TiedHand),
        "tied_max_hp" => atom(Atom::TiedHp),
        "tied_max_equip" | "tied_max_equip_nonempty" => atom(Atom::TiedEquip),
        "not_unique" | "not_unique_max" | "not_unique_max_current_hp" => atom(Atom::NotUniqueHp),
        "judge_nonempty" => atom(Atom::JudgeNonEmpty),
        "equip_nonempty" | "equip_card" => atom(Atom::EquipNonEmpty),
        "equip_empty" | "empty_equip" | "empty_equip_slot" => atom(Atom::EquipEmpty),
        "chain_damage" | "from_chain" => atom(Atom::Chain),
        "direct_damage" | "direct_sha" => vec![Seed::Atom(Atom::Direct), Seed::Atom(Atom::CardSha)],
        "draw_not_fixed" | "not_fixed" => atom(Atom::DrawNotFixed),
        "lost_cards" | "lost_hand" | "lost_this_turn" => atom(Atom::Lost),
        "discard_batch" => atom(Atom::Batch),
        "this_turn" | "this_round" | "this_phase" => vec![Seed::Skip],
        "event_player" => atom(Atom::EvPlayer),
        "card_used" | "cards_used" => atom(Atom::Used),
        "first_damage" | "first_sha" => atom(Atom::FirstHit),
        "second_damage" => atom(Atom::SecondHit),
        "after_first" | "after_first_damage" => atom(Atom::AfterFirst),
        "same_sex" => atom(Atom::SexSame),
        "same_group" => atom(Atom::GroupSame),
        "attack_range" => atom(Atom::DistLe1),
        "distance_le_one" | "distance_at_most_one" | "within_one" | "at_most_one" => {
            atom(Atom::DistLe1)
        }
        "ordinary_trick" => atom(Atom::CardTrick),
        "playable" => atom(Atom::Used),
        "category_count_positive" | "count_positive" => atom(Atom::BtePos),
        "single_target" => atom(Atom::SingleTarget),
        "outside_turn" => atom(Atom::OutsideTurn),
        "in_range" => atom(Atom::DistLe1),
        "hand_equip" => atom(Atom::HandOrEquip),
        "full_health" => atom(Atom::Healthy),
        "not_replaced" => atom(Atom::DrawNotFixed),
        _ => return None,
    })
}

fn word(w: &str) -> Option<Vec<Seed>> {
    Some(match w {
        "is" | "the" | "a" | "an" | "by" | "to" | "from" | "in" | "of" | "per" | "this" | "and"
        | "with" | "has" | "its" | "own" | "on" | "for" | "as" | "at" | "be" | "was" | "into"
        | "out" | "than" | "then" | "any" | "all" | "can" | "current" | "event" | "player"
        | "skill" | "cards" | "card" | "count" | "during" | "when" | "that" | "which" | "their"
        | "his" | "her" | "each" | "both" | "only" | "also" | "now" | "still" | "already"
        | "just" | "more" | "most" | "least" | "thanx" | "positive" | "exists" | "global"
        | "living" | "point" | "points" | "amount" | "result" | "state" | "end" | "begin"
        | "ended" | "using" | "usedx" => vec![Seed::Skip],
        "not" | "no" | "without" | "non" => vec![Seed::Not],
        "available" | "unused" | "untriggered" | "ready" => vec![Seed::Avail],
        "pending" | "assigned" | "armed" | "triggered" => vec![Seed::Pending],
        "below" | "under" => vec![Seed::Below],
        "one" | "1" => vec![Seed::Num(1)],
        "two" | "2" => vec![Seed::Num(2)],
        "three" | "3" => vec![Seed::Num(3)],
        "four" | "4" => vec![Seed::Num(4)],
        "seven" | "7" => vec![Seed::Num(7)],
        "eight" | "8" => vec![Seed::Num(8)],
        "nine" | "9" => vec![Seed::Num(9)],
        "zero" | "0" => vec![Seed::Num(0)],
        "once" => vec![Seed::Once(1)],
        "wounded" => atom(Atom::Wounded),
        "healthy" | "undamaged" | "unharmed" => atom(Atom::Healthy),
        "linked" => atom(Atom::Linked),
        "unlinked" => atom(Atom::Unlinked),
        "sha" => atom(Atom::CardSha),
        "trick" => atom(Atom::CardTrick),
        "juedou" | "duel" => atom(Atom::CardJuedou),
        "basic" => atom(Atom::CardBasic),
        "equip" | "equipment" | "weapon" | "armor" | "mount" => atom(Atom::EquipNonEmpty),
        "hand" => atom(Atom::HandNonEmpty),
        "empty" => atom(Atom::HandEmpty),
        "nonempty" => atom(Atom::HandNonEmpty),
        "alive" => atom(Atom::OwnerAlive),
        "dead" | "killed" => atom(Atom::Death),
        "damage" | "damaged" | "dealt" => atom(Atom::Damage),
        "phase" => atom(Atom::PhaseEnd),
        "play" => atom(Atom::PhasePlay),
        "draw" | "draws" | "drawn" => atom(Atom::PhaseDraw),
        "discard" | "discarded" | "discardable" => atom(Atom::DiscardEvt),
        "lost" | "loss" => atom(Atom::Lost),
        "before" => atom(Atom::Damage),
        "fire" => atom(Atom::Fire),
        "thunder" => atom(Atom::Thunder),
        "red" => atom(Atom::CardRed),
        "black" | "nonred" => atom(Atom::CardBlack),
        "spade" => atom(Atom::CardSpade),
        "heart" => atom(Atom::CardHeart),
        "club" => atom(Atom::CardClub),
        "diamond" => atom(Atom::CardDiamond),
        "turn" => atom(Atom::InTurn),
        "round" => vec![Seed::Skip],
        "other" => atom(Atom::OtherAlive),
        "source" => atom(Atom::SourceAlive),
        "target" | "targets" | "targeted" => atom(Atom::InTargets),
        "first" => atom(Atom::FirstHit),
        "second" => atom(Atom::SecondHit),
        "outside" => atom(Atom::OutsideTurn),
        "dying" => atom(Atom::Dying),
        "death" => atom(Atom::Death),
        "judge" => atom(Atom::JudgeNonEmpty),
        "mark" | "marked" | "marks" => atom(Atom::MarkPos),
        "pile" | "piles" => atom(Atom::PileNonEmpty),
        "batch" | "moved" | "move" => atom(Atom::Moved),
        "responded" | "respond" | "response" => atom(Atom::Responded),
        "used" | "use" => atom(Atom::Used),
        "gained" | "gain" | "gains" | "gainable" => atom(Atom::Gained),
        "lord" => atom(Atom::Lord),
        "female" => atom(Atom::Female),
        "male" => atom(Atom::Male),
        "linkedx" => atom(Atom::Linked),
        "face" => atom(Atom::FaceDown),
        "down" => atom(Atom::FaceDown),
        "up" => atom(Atom::FaceUp),
        "chain" => atom(Atom::Chain),
        "direct" => atom(Atom::Direct),
        "transformed" | "virtual" | "converted" => atom(Atom::Transformed),
        "odd" => atom(Atom::OddHand),
        "even" => vec![Seed::Not, Seed::Atom(Atom::OddHand)],
        "min" => atom(Atom::MinHp),
        "max" | "unique" | "tied" | "above" | "gte" | "lte" | "gt" | "eq" | "equals" => {
            vec![Seed::Skip]
        }
        "shaORjuedou" | "shaORduel" => atom(Atom::ShaOrJuedou),
        "shaORtrick" | "shaORordinary" | "shaORblack" => atom(Atom::ShaOrTrick),
        "handORequip" | "armorORmount" => atom(Atom::HandOrEquip),
        "elementalORtrick" => atom(Atom::ElementalOrTrick),
        "redORjiu" => atom(Atom::RedOrJiu),
        "basicORtrick" => atom(Atom::BasicOrTrick),
        "juedouORred" => atom(Atom::JuedouOrRedSha),
        "virtualORconverted" => atom(Atom::Transformed),
        "hpORhand" => vec![Seed::Atom(Atom::HpEq(1)), Seed::Atom(Atom::HandEmpty)],
        "selfORin" => atom(Atom::DistLe1),
        "ownerORfemale" => atom(Atom::Female),
        "dealtORreceived" => atom(Atom::Damage),
        "suitORrank" => atom(Atom::CardRed),
        "faceORlinked" | "becameORface" => atom(Atom::Linked),
        "woundedORguiming" => atom(Atom::OtherWounded),
        "deadORdamaged" => atom(Atom::Damage),
        "usedORresponded" | "cardORresponded" => atom(Atom::Used),
        "singleORdiscardable" => atom(Atom::CanDiscard(1)),
        "currentORturn" => atom(Atom::InTurn),
        "ownerORmin" | "ownerORduring" => atom(Atom::MinHp),
        "jiu" => atom(Atom::CardJiu),
        "shan" => atom(Atom::CardShan),
        "nanman" => atom(Atom::CardNanman),
        "wanjian" => atom(Atom::CardWanjian),
        "wuzhong" => atom(Atom::CardWuzhong),
        "wuxie" | "nullify" | "huogong" => atom(Atom::CardTrick),
        "delayed" => atom(Atom::CardDelayed),
        "ordinary" => atom(Atom::CardTrick),
        "elemental" => atom(Atom::ElementalOrTrick),
        "wu" => atom(Atom::FactionIs(2)),
        "qun" => atom(Atom::FactionIs(3)),
        "wei" => atom(Atom::FactionIs(0)),
        "shu" => atom(Atom::FactionIs(1)),
        "group" | "sex" | "same" => atom(Atom::SexSame),
        "distance" | "range" => atom(Atom::DistLe1),
        "single" => atom(Atom::SingleTarget),
        "multi" | "multitarget" => atom(Atom::MultiTarget),
        "fixed" => atom(Atom::DrawNotFixed),
        "skipped" | "skip" => atom(Atom::PhaseEnd),
        "recover" | "recovered" | "recovery" | "recoverable" => atom(Atom::Wounded),
        "changed" | "change" => atom(Atom::Damage),
        "compare" | "differs" | "differ" => atom(Atom::HandNonEmpty),
        "numeric" | "rank" | "suit" | "color" | "name" => atom(Atom::Used),
        "physical" | "original" | "selected" | "recorded" | "tracked" | "tracking" => {
            vec![Seed::Skip]
        }
        "unprompted" | "active" | "enabled" | "disabled" => vec![Seed::Avail],
        "condition" | "unusedx" => vec![Seed::Avail],
        _ => return None,
    })
}

fn finish(seeds: Vec<Seed>) -> Vec<Clause> {
    let mut out: Vec<Clause> = Vec::new();
    let mut neg = false;
    let mut i = 0;
    while i < seeds.len() {
        match &seeds[i] {
            Seed::Skip => {}
            Seed::Not => neg = !neg,
            Seed::Avail => {
                if let Some(Clause {
                    atom: Atom::Tag(k), ..
                }) = out.last()
                {
                    let k = k.clone();
                    out.pop();
                    push(&mut out, &mut neg, Atom::FlagUnset(k));
                } else {
                    push(&mut out, &mut neg, Atom::Once { scope: 1 });
                }
            }
            Seed::Pending => {
                if let Some(Clause {
                    atom: Atom::Tag(k), ..
                }) = out.last()
                {
                    let k = k.clone();
                    out.pop();
                    push(&mut out, &mut neg, Atom::FlagSet(k));
                } else {
                    push(&mut out, &mut neg, Atom::FlagSet("pending".into()));
                }
            }
            Seed::Once(scope) => push(&mut out, &mut neg, Atom::Once { scope: *scope }),
            Seed::Below => {
                let n = if let Some(Seed::Num(n)) = seeds.get(i + 1) {
                    i += 1;
                    *n
                } else {
                    1
                };
                push(&mut out, &mut neg, Atom::HpBelow(n));
            }
            Seed::Num(n) => push(&mut out, &mut neg, Atom::CountAtLeast(*n)),
            Seed::AtLeast(n) => {
                let n = *n;
                match out.last().map(|c| &c.atom) {
                    Some(Atom::Lost) => {
                        out.pop();
                        push(&mut out, &mut neg, Atom::LostAtLeast(n));
                    }
                    Some(Atom::TurnDmg) => {
                        out.pop();
                        push(&mut out, &mut neg, Atom::TurnDmgAtLeast(n));
                    }
                    Some(Atom::HandNonEmpty) => {
                        out.pop();
                        push(&mut out, &mut neg, Atom::HandAtLeast(n));
                    }
                    _ => push(&mut out, &mut neg, Atom::CountAtLeast(n)),
                }
            }
            Seed::Atom(a) => push(&mut out, &mut neg, a.clone()),
        }
        i += 1;
    }
    out
}

fn push(out: &mut Vec<Clause>, neg: &mut bool, atom: Atom) {
    out.push(Clause { neg: *neg, atom });
    *neg = false;
}

fn phase_is(ctx: &QueryCtx<'_>, v: &FilterView, phase: Phase) -> bool {
    match v.phase {
        Some(p) => p == phase,
        None => ctx.state.turn.phase == phase,
    }
}

fn eval_atom(
    atom: &Atom,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    skill: &str,
    v: &FilterView,
) -> bool {
    let me = ctx.state.player(owner);
    let alive = me.alive;
    let wounded = me.hp > 0 && (me.hp as u32) < me.max_hp;
    let card_kind = v
        .event_card
        .and_then(|c| ctx.catalog.cards.get(c as usize).map(|d| d.kind));
    let suit = v
        .event_card
        .and_then(|c| ctx.catalog.cards.get(c as usize).map(|d| d.suit));
    let logical = v.use_as.or(card_kind);
    let sha = logical.is_some_and(|k| k.is_sha_like());
    let red = suit.is_some_and(|s| matches!(s, Suit::Heart | Suit::Diamond));
    let actor = v.event_source.or(v.event_player);
    match atom {
        Atom::EvPlayer => v.event_player == Some(owner),
        Atom::EvActor => actor == Some(owner),
        Atom::OwnerAlive => alive,
        Atom::InTargets => v.targets.contains(&owner) || v.event_player == Some(owner),
        Atom::UserOther => {
            let user = if v.kind == 1 {
                v.event_source
            } else {
                v.event_player
            };
            user.is_some_and(|u| u != owner)
        }
        Atom::Wounded => wounded,
        Atom::Healthy => alive && !wounded && me.hp > 0,
        Atom::Linked => me.linked,
        Atom::Unlinked => !me.linked,
        Atom::FaceDown => me.face_down,
        Atom::FaceUp => !me.face_down,
        Atom::HandEmpty => me.hand.is_empty(),
        Atom::HandNonEmpty => !me.hand.is_empty(),
        Atom::HandBelowMax => me.hand.len() < me.max_hp as usize,
        Atom::HandAboveHp => me.hand.len() > me.hp.max(0) as usize,
        Atom::HandAtLeastHp => me.hand.len() >= me.hp.max(0) as usize,
        Atom::HandAtLeast(n) => me.hand.len() >= *n as usize,
        Atom::HpBelow(n) => {
            if *n == 255 {
                me.max_hp > 1
            } else {
                me.hp > 0 && me.hp < *n as i32
            }
        }
        Atom::HpEq(n) => me.hp == *n as i32,
        Atom::JudgeNonEmpty => !me.judge.is_empty(),
        Atom::EquipNonEmpty => !me.equips.is_empty(),
        Atom::EquipEmpty => me.equips.is_empty(),
        Atom::HandOrEquip => !me.hand.is_empty() || !me.equips.is_empty(),
        Atom::HandOrEquipEmpty => me.hand.is_empty() && me.equips.is_empty(),
        Atom::OtherAlive => {
            (0..ctx.state.players.len()).any(|i| i as u8 != owner && ctx.state.players[i].alive)
        }
        Atom::OtherHand => (0..ctx.state.players.len()).any(|i| {
            i as u8 != owner && ctx.state.players[i].alive && !ctx.state.players[i].hand.is_empty()
        }),
        Atom::OtherWounded => (0..ctx.state.players.len()).any(|i| {
            let p = &ctx.state.players[i];
            p.alive && p.hp > 0 && (p.hp as u32) < p.max_hp
        }),
        Atom::CardSha => sha,
        Atom::CardTrick => logical.is_some_and(kind_is_ordinary_trick),
        Atom::CardBasic => logical.is_some_and(|k| k.is_basic_kind()),
        Atom::CardEquipKind => logical.is_some_and(|k| k.equip_slot().is_some()),
        Atom::CardJuedou => logical.is_some_and(|k| matches!(k, CardKind::Duel)),
        Atom::CardRed => red,
        Atom::CardBlack => suit.is_some_and(|s| matches!(s, Suit::Spade | Suit::Club)),
        Atom::CardSpade => suit.is_some_and(|s| s == Suit::Spade),
        Atom::CardHeart => suit.is_some_and(|s| s == Suit::Heart),
        Atom::CardClub => suit.is_some_and(|s| s == Suit::Club),
        Atom::CardDiamond => suit.is_some_and(|s| s == Suit::Diamond),
        Atom::CardJiu => logical.is_some_and(|k| matches!(k, CardKind::Jiu)),
        Atom::CardShan => logical.is_some_and(|k| matches!(k, CardKind::Shan)),
        Atom::CardNanman => logical.is_some_and(|k| matches!(k, CardKind::SavageAttack)),
        Atom::CardWanjian => logical.is_some_and(|k| matches!(k, CardKind::ArcheryAttack)),
        Atom::CardWuzhong => logical.is_some_and(|k| matches!(k, CardKind::Wuzhong)),
        Atom::CardDelayed => {
            logical.is_some_and(|k| matches!(k, CardKind::Le | CardKind::Bing | CardKind::ShanDian))
        }
        Atom::ShaOrTrick => sha || logical.is_some_and(kind_is_ordinary_trick),
        Atom::ShaOrJuedou => sha || logical.is_some_and(|k| matches!(k, CardKind::Duel)),
        Atom::BasicOrTrick => {
            logical.is_some_and(|k| k.is_basic_kind() || kind_is_ordinary_trick(k))
        }
        Atom::ElementalOrTrick => {
            matches!(v.nature, Some(DamageNature::Fire | DamageNature::Thunder))
                || logical.is_some_and(kind_is_ordinary_trick)
        }
        Atom::RedOrJiu => red || logical.is_some_and(|k| matches!(k, CardKind::Jiu)),
        Atom::JuedouOrRedSha => {
            logical.is_some_and(|k| matches!(k, CardKind::Duel)) || (sha && red)
        }
        Atom::Fire => v.nature == Some(DamageNature::Fire),
        Atom::Thunder => v.nature == Some(DamageNature::Thunder),
        Atom::Damage => v.kind == 1 || v.nature.is_some() || v.event_amount > 0,
        // 阶段帧带自己的阶段。其他帧（伤害、用牌）不带阶段，读当前回合阶段。
        Atom::PhasePlay => phase_is(ctx, v, Phase::Play),
        Atom::PhaseDraw => phase_is(ctx, v, Phase::Draw),
        Atom::PhaseDiscard => phase_is(ctx, v, Phase::Discard),
        Atom::PhaseJudge => phase_is(ctx, v, Phase::Judge),
        Atom::PhaseEnd => v.kind == 3 && v.begin == Some(false),
        Atom::PhaseBegin => v.kind == 3 && v.begin == Some(true),
        Atom::TurnDmg0 => me.turn_damage == 0,
        Atom::TurnDmg => me.turn_damage > 0,
        Atom::TurnDmgAtLeast(n) => me.turn_damage >= *n,
        Atom::BtePos => ctx.state.turn.bte_used_play_phase > 0,
        Atom::SourceAlive => v
            .event_source
            .is_some_and(|s| ctx.state.players.get(s as usize).is_some_and(|p| p.alive)),
        Atom::DistLe1 => {
            let other = v.event_player.filter(|p| *p != owner).or(v.event_source);
            other.is_some_and(|o| ctx.state.computed_distance(ctx.catalog, owner, o) <= 1)
        }
        Atom::TiedHand => {
            let n = me.hand.len();
            n > 0
                && ctx
                    .state
                    .players
                    .iter()
                    .all(|p| !p.alive || p.hand.len() <= n)
        }
        Atom::TiedHp => me.hp > 0 && ctx.state.players.iter().all(|p| !p.alive || p.hp <= me.hp),
        Atom::TiedEquip => {
            let n = me.equips.len();
            n > 0
                && ctx
                    .state
                    .players
                    .iter()
                    .all(|p| !p.alive || p.equips.len() <= n)
        }
        Atom::NotUniqueHp => ctx
            .state
            .players
            .iter()
            .enumerate()
            .any(|(i, p)| p.alive && i as u8 != owner && p.hp >= me.hp),
        Atom::Chain => v.from_chain,
        Atom::Direct => v.kind == 1 && !v.from_chain,
        Atom::Dying => v.kind == 6,
        Atom::Death => v.kind == 7,
        Atom::DiscardEvt => v.kind == 4 || v.batch_len > 0,
        Atom::Moved => v.kind == 5,
        Atom::Responded => v.kind == 8,
        Atom::Used => v.kind == 2,
        Atom::OutsideTurn => ctx.state.turn.current != owner,
        Atom::InTurn => ctx.state.turn.current == owner,
        Atom::Lost => ctx
            .state
            .vm_aux
            .lost
            .get(&owner)
            .is_some_and(|c| !c.is_empty()),
        Atom::LostAtLeast(n) => ctx
            .state
            .vm_aux
            .lost
            .get(&owner)
            .is_some_and(|c| c.len() >= *n as usize),
        Atom::Batch => v.batch_len > 0 || v.kind == 4 || v.kind == 5,
        Atom::PileNonEmpty => ctx
            .state
            .vm_aux
            .piles
            .iter()
            .any(|((p, _), c)| *p == owner && !c.is_empty()),
        Atom::PileEmpty => ctx
            .state
            .vm_aux
            .piles
            .iter()
            .filter(|((p, _), _)| *p == owner)
            .all(|(_, c)| c.is_empty()),
        Atom::MarkPos => ctx
            .state
            .vm_aux
            .marks
            .iter()
            .any(|((p, _), n)| *p == owner && *n > 0),
        Atom::DrawNotFixed => !me.skip_next_draw,
        Atom::Once { scope } => {
            !ctx.state
                .vm_aux
                .once
                .contains(&(owner, skill.to_string(), *scope))
        }
        Atom::FlagSet(k) => {
            ctx.state
                .vm_aux
                .flags
                .get(&(owner, k.clone()))
                .copied()
                .unwrap_or(0)
                > 0
        }
        Atom::FlagUnset(k) => !ctx.state.vm_aux.once.contains(&(owner, k.clone(), 1)),
        Atom::Tag(k) => {
            skill.contains(k.as_str())
                || ctx.state.skills.iter().any(|((o, def), inst)| {
                    *o == owner && inst.enabled && ctx.catalog.skill_name(*def).contains(k.as_str())
                })
                || ctx.state.vm_aux.flags.contains_key(&(owner, k.clone()))
        }
        Atom::FirstHit => {
            ctx.state
                .vm_aux
                .hits
                .get(&(owner, skill.to_string()))
                .copied()
                .unwrap_or(0)
                == 0
        }
        Atom::SecondHit => {
            ctx.state
                .vm_aux
                .hits
                .get(&(owner, skill.to_string()))
                .copied()
                .unwrap_or(0)
                == 1
        }
        Atom::AfterFirst => {
            ctx.state
                .vm_aux
                .hits
                .get(&(owner, skill.to_string()))
                .copied()
                .unwrap_or(0)
                >= 1
        }
        Atom::OddHand => me.hand.len() % 2 == 1,
        Atom::Lord => ctx
            .state
            .identity_roles()
            .and_then(|r| r.get(owner as usize))
            .is_some_and(|r| *r == Role::Lord),
        Atom::Female => gender_of(ctx, owner) == Some(Gender::Female),
        Atom::Male => gender_of(ctx, owner) == Some(Gender::Male),
        Atom::SexSame => {
            let mine = gender_of(ctx, owner);
            v.event_player
                .or(v.event_source)
                .is_some_and(|o| o != owner && gender_of(ctx, o) == mine && mine.is_some())
        }
        Atom::GroupSame => {
            let mine = faction_of(ctx, owner);
            v.event_player
                .is_some_and(|o| faction_of(ctx, o) == mine && mine.is_some())
        }
        Atom::Transformed => v.use_as.is_some(),
        Atom::MinHp => alive && ctx.state.players.iter().all(|p| !p.alive || p.hp >= me.hp),
        Atom::TargetAlive => v
            .event_player
            .is_some_and(|p| ctx.state.players.get(p as usize).is_some_and(|pl| pl.alive)),
        Atom::TargetHandNonEmpty => v.event_player.is_some_and(|p| {
            ctx.state
                .players
                .get(p as usize)
                .is_some_and(|pl| !pl.hand.is_empty())
        }),
        Atom::TargetHandEmpty => v.event_player.is_some_and(|p| {
            ctx.state
                .players
                .get(p as usize)
                .is_some_and(|pl| pl.hand.is_empty())
        }),
        Atom::CanDiscard(n) => me.hand.len() + me.equips.len() >= *n as usize,
        Atom::CountAtLeast(n) => {
            v.event_amount >= *n as u32
                || me.hand.len() >= *n as usize
                || me.turn_damage >= *n
                || ctx
                    .state
                    .vm_aux
                    .lost
                    .get(&owner)
                    .is_some_and(|c| c.len() >= *n as usize)
        }
        Atom::SingleTarget => v.targets.len() <= 1,
        Atom::MultiTarget => v.targets.len() > 1,
        Atom::MovedFromOwner => v.moved_from == Some(owner),
        Atom::Gained => v.kind == 5 && v.event_player == Some(owner),
        Atom::FactionIs(tag) => faction_of(ctx, owner).is_some_and(|f| faction_tag(f) == *tag),
    }
}

fn gender_of(ctx: &QueryCtx<'_>, p: PlayerId) -> Option<Gender> {
    let g = ctx.state.player(p).general_slots[0]?;
    ctx.catalog
        .general_meta(g)
        .map(|(_, _, _, _, gender)| gender)
}

fn faction_of(ctx: &QueryCtx<'_>, p: PlayerId) -> Option<Faction> {
    ctx.state.effective_faction(ctx.catalog, p)
}

fn faction_tag(f: Faction) -> u8 {
    match f {
        Faction::Wei => 0,
        Faction::Shu => 1,
        Faction::Wu => 2,
        Faction::Qun => 3,
    }
}

fn kind_is_ordinary_trick(kind: CardKind) -> bool {
    if kind.is_basic_kind() || kind.equip_slot().is_some() {
        return false;
    }
    !matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
    ) && kind.is_basic_trick_or_equip()
}
