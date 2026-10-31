//! jsrg 议事（debate）结算原语（ADR 0644 P1）。P1 近似：参与者亮「首张手牌」
//! 为意见（上游 AI 随机），意见牌展示后不移动（与上游收回一致）。

use super::*;

/// 多目标授标（ABI 670，jsrg 政略①「至多两名」）：选 1..max_count 名其他存活
/// 角色，各自获得 delta 点标记；cancel = 放弃授标（上游 bool:true 强制——此处
/// 允许放弃为已注记近似）。候选含全部其他存活角色（不检查已有标记者，延续
/// choose_other_add_mark 的 absence-注记）。
pub(crate) fn choose_others_add_mark(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    delta: i32,
    max_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || max_count == 0
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .map(PlayerId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        max_count.min(options.len() as u32).min(u8::MAX as u32) as u8,
        true,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOthersAddMark,
        vec![player.raw(), mark, delta as u32, max_count, skill.raw()],
    )
}

/// 城族消耗（ABI 668，jsrg 沙城②）：移去自己「城」堆最后一张，令一名本回合
/// 失去过牌的其他角色摸 X（X=其本回合失去牌数，至多 5）。无城或无合格目标则
/// 静默 no-op。两阶段续体：confirm 后开放目标选择。
pub(crate) fn city_consume(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let Some(skill_def_raw) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
    else {
        return Err(EffectError::Illegal("bad originating skill".into()));
    };
    let has_city = tx.state().skill_piles.iter().any(|pile| {
        pile.owner == player
            && pile.skill_def == skill_def_raw
            && tx
                .state()
                .zones
                .get(pile.zone)
                .is_some_and(|zone| !zone.cards.is_empty())
    });
    if !has_city {
        return Ok(Vec::new());
    }
    let eligible: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .filter(|&target| {
            tx.state()
                .turn
                .flag(crate::state::turn_flag::lost_card_count_by_player(target))
                > 0
        })
        .map(PlayerId::raw)
        .collect();
    if eligible.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        eligible,
        ChoiceKind::SelectTargets,
        ContinuationKind::CityConsume,
        vec![player.raw(), skill.raw()],
    )
}

/// Cont 280 resolver（政略①多目标授标）：对每个选中目标各施加 delta。
pub(crate) fn resolve_choose_others_add_mark(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, mark, delta, max_count, _skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad choose-others-add-mark payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let delta = *delta as i32;
    let max_count = *max_count;
    let mark_id = engine_api::MarkDefId::new(*mark);
    if observer != player {
        return Err(EffectError::Illegal(
            "choose-others-add-mark chooser mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.is_empty() || selections.len() > max_count as usize {
        return Err(EffectError::Illegal(
            "choose-others-add-mark target count exceeds max".into(),
        ));
    }
    let mut effects = Vec::new();
    for raw in selections {
        let target = PlayerId::new(*raw);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("bad multi add-mark target".into()));
        }
        effects.push(EffectRequest::AddMark {
            player: target,
            mark: mark_id,
            delta,
        });
    }
    run_effects(tx, effects)?;
    Ok(())
}

/// Cont 279 resolver（沙城②消耗）：移去「城」堆最后一张入弃牌堆，并令选中目标
/// 摸 X 张（X=min(5, 其本回合失去过的牌数)）；cancelled = 不发动。
pub(crate) fn resolve_city_consume(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault("bad city-consume payload".into()));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("city-consume chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "city-consume must pick one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    let Some(skill_def_raw) = tx
        .state()
        .skills
        .get(engine_api::SkillId::new(*skill))
        .map(|instance| instance.def.raw())
    else {
        return Err(EffectError::EngineFault("bad originating skill".into()));
    };
    let pile_zone = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == skill_def_raw)
        .map(|pile| pile.zone);
    let Some(zone_id) = pile_zone else {
        return Err(EffectError::Illegal("missing city pile".into()));
    };
    let Some(&city) = tx.state().zones.get(zone_id).and_then(|z| z.cards.last()) else {
        return Err(EffectError::Illegal("city pile is empty".into()));
    };
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let lost = tx
        .state()
        .turn
        .flag(crate::state::turn_flag::lost_card_count_by_player(target));
    let draw = lost.min(5);
    let mut effects = vec![EffectRequest::MoveCards {
        cards: vec![city],
        to: discard,
        from: Some(zone_id),
    }];
    if draw > 0 {
        effects.push(EffectRequest::DrawCards {
            player: target,
            count: draw as u32,
        });
    }
    run_effects(tx, effects)?;
    Ok(())
}

/// 变更势力（ABI 661，势力族）：写入 group_overrides（既有势力覆盖机制，
/// 先于角色定义组——jinfa/延祚/邀虎/挟雄 4+ 消费者）。
pub(crate) fn change_group(
    tx: &mut Transaction,
    player: PlayerId,
    group: String,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || group.is_empty() {
        return Ok(Vec::new());
    }
    if !tx.state().players.get(player).is_some_and(|p| p.alive) {
        return Ok(Vec::new());
    }
    tx.state_mut().group_overrides.insert(player, group);
    Ok(Vec::new())
}

/// 获得【影】（ABI 660，jsrg 影族）：生成 count 张影牌（黑桃 1 虚拟
/// 基本牌 CardKind::Ying）置入手牌。
pub(crate) fn gain_shadow_cards(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || count == 0 {
        return Ok(Vec::new());
    }
    if !tx.state().players.get(player).is_some_and(|p| p.alive) {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("bad hand zone".into()))?;
    for _ in 0..count {
        let card = tx.state_mut().cards.alloc(Card {
            id: engine_api::CardId::NONE,
            def: engine_api::CardDefId::new(0),
            kind: crate::state::CardKind::Ying,
            suit: engine_api::Suit::Spade,
            rank: 1,
            weapon_range: 0,
            zone: hand,
            owner: player,
            face_up: true,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    }
    Ok(Vec::new())
}

/// 议事意见对齐（ABI 659，ADR 0644 P3 霸政）：议事窗口中，本回合受到过
/// player 伤害的参与者意见对齐为 player 的意见（钉死为 player 的意见牌）。
pub(crate) fn align_debate_opinions(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .debate_pending_participants
        .contains(&player.raw())
    {
        return Ok(Vec::new());
    }
    // 参考意见牌：已钉死意见取首张；否则首张手牌；无手牌不生效。
    let reference = tx
        .state()
        .debate_pinned_opinions
        .get(&player.raw())
        .and_then(|cards| cards.first().copied())
        .or_else(|| tx.state().hand_cards(player).first().copied());
    let Some(reference) = reference else {
        return Ok(Vec::new());
    };
    let bits = tx
        .state()
        .turn
        .flag(crate::state::turn_flag::damaged_by_player(player)) as u32;
    if bits == 0 {
        return Ok(Vec::new());
    }
    let participants: Vec<u32> = tx.state().debate_pending_participants.clone();
    for raw in participants {
        if raw == player.raw() {
            continue;
        }
        let other = PlayerId::new(raw);
        let Some(seat) = tx.state().players.get(other).map(|p| p.seat.raw()) else {
            continue;
        };
        if bits & (1 << seat) == 0 {
            continue;
        }
        tx.state_mut()
            .debate_pinned_opinions
            .insert(raw, vec![reference]);
    }
    Ok(Vec::new())
}

/// Cont 274 resolver：钉死自选双意见 + 他人随机意见，随后收尾议事结算。
pub(crate) fn resolve_pin_debate_opinions(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, _skill] = payload else {
        return Err(EffectError::EngineFault("bad pin-debate payload".into()));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("pin-debate chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 2 {
        return Err(EffectError::Illegal(
            "pin-debate expects exactly two cards".into(),
        ));
    }
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|c| c.raw())
        .collect();
    for &card in selections {
        if !hand.contains(&card) {
            return Err(EffectError::Illegal("pin-debate card not in hand".into()));
        }
    }
    tx.state_mut().debate_pinned_opinions.insert(
        player.raw(),
        selections
            .iter()
            .map(|&raw| engine_api::CardId::new(raw))
            .collect(),
    );
    // 其他有手牌参与者：随机钉死一张（确定性 rng）。
    let others: Vec<u32> = tx
        .state()
        .debate_pending_participants
        .iter()
        .copied()
        .filter(|&raw| raw != player.raw())
        .collect();
    for raw in others {
        let other = PlayerId::new(raw);
        let hand = tx.state().hand_cards(other);
        if hand.is_empty() {
            continue;
        }
        let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
        let index = (rng.next_u64() % hand.len() as u64) as usize;
        tx.state_mut().rng = rng.into_state();
        let card = hand[index];
        tx.state_mut()
            .debate_pinned_opinions
            .insert(raw, vec![card]);
    }
    finish_debate(tx).map(|_| ())
}

/// 朝争议事（ABI 653）：准备阶段可选发动。所有其他有手牌角色亮首张手牌为
/// 意见→红黑分组→多数决：红=红意见者各回复 1；黑=红意见者各失去 1 点体力；
/// 全体意见一致（或仅一个颜色且覆盖全部参与者）时发起者摸参与者数张牌。
pub(crate) fn resolve_jsrgchaozheng(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    // 上游 chooseToDebate：仅「有手牌」的参与者亮牌；无手牌者不属于议事目标。
    let candidates: Vec<PlayerId> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&other| other != player)
        .collect();
    let (red, _black) = debate_groups_first_card(tx.state(), &candidates);
    let participants = red.len() + _black.len();
    let red_count = red.len();
    let black_count = _black.len();
    let result: u8 = if red_count > black_count {
        1
    } else if black_count > red_count {
        2
    } else {
        0
    };
    let mut effects: Vec<EffectRequest> = Vec::new();
    {
        // 多数决（平票无结果）；结算对象为红意见者。单色即该色多数。
        if red_count > black_count {
            for target in &red {
                effects.push(EffectRequest::RecoverHp {
                    source: player,
                    player: *target,
                    amount: 1,
                });
            }
        } else if black_count > red_count {
            for target in &red {
                effects.push(EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: *target,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                });
            }
        }
    }
    // 全体意见一致（单色覆盖全部参与者）→ 发起者摸参与者数张。
    if participants > 0 && (red_count == participants || black_count == participants) {
        effects.push(EffectRequest::DrawCards {
            player,
            count: participants as u32,
        });
    }
    let debate_event = DomainEvent::DebateResolved {
        initiator: player,
        red: red.clone(),
        black: _black.clone(),
        result,
    };
    let settled = run_effects(tx, effects)?;
    let mut events = settled;
    events.push(debate_event);
    Ok(events)
}

/// 发起议事·全员（ABI 655，ADR 0644 P2）：所有其他有手牌角色亮首张手牌分组，
/// 只发 DebateResolved 事件——结算分支由反应型技能在 debate 触发中表达。
pub(crate) fn start_debate_all_others(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let candidates: Vec<PlayerId> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&other| other != player)
        .collect();
    open_debate_window(tx, player, candidates)
}

/// 设居双人议事（ABI 654）：用【杀】指定唯一目标后（或成为其唯一目标后），
/// 与对方议事：双方亮牌，结果为黑 → 双方各减 1 点体力上限。
pub(crate) fn resolve_jsrgsheju(
    tx: &mut Transaction,
    player: PlayerId,
    counterpart: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || counterpart == PlayerId::NONE
        || counterpart == player
    {
        return Ok(Vec::new());
    }
    for id in [player, counterpart] {
        if !tx
            .state()
            .players
            .get(id)
            .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
    }
    let participants = [player, counterpart];
    let (red, black) = debate_groups_first_card(tx.state(), &participants);
    // 多数决：黑多数 → 双方各减 1 体力上限（红多数/平票/任一方无牌无效果）。
    if black.len() > red.len() {
        return run_effects(
            tx,
            vec![
                EffectRequest::ChangeMaxHp { player, delta: -1 },
                EffectRequest::ChangeMaxHp {
                    player: counterpart,
                    delta: -1,
                },
            ],
        );
    }
    Ok(Vec::new())
}

/// 发起议事·指定目标（ABI 656，ADR 0644 P2）：选择 1..N 名其他角色（Cont 272），
/// 选中者亮首张手牌分组后发 DebateResolved 事件。
pub(crate) fn start_debate_targets(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&other| other != player)
        .map(PlayerId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        options.len() as u8,
        true,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::StartDebateTargets,
        vec![player.raw(), skill.raw()],
    )
}

/// Cont 272 resolver：对选中目标分组并发 DebateResolved。
pub(crate) fn resolve_start_debate_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault("bad start-debate payload".into()));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("debate chooser mismatch".into()));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled debate has selections".into(),
            ));
        }
        return Ok(());
    }
    let targets: Vec<PlayerId> = selections.iter().map(|&s| PlayerId::new(s)).collect();
    for t in &targets {
        if t == &player
            || !tx
                .state()
                .players
                .get(*t)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("bad debate target".into()));
        }
    }
    open_debate_window(tx, player, targets).map(|_| ())
}

/// 玉蝉引擎（ABI 657，g_yuchan_equip）：出牌阶段限一次，弃置任意张（≥1）基本
/// 手牌并摸等量的牌（选择续体 Cont 273）。
pub(crate) fn discard_basic_draw_equal(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    let basics: Vec<u32> = tx
        .state()
        .zones
        .get(hand)
        .map(|zone| {
            zone.cards
                .iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(**card)
                        .is_some_and(|card| card.kind.is_basic())
                })
                .map(|card| card.raw())
                .collect()
        })
        .unwrap_or_default();
    if basics.is_empty() {
        return Ok(Vec::new());
    }
    let max = u8::try_from(basics.len()).unwrap_or(u8::MAX);
    open_choice(
        tx,
        player,
        1,
        max,
        true,
        basics,
        ChoiceKind::SelectCards,
        ContinuationKind::DiscardBasicDrawEqual,
        vec![player.raw(), skill.raw()],
    )
}

/// Cont 273 resolver：弃所选基本牌并摸等量。
pub(crate) fn resolve_discard_basic_draw_equal(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, _skill] = payload else {
        return Err(EffectError::EngineFault("bad yuchan payload".into()));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("yuchan chooser mismatch".into()));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let cards: Vec<engine_api::CardId> = selections
        .iter()
        .map(|&s| engine_api::CardId::new(s))
        .collect();
    for card in &cards {
        let is_basic = tx
            .state()
            .cards
            .get(*card)
            .is_some_and(|card| card.kind.is_basic());
        let in_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| zone.cards.contains(card));
        if !is_basic || !in_hand {
            return Err(EffectError::Illegal("yuchan card not legal".into()));
        }
    }
    let count = cards.len() as u32;
    run_effects(
        tx,
        vec![
            EffectRequest::DiscardCards { player, cards },
            EffectRequest::DrawCards { player, count },
        ],
    )
    .map(|_| ())
}
