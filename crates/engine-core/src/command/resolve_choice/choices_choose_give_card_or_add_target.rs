//! Choice continuations for `ChooseGiveCardOrAddTarget` and related requests.

use super::*;

pub(super) fn resolve_choice(
    tx: &mut Transaction,
    _actor: PlayerId,
    observer: PlayerId,
    kind: ContinuationKind,
    payload: Vec<u32>,
    selections: &[u32],
    cancelled: bool,
    vm_cont: (Vec<u8>, String, u32, u32, [u32; 4], u32, [u32; 3]),
    optional_trigger: Option<crate::trigger::PendingTrigger>,
) -> Result<(), EffectError> {
    match kind {
        ContinuationKind::RevealHandChooseDiscardColor => {
            resolve_reveal_hand_choose_discard_color(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::LoseHpGainDiscardCardForOpeningExtraPlay => {
            resolve_opening_extra_play_hp_loss_gain(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::SourceFreeViewAsEquipCost => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose exactly one enabled equipment slot".into(),
                ));
            }
            let skill = engine_api::SkillId::new(*payload.first().unwrap_or(&0));
            let kind = payload
                .get(1)
                .and_then(|tag| CardKind::from_tag(*tag as u8))
                .ok_or_else(|| EffectError::Illegal("bad response card kind".into()))?;
            let action = *payload.get(2).unwrap_or(&0);
            super::super::skill::complete_source_free_response_cost(
                tx,
                observer,
                skill,
                kind,
                selections[0] as u8,
                action,
                payload.get(3..).unwrap_or_default(),
            )
        }

        ContinuationKind::SourceFreeViewAsPlay | ContinuationKind::SourceFreeViewAsResponse => {
            Err(EffectError::EngineFault(
                "saved source-free continuation opened as a direct choice".into(),
            ))
        }

        ContinuationKind::MultiCardViewAsResponse => {
            resolve_multi_card_view_as_response(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::PlaceHandCardsToDeckTop => {
            resolve_place_hand_cards_to_deck_top(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveDcbingjiStage => {
            resolve_resolve_dcbingji(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolvePxqixianStage => {
            resolve_resolve_pxqixian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveJsrgdaorenStage => {
            resolve_resolve_jsrgdaoren(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveQianlongStage => {
            resolve_resolve_qianlong(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveJsrgzuozhanStage => {
            resolve_resolve_jsrgzuozhan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveJsrgzuozhanGainStage => {
            resolve_resolve_jsrgzuozhan_gain(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwdaiguiStage if !payload.is_empty() && payload[3] == 3 => {
            resolve_resolve_twdaigui_round(tx, observer, &payload, selections)
        }

        ContinuationKind::ResolveTwdaiguiStage => {
            resolve_resolve_twdaigui(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveTwmituStage if !payload.is_empty() && payload[3] >= 3 => {
            resolve_resolve_twmitu_round(tx, observer, &payload, selections)
        }

        ContinuationKind::ResolveTwmituStage => {
            resolve_resolve_twmitu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveQiantunStage => {
            resolve_resolve_qiantun(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveYangmingStage if !payload.is_empty() && payload[3] >= 2 => {
            resolve_resolve_yangming_round(tx, observer, &payload, selections)
        }

        ContinuationKind::ResolveYangmingStage => {
            resolve_resolve_yangming(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveCanyunStage => {
            resolve_resolve_canyun(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveBiaozhao3Stage => {
            resolve_resolve_biaozhao3(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveDcmoukuiStage => {
            resolve_resolve_dcmoukui(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveZhongyanStage => {
            resolve_resolve_zhongyan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveChumeiStage => {
            resolve_resolve_chumei(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveShuaiyanStage => {
            resolve_resolve_shuaiyan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveMbbeimingStage => {
            resolve_resolve_mbbeiming(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveGaoyuanStage => {
            resolve_resolve_gaoyuan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveDuoduanStage => {
            resolve_resolve_duoduan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveDcanxuStage => {
            resolve_resolve_dcanxu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveChulaoStage => {
            resolve_resolve_chulao(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveMbzhenannStage => {
            resolve_resolve_mbzhenann(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveMeiyongStage => {
            resolve_resolve_meiyong(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveDcqiaolveStage => {
            resolve_resolve_dcqiaolve(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveXinfuQiaiStage => {
            resolve_resolve_xinfu_qiai(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOlzhijieStage => {
            resolve_resolve_olzhijie(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveOlqiaozhiStage => {
            resolve_resolve_olqiaozhi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveZhongjieStage => {
            resolve_resolve_zhongjie(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveNhlundaoStage => {
            resolve_resolve_nhlundao(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveDingceStage => {
            resolve_resolve_dingce(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveQiangyongStage => {
            resolve_resolve_qiangyong(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveZhengjianStage => {
            resolve_resolve_zhengjian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ResolveReliangyinStage => {
            resolve_resolve_liangyin(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::StoreSelectedInSkillPile => {
            resolve_store_selected_in_skill_pile(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::RearrangePileToDeckTop => {
            resolve_rearrange_pile_to_deck_top(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::VmResume => {
            let (frames, module_name, pc, fuel, ctx, skill_id, trigger_marker) = vm_cont;
            if cancelled {
                // Cancelling a DSL choose just ends the skill with no further effects.
                if trigger_marker != [0; 3] {
                    tx.state_mut()
                        .resolution
                        .deferred_effects
                        .push(vm_trigger_marker(trigger_marker));
                }
                return Ok(());
            }
            // The chosen token is the single selection (choose is single-select).
            let chosen = *selections.first().unwrap_or(&0);
            let module = tx
                .state()
                .content
                .module(&module_name)
                .cloned()
                .ok_or_else(|| {
                    EffectError::ContentFault(format!("missing module {module_name}"))
                })?;
            let outcome = crate::vm_host::resume_module(
                tx.state(),
                &module,
                skill_id,
                ctx,
                frames,
                pc,
                fuel,
                chosen,
            )
            .map_err(|e| EffectError::ContentFault(format!("vm resume: {e}")))?;
            if trigger_marker == [0; 3] {
                run_effects(tx, outcome.effects)?;
            } else {
                tx.state_mut()
                    .resolution
                    .deferred_effects
                    .extend(outcome.effects);
            }
            if let Some(pause) = outcome.paused {
                open_vm_choice(
                    tx,
                    PlayerId::new(ctx[0]),
                    &module_name,
                    skill_id,
                    ctx,
                    trigger_marker,
                    pause,
                )?;
                return Err(EffectError::ChoiceRequested);
            }
            if trigger_marker != [0; 3] {
                tx.state_mut()
                    .resolution
                    .deferred_effects
                    .push(vm_trigger_marker(trigger_marker));
            }
            Ok(())
        }

        ContinuationKind::DiscardOneHandOrEquip => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "must discard exactly one hand or equipment card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "discard card not in player hand or equipment".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                }],
            )
            .map(|_| ())
        }

        ContinuationKind::AdjustHandTo => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let need = *payload.get(1).unwrap_or(&0) as usize;
            if cancelled || selections.len() != need {
                return Err(EffectError::Illegal(format!(
                    "must discard exactly {need} hand cards"
                )));
            }
            let legal: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .map(CardId::raw)
                .collect();
            let selected: std::collections::BTreeSet<u32> = selections.iter().copied().collect();
            if selected.len() != need || !selected.is_subset(&legal) {
                return Err(EffectError::Illegal(
                    "adjust-hand selection must contain distinct current hand cards".into(),
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

        ContinuationKind::DiscardHandAddSkillByColor => {
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 || payload.len() != 3 {
                return Err(EffectError::Illegal(
                    "discard exactly one hand card to gain a color skill".into(),
                ));
            }
            let player = PlayerId::new(payload[0]);
            let card = CardId::new(selections[0]);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
            let card_state = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("bad discard card".into()))?;
            if card_state.zone != hand
                || !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
            {
                return Err(EffectError::Illegal(
                    "discard card not in player hand".into(),
                ));
            }
            let skill_def = match crate::query::effective_card_suit(tx.state(), player, card_state)
            {
                engine_api::Suit::Heart | engine_api::Suit::Diamond => payload[1],
                engine_api::Suit::Spade | engine_api::Suit::Club => payload[2],
                engine_api::Suit::None => {
                    return Err(EffectError::Illegal(
                        "suitless card cannot select a color skill".into(),
                    ));
                }
            };
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    },
                    EffectRequest::AddSkill {
                        player,
                        skill_def: engine_api::SkillDefId::new(skill_def),
                    },
                ],
            )
            .map(|_| ())
        }

        ContinuationKind::RevealTopGainNonBasicOrSwapBasic => {
            if payload.len() != 2 {
                return Err(EffectError::EngineFault(
                    "bad top-card swap continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[0]);
            let revealed = CardId::new(payload[1]);
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
            if !tx
                .state()
                .cards
                .get(revealed)
                .is_some_and(|card| card.zone == processing && card.owner.is_none())
            {
                return Err(EffectError::Illegal(
                    "revealed card left processing zone".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled top-card swap has selections".into(),
                    ));
                }
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
                return run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: vec![revealed],
                        to: discard,
                        from: Some(processing),
                    }],
                )
                .map(|_| ());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "select one hand card for top-card swap".into(),
                ));
            }
            let replacement = CardId::new(selections[0]);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
            if !tx
                .state()
                .cards
                .get(replacement)
                .is_some_and(|card| card.zone == hand && card.owner == player)
            {
                return Err(EffectError::Illegal(
                    "replacement card not in player hand".into(),
                ));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![replacement],
                        to: draw,
                        from: Some(hand),
                    },
                    EffectRequest::MoveCards {
                        cards: vec![revealed],
                        to: hand,
                        from: Some(processing),
                    },
                ],
            )
            .map(|_| ())
        }

        ContinuationKind::DrawThenChooseGiveOrDiscardHand => {
            resolve_draw_then_choose_give_or_discard_hand(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::GiveOneHandOrEquip => {
            if cancelled || selections.len() != 1 || payload.len() < 6 {
                return Err(EffectError::Illegal(
                    "must give exactly one hand or equipment card".into(),
                ));
            }
            let from_player = PlayerId::new(payload[0]);
            let to_player = PlayerId::new(payload[1]);
            let use_given_equipment = payload[3] != 0;
            let skill = engine_api::SkillId::new(payload[4]);
            let option_count = payload[5] as usize;
            if payload.len() != 6 + option_count {
                return Err(EffectError::EngineFault(
                    "truncated give-card continuation".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !payload[6..].contains(&card.raw()) {
                return Err(EffectError::Illegal(
                    "card was not offered for giving".into(),
                ));
            }
            let from = response_card_zone(tx, from_player, card)?;
            if !tx
                .state()
                .players
                .get(to_player)
                .is_some_and(|player| player.alive)
            {
                return Ok(());
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, to_player)
                .ok_or_else(|| EffectError::Illegal("no recipient hand".into()))?;
            if use_given_equipment {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::DrawThenGiveOne {
                        player: from_player,
                        target: to_player,
                        count: 0,
                        drawn_only: false,
                        use_given_equipment: true,
                        skill,
                        drawn: vec![card],
                        ready_to_give: true,
                    },
                );
            }
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to,
                    from,
                }],
            )
            .map(|_| ())
        }

        ContinuationKind::UseGivenEquipment => {
            let [recipient_raw, card_raw] = payload.as_slice() else {
                return Err(EffectError::EngineFault(
                    "bad use-given-equipment continuation".into(),
                ));
            };
            if observer != PlayerId::new(*recipient_raw) {
                return Err(EffectError::Illegal(
                    "given-equipment observer mismatch".into(),
                ));
            }
            if cancelled || selections.first().copied() == Some(0) {
                return if selections.is_empty() || selections == [0] {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "invalid given-equipment cancellation".into(),
                    ))
                };
            }
            if selections != [1] {
                return Err(EffectError::Illegal(
                    "invalid given-equipment confirmation".into(),
                ));
            }
            super::super::play::use_equipment_from_hand(tx, observer, CardId::new(*card_raw))
        }

        ContinuationKind::RevealDiscardSuitGainOrUse => {
            crate::command::reveal_options::resolve_choice(tx, &payload, selections, cancelled)
        }

        ContinuationKind::DiscardOrFireDamageSkipPhases => {
            crate::command::discard_damage_options::resolve_choice(
                tx, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ResumePlayedSha
        | ContinuationKind::ResumePlayedShaQueue
        | ContinuationKind::IndependentTrickQueue
        | ContinuationKind::FinishPlayedSha
        | ContinuationKind::ContinueEnteredDrawPhase
        | ContinuationKind::FinishDelayedJudgment => Err(EffectError::EngineFault(
            "saved continuation cannot own a choice".into(),
        )),

        ContinuationKind::OptionalTrigger => {
            let task = optional_trigger.ok_or_else(|| {
                EffectError::EngineFault("missing optional trigger continuation".into())
            })?;
            if !cancelled && selections.first().copied() == Some(1) {
                tx.state_mut().resolution.deferred_triggers.insert(0, task);
            }
            Ok(())
        }
        _ => unreachable!("choice routed to a different continuation group"),
    }
}
