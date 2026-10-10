//! jsrg 议事（debate）结算原语（ADR 0644 P1）。P1 近似：参与者亮「首张手牌」
//! 为意见（上游 AI 随机），意见牌展示后不移动（与上游收回一致）。

use super::*;
use crate::state::GameState;

/// 将参与者的首张手牌按颜色分组，返回 (红意见者, 黑意见者)。无手牌者不参与。
fn debate_groups_first_card(
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
        let Some(card_id) = state.zones.get(hand).map(|z| z.cards.first().copied()).flatten()
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
    let hand: Vec<u32> = tx.state().hand_cards(player).iter().map(|c| c.raw()).collect();
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
            tx.state_mut()
                .zones
                .get_mut(hand)
                .unwrap()
                .cards
                .push(card);
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
        return Err(EffectError::EngineFault("bad register-debate payload".into()));
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
    let mine = tx.state().player_group(player).map(str::to_string).unwrap_or_default();
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
fn move_card_to_judge_as_xumou(
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
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|p| p.alive)
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
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|p| p.alive)
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
        return Err(EffectError::Illegal("xumou must select exactly one hand card".into()));
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
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|p| p.alive)
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
    let zone = match tx.state().skill_piles.iter().find(|pile| {
        pile.owner == player && pile.skill_def == skill_def_raw
    }) {
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
        tx.state_mut().turn.set_flag(crate::state::turn_flag::SKIP_DRAW, 1);
    }
    if phases_mask & 0b1000 != 0 {
        tx.state_mut().turn.set_flag(crate::state::turn_flag::SKIP_PLAY, 1);
    }
    if phases_mask & 0b1_0000 != 0 {
        tx.state_mut().turn.set_flag(crate::state::turn_flag::SKIP_DISCARD, 1);
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
        return Err(EffectError::Illegal("city-consume must pick one target".into()));
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
    let Some(&city) = tx.state().zones.get(zone_id).and_then(|z| z.cards.last())
    else {
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
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .push(card);
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
        .or_else(|| {
            tx.state()
                .hand_cards(player)
                .first()
                .copied()
        });
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
    let hand: Vec<u32> = tx.state().hand_cards(player).iter().map(|c| c.raw()).collect();
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
        tx.state_mut().debate_pinned_opinions.insert(raw, vec![card]);
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
                EffectRequest::ChangeMaxHp {
                    player,
                    delta: -1,
                },
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
        return Err(EffectError::EngineFault(
            "bad start-debate payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("debate chooser mismatch".into()));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal("cancelled debate has selections".into()));
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
    let cards: Vec<engine_api::CardId> = selections.iter().map(|&s| engine_api::CardId::new(s)).collect();
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
