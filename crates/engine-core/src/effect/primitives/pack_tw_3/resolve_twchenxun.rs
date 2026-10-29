//! Effect primitives for `resolve_twchenxun` and related helpers.

use super::*;

pub(crate) fn resolve_twchenxun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || action > 4 {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let round = tx.state().turn.round;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .reset_twchenxun_round(round);
            open_twchenxun_target(tx, player, skill)
        }
        1 => {
            if tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.twchenxun_card())
                != Some(card)
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_twchenxun_card();
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 1,
                        skill,
                    },
                    EffectRequest::ResolveTwchenxun {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 4,
                        skill,
                    },
                ],
            )
        }
        2 => {
            if tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.twchenxun_card())
                != Some(card)
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_twchenxun_card();
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
        }
        3 => {
            if !twchenxun_targets(tx, player, skill).contains(&target) {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .add_twchenxun_target(target);
            let card = materialize_virtual_card(tx, player, CardKind::TrickJuedou)?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .set_twchenxun_card(card);
            crate::command::play::play_as_kind(
                tx,
                player,
                card,
                CardKind::TrickJuedou,
                &[target],
                false,
                false,
                false,
                false,
            )?;
            Ok(Vec::new())
        }
        4 => open_twchenxun_target(tx, player, skill),
        _ => unreachable!(),
    }
}

pub(crate) fn twfenxian_duel_targets(
    tx: &Transaction,
    owner: PlayerId,
    actor: PlayerId,
) -> Vec<PlayerId> {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            *target != owner
                && crate::command::play::validate_source_free_card_on_target(
                    tx,
                    actor,
                    CardKind::TrickJuedou,
                    *target,
                )
                .is_ok()
        })
        .collect()
}

pub(crate) fn twfenxian_duel_cards(
    tx: &Transaction,
    owner: PlayerId,
    actor: PlayerId,
) -> Vec<CardId> {
    let mut players = vec![owner];
    if actor != owner {
        players.push(actor);
    }
    let mut cards: Vec<_> = players
        .into_iter()
        .flat_map(|player| {
            [ZoneKind::Equip, ZoneKind::Judge]
                .into_iter()
                .filter_map(move |kind| tx.state().zones.find(kind, player))
        })
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect();
    if twfenxian_duel_targets(tx, owner, actor).is_empty() {
        cards.clear();
    }
    cards
}

pub(crate) fn open_twfenxian_choice(
    tx: &mut Transaction,
    actor: PlayerId,
    kind: ChoiceKind,
    options: Vec<u32>,
    payload: Vec<u32>,
) -> Result<Vec<DomainEvent>, EffectError> {
    open_choice(
        tx,
        actor,
        1,
        1,
        false,
        options,
        kind,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_twfenxian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return Ok(Vec::new());
    };
    let owner = instance.owner;
    if !tx.state().skill_effectively_enabled(instance) || action > 4 {
        return Ok(Vec::new());
    }
    if action < 4 && player != owner {
        return Err(EffectError::ContentFault(
            "invalid TW Fenxian owner context".into(),
        ));
    }
    match action {
        0 => {
            let mut options = Vec::new();
            if !twfenxian_duel_cards(tx, owner, target).is_empty() {
                options.push(0);
            }
            if crate::command::play::validate_source_free_card_on_target(
                tx,
                owner,
                CardKind::TrickHuogong,
                target,
            )
            .is_ok()
            {
                options.push(1);
            }
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let branch_mask = options.iter().fold(0, |mask, branch| mask | (1 << branch));
            open_twfenxian_choice(
                tx,
                target,
                ChoiceKind::Custom,
                options,
                vec![
                    TWFENXIAN_CUSTOM_CHOICE,
                    0,
                    owner.raw(),
                    target.raw(),
                    skill.raw(),
                    branch_mask,
                ],
            )
        }
        1 => {
            crate::command::play::validate_source_free_card_on_target(
                tx,
                owner,
                CardKind::TrickHuogong,
                target,
            )?;
            let card = materialize_virtual_card(tx, owner, CardKind::TrickHuogong)?;
            crate::command::play::play_as_kind(
                tx,
                owner,
                card,
                CardKind::TrickHuogong,
                &[target],
                true,
                false,
                true,
                false,
            )?;
            Ok(Vec::new())
        }
        2 => {
            let cards = twfenxian_duel_cards(tx, owner, target);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            open_twfenxian_choice(
                tx,
                target,
                ChoiceKind::SelectCards,
                cards.iter().map(|card| card.raw()).collect(),
                vec![
                    TWFENXIAN_CUSTOM_CHOICE,
                    2,
                    owner.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            if !twfenxian_duel_cards(tx, owner, target).contains(&card) {
                return Err(EffectError::Illegal(
                    "TW Fenxian field card is stale".into(),
                ));
            }
            let targets = twfenxian_duel_targets(tx, owner, target);
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_twfenxian_choice(
                tx,
                target,
                ChoiceKind::SelectTargets,
                targets.iter().map(|target| target.raw()).collect(),
                vec![
                    TWFENXIAN_CUSTOM_CHOICE,
                    3,
                    owner.raw(),
                    target.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )
        }
        4 => {
            if !twfenxian_duel_cards(tx, owner, player).contains(&card)
                || !twfenxian_duel_targets(tx, owner, player).contains(&target)
            {
                return Err(EffectError::Illegal(
                    "TW Fenxian Duel context changed".into(),
                ));
            }
            crate::command::play::play_field_card_as_kind(
                tx,
                player,
                card,
                CardKind::TrickJuedou,
                &[target],
            )?;
            Ok(Vec::new())
        }
        _ => unreachable!(),
    }
}

pub(crate) fn set_skill_content_flag(
    tx: &mut Transaction,
    player: PlayerId,
    bit: u8,
    value: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .filter(|instance| instance.owner == player)
        .ok_or_else(|| EffectError::ContentFault("invalid originating skill flag owner".into()))?;
    if !instance.set_content_flag(bit, value) {
        return Err(EffectError::ContentFault(
            "skill content flag bit must be below 64".into(),
        ));
    }
    Ok(Vec::new())
}

pub(crate) fn open_twmuyue_target(
    tx: &mut Transaction,
    player: PlayerId,
    card_kind: u8,
    matching_discard: bool,
    helper_skill_def: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = tx.state().players.alive_by_seat();
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.iter().map(|target| target.raw()).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveTwchenzhen,
        vec![
            3,
            player.raw(),
            helper_skill_def,
            skill.raw(),
            card_kind.into(),
            u32::from(matching_discard),
        ],
    )
}

pub(crate) fn apply_twchayi_initial(
    tx: &mut Transaction,
    target: PlayerId,
    branch: u8,
    effect_skill_def: u32,
    record_skill_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand_count = tx.state().hand_count(target);
    let mut events = run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: engine_api::SkillDefId::new(record_skill_def),
        }],
    )?;
    let record = tx
        .state()
        .players
        .get(target)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .copied()
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == record_skill_def)
        })
        .ok_or_else(|| EffectError::EngineFault("TW Chayi record skill missing".into()))?;
    tx.state_mut()
        .skills
        .get_mut(record)
        .expect("validated record skill")
        .set_twchayi_state(branch, hand_count);
    if branch == 0 {
        let cards = tx.state().hand_cards(target);
        if !cards.is_empty() {
            events.push(DomainEvent::CardsRevealed {
                cards,
                to: Vec::new(),
            });
        }
    } else {
        events.extend(run_effects(
            tx,
            vec![
                EffectRequest::AddSkill {
                    player: target,
                    skill_def: engine_api::SkillDefId::new(effect_skill_def),
                },
                EffectRequest::AddMark {
                    player: target,
                    mark: engine_api::MarkDefId::new(effect_skill_def),
                    delta: 1,
                },
            ],
        )?);
    }
    Ok(events)
}

/// 嬉战：弃置一张牌或失去 1 点体力；弃牌时暂时禁用芳踪，并按花色视为用牌。
pub(crate) fn resolve_spxizhan(
    tx: &mut Transaction,
    player: PlayerId,
    phase_player: PlayerId,
    fangzong_skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == phase_player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(phase_player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = crate::query::discardable_target_cards(
        tx.state(),
        player,
        player,
        rule_api::TargetCardScope::HandEquip,
    );
    if cards.is_empty() {
        // 无可弃牌时只能失去体力。
        return apply_effect(
            tx,
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: player,
                amount: 1,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            },
        );
    }
    open_choice(
        tx,
        player,
        0,
        1,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseSpxizhanDiscardOrLose,
        vec![player.raw(), phase_player.raw(), fangzong_skill_def.raw()],
    )
}

/// 送枢（SP 张温）：其他角色摸牌阶段开始时，将摸牌数置零，
/// 令其从仁库（拥有者「割卜」技能牌堆）选至多 min(5, 拥有者体力, 仁库牌数) 张获得，
/// 并临时获得“不能指定其他角色”技能。
pub(crate) fn resolve_spsongshu(
    tx: &mut Transaction,
    player: PlayerId,
    phase_player: PlayerId,
    renku_skill_def: engine_api::SkillDefId,
    block_skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == phase_player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && participant.hp > 0)
        || !tx
            .state()
            .players
            .get(phase_player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, renku_skill_def.raw());
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let owner_hp = tx
        .state()
        .players
        .get(player)
        .map(|p| p.hp.max(0) as usize)
        .unwrap_or(0);
    let max_count = cards.len().min(5).min(owner_hp.max(1)) as u8;
    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    open_choice(
        tx,
        phase_player,
        1,
        max_count,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseSpsongshuCards,
        vec![
            player.raw(),
            phase_player.raw(),
            renku_skill_def.raw(),
            block_skill_def.raw(),
        ],
    )
}

/// 存嗣：拥有者翻面，从牌堆/弃牌堆取一张【杀】移动到目标手牌。
pub(crate) fn resolve_spcunsi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let mut events = Vec::new();
    events.extend(apply_effect(tx, EffectRequest::TurnOver { player })?);
    if let Some((from, card)) = first_shared_pile_card_by_kind(tx, CardKind::Sha)? {
        events.extend(move_cards(tx, vec![card], hand, Some(from))?);
    }
    Ok(events)
}

/// 驭雳（神马超，ABI 409）：
/// mode 0 —— 自己造成伤害前：非雷伤害改为雷属性（prevent + 同源同量同牌雷重放），
/// 已是雷则伤害 +1；mode 1 —— 自己受伤前：取消伤害并摸伤害点数张。
/// 重放复用 ABI 374 伤害重放管线（prevented_damage_ids + damage_redirect_in_progress）
/// 防止重放伤害再次触发本技能形成递归。
pub(crate) fn resolve_yuli(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u8,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
    card: CardId,
    card_kind: u8,
    card_flags: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let damage_id = tx.state().resolution.current_damage_id;
    // mode 0（改雷重放）在重放期间跳过以防递归；mode 1（防御）必须正常响应
    // 重放出的雷伤害（它是重放链的合法观察者）。
    if damage_id == 0
        || (mode == 0 && tx.state().resolution.damage_redirect_in_progress)
        || amount == 0
        || tx
            .state()
            .resolution
            .prevented_damage_ids
            .contains(&damage_id)
    {
        return Ok(Vec::new());
    }
    match mode {
        0 => {
            let already_thunder = nature == rule_api::damage_nature::THUNDER;
            tx.state_mut()
                .resolution
                .prevented_damage_ids
                .push(damage_id);
            tx.state_mut().resolution.damage_redirect_in_progress = true;
            let result = run_effects(
                tx,
                vec![EffectRequest::DealCardDamage {
                    source,
                    target,
                    amount: amount.saturating_add(u32::from(already_thunder)),
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::THUNDER,
                    card,
                    card_kind,
                    card_flags,
                }],
            );
            tx.state_mut().resolution.damage_redirect_in_progress = false;
            result
        }
        1 => {
            // 旧 JS 中「自己受伤」分支仅在伤害为雷属性时触发（damageBegin4 filter）。
            if nature != rule_api::damage_nature::THUNDER {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .prevented_damage_ids
                .push(damage_id);
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player,
                    count: amount,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("unknown Yuli mode".into())),
    }
}

/// 霆威（神马超，ABI 410）：
/// mode 0 —— 使用【杀】指定首个目标后：加 4 枚霆标记，打开「选择该杀一个目标」choice
/// （观察者=拥有者）；mode 1 —— 所选目标四选一：非锁定技失效至其回合结束 / 交给拥有者
/// 一张装备牌 / 此【杀】对其伤害 +1 / 随机弃一张牌；取消则其横置。
/// 两段 choice 由 resolve_choice.rs 的 ContinuationKind::TingweiSelectTarget /
/// TingweiTargetOption 驱动。
pub(crate) fn resolve_tingwei(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u8,
    card: CardId,
    targets: Vec<PlayerId>,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    // “霆”标记定义 ID（与 shiji 寂灭 jimie 共用）。
    const TING_MARK_DEF: u32 = 4_145_075_227;
    let mut events = Vec::new();
    match mode {
        0 => {
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            events.extend(apply_effect(
                tx,
                EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(TING_MARK_DEF),
                    delta: 4,
                },
            )?);
            let mut payload: Vec<u32> = vec![RESOLVE_TINGWEI_SELECT, player.raw(), card.raw()];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets.iter().map(|target| target.raw()).collect(),
                ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Ok(events)
        }
        1 => {
            if target.is_none() || !tx.state().players.get(target).is_some_and(|p| p.alive) {
                return Ok(Vec::new());
            }
            let payload: Vec<u32> = vec![
                RESOLVE_TINGWEI_OPTION,
                player.raw(),
                target.raw(),
                card.raw(),
            ];
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                vec![0, 1, 2, 3],
                ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown Tingwei mode".into())),
    }
}

/// 新名士（孔融，ABI 411）：拥有者受伤后（有谦标记）伤害来源强制弃一张
/// 手牌/装备/判定区牌；弃牌由 resolve_choice 的
/// RESOLVE_XINMINGSHI_CUSTOM_CHOICE 分支结算（红色→拥有者回复 1；
/// 进入弃牌堆→拥有者获得）。
pub(crate) fn resolve_xinmingshi(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    const QIAN_MARK: u32 = 2_286_538_070;
    // 须持有谦标记（新礼让授予）。
    if !tx.state().players.get(player).is_some_and(|participant| {
        participant
            .marks
            .iter()
            .any(|(m, n)| *m == QIAN_MARK && *n > 0)
    }) {
        return Ok(Vec::new());
    }
    let gainable = crate::query::gainable_target_cards(
        tx.state(),
        source,
        source,
        rule_api::TargetCardScope::HandEquipJudge,
    );
    GuardedChoice::new(
        skill,
        Participants::distinct_pair(player, source),
        ContinuationKind::Custom,
        vec![RESOLVE_XINMINGSHI_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
    .chosen_by(source)
    .open_cards(tx, gainable)
}
