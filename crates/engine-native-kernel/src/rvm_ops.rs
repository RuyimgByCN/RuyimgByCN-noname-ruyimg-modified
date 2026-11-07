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
mod codec_ext;
mod codec_ext2;
mod codec_ext3;
mod commit;
pub use commit::*;
mod commit_combat;
mod commit_ext;
mod accept_batch15;
pub(crate) use accept_batch15::backfill_batch17_skill;
mod commit_batch13;
mod commit_batch14;
mod commit_batch15;
mod commit_batch15_rest;
mod commit_batch16;
mod commit_batch17;
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

pub(crate) fn card_kind_from_tag(tag: u8) -> Option<CardKind> {
    match tag {
        1 => Some(CardKind::Sha),
        2 => Some(CardKind::Tao),
        3 => Some(CardKind::Shan),
        4 => Some(CardKind::Jiu),
        5 => Some(CardKind::FireSha),
        6 => Some(CardKind::LeiSha),
        10 => Some(CardKind::Weapon),
        11 => Some(CardKind::Armor),
        12 => Some(CardKind::HorseMinus),
        13 => Some(CardKind::HorsePlus),
        14 => Some(CardKind::WuxingPan),
        20 => Some(CardKind::Wuzhong),
        21 => Some(CardKind::Dismantle),
        22 => Some(CardKind::Snatch),
        23 => Some(CardKind::Duel),
        24 => Some(CardKind::SavageAttack),
        25 => Some(CardKind::ArcheryAttack),
        26 => Some(CardKind::Taoyuan),
        27 => Some(CardKind::Wugu),
        30 => Some(CardKind::Le),
        31 => Some(CardKind::Bing),
        32 => Some(CardKind::ShanDian),
        33 => Some(CardKind::Jiedao),
        34 => Some(CardKind::Chain),
        36 => Some(CardKind::Nullify),
        40 => Some(CardKind::Youdishenru),
        42 => Some(CardKind::WangmeiZhike),
        _ => None,
    }
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

pub fn resolve_skill_def(catalog: &Catalog, raw: u32) -> Option<u16> {
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

pub(super) fn pile_name(catalog: &Catalog, executing: &str, pile_def: u32) -> String {
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
        | EffectRequest::RemoveAllSkills { player }
        | EffectRequest::DiscardAllHandAndEquip { player }
        | EffectRequest::DiscardAllHandEquipJudge { player }
        | EffectRequest::RestoreCharacterState { player }
        | EffectRequest::EquipWeaponFromDrawPile { player }
        | EffectRequest::QueueOpeningExtraPlayPhase { player, .. }
        | EffectRequest::AdjustHandTo { player, .. }
        | EffectRequest::DrawByPlayPhaseCardCategoryCount { player, .. }
        | EffectRequest::SetDrawPhaseFromBottom { player }
        | EffectRequest::SetNonLockedSkillSuppression { player, .. }
        | EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player }
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
        EffectRequest::DrawEachBySeat {
            first,
            second,
            count,
            ..
        } => {
            ok_player(*first, "DrawEachBySeat.first")?;
            ok_player(*second, "DrawEachBySeat.second")?;
            if *count == 0 {
                return Err(reject("DrawEachBySeat.count"));
            }
        }
        EffectRequest::DrawCardsFromBottom { player, count } => {
            ok_player(*player, "DrawCardsFromBottom.player")?;
            if *count == 0 {
                return Err(reject("DrawCardsFromBottom.count"));
            }
        }
        EffectRequest::CancelIncomingCardUse {
            player,
            source,
            card,
            ..
        } => {
            ok_player(*player, "CancelIncomingCardUse.player")?;
            ok_player(*source, "CancelIncomingCardUse.source")?;
            if !card.is_some() {
                return Err(reject("CancelIncomingCardUse.card"));
            }
        }
        EffectRequest::ExcludeSavedShaTarget { card, target } => {
            ok_player(*target, "ExcludeSavedShaTarget.target")?;
            if !card.is_some() {
                return Err(reject("ExcludeSavedShaTarget.card"));
            }
        }
        EffectRequest::TakeOneCard { from, to, .. }
        | EffectRequest::TakeOneHandCard { from, to } => {
            ok_player(*from, "TakeOneCard.from")?;
            ok_player(*to, "TakeOneCard.to")?;
        }
        EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def } => {
            ok_player(*player, "DisableSkillByDefUntilTurnEnd.player")?;
            if !skill_def.is_some() {
                return Err(reject("DisableSkillByDefUntilTurnEnd.skill_def"));
            }
        }
        EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
            ok_player(*player, "BanActiveUntilTurnEnd.player")?;
            if !skill.is_some() {
                return Err(reject("BanActiveUntilTurnEnd.skill"));
            }
        }
        EffectRequest::DrawLinkedPlayers { count } => {
            if *count == 0 {
                return Err(reject("DrawLinkedPlayers.count"));
            }
        }
        EffectRequest::RecoverIfMinCurrentHp { player, amount } => {
            ok_player(*player, "RecoverIfMinCurrentHp.player")?;
            if *amount == 0 {
                return Err(reject("RecoverIfMinCurrentHp.amount"));
            }
        }
        EffectRequest::RemoveCardsByKind { kind } => {
            if *kind == 0 {
                return Err(reject("RemoveCardsByKind.kind"));
            }
        }
        EffectRequest::IncreaseCurrentShaBaseDamage { player, amount, .. }
        | EffectRequest::IncreaseCurrentCardBaseDamage { player, amount, .. } => {
            ok_player(*player, "IncreaseCurrentCardDamage.player")?;
            if *amount == 0 {
                return Err(reject("IncreaseCurrentCardDamage.amount"));
            }
        }
        EffectRequest::IncreaseCurrentShaTargetDamage {
            source,
            target,
            amount,
            ..
        } => {
            ok_player(*source, "IncreaseCurrentShaTargetDamage.source")?;
            ok_player(*target, "IncreaseCurrentShaTargetDamage.target")?;
            if *amount == 0 {
                return Err(reject("IncreaseCurrentShaTargetDamage.amount"));
            }
        }
        EffectRequest::MakeCurrentShaTargetUnrespondable { source, target, .. } => {
            ok_player(*source, "MakeCurrentShaTargetUnrespondable.source")?;
            ok_player(*target, "MakeCurrentShaTargetUnrespondable.target")?;
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
        // PA12 批⑫：25 种选牌/选目标与确定性效果校验（84..=108）。
        EffectRequest::DiscardCards { player, cards } => {
            ok_player(*player, "DiscardCards.player")?;
            if cards.iter().any(|c| !c.is_some()) {
                return Err(reject("DiscardCards.card"));
            }
        }
        EffectRequest::StoreSelectedInSkillPile { player, cards, .. } => {
            ok_player(*player, "StoreSelectedInSkillPile.player")?;
            if cards.iter().any(|c| !c.is_some()) {
                return Err(reject("StoreSelectedInSkillPile.card"));
            }
        }
        EffectRequest::UseVirtualCardOn {
            player,
            kind,
            target,
        } => {
            ok_player(*player, "UseVirtualCardOn.player")?;
            ok_player(*target, "UseVirtualCardOn.target")?;
            if card_kind_from_tag(*kind).is_none() {
                return Err(reject("UseVirtualCardOn.kind"));
            }
        }
        EffectRequest::RecastCards { player, cards } => {
            ok_player(*player, "RecastCards.player")?;
            if cards.iter().any(|c| !c.is_some()) {
                return Err(reject("RecastCards.card"));
            }
        }
        EffectRequest::AddCardUsePolicySkill { player, .. }
        | EffectRequest::QueueExtraTurn { player, .. }
        | EffectRequest::GainTargetHandCard { player, .. }
        | EffectRequest::PlaceHandCardToExpansion { player, .. }
        | EffectRequest::ClearPhaseGiftTarget { player, .. } => {
            ok_player(*player, "Batch12.player")?;
        }
        EffectRequest::DrawSelectedTargets { targets, count, .. } => {
            if *count == 0 {
                return Err(reject("DrawSelectedTargets.count"));
            }
            for t in targets {
                ok_player(*t, "DrawSelectedTargets.target")?;
            }
        }
        EffectRequest::DisableEquipSlots { player, slots } => {
            ok_player(*player, "DisableEquipSlots.player")?;
            if slots.is_empty() || slots.iter().any(|s| !matches!(*s, 1..=5)) {
                return Err(reject("DisableEquipSlots.slots"));
            }
        }
        EffectRequest::EnableEquipSlot { player, slot, .. } => {
            ok_player(*player, "EnableEquipSlot.player")?;
            if !matches!(*slot, 1..=5) {
                return Err(reject("EnableEquipSlot.slot"));
            }
        }
        EffectRequest::GainShadowCards { player, count, .. }
        | EffectRequest::DistributeShadowCards { player, count, .. } => {
            ok_player(*player, "ShadowCards.player")?;
            if *count == 0 {
                return Err(reject("ShadowCards.count"));
            }
        }
        EffectRequest::AddBoundSkill {
            player,
            skill_def,
            bound_target,
        } => {
            ok_player(*player, "AddBoundSkill.player")?;
            ok_player(*bound_target, "AddBoundSkill.bound_target")?;
            if !skill_def.is_some() {
                return Err(reject("AddBoundSkill.skill_def"));
            }
        }
        EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards,
            ..
        } => {
            ok_player(*player, "GiveSelectedThenRecoverTarget.player")?;
            ok_player(*target, "GiveSelectedThenRecoverTarget.target")?;
            if cards.iter().any(|c| !c.is_some()) {
                return Err(reject("GiveSelectedThenRecoverTarget.card"));
            }
        }
        EffectRequest::RecordActiveTarget { player, target, .. }
        | EffectRequest::DiscardSelectedRecordUniqueCategoryTarget { player, target, .. }
        | EffectRequest::DiscardTargetCardGainMount { player, target, .. } => {
            ok_player(*player, "Batch12.player")?;
            ok_player(*target, "Batch12.target")?;
        }
        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            ..
        } => {
            ok_player(*source, "DealDamageBanActiveIfDying.source")?;
            ok_player(*target, "DealDamageBanActiveIfDying.target")?;
            if *amount == 0 || *amount > i32::MAX as u32 {
                return Err(reject("DealDamageBanActiveIfDying.amount"));
            }
        }
        EffectRequest::DiscardSelectedDrawAllHandBonus { player, cards, .. } => {
            ok_player(*player, "DiscardSelectedDrawAllHandBonus.player")?;
            if cards.iter().any(|c| !c.is_some()) {
                return Err(reject("DiscardSelectedDrawAllHandBonus.card"));
            }
        }
        EffectRequest::DrawForLifetimeCardActionDivisors { player, divisors } => {
            ok_player(*player, "DrawForLifetimeCardActionDivisors.player")?;
            if divisors.is_empty() || divisors.contains(&0) {
                return Err(reject("DrawForLifetimeCardActionDivisors.divisors"));
            }
        }
        EffectRequest::RecoverSelectedTargets {
            source,
            targets,
            amount,
            ..
        } => {
            if source.is_some() {
                ok_player(*source, "RecoverSelectedTargets.source")?;
            }
            if *amount == 0 {
                return Err(reject("RecoverSelectedTargets.amount"));
            }
            for t in targets {
                ok_player(*t, "RecoverSelectedTargets.target")?;
            }
        }
        EffectRequest::RevealTopGainIfColor { player, color_mask } => {
            ok_player(*player, "RevealTopGainIfColor.player")?;
            if !matches!(*color_mask, 1..=3) {
                return Err(reject("RevealTopGainIfColor.color_mask"));
            }
        }
        EffectRequest::UseVirtualCardOnSelectedTargets {
            player,
            kind,
            targets,
        } => {
            ok_player(*player, "UseVirtualCardOnSelectedTargets.player")?;
            if card_kind_from_tag(*kind).is_none() {
                return Err(reject("UseVirtualCardOnSelectedTargets.kind"));
            }
            for t in targets {
                ok_player(*t, "UseVirtualCardOnSelectedTargets.target")?;
            }
        }
        // PA12 批⑬：12 种高频交互式效果 + UP37-R01 敌仇 + 7 种确定性效果（tags 109..=128）。
        EffectRequest::ChooseOtherDamage { player, amount, .. }
        | EffectRequest::ChooseOtherRecover { player, amount } => {
            ok_player(*player, "Batch13.player")?;
            if *amount == 0 || *amount > i32::MAX as u32 {
                return Err(reject("Batch13.amount"));
            }
        }
        EffectRequest::ChooseTargetDraw { player, count, .. } => {
            ok_player(*player, "ChooseTargetDraw.player")?;
            if *count == 0 {
                return Err(reject("ChooseTargetDraw.count"));
            }
        }
        EffectRequest::ChooseDrawOrRecover {
            player,
            draw_count,
            recover_count,
            ..
        } => {
            ok_player(*player, "ChooseDrawOrRecover.player")?;
            if *draw_count == 0 && *recover_count == 0 {
                return Err(reject("ChooseDrawOrRecover.count"));
            }
        }
        EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            ..
        } => {
            ok_player(*player, "ChooseOtherAddMark.player")?;
            if *mark == 0 || *delta == 0 {
                return Err(reject("ChooseOtherAddMark.mark_delta"));
            }
        }
        EffectRequest::ResolveHfjieying {
            player,
            target,
            grant_def,
            ..
        } => {
            ok_player(*player, "ResolveHfjieying.player")?;
            if target.is_some() {
                ok_player(*target, "ResolveHfjieying.target")?;
            }
            if !grant_def.is_some() {
                return Err(reject("ResolveHfjieying.grant_def"));
            }
        }
        EffectRequest::ChooseDiscardOneHandOrEquip { player }
        | EffectRequest::DiscardBasicDrawEqual { player, .. }
        | EffectRequest::ChooseUseVirtualSha { player, .. }
        | EffectRequest::ChooseOtherDiscardTargetCard { player, .. }
        | EffectRequest::ChooseOtherGiveOneHandEquip { player, .. }
        | EffectRequest::RecordCurrentShaAllTargets { player, .. }
        | EffectRequest::FinishCurrentPhaseAbortResolution { player, .. } => {
            ok_player(*player, "Batch13.player")?;
        }
        EffectRequest::ChooseDiscardTargetCard { player, target, .. } => {
            ok_player(*player, "ChooseDiscardTargetCard.player")?;
            ok_player(*target, "ChooseDiscardTargetCard.target")?;
        }
        EffectRequest::ResolveDcdichou {
            player,
            target,
            action,
            helper_skill_def,
            ..
        } => {
            ok_player(*player, "ResolveDcdichou.player")?;
            if target.is_some() {
                ok_player(*target, "ResolveDcdichou.target")?;
            }
            if !matches!(*action, 0 | 1 | 2 | 3 | 10) {
                return Err(reject("ResolveDcdichou.action"));
            }
            if !helper_skill_def.is_some() {
                return Err(reject("ResolveDcdichou.helper_skill_def"));
            }
        }
        EffectRequest::TakeOneScopedCard { from, to, .. } => {
            if from.is_some() {
                ok_player(*from, "TakeOneScopedCard.from")?;
            }
            ok_player(*to, "TakeOneScopedCard.to")?;
        }
        EffectRequest::DrawAllEventTargets { players, count } => {
            if *count == 0 {
                return Err(reject("DrawAllEventTargets.count"));
            }
            for p in players {
                ok_player(*p, "DrawAllEventTargets.player")?;
            }
        }
        EffectRequest::RefreshSkill { .. } => {}
        EffectRequest::AddFirstMissingSkill {
            player, skill_defs, ..
        } => {
            ok_player(*player, "AddFirstMissingSkill.player")?;
            if skill_defs.iter().all(|d| !d.is_some()) {
                return Err(reject("AddFirstMissingSkill.skill_defs"));
            }
        }
        EffectRequest::DiscardEquipmentSlot { player, slot } => {
            ok_player(*player, "DiscardEquipmentSlot.player")?;
            if !matches!(*slot, 1..=5) {
                return Err(reject("DiscardEquipmentSlot.slot"));
            }
        }
        // PA12 批⑭：25 种高频交互式效果校验（tags 129..=153）。
        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            ..
        } => {
            ok_player(*player, "ChooseOtherDrawOrRecover.player")?;
            if (*draw_count == 0 && *recover_count == 0) || *times == 0 {
                return Err(reject("ChooseOtherDrawOrRecover.counts"));
            }
        }
        EffectRequest::ChooseAddOrRemoveCurrentCardTarget { player, .. }
        | EffectRequest::ChooseAddCurrentShaTarget { player, .. }
        | EffectRequest::ChooseHandCompare { player, .. }
        | EffectRequest::ChooseLinkOther { player, .. }
        | EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, .. }
        | EffectRequest::ChooseOneFromSkillPileDiscard { player, .. }
        | EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { player, .. } => {
            ok_player(*player, "Batch14.player")?;
        }
        EffectRequest::ChooseCompareSelectedHandCard { player, target, .. } => {
            ok_player(*player, "ChooseCompareSelectedHandCard.player")?;
            if target.is_some() {
                ok_player(*target, "ChooseCompareSelectedHandCard.target")?;
            }
        }
        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            amount,
            ..
        } => {
            ok_player(*player, "ChooseTargetsDamage.player")?;
            if *max_count == 0
                || *max_count < *min_count
                || *amount == 0
                || *amount > i32::MAX as u32
            {
                return Err(reject("ChooseTargetsDamage.args"));
            }
        }
        EffectRequest::DrawThenGiveOne {
            player,
            target,
            count,
            drawn,
            ..
        } => {
            ok_player(*player, "DrawThenGiveOne.player")?;
            ok_player(*target, "DrawThenGiveOne.target")?;
            if *count == 0 || drawn.iter().any(|c| !c.is_some()) {
                return Err(reject("DrawThenGiveOne.args"));
            }
        }
        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            max_count,
            ..
        } => {
            ok_player(*player, "ChooseOptionalTargetsDraw.player")?;
            if *count == 0 || *max_count == 0 {
                return Err(reject("ChooseOptionalTargetsDraw.counts"));
            }
        }
        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            ..
        } => {
            ok_player(*player, "ChooseTargetsDraw.player")?;
            if *count == 0 || *target_count == 0 {
                return Err(reject("ChooseTargetsDraw.counts"));
            }
        }
        EffectRequest::GuanxingTop {
            player,
            count,
            round,
            flipped,
            top,
            bottom,
            ..
        } => {
            ok_player(*player, "GuanxingTop.player")?;
            if *count == 0
                || *round > 1
                || flipped.iter().any(|c| !c.is_some())
                || top.iter().any(|c| !c.is_some())
                || bottom.iter().any(|c| !c.is_some())
            {
                return Err(reject("GuanxingTop.args"));
            }
        }
        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            max_distance,
            ..
        } => {
            ok_player(*player, "ChooseAddCurrentShaTargetWithinDistance.player")?;
            if *max_distance == 0 {
                return Err(reject(
                    "ChooseAddCurrentShaTargetWithinDistance.max_distance",
                ));
            }
        }
        EffectRequest::ChooseRecoverTarget {
            chooser,
            target,
            amount,
        } => {
            ok_player(*chooser, "ChooseRecoverTarget.chooser")?;
            ok_player(*target, "ChooseRecoverTarget.target")?;
            if *amount == 0 || *amount > i32::MAX as u32 {
                return Err(reject("ChooseRecoverTarget.amount"));
            }
        }
        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, .. } => {
            ok_player(*player, "ChooseTargetDrawToMaxHpCap.player")?;
            if *cap == 0 {
                return Err(reject("ChooseTargetDrawToMaxHpCap.cap"));
            }
        }
        EffectRequest::ChooseDiscardHandRange {
            player, min, max, ..
        } => {
            ok_player(*player, "ChooseDiscardHandRange.player")?;
            if *max == 0 || *max < *min {
                return Err(reject("ChooseDiscardHandRange.range"));
            }
        }
        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { player, count, .. } => {
            ok_player(*player, "ChooseDiscardHandEquipThenRemoveSelfSkill.player")?;
            if *count == 0 {
                return Err(reject("ChooseDiscardHandEquipThenRemoveSelfSkill.count"));
            }
        }
        EffectRequest::ChooseHandCardsToSkillPile {
            player,
            min_count,
            max_count,
            ..
        } => {
            ok_player(*player, "ChooseHandCardsToSkillPile.player")?;
            if *max_count == 0 || *max_count < *min_count {
                return Err(reject("ChooseHandCardsToSkillPile.range"));
            }
        }
        EffectRequest::ChooseGainDiscardCardByKind { player, kind, .. } => {
            ok_player(*player, "ChooseGainDiscardCardByKind.player")?;
            if *kind == 0 {
                return Err(reject("ChooseGainDiscardCardByKind.kind"));
            }
        }
        EffectRequest::ChooseAddMissingSkill {
            player, skill_defs, ..
        } => {
            ok_player(*player, "ChooseAddMissingSkill.player")?;
            if skill_defs.iter().all(|d| !d.is_some()) {
                return Err(reject("ChooseAddMissingSkill.skill_defs"));
            }
        }
        EffectRequest::ChooseMoveFieldCardTo { player, target, .. } => {
            ok_player(*player, "ChooseMoveFieldCardTo.player")?;
            ok_player(*target, "ChooseMoveFieldCardTo.target")?;
        }
        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed { player, target, .. } => {
            ok_player(*player, "JudgeMakeCurrentShaTargetUnrespondableOnRed.player")?;
            if target.is_some() {
                ok_player(*target, "JudgeMakeCurrentShaTargetUnrespondableOnRed.target")?;
            }
        }
        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { player, mark, .. } => {
            ok_player(
                *player,
                "ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan.player",
            )?;
            if !mark.is_some() {
                return Err(reject(
                    "ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan.mark",
                ));
            }
        }
        other => {
            if let Some(res) = accept_batch15::accept_batch15_effect(other, seats) {
                return res;
            }
            let name = format!("{other:?}");
            return Err(reject(name.split([' ', '(', '{']).next().unwrap_or("?")));
        }
    }
    Ok(())
}
