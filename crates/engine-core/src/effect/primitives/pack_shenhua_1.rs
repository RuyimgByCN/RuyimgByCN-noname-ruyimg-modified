//! Effect primitive implementations — shenhua 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) fn reveal_hand_to_player(
    tx: &Transaction,
    player: PlayerId,
    observer: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ![player, observer].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    }) {
        return Err(EffectError::Illegal(
            "hand reveal participants are not alive".into(),
        ));
    }
    Ok(vec![DomainEvent::CardsRevealed {
        cards: tx.state().hand_cards(player),
        to: vec![observer],
    }])
}

pub(crate) fn reveal_hand(tx: &Transaction, player: PlayerId) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().hand_cards(player);
    Ok((!cards.is_empty())
        .then_some(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        })
        .into_iter()
        .collect())
}

pub(crate) fn reveal_target_hand_sha_or_discard_bind(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
    stage: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target || stage > 3 {
        return Err(EffectError::ContentFault(
            "Weikui branch has invalid target or stage".into(),
        ));
    }
    let participants_alive = [player, target].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    });
    if !participants_alive {
        return Ok(Vec::new());
    }

    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Err(EffectError::ContentFault(
                    "Weikui branch requires its owner's enabled skill".into(),
                ));
            }
            if tx.state().hand_cards(target).is_empty() {
                return Err(EffectError::ContentFault(
                    "Weikui target must have a hand card before the HP cost".into(),
                ));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::RevealTargetHandShaOrDiscardBind {
                    player,
                    target,
                    helper_skill_def,
                    skill,
                    stage: 1,
                },
            );
            super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| Vec::new())
        }
        1 => {
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let has_shan = hand.into_iter().any(|card| {
                tx.state().cards.get(card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), target, card) == CardKind::Shan
                })
            });
            if !has_shan {
                return choose_discard_target_card(
                    tx,
                    player,
                    target,
                    rule_api::TargetCardScope::Hand,
                );
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::RevealTargetHandShaOrDiscardBind {
                    player,
                    target,
                    helper_skill_def,
                    skill,
                    stage: 2,
                },
            );
            reveal_hand_to_player(tx, target, player)
        }
        2 => {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::RevealTargetHandShaOrDiscardBind {
                    player,
                    target,
                    helper_skill_def,
                    skill,
                    stage: 3,
                },
            );
            try_use_virtual_sha_on_without_distance(tx, player, target, false, 0)
        }
        3 => add_bound_skill(tx, player, helper_skill_def, target),
        _ => unreachable!(),
    }
}

pub(crate) fn increase_current_draw_count(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current != player || tx.state().turn.phase != Phase::Draw {
        return Err(EffectError::ContentFault(
            "draw-count increase requires the current draw-phase player".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::ContentFault(
            "draw-count increase requires a living player".into(),
        ));
    }
    let current = tx
        .state()
        .turn
        .flag(turn_flag::CURRENT_DRAW_COUNT_BONUS)
        .max(0) as u32;
    let next = current.saturating_add(count).min(i32::MAX as u32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::CURRENT_DRAW_COUNT_BONUS, next as i32);
    Ok(Vec::new())
}

pub(crate) fn skip_next_discard_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if state.turn.current != player
        || !state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "skip-next-discard-phase owner is no longer current and alive".into(),
        ));
    }
    if !matches!(
        state.turn.phase,
        Phase::Prepare | Phase::Judge | Phase::Draw | Phase::Play
    ) {
        return Err(EffectError::ContentFault(
            "skip_next_discard_phase is only valid before the Discard phase".into(),
        ));
    }
    tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1);
    Ok(Vec::new())
}

/// Custom-continuation marker for ABI 471 `resolve_spfengyin`.
pub(crate) const RESOLVE_SPFENGYIN_CUSTOM_CHOICE: u32 = 0xDCFC_4710;

/// Custom-continuation marker for ABI 472 `resolve_xinfu_lianpian`.
pub(crate) const RESOLVE_XINFU_LIANPIAN_CUSTOM_CHOICE: u32 = 0xDCFD_4720;

/// Custom-continuation marker for ABI 473 `resolve_gongjian`.
pub(crate) const RESOLVE_GONGJIAN_CUSTOM_CHOICE: u32 = 0xDCFE_4730;

/// Custom-continuation marker for ABI 474 `resolve_xinfu_qinguo`.
pub(crate) const RESOLVE_XINFU_QINGUO_CUSTOM_CHOICE: u32 = 0xDCFF_4740;

/// Custom-continuation marker for ABI 475 `resolve_dcmanzhi`.
pub(crate) const RESOLVE_DCMANZHI_CUSTOM_CHOICE: u32 = 0xDCD0_4750;

/// Custom-continuation marker for ABI 476 `resolve_xinxingluan`.
pub(crate) const RESOLVE_XINXINGLUAN_CUSTOM_CHOICE: u32 = 0xDCD1_4760;

/// Custom-continuation marker for ABI 477 `resolve_dckuizhen`.
pub(crate) const RESOLVE_DCKUIZHEN_CUSTOM_CHOICE: u32 = 0xDCD3_4770;

/// Custom-continuation marker for ABI 478 `resolve_nsdingzhou`.
pub(crate) const RESOLVE_NSDINGZHOU_CUSTOM_CHOICE: u32 = 0xDCD4_4780;

/// Custom-continuation marker for ABI 479 `resolve_sxrmlanjiao`.
pub(crate) const RESOLVE_SXRMLANJIAO_CUSTOM_CHOICE: u32 = 0xDCD5_4790;

/// Custom-continuation marker for ABI 480 `resolve_rekuanshi`.
pub(crate) const RESOLVE_REKUANSHI_CUSTOM_CHOICE: u32 = 0xDCD6_4800;

/// Custom-continuation marker for ABI 481 `resolve_dcmoyu`.
pub(crate) const RESOLVE_DCMOYU_CUSTOM_CHOICE: u32 = 0xDCD7_4810;

/// Custom-continuation marker for ABI 482 `resolve_xizhen`.
pub(crate) const RESOLVE_XIZHEN_CUSTOM_CHOICE: u32 = 0xDCD8_4820;

/// Custom-continuation marker for ABI 483 `resolve_xuxie`.
pub(crate) const RESOLVE_XUXIE_CUSTOM_CHOICE: u32 = 0xDCD9_4830;

/// Custom-continuation marker for ABI 484 `resolve_dcchiying`.
pub(crate) const RESOLVE_DCCHIYING_CUSTOM_CHOICE: u32 = 0xDCDA_4840;

/// Custom-continuation marker for ABI 485 `resolve_xinkuangfu`.
pub(crate) const RESOLVE_XINKUANGFU_CUSTOM_CHOICE: u32 = 0xDCDB_4850;

/// Custom-continuation marker for ABI 486 `resolve_dcxiangshu`.
pub(crate) const RESOLVE_DCXIANG_SHU_CUSTOM_CHOICE: u32 = 0xDCDC_4860;

/// 双雄 recorded judgment-color mark on the owner (count: 1 = red, 2 = black). ABI 511.
pub(crate) const SHUANGXIONG_COLOR_MARK: u32 = 6_0000_0511;

/// 双雄·视为：本回合由 `resolve_shuangxiong` 授予的临时视为技能定义。ABI 511.
pub(crate) const SHUANGXIONG_VIEW_AS_DEF: u32 = 1761_4351_31;

/// Custom-continuation marker for ABI 512 `resolve_zhijian`.
pub(crate) const RESOLVE_ZHIJIAN_CUSTOM_CHOICE: u32 = 0xDCDC_5120;

/// Custom-continuation marker for ABI 513 `resolve_guzheng`.
pub(crate) const RESOLVE_GUZHENG_CUSTOM_CHOICE: u32 = 0xDCDC_5130;
/// 制霸 拼点前拒绝/拼点后取牌 自定义续体标记。ABI 523。
pub(crate) const RESOLVE_ZHIBA_CUSTOM_CHOICE: u32 = 0xDCDC_5230;
/// 挑衅（姜维）自定义续体标签。ABI 537。
pub(crate) const RESOLVE_TIAOXIN_CUSTOM_CHOICE: u32 = 0xDCDC_5370;
/// 征荣（毌丘俭）自定义续体标签。ABI 545。
pub(crate) const RESOLVE_ZHENGRONG_CUSTOM_CHOICE: u32 = 0xDCDC_5450;
/// 鸿举（毌丘俭）自定义续体标签。ABI 546。
pub(crate) const RESOLVE_HONGJU_CUSTOM_CHOICE: u32 = 0xDCDC_5460;
/// 清俭（毌丘俭）自定义续体标签。ABI 547。
pub(crate) const RESOLVE_QINGJIAN_CUSTOM_CHOICE: u32 = 0xDCDC_5470;
/// 雷击（张角）自定义续体标签。ABI 549。
pub(crate) const RESOLVE_RELEIJI_CUSTOM_CHOICE: u32 = 0xDCDC_5490;
/// 黄天（张角）自定义续体标签。ABI 551。
pub(crate) const RESOLVE_HUANGTIAN_CUSTOM_CHOICE: u32 = 0xDCDC_5510;
/// 好施（鲁肃）自定义续体标签。ABI 555。
pub(crate) const RESOLVE_HAOSHI_CUSTOM_CHOICE: u32 = 0xDCDC_5550;
/// 缔盟（鲁肃）自定义续体标签。ABI 556。
pub(crate) const RESOLVE_DIMENG_CUSTOM_CHOICE: u32 = 0xDCDC_5560;
pub(crate) const RESOLVE_QIAOBIAN_DISCARD: u32 = 0xDCDC_5590;
pub(crate) const RESOLVE_QIAOBIAN_DRAW_TARGETS: u32 = 0xDCDC_5591;
pub(crate) const RESOLVE_QIAOBIAN_DRAW_PICK: u32 = 0xDCDC_5592;
pub(crate) const RESOLVE_QIAOBIAN_PLAY_CARD: u32 = 0xDCDC_5593;
pub(crate) const RESOLVE_QIAOBIAN_PLAY_TARGET: u32 = 0xDCDC_5594;
pub(crate) const RESOLVE_XIANGLE_CUSTOM_CHOICE: u32 = 0xDCDC_5600;
pub(crate) const RESOLVE_FANGQUAN_DISCARD: u32 = 0xDCDC_5601;
pub(crate) const RESOLVE_FANGQUAN_TARGET: u32 = 0xDCDC_5602;
pub(crate) const RESOLVE_ZUILUN_PICK: u32 = 0xDCDC_5630;
pub(crate) const RESOLVE_ZUILUN_TARGET: u32 = 0xDCDC_5631;
pub(crate) const RESOLVE_JUZHAN_GAIN: u32 = 0xDCDC_5710;
pub(crate) const RESOLVE_QIZHI_TARGET: u32 = 0xDCDC_5730;
pub(crate) const RESOLVE_QIZHI_CARD: u32 = 0xDCDC_5731;
pub(crate) const RESOLVE_JINQU_DISCARD: u32 = 0xDCDC_5732;
pub(crate) const RESOLVE_JIANXIANG_TARGET: u32 = 0xDCDC_5740;
pub(crate) const RESOLVE_SHENSHI_GIVE: u32 = 0xDCDC_5741;
pub(crate) const RESOLVE_SHENSHI_TARGET: u32 = 0xDCDC_5742;
pub(crate) const RESOLVE_SHENSHI_REWARD: u32 = 0xDCDC_5743;
pub(crate) const RESOLVE_MINGREN_PLACE: u32 = 0xDCDC_5760;
pub(crate) const RESOLVE_MINGREN_REPLACE: u32 = 0xDCDC_5761;
pub(crate) const RESOLVE_ZHENLIANG_DISCARD: u32 = 0xDCDC_5762;
pub(crate) const RESOLVE_ZHENLIANG_DAMAGE: u32 = 0xDCDC_5763;
pub(crate) const RESOLVE_ZHENLIANG_DRAW: u32 = 0xDCDC_5764;
pub(crate) const RESOLVE_KUIZHU_DRAW: u32 = 0xDCDC_5770;
pub(crate) const RESOLVE_KUIZHU_DAMAGE: u32 = 0xDCDC_5771;
pub(crate) const RESOLVE_ZHIZHENG_CARD: u32 = 0xDCDC_5772;
pub(crate) const RESOLVE_LIANGYIN_DRAW: u32 = 0xDCDC_5790;
pub(crate) const RESOLVE_LIANGYIN_RECOVER: u32 = 0xDCDC_5791;
pub(crate) const RESOLVE_LIANGYIN_DISCARD: u32 = 0xDCDC_5792;
pub(crate) const RESOLVE_LIANGYIN_TARGET: u32 = 0xDCDC_5793;
pub(crate) const RESOLVE_LIANGYIN_COMMIT: u32 = 0xDCDC_5794;
pub(crate) const RESOLVE_KONGSHENG_PLACE: u32 = 0xDCDC_5795;
pub(crate) const RESOLVE_KONGSHENG_TARGET: u32 = 0xDCDC_5796;
/// 从俭 目标/选牌续体标记。ABI 524。
pub(crate) const RESOLVE_DRLT_CONGJIAN_CUSTOM_CHOICE: u32 = 0xDCDC_5240;
pub(crate) const RESOLVE_NZRY_FEIJUN_CUSTOM_CHOICE: u32 = 0xDCDC_5260;
pub(crate) const RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE: u32 = 0xDCDC_5280;
/// 庸肆弃牌阶段状态（既有 drlt_yongsi1 定义）。ABI 527。
pub(crate) const DRLT_YONGSI1_DEF: u32 = 3870932875;
/// 飞军 目标首次被选标记（置于目标上，永久）。ABI 526。
pub(crate) const FEIJUN_TARGET_MARK: u32 = 6_0000_0526;
/// 魂姿（hunzi）稳定技能定义。ABI 523 觉醒检查。
pub(crate) const HUNZI_DEF: u32 = 4151424761;
/// 界魂姿（rehunzi）稳定技能定义。ABI 523 觉醒检查。
pub(crate) const REHUNZI_DEF: u32 = 3543073615;

/// 相术 secret-number mark on the owner (count = the chosen number).
pub(crate) const DCXIANG_SHU_NUMBER_MARK: u32 = 6_0000_0005;

pub(crate) const RESOLVE_DCYINGTU_CUSTOM_CHOICE: u32 = 0xDCDA_4870;

/// Custom-continuation marker for ABI 410 霆威选择该杀目标阶段（payload[0]）。
pub(crate) const RESOLVE_TINGWEI_SELECT: u32 = 0xDCDA_4100;
/// Custom-continuation marker for ABI 410 霆威目标四选一阶段（payload[0]）。
pub(crate) const RESOLVE_TINGWEI_OPTION: u32 = 0xDCDA_4101;
/// Custom-continuation marker for ABI 411 新名士强制弃牌结算（payload[0]）。
pub(crate) const RESOLVE_XINMINGSHI_CUSTOM_CHOICE: u32 = 0xDCDA_4110;
/// Custom-continuation marker for ABI 412 殉义选择义目标（payload[0]）。
pub(crate) const RESOLVE_XUNYI_SELECT: u32 = 0xDCDA_4120;
/// Custom-continuation marker for ABI 414 恶荐二选一（payload[0]）。
pub(crate) const RESOLVE_EJIAN_CHOICE: u32 = 0xDCDA_4140;
/// Custom-continuation marker for ABI 417 舍医选牌防伤（payload[0]）。
pub(crate) const RESOLVE_SHEYI_CHOICE: u32 = 0xDCDA_4170;
/// Custom-continuation marker for ABI 583 杨彪「举讹」确认/选目标（payload[0]）。
pub(crate) const RESOLVE_DCJUE_CHOICE: u32 = 0xDCDA_5830;
/// Custom-continuation marker for ABI 584 阮籍「酒遁」分支（payload[0]）。
pub(crate) const RESOLVE_RUANJI_CHOICE: u32 = 0xDCDA_5840;
pub(crate) const RESOLVE_ZHENGBI_CHOICE: u32 = 0xDCDA_5850;
pub(crate) const RESOLVE_QIAOJIAN_CHOICE: u32 = 0xDCDA_5860;
pub(crate) const RESOLVE_XICHA_CHOICE: u32 = 0xDCDA_5861;
pub(crate) const RESOLVE_HUZUN_CHOICE: u32 = 0xDCDC_5870;
pub(crate) const RESOLVE_CHENGLVE_DISCARD: u32 = 0xDCDC_5890;
pub(crate) const RESOLVE_SHICAI_PLACE: u32 = 0xDCDC_5891;
/// 踞襄（张绣）装备牌选择续体标记。ABI 590。
pub const RESOLVE_TWJUXIANG_CARD: u32 = 0xDCDC_5900;
/// 旧不屈（旧周泰）恢复移除创牌续体标记。ABI 591。
pub const RESOLVE_GZBUQU_RECOVER_CARD: u32 = 0xDCDC_5910;
pub(crate) const RESOLVE_ZHUGEMENGXUE_CHOICE: u32 = 0xDCDC_5880;
pub(crate) const RESOLVE_BAILINGYUN_CHOICE: u32 = 0xDCDC_5930;
pub(crate) const RESOLVE_DCDICHOU_CHOICE: u32 = 0xDCDC_5940;
pub(crate) const RESOLVE_DCDUFENG_CHOICE: u32 = 0xDCDC_5950;
/// Custom-continuation marker for ABI 420 曲迪三选一（payload[0]）。
pub(crate) const RESOLVE_DBQUEDI_CHOICE: u32 = 0xDCDA_4200;
/// Custom-continuation marker for ABI 421 隐据二选一（payload[0]）。
pub(crate) const RESOLVE_SPYINJU_CHOICE: u32 = 0xDCDA_4210;

pub(crate) fn resolve_dcyingtu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    cards: Vec<CardId>,
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
    match stage {
        0 => {
            let neighbors = crate::query::alive_seat_neighbors(tx.state(), player);
            let target = if neighbors.contains(&target) {
                target
            } else {
                neighbors
                    .into_iter()
                    .find(|candidate| {
                        let hand = tx.state().zones.find(ZoneKind::Hand, *candidate);
                        cards.iter().any(|card| {
                            tx.state().cards.get(*card).is_some_and(|state| {
                                state.owner == *candidate && Some(state.zone) == hand
                            })
                        })
                    })
                    .unwrap_or(PlayerId::NONE)
            };
            if target.is_none() {
                return Ok(Vec::new());
            }
            let options = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DCYINGTU_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    1,
                ],
            )
        }
        2 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DCYINGTU_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    2,
                ],
            )
        }
        _ => Err(EffectError::ContentFault("bad Dcyingtu stage".into())),
    }
}

pub(crate) fn resolve_dcxiangshu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    // Guard: the observed player's hand count >= their HP.
    let observed_hp = tx.state().players.get(target).map(|p| p.hp).unwrap_or(0);
    if tx.state().hand_count(target) < observed_hp as u32 {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = (0..=5).collect();
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            RESOLVE_DCXIANG_SHU_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            target.raw(),
        ],
    )
}

pub(crate) fn resolve_dcxiangshu_lottery(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
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
    let number = tx
        .state()
        .players
        .get(player)
        .map(|p| {
            p.marks
                .iter()
                .find(|(mark, _)| *mark == DCXIANG_SHU_NUMBER_MARK)
                .map(|(_, count)| *count)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    if number == 0 {
        return Ok(Vec::new());
    }
    let hand = tx.state().hand_count(target) as i32;
    let delta = (hand - number).abs();
    let mut effects = vec![EffectRequest::RemoveMark {
        player,
        mark: engine_api::MarkDefId::new(DCXIANG_SHU_NUMBER_MARK),
        amount: number.max(0) as u32,
    }];
    if delta <= 1
        && !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty()
    {
        let options: Vec<_> = crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .into_iter()
        .map(CardId::raw)
        .collect();
        crate::effect::run_effects(tx, effects.clone())?;
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_DCXIANG_SHU_CUSTOM_CHOICE,
                player.raw(),
                engine_api::SkillId::NONE.raw(),
                target.raw(),
                1,
                delta as u32,
            ],
        );
    }
    if delta == 0 {
        effects.push(EffectRequest::DealDamage {
            source: player,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    run_effects(tx, effects)
}

/// Cards one player lost during their own Discard phase that are still in the discard pile.
/// Requires the ABI 513 slice base; yields nothing when no Discard phase was entered.
pub(crate) fn discard_phase_discards_for(
    state: &crate::state::GameState,
    player: PlayerId,
) -> impl Iterator<Item = CardId> + '_ {
    let base = state.turn.flag(turn_flag::DISCARD_PHASE_LOST_BASE).max(0) as usize;
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    state
        .turn
        .lost_cards
        .iter()
        .skip(base.saturating_sub(1))
        .filter(move |(loser, _)| *loser == player)
        .filter_map(move |(_, card)| {
            state.cards.get(*card).and_then(|card_state| {
                discard
                    .is_some_and(|zone| card_state.zone == zone)
                    .then_some(*card)
            })
        })
}

/// Physical cards the player discarded during the current Discard phase, including cards that
/// nested triggers subsequently moved out of the discard pile.
pub(crate) fn discard_phase_discarded_cards_for(
    state: &crate::state::GameState,
    player: PlayerId,
) -> impl Iterator<Item = CardId> + '_ {
    let base = state.turn.flag(turn_flag::DISCARD_PHASE_LOST_BASE).max(0) as usize;
    state
        .turn
        .lost_cards
        .iter()
        .skip(base.saturating_sub(1))
        .filter(move |(loser, card)| *loser == player && state.turn.discarded_cards.contains(card))
        .map(|(_, card)| *card)
}

/// Every card discarded into the discard pile during the current Discard phase (any player),
/// still present in the discard pile. Requires the ABI 513 slice base.
pub(crate) fn discard_phase_discards_all(
    state: &crate::state::GameState,
) -> impl Iterator<Item = CardId> + '_ {
    let base = state.turn.flag(turn_flag::DISCARD_PHASE_LOST_BASE).max(0) as usize;
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    state
        .turn
        .lost_cards
        .iter()
        .skip(base.saturating_sub(1))
        .filter_map(move |(_, card)| {
            state.cards.get(*card).and_then(|card_state| {
                discard
                    .is_some_and(|zone| card_state.zone == zone)
                    .then_some(*card)
            })
        })
}

/// 双雄（shenhua 颜良文丑）：摸牌阶段可选改为判定，获得判定牌并记录判定颜色，
/// 本回合非该颜色手牌可当【决斗】使用。ABI 511。
pub(crate) fn resolve_shuangxiong(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Draw
        || tx.state().turn.draw_phase_is_fixed()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    // 双雄 replaces the pending normal draw with a judgment.
    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let color = match result.suit {
        Suit::Spade | Suit::Club => 2,
        _ => 1,
    };
    let mut events = vec![completed_judgment_event(player, result, CardId::NONE)];
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Shuangxiong owner hand is missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Shuangxiong discard pile is missing".into()))?;
    events.extend(move_cards(tx, vec![result.card], hand, Some(discard))?);
    events.extend(add_mark(tx, player, SHUANGXIONG_COLOR_MARK, color)?);
    events.extend(add_skill_until_turn_end(
        tx,
        player,
        engine_api::SkillDefId::new(SHUANGXIONG_VIEW_AS_DEF),
    )?);
    Ok(events)
}

/// 直谏（shenhua 张昭张纮）：出牌阶段限一次，将一张手牌区装备牌交给一名可装备该牌的
/// 其他角色，然后摸一张。ABI 512。stage 0 选择目标，stage 1 执行赠予与摸牌。
pub(crate) fn resolve_zhijian(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
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
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let slot = tx
        .state()
        .cards
        .get(card)
        .and_then(|card_state| card_state.kind.equip_slot());
    if stage == 0 {
        if !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card_state| hand.is_some_and(|zone| card_state.zone == zone))
        {
            return Ok(Vec::new());
        }
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|candidate| *candidate != player)
            .filter(|candidate| {
                tx.state()
                    .players
                    .get(*candidate)
                    .is_some_and(|candidate_state| {
                        candidate_state.alive
                            && slot.is_some_and(|s| candidate_state.equip_slot_enabled(s))
                    })
            })
            .map(|candidate| candidate.raw())
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                RESOLVE_ZHIJIAN_CUSTOM_CHOICE,
                player.raw(),
                card.raw(),
                skill.raw(),
            ],
        );
    }
    if target.is_none()
        || target == player
        || !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card_state| hand.is_some_and(|zone| card_state.zone == zone))
        || !tx.state().players.get(target).is_some_and(|candidate| {
            candidate.alive && slot.is_some_and(|s| candidate.equip_slot_enabled(s))
        })
    {
        return Ok(Vec::new());
    }
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, target)
        .ok_or_else(|| EffectError::Illegal("Zhijian target equip zone is missing".into()))?;
    let mut events = give_cards(
        tx,
        vec![card],
        equip,
        hand.ok_or_else(|| EffectError::Illegal("Zhijian owner hand is missing".into()))?,
    )?;
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DrawCards { player, count: 1 }],
    )?);
    Ok(events)
}

/// 固政（shenhua 张昭张纮）：其他角色弃牌阶段结束后，归还其一张该阶段弃置且仍处弃牌堆
/// 的牌，并可选获得其余弃牌。ABI 513。stage 0 选归还牌，stage 1 选是否取回剩余。
pub(crate) fn resolve_guzheng(
    tx: &mut Transaction,
    player: PlayerId,
    event_player: PlayerId,
    stage: u8,
    _cards: Vec<u32>,
    return_card: CardId,
    take_rest: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || event_player == player
        || !tx
            .state()
            .players
            .get(event_player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if stage == 0 {
        let options: Vec<_> = discard_phase_discards_for(tx.state(), event_player)
            .map(|card| card.raw())
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_GUZHENG_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                event_player.raw(),
                1, // stage: return-card chosen
            ],
        );
    }
    if stage == 1 {
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![1, 0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                RESOLVE_GUZHENG_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                event_player.raw(),
                2, // stage: take-rest chosen
                return_card.raw(),
            ],
        );
    }
    // stage 2: commit the return and optional take-rest.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Guzheng discard pile is missing".into()))?;
    let still_in_discard = |tx: &Transaction, card: CardId| {
        tx.state()
            .cards
            .get(card)
            .is_some_and(|card_state| card_state.zone == discard)
    };
    if !still_in_discard(tx, return_card) {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, event_player)
        .ok_or_else(|| EffectError::Illegal("Guzheng target hand is missing".into()))?;
    let mut events = move_cards(tx, vec![return_card], hand, Some(discard))?;
    if take_rest {
        let owner_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("Guzheng owner hand is missing".into()))?;
        let rest: Vec<_> = discard_phase_discards_all(tx.state())
            .filter(|card| *card != return_card)
            .collect();
        if !rest.is_empty() {
            events.extend(move_cards(tx, rest, owner_hand, Some(discard))?);
        }
    }
    Ok(events)
}

/// 制霸（孙策，主公技）：其他吴势力角色出牌阶段限一次可与拥有者拼点；拥有者未赢时
/// 可选择获得两张拼点牌，已觉醒时可先拒绝。ABI 523。
pub(crate) fn resolve_zhiba(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|participant| participant.alive)
        || actor == player
        || tx.state().turn.current != actor
        || tx.state().turn.phase != Phase::Play
        || tx.state().player_group(actor) != Some("wu")
        || tx.state().hand_cards(player).is_empty()
        || tx.state().hand_cards(actor).is_empty()
    {
        return Ok(Vec::new());
    }
    // 已觉醒（魂姿/界魂姿均已移除）时，拥有者可先拒绝本次拼点。
    let awakened = !tx.state().skills.sorted_live().iter().any(|instance| {
        instance.owner == player
            && (instance.def.raw() == HUNZI_DEF || instance.def.raw() == REHUNZI_DEF)
    });
    if awakened {
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![1, 0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                RESOLVE_ZHIBA_CUSTOM_CHOICE,
                player.raw(),
                actor.raw(),
                skill.raw(),
                0, // stage: refuse choice
            ],
        );
    }
    // 未觉醒：记录待决拼点并让发起方选择拼点牌（复用通用拼点选择续体）。
    tx.state_mut().skills.get_mut(skill).map(|instance| {
        instance.set_zhiba_compare(actor);
    });
    let options: Vec<_> = tx
        .state()
        .hand_cards(actor)
        .into_iter()
        .map(CardId::raw)
        .collect();
    open_choice(
        tx,
        actor,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseHandComparePlayerCard,
        vec![
            actor.raw(),
            player.raw(),
            engine_api::SkillDefId::NONE.raw(),
            engine_api::SkillDefId::NONE.raw(),
            skill.raw(),
        ],
    )
}

/// 制霸拼点完成结算：若拥有者未赢，可选择获得两张拼点牌。ABI 523。
pub(crate) fn resolve_zhiba_settle(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    winner: PlayerId,
    player_card: CardId,
    target_card: CardId,
    stage: u8,
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
    if stage == 0 {
        // 发起方（吴势力角色）获胜时没有取牌分支；平局或拥有者获胜时提供选择。
        if winner == actor {
            tx.state_mut().skills.get_mut(skill).map(|instance| {
                instance.clear_zhiba_compare();
            });
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![1, 0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                RESOLVE_ZHIBA_CUSTOM_CHOICE,
                player.raw(),
                actor.raw(),
                skill.raw(),
                1, // stage: accept-gain choice
                winner.raw(),
                player_card.raw(),
                target_card.raw(),
            ],
        );
    }
    // stage 1: 拥有者选择获得两张拼点牌。
    tx.state_mut().skills.get_mut(skill).map(|instance| {
        instance.clear_zhiba_compare();
    });
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Zhiba discard pile is missing".into()))?;
    let still_in_discard = |tx: &Transaction, card: CardId| {
        tx.state()
            .cards
            .get(card)
            .is_some_and(|card_state| card_state.zone == discard)
    };
    if !still_in_discard(tx, player_card) && !still_in_discard(tx, target_card) {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Zhiba owner hand is missing".into()))?;
    let cards: Vec<_> = [player_card, target_card]
        .into_iter()
        .filter(|card| still_in_discard(tx, *card))
        .collect();
    move_cards(tx, cards, hand, Some(discard))
}

/// 从俭（张绣）：成为多目标锦囊牌的目标后，交给其中一名其他目标一张手牌/装备牌，
/// 然后摸一张（所给为装备牌则摸两张）。ABI 524。
pub(crate) fn resolve_drlt_congjian(
    tx: &mut Transaction,
    player: PlayerId,
    mut targets: Vec<PlayerId>,
    card: CardId,
    target: PlayerId,
    stage: u8,
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
    if stage == 0 {
        let options: Vec<_> = targets
            .into_iter()
            .filter(|candidate| {
                *candidate != player
                    && tx
                        .state()
                        .players
                        .get(*candidate)
                        .is_some_and(|participant| participant.alive)
            })
            .map(|candidate| candidate.raw())
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                RESOLVE_DRLT_CONGJIAN_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                0, // stage: recipient choice
            ],
        );
    }
    if stage == 1 {
        let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(CardId::raw)
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_DRLT_CONGJIAN_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                1, // stage: card choice
                target.raw(),
            ],
        );
    }
    // stage 2: 赠予并摸牌。
    if target.is_none() || target == player {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Congjian owner hand is missing".into()))?;
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Congjian target hand is missing".into()))?;
    let is_equip = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card_state| card_state.kind.is_equip());
    let mut events = give_cards(tx, vec![card], target_hand, hand)?;
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DrawCards {
            player,
            count: if is_equip { 2 } else { 1 },
        }],
    )?);
    Ok(events)
}

/// 飞军（王平）：出牌阶段限一次弃一张牌，选择“手牌数大于你者交给你一张牌”或
/// “装备区牌数大于你者弃置一张装备牌”分支（两项均可用时二选一）；目标首次被选时
/// 置兵略旗标（摸两张由 nzry_binglve 于出牌阶段结束结算）。ABI 526。
/// 庸肆（袁术）：摸牌阶段把摸牌数固定为存活势力数；出牌阶段结束时按本回合是否造成
/// 伤害：未造成伤害摸至体力值，否则授予既有庸肆弃牌阶段状态（回合末移除）。
/// ABI 527。
pub(crate) fn resolve_drlt_yongsi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().turn.phase != Phase::Draw || tx.state().turn.draw_phase_is_fixed() {
                return Ok(Vec::new());
            }
            let groups = crate::query::alive_group_count(tx.state()).max(1) as u32;
            tx.state_mut().turn.set_draw_phase_fixed_count(groups);
            Ok(Vec::new())
        }
        _ => {
            if tx.state().turn.damage_dealt_count(player) == 0 {
                let hp = tx
                    .state()
                    .players
                    .get(player)
                    .map(|p| p.hp.max(0) as u32)
                    .unwrap_or(0);
                let hand = tx.state().hand_count(player);
                if hand < hp {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player,
                            count: hp - hand,
                        }],
                    )
                } else {
                    Ok(Vec::new())
                }
            } else {
                add_skill_until_turn_end(tx, player, engine_api::SkillDefId::new(DRLT_YONGSI1_DEF))
            }
        }
    }
}

/// 伪帝（袁术，主公技）：弃牌阶段开始把至多 min(需弃置数, 其他群势力角色数) 张手牌
/// 分别交给等量其他群势力角色；给出的牌离开手牌后弃牌需求自然下降。ABI 528。
pub(crate) fn resolve_drlt_weidi(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    stage: u8,
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
    match stage {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Discard {
                return Ok(Vec::new());
            }
            let limit = crate::query::query(
                tx.state(),
                &rule_api::QueryRequest::HandLimit { player },
                &mut crate::query::QueryContext::new(),
            );
            let limit = match limit {
                rule_api::QueryResponse::Int(value) => value.max(0) as u32,
                _ => return Ok(Vec::new()),
            };
            let hand = tx.state().hand_count(player);
            let need = hand.saturating_sub(limit);
            let qun_others: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player && tx.state().player_group(*candidate) == Some("qun")
                })
                .collect();
            let num = need.min(qun_others.len() as u32);
            if num == 0 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    !crate::query::card_discard_protected_from(tx.state(), player, player, *card)
                })
                .map(CardId::raw)
                .collect();
            if (options.len() as u32) < 1 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                num.min(u8::MAX as u32) as u8,
                true,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    0, // stage: card choice
                    qun_others.len() as u32,
                ],
            )
        }
        1 => {
            // targets choice; cards come from the payload.
            if cards.is_empty() || cards.len() > u8::MAX as usize {
                return Ok(Vec::new());
            }
            let qun_others: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player && tx.state().player_group(*candidate) == Some("qun")
                })
                .collect();
            if qun_others.len() < cards.len() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                cards.len() as u8,
                cards.len() as u8,
                true,
                qun_others
                    .into_iter()
                    .map(|candidate| candidate.raw())
                    .collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                {
                    let mut payload = vec![
                        RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE,
                        player.raw(),
                        skill.raw(),
                        1, // stage: target choice
                        cards.len() as u32,
                    ];
                    payload.extend(cards.iter().map(|card| card.raw()));
                    payload
                },
            )
        }
        _ => {
            // stage 2: give card[i] to target[i].
            if cards.len() != targets.len() || cards.is_empty() {
                return Ok(Vec::new());
            }
            let mut events = Vec::new();
            for (card, target) in cards.into_iter().zip(targets.into_iter()) {
                if target.is_none() || target == player {
                    continue;
                }
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::Illegal("Weidi owner hand is missing".into()))?;
                let target_hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::Illegal("Weidi target hand is missing".into()))?;
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card_state| card_state.zone)
                    .ok_or_else(|| EffectError::Illegal("Weidi card missing".into()))?;
                if from != hand {
                    continue;
                }
                events.extend(give_cards(tx, vec![card], target_hand, hand)?);
            }
            Ok(events)
        }
    }
}
pub(crate) fn resolve_lieren(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    win_skill_def: engine_api::SkillDefId,
    lose_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 拼点需要双方都有手牌。
    if tx.state().hand_cards(target).is_empty() {
        return Ok(Vec::new());
    }
    let own_hand = tx.state().hand_cards(player);
    GuardedChoice::new(
        skill,
        Participants::distinct_pair(player, target),
        ContinuationKind::ChooseHandComparePlayerCard,
        vec![
            player.raw(),
            target.raw(),
            win_skill_def.raw(),
            lose_skill_def.raw(),
            skill.raw(),
        ],
    )
    .open_cards(tx, own_hand)?;
    Ok(Vec::new())
}

pub(crate) fn resolve_tuntian(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let skill_def = skill_pile_def(tx, player, skill)?;
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let judgment_event = completed_judgment_event(player, result, engine_api::CardId::NONE);
    if result.suit == engine_api::Suit::Heart {
        return Ok(vec![judgment_event]);
    }
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    move_cards(tx, vec![result.card], pile, Some(discard))
        .map(|events| prepend_event(judgment_event, events))
}

pub(crate) fn resolve_zaoxian(
    tx: &mut Transaction,
    player: PlayerId,
    jixi_def: engine_api::SkillDefId,
    tuntian_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    if tx.state().skill_pile_cards(player, tuntian_def.raw()).len() < 3 {
        return Ok(Vec::new());
    }
    // 已觉醒（持有急袭）时不再重复结算。
    if tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == player && instance.def == jixi_def)
    {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        vec![
            EffectRequest::ChangeMaxHp { player, delta: -1 },
            EffectRequest::AddSkill {
                player,
                skill_def: jixi_def,
            },
        ],
    )
}

pub(crate) fn resolve_tiaoxin(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let in_range_with_cards: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .filter(|&candidate| {
                    crate::query::player_in_attack_range(tx.state(), candidate, player)
                })
                .filter(|&candidate| {
                    !crate::query::discardable_hand_equip_cards(tx.state(), candidate).is_empty()
                })
                .collect();
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    PlayerId::NONE.raw(),
                    skill.raw(),
                ],
            )
            .during(TurnContext::OwnersPhase(Phase::Play))
            .open_targets(tx, in_range_with_cards)
        }
        _ => Err(stage_must_resume_via_continuation("挑衅")),
    }
}

pub(crate) fn resolve_zhiji(
    tx: &mut Transaction,
    player: PlayerId,
    guanxing_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || !tx.state().hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
    // 已觉醒（持有观星）时不再重复结算。
    if tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == player && instance.def == guanxing_def)
    {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DrawCards { player, count: 2 },
            EffectRequest::RecoverHp {
                source: PlayerId::NONE,
                player,
                amount: 1,
            },
            EffectRequest::ChangeMaxHp { player, delta: -1 },
            EffectRequest::AddSkill {
                player,
                skill_def: guanxing_def,
            },
        ],
    )
}

/// ABI 681 (batch 176): watch the top `count` cards and re-order them one by one —
/// each round the player picks top-sequence or bottom-sequence for the current card.
/// Multi-round state rides the continuation payload:
/// `[player, count, round, flipped_len, flipped..., top_len, top..., bottom_len, bottom..., skill]`.
pub(crate) fn guanxing_top(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|p| p.alive)
        || count == 0
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let mut flipped = Vec::new();
    for _ in 0..count {
        if tx
            .state()
            .zones
            .get(draw)
            .is_none_or(|zone| zone.cards.is_empty())
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let Some(card) = tx
            .state()
            .zones
            .get(draw)
            .and_then(|zone| zone.cards.last().copied())
        else {
            break;
        };
        move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
        flipped.push(card);
    }
    if flipped.is_empty() {
        return Ok(Vec::new());
    }
    let payload = vec![
        player.raw(),
        count,
        0,
        flipped.len() as u32,
    ]
    .into_iter()
    .chain(flipped.iter().copied().map(CardId::raw))
    .chain([0u32, 0u32])
    .chain(std::iter::once(skill.raw()))
    .collect::<Vec<_>>();
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::SelectTargets,
        ContinuationKind::GuanxingPick,
        payload,
    )
}

/// ABI 681 guanxing_top commit: re-seat the picked sequences onto the draw pile —
/// bottom sequence goes first (lowest), then remaining pile, then top sequence on top.
pub(crate) fn commit_guanxing_top(
    tx: &mut Transaction,
    player: PlayerId,
    top: &[CardId],
    bottom: &[CardId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    // 所有翻开的牌都在 processing；按 bottom（原序）→ 其余 → top（栈序）落位：
    // top 序列最后放置且每张压到牌堆顶（后放者更靠顶），bottom 序列最先放且压到
    // 牌堆顶（后放的 bottom 更靠顶——与「置于牌堆底」语义一致：底序列逆序垫底）。
    for card in bottom.iter() {
        crate::effect::move_one_card(tx, *card, processing, draw, PlayerId::NONE, true)?;
    }
    for card in top.iter().rev() {
        crate::effect::move_one_card(tx, *card, processing, draw, PlayerId::NONE, true)?;
    }
    Ok(Vec::new())
}

/// ABI 683 (batch 183): count non-zero stacks among the four falu marks and run
/// [`guanxing_top`] for that many top cards.
pub(crate) fn guanxing_top_by_falu_marks(
    tx: &mut Transaction,
    player: PlayerId,
    marks: [engine_api::SkillDefId; 4],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad guanxing viewer".into()))?;
    let n = marks
        .iter()
        .filter(|def| {
            participant
                .marks
                .iter()
                .any(|(found, count)| *found == def.raw() && *count > 0)
        })
        .count() as u32;
    guanxing_top(tx, player, n, skill)
}

pub(crate) fn resolve_zaiqi(
    tx: &mut Transaction,
    player: PlayerId,
    extra: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Draw
        || tx.state().turn.draw_phase_is_fixed()
    {
        return Ok(Vec::new());
    }
    let Some(owner) = tx.state().players.get(player) else {
        return Ok(Vec::new());
    };
    if !owner.alive || owner.hp as u32 >= owner.max_hp {
        return Ok(Vec::new());
    }
    let lost = (owner.max_hp as i32 - owner.hp).max(1) as u32;
    let count = lost.min(12) as usize + extra as usize;
    if count == 0 {
        return Ok(Vec::new());
    }
    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    // 翻亮牌堆顶 count 张（旧 JS showCards）。
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let mut flipped = Vec::new();
    for _ in 0..count {
        if tx
            .state()
            .zones
            .get(draw)
            .is_none_or(|zone| zone.cards.is_empty())
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let Some(card) = tx
            .state()
            .zones
            .get(draw)
            .and_then(|zone| zone.cards.last().copied())
        else {
            break;
        };
        move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
        flipped.push(card);
    }
    if flipped.is_empty() {
        return Ok(Vec::new());
    }
    let mut events = vec![DomainEvent::CardsMoved {
        cards: flipped.clone(),
        from: draw,
        to: processing,
        reason: CardMoveReason::Unspecified,
    }];
    events.push(DomainEvent::CardsRevealed {
        cards: flipped.clone(),
        to: Vec::new(),
    });
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let mut hearts = 0u32;
    let mut gained = Vec::new();
    let mut discard_cards = Vec::new();
    for card in flipped {
        let suit = tx
            .state()
            .cards
            .get(card)
            .map(|value| crate::query::effective_card_suit(tx.state(), player, value))
            .unwrap_or(engine_api::Suit::None);
        if suit == engine_api::Suit::Heart {
            hearts += 1;
            discard_cards.push(card);
        } else {
            gained.push(card);
        }
    }
    events.extend(move_cards(tx, gained, hand, Some(processing))?);
    events.extend(move_cards(tx, discard_cards, discard, Some(processing))?);
    if hearts > 0 {
        events.extend(run_effects(
            tx,
            vec![EffectRequest::RecoverHp {
                source: PlayerId::NONE,
                player,
                amount: hearts,
            }],
        )?);
    }
    Ok(events)
}

pub(crate) fn resolve_twqiushou(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            matches!(
                tx.state().player_group(candidate),
                Some("shu") | Some("qun")
            )
        })
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        targets
            .into_iter()
            .map(|target| EffectRequest::DrawCards {
                player: target,
                count: 1,
            })
            .collect(),
    )
}

pub(crate) const POTTUNTIAN_CUSTOM_CHOICE: u32 = 0x5054_5431;
pub(crate) const POTJIXI_CUSTOM_CHOICE: u32 = 0x504a_5431;
pub(crate) const POTCEDUAN_CUSTOM_CHOICE: u32 = 0x5043_5441;
pub(crate) const POTTAISHICI_CUSTOM_CHOICE: u32 = 0x5054_5343;
pub(crate) const MBZHONGHUI_CUSTOM_CHOICE: u32 = 0x4d42_5a48;
/// 势辛宪英诫节/清识自定义续体标签（Codex ABI 554 在建占位）。
pub(crate) const POTJIEJIE_CUSTOM_CHOICE: u32 = 0x504a_4a45;
pub(crate) const POTQINGSHI_CUSTOM_CHOICE: u32 = 0x5051_5348;
pub(crate) const POTWEIYAN_CUSTOM_CHOICE: u32 = 0x5057_594e;
pub(crate) const POTZHOUYU_CUSTOM_CHOICE: u32 = 0x505a_5955;
pub(crate) const POTLVYI_CUSTOM_CHOICE: u32 = 0x504c_5659;
pub(crate) const POTCAOSHUANG_CUSTOM_CHOICE: u32 = 0x5043_5348;

pub(crate) const POTWEIYAN_KUANGGU_DEF: u32 = 4_200_000_562;
pub(crate) const POTWEIYAN_ZHUANGSHI_DEF: u32 = 4_200_000_560;
pub(crate) const POTWEIYAN_KUNFEN_DEF: u32 = 3_536_332_483;
pub(crate) const POTWEIYAN_DIRECT_DEF: u32 = 4_200_000_563;
pub(crate) const POTWEIYAN_LIMIT_DEF: u32 = 4_200_000_564;
pub(crate) const POTWEIYAN_KUANGGU_LIMIT_DEF: u32 = 4_200_000_565;

pub(crate) fn skill_instance_by_def(
    tx: &Transaction,
    player: PlayerId,
    def: engine_api::SkillDefId,
) -> Option<engine_api::SkillId> {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == player && instance.def == def)
        .map(|instance| instance.id)
}

pub(crate) fn state_u32(tx: &Transaction, skill: engine_api::SkillId, offset: usize) -> u32 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| instance.state_blob.get(offset..offset + 4))
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .unwrap_or(0)
}

pub(crate) fn set_state_u32(tx: &mut Transaction, skill: engine_api::SkillId, offset: usize, value: u32) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        if instance.state_blob.len() < offset + 4 {
            instance.state_blob.resize(offset + 4, 0);
        }
        instance.state_blob[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mbzhonghui(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    amount: u32,
    mbsizi_def: engine_api::SkillDefId,
    mbkechang_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let charge = pottuntian_charge(tx, player, mbsizi_def.raw());
    match action {
        0 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || charge == 0
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                (1..=charge.min(4)).collect(),
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    MBZHONGHUI_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    mbsizi_def.raw(),
                    mbkechang_def.raw(),
                ],
            )
        }
        1 => {
            if charge >= 4 {
                Ok(Vec::new())
            } else {
                add_mark(tx, player, mbsizi_def.raw(), (4 - charge) as i32)
            }
        }
        2 => {
            let countdown = state_u32(tx, skill, 0);
            if countdown == 0 {
                return Ok(Vec::new());
            }
            set_state_u32(tx, skill, 0, countdown - 1);
            let users: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    tx.state()
                        .turn
                        .flag(turn_flag::sha_used_by_player(*candidate))
                        > 0
                })
                .collect();
            let mut effects = vec![EffectRequest::DrawCards { player, count: 2 }];
            effects.extend(users.iter().map(|candidate| EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: *candidate,
                amount: 1,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            }));
            if users.is_empty()
                && tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.state_blob.get(4) == Some(&1))
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                effects.push(EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                });
            }
            run_effects(tx, effects)
        }
        3 => {
            let damage_id = tx.state().resolution.current_damage_id;
            if state_u32(tx, skill, 0) > 0
                && bound_sha_damage_matches(tx, damage_id, player, target)
            {
                increase_bound_damage_by_one(tx, damage_id)?;
            }
            Ok(Vec::new())
        }
        4 => {
            let gained = amount.min(4_u32.saturating_sub(charge));
            let mut events = add_mark(tx, player, mbsizi_def.raw(), gained as i32)?;
            if amount > gained {
                set_state_u32(tx, skill, 0, state_u32(tx, skill, 0).saturating_add(1));
                events.push(DomainEvent::RuleNote {
                    code: 553,
                    payload: vec![player.raw(), skill.raw()],
                });
            }
            Ok(events)
        }
        5 => {
            set_state_u32(tx, skill, 0, tx.state().turn.round.saturating_add(1));
            Ok(Vec::new())
        }
        6 => {
            if state_u32(tx, skill, 4) != 0
                || state_u32(tx, skill, 0) != tx.state().turn.round.saturating_add(1)
            {
                return Ok(Vec::new());
            }
            set_state_u32(tx, skill, 4, 1);
            tx.state_mut().group_overrides.insert(player, "qun".into());
            let mut events = if skill_instance_by_def(tx, player, mbkechang_def).is_none() {
                add_skill(tx, player, mbkechang_def)?
            } else {
                Vec::new()
            };
            if let Some(kechang) = skill_instance_by_def(tx, player, mbkechang_def) {
                if let Some(instance) = tx.state_mut().skills.get_mut(kechang) {
                    instance.state_blob = vec![1];
                }
            }
            events.push(DomainEvent::RuleNote {
                code: 553,
                payload: vec![player.raw(), target.raw(), mbkechang_def.raw()],
            });
            Ok(events)
        }
        7 => {
            if tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.state_blob.first() == Some(&1))
            {
                make_current_sha_target_unrespondable(tx, player, CardId::new(amount), target)
            } else {
                Ok(Vec::new())
            }
        }
        8 => {
            if amount == 0 || amount > charge.min(4) {
                return Err(EffectError::Illegal("俟资蓄力消耗已变化".into()));
            }
            add_mark(tx, player, mbsizi_def.raw(), -(amount as i32))?;
            set_state_u32(tx, skill, 0, amount);
            let hp = tx
                .state()
                .players
                .get(player)
                .map_or(0, |p| p.hp.max(0) as u32);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                if instance.state_blob.len() < 5 {
                    instance.state_blob.resize(5, 0);
                }
                instance.state_blob[4] = u8::from(amount > hp);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("unknown 势钟会 action".into())),
    }
}

pub(crate) fn resolve_potjiejie(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    used_def: engine_api::SkillDefId,
    suit_defs: [engine_api::SkillDefId; 4],
    qingshi_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != actor
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|state| state.alive)
        || tx.state().hand_cards(actor).is_empty()
        || skill_instance_by_def(tx, actor, used_def).is_some()
    {
        return Ok(Vec::new());
    }
    if [used_def, qingshi_def]
        .into_iter()
        .chain(suit_defs)
        .any(|def| def.is_none() || tx.state().content.skill_metadata(def).is_none())
    {
        return Err(EffectError::ContentFault(
            "诫节 helper definition missing".into(),
        ));
    }
    let hand = tx.state().hand_cards(actor);
    let suits = hand.iter().fold(0u8, |mask, card| {
        mask | match tx
            .state()
            .cards
            .get(*card)
            .map(|card| crate::query::effective_card_suit(tx.state(), actor, card))
        {
            Some(Suit::Spade) => 1,
            Some(Suit::Heart) => 2,
            Some(Suit::Club) => 4,
            Some(Suit::Diamond) => 8,
            _ => 0,
        }
    });
    let round = tx.state().turn.round;
    let alive_players = tx.state().players.alive_by_seat();
    let bonus = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| {
            instance.record_potjiejie_suits(round, actor, suits, &alive_players)
        });
    let mut events = add_skill_until_turn_end(tx, actor, used_def)?;
    events.push(DomainEvent::CardsRevealed {
        cards: hand,
        to: vec![player],
    });
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![0, 1, 2, 3],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            POTJIEJIE_CUSTOM_CHOICE,
            player.raw(),
            actor.raw(),
            skill.raw(),
            qingshi_def.raw(),
            suit_defs[0].raw(),
            suit_defs[1].raw(),
            suit_defs[2].raw(),
            suit_defs[3].raw(),
            u32::from(bonus),
        ],
    )?);
    Ok(events)
}

pub(crate) fn resolve_potqingshi(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets = tx.state().players.alive_by_seat();
    GuardedChoice::new(
        skill,
        Participants::owner_any_state(player),
        ContinuationKind::Custom,
        vec![POTQINGSHI_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
    .shape(ChoiceShape::one_optional())
    .open_targets(tx, targets)
}

pub(crate) fn resolve_zhengrong(
    tx: &mut Transaction,
    player: PlayerId,
    candidates: &[PlayerId],
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let owner_hand = tx.state().hand_count(player);
            let richer_with_cards: Vec<_> = candidates
                .iter()
                .copied()
                .filter(|&candidate| {
                    candidate != player
                        && tx.state().players.get(candidate).is_some_and(|p| p.alive)
                        && tx.state().hand_count(candidate) >= owner_hand
                        && !crate::query::discardable_hand_equip_cards(tx.state(), candidate)
                            .is_empty()
                })
                .collect();
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_ZHENGRONG_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    PlayerId::NONE.raw(),
                    skill.raw(),
                ],
            )
            .open_targets(tx, richer_with_cards)
        }
        _ => Err(stage_must_resume_via_continuation("征荣")),
    }
}

pub(crate) fn resolve_hongju(
    tx: &mut Transaction,
    player: PlayerId,
    qingjian_def: engine_api::SkillDefId,
    rong_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 业务前置：荣标记不足或已觉醒（持有清俭）则不再结算。
            if tx.state().skill_pile_cards(player, rong_def.raw()).len() < 3 {
                return Ok(Vec::new());
            }
            if tx
                .state()
                .skills
                .sorted_live()
                .iter()
                .any(|instance| instance.owner == player && instance.def == qingjian_def)
            {
                return Ok(Vec::new());
            }
            // 技能启用 + 存活 + 自己回合由 GuardedChoice 守卫（ADR 0595）。
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_HONGJU_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    qingjian_def.raw(),
                    rong_def.raw(),
                    skill.raw(),
                ],
            )
            .during(TurnContext::OwnersTurn)
            .shape(ChoiceShape::one_optional())
            .open_tokens(tx, vec![1, 0], ChoiceKind::Custom)
        }
        _ => Err(EffectError::ContentFault(
            "鸿举 stage must resolve through its choice continuation".into(),
        )),
    }
}

pub(crate) fn resolve_qingjian(
    tx: &mut Transaction,
    player: PlayerId,
    rong_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 荣牌堆有牌才能赠，且赠牌需要有手牌可换。
            if tx.state().hand_cards(player).is_empty() {
                return Ok(Vec::new());
            }
            let rong = tx.state().skill_pile_cards(player, rong_def.raw());
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_QINGJIAN_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    rong_def.raw(),
                    skill.raw(),
                ],
            )
            .during(TurnContext::OwnersPhase(Phase::Play))
            .open_cards(tx, rong)
        }
        _ => Err(stage_must_resume_via_continuation("清俭")),
    }
}

pub(crate) fn resolve_releiji(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let others: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .collect();
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_RELEIJI_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    PlayerId::NONE.raw(),
                    skill.raw(),
                ],
            )
            .open_targets(tx, others)
        }
        _ => Err(stage_must_resume_via_continuation("雷击")),
    }
}

pub(crate) fn resolve_huangtian(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 仅群雄角色可响应黄天。
            if tx.state().player_group(actor) != Some("qun") {
                return Ok(Vec::new());
            }
            let shan_like: Vec<_> = tx
                .state()
                .hand_cards(actor)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|value| {
                        matches!(
                            crate::query::effective_card_kind(tx.state(), actor, value),
                            CardKind::Shan | CardKind::TrickShandian
                        )
                    })
                })
                .collect();
            GuardedChoice::new(
                skill,
                Participants::pair(player, actor),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_HUANGTIAN_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
            .chosen_by(actor)
            .shape(ChoiceShape::one_optional())
            .open_cards(tx, shan_like)
        }
        _ => Err(stage_must_resume_via_continuation("黄天")),
    }
}

pub(crate) fn resolve_haoshi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Draw
                || tx.state().turn.draw_phase_is_fixed()
            {
                return Ok(Vec::new());
            }
            let count = tx
                .state()
                .turn
                .draw_phase_fixed_count()
                .unwrap_or(2)
                .saturating_add(2);
            tx.state_mut().turn.set_draw_phase_fixed_count(count);
            Ok(Vec::new())
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().hand_count(player) <= 5
            {
                return Ok(Vec::new());
            }
            let min_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .map(|candidate| (candidate, tx.state().hand_count(candidate)))
                .min_by_key(|(_, count)| *count)
                .map(|(candidate, _)| candidate);
            let Some(target) = min_hand else {
                return Ok(Vec::new());
            };
            let give = tx.state().hand_count(player) / 2;
            if give == 0 {
                return Ok(Vec::new());
            }
            let give_u8 = give.min(u8::MAX as u32) as u8;
            // 目标已确定（手牌最少者），直接开选牌。
            open_choice(
                tx,
                player,
                give_u8,
                give_u8,
                false,
                tx.state()
                    .hand_cards(player)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_HAOSHI_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "好施 stage must resolve through its choice continuation".into(),
        )),
    }
}

pub(crate) fn resolve_dimeng(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let others: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .collect();
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![RESOLVE_DIMENG_CUSTOM_CHOICE, 1, player.raw(), skill.raw()],
            )
            .during(TurnContext::OwnersPhase(Phase::Play))
            .shape(ChoiceShape::exactly(2))
            .open_targets(tx, others)
        }
        _ => Err(stage_must_resume_via_continuation("缔盟")),
    }
}

pub(crate) fn exclude_saved_sha_target(
    tx: &mut Transaction,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|player| player.alive)
        || !tx.state().cards.get(card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), target, value).is_sha()
        })
    {
        return Err(EffectError::Illegal(
            "exclude-saved-sha-target participant no longer legal".into(),
        ));
    }
    if !tx
        .state_mut()
        .resolution
        .exclude_saved_sha_target(card, target)
    {
        return Err(EffectError::Illegal(
            "no matching currently resolving Sha target".into(),
        ));
    }
    Ok(vec![DomainEvent::RuleNote {
        code: 560,
        payload: vec![card.raw(), target.raw()],
    }])
}

pub(crate) fn resolve_qiaobian(
    tx: &mut Transaction,
    player: PlayerId,
    phase: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // phase 参数选中要跳过的阶段，须与当前阶段一致。
            let phase_ok = match phase {
                0 => tx.state().turn.phase == Phase::Judge,
                1 => tx.state().turn.phase == Phase::Draw,
                2 => tx.state().turn.phase == Phase::Play,
                3 => tx.state().turn.phase == Phase::Discard,
                _ => false,
            };
            if !phase_ok {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_QIAOBIAN_DISCARD,
                    1,
                    u32::from(phase),
                    player.raw(),
                    skill.raw(),
                ],
            )
            .during(TurnContext::OwnersTurn)
            .shape(ChoiceShape::one_optional())
            .open_cards(tx, hand)
        }
        _ => Err(stage_must_resume_via_continuation("巧变")),
    }
}

pub(crate) fn resolve_xiangle(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let choice = GuardedChoice::new(
                skill,
                Participants::distinct_pair(player, source),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XIANGLE_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    source.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
            .chosen_by(source)
            .shape(ChoiceShape::one_optional());
            if !choice.guard_passes(tx)
                || !tx.state().cards.get(card).is_some_and(|value| {
                    crate::query::effective_card_kind(tx.state(), source, value).is_sha()
                })
            {
                return Ok(Vec::new());
            }
            let basics: Vec<_> = tx
                .state()
                .hand_cards(source)
                .into_iter()
                .filter(|&candidate| {
                    tx.state()
                        .cards
                        .get(candidate)
                        .is_some_and(|value| value.kind.is_basic())
                })
                .collect();
            if basics.is_empty() {
                // 无可弃基本牌：直接令此杀无效。
                return exclude_saved_sha_target(tx, card, player);
            }
            choice.open_cards(tx, basics)
        }
        _ => Err(stage_must_resume_via_continuation("享乐")),
    }
}

/// 放权：跳过本回合出牌阶段并标记结束阶段可授额外回合。
pub(crate) fn resolve_fangquan_skip(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.state_blob == b"FQ1")
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("放权 skill disappeared".into()))?
        .state_blob = b"FQ1".to_vec();
    run_effects(tx, vec![EffectRequest::SkipCurrentPhase { player }])
}

/// 放权：结束阶段弃一张手牌并令一名其他角色进行一个额外回合。
pub(crate) fn resolve_fangquan_grant(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 仅在本回合已跳过出牌阶段（FQ1）后于结束阶段兑现。
            if !tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.state_blob == b"FQ1")
            {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![RESOLVE_FANGQUAN_DISCARD, 1, player.raw(), skill.raw()],
            )
            .during(TurnContext::OwnersPhase(Phase::End))
            .shape(ChoiceShape::one_optional())
            .open_cards(tx, hand)
        }
        _ => Err(stage_must_resume_via_continuation("放权")),
    }
}

/// 若愚（主公技）：准备阶段体力值为全场最少时觉醒，加上限、回复并授予激将。
pub(crate) fn resolve_ruoyu(
    tx: &mut Transaction,
    player: PlayerId,
    jijiang_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Prepare
        || tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.state_blob == b"RY1")
    {
        return Ok(Vec::new());
    }
    let hp = tx
        .state()
        .players
        .get(player)
        .map(|p| p.hp)
        .unwrap_or(i32::MAX);
    let min_hp = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|candidate| {
            tx.state()
                .players
                .get(candidate)
                .map(|p| p.hp)
                .unwrap_or(i32::MAX)
        })
        .min()
        .unwrap_or(hp);
    if hp != min_hp {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("若愚 skill disappeared".into()))?
        .state_blob = b"RY1".to_vec();
    run_effects(
        tx,
        vec![
            EffectRequest::ChangeMaxHp { player, delta: 1 },
            EffectRequest::RecoverHp {
                player,
                amount: 1,
                source: PlayerId::NONE,
            },
            EffectRequest::AddSkill {
                player,
                skill_def: jijiang_def,
            },
        ],
    )
}

/// 罪论：揭示牌堆顶三张并按条件计数分派；N==3 时选择一名其他角色一起失体力。
pub(crate) fn resolve_zuilun(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::End
            {
                return Ok(Vec::new());
            }
            let lost = tx.state().turn.flag(turn_flag::DISCARD_PHASE_DISCARDED) >= 1
                || tx
                    .state()
                    .turn
                    .lost_cards
                    .iter()
                    .any(|(owner, _)| *owner == player);
            let hand = tx.state().hand_count(player);
            let min_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .all(|candidate| tx.state().hand_count(candidate) >= hand);
            let undamaged = !tx.state().turn.player_received_damage(player);
            let count = u32::from(lost) + u32::from(min_hand) + u32::from(undamaged);
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .is_none_or(|zone| zone.cards.is_empty())
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let revealed = tx
                .state()
                .zones
                .get(draw)
                .map(|zone| zone.cards.iter().rev().take(3).copied().collect::<Vec<_>>())
                .unwrap_or_default();
            if revealed.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
            for &card in &revealed {
                move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
            }
            let hand_zone = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            if count == 0 {
                return crate::effect::run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: revealed,
                        to: hand_zone,
                        from: Some(processing),
                    }],
                );
            }
            if count == 3 {
                // 全部置顶，并选择一名其他角色一起失去 1 点体力。
                crate::effect::run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: revealed.clone(),
                        to: draw,
                        from: Some(processing),
                    }],
                )?;
                let options: Vec<_> = tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|&candidate| candidate != player)
                    .collect();
                if options.is_empty() {
                    return Ok(Vec::new());
                }
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    true,
                    options.into_iter().map(PlayerId::raw).collect(),
                    ChoiceKind::SelectTargets,
                    ContinuationKind::Custom,
                    vec![RESOLVE_ZUILUN_TARGET, 2, player.raw(), skill.raw()],
                );
            }
            let mut payload = vec![RESOLVE_ZUILUN_PICK, 1, player.raw(), skill.raw(), count];
            payload.extend(revealed.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                count.min(u8::MAX as u32) as u8,
                count.min(u8::MAX as u32) as u8,
                false,
                revealed.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "罪论 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 父荫：本回合首次被手牌数不小于自己的其他角色以【杀】指定为目标时，此杀无效。
pub(crate) fn resolve_fuyin(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || !tx.state().players.get(source).is_some_and(|p| p.alive)
        || source == player
        || !tx.state().cards.get(card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), source, value).is_sha()
        })
        || tx.state().hand_count(source) < tx.state().hand_count(player)
    {
        return Ok(Vec::new());
    }
    let round = tx.state().turn.round;
    let used_this_round = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 7 && &instance.state_blob[..3] == b"FY1").then(|| {
                (u32::from_le_bytes(
                    instance.state_blob[3..7].try_into().unwrap(),
                ),)
            })
        })
        .is_some_and(|(marked_round,)| marked_round == round);
    if used_this_round {
        return Ok(Vec::new());
    }
    let mut blob = b"FY1".to_vec();
    blob.extend_from_slice(&round.to_le_bytes());
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("父荫 skill disappeared".into()))?
        .state_blob = blob;
    exclude_saved_sha_target(tx, card, player)
}

/// 回合内禁用指定角色对另一角色使用牌。
pub(crate) fn ban_use_against(
    tx: &mut Transaction,
    user: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if user == target
        || !tx.state().players.get(user).is_some_and(|p| p.alive)
        || !tx.state().players.get(target).is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal(
            "ban-use-against participant no longer legal".into(),
        ));
    }
    let pairs = &mut tx.state_mut().turn.use_ban_pairs;
    if !pairs.contains(&(user, target)) {
        pairs.push((user, target));
    }
    Ok(vec![DomainEvent::RuleNote {
        code: 571,
        payload: vec![user.raw(), target.raw()],
    }])
}

/// 拒战（转换技）：阴——受杀各摸一张并禁来源用牌；阳——用杀获目标一张牌并禁己方用牌。
pub(crate) fn resolve_juzhan(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || !tx.state().cards.get(card).is_some_and(|value| {
                    crate::query::effective_card_kind(tx.state(), source, value).is_sha()
                })
            {
                return Ok(Vec::new());
            }
            // 转换技分支：技能状态首字节为 0（阴，受杀）或 1（阳，用杀）。
            let branch = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| {
                    (instance.state_blob.len() == 4 && &instance.state_blob[..3] == b"JZ1")
                        .then(|| instance.state_blob[3])
                })
                .unwrap_or(0);
            let mut blob = b"JZ1".to_vec();
            blob.push(if branch == 0 { 1 } else { 0 });
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("拒战 skill disappeared".into()))?
                .state_blob = blob;
            let target = if branch == 0 {
                // 阴：source 为杀使用者。
                source
            } else {
                // 阳：source 为杀目标（事件来源之外取首个声明目标）。
                targets
                    .iter()
                    .copied()
                    .find(|&candidate| candidate != player && candidate == source)
                    .or_else(|| {
                        targets
                            .iter()
                            .copied()
                            .find(|&candidate| candidate != player)
                    })
                    .unwrap_or(PlayerId::NONE)
            };
            if branch == 0 {
                // 双方各摸一张；来源本回合不能对持有者使用牌。
                let mut events = Vec::new();
                events.extend(run_effects(
                    tx,
                    vec![
                        EffectRequest::DrawCards { player, count: 1 },
                        EffectRequest::DrawCards {
                            player: source,
                            count: 1,
                        },
                    ],
                )?);
                events.extend(ban_use_against(tx, source, player)?);
                return Ok(events);
            }
            if target == PlayerId::NONE || !tx.state().players.get(target).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            // 阳：获得目标一张手牌/装备牌，随后持有者本回合不能对目标使用牌。
            let options: Vec<_> = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .chain(
                    tx.state()
                        .zones
                        .find(ZoneKind::Equip, target)
                        .map(|zone| {
                            tx.state()
                                .zones
                                .get(zone)
                                .map(|zone| zone.cards.clone())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                )
                .collect();
            if options.is_empty() {
                return ban_use_against(tx, player, target);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_JUZHAN_GAIN,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "拒战 stage must resolve through its choice continuation".into(),
        )),
    }
}
