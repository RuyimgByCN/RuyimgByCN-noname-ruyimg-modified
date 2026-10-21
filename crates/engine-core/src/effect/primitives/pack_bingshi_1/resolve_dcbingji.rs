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

/// 事件牌颜色（上游 get.color 口径）：1=红（♥♦），2=黑（♠♣），None=无牌/无色。
pub(crate) fn twmouzhi_card_color(tx: &Transaction, raw: u32) -> Option<u32> {
    if raw == 0 {
        return None;
    }
    tx.state()
        .cards
        .get(CardId::new(raw))
        .and_then(|card| match card.suit {
            engine_api::Suit::Heart | engine_api::Suit::Diamond => Some(1),
            engine_api::Suit::Spade | engine_api::Suit::Club => Some(2),
            _ => None,
        })
}

/// 专属状态机 (批 245, ADR 0641 §8.39)：谋制——锁定技。mode 0（damage_before）：
/// 事件牌有颜色且与记录的上次受伤牌颜色相同时防止本次伤害（防止不更新记录）；
/// mode 1（damage）：记录事件牌颜色（无牌/无色清除记录）。
pub(crate) fn resolve_twmouzhi(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    event_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match mode {
        0 => {
            let Some(color) = twmouzhi_card_color(tx, event_card) else {
                return Ok(Vec::new());
            };
            let last = tx.state().skills.storage_get(skill, TWMOUZHI_COLOR_KEY);
            if last != 0 && last == color {
                let damage_id = tx.state().resolution.current_damage_id;
                if damage_id != 0
                    && !tx
                        .state()
                        .resolution
                        .prevented_damage_ids
                        .contains(&damage_id)
                {
                    tx.state_mut()
                        .resolution
                        .prevented_damage_ids
                        .push(damage_id);
                }
            }
        }
        1 => {
            let color = twmouzhi_card_color(tx, event_card).unwrap_or(0);
            if !tx
                .state_mut()
                .skills
                .storage_set(skill, TWMOUZHI_COLOR_KEY, color)
            {
                return Err(EffectError::Illegal("twmouzhi: bad skill".into()));
            }
        }
        _ => {}
    }
    Ok(Vec::new())
}

/// stage 3 轮次载荷：[magic, player, skill, 3, k, m, targets(k), revealed(m),
/// picks(2×r)]——picks 按 (chooser_raw, card_raw) 记录，card=0 表示该轮跳过
/// （目标死亡或余牌为空）。
pub(crate) fn twdaigui_round_payload(
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: &[u32],
    revealed: &[u32],
    picks: &[(u32, u32)],
) -> Vec<u32> {
    let mut payload = vec![
        RESOLVE_TWDAIGUI_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        targets.len() as u32,
        revealed.len() as u32,
    ];
    payload.extend_from_slice(targets);
    payload.extend_from_slice(revealed);
    for (chooser, card) in picks {
        payload.push(*chooser);
        payload.push(*card);
    }
    payload
}

/// 专属状态机 (批 247/248, ADR 0641 §8.40)：待归——出牌阶段结束时，若你的
/// 手牌颜色均相同，选择至多 X（手牌数）名角色，亮出牌堆底等量的牌，这些角色
/// 按座次依次选获其中一张（slice 2）。
pub(crate) fn resolve_twdaigui(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            // 手牌同色门控（机内权威校验，上游 filter：所有手牌 get.color 一致）。
            let mut red = 0usize;
            let mut black = 0usize;
            for card in &hand {
                if let Some(card_state) = tx.state().cards.get(*card) {
                    match card_state.suit {
                        engine_api::Suit::Heart | engine_api::Suit::Diamond => red += 1,
                        engine_api::Suit::Spade | engine_api::Suit::Club => black += 1,
                        _ => {}
                    }
                }
            }
            if red > 0 && black > 0 {
                return Ok(Vec::new());
            }
            if red == 0 && black == 0 {
                return Ok(Vec::new());
            }
            // 选 [1, 手牌数] 名任意存活角色（可含自己，可取消）。
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            let max_n = hand.len().min(u8::MAX as usize) as u8;
            open_choice(
                tx,
                player,
                1,
                max_n,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwdaiguiStage,
                vec![RESOLVE_TWDAIGUI_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 目标选择已确认（resolver 经由 crate::effect 重导出调用）：
/// selected = 「待归」目标（1..=手牌数）。亮出等量牌堆底牌（公开）并按座次
/// 打开第一轮选获。
pub(crate) fn resolve_twdaigui_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    // 目标按座次排序（上游 sortBySeat）。
    let seat: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    let mut targets: Vec<u32> = selected.to_vec();
    targets.sort_by_key(|raw| seat.iter().position(|s| s == raw).unwrap_or(usize::MAX));

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let n = targets.len().min(cards.len());
    if n == 0 {
        return Ok(Vec::new());
    }
    let revealed: Vec<u32> = cards[..n].iter().map(|card| card.raw()).collect();
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::RevealCards {
            cards: revealed.iter().map(|raw| CardId::new(*raw)).collect(),
            to: Vec::new(),
        }],
    )?;
    open_twdaigui_round(tx, player, skill, targets, revealed, Vec::new())?;
    Ok(Vec::new())
}

/// 打开第 r 轮选获（r = picks.len()）。已死亡目标与余牌为空的轮次按上游
/// `continue` 语义跳过；全部轮次完成后余牌归弃牌堆。
pub(crate) fn open_twdaigui_round(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<u32>,
    revealed: Vec<u32>,
    mut picks: Vec<(u32, u32)>,
) -> Result<(), EffectError> {
    let k = targets.len();
    loop {
        let r = picks.len();
        if r >= k {
            return twdaigui_finish(tx, revealed, &picks);
        }
        let chooser_raw = targets[r];
        let chooser = PlayerId::new(chooser_raw);
        if !tx
            .state()
            .players
            .get(chooser)
            .is_some_and(|participant| participant.alive)
        {
            picks.push((chooser_raw, 0));
            continue;
        }
        let picked: std::collections::BTreeSet<u32> = picks.iter().map(|(_, card)| *card).collect();
        let remaining: Vec<u32> = revealed
            .iter()
            .copied()
            .filter(|card| !picked.contains(card))
            .collect();
        if remaining.is_empty() {
            picks.push((chooser_raw, 0));
            continue;
        }
        let payload = twdaigui_round_payload(player, skill, &targets, &revealed, &picks);
        open_choice(
            tx,
            chooser,
            1,
            1,
            false,
            remaining,
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveTwdaiguiStage,
            payload,
        )?;
        return Ok(());
    }
}

/// 轮次选获已确认（resolver 调用）：observer = 当前选获目标，selections[0] =
/// 选获牌。入手后开下一轮（或终局）。
pub(crate) fn resolve_twdaigui_round(
    tx: &mut Transaction,
    observer: PlayerId,
    player: PlayerId,
    skill: engine_api::SkillId,
    payload: &[u32],
    selections: &[u32],
) -> Result<(), EffectError> {
    if payload.len() < 7 || payload[0] != RESOLVE_TWDAIGUI_MAGIC || payload[3] != 3 {
        return Err(EffectError::EngineFault(
            "bad resolve-twdaigui round payload".into(),
        ));
    }
    let k = payload[4] as usize;
    let m = payload[5] as usize;
    if payload.len() < 6 + k + m || (payload.len() - 6 - k - m) % 2 != 0 {
        return Err(EffectError::EngineFault(
            "bad resolve-twdaigui round payload layout".into(),
        ));
    }
    let targets: Vec<u32> = payload[6..6 + k].to_vec();
    let revealed: Vec<u32> = payload[6 + k..6 + k + m].to_vec();
    let picks_rest = &payload[6 + k + m..];
    let mut picks: Vec<(u32, u32)> = picks_rest
        .chunks_exact(2)
        .map(|pair| (pair[0], pair[1]))
        .collect();
    let r = picks.len();
    let chooser_raw = targets.get(r).copied().ok_or_else(|| {
        EffectError::EngineFault("resolve-twdaigui round index out of range".into())
    })?;
    let chooser = PlayerId::new(chooser_raw);
    if observer != chooser {
        return Err(EffectError::Illegal(
            "resolve-twdaigui round chooser mismatch".into(),
        ));
    }
    let picked: std::collections::BTreeSet<u32> = picks.iter().map(|(_, card)| *card).collect();
    let card_raw = *selections
        .first()
        .ok_or_else(|| EffectError::Illegal("resolve-twdaigui pick required".into()))?;
    let revealed_set: std::collections::BTreeSet<u32> = revealed.iter().copied().collect();
    if !revealed_set.contains(&card_raw)
        || picked.contains(&card_raw)
        || !tx.state().players.get(chooser).is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal(
            "resolve-twdaigui pick outside revealed pool".into(),
        ));
    }
    // 选获入手：牌堆 → 选获者手牌。
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, chooser)
        .ok_or_else(|| EffectError::Illegal("no chooser hand".into()))?;
    let card = CardId::new(card_raw);
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = hand;
        card_state.owner = chooser;
    }
    picks.push((chooser_raw, card_raw));
    open_twdaigui_round(tx, player, skill, targets, revealed, picks)?;
    Ok(())
}

/// 终局：余牌（亮出未选获）归弃牌堆。
pub(crate) fn twdaigui_finish(
    tx: &mut Transaction,
    revealed: Vec<u32>,
    picks: &[(u32, u32)],
) -> Result<(), EffectError> {
    let picked: std::collections::BTreeSet<u32> = picks.iter().map(|(_, card)| *card).collect();
    let leftover: Vec<CardId> = revealed
        .iter()
        .copied()
        .filter(|card| !picked.contains(card))
        .map(CardId::new)
        .collect();
    if leftover.is_empty() {
        return Ok(());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::move_cards(tx, leftover, discard, None)?;
    Ok(())
}

/// 专属状态机 (批 249, ADR 0641 §8.41 slice 1)：迷途——准备阶段，你选择至多
/// 3 名角色，这些角色各摸 2 张牌、摸得牌打上「迷途」标记并公开亮出
/// （slice 2 将接续拼点轮与崩溃面）。
pub(crate) fn resolve_twmitu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let _ = count;
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                3,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwmituStage,
                vec![RESOLVE_TWMITU_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 目标选择已确认（resolver 调用）：各目标按座次摸 2 张、打「迷途」标记并
/// 公开亮出，然后进入拼点轮推进（slice 2）。
pub(crate) fn resolve_twmitu_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    let seat: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    let mut targets: Vec<u32> = selected.to_vec();
    targets.sort_by_key(|raw| seat.iter().position(|s| s == raw).unwrap_or(usize::MAX));

    let mut events = Vec::new();
    let mut shown: Vec<CardId> = Vec::new();
    let mut shown_by_target: Vec<u32> = Vec::new();
    for raw in targets.clone() {
        let target = PlayerId::new(raw);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            shown_by_target.push(0);
            continue;
        }
        let drawn = crate::effect::draw_cards(tx, target, 2)?;
        let mut first = 0u32;
        for event in &drawn {
            if let engine_api::DomainEvent::CardsDrawn { cards, .. } = event {
                for card in cards {
                    tx.state_mut().add_content_card_tag(*card, TWMITU_TAG);
                    shown.push(*card);
                    if first == 0 {
                        first = card.raw();
                    }
                }
            }
        }
        shown_by_target.push(first);
        events.extend(drawn);
    }
    if !shown.is_empty() {
        crate::effect::run_effects(
            tx,
            vec![EffectRequest::RevealCards {
                cards: shown,
                to: Vec::new(),
            }],
        )?;
    }
    // slice 2：拼点轮推进（第 0 轮；upstream compare 轮按座次逐目标）。
    twmitu_advance(
        tx,
        player,
        skill,
        &targets,
        &shown_by_target,
        0,
        &[],
        &[],
        &[],
    )?;
    Ok(events)
}

/// 拼点轮载荷：[magic, player, skill, stage, k, done, targets(k), opponents(d),
/// consents(d), pend]——stage 3=owner 选对手；4=目标知情（Custom 1/2）；
/// 5=目标选拼点牌；6=对手选拼点牌。pend 为 5→6 之间暂存的目标牌。
pub(crate) fn twmitu_round_payload(
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    targets: &[u32],
    shown: &[u32],
    done: usize,
    opponents: &[u32],
    consents: &[u32],
    pend: &[u32],
) -> Vec<u32> {
    let mut payload = vec![
        RESOLVE_TWMITU_MAGIC,
        player.raw(),
        skill.raw(),
        stage,
        targets.len() as u32,
        done as u32,
    ];
    payload.extend_from_slice(targets);
    payload.extend_from_slice(shown);
    payload.extend_from_slice(opponents);
    payload.extend_from_slice(consents);
    payload.extend_from_slice(pend);
    payload
}

/// 轮次推进：跳过死亡/无手牌/无合法对手的轮次；done == k 时收口（slice 3
/// 崩溃面将在此接续）。
pub(crate) fn twmitu_advance(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: &[u32],
    shown: &[u32],
    done: usize,
    opponents: &[u32],
    consents: &[u32],
    pend: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let k = targets.len();
    if done >= k {
        return Ok(Vec::new());
    }
    let target_raw = targets[done];
    let target = PlayerId::new(target_raw);
    // 上游 !isIn / canCompare 不可满足 → 跳过该轮。
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != target)
        .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
        .map(PlayerId::raw)
        .collect();
    if !target_alive || tx.state().hand_cards(target).is_empty() || candidates.is_empty() {
        return twmitu_advance(
            tx,
            player,
            skill,
            targets,
            shown,
            done + 1,
            opponents,
            consents,
            pend,
        );
    }
    // stage 3：owner 为该目标指定拼点对手（上游 forced chooseTarget）。
    let payload = twmitu_round_payload(
        player, skill, 3, targets, shown, done, opponents, consents, pend,
    );
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveTwmituStage,
        payload,
    )?;
    Ok(Vec::new())
}
