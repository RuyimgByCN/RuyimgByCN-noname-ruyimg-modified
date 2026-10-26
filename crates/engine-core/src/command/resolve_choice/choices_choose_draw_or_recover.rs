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
        ContinuationKind::ChooseDrawOrRecover => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let draw_count = *payload.get(1).unwrap_or(&1);
            let recover_count = *payload.get(2).unwrap_or(&1);
            let allow_cancel = *payload.get(3).unwrap_or(&0) != 0;
            let recover_target = PlayerId::new(payload.get(4).copied().unwrap_or(player.raw()));
            if cancelled {
                return if allow_cancel {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "draw-or-recover choice cannot cancel".into(),
                    ))
                };
            }
            let effect = match selections {
                [1] => EffectRequest::DrawCards {
                    player,
                    count: draw_count,
                },
                [2] => EffectRequest::RecoverHp {
                    source: recover_target,
                    player: recover_target,
                    amount: recover_count,
                },
                _ => {
                    return Err(EffectError::Illegal(
                        "choose draw or recover exactly once".into(),
                    ));
                }
            };
            run_effects(tx, vec![effect]).map(|_| ())
        }

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

        ContinuationKind::ChooseDingpanBranch => {
            resolve_choose_dingpan_branch(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTanlu => {
            resolve_tanlu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner => {
            resolve_target_discard_then_duel(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTargetLoseHpDiscardHandShaIfBasic => {
            resolve_choose_target_lose_hp_discard_hand_sha_if_basic(
                tx, observer, &payload, selections, cancelled,
            )
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

        ContinuationKind::ChooseTargetRevealHandDamageIfSha => {
            resolve_choose_target_reveal_hand_damage_if_sha(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOtherGiveOneHandEquip => {
            resolve_choose_other_give_one_hand_equip(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGiveOrRecastOneHand => {
            resolve_choose_give_or_recast_one_hand(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOlJieDanshou => {
            crate::effect::resolve_ol_jie_danshou_choice(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOtherGiveThenRecover => {
            resolve_choose_other_give_then_recover(tx, observer, &payload, selections, cancelled)
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

        ContinuationKind::ChooseDiscardTargetCard => {
            resolve_choose_discard_target_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardTargetCardResolveCurrentSha => {
            resolve_choose_discard_target_card_current_sha(
                tx, observer, &payload, selections, cancelled,
            )
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

        ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable => {
            resolve_choose_category_then_target_give_matching_or_sha_unrespondable(
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

        ContinuationKind::ChooseTargetDrawChoice => {
            // Delegate to the concurrent process's resolve_choose_target_draw
            crate::command::resolve_choice::group_post_3::resolve_choose_target_draw(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ResolveJiaohua => {
            resolve_jiaohua(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveLuanqun => {
            resolve_luanqun(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTargetReclaimEquipmentDrawIfRangeReduced => {
            resolve_choose_target_reclaim_equipment_draw_if_range_reduced(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseUseVirtualCard => {
            resolve_choose_use_virtual_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardOneThenGainTargetEquipment => {
            resolve_choose_discard_one_then_gain_target_equipment(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDraw => {
            resolve_choose_target_draw(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTargetsDamage => {
            resolve_choose_targets_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp => {
            resolve_choose_target_draw_then_owner_if_below_max_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess => {
            resolve_choose_target_draw_max_hp_cap_then_discard_excess(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp => {
            resolve_give_selected_then_target_choose_discard_same_suit_or_lose_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::FanjianChooseSuitOrHand => {
            resolve_fanjian_choose_suit_or_hand(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp => {
            resolve_choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGiveOneHandEquipOrRecover => {
            resolve_choose_give_one_hand_equip_or_recover(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGiveTwoDifferentColorsOrRecover => {
            resolve_choose_give_two_different_colors_or_recover(
                tx, observer, &payload, selections, cancelled,
            )
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

        ContinuationKind::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp => {
            resolve_choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOtherDrawOwnerLostHpTurnOver => {
            resolve_choose_other_draw_owner_lost_hp_turn_over(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ForceOthersUseShaOnNearestOrLoseHp => {
            resolve_force_others_use_sha_on_nearest_or_lose_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed => {
            resolve_choose_discard_one_for_sha_refund_and_draw_if_red(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ResolveOllinjie => {
            resolve_ollinjie(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTargetGainMaxHp => {
            resolve_choose_target_gain_max_hp(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::GainTargetHandCard => {
            resolve_gain_target_hand_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::GainFromSkillPile => {
            resolve_gain_from_skill_pile(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherAddMark => {
            resolve_choose_other_add_mark(tx, observer, &payload, selections, cancelled)
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

        ContinuationKind::ChooseGroup => {
            crate::effect::resolve_choose_group(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGainOtherSkill => crate::effect::resolve_choose_gain_other_skill(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::LoseGainedSkill => {
            crate::effect::resolve_lose_gained_skill(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolvePotheyun => {
            crate::effect::resolve_potheyun_pick(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChoosePlaceHandXumou => crate::effect::resolve_choose_place_hand_xumou(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::CityConsume => {
            crate::effect::resolve_city_consume(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOthersAddMark => crate::effect::resolve_choose_others_add_mark(
            tx, observer, &payload, selections, cancelled,
        ),

        ContinuationKind::ChooseOthersInsertExtraTurn => {
            crate::effect::resolve_choose_others_insert_extra_turn(
                tx, observer, &payload, selections, cancelled,
            )
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

        ContinuationKind::ChooseClanNotMaxHandLimitGain => {
            resolve_choose_clan_not_max_hand_limit_gain(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseAddCurrentShaTarget => {
            resolve_choose_add_current_sha_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseStoreHandShaInSkillPile => {
            resolve_choose_store_hand_sha_in_skill_pile(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseSpendSkillPileForDyingJiu => {
            resolve_choose_spend_skill_pile_for_dying_jiu(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseCompareTargetHandCard => {
            resolve_choose_compare_target_hand_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseHandCompareTarget => {
            resolve_choose_hand_compare_target(tx, observer, &payload, selections, cancelled)
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

        ContinuationKind::ChooseHandComparePlayerCard => {
            resolve_choose_hand_compare_player_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherDamage => {
            resolve_choose_other_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseBoundSourceAttackRangeDamage => {
            resolve_choose_bound_source_attack_range_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOtherDiscardTargetCard => {
            resolve_choose_other_discard_target_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGainDiscardCardByKind => {
            resolve_choose_gain_discard_card_by_kind(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGainOwnLostDiscardCardThisTurn => {
            resolve_choose_gain_own_lost_discard_card_this_turn(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseMoveFieldCardTo => {
            resolve_choose_move_field_card_to(tx, observer, &payload, selections, cancelled)
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

        ContinuationKind::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan => {
            resolve_choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver => {
            resolve_choose_zone_priority_gain_random_card_from_each_other_then_turn_over(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver => {
            resolve_choose_gain_card_from_each_other_then_turn_over(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseHiddenSkillPileHandCards => {
            resolve_choose_hidden_skill_pile_hand_cards(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseHiddenSkillPileCards => {
            resolve_choose_hidden_skill_pile_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseChangandajianTarget => {
            resolve_choose_changandajian_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseChangandajianCard => {
            resolve_choose_changandajian_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::DiscardTiaojiyanmeiCard => {
            resolve_discard_tiaojiyanmei_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseTiaojiyanmeiGainTarget => {
            resolve_choose_tiaojiyanmei_gain_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherYinghun => {
            resolve_choose_other_yinghun(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherBizheng => {
            resolve_choose_other_bizheng(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseOtherXiashu => {
            resolve_choose_other_xiashu(tx, observer, &payload, selections, cancelled)
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

        ContinuationKind::ChooseDiscardSkillPileCardAddMarkedSkill => {
            let [player, target, helper_skill_def, skill] = payload.as_slice() else {
                return Err(EffectError::EngineFault(
                    "bad skill-pile discard continuation".into(),
                ));
            };
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "skill-pile discard requires one card".into(),
                ));
            };
            if cancelled || observer != PlayerId::new(*player) {
                return Err(EffectError::Illegal(
                    "invalid skill-pile discard choice".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
                    player: PlayerId::new(*player),
                    target: PlayerId::new(*target),
                    helper_skill_def: engine_api::SkillDefId::new(*helper_skill_def),
                    selected: CardId::new(*selected),
                    skill: engine_api::SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }

        ContinuationKind::GainOneHandFromSelectedTargets => {
            resolve_gain_one_hand_from_selected_targets(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseRepoluTargetsDraw => {
            resolve_choose_repolu_targets_draw(tx, observer, &payload, selections, cancelled)
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

        ContinuationKind::ChooseMarkParityControlTarget => {
            resolve_choose_mark_parity_control_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::DiscardMarkParityControlTargetCard => {
            resolve_discard_mark_parity_control_target_card(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ConfirmMarkParityMassDamage => {
            resolve_confirm_mark_parity_mass_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseSelectedTargetDamage => {
            resolve_choose_selected_target_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDistinctSuitCards => {
            resolve_choose_distinct_suit_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseInspectedHeartCard => {
            resolve_choose_inspected_heart_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseInspectedHeartDestination => {
            resolve_choose_inspected_heart_destination(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseYeyanMode => {
            resolve_choose_yeyan_mode(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseYeyanBigCards => {
            resolve_choose_yeyan_big_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseYeyanTargets => {
            resolve_choose_yeyan_targets(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseYeyanSingleDamage => {
            resolve_choose_yeyan_single_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseYeyanDoubleTarget => {
            resolve_choose_yeyan_double_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseAllLoseHpOrRecover => {
            resolve_choose_all_lose_hp_or_recover(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardWeaponOrLoseHp => {
            resolve_choose_discard_weapon_or_lose_hp(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardOneHeOrLoseHp => {
            resolve_choose_discard_one_he_or_lose_hp(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseSourceConfirmOwnerLoseHp => {
            resolve_choose_source_confirm_owner_lose_hp(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGainTargetCardAlternateZoneThenTargetDraw => {
            resolve_choose_gain_target_card_alternate_zone_then_target_draw(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseQixingHandCards => {
            resolve_choose_qixing_hand_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseQixingPileCards => {
            resolve_choose_qixing_pile_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseQixingWeatherCards => {
            resolve_choose_qixing_weather_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseQixingWeatherTargets => {
            resolve_choose_qixing_weather_targets(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseLinkOther => {
            resolve_choose_link_other(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltPoxiCards => {
            resolve_choose_drlt_poxi_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltJieyingTransfer => {
            resolve_choose_drlt_jieying_transfer(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltDuoruiDisableSlot => {
            resolve_choose_drlt_duorui_disable_slot(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltDuoruiSkill => {
            resolve_choose_drlt_duorui_skill(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrltZhitiEnableSlot => {
            resolve_choose_drlt_zhiti_enable_slot(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseJilueJizhiDiscard => {
            resolve_choose_jilue_jizhi_discard(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseAddMissingSkill => {
            resolve_choose_add_missing_skill(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseAddSkillBoundCardTarget => {
            resolve_choose_add_skill_bound_card_target(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDiscardMatchingEquipColorAddSkill => {
            resolve_choose_discard_matching_equip_color_add_skill(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseCurrentCardUseEnhancements => {
            resolve_choose_current_card_use_enhancements(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseAddCurrentCardTarget => {
            resolve_choose_add_current_card_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget => {
            resolve_choose_add_or_remove_current_card_target(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDrawOrDiscardAddSkill => {
            resolve_choose_draw_or_discard_add_skill(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardedBlackBasicAsBingliang => {
            resolve_choose_discarded_black_basic_as_bingliang(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage => {
            resolve_choose_discard_judge_use_virtual_sha_draw_if_no_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded => {
            resolve_choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseRevealOtherHandCardRememberCategory => {
            resolve_choose_reveal_other_hand_card_remember_category(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::GiveExactRecordPhaseTarget => {
            resolve_give_exact_record_phase_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha => {
            resolve_choose_gain_resolved_sha_or_shan_then_optional_use_sha(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOtherTargetsBySkippedPhaseCountDamage => {
            resolve_choose_other_targets_by_skipped_phase_count_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOthersAfterUniformHandReveal => {
            resolve_choose_others_after_uniform_hand_reveal(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseCardUserOrAddTarget => {
            resolve_choose_card_user_or_add_target(tx, observer, &payload, selections, cancelled)
        }
        _ => unreachable!("choice routed to a different continuation group"),
    }
}
