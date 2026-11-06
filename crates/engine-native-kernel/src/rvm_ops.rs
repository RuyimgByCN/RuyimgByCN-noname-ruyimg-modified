//! T8 余项：VM 效果与查询的真实读写。
//!
//! 未列入本文件的效果（具名结算器、选择链、用牌管线）在入队前拒绝，
//! 避免记成已支持的空成功。区域号由 `QueryRequest::Zone` 打包
//! （`rule_player << 8 | kind`），`MoveCards` 按同一布局拆开。
//!
//! 登记差异：终身伤害只累计本桥提交的 `DealDamage`（写入 `vm_aux.flags`
//! 键 `lifetime`），内核伤害仍只在 `turn_damage`。规则牌 id 0 与哨兵
//! 冲突，效果里的 0 视为无牌。`SkipCurrentPhase` / `UseCard` 无内核
//! 落点，拒绝。

use rule_api::{EffectRequest, HostTrap, QueryRequest, QueryResponse, RelationTag};

use crate::catalog::{CardKind, Catalog, Faction, Suit};
use crate::event::{DamageNature, EventData, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, LogEvent};
use crate::state::{GameStateCore, SkillInstance};
mod codec;
pub use codec::*;
mod commit;
pub use commit::*;
mod query;
pub use query::*;

fn reject(name: &str) -> HostTrap {
    HostTrap::InvalidRequest {
        message: format!("rvm_bridge 未映射: {name}"),
    }
}

pub(crate) fn seat_of(p: rule_api::PlayerId, seats: u8) -> Option<PlayerId> {
    if !p.is_some() {
        return None;
    }
    let raw = p.raw() - 1;
    let s = u8::try_from(raw).ok()?;
    (s < seats).then_some(s)
}

fn kind_tag(kind: CardKind) -> u8 {
    match kind {
        CardKind::Sha => 1,
        CardKind::Tao => 2,
        CardKind::Shan => 3,
        CardKind::Jiu => 4,
        CardKind::FireSha => 5,
        CardKind::LeiSha => 6,
        CardKind::Wuzhong => 20,
        CardKind::Dismantle => 21,
        CardKind::Snatch => 22,
        CardKind::Duel => 23,
        CardKind::SavageAttack => 24,
        CardKind::ArcheryAttack => 25,
        CardKind::Taoyuan => 26,
        CardKind::Wugu => 27,
        CardKind::Le => 30,
        CardKind::Bing => 31,
        CardKind::ShanDian => 32,
        CardKind::Jiedao => 33,
        CardKind::Chain => 34,
        CardKind::Nullify => 36,
        CardKind::Youdishenru => 40,
        CardKind::WangmeiZhike => 42,
        other => match other.equip_slot() {
            Some(crate::catalog::EquipSlot::Weapon) => 10,
            Some(crate::catalog::EquipSlot::Armor) => 11,
            Some(crate::catalog::EquipSlot::HorseMinus) => 12,
            Some(crate::catalog::EquipSlot::HorsePlus) => 13,
            Some(crate::catalog::EquipSlot::Treasure) => 14,
            None => 255,
        },
    }
}

pub(crate) fn resolve_skill_def(catalog: &Catalog, raw: u32) -> Option<u16> {
    if raw != 0 {
        if let Some((i, _)) = catalog
            .skills
            .iter()
            .enumerate()
            .find(|(_, s)| s.behavior.package_skill_id() == Some(raw))
        {
            return u16::try_from(i).ok();
        }
    }
    let id = u16::try_from(raw).ok()?;
    ((id as usize) < catalog.skills.len()).then_some(id)
}

fn skill_label(catalog: &Catalog, def: u32) -> Option<String> {
    let id = resolve_skill_def(catalog, def)?;
    Some(catalog.skill_name(id).to_string())
}

fn pile_name(catalog: &Catalog, executing: &str, pile_def: u32) -> String {
    skill_label(catalog, pile_def).unwrap_or_else(|| executing.to_string())
}

/// The private storage namespace is shared by host writes and VM queries.
pub fn storage_key(skill_name: &str, key: u32) -> String {
    format!("storage:{skill_name}#{key}")
}

/// 入队前校验。通过的变体必须能编码、能落账。
pub fn accept_effect(req: &EffectRequest, seats: u8) -> Result<(), HostTrap> {
    let ok_player = |p: rule_api::PlayerId, what: &str| -> Result<PlayerId, HostTrap> {
        seat_of(p, seats).ok_or_else(|| reject(what))
    };
    match req {
        EffectRequest::RecoverHp { player, amount, .. } => {
            ok_player(*player, "RecoverHp")?;
            if *amount == 0 {
                return Err(reject("RecoverHp 零量"));
            }
        }
        EffectRequest::DrawCards { player, count } => {
            ok_player(*player, "DrawCards")?;
            if *count == 0 {
                return Err(reject("DrawCards 零量"));
            }
        }
        EffectRequest::DealDamage {
            target,
            amount,
            nature,
            ..
        } => {
            ok_player(*target, "DealDamage")?;
            if *amount == 0 || *amount > i32::MAX as u32 || *nature > 2 {
                return Err(reject("DealDamage 越界"));
            }
        }
        EffectRequest::ChangeMaxHp { player, delta } => {
            ok_player(*player, "ChangeMaxHp")?;
            if *delta == 0 {
                return Err(reject("ChangeMaxHp 零量"));
            }
        }
        EffectRequest::AddMark { player, .. } | EffectRequest::RemoveMark { player, .. } => {
            ok_player(*player, "Mark")?;
        }
        EffectRequest::DiscardFromHand { player, count }
        | EffectRequest::DiscardFromHandEquip { player, count } => {
            ok_player(*player, "Discard")?;
            if *count == 0 {
                return Err(reject("Discard 零量"));
            }
        }
        EffectRequest::TurnOver { player } => {
            ok_player(*player, "TurnOver")?;
        }
        // PA12 批④-3：伤害改写对——只在 DamageBefore 前置窗口内合法
        //（当前帧即伤害帧；宿主技同口径直写 DamageData）。
        EffectRequest::PreventDamage { .. } => {}
        EffectRequest::ModifyCurrentDamage { player, .. } => {
            ok_player(*player, "ModifyCurrentDamage")?;
        }
        EffectRequest::SkipCurrentPhase { player } => {
            ok_player(*player, "SkipCurrentPhase")?;
        }
        // PA12 批⑤-3：DSL `storage_set(self_skill, key, value)`——技能实例
        // 私有存储（上游 player.storage.<skill>[key]）。
        EffectRequest::StorageSet { .. } => {}
        EffectRequest::ChooseGiveOrRecastOneHand { player, skill, .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
            player, skill, ..
        }
        | EffectRequest::ResolveOlJieDanshou { player, skill } => {
            ok_player(*player, "ResolveOlJieDanshou")?;
            if !skill.is_some() {
                return Err(reject("ResolveOlJieDanshou skill"));
            }
        }
        // R05-E（裁决 28 合同 v3 §2）：三固定 compound 的入队校验——
        // player 在座、skill 非 NONE。触发窗口资格（D 终窗/Prepare/
        // Source/End/回合开始）由内容机在执行时按当前帧核验；语义槽
        // （I,role,target）经 VmAux.marks 隔离，通用 AddMark/RemoveMark
        // 碰派生 helper 键由内容提交钩（vm_effects::step）复检拒绝。
        EffectRequest::ResolveTiqi { player, skill }
        | EffectRequest::ResolveBaoshu { player, skill }
        | EffectRequest::ResolveDcbihuo { player, skill } => {
            ok_player(*player, "R05 compound")?;
            if !skill.is_some() {
                return Err(reject("R05 compound skill"));
            }
        }
        // PA12 批④-2：DSL `discard_hand_equip_count(player, N)`（N 为字面
        // 正数）。`count == 0` 是 VM 首发（打开选牌）——选择挂起链未接，
        // 拒绝。
        EffectRequest::DiscardHandEquipCount { player, count, .. } => {
            ok_player(*player, "DiscardHandEquipCount")?;
            if *count == 0 {
                return Err(reject("DiscardHandEquipCount 选牌首发未接"));
            }
        }
        EffectRequest::AddSkill { player, .. }
        | EffectRequest::AddSkillUntilTurnEnd { player, .. }
        | EffectRequest::RemoveSkillByDef { player, .. }
        | EffectRequest::RevealHand { player }
        | EffectRequest::GainEquipmentFromDrawPile { player }
        | EffectRequest::GainEventCards { player, .. }
        | EffectRequest::SkipNextPlayPhase { player }
        | EffectRequest::SkipNextDiscardPhase { player }
        | EffectRequest::SkipNextJudgePhase { player }
        | EffectRequest::SkipJudgeAndDrawPhases { player, .. }
        | EffectRequest::SkipRemainingPhases { player, .. }
        | EffectRequest::QueueExtraPhases { player, .. }
        | EffectRequest::RefundCurrentShaUse { player, .. }
        | EffectRequest::EnterDying { player }
        | EffectRequest::DiscardAllOwnJudge { player, .. }
        | EffectRequest::GainTopCardToSkillPile { player, .. }
        | EffectRequest::GainAllSkillPileCards { player, .. }
        | EffectRequest::GainFromSkillPile { player, .. }
        | EffectRequest::DiscardSkillPile { player, .. }
        | EffectRequest::PlaceTopDeckToExpansion { player, .. }
        | EffectRequest::GainWholeSkillPile { player, .. }
        | EffectRequest::StoreEventCardInSkillPile { player, .. } => {
            ok_player(*player, "player")?;
        }
        EffectRequest::IncreaseCurrentDrawCount { player, count } => {
            ok_player(*player, "IncreaseCurrentDrawCount.player")?;
            if *count == 0 {
                return Err(reject("IncreaseCurrentDrawCount.count"));
            }
        }
        EffectRequest::GainTargetEquipment {
            player,
            target,
            slot,
        } => {
            ok_player(*player, "GainTargetEquipment.player")?;
            ok_player(*target, "GainTargetEquipment.target")?;
            if !matches!(*slot, 1..=5) {
                return Err(reject("GainTargetEquipment.slot"));
            }
        }
        EffectRequest::GainCardByKind { player, kind }
        | EffectRequest::GainDiscardCardByKind { player, kind } => {
            ok_player(*player, "GainCardByKind.player")?;
            if *kind == 0 {
                return Err(reject("GainCardByKind.kind"));
            }
        }
        EffectRequest::GainCardIfDiscard { player, card } => {
            ok_player(*player, "GainCardIfDiscard.player")?;
            if !card.is_some() {
                return Err(reject("GainCardIfDiscard.card"));
            }
        }
        EffectRequest::RevealHandToPlayer { player, observer } => {
            ok_player(*player, "RevealHandToPlayer.player")?;
            ok_player(*observer, "RevealHandToPlayer.observer")?;
        }
        EffectRequest::GiveAllHand { player, target } => {
            ok_player(*player, "GiveAllHand.player")?;
            ok_player(*target, "GiveAllHand.target")?;
            if player == target {
                return Err(reject("GiveAllHand.self"));
            }
        }
        EffectRequest::DiscardAllTargetEquipment { actor, target } => {
            ok_player(*actor, "DiscardAllTargetEquipment.actor")?;
            ok_player(*target, "DiscardAllTargetEquipment.target")?;
        }
        EffectRequest::GainOtherSkillPileCards { player, target, .. } => {
            ok_player(*player, "GainOther.player")?;
            ok_player(*target, "GainOther.target")?;
        }
        EffectRequest::RemoveSkill { skill } => {
            if !skill.is_some() {
                return Err(reject("RemoveSkill.NONE"));
            }
        }
        EffectRequest::KillPlayer { player, .. } => {
            ok_player(*player, "KillPlayer")?;
        }
        EffectRequest::TakeFromHand { from, to } => {
            ok_player(*from, "TakeFromHand.from")?;
            ok_player(*to, "TakeFromHand.to")?;
        }
        EffectRequest::MoveCards { cards, to, .. } => {
            if cards.is_empty() || to.raw() == 0 {
                return Err(reject("MoveCards 空"));
            }
        }
        EffectRequest::RevealCards { cards, .. } => {
            if cards.is_empty() {
                return Err(reject("RevealCards 空"));
            }
        }
        EffectRequest::SetTurnState { current, phase } => {
            ok_player(*current, "SetTurnState")?;
            if *phase > 4 {
                return Err(reject("SetTurnState phase"));
            }
        }
        other => {
            let name = format!("{other:?}");
            return Err(reject(name.split([' ', '(', '{']).next().unwrap_or("?")));
        }
    }
    Ok(())
}
