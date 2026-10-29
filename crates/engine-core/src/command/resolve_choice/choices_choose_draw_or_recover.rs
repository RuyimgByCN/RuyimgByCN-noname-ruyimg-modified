//! Choice continuations for `ChooseDrawOrRecover` and related requests.

use super::*;

pub(super) fn resolve_choice(
    tx: &mut Transaction,
    _actor: PlayerId,
    observer: PlayerId,
    kind: ContinuationKind,
    payload: Vec<u32>,
    selections: &[u32],
    cancelled: bool,
    _vm_cont: (Vec<u8>, String, u32, u32, [u32; 4], u32, [u32; 3]),
    _optional_trigger: Option<crate::trigger::PendingTrigger>,
) -> Result<(), EffectError> {
    match kind {
        ContinuationKind::VirtualShaTargets => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let card = CardId::new(*payload.get(1).unwrap_or(&0));
            let ignore_distance = *payload.get(2).unwrap_or(&0) != 0;
            let ignore_use_count = *payload.get(3).unwrap_or(&0) != 0;
            let draw_on_no_use = *payload.get(4).unwrap_or(&0) != 0;
            let skill = engine_api::SkillId::new(*payload.get(5).unwrap_or(&0));
            let lose_hp_before_use = *payload.get(6).unwrap_or(&0);
            let damage_bonus = *payload.get(7).unwrap_or(&0);
            if cancelled {
                let processing = tx
                    .state()
                    .zones
                    .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
                let removed = tx
                    .state()
                    .zones
                    .find(crate::state::ZoneKind::Removed, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
                crate::effect::move_one_card(tx, card, processing, removed, PlayerId::NONE, true)?;
                return if draw_on_no_use {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCardsFromSkill {
                            player,
                            count: 1,
                            skill,
                        }],
                    )
                    .map(|_| ())
                } else {
                    Ok(())
                };
            }
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            if lose_hp_before_use > 0 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
                        player,
                        card,
                        targets,
                        stage: 1,
                        damage_bonus,
                        skill,
                    },
                );
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: lose_hp_before_use,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )
                .map(|_| ());
            }
            super::super::play::handle_skill_card_with_policies(
                tx,
                player,
                card,
                &targets,
                ignore_distance,
                ignore_use_count,
            )
        }

        ContinuationKind::JudgeDamageOtherOnSuit => {
            if cancelled {
                return Err(EffectError::Illegal(
                    "judgment damage target choice cannot cancel".into(),
                ));
            }
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let amount = *payload.get(1).unwrap_or(&0);
            let nature = *payload.get(2).unwrap_or(&0) as u8;
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "choose exactly one judgment damage target".into(),
                ));
            };
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target: PlayerId::new(*target),
                    amount,
                    is_lose_hp: false,
                    nature,
                }],
            )
            .map(|_| ())
        }

        ContinuationKind::GanglieSourceDiscardOrDamage => {
            let [player, source, rest @ ..] = payload.as_slice() else {
                return Err(EffectError::EngineFault(
                    "bad Ganglie discard continuation".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let source = PlayerId::new(*source);
            let hand_only = rest.first().copied().unwrap_or(1) != 0;
            if observer != source
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Ganglie discard context changed".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Ganglie discard contains cards".into(),
                    ));
                }
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target: source,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )
                .map(|_| ());
            }
            let available = crate::effect::ganglie_discardable_cards(tx, source, hand_only);
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards.len() != 2
                || cards[0] == cards[1]
                || cards.iter().any(|card| !available.contains(card))
            {
                return Err(EffectError::Illegal(
                    "Ganglie requires exactly two discardable cards".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: source,
                    cards,
                }],
            )
            .map(|_| ())
        }

        ContinuationKind::ResolveGanglieThree => {
            let [player, skill] = payload.as_slice() else {
                return Err(EffectError::EngineFault(
                    "bad Ganglie Three target continuation".into(),
                ));
            };
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Ganglie Three requires one enemy target".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveGanglieThree {
                    player: PlayerId::new(*player),
                    target: PlayerId::new(selections[0]),
                    skill: engine_api::SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }

        ContinuationKind::TuxiGainOtherHandsReplaceDraw => {
            resolve_tuxi_gain_other_hands_replace_draw(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::YijiDistributeTopCards => {
            resolve_yiji_distribute_top_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage => {
            resolve_xiaoguo_discard_basic_then_equip_or_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ResolveSpXiaoguo => {
            resolve_sp_xiaoguo(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTanlu => {
            resolve_tanlu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner => {
            resolve_target_discard_then_duel(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::TongjiDiscardRedirectSha => {
            resolve_tongji_discard_redirect_sha(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::LiuliDiscardRedirectSha => {
            resolve_liuli_discard_redirect_sha(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::JijieChooseBottomCardRecipient => {
            resolve_jijie_choose_bottom_card_recipient(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::LuoshenRepeatBlackJudgment => {
            resolve_luoshen_repeat_black_judgment(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::JudgeSameColorGainRepeat => {
            resolve_judge_same_color_gain_repeat(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::DelegatedGroupResponse => {
            resolve_delegated_group_response(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::MoveEquipmentThenDraw => {
            resolve_move_equipment_then_draw(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::DiscardEquipmentPairWithArmor => {
            resolve_discard_equipment_pair_with_armor(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::DiscardTargetEquipmentGainMount => {
            resolve_discard_target_equipment_gain_mount(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::DiscardTargetEquipmentDrawWeaponRange => {
            resolve_discard_target_equipment_draw_weapon_range(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ResolveOlJieDanshou => {
            crate::effect::resolve_ol_jie_danshou_choice(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::GuanxingPick => {
            resolve_guanxing_pick(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::RecoverToMaxThenOptionalExchangeSkill => {
            resolve_recover_to_max_then_optional_exchange_skill(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::DiscardTargetCardGainMount => {
            resolve_discard_target_card_gain_mount(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveDcqiaomeng => {
            resolve_dcqiaomeng(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveRetieji => {
            resolve_retieji(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveNewReluoyi => {
            resolve_new_reluoyi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveXinyaoming => {
            resolve_xinyaoming(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveRehuaiyi => {
            resolve_rehuaiyi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveReanguo => {
            resolve_reanguo(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveBeizhu => {
            resolve_beizhu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveMbshanjia => {
            resolve_mbshanjia(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveDaoji => {
            resolve_daoji(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::GiveAllHandThenChooseUseOneOrDamage => {
            resolve_give_all_hand_then_choose_use_one_or_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ResolveTwfupan => {
            resolve_twfupan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwruilian => {
            resolve_twruilian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwjianwei => {
            resolve_twjianwei(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwtanfeng => {
            resolve_twtanfeng(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwzhibian => {
            resolve_twzhibian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwyuyan => {
            resolve_twyuyan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwguoyi => {
            resolve_twguoyi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwchuhai => {
            resolve_twchuhai(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwyizhu => {
            resolve_twyizhu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwxiawei => {
            resolve_twxiawei(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwbianfuren => {
            resolve_twbianfuren(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwfeiyi => {
            resolve_twfeiyi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwchenzhen => {
            resolve_twchenzhen(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwqingxi => {
            resolve_twqingxi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwzaoli => {
            resolve_twzaoli(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwzhengjian => {
            resolve_twzhengjian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwtianyu => {
            resolve_twtianyu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwwujing => {
            resolve_twwujing(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwshanghe => {
            resolve_twshanghe(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwwangchang => {
            resolve_twwangchang(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwcaozhao => {
            resolve_twcaozhao(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwguohuai => {
            resolve_twguohuai(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ClanshenjunUseCard => crate::effect::resolve_clanshenjun_use_choice(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::ClanyuzhiRevealChoice => crate::effect::resolve_clanyuzhi_reveal_choice(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::ClanyuzhiEndChoice => crate::effect::resolve_clanyuzhi_end_choice(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::QieyiPutTopChoice => crate::effect::resolve_qieyi_put_top_choice(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::JianzhiCountChoice => crate::effect::resolve_jianzhi_count_choice(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::JianzhiAssignCard => crate::effect::resolve_jianzhi_assign_card(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::ResolveJiaohua => {
            resolve_jiaohua(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveLuanqun => {
            resolve_luanqun(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp => {
            resolve_give_selected_then_target_choose_discard_same_suit_or_lose_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::FanjianChooseSuitOrHand => {
            resolve_fanjian_choose_suit_or_hand(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveMbzhuguo => {
            resolve_mbzhuguo(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveMbanxian => {
            resolve_mbanxian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveMbjinzu => {
            resolve_mbjinzu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ForceOthersUseShaOnNearestOrLoseHp => {
            resolve_force_others_use_sha_on_nearest_or_lose_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ResolveOllinjie => {
            resolve_ollinjie(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::GainTargetHandCard => {
            resolve_gain_target_hand_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::GainFromSkillPile => {
            resolve_gain_from_skill_pile(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::StartDebateTargets => {
            resolve_start_debate_targets(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::PinDebateOpinions => crate::effect::resolve_pin_debate_opinions(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::DistributeShadowCards => crate::effect::resolve_distribute_shadow_cards(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::RegisterTurnEndDebate => crate::effect::resolve_register_turn_end_debate(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::LoseGainedSkill => {
            crate::effect::resolve_lose_gained_skill(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolvePotheyun => {
            crate::effect::resolve_potheyun_pick(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::CityConsume => {
            crate::effect::resolve_city_consume(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::TwjieqiuSelect => {
            crate::effect::resolve_twjieqiu_select(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::DiscardBasicDrawEqual => {
            resolve_discard_basic_draw_equal(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::PlaceHandCardToExpansion => {
            resolve_place_hand_card_to_expansion(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::BetweenOpponent => {
            resolve_between_opponent(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::BetweenFirstCard => {
            resolve_between_first_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::BetweenSecondCard => {
            resolve_between_second_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::DrawThenOptionallyDistributeHandCards => {
            resolve_draw_then_optionally_distribute_hand_cards(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::GiveSelectedThenContinueDistinctRecoverAtTwo => {
            resolve_give_selected_then_continue_distinct_recover_at_two(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::DiscardTiaojiyanmeiCard => {
            resolve_discard_tiaojiyanmei_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveGuanwei => {
            resolve_guanwei(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveLanjiang => {
            resolve_lanjiang(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveRebingyi => {
            resolve_rebingyi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOldimeng => {
            resolve_oldimeng(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveRejieyin => {
            resolve_rejieyin(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveReanxu => {
            resolve_reanxu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOldAnxu => {
            resolve_oldanxu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOldQinqing => {
            resolve_oldqinqing(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOldMingjian => {
            resolve_oldmingjian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOldXuanfeng => {
            resolve_oldxuanfeng(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveJianrou => {
            resolve_jianrou(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOldHuisheng => {
            resolve_oldhuisheng(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOldJiefan => {
            resolve_oldjiefan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveChegu => {
            resolve_chegu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveJunkguixin => {
            resolve_junkguixin(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveGuhuo => {
            resolve_guhuo(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveFaceDownViewAs => {
            resolve_face_down_view_as(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveSwapLowerHandViewAs => {
            resolve_swap_lower_hand_view_as(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveYaoChenwei => {
            resolve_yao_chenwei(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveYoudi => {
            resolve_youdi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveKuangfu => {
            resolve_kuangfu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveShanjia => {
            resolve_shanjia(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveNewYajiao => {
            resolve_new_yajiao(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::GainOneHandFromSelectedTargets => {
            resolve_gain_one_hand_from_selected_targets(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ResolveZhuiyi => {
            resolve_zhuiyi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveXinfuGuolun => {
            resolve_xinfu_guolun(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveDrltJueyan => {
            resolve_drlt_jueyan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveRejizhi => {
            resolve_rejizhi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveJiexun => {
            resolve_jiexun(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveWengua => {
            resolve_wengua(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::DiscardHandForMassDamage => {
            if cancelled || payload.len() < 3 {
                return Err(EffectError::Illegal(
                    "mass hand discard cannot be cancelled".into(),
                ));
            }
            let player = PlayerId::new(payload[0]);
            let required = payload[1] as usize;
            let option_count = payload[2] as usize;
            let (hand_equip, option_start) = match payload.len() {
                len if len == 3 + option_count => (false, 3),
                len if len == 4 + option_count && payload[3] == 1 => (true, 4),
                _ => return Err(EffectError::Illegal("invalid mass discard payload".into())),
            };
            if player != observer || required == 0 || selections.len() != required {
                return Err(EffectError::Illegal(
                    "invalid mass hand discard selection".into(),
                ));
            }
            let original: std::collections::BTreeSet<_> =
                payload[option_start..].iter().copied().collect();
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let current: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .into_iter()
                    .filter(|card| {
                        hand_equip
                            || hand.is_some_and(|zone| {
                                tx.state()
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.zone == zone)
                            })
                    })
                    .map(CardId::raw)
                    .collect();
            let mut selected = std::collections::BTreeSet::new();
            if !selections.iter().all(|card| {
                selected.insert(*card) && original.contains(card) && current.contains(card)
            }) {
                return Err(EffectError::Illegal(
                    "mass hand discard card is duplicated or no longer legal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }

        ContinuationKind::DiscardMarkParityControlTargetCard => {
            resolve_discard_mark_parity_control_target_card(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ConfirmMarkParityMassDamage => {
            resolve_confirm_mark_parity_mass_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::GiveExactRecordPhaseTarget => {
            resolve_give_exact_record_phase_target(tx, observer, &payload, selections, cancelled)
        }

        _ => unreachable!("choice routed to a different continuation group"),
    }
}
