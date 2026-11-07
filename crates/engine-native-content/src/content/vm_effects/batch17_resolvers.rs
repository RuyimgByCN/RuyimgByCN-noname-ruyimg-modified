//! PA12 批⑰：60 种效果与具名复合结算器（tags 209..=254 + ext_tag 256..=269）的交互选牌/选目标/选分支状态机。
//!
//! 当入队请求已携带确定参数（或当前局面仅存唯一/零可选项）时返回 `None`，
//! 交由内核 `rvm_ops::commit_batch17` 直接权威落账；当存在需玩家决策的多选一
//! 窗口时挂起 `Flow::WaitChoice`，并在应答到达后将所选参数注入请求落账。

use super::batch13_targets::other_alive_options;
use super::*;

fn hand_equip_options(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .chain(core.player(owner).equips.iter())
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

fn batch17_owner(effect: &EffectRequest, core: &GameStateCore) -> Option<u8> {
    match effect {
        EffectRequest::ChooseCardUserOrAddTarget { player, .. }
        | EffectRequest::ChooseCurrentCardUseEnhancements { player, .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha { player, .. }
        | EffectRequest::ChooseOptionalDiscardTargetCard { player, .. }
        | EffectRequest::ChooseUseVirtualCard { player, .. }
        | EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecover { player, .. }
        | EffectRequest::ChooseDiscardWeaponOrLoseHp { player, .. }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { player, .. }
        | EffectRequest::ChooseOtherHandStoreSkillPile { player, .. }
        | EffectRequest::ChooseOtherYinghun { player, .. }
        | EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, .. }
        | EffectRequest::ResolveDiliYuheng { player, .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { player, .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { player, .. }
        | EffectRequest::LoseHpDoubleCurrentDamageThenReplace { player, .. }
        | EffectRequest::DrawThenOptionallyDistributeHandCards { player, .. }
        | EffectRequest::ReplaceDrawWithDistinctSuitSelection { player, .. }
        | EffectRequest::StartDebateAllOthers { player, .. }
        | EffectRequest::StartDebateTargets { player, .. }
        | EffectRequest::ResolveBailingyun { player, .. }
        | EffectRequest::ResolveJieliangGain { player, .. }
        | EffectRequest::ResolveJsrgchaozheng { player, .. }
        | EffectRequest::ResolveStarcuxiaWeiming { player, .. }
        | EffectRequest::ResolveReanxu { player, .. }
        | EffectRequest::ResolveRebingyi { player, .. }
        | EffectRequest::ResolveRejizhi { player, .. }
        | EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { player, .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { player, .. }
        | EffectRequest::ResolveHoufeng { player, .. }
        | EffectRequest::ResolveHuzun { player, .. }
        | EffectRequest::ResolveMalingli { player, .. }
        | EffectRequest::ResolveZhugemengxue { player, .. }
        | EffectRequest::ResolveTwyuyan { player, .. }
        | EffectRequest::ResolveTwzhibian { player, .. }
        | EffectRequest::ChooseAddSkillBoundCardTarget { player, .. }
        | EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill { player, .. }
        | EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory { player, .. }
        | EffectRequest::ChooseGainCardFromEachOtherReplaceDraw { player, .. }
        | EffectRequest::ChooseGainCardFromEachOtherThenTurnOver { player, .. }
        | EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, .. }
        | EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw { player, .. }
        | EffectRequest::ChooseGainTopCardsBySuit { player, .. }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { player, .. }
        | EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, .. }
        | EffectRequest::ChooseTargetHandCardToPile { player, .. }
        | EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player, ..
        }
        | EffectRequest::CityConsume { player, .. }
        | EffectRequest::DiscardEquipmentPairWithArmor { player, .. }
        | EffectRequest::DiscardOneOwnXumou { player, .. }
        | EffectRequest::DiscardRememberedHandCards { player, .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { player, .. }
        | EffectRequest::ExtraResolveCurrentTrick { player, .. }
        | EffectRequest::GainDrawCardsForLifetimeCardActionDivisors { player, .. }
        | EffectRequest::GainHandCompareResultCard { player, .. }
        | EffectRequest::GainHighestRankShaFromEventCards { player, .. }
        | EffectRequest::MoveEquipmentThenDraw { player, .. } => seat(*player, core),
        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget { chooser, .. } => {
            seat(*chooser, core)
        }
        EffectRequest::ChooseDiscardTargetCardResolveCurrentSha { source, .. }
        | EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus { source, .. } => {
            seat(*source, core)
        }
        _ => None,
    }
}

fn classify_interactive(
    effect: &EffectRequest,
    core: &GameStateCore,
    _catalog: &Catalog,
) -> Option<(u8, ChoiceKind, Vec<ChoiceOption>, u8, u8, bool)> {
    match effect {
        EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player, target, ..
        }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { player, target, .. }
        | EffectRequest::ChooseOtherHandStoreSkillPile { player, target, .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { player, target, .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { player, target, .. }
        | EffectRequest::ResolveTwzhibian {
            player,
            target,
            stage: 0,
            ..
        }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { player, target, .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { player, target, .. }
            if !target.is_some() || *target == *player =>
        {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ChooseOtherYinghun {
            player,
            target,
            branch: 0,
            ..
        } if !target.is_some() => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ChooseOtherYinghun {
            player,
            target,
            branch: 0,
            ..
        } if target.is_some() => {
            let owner = seat(*player, core)?;
            let opts = vec![ChoiceOption::Tag(1), ChoiceOption::Tag(2)];
            Some((owner, ChoiceKind::PickTag, opts, 1, 1, false))
        }
        EffectRequest::ChooseDiscardHandEquipThenRecover { player, count, .. } => {
            let owner = seat(*player, core)?;
            let opts = hand_equip_options(core, owner);
            let n = (*count as u8).min(opts.len() as u8);
            (opts.len() > usize::from(n) && n > 0)
                .then_some((owner, ChoiceKind::SelectCards, opts, n, n, true))
        }
        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser, count, ..
        } => {
            let owner = seat(*chooser, core)?;
            let opts = hand_equip_options(core, owner);
            let n = (*count as u8).min(opts.len() as u8);
            (opts.len() > usize::from(n) && n > 0)
                .then_some((owner, ChoiceKind::SelectCards, opts, n, n, true))
        }
        EffectRequest::ResolveDiliYuheng {
            player,
            action: 1..=4,
            card,
            event_cards,
            ..
        } if !card.is_some() && event_cards.len() > 1 => {
            let owner = seat(*player, core)?;
            let opts: Vec<ChoiceOption> = event_cards
                .iter()
                .map(|c| ChoiceOption::Card(c.raw()))
                .collect();
            Some((owner, ChoiceKind::SelectCards, opts, 1, 1, false))
        }
        EffectRequest::ResolveBailingyun {
            player,
            target,
            action: 10,
            ..
        } if !target.is_some() || *target == *player => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveJieliangGain { player, cards, .. } if cards.len() > 1 => {
            let owner = seat(*player, core)?;
            let opts: Vec<ChoiceOption> =
                cards.iter().map(|c| ChoiceOption::Card(c.raw())).collect();
            Some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveReanxu {
            player,
            stage: 0,
            first,
            ..
        } if !first.is_some() => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveHoufeng {
            player,
            target,
            mode: 0,
            stage: 0,
            ..
        } if !target.is_some() || *target == *player => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveHuzun {
            player,
            action: 1,
            targets,
            ..
        } if targets.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = alive_options(core);
            let max = (opts.len() as u8).min(2);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, max, true))
        }
        EffectRequest::ResolveMalingli {
            player,
            action: 20,
            target,
            ..
        } if !target.is_some() => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveZhugemengxue {
            player,
            target,
            action: 10 | 12,
            ..
        } if !target.is_some() => {
            let owner = seat(*player, core)?;
            let opts = alive_options(core);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
            player,
            target,
            selected,
            ..
        } if !selected.is_some() => {
            let owner = seat(*player, core)?;
            let t = seat(*target, core).unwrap_or(owner);
            let opts = hand_equip_options(core, t);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, false))
        }
        EffectRequest::DiscardEquipmentPairWithArmor {
            player, targets, ..
        } if targets.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        _ => None,
    }
}

pub(super) fn active_ask(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if state != [1] {
        return None;
    }
    let (observer, kind, options, min, max, allow_cancel) =
        classify_interactive(effect, core, catalog)?;
    Some(ask(
        observer,
        "rvm_choice",
        kind,
        options,
        usize::from(min),
        usize::from(max),
        allow_cancel,
    ))
}

pub(super) fn step(
    ctx: &mut Ctx<'_>,
    skill: &str,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    let owner = batch17_owner(effect, ctx.core)?;
    if state.is_empty() {
        let (observer, kind, options, min, max, allow_cancel) =
            classify_interactive(effect, ctx.core, ctx.catalog)?;
        state.push(1);
        return Some(Flow::WaitChoice {
            observer,
            spec: ChoiceSpec {
                kind,
                prompt: "rvm_choice",
                options,
                min: usize::from(min),
                max: usize::from(max),
                allow_cancel,
            },
        });
    }
    let ans = ctx.answer();
    if ans.is_empty() {
        return Some(Flow::Done);
    }
    let picked_cards: Vec<rule_api::CardId> = ans
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(rule_api::CardId::new(*c)),
            _ => None,
        })
        .collect();
    let picked_players: Vec<rule_api::PlayerId> = ans
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(rule_api::PlayerId::new(*p as u32 + 1)),
            _ => None,
        })
        .collect();
    let picked_tag: Option<u8> = ans.iter().find_map(|o| match o {
        ChoiceOption::Tag(t) => Some(*t),
        _ => None,
    });
    let mut resolved = effect.clone();
    match &mut resolved {
        EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha { target, .. }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { target, .. }
        | EffectRequest::ChooseOtherHandStoreSkillPile { target, .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { target, .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { target, .. }
        | EffectRequest::ResolveBailingyun { target, .. }
        | EffectRequest::ResolveHoufeng { target, .. }
        | EffectRequest::ResolveMalingli { target, .. }
        | EffectRequest::ResolveZhugemengxue { target, .. }
        | EffectRequest::ResolveTwzhibian { target, .. }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { target, .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { target, .. } => {
            if let Some(&p) = picked_players.first() {
                *target = p;
            }
        }
        EffectRequest::ChooseOtherYinghun { target, branch, .. } => {
            if let Some(&p) = picked_players.first() {
                *target = p;
                *branch = 1;
            }
            if let Some(t) = picked_tag {
                *branch = t;
            }
        }
        EffectRequest::ChooseDiscardHandEquipThenRecover { count, .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecoverTarget { count, .. } => {
            for c in &picked_cards {
                let raw = c.raw();
                if ctx.core.player(owner).hand.contains(&raw) {
                    ctx.discard_from_hand(owner, raw);
                } else if ctx.core.player(owner).equips.contains(&raw) {
                    ctx.take_owned_card(owner, raw);
                    ctx.core.discard.push(raw);
                }
            }
            *count = 0;
        }
        EffectRequest::ResolveDiliYuheng { card, .. } => {
            if let Some(&c) = picked_cards.first() {
                *card = c;
            }
        }
        EffectRequest::ResolveJieliangGain { cards, .. } => {
            if !picked_cards.is_empty() {
                *cards = picked_cards;
            }
        }
        EffectRequest::ResolveReanxu { first, .. } => {
            if let Some(&p) = picked_players.first() {
                *first = p;
            }
        }
        EffectRequest::ResolveHuzun { targets, .. }
        | EffectRequest::DiscardEquipmentPairWithArmor { targets, .. } => {
            if !picked_players.is_empty() {
                *targets = picked_players;
            }
        }
        EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw { selected, .. } => {
            if let Some(&c) = picked_cards.first() {
                *selected = c;
            }
        }
        _ => {}
    }
    crate::rvm_ops::commit_effect(ctx, skill, owner, resolved);
    Some(Flow::Done)
}

pub(super) fn validate(effect: &EffectRequest, state: &[u32], core: &GameStateCore) -> bool {
    state == [1] && batch17_owner(effect, core).is_some()
}
