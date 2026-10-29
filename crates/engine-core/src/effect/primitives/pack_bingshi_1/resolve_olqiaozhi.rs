//! Effect primitives for `resolve_olqiaozhi` and related helpers.

use super::*;

/// 专属状态机 (批 296, ADR 0641 §8.80)：巧织——弃一张牌，亮牌堆顶两张
/// 并获得其中一张（所得打「巧织」标记；手牌有巧织牌时不可发动）。
/// stage 0=弃成本（强制 1..1 he）→ 1=弃置+亮顶 2 选 1 → 2=所得打标入手
/// 牌、未选牌入弃牌堆（上游 cardsGotoOrdering 语义）。
pub(crate) fn resolve_olqiaozhi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 手牌有巧织牌 → 不可发动（上游 filter 的 gaintag 门控）。
            let owns_tagged = tx.state().hand_cards(player).iter().any(|card| {
                tx.state()
                    .content_card_tags
                    .get(card)
                    .is_some_and(|tags| tags.contains(&OLQIAOZHI_TAG))
            });
            if owns_tagged {
                return Ok(Vec::new());
            }
            let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .map(CardId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOlqiaozhiStage,
                vec![RESOLVE_OLQIAOZHI_MAGIC, 1, player.raw()],
            )
        }
        _ => Err(EffectError::ContentFault("bad olqiaozhi stage".into())),
    }
}

/// 专属状态机 (批 302, ADR 0641 §8.86)：dczhee——forced。mode 0=入场/
/// 开局从牌堆顶向下检索；1=每名角色结束阶段从弃牌堆检索。找到第一张
/// 【闪电】且 owner 判定区无闪电 → 挂为判定牌（MoveCards 到 Judge 区，
/// 引擎判定管道自然结算）。canAddJudge 以「判定区无同名延迟锦囊」近似。
pub(crate) fn resolve_dczhee(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let judge = tx
        .state()
        .zones
        .find(ZoneKind::Judge, player)
        .ok_or_else(|| EffectError::Illegal("judge zone missing".into()))?;
    if tx
        .state()
        .zones
        .get(judge)
        .unwrap()
        .cards
        .iter()
        .any(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|c| c.kind == CardKind::TrickShandian)
        })
    {
        return Ok(Vec::new());
    }
    let zone_kind = if mode == 1 {
        ZoneKind::DiscardPile
    } else {
        ZoneKind::DrawPile
    };
    let Some(zone) = tx.state().zones.find(zone_kind, PlayerId::NONE) else {
        return Ok(Vec::new());
    };
    let candidate = tx
        .state()
        .zones
        .get(zone)
        .unwrap()
        .cards
        .iter()
        .rev()
        .copied()
        .find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|c| c.kind == CardKind::TrickShandian)
        });
    let Some(card) = candidate else {
        return Ok(Vec::new());
    };
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card_state| engine_api::ZoneId::new(card_state.zone.raw()));
    move_cards(tx, vec![card], judge, from)
}

/// 专属状态机 (批 304, ADR 0641 §8.88)：张介——死亡时（forceDie 语义，
/// owner 已 dead 仍可发动）选一名其他角色：其 +1 体力上限、回复 1 体力、
/// 摸一张牌。stage 0=选目标（可取消）→ 1=三连增益。
pub(crate) fn resolve_zhongjie(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| *candidate != player)
        .map(PlayerId::raw)
        .collect();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveZhongjieStage,
        vec![RESOLVE_ZHONGJIE_MAGIC, 1, player.raw()],
    )
}

/// 专属状态机 (批 305, ADR 0641 §8.90)：粗遐/未命孪生——forced。
/// ①使用牌者随机弃 1 手牌（RuleRng shuffle 工艺）；②条件仍成立则
/// owner 摸 1（starweiming 反转条件）。单发无续体。
/// 登记债：round 级伤害历史以 lifetime damage_history 放宽。
pub(crate) fn resolve_starcuxia_weiming(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    variant: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_count(source) == 0
    {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    // ① 随机弃 1 手牌。
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, source)
        .ok_or_else(|| EffectError::Illegal("source hand zone missing".into()))?;
    let mut candidates: Vec<CardId> = tx
        .state()
        .zones
        .get(hand)
        .unwrap()
        .cards
        .iter()
        .copied()
        .collect();
    let mut rng = RuleRng::from_state(tx.state().rng);
    rng.shuffle(&mut candidates);
    tx.journal(JournalEntry::RngCheckpoint);
    tx.state_mut().rng = rng.into_state();
    let discarded = candidates[0];
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: source,
            cards: vec![discarded],
        }],
    )?);
    // ② 条件复验（弃牌不影响 hp/伤害历史）→ owner 摸 1。
    let (source_hp, owner_hp) = {
        let s = tx.state();
        (
            s.players.get(source).map(|p| p.hp).unwrap_or(0),
            s.players.get(player).map(|p| p.hp).unwrap_or(0),
        )
    };
    let dominant = if variant == 0 {
        source_hp > owner_hp
    } else {
        owner_hp > source_hp
    };
    let retaliated = tx.state().damage_history.contains(&(source, player));
    if dominant && retaliated {
        events.extend(crate::effect::draw_cards(tx, player, 1)?);
    }
    Ok(events)
}

/// 专属状态机 (批 303, ADR 0641 §8.87)：朝献——限定技（once_per_game）。
/// ①弃牌堆中全部点数为 3 的牌移入牌堆（随机插入以置牌堆底近似，zone
/// 内 reorder）；②依座位序收齐其他存活角色手牌中的 3 点牌（give 语义
/// 移入 owner 手牌区）。单发无续体。
pub(crate) fn resolve_dchaoxian(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = Vec::new();
    // ① 弃牌堆 3 点牌回牌堆底。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let threes: Vec<CardId> = tx
        .state()
        .zones
        .get(discard)
        .unwrap()
        .cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card_state| card_state.rank == 3)
        })
        .collect();
    if !threes.is_empty() {
        let from = tx
            .state()
            .cards
            .get(threes[0])
            .map(|card_state| engine_api::ZoneId::new(card_state.zone.raw()));
        events.extend(move_cards(tx, threes.clone(), draw, from)?);
        // zone 内 reorder：移到 cards 头部 = 牌堆底（上游随机插入近似）。
        let zone_cards = &mut tx.state_mut().zones.get_mut(draw).unwrap().cards;
        let len = zone_cards.len();
        let tail: Vec<CardId> = zone_cards.drain(len - threes.len()..).collect();
        for card in tail.iter().rev() {
            zone_cards.insert(0, *card);
        }
        let _ = zone_cards;
    }
    // ② 逐其他存活角色收 3 点手牌。
    for target in tx.state().players.alive_by_seat() {
        if target == player {
            continue;
        }
        let hand = match tx.state().zones.find(ZoneKind::Hand, target) {
            Some(hand) => hand,
            None => continue,
        };
        let picks: Vec<CardId> = tx
            .state()
            .zones
            .get(hand)
            .unwrap()
            .cards
            .iter()
            .copied()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card_state| card_state.rank == 3)
            })
            .collect();
        if picks.is_empty() {
            continue;
        }
        let from = tx
            .state()
            .cards
            .get(picks[0])
            .map(|card_state| engine_api::ZoneId::new(card_state.zone.raw()));
        let owner_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("owner hand zone missing".into()))?;
        events.extend(run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: picks,
                to: owner_hand,
                from,
            }],
        )?);
    }
    Ok(events)
}

/// 专属状态机 (批 307, ADR 0641 §8.91)：三首——可选（上游 damageBegin4
/// confirm）。亮牌堆顶三张：若存在类型组（基本/锦囊/装备）未被本局任何
/// 使用记录收录的牌，取消本次伤害（PreventDamage，damage_before 窗口内）。
/// 亮出的牌移入弃牌堆（上游 cardsGotoOrdering 收尾近似）。
pub(crate) fn resolve_sanshou(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let mask = tx.state().lifetime_used_type_groups;
    let three: Vec<CardId> = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .copied()
        .take(3)
        .collect();
    if three.is_empty() {
        return Ok(Vec::new());
    }
    let has_new = three.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|c| mask & (1u8 << c.kind.card_type_group().min(2)) == 0)
    });
    let mut events = Vec::new();
    if has_new {
        // 机器侧不走 vm_host 的 damage_id=0 重写，直接读当前伤害 id。
        let damage_id = tx.state().resolution.current_damage_id;
        if damage_id == 0 {
            return Err(EffectError::ContentFault(
                "sanshou prevent outside damage window".into(),
            ));
        }
        events.extend(run_effects(
            tx,
            vec![EffectRequest::PreventDamage { damage_id }],
        )?);
    }
    // 亮出的牌移入弃牌堆（cardsGotoOrdering 收尾近似）。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
    let from = tx
        .state()
        .cards
        .get(three[0])
        .map(|card_state| engine_api::ZoneId::new(card_state.zone.raw()));
    events.extend(move_cards(tx, three, discard, from)?);
    Ok(events)
}

/// 专属状态机 (批 308, ADR 0641 §8.93)：论道——受伤后若伤害来源存活且
/// 手牌数与 owner 不同：owner 手牌多则摸 1；否则 owner 从来源 he 区弃
/// 一张（上游 discardPlayerCard forced，owner 视角选牌）。
/// stage 0=gate+分支 → 摸牌直结 / 开选牌续体 → 1=弃置。
pub(crate) fn resolve_nhlundao(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if source == player
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
        || tx.state().hand_count(source) == tx.state().hand_count(player)
    {
        return Ok(Vec::new());
    }
    if tx.state().hand_count(player) > tx.state().hand_count(source) {
        return crate::effect::draw_cards(tx, player, 1);
    }
    // owner 手牌不多于来源 → owner 从来源 he 区弃一张（强制选牌）。
    let mut options: Vec<u32> = Vec::new();
    for kind in [ZoneKind::Hand, ZoneKind::Equip] {
        if let Some(zone) = tx.state().zones.find(kind, source) {
            options.extend(
                tx.state()
                    .zones
                    .get(zone)
                    .unwrap()
                    .cards
                    .iter()
                    .map(|card| card.raw()),
            );
        }
    }
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveNhlundaoStage,
        vec![RESOLVE_NHLUNDAO_MAGIC, 1, player.raw(), source.raw()],
    )
}

/// 专属状态机 (批 309, ADR 0641 §8.94)：襄戍——限定技（once_per_game）。
/// 结束阶段，若本回合造成过伤害：令一名受伤角色回复 X 点体力并摸 X 张
/// 牌（X=min(5, 本回合造成伤害事件数，damage_dealt_count 机内直读)）。
/// 单发无续体。
pub(crate) fn resolve_xiangshu(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(target) = targets.first().copied() else {
        return Ok(Vec::new());
    };
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let dealt = tx.state().turn.damage_dealt_count(player);
    if dealt == 0 {
        return Ok(Vec::new());
    }
    let num = dealt.min(5);
    let mut events = Vec::new();
    let hp = tx
        .state()
        .players
        .get(target)
        .map(|participant| participant.hp)
        .unwrap_or(0);
    let max_hp = tx
        .state()
        .players
        .get(target)
        .map(|participant| participant.max_hp)
        .unwrap_or(0);
    if (i64::from(max_hp) - i64::from(hp)) > 0 {
        let recover = i64::from(max_hp) - i64::from(hp);
        let amount = u32::try_from(recover.min(i64::from(num))).unwrap_or(0);
        if amount > 0 {
            events.extend(recover_hp(tx, player, target, amount)?);
        }
    }
    events.extend(draw_cards(tx, target, num)?);
    Ok(events)
}

/// 姝勇可选取牌区：手牌+装备+判定（上游 hej）。
pub(crate) fn meiyong_hej_cards(tx: &Transaction, target: PlayerId) -> Vec<CardId> {
    let mut cards = Vec::new();
    for kind in [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge] {
        if let Some(zone) = tx.state().zones.find(kind, target) {
            cards.extend(tx.state().zones.get(zone).unwrap().cards.iter().copied());
        }
    }
    cards
}

/// 专属状态机 (批 277, ADR 0641 §8.65)：姝勇——使用/打出【杀】时，获得一名
/// 其他角色 hej 区的一张牌（owner 选牌），然后其摸 1。
/// stage 0=选目标（可取消）→ 1=选其区域一张牌（强制）→ 2=移牌+目标摸 1。
pub(crate) fn resolve_meiyong(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| *candidate != player && !meiyong_hej_cards(tx, *candidate).is_empty())
        .map(PlayerId::raw)
        .collect();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveMeiyongStage,
        vec![RESOLVE_MEIYONG_MAGIC, 1, player.raw()],
    )
}

/// 专属状态机 (批 278, ADR 0641 §8.66)：羌勇——锁定技，【杀】造成伤害时
/// 弃置受伤角色的 X 张牌（X=本回合 owner 用【杀】次数，SHA_PLAYED 旗），
/// 然后若其没有手牌，此伤害 +1。
/// stage 0=owner 从目标 he 区选 X 张（X>0 且有可弃时）→ 1=弃牌结算；
/// 无可弃或 X=0 直接做 +1 判定。
pub(crate) fn resolve_qiangyong(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 同一伤害只结算一次（引擎可能重复交付 damage_before）。
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id != 0
        && tx
            .state()
            .skills
            .storage_get(skill, RESOLVE_QIANGYONG_LAST_DAMAGE)
            == damage_id
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .storage_set(skill, RESOLVE_QIANGYONG_LAST_DAMAGE, damage_id);
    let x = tx
        .state()
        .turn
        .flag(crate::state::turn_flag::SHA_PLAYED)
        .max(0) as u32;
    if x == 0 {
        qiangyong_bonus_if_no_hand(tx, player, target, skill)?;
        return Ok(Vec::new());
    }
    let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), target)
        .into_iter()
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        qiangyong_bonus_if_no_hand(tx, player, target, skill)?;
        return Ok(Vec::new());
    }
    let count = x.min(options.len() as u32) as u8;
    open_choice(
        tx,
        player,
        count,
        count,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQiangyongStage,
        vec![
            RESOLVE_QIANGYONG_MAGIC,
            1,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
}

/// 目标无手牌 → 当前伤害 +1（上游 trigger.num++）。
pub(crate) fn qiangyong_bonus_if_no_hand(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    if tx.state().hand_count(target) == 0 {
        run_effects(
            tx,
            vec![EffectRequest::ModifyCurrentDamage {
                player,
                delta: 1,
                skill,
            }],
        )?;
    }
    Ok(())
}
