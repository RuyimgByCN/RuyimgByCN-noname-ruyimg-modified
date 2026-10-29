//! Content registry operations for `load_package_inner` and related helpers.

use super::*;

impl ContentRegistry {
    pub(crate) fn load_package_inner(
        &mut self,
        pkg: &ContentPackage,
    ) -> Result<(), ContentLoadError> {
        if !self.validate_package_for_load(pkg)? {
            return Ok(());
        }
        self.enable_package_tracking(pkg);

        let mut local_modules = BTreeSet::new();

        for m in &pkg.modules {
            let verified = VerifiedModule::verify(m.clone())
                .map_err(|e| ContentLoadError::Verify(e.to_string()))?;
            if !local_modules.insert(m.name.clone()) {
                return Err(ContentLoadError::Conflict(format!(
                    "duplicate module {} in {}/{}",
                    m.name, pkg.manifest.namespace, pkg.manifest.package_id
                )));
            }
            let key =
                package_module_key(&pkg.manifest.namespace, &pkg.manifest.package_id, &m.name);
            if let Some(existing) = self.modules.get(&key) {
                if existing.logic_hash != m.logic_hash {
                    return Err(ContentLoadError::Conflict(key));
                }
            } else {
                self.modules.insert(key, verified);
            }
        }

        for skill in &pkg.skill_index {
            let metadata = SkillMetadata {
                skill_def: SkillDefId::new(skill.skill_def_id),
                name: skill.name.clone(),
                locked: skill.locked,
                lord_only: skill.lord_only,
                non_gainable: skill.non_gainable,
                conversion: skill.conversion,
            };
            if let Some(existing) = self.package_skills.get(&skill.skill_def_id) {
                if existing != &metadata {
                    return Err(ContentLoadError::Conflict(format!(
                        "skill definition {}",
                        skill.skill_def_id
                    )));
                }
            } else {
                self.package_skills.insert(skill.skill_def_id, metadata);
            }
            // Phase 3: reverse name→def index for AddSkillByName resolution.
            self.skill_def_by_name
                .entry(skill.name.clone())
                .or_insert(skill.skill_def_id);
        }

        for character in &pkg.character_index {
            if character.max_hp == 0 {
                return Err(ContentLoadError::Invalid(format!(
                    "character {} has zero max_hp",
                    character.id
                )));
            }
            if character.initial_hp > character.max_hp {
                return Err(ContentLoadError::Invalid(format!(
                    "character {} initial_hp exceeds max_hp",
                    character.id
                )));
            }
            if let Some(existing) = self.package_characters.get(&character.id) {
                if existing != character {
                    return Err(ContentLoadError::Conflict(format!(
                        "character definition {}",
                        character.id
                    )));
                }
            } else {
                self.package_characters
                    .insert(character.id, character.clone());
            }
        }
        for card in &pkg.card_index {
            if let Some(existing) = self.package_cards.get(&card.card_def_id) {
                if existing != card {
                    return Err(ContentLoadError::Conflict(format!(
                        "card definition {}",
                        card.card_def_id
                    )));
                }
            } else {
                self.package_cards.insert(card.card_def_id, card.clone());
            }
        }

        for t in &pkg.trigger_index {
            let event = map_event(&t.event)
                .ok_or_else(|| ContentLoadError::Invalid(format!("unknown event {}", t.event)))?;
            let filter = match map_filter(&t.filter) {
                Some(f) => f,
                None => {
                    eprintln!("DBG unknown filter: {:?}", t.filter);
                    return Err(ContentLoadError::Invalid(format!(
                        "unknown trigger filter {}",
                        t.filter
                    )));
                }
            };
            let tracks_turn_card_name = matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName
            );
            self.track_turn_card_history |= matches!(
                &filter,
                TriggerFilter::OwnerSourceAlternatesColor
                    | TriggerFilter::OwnerSourceRepeatsColorHasDiscardableHandEquip
                    | TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName
            );
            self.track_turn_card_name_history |= tracks_turn_card_name
                || matches!(
                    &filter,
                    TriggerFilter::OwnerSourcePlayCardRepeatsPreviousTargets
                        | TriggerFilter::OwnerSourceShaRepeatsPreviousShaTargets
                );
            self.track_play_card_history |=
                matches!(&filter, TriggerFilter::OwnerSourceMatchesPreviousSuitOrRank);
            self.track_per_player_turn_card_count |= matches!(
                &filter,
                TriggerFilter::OwnerAliveSourceFirstCardOrdinaryTrick
                    | TriggerFilter::OwnerAlivePhaseEndUsedCardNoDamage
            );
            self.track_per_player_play_phase_card_count |= matches!(
                &filter,
                TriggerFilter::OwnerSourcePlayCardCountEquals(_)
                    | TriggerFilter::OwnerSourcePlayCardCountEqualsHp
                    | TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
                    | TriggerFilter::OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty
            );
            self.track_play_phase_card_suit_history |= matches!(
                &filter,
                TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
            );
            self.track_discard_pile_suit_history |= matches!(
                &filter,
                TriggerFilter::OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap
                    | TriggerFilter::OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit
            );
            self.track_per_player_turn_card_action_count |= matches!(
                &filter,
                TriggerFilter::OwnerCardActionCountEqualsAttackRange
                    | TriggerFilter::OwnerAliveDddZhilianAvailable
            );
            self.track_lifetime_card_action_count |=
                matches!(&filter, TriggerFilter::OwnerLifetimeCardAction);
            self.track_play_card_category_duplicates |= matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherPlayPhaseDuplicateCardCategoryHasHandEquip
            );
            self.track_per_player_turn_sha_use_count |= matches!(
                &filter,
                TriggerFilter::OwnerAliveUsedShaThisTurn
                    | TriggerFilter::OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn
                    | TriggerFilter::OwnerSourceFirstShaTargetThisTurn
                    | TriggerFilter::OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable
            );
            self.track_turn_use_response_discard_count |= matches!(
                &filter,
                TriggerFilter::OwnerIsEventPlayerUseResponseDiscardedAtLeastHp
            );
            self.track_discard_phase_discard_count |= matches!(
                &filter,
                TriggerFilter::OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase
                    | TriggerFilter::OwnerAliveTurnEndedRehengjiangPending
            );
            self.track_discard_phase_lost_cards |= matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase
                    | TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase
            );
            self.track_turn_discarded_cards |= matches!(
                &filter,
                TriggerFilter::OwnerAliveGlobalPhaseEndDiscardedCardsAvailable
                    | TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase
            );
            self.track_turn_haoyi_cards |= matches!(
                &filter,
                TriggerFilter::OwnerPhaseEndDcsbhaoyiAvailable
                    | TriggerFilter::OwnerAliveDddZhilianAvailable
            );
            self.track_ddd_chengjing_history |=
                matches!(&filter, TriggerFilter::OwnerAliveDddChengjingAvailable);
            self.track_game_card_history |=
                matches!(&filter, TriggerFilter::OwnerSourceMatchesLastUsedSuitOrRank);
            self.track_turn_red_card_target_history |= matches!(
                &filter,
                TriggerFilter::OwnerAliveShaTargetNoOtherRedCardThisTurn
            );
            self.track_turn_card_target_counts |= matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherPhaseEndMobiledanshou
                    | TriggerFilter::OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn
            );
            self.track_phase_target_hp_history |= matches!(
                &filter,
                TriggerFilter::OwnerAlivePhaseTargetedHpChangedCanUseWanjian
                    | TriggerFilter::OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou
            );
            self.track_turn_hp_change_history |= matches!(
                &filter,
                TriggerFilter::OwnerIsEventPlayerFirstHpChangeTurnUniqueTrickMajority
                    | TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable
            );
            self.track_turn_haoyi_cards |= matches!(
                &filter,
                TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable
            );
            self.track_turn_card_gain_count |= matches!(
                &filter,
                TriggerFilter::OwnerAliveGainedThreeThisTurnOtherAlive
                    | TriggerFilter::OwnerAliveGainedTwoThisTurnCanCompareOther
                    | TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther
            );
            self.track_play_phase_card_gain_history |= matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherPhasePlayEndTwzhengjianCondition
                    | TriggerFilter::OwnerAlivePhasePlayEndZhengbiPending
            );
            self.track_card_target_history |= matches!(
                &filter,
                TriggerFilter::OwnerUsedAndWasTargetedThisTurnInRange
            );
            self.track_used_type_groups |=
                matches!(&filter, TriggerFilter::OwnerIsTargetSanshouTypeTracked);
            self.track_turn_card_loss_history |= matches!(
                &filter,
                TriggerFilter::OwnerPhaseEndLostCards
                    | TriggerFilter::OwnerIsEventPlayerLostCardsAtMostHpGainable
                    | TriggerFilter::OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive
                    | TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase
                    | TriggerFilter::OwnerAliveLostHandThisTurn
                    | TriggerFilter::OwnerAlivePhaseEndLostCardsAtLeastTwo
            );
            if !local_modules.contains(&t.module) {
                return Err(ContentLoadError::Invalid(format!(
                    "trigger references missing module {}",
                    t.module
                )));
            }
            let id = TriggerDefId(self.next_trigger_id);
            self.next_trigger_id = self.next_trigger_id.saturating_add(1);
            self.package_triggers.push(TriggerDefinition {
                id,
                skill_def: SkillDefId::new(t.skill_def_id),
                event,
                timing: TimingBand::Normal,
                priority: 0,
                optional: t.optional,
                choice_actor: t.choice_actor,
                filter,
                max_per_segment: 8,
                handler: TriggerHandler::RunVm {
                    module: package_module_key(
                        &pkg.manifest.namespace,
                        &pkg.manifest.package_id,
                        &t.module,
                    ),
                    entry_pc: t.entry_pc,
                },
            });
        }

        for m in &pkg.modifier_index {
            if m.target == "card_target_cancel_if_other_user_card_use_count_lt_owner_mark"
                && (m.mode != "override" || m.value != 1)
            {
                return Err(ContentLoadError::Invalid(
                    "平定目标限制只接受 override 1".into(),
                ));
            }
            if m.target == "card_number"
                && (m.selector.is_some()
                    || m.mode != "override"
                    || m.value_source.is_some()
                    || !(1..=13).contains(&m.value))
            {
                return Err(ContentLoadError::Invalid(
                    "card_number requires `override` with an integer from 1 to 13".into(),
                ));
            }
            let target = map_mod_target(&m.target, m.selector.as_deref()).ok_or_else(|| {
                ContentLoadError::Invalid(format!("unknown modifier target {}", m.target))
            })?;
            let mode = map_mod_mode(&m.mode).ok_or_else(|| {
                ContentLoadError::Invalid(format!("unknown modifier mode {}", m.mode))
            })?;
            let value_source = m
                .value_source
                .as_deref()
                .map(map_mod_value_source)
                .transpose()
                .map_err(ContentLoadError::Invalid)?;
            self.track_per_player_turn_card_count |= matches!(
                value_source,
                Some(ModifierValueSource::OwnerCurrentTurnCardUseCount)
            );
            self.track_per_player_play_phase_card_count |= matches!(
                value_source,
                Some(ModifierValueSource::OwnerPlayPhaseCardUseCountZero)
            );
            self.track_turn_card_gain_count |=
                matches!(target, ModifierTarget::HandCardGainedThisTurnOnly);
            let id = self.next_modifier_id;
            self.next_modifier_id = self.next_modifier_id.saturating_add(1);
            self.package_modifiers.push(ModifierDefinition {
                id,
                skill_def: SkillDefId::new(m.skill_def_id),
                target,
                mode,
                value_source,
                value: m.value,
                order: 0,
                label_key: match &m.selector {
                    Some(selector) => {
                        format!("pkg.{}.{}.{}", m.target, selector, m.skill_def_id)
                    }
                    None => format!("pkg.{}.{}", m.target, m.skill_def_id),
                },
            });
        }

        for a in &pkg.active_index {
            if !local_modules.contains(&a.module) {
                return Err(ContentLoadError::Invalid(format!(
                    "active skill references missing module {}",
                    a.module
                )));
            }
            if a.targets
                .as_ref()
                .is_some_and(|targets| targets.max < targets.min)
            {
                return Err(ContentLoadError::Invalid(
                    "active target contract requires min <= max".into(),
                ));
            }
            if a.selection
                .as_ref()
                .is_some_and(|selection| !matches!(selection.zone.as_str(), "hand" | "hand_equip"))
            {
                return Err(ContentLoadError::Invalid(
                    "unsupported active selection zone".into(),
                ));
            }
            if a.selection.as_ref().is_some_and(|selection| {
                selection.filter.as_deref().is_some_and(|filter| {
                    !matches!(
                        filter,
                        "card_equip"
                            | "card_armor"
                            | "card_sha"
                            | "card_basic"
                            | "card_nonbasic"
                            | "card_non_equip"
                            | "card_red"
                            | "card_black"
                            | "card_heart"
                    )
                })
            }) {
                return Err(ContentLoadError::Invalid(
                    "unsupported active selection filter".into(),
                ));
            }
            if a.uses_per_turn == Some(0) {
                return Err(ContentLoadError::Invalid(
                    "active uses_per_turn must be positive".into(),
                ));
            }
            if a.uses_per_play_phase == Some(0) {
                return Err(ContentLoadError::Invalid(
                    "active uses_per_play_phase must be positive".into(),
                ));
            }
            if a.uses_per_play_phase.is_some() && a.window != "play_phase" {
                return Err(ContentLoadError::Invalid(
                    "active uses_per_play_phase requires the play_phase window".into(),
                ));
            }
            if a.uses_per_turn_source.as_deref().is_some_and(|source| {
                !matches!(
                    source,
                    "owner_hp" | "owner_lost_hp_min_one" | "living_rebel_count"
                )
            }) {
                return Err(ContentLoadError::Invalid(
                    "unsupported active uses_per_turn source".into(),
                ));
            }
            if (a.uses_per_turn.is_some() && a.uses_per_turn_source.is_some())
                || (a.uses_per_play_phase.is_some()
                    && (a.uses_per_turn.is_some() || a.uses_per_turn_source.is_some()))
            {
                return Err(ContentLoadError::Invalid(
                    "active use limits cannot combine turn and Play-phase scopes".into(),
                ));
            }
            if a.selection.as_ref().is_some_and(|selection| {
                selection.sum_effective_number_exact.is_some() && selection.zone != "hand"
            }) {
                return Err(ContentLoadError::Invalid(
                    "sum_effective_number_exact requires a hand selection".into(),
                ));
            }
            if a.selection
                .as_ref()
                .is_some_and(|selection| selection.sum_effective_number_exact == Some(0))
            {
                return Err(ContentLoadError::Invalid(
                    "sum_effective_number_exact must be positive".into(),
                ));
            }
            if a.mark_at_least
                .is_some_and(|(mark, count)| mark == 0 || count == 0)
            {
                return Err(ContentLoadError::Invalid(
                    "active mark_at_least requires non-zero mark and count".into(),
                ));
            }
            if a.owner_missing_skill == Some(0) {
                return Err(ContentLoadError::Invalid(
                    "active owner_missing_skill requires a non-zero skill definition".into(),
                ));
            }
            if a.targets.as_ref().is_some_and(|targets| {
                (targets.linked && targets.unlinked) || targets.max_by_owner_mark == Some(0)
            }) {
                return Err(ContentLoadError::Invalid(
                    "active target linked/max_by_owner_mark constraint is invalid".into(),
                ));
            }
            self.package_actives.push(ActiveSkillDef {
                skill_def: SkillDefId::new(a.skill_def_id),
                window: a.window.clone(),
                module: package_module_key(
                    &pkg.manifest.namespace,
                    &pkg.manifest.package_id,
                    &a.module,
                ),
                entry_pc: a.entry_pc,
                targets: a.targets.clone(),
                uses_per_turn: a.uses_per_turn,
                uses_per_turn_source: a.uses_per_turn_source.clone(),
                uses_per_play_phase: a.uses_per_play_phase,
                mark_at_least: a.mark_at_least,
                owner_missing_skill: a.owner_missing_skill,
                hand_gt_hp: a.hand_gt_hp,
                hand_gt_max_hp: a.hand_gt_max_hp,
                hand_limit_positive: a.hand_limit_positive,
                equip_nonempty: a.equip_nonempty,
                hand_nonempty: a.hand_nonempty,
                hand_missing_gaintag: a.hand_missing_gaintag,
                face_up: a.face_up,
                owner_unlinked: a.owner_unlinked,
                once_per_game: a.once_per_game,
                selection: a.selection.clone(),
            });
        }

        for v in &pkg.view_as_index {
            let source_free = v.source_zone.as_deref() == Some("none");
            let dynamic_family = matches!(v.as_kind.as_str(), "basic" | "trick" | "ordinary_trick")
                || source_free && matches!(v.as_kind.as_str(), "tao" | "jiu");
            if source_free != dynamic_family {
                return Err(ContentLoadError::Invalid(
                    "source-free view_as family contract mismatch".into(),
                ));
            }
            if source_free
                && (v.filter != "any"
                    || v.min_cards != 1
                    || v.max_cards != 1
                    || v.owner_hp_min_one_cards
                    || v.owner_basic_uses_plus_one_cards
                    || v.owner_basic_uses_plus_one_cards
                    || match v.cost.as_deref() {
                        Some(
                            "guhuo"
                            | "face_down_hand"
                            | "dczhaowen"
                            | "draw_one_outside_turn_empty_hand",
                        ) => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.window != "any"
                        }
                        Some("turn_over") => {
                            v.as_kind != "jiu"
                                || v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn.is_some()
                                || v.window != "any"
                        }
                        Some("swap_lower_hand") => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                        }
                        Some("dddshilie") => {
                            v.as_kind != "basic"
                                || v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                                || v.window != "any"
                        }
                        Some("dddfusi") => {
                            !matches!(v.as_kind.as_str(), "basic" | "trick")
                                || v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn.is_some()
                                || v.window != "any"
                        }
                        Some("all_hand") => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                                || v.window != "play_phase"
                        }
                        Some("hand_equip_mark") => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                                || v.window != "any"
                                || v.mark_cost.is_none()
                        }
                        Some("lose_max_hp_1") => {
                            !matches!(v.as_kind.as_str(), "tao" | "jiu")
                                || v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                                || v.window != "any"
                        }
                        Some("potdaozhuan") => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn.is_some()
                                || v.window != "any"
                        }
                        Some("gain_max_hp_1" | "disable_equip_slot") => {
                            v.branch.is_none() || !v.once_per_round || v.next_branch.is_none()
                        }
                        _ => true,
                    })
            {
                return Err(ContentLoadError::Invalid(
                    "incomplete source-free view_as contract".into(),
                ));
            }
            if (v.mark_cost.is_some() && v.cost.as_deref() != Some("hand_equip_mark"))
                || (v.post_draw != 0
                    && !matches!(v.cost.as_deref(), Some("hand_equip_mark" | "dddzhijie")))
            {
                return Err(ContentLoadError::Invalid(
                    "view_as mark/post-draw contract mismatch".into(),
                ));
            }
            let valid_bonus = match v.multi_card_bonus.as_deref() {
                None => true,
                Some("damage_1") => v.as_kind == "sha" && v.max_cards > 1,
                Some("recover_1") => v.as_kind == "tao" && v.max_cards > 1,
                Some("discard_current_hand_equip") => {
                    matches!(v.as_kind.as_str(), "shan" | "wuxie") && v.max_cards > 1
                }
                Some(_) => false,
            };
            let potqingyan = v.cost.as_deref() == Some("potqingyan");
            let dddzhijie = v.cost.as_deref() == Some("dddzhijie");
            let dddtongyu = v.cost.as_deref() == Some("dddtongyu");
            if v.min_cards == 0
                || v.min_cards > v.max_cards
                || if potqingyan {
                    v.max_cards != 5
                } else if dddtongyu {
                    v.max_cards != 4
                } else {
                    v.max_cards > 2 && v.max_cards != u8::MAX
                }
                || (v.refund_use_count_if_no_damage && v.as_kind != "sha")
                || (v.same_suit && v.max_cards == 1 && !v.owner_hp_min_one_cards)
                || v.nature.as_deref().is_some_and(|nature| {
                    !matches!(nature, "fire" | "thunder") || v.as_kind != "sha"
                })
                || !valid_bonus
                || potqingyan
                    && (!matches!(v.as_kind.as_str(), "shan" | "wuxie")
                        || v.window != "response"
                        || v.filter != "any"
                        || v.source_zone.as_deref().is_some_and(|zone| zone != "hand")
                        || v.min_cards != 1
                        || v.uses_per_turn.is_some()
                        || v.owner_hp_min_one_cards
                        || v.owner_basic_uses_plus_one_cards
                        || v.same_suit
                        || v.nature.is_some()
                        || v.multi_card_bonus.is_some()
                        || v.branch.is_some()
                        || v.once_per_round
                        || v.next_branch.is_some())
                || dddzhijie
                    && (!matches!(v.as_kind.as_str(), "shan" | "wuxie")
                        || v.window != "response"
                        || v.filter != "any"
                        || v.source_zone.as_deref() != Some("hand_expansion")
                        || v.min_cards != 2
                        || v.max_cards != 2
                        || v.post_draw != 2
                        || v.uses_per_turn.is_some()
                        || v.owner_hp_min_one_cards
                        || v.owner_basic_uses_plus_one_cards
                        || v.same_suit
                        || v.nature.is_some()
                        || v.multi_card_bonus.is_some()
                        || v.branch.is_some()
                        || v.once_per_round
                        || v.next_branch.is_some())
                || dddtongyu
                    && (v.as_kind != "wugu"
                        || v.window != "play_phase"
                        || v.filter != "any"
                        || v.source_zone.as_deref().is_some_and(|zone| zone != "hand")
                        || v.min_cards != 1
                        || v.max_cards != 4
                        || v.uses_per_turn != Some(1)
                        || v.owner_hp_min_one_cards
                        || v.owner_basic_uses_plus_one_cards
                        || v.same_suit
                        || v.nature.is_some()
                        || v.multi_card_bonus.is_some()
                        || v.branch.is_some()
                        || v.once_per_round
                        || v.next_branch.is_some())
            {
                return Err(ContentLoadError::Invalid(
                    "invalid multi-card view_as contract".into(),
                ));
            }
            self.track_basic_use_history |= v.owner_basic_uses_plus_one_cards;
            self.package_view_as.push(ViewAsDef {
                skill_def: SkillDefId::new(v.skill_def_id),
                as_kind: v.as_kind.clone(),
                window: v.window.clone(),
                filter: v.filter.clone(),
                source_zone: v.source_zone.clone(),
                uses_per_turn: v.uses_per_turn,
                ignore_distance: v.ignore_distance,
                red_unrespondable: v.red_unrespondable,
                black_no_use_count: v.black_no_use_count,
                no_use_count: v.no_use_count,
                target_lacks_source_suit_unrespondable: v.target_lacks_source_suit_unrespondable,
                refund_use_count_if_no_damage: v.refund_use_count_if_no_damage,
                min_cards: v.min_cards,
                max_cards: v.max_cards,
                owner_hp_min_one_cards: v.owner_hp_min_one_cards,
                owner_basic_uses_plus_one_cards: v.owner_basic_uses_plus_one_cards,
                once_per_turn_per_branch: v.once_per_turn_per_branch,
                same_suit: v.same_suit,
                nature: v.nature.clone(),
                multi_card_bonus: v.multi_card_bonus.clone(),
                branch: v.branch,
                once_per_round: v.once_per_round,
                next_branch: v.next_branch,
                cost: v.cost.clone(),
                mark_cost: v.mark_cost,
                post_draw: v.post_draw,
            });
        }

        self.packages.push(LoadedPackageMeta {
            namespace: pkg.manifest.namespace.clone(),
            package_id: pkg.manifest.package_id.clone(),
            version: pkg.manifest.version.clone(),
            logic_hash: pkg.manifest.logic_hash,
            rule_vm_abi_version: Some(pkg.manifest.rule_vm_abi_version),
        });
        self.packages.sort_by(|a, b| {
            (&a.namespace, &a.package_id, &a.version).cmp(&(
                &b.namespace,
                &b.package_id,
                &b.version,
            ))
        });
        self.package_triggers
            .sort_by_key(|definition| definition.id);
        self.package_modifiers
            .sort_by_key(|definition| definition.id);
        self.package_actives.sort_by(|a, b| {
            (a.skill_def.raw(), &a.window, &a.module).cmp(&(
                b.skill_def.raw(),
                &b.window,
                &b.module,
            ))
        });
        self.package_view_as.sort_by(|a, b| {
            (
                a.skill_def.raw(),
                &a.window,
                &a.as_kind,
                &a.filter,
                &a.source_zone,
                a.uses_per_turn,
                a.branch,
            )
                .cmp(&(
                    b.skill_def.raw(),
                    &b.window,
                    &b.as_kind,
                    &b.filter,
                    &b.source_zone,
                    b.uses_per_turn,
                    b.branch,
                ))
        });
        self.recompute_combined_hash();
        Ok(())
    }

    /// Validate character skill references after a dependency set has fully loaded.
    pub fn validate_character_skill_closure(&self) -> Result<(), ContentLoadError> {
        validate_character_skills(
            self.package_characters.values(),
            &self.package_skills.keys().copied().collect(),
        )
    }

    pub(crate) fn recompute_combined_hash(&mut self) {
        let mut h = blake3::Hasher::new();
        h.update(b"noname-next/content-registry/v2");
        for p in &self.packages {
            h.update(&p.logic_hash);
            h.update(p.namespace.as_bytes());
            h.update(p.package_id.as_bytes());
            h.update(p.version.as_bytes());
        }
        for (name, m) in &self.modules {
            h.update(name.as_bytes());
            h.update(&m.logic_hash);
        }
        for metadata in self.package_skills.values() {
            h.update(&metadata.skill_def.raw().to_le_bytes());
            h.update(metadata.name.as_bytes());
            h.update(&[u8::from(metadata.locked)]);
            if metadata.lord_only {
                h.update(b"lord_only/v1");
            }
            if metadata.non_gainable {
                h.update(b"non_gainable/v1");
            }
            if metadata.conversion {
                h.update(b"conversion/v1");
            }
        }
        for character in self.package_characters.values() {
            h.update(&character.id.to_le_bytes());
            h.update(character.name_key.as_bytes());
            h.update(&character.max_hp.to_le_bytes());
            h.update(&character.initial_hp.to_le_bytes());
            for skill in &character.skills {
                h.update(&skill.to_le_bytes());
            }
        }
        for m in &self.package_modifiers {
            h.update(&m.id.to_le_bytes());
            h.update(&m.skill_def.raw().to_le_bytes());
            h.update(mod_target_tag(m.target).as_bytes());
            if let ModifierTarget::CardIgnoredHandLimit(selector)
            | ModifierTarget::CardDistanceUnlimitedFor(selector)
            | ModifierTarget::CardDistanceUnlimitedIfTargetHandGteOwnerFor(selector)
            | ModifierTarget::CardDistanceLimitFor(selector)
            | ModifierTarget::CardTargetsAllFor(selector)
            | ModifierTarget::CardTargetCancelFor(selector)
            | ModifierTarget::CardTargetCancelFromCurrentFor(selector)
            | ModifierTarget::CardTargetCancelTurnBannedFor(selector)
            | ModifierTarget::ArmorCardTargetCancelFor(selector)
            | ModifierTarget::CardTargetCancelIfOtherUserHandGteTargetAndArmorEmptyFor(
                selector,
            )
            | ModifierTarget::CardUseCancelFor(selector)
            | ModifierTarget::CardResponseCancelFor(selector)
            | ModifierTarget::CardUseUnlimitedFor(selector)
            | ModifierTarget::CardTargetLimitFor(selector)
            | ModifierTarget::CardSaveCancelFor(selector)
            | ModifierTarget::CardName(selector)
            | ModifierTarget::CardNatureFor(selector)
            | ModifierTarget::CardGainProtectedFor(selector)
            | ModifierTarget::CardUseUnrespondableFor(selector) = m.target
            {
                h.update(selector.tag().as_bytes());
            }
            if let ModifierTarget::CardSuit(suit) = m.target {
                h.update(mod_suit_tag(suit).as_bytes());
            }
            h.update(mod_mode_tag(m.mode).as_bytes());
            if let Some(source) = m.value_source {
                h.update(b"value-source/v1");
                h.update(mod_value_source_tag(source).as_bytes());
            }
            h.update(&m.value.to_le_bytes());
        }
        self.combined_logic_hash = *h.finalize().as_bytes();
    }
}
