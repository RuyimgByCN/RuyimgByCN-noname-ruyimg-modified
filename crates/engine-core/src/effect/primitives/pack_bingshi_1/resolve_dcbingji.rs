//! Effect primitives for `resolve_dcbingji` and related helpers.

use super::*;

/// 专属状态机 (批 235, ADR 0641 §8.30)：秉纪——出牌阶段，若所有手牌同花色且
/// 本回合未使用过该花色，可展示所有手牌并视为使用一张【杀】或【桃】（该花色
/// 本回合记为已用）。
pub(crate) fn resolve_dcbingji(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    kind: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.phase != crate::state::Phase::Play
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            // 校验：手牌非空且全部同花色。
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let first = crate::query::effective_card_suit(
                tx.state(),
                player,
                &tx.state().cards.get(hand[0]).expect("card"),
            );
            if hand.iter().any(|card| {
                crate::query::effective_card_suit(
                    tx.state(),
                    player,
                    &tx.state().cards.get(*card).expect("card"),
                ) != first
            }) {
                return Ok(Vec::new());
            }
            // 校验：该花色本回合未使用过。
            let bit = suit_history_bit(first);
            let history =
                tx.state()
                    .turn
                    .flag(crate::state::turn_flag::used_card_suit_history_by_player(
                        player,
                    ));
            if history & bit != 0 {
                return Ok(Vec::new());
            }
            let suit_tag = suit_tag(first);
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveDcbingjiStage,
                vec![
                    RESOLVE_DCBINGJI_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    1,
                    suit_tag,
                ],
            )
        }
        1 => {
            // kind=1 (sha)：选目标；kind=2 (tao)：对自己使用。
            if kind == 2 {
                use_dcbingji_virtual(tx, player, 2, player)?;
                Ok(Vec::new())
            } else {
                let targets: Vec<u32> = tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|&candidate| candidate != player)
                    .map(PlayerId::raw)
                    .collect();
                if targets.is_empty() {
                    return Ok(Vec::new());
                }
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    targets,
                    ChoiceKind::SelectTargets,
                    ContinuationKind::ResolveDcbingjiStage,
                    vec![
                        RESOLVE_DCBINGJI_CUSTOM_CHOICE,
                        player.raw(),
                        skill.raw(),
                        2,
                        1,
                    ],
                )
            }
        }
        2 => {
            // kind=1 (sha)：对所选目标使用。
            use_dcbingji_virtual(tx, player, 1, PlayerId::new(kind))?;
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

pub(crate) fn suit_history_bit(suit: engine_api::Suit) -> i32 {
    match suit {
        engine_api::Suit::Spade => 1,
        engine_api::Suit::Heart => 1 << 1,
        engine_api::Suit::Club => 1 << 2,
        engine_api::Suit::Diamond => 1 << 3,
        engine_api::Suit::None => 0,
    }
}

pub(crate) fn suit_tag(suit: engine_api::Suit) -> u32 {
    match suit {
        engine_api::Suit::Spade => 1,
        engine_api::Suit::Heart => 2,
        engine_api::Suit::Club => 3,
        engine_api::Suit::Diamond => 4,
        engine_api::Suit::None => 0,
    }
}

pub(crate) fn suit_from_tag(tag: u32) -> engine_api::Suit {
    match tag {
        1 => engine_api::Suit::Spade,
        2 => engine_api::Suit::Heart,
        3 => engine_api::Suit::Club,
        4 => engine_api::Suit::Diamond,
        _ => engine_api::Suit::None,
    }
}

pub(crate) fn use_dcbingji_virtual(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u32,
    target: PlayerId,
) -> Result<(), EffectError> {
    // kind: 1 = sha, 2 = tao（物化虚拟牌并按规则使用）。
    let card_kind = if kind == 1 {
        crate::state::CardKind::Sha
    } else {
        crate::state::CardKind::Tao
    };
    let card = crate::effect::materialize_virtual_card(tx, player, card_kind)?;
    crate::command::play::play_as_kind(
        tx,
        player,
        card,
        card_kind,
        &[target],
        false,
        false,
        true,
        false,
    )?;
    // 记录该花色本回合已用（批 230 入册缺口的面内实现）：虚拟牌无花色，
    // 取持有者当前统一手牌花色（虚拟使用不消耗手牌，同花色不变）。
    let hand = tx.state().hand_cards(player);
    if let Some(first) = hand.first() {
        let suit = crate::query::effective_card_suit(
            tx.state(),
            player,
            &tx.state().cards.get(*first).expect("card"),
        );
        if hand.iter().all(|card| {
            crate::query::effective_card_suit(
                tx.state(),
                player,
                &tx.state().cards.get(*card).expect("card"),
            ) == suit
        } && suit != engine_api::Suit::None)
        {
            tx.state_mut().turn.remember_used_card_suit(player, suit);
        }
    }
    Ok(())
}

/// 变张置入（A0 置入面，ADR 0641 §8.12）：从自身手牌选 [min_count, max_count]
/// 张置入自身技能堆（允许取消=不置入）。上限以当前手牌数钳位；max==0 或空手
/// no-op。
pub(crate) fn choose_hand_cards_to_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    min_count: u32,
    max_count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if max_count == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let hand_cards: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    if hand_cards.is_empty() {
        return Ok(Vec::new());
    }
    let n = hand_cards.len() as u32;
    let lo = min_count.min(n) as u8;
    let hi = max_count.min(n) as u8;
    open_choice(
        tx,
        player,
        lo,
        hi,
        true,
        hand_cards,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseHandCardsToSkillPile,
        vec![player.raw(), skill.raw(), min_count, max_count],
    )
}

pub(crate) fn rearrange_pile_to_deck_top(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if count == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, pile_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let n = (count as usize).min(cards.len()) as u8;
    open_choice(
        tx,
        player,
        n,
        n,
        true,
        cards.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::RearrangePileToDeckTop,
        vec![player.raw(), skill.raw(), pile_def, count],
    )
}

pub(crate) fn discard_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    // Gate on the originating skill being live (provenance); the pile itself is
    // addressed by the `pile_def` literal so sibling piles (e.g. a parent skill's
    // marker pile) can be discarded.
    let _ = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, pile_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let pile_zone = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == pile_def)
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::Illegal("skill pile zone missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    move_cards_with_reason(tx, cards, discard, Some(pile_zone), CardMoveReason::Discard)
}

pub(crate) fn gain_from_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, skill_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        cards.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::GainFromSkillPile,
        vec![player.raw(), skill.raw()],
    )
}

/// §7 技能夺取设施 S0（ABI 675，第一百四十三批）：选择一名其他存活角色，
/// 再从其可获得技能名单中获得一个。名单=目标存活技能 def 元数据
/// `locked==false && non_gainable==false`，按 def raw 升序去重。
/// 分段共用 Cont 283：payload [owner, skill, stage(0|1), (stage1: target, defs...)]。
pub(crate) fn choose_gain_other_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mut targets: Vec<u32> = Vec::new();
    for candidate in tx.state().players.alive_by_seat() {
        if candidate == player {
            continue;
        }
        if !gainable_skill_defs(tx, candidate).is_empty() {
            targets.push(candidate.raw());
        }
    }
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::Custom,
        ContinuationKind::ChooseGainOtherSkill,
        vec![player.raw(), skill.raw(), 0],
    )
}

/// 目标可获得技能 def 名单（升序、去重、过滤 locked/non_gainable）。
pub(crate) fn gainable_skill_defs(tx: &Transaction, target: PlayerId) -> Vec<u32> {
    use std::collections::BTreeSet;
    let mut defs: BTreeSet<u32> = BTreeSet::new();
    for instance in tx.state().skills.sorted_live() {
        if instance.owner != target || !instance.enabled {
            continue;
        }
        let Some(meta) = tx.state().content.skill_metadata(instance.def) else {
            continue;
        };
        if meta.locked || meta.non_gainable {
            continue;
        }
        defs.insert(instance.def.raw());
    }
    defs.into_iter().collect()
}

/// Cont 283 resolver：stage0 选目标 → stage1 名单选技并 AddSkill。
pub(crate) fn resolve_choose_gain_other_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::EngineFault("bad gain-skill payload".into()));
    }
    let (owner_raw, skill_raw, stage) = (payload[0], payload[1], payload[2]);
    let player = PlayerId::new(owner_raw);
    let skill = engine_api::SkillId::new(skill_raw);
    if observer != player {
        return Err(EffectError::Illegal("gain-skill chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    if stage == 0 {
        let Some(&target_raw) = selections.first() else {
            return Ok(());
        };
        let target = PlayerId::new(target_raw);
        let defs = gainable_skill_defs(tx, target);
        if defs.is_empty() {
            return Ok(());
        }
        let mut payload: Vec<u32> = vec![owner_raw, skill_raw, 1, target_raw];
        payload.extend_from_slice(&defs);
        open_choice(
            tx,
            player,
            1,
            1,
            true,
            defs,
            ChoiceKind::Custom,
            ContinuationKind::ChooseGainOtherSkill,
            payload,
        )?;
        return Ok(());
    }
    let Some(&target_raw) = payload.get(3) else {
        return Err(EffectError::EngineFault("bad gain-skill stage1".into()));
    };
    let defs = &payload[4..];
    let Some(&index) = selections.first() else {
        return Ok(());
    };
    let Some(def) = defs.get(index as usize).copied() else {
        return Ok(());
    };
    let _ = target_raw;
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: engine_api::SkillDefId::new(def),
        }],
    )?;
    tx.state_mut()
        .facility_gained_skills
        .entry(owner_raw)
        .or_default()
        .push(def);
    Ok(())
}

/// §7 技能夺取设施 S1（ABI 676）：从本设施获得清单中选择一个技能移除
/// （RemoveSkill+出清登记）。清单空则静默。Cont 284，payload [player, skill, defs...]。
pub(crate) fn lose_gained_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let Some(defs) = tx.state().facility_gained_skills.get(&player.raw()) else {
        return Ok(Vec::new());
    };
    if defs.is_empty() {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = defs.clone();
    let mut payload = vec![player.raw(), skill.raw()];
    payload.extend(options.iter().copied());
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::LoseGainedSkill,
        payload,
    )
}

/// Cont 284 resolver：移除所选已获得技能并出清登记。
pub(crate) fn resolve_lose_gained_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 2 {
        return Err(EffectError::EngineFault("bad lose-skill payload".into()));
    }
    let (owner_raw, skill_raw) = (payload[0], payload[1]);
    let player = PlayerId::new(owner_raw);
    let skill = engine_api::SkillId::new(skill_raw);
    if observer != player {
        return Err(EffectError::Illegal("lose-skill chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let defs = &payload[2..];
    let Some(&index) = selections.first() else {
        return Ok(());
    };
    let Some(def) = defs.get(index as usize).copied() else {
        return Ok(());
    };
    // 找到该 def 的存活实例（可能被移除/刷新过，找不到则仅出清登记）。
    let instance_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.owner == player && s.def.raw() == def)
        .map(|s| s.id);
    if let Some(id) = instance_id {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill: id }])?;
    }
    if let Some(list) = tx.state_mut().facility_gained_skills.get_mut(&owner_raw) {
        list.retain(|d| *d != def);
        if list.is_empty() {
            tx.state_mut().facility_gained_skills.remove(&owner_raw);
        }
    }
    let _ = skill;
    Ok(())
}

/// §7 技能夺取设施 S2（ABI 677）：合韵延伸。三段链（共用 Cont 285，stage 位区分）：
/// 段0 选一名共享「可获得技能」的其他存活角色 → 段1 选一个共同技能 def →
/// 段2 持有者失去该技能+目标摸 2。
/// 可共享判定：!locked && !non_gainable && !lord_only（上游 charlotte 排除在
/// 元数据中无对应标志，以此近似，已注明）。
pub(crate) fn eligible_shared_defs(tx: &Transaction, owner: PlayerId) -> Vec<u32> {
    use std::collections::BTreeSet;
    let mut defs: BTreeSet<u32> = BTreeSet::new();
    for instance in tx.state().skills.sorted_live() {
        if instance.owner != owner || !instance.enabled {
            continue;
        }
        let Some(meta) = tx.state().content.skill_metadata(instance.def) else {
            continue;
        };
        if meta.locked || meta.non_gainable || meta.lord_only {
            continue;
        }
        defs.insert(instance.def.raw());
    }
    defs.into_iter().collect()
}

pub(crate) fn resolve_potheyun(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mine = eligible_shared_defs(tx, player);
    if mine.is_empty() {
        return Ok(Vec::new());
    }
    let mine_set: std::collections::BTreeSet<u32> = mine.iter().copied().collect();
    let mut targets: Vec<u32> = Vec::new();
    for candidate in tx.state().players.alive_by_seat() {
        if candidate == player {
            continue;
        }
        let theirs: std::collections::BTreeSet<u32> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .filter(|s| s.owner == candidate && s.enabled)
            .map(|s| s.def.raw())
            .collect();
        if !mine_set.intersection(&theirs).next().is_none() {
            targets.push(candidate.raw());
        }
    }
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::Custom,
        ContinuationKind::ResolvePotheyun,
        vec![player.raw(), skill.raw(), 0],
    )
}

/// Cont 285 resolver：stage1 选共同技能；stage2 失去技能+目标摸 2。
pub(crate) fn resolve_potheyun_pick(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::EngineFault("bad potheyun payload".into()));
    }
    let (owner_raw, skill_raw, stage) = (payload[0], payload[1], payload[2]);
    let player = PlayerId::new(owner_raw);
    let skill = engine_api::SkillId::new(skill_raw);
    if observer != player {
        return Err(EffectError::Illegal("potheyun chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    if stage == 0 {
        let Some(&target_raw) = selections.first() else {
            return Ok(());
        };
        let target = PlayerId::new(target_raw);
        let mine: std::collections::BTreeSet<u32> =
            eligible_shared_defs(tx, player).into_iter().collect();
        let theirs: std::collections::BTreeSet<u32> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .filter(|s| s.owner == target && s.enabled)
            .map(|s| s.def.raw())
            .collect();
        let shared: Vec<u32> = mine.intersection(&theirs).copied().collect();
        if shared.is_empty() {
            return Ok(());
        }
        let mut payload: Vec<u32> = vec![owner_raw, skill_raw, 1, target_raw];
        payload.extend_from_slice(&shared);
        open_choice(
            tx,
            player,
            1,
            1,
            true,
            shared,
            ChoiceKind::Custom,
            ContinuationKind::ResolvePotheyun,
            payload,
        )?;
        return Ok(());
    }
    let Some(&target_raw) = payload.get(3) else {
        return Err(EffectError::EngineFault("bad potheyun stage1".into()));
    };
    let defs = &payload[4..];
    let Some(&index) = selections.first() else {
        return Ok(());
    };
    let Some(def) = defs.get(index as usize).copied() else {
        return Ok(());
    };
    let instance_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.owner == player && s.def.raw() == def)
        .map(|s| s.id);
    if let Some(id) = instance_id {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill: id }])?;
    }
    run_effects(
        tx,
        vec![EffectRequest::DrawCards {
            player: PlayerId::new(target_raw),
            count: 2,
        }],
    )?;
    Ok(())
}
