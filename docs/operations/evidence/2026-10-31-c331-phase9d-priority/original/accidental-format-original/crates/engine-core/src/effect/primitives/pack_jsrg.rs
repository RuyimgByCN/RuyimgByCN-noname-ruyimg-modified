//! jsrg 议事（debate）结算原语（ADR 0644 P1）。P1 近似：参与者亮「首张手牌」
//! 为意见（上游 AI 随机），意见牌展示后不移动（与上游收回一致）。

use super::*;
use crate::state::GameState;

/// 将参与者的首张手牌按颜色分组，返回 (红意见者, 黑意见者)。无手牌者不参与。
pub(super) fn debate_groups_first_card(
    state: &GameState,
    participants: &[PlayerId],
) -> (Vec<PlayerId>, Vec<PlayerId>) {
    let mut red = Vec::new();
    let mut black = Vec::new();
    for &participant in participants {
        // ADR 0644 P3：被钉死意见优先（多张=多票，jsrgxiongbao 双意见）。
        if let Some(pinned) = state.debate_pinned_opinions.get(&participant.raw()) {
            let mut voted = false;
            for &card_id in pinned {
                if let Some(card) = state.cards.get(card_id) {
                    if card.suit == Suit::Heart || card.suit == Suit::Diamond {
                        red.push(participant);
                    } else {
                        black.push(participant);
                    }
                    voted = true;
                }
            }
            if voted {
                continue;
            }
        }
        let Some(hand) = state.zones.find(ZoneKind::Hand, participant) else {
            continue;
        };
        let Some(card_id) = state
            .zones
            .get(hand)
            .map(|z| z.cards.first().copied())
            .flatten()
        else {
            continue;
        };
        let Some(card) = state.cards.get(card_id) else {
            continue;
        };
        if card.suit == Suit::Heart || card.suit == Suit::Diamond {
            red.push(participant);
        } else {
            black.push(participant);
        }
    }
    (red, black)
}

/// ADR 0644 P3：议事开始窗口（debateShowOpinion 时机）——登记待议事名单，
/// 派发 DebateStarted 供 pin_debate_opinions 类技能钉死意见；无技能应答时
/// 立即结算，有选择挂起时由 Cont 274 resolver 收尾。
pub(crate) fn open_debate_window(
    tx: &mut Transaction,
    initiator: PlayerId,
    participants: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    tx.state_mut().debate_pending_participants = participants.iter().map(|p| p.raw()).collect();
    tx.state_mut().debate_pending_initiator = initiator.raw();
    let planned = crate::trigger::dispatch_for_events(
        tx,
        &[DomainEvent::DebateStarted {
            initiator,
            participants,
        }],
        &mut crate::trigger::TriggerFireRecord::new(),
    );
    match planned {
        Ok(effects) => {
            if effects.is_empty() {
                return finish_debate(tx);
            }
            // 钉死原语等效果开选择时挂起（Cont 274 收尾）；否则收尾结算。
            match crate::effect::run_effects(tx, effects) {
                Ok(_) => finish_debate(tx),
                Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
                Err(error) => Err(error),
            }
        }
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Err(error) => Err(error),
    }
}

/// ADR 0644 P3：按钉死意见结算待议事并发 DebateResolved，清理钉死状态。
pub(crate) fn finish_debate(tx: &mut Transaction) -> Result<Vec<DomainEvent>, EffectError> {
    let initiator = PlayerId::new(tx.state().debate_pending_initiator);
    let participants: Vec<PlayerId> = tx
        .state()
        .debate_pending_participants
        .iter()
        .map(|&raw| PlayerId::new(raw))
        .collect();
    tx.state_mut().debate_pinned_opinions.clear();
    tx.state_mut().debate_pending_participants.clear();
    tx.state_mut().debate_pending_initiator = 0;
    let (red, black) = debate_groups_first_card(tx.state(), &participants);
    let result: u8 = if red.len() > black.len() {
        1
    } else if black.len() > red.len() {
        2
    } else {
        0
    };
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::EmitDebateResolved {
            initiator,
            red,
            black,
            result,
        }],
    )
}

/// 议事意见钉死（ABI 658，ADR 0644 P3）：议事开始窗口中，参与者自选
/// 2 张手牌为双意见；其他有手牌参与者意见随机钉死一张（Cont 274）。
pub(crate) fn pin_debate_opinions(
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
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|c| c.raw())
        .collect();
    if hand.len() < 2 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        2,
        2,
        true,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::PinDebateOpinions,
        vec![player.raw(), skill.raw()],
    )
}

/// 分发【影】（ABI 662，jsrg funi②）：选 1..N 角色，共计 count 张影
/// 均分给予（Cont 275）。
pub(crate) fn distribute_shadow_cards(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || count == 0 {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
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
        ContinuationKind::DistributeShadowCards,
        vec![player.raw(), count, skill.raw()],
    )
}

/// Cont 275 resolver：将 count 张影均分给选中角色（余数给首目标）。
pub(crate) fn resolve_distribute_shadow_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, count, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad distribute-shadows payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("distribute chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let count = *count;
    let targets: Vec<PlayerId> = selections.iter().map(|&s| PlayerId::new(s)).collect();
    if targets.is_empty() {
        return Ok(());
    }
    let base = count / targets.len() as u32;
    let rem = count % targets.len() as u32;
    for (i, target) in targets.iter().enumerate() {
        let n = base + if (i as u32) < rem { 1 } else { 0 };
        if n == 0 {
            continue;
        }
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, *target)
            .ok_or_else(|| EffectError::Illegal("bad hand zone".into()))?;
        for _ in 0..n {
            let card = tx.state_mut().cards.alloc(Card {
                id: engine_api::CardId::NONE,
                def: engine_api::CardDefId::new(0),
                kind: crate::state::CardKind::Ying,
                suit: engine_api::Suit::Spade,
                rank: 1,
                weapon_range: 0,
                zone: hand,
                owner: *target,
                face_up: true,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        }
    }
    let _ = skill;
    Ok(())
}

/// 登记回合末议事（ABI 663，jsrg yaoyan）：逐角色选择是否参加，登记
/// 同意者为发起者的回合末议事参与者（Cont 276）。
pub(crate) fn register_turn_end_debate(
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
        .map(PlayerId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        0,
        options.len() as u8,
        true,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::RegisterTurnEndDebate,
        vec![player.raw(), skill.raw()],
    )
}

/// Cont 276 resolver：登记选中参与者为回合末议事名单。
pub(crate) fn resolve_register_turn_end_debate(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad register-debate payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("register chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let list: Vec<u32> = selections.iter().map(|&s| s).collect();
    tx.state_mut()
        .pending_turn_end_debates
        .insert(player.raw(), list);
    let _ = skill;
    Ok(())
}

/// 回合末议事结算（ABI 663）：对登记名单发起议事并收尾。
pub(crate) fn resolve_turn_end_debates(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let list: Vec<PlayerId> = tx
        .state_mut()
        .pending_turn_end_debates
        .remove(&player.raw())
        .unwrap_or_default()
        .into_iter()
        .map(PlayerId::new)
        .collect();
    if list.is_empty() {
        return Ok(Vec::new());
    }
    open_debate_window(tx, player, list)
}

/// 选择势力并变更（ABI 664，势力族动态）：选现存在且不同于自己的势力
/// （Cont 277）。
pub(crate) fn choose_group(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mine = tx
        .state()
        .player_group(player)
        .map(str::to_string)
        .unwrap_or_default();
    let options: Vec<u32> = ["wei", "shu", "wu", "qun", "jin"]
        .iter()
        .enumerate()
        .filter(|(_, g)| **g != mine)
        .filter(|(_, g)| {
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .any(|p| tx.state().player_group(p).map(str::to_string) == Some(g.to_string()))
        })
        .map(|(i, _)| i as u32)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ChooseGroup,
        vec![player.raw(), skill.raw()],
    )
}

/// Cont 277 resolver：将选择势力写入 group_overrides。
pub(crate) fn resolve_choose_group(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault("bad choose-group payload".into()));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("choose-group chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let Some(&code) = selections.first() else {
        return Ok(());
    };
    let group = ["wei", "shu", "wu", "qun", "jin"]
        .get(code as usize)
        .map(|s| s.to_string())
        .unwrap_or_default();
    if group.is_empty() {
        return Ok(());
    }
    tx.state_mut().group_overrides.insert(player, group);
    let _ = skill;
    Ok(())
}

/// 蓄谋（ABI 665，jsrg 蓄谋族切片 1）：将一张牌作为蓄谋牌置入判定区
/// （特殊延时虚拟牌，可重复——上游 addJudge({name:"xumou_jsrg"})）。
pub(crate) fn place_xumou(
    tx: &mut Transaction,
    player: PlayerId,
    card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let Some(source) = tx.state().cards.get(card) else {
        return Ok(Vec::new());
    };
    let (def, suit, rank) = (source.def, source.suit, source.rank);
    let judge = tx
        .state()
        .zones
        .find(ZoneKind::Judge, player)
        .ok_or_else(|| EffectError::Illegal("bad judge zone".into()))?;
    let placed = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def,
        kind: crate::state::CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone: judge,
        owner: player,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(judge)
        .unwrap()
        .cards
        .push(placed);
    tx.state_mut()
        .content_card_tags
        .entry(placed)
        .or_default()
        .insert(crate::content::XUMOU_TAG);
    Ok(Vec::new())
}

/// 将一张实体牌移入判定区并打上蓄谋标记（真实移动语义：原牌离开原区，
/// 与上游 addJudge 移动实体一致）。
pub(super) fn move_card_to_judge_as_xumou(
    tx: &mut Transaction,
    player: PlayerId,
    card: engine_api::CardId,
) -> Result<(), EffectError> {
    let judge = tx
        .state()
        .zones
        .find(ZoneKind::Judge, player)
        .ok_or_else(|| EffectError::Illegal("bad judge zone".into()))?;
    // 从当前所在区移除。
    if let Some(zone_id) = tx.state().cards.get(card).map(|c| c.zone) {
        if zone_id != ZoneId::NONE {
            if let Some(zone) = tx.state_mut().zones.get_mut(zone_id) {
                zone.cards.retain(|c| *c != card);
            }
        }
    }
    if let Some(c) = tx.state_mut().cards.get_mut(card) {
        c.zone = judge;
        c.owner = player;
        c.face_up = true;
    }
    tx.state_mut()
        .zones
        .get_mut(judge)
        .ok_or_else(|| EffectError::Illegal("bad judge zone".into()))?
        .cards
        .push(card);
    tx.state_mut()
        .content_card_tags
        .entry(card)
        .or_default()
        .insert(crate::content::XUMOU_TAG);
    Ok(())
}

/// 蓄谋·牌堆顶（ABI 666，jsrg 殆谋「他人」分支）：将牌堆顶实体牌移入判定区
/// 作为蓄谋牌（上游 addJudge({name:"xumou_jsrg"}, get.cards())）。牌堆空则 no-op。
pub(crate) fn place_deck_top_xumou(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    let Some(&top) = tx.state().zones.get(draw).and_then(|z| z.cards.last()) else {
        return Ok(Vec::new());
    };
    move_card_to_judge_as_xumou(tx, player, top)?;
    Ok(Vec::new())
}

/// 蓄谋·选牌（ABI 666，jsrg 遏前①）：选一张手牌移入判定区作为蓄谋牌；
/// cancel = 停止蓄谋（Cont 278 两阶段取 stage 0 即终态）。
pub(crate) fn choose_place_hand_xumou(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let options = tx.state().hand_cards(player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChoosePlaceHandXumou,
        vec![player.raw(), skill.raw()],
    )
}

/// Cont 278 resolver（蓄谋·选牌）：把选中的一张手牌真实移入判定区并打
/// XUMOU_TAG；cancel = 停止蓄谋。
pub(crate) fn resolve_choose_place_hand_xumou(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault("bad xumou-hand payload".into()));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal("xumou-hand chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "xumou must select exactly one hand card".into(),
        ));
    }
    let card = CardId::new(selections[0]);
    if !tx.state().hand_cards(player).contains(&card) {
        return Err(EffectError::Illegal(
            "xumou card is no longer in the owner's hand".into(),
        ));
    }
    move_card_to_judge_as_xumou(tx, player, card)
}

/// 弃置一张蓄谋牌（ABI 666，jsrg 殆谋「我」分支）：确定性弃置判定区最后一张
/// 蓄谋牌（无 tagged 牌则静默 no-op——上游 filter 已保证有牌时才触发）。
pub(crate) fn discard_one_own_xumou(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let judge = tx
        .state()
        .zones
        .find(ZoneKind::Judge, player)
        .ok_or_else(|| EffectError::Illegal("bad judge zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let target = tx
        .state()
        .zones
        .get(judge)
        .map(|z| z.cards.clone())
        .unwrap_or_default()
        .into_iter()
        .rev()
        .find(|c| {
            tx.state()
                .content_card_tags
                .get(c)
                .is_some_and(|tags| tags.contains(&crate::content::XUMOU_TAG))
        });
    let Some(card) = target else {
        return Ok(Vec::new());
    };
    super::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(judge),
        }],
    )
}

/// 筑城·堆顶置入扩张区（ABI 667，jsrg 沙城 build 子技）：将牌堆顶 count 张实体牌
/// 明置移入自己的「城」扩张堆（上游 addToExpansion(get.cards(2),"gain2")）。
/// 扩张区按需创建并以本技能 def 作键（skill_piles 惯例，bingshi 同工艺）。
pub(crate) fn place_deck_top_expansion(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    let Some(skill_def_raw) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
    else {
        return Err(EffectError::Illegal("bad originating skill".into()));
    };
    // create-or-find the owner's expansion pile keyed by this skill def.
    let zone = match tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == skill_def_raw)
    {
        Some(pile_ref) => pile_ref.zone,
        None => {
            let zid = tx.state_mut().zones.alloc(crate::state::Zone {
                id: ZoneId::NONE,
                kind: ZoneKind::Expansion,
                owner: player,
                cards: Vec::new(),
            });
            tx.state_mut().skill_piles.push(crate::state::SkillPileRef {
                owner: player,
                skill_def: skill_def_raw,
                zone: zid,
                usable_as_hand: false,
                wants_lost_count: true,
            });
            tx.state_mut()
                .skill_piles
                .sort_by_key(|pile| (pile.owner.raw(), pile.skill_def));
            zid
        }
    };
    for _ in 0..count {
        let Some(&top) = tx.state().zones.get(draw).and_then(|z| z.cards.last()) else {
            break;
        };
        if let Some(zone_state) = tx.state_mut().zones.get_mut(draw) {
            zone_state.cards.pop();
        }
        if let Some(c) = tx.state_mut().cards.get_mut(top) {
            c.zone = zone;
            c.owner = player;
            c.face_up = true;
        }
        tx.state_mut()
            .zones
            .get_mut(zone)
            .ok_or_else(|| EffectError::Illegal("bad expansion zone".into()))?
            .cards
            .push(top);
    }
    Ok(Vec::new())
}

/// 额外回合（ABI 669，猎族 S3，jsrgpingrong insertPhase）：把发起者排入引擎
/// 既有 extra_turn_queue（消费管线在 command/turn.rs continue_turn_advance），
/// 并登记尾随 check 标记——该额外回合结束时若 DAMAGE_DEALT==0 则失去 1 点体力
/// （check 结算同样位于 continue_turn_advance 头部）。
/// 相位裁剪额外回合（ABI 672，§6 S0）：把玩家排入裁剪队列，仅执行
/// phases_mask 置位阶段（bit = 1 << TurnPhaseTag 判别值）。
/// 跳过本回合剩余阶段（ABI 674，§6 S2）：对 mask 置位的 Draw/Play/Discard
/// 阶段置 SKIP 标志——既有阶段推进流程自动跳过（olpianan 先例）。
pub(crate) fn skip_remaining_phases(
    tx: &mut Transaction,
    player: PlayerId,
    phases_mask: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx.state().players.get(player).is_some_and(|p| p.alive) {
        return Ok(Vec::new());
    }
    if phases_mask & 0b0100 != 0 {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::SKIP_DRAW, 1);
    }
    if phases_mask & 0b1000 != 0 {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::SKIP_PLAY, 1);
    }
    if phases_mask & 0b1_0000 != 0 {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::SKIP_DISCARD, 1);
    }
    Ok(Vec::new())
}

pub(crate) fn queue_extra_phases(
    tx: &mut Transaction,
    player: PlayerId,
    phases_mask: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx.state().players.get(player).is_some_and(|p| p.alive) {
        return Ok(Vec::new());
    }
    if tx.state().turn.extra_phase_queue.len() >= 8 {
        return Err(EffectError::Illegal(
            "extra phase queue overflow (max 8)".into(),
        ));
    }
    tx.state_mut()
        .turn
        .extra_phase_queue
        .push(crate::state::ExtraPhaseTurn {
            player,
            phases: (1u8..=5u8)
                .filter(|tag| phases_mask & (1 << tag) != 0)
                .collect(),
            no_turn_over: false,
        });
    Ok(Vec::new())
}

pub(crate) fn queue_extra_turn(
    tx: &mut Transaction,
    player: PlayerId,
    check: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    if tx.state().turn.extra_turn_queue.len() >= 8 {
        return Err(EffectError::Illegal(
            "extra turn queue overflow (max 8)".into(),
        ));
    }
    let _ = engine_api::SkillDefId::new(0); // keep SkillDefId import parity
    tx.state_mut().turn.extra_turn_queue.push(player);
    if check != 0 {
        tx.state_mut().turn.set_flag(
            crate::state::turn_flag::extra_turn_check_pending_by_player(player),
            1,
        );
    }
    Ok(Vec::new())
}
