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

/// 专属状态机 (批 276, ADR 0641 §8.64)：镇南——成为其他角色使用的锦囊目标
/// （目标数 > max(1, 使用者体力)）时，可弃一张牌并对任意角色造成 1 点伤害。
/// stage 0=选成本牌（可取消）→ 1=弃牌+选伤害目标（任意存活角色）→
/// 2=弃牌结算+伤害。
pub(crate) fn resolve_mbzhenann(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
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
        ContinuationKind::ResolveMbzhenannStage,
        vec![RESOLVE_MBZHENANN_MAGIC, 1, player.raw()],
    )
}

/// 专属状态机 (批 273, ADR 0641 §8.61)：暗蓄——出牌阶段限一次，选两名手牌
/// 数不同的角色，手牌少者从多者处选得一张手牌；所得牌非黑桃则 owner 摸 1，
/// 此后两人手牌数相等则 owner 回 1。stage 0=少者从多者手牌选 1 →
/// 1=移牌+花色/相等结算。
pub(crate) fn resolve_dcanxu(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    if targets.len() != 2 || targets[0] == targets[1] {
        return Err(EffectError::Illegal(
            "resolve-dcanxu requires exactly two distinct targets".into(),
        ));
    }
    let (a, b) = (targets[0], targets[1]);
    let (a_count, b_count) = (tx.state().hand_count(a), tx.state().hand_count(b));
    if a_count == b_count {
        return Err(EffectError::Illegal(
            "resolve-dcanxu targets must differ in hand count".into(),
        ));
    }
    let (gainner, giver) = if a_count < b_count { (a, b) } else { (b, a) };
    let options: Vec<u32> = tx
        .state()
        .hand_cards(giver)
        .iter()
        .map(|card| card.raw())
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        gainner,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDcanxuStage,
        vec![
            RESOLVE_DCANXU_MAGIC,
            player.raw(),
            gainner.raw(),
            giver.raw(),
            1,
        ],
    )
}

/// 专属状态机 (批 272, ADR 0641 §8.60)：度断——每回合一次，成为【杀】目标
/// 时可重铸一张牌；再选一项：①使用者摸 2 且此杀对 owner 无效；②使用者弃
/// 一张牌且 owner 不可响应此杀。stage 0=选重铸牌 → 1=重铸+双选（使用者无
/// 可弃牌时强制①）→ 2=分支执行/选使用者弃牌 → 3=使用者弃牌+不可响应。
pub(crate) fn resolve_duoduan(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
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
        ContinuationKind::ResolveDuoduanStage,
        vec![
            RESOLVE_DUODUAN_MAGIC,
            player.raw(),
            source.raw(),
            card.raw(),
            skill.raw(),
            1,
        ],
    )
}

/// 选项①：使用者摸 2 张，此【杀】对 owner 无效。排除在解析期内同步生效
/// （直接改 sha 挂起载荷；经 CancelIncomingCardUse 尾随提交会在响应询问
/// 之后才落地，且附带「获得实体牌」的杜达语义，与度断不符）。
pub(crate) fn resolve_duoduan_draw_and_cancel(
    tx: &mut Transaction,
    source: PlayerId,
    player: PlayerId,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = crate::effect::draw_cards(tx, source, 2)?;
    crate::command::play::exclude_saved_card_target(tx, card, player)?;
    Ok(events)
}

/// 告援候选：非 owner、非使用者的存活角色，持「诤」标，且为此【杀】在
/// 使用者射程内的合法目标（转移语义 targetEnabled）。
pub(crate) fn gaoyuan_redirect_candidates(
    tx: &Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
) -> Result<Vec<PlayerId>, EffectError> {
    let legal = crate::command::play::saved_sha_redirect_targets_in_attack_range(
        tx, source, card, player, source,
    )?;
    Ok(legal
        .into_iter()
        .filter(|candidate| {
            *candidate != source
                && tx
                    .state()
                    .players
                    .get(*candidate)
                    .is_some_and(|participant| {
                        participant.marks.iter().any(|(mark, count)| {
                            *mark == crate::effect::ZHENGJIAN_MARK && *count > 0
                        })
                    })
        })
        .collect())
}

/// 专属状态机 (批 271, ADR 0641 §8.59)：告援——成为【杀】目标时，可弃一张
/// 牌，将此【杀】转移给一名有「诤」标且为此杀合法目标的其他角色。
/// stage 0：选弃牌（0..1 可取消）→ stage 1：选转移目标（1..1 可取消）→
/// stage 2：弃牌 + redirect_saved_sha_target + CardTargetAdded。
pub(crate) fn resolve_gaoyuan(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let candidates = gaoyuan_redirect_candidates(tx, player, source, card)?;
    if options.is_empty() || candidates.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveGaoyuanStage,
        vec![
            RESOLVE_GAOYUAN_MAGIC,
            player.raw(),
            source.raw(),
            card.raw(),
            1,
        ],
    )
}

/// 专属状态机 (批 269, ADR 0641 §8.57)：孛明——游戏开始时，令至多两名角色各
/// 获得牌堆中一张攻击范围等于其手牌花色数的武器牌（无 distance 信息的武器
/// 视作范围 1）。
pub(crate) fn resolve_mbbeiming(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
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
            let candidates: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                2,
                true,
                candidates,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveMbbeimingStage,
                vec![RESOLVE_MBBEIMING_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// stage 2（解析器直调）：对每个选定目标，从牌堆搜索攻击范围 == 其手牌花色数
/// 的武器牌并置入其手牌区；找不到则跳过该目标。
pub(crate) fn resolve_mbbeiming_grant(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = player;
    let _ = skill;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile is missing".into()))?;
    let mut events = Vec::new();
    for raw in targets {
        let target = PlayerId::new(*raw);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let suits_count = {
            let mut seen: [bool; 4] = [false; 4];
            for card in tx.state().hand_cards(target) {
                if let Some(card) = tx.state().cards.get(card) {
                    match card.suit {
                        Suit::Spade => seen[0] = true,
                        Suit::Heart => seen[1] = true,
                        Suit::Club => seen[2] = true,
                        Suit::Diamond => seen[3] = true,
                        Suit::None => {}
                    }
                }
            }
            seen.into_iter().filter(|seen| *seen).count()
        };
        let equip = tx.state().zones.get(draw).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    card.kind == CardKind::Weapon
                        && u32::from(card.weapon_range.max(1)) == suits_count as u32
                })
            })
        });
        let Some(equip) = equip else {
            continue;
        };
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
        events.extend(move_cards(tx, vec![equip], hand, Some(draw))?);
    }
    Ok(events)
}

/// 专属状态机 (批 267, ADR 0641 §8.55)：诤荐——mode 0：结束阶段选一名角色
/// 授一枚「诤」标记；mode 1/2：该角色用牌/打出牌时标记 +1（机内直读直加，
/// 无选择）；mode 3：owner 回合开始，各有标角色摸 min(5, 上限, 标数) 张并
/// 清标。
pub(crate) fn resolve_zhengjian(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    event_player: PlayerId,
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
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveZhengjianStage,
                vec![RESOLVE_ZHENGJIAN_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        1 | 2 => {
            // 用牌/打出计数：目标带标则标数 +1（无选择，inline）。
            if event_player != PlayerId::NONE
                && tx
                    .state()
                    .players
                    .get(event_player)
                    .is_some_and(|p| p.marks.iter().any(|(m, c)| *m == ZHENGJIAN_MARK && *c > 0))
            {
                events_extend_add_mark(tx, event_player, ZHENGJIAN_MARK, 1)?;
            }
            Ok(Vec::new())
        }
        3 => {
            // owner 回合开始：各有标角色摸 min(5, 上限, 标数) 张并清标。
            let mut events = Vec::new();
            for participant in tx.state().players.alive_by_seat() {
                let marks = tx
                    .state()
                    .players
                    .get(participant)
                    .and_then(|p| p.marks.iter().find(|(m, _)| *m == ZHENGJIAN_MARK))
                    .map_or(0, |(_, c)| *c);
                if marks <= 0 {
                    continue;
                }
                let draw_n = marks.min(5).max(0) as u32;
                if draw_n > 0 {
                    events.extend(crate::effect::draw_cards(tx, participant, draw_n)?);
                }
                if let Some(p) = tx.state_mut().players.get_mut(participant) {
                    p.marks.retain(|(m, _)| *m != ZHENGJIAN_MARK);
                }
            }
            Ok(events)
        }
        _ => Ok(Vec::new()),
    }
}

pub(crate) fn events_extend_add_mark(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    delta: i32,
) -> Result<(), EffectError> {
    crate::effect::add_mark(tx, player, mark, delta)?;
    Ok(())
}

/// 专属状态机 (批 268, ADR 0641 §8.56)：良姻——①牌被置于武将牌上时（cards_
/// moved 入 expansion 区），你可令一名角色摸一张牌；②牌从武将牌上被获得时
/// （cards_moved 出 expansion 区），你可令一名角色弃置一张手牌。
pub(crate) fn resolve_reliangyin(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
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
    let targets: Vec<u32> = match mode {
        0 => tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(PlayerId::raw)
            .collect(),
        1 => tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
            .map(PlayerId::raw)
            .collect(),
        _ => Vec::new(),
    };
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
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveReliangyinStage,
        vec![RESOLVE_LIANGYIN_MAGIC, player.raw(), skill.raw(), mode],
    )
}

/// 续体：selections[0] = 受益/弃牌角色。
pub(crate) fn resolve_reliangyin_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    who_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let who = PlayerId::new(who_raw);
    match mode {
        0 => crate::effect::draw_cards(tx, who, 1),
        1 => {
            // 该角色弃置一张手牌（其自选，强制）。
            let hand: Vec<u32> = tx
                .state()
                .hand_cards(who)
                .iter()
                .map(|card| card.raw())
                .collect();
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let payload = vec![RESOLVE_LIANGYIN_MAGIC, who.raw(), 0, 9];
            open_choice(
                tx,
                who,
                1,
                1,
                false,
                hand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveReliangyinStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

/// mode 1 子续体：who 选定的手牌弃置。
pub(crate) fn resolve_reliangyin_discard(
    tx: &mut Transaction,
    who: PlayerId,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let card = CardId::new(card_raw);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, who)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    if !tx.state().zones.get(hand).unwrap().cards.contains(&card) {
        return Err(EffectError::Illegal("liangyin card outside hand".into()));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = discard;
    }
    Ok(Vec::new())
}
