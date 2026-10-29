//! Canonical encoding for `DrawCards` and related requests.

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ModifyCurrentDamage {
            player,
            delta,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&999u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *delta as u32);
            put_u32(h, skill.raw());
        }

        EffectRequest::ModifyCurrentDamage {
            player,
            delta,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&562u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, delta.unsigned_abs());
            put_u8(h, u8::from(*delta < 0));
            put_u32(h, skill.raw());
        }

        EffectRequest::MoveCards { cards, to, from } => {
            put_u8(h, 2);
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, to.raw());
            put_u32(h, from.map(|id| id.raw()).unwrap_or(0));
        }

        EffectRequest::MoveCardsWithReason {
            cards,
            to,
            from,
            reason,
        } => {
            put_u8(h, 0);
            h.update(&385u16.to_le_bytes());
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, to.raw());
            put_u32(h, from.map(|id| id.raw()).unwrap_or(0));
            put_u8(h, card_move_reason_tag(*reason));
        }

        EffectRequest::UseCard {
            player,
            card,
            targets,
        } => {
            put_u8(h, 3);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|id| id.raw()));
        }

        EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
        } => {
            put_u8(h, 4);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_bool(h, *is_lose_hp);
            put_u8(h, *nature);
        }

        EffectRequest::RecoverHp {
            source,
            player,
            amount,
        } => {
            put_u8(h, 5);
            put_u32(h, source.raw());
            put_u32(h, player.raw());
            put_u32(h, *amount);
        }

        EffectRequest::ChangeMaxHp { player, delta } => {
            put_u8(h, 6);
            put_u32(h, player.raw());
            put_i32(h, *delta);
        }

        EffectRequest::AddSkill { player, skill_def } => {
            put_u8(h, 7);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
        }

        EffectRequest::RemoveSkill { skill } => {
            put_u8(h, 8);
            put_u32(h, skill.raw());
        }

        EffectRequest::AddMark {
            player,
            mark,
            delta,
        } => {
            put_u8(h, 9);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_i32(h, *delta);
        }

        EffectRequest::RemoveMark {
            player,
            mark,
            amount,
        } => {
            put_u8(h, 10);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *amount);
        }

        EffectRequest::RevealCards { cards, to } => {
            put_u8(h, 11);
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_ids(h, to.iter().map(|id| id.raw()));
        }

        EffectRequest::RequestChoice {
            observer,
            min_count,
            max_count,
            allow_cancel,
            option_tokens,
        } => {
            put_u8(h, 12);
            put_u32(h, observer.raw());
            put_u8(h, *min_count);
            put_u8(h, *max_count);
            put_bool(h, *allow_cancel);
            put_ids(h, option_tokens.iter().copied());
        }

        EffectRequest::EnterDying { player } => {
            put_u8(h, 13);
            put_u32(h, player.raw());
        }

        EffectRequest::KillPlayer { player, killer } => {
            put_u8(h, 14);
            put_u32(h, player.raw());
            put_u32(h, killer.raw());
        }

        EffectRequest::SetTurnState { current, phase } => {
            put_u8(h, 15);
            put_u32(h, current.raw());
            put_u8(h, *phase);
        }

        EffectRequest::EmitDebateResolved { .. } => {
            // Engine-internal event emission; no content hash contribution.
        }

        EffectRequest::EmitRuleEvent { code, payload } => {
            put_u8(h, 16);
            put_u32(h, *code);
            put_ids(h, payload.iter().copied());
        }

        EffectRequest::DiscardFromHand { player, count } => {
            put_u8(h, 17);
            put_u32(h, player.raw());
            put_u32(h, *count);
        }

        EffectRequest::DiscardFromHandEquip { player, count } => {
            put_u8(h, 0);
            h.update(&359u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *count);
        }

        EffectRequest::TakeFromHand { from, to } => {
            put_u8(h, 18);
            put_u32(h, from.raw());
            put_u32(h, to.raw());
        }

        EffectRequest::DiscardCards { player, cards } => {
            put_u8(h, 19);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
        }

        EffectRequest::CardResponded {
            player,
            respond_to,
            card,
            response_kind,
        } => {
            put_u8(h, 20);
            put_u32(h, player.raw());
            put_u32(h, respond_to.raw());
            put_u32(h, card.raw());
            put_u8(h, *response_kind);
        }

        EffectRequest::RemoveCardsByKind { kind } => {
            put_u8(h, 21);
            put_u8(h, *kind);
        }

        EffectRequest::DealShaDamage {
            source,
            target,
            card,
            amount,
            nature,
            card_flags,
        } => {
            put_u8(h, 22);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u8(h, *card_flags);
        }

        EffectRequest::JudgeGainEquipmentByColor { player } => {
            put_u8(h, 24);
            put_u32(h, player.raw());
        }

        EffectRequest::RevealDiscardSuitGainOrUse {
            player,
            mark,
            base_count,
            max_mark,
            revealed,
            completed,
            step,
            data,
        } => {
            put_u8(h, 25);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u8(h, *base_count);
            put_u8(h, *max_mark);
            put_ids(h, revealed.iter().map(|card| card.raw()));
            put_u8(h, *completed);
            put_u8(h, *step);
            put_u32(h, *data);
        }

        EffectRequest::DiscardAllHandAndEquip { player } => {
            put_u8(h, 28);
            put_u32(h, player.raw());
        }

        EffectRequest::RequestDyingSave { player, source } => {
            put_u8(h, 29);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
        }

        EffectRequest::TakeOneCard {
            from,
            to,
            equip_only,
        } => {
            put_u8(h, 33);
            put_u32(h, from.raw());
            put_u32(h, to.raw());
            put_bool(h, *equip_only);
        }

        EffectRequest::TurnOver { player } => {
            put_u8(h, 34);
            put_u32(h, player.raw());
        }

        EffectRequest::RemoveSkillByDef { player, skill_def } => {
            put_u8(h, 35);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
        }

        EffectRequest::RevealTopGainNonBasicOrSwapBasic { player } => {
            put_u8(h, 37);
            put_u32(h, player.raw());
        }

        EffectRequest::PreventDamage { damage_id } => {
            put_u8(h, 38);
            put_u32(h, *damage_id);
        }

        EffectRequest::CommitDamage {
            damage_id,
            source,
            target,
            amount,
            base_amount,
            is_lose_hp,
            nature,
            card,
            card_kind,
            card_flags,
        } => {
            put_u8(h, 39);
            put_u32(h, *damage_id);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, *base_amount);
            put_bool(h, *is_lose_hp);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
        }

        EffectRequest::JudgeCancelCurrentShaTargetOnRed {
            player,
            card,
            target,
        } => {
            put_u8(h, 40);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::DealCardDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            card,
            card_kind,
            card_flags,
        } => {
            put_u8(h, 42);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_bool(h, *is_lose_hp);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
        }

        EffectRequest::AdjustHandTo { player, count } => {
            put_u8(h, 44);
            put_u32(h, player.raw());
            put_u32(h, *count);
        }

        EffectRequest::GainCardByKind { player, kind } => {
            put_u8(h, 45);
            put_u32(h, player.raw());
            put_u8(h, *kind);
        }

        EffectRequest::JudgeRecoverOnSuit {
            source,
            player,
            suit,
            amount,
        } => {
            put_u8(h, 46);
            put_u32(h, source.raw());
            put_u32(h, player.raw());
            put_u8(h, suit_tag(*suit));
            put_u32(h, *amount);
        }

        EffectRequest::GiveCards { cards, to, from } => {
            put_u8(h, 47);
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, to.raw());
            put_u32(h, from.raw());
        }

        EffectRequest::UseVirtualCardOn {
            player,
            kind,
            target,
        } => {
            put_u8(h, 48);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_u32(h, target.raw());
        }

        EffectRequest::UseFieldCard {
            player,
            card,
            targets,
        } => {
            put_u8(h, 0);
            h.update(&438u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
        }

        EffectRequest::EmitDuelResolved {
            card,
            initiator,
            target,
            winner,
            loser,
        } => {
            put_u8(h, 49);
            put_u32(h, card.raw());
            put_u32(h, initiator.raw());
            put_u32(h, target.raw());
            put_u32(h, winner.raw());
            put_u32(h, loser.raw());
        }

        EffectRequest::JudgeDamageOtherOnSuit {
            player,
            suit,
            amount,
            nature,
        } => {
            put_u8(h, 50);
            put_u32(h, player.raw());
            put_u8(h, suit_tag(*suit));
            put_u32(h, *amount);
            put_u8(h, *nature);
        }

        EffectRequest::RecastCards { player, cards } => {
            put_u8(h, 51);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
        }

        EffectRequest::DisableEquipSlots { player, slots } => {
            put_u8(h, 52);
            put_u32(h, player.raw());
            put_u32(h, slots.len() as u32);
            for slot in slots {
                put_u8(h, *slot);
            }
        }

        EffectRequest::MoveEquipmentThenDraw { player, skill } => {
            put_u8(h, 53);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::DiscardEquipmentPairWithArmor {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 55);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::DiscardTargetEquipmentGainMount { player, target } => {
            put_u8(h, 56);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::DiscardTargetCardGainMount {
            player,
            target,
            scope,
        } => {
            put_u8(h, 57);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, scope.tag());
        }

        EffectRequest::DiscardTargetCards {
            actor,
            target,
            cards,
            scope,
        } => {
            put_u8(h, 58);
            put_u32(h, actor.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, scope.tag());
        }

        EffectRequest::MakeCurrentShaTargetUnrespondable {
            source,
            card,
            target,
        } => {
            put_u8(h, 59);
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::ShaDodged {
            source,
            target,
            card,
        } => {
            put_u8(h, 60);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
        }

        EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
            player,
            target,
            damage_id,
        } => {
            put_u8(h, 62);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
        }

        EffectRequest::RefundCurrentShaUseAndDrawIfRed {
            player,
            sha_user,
            card,
            skill,
        } => {
            put_u8(h, 68);
            put_u32(h, player.raw());
            put_u32(h, sha_user.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::QueueOpeningExtraPlayPhase { player, skill } => {
            put_u8(h, 70);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::BeginTurn { player } => {
            put_u8(h, 71);
            put_u32(h, player.raw());
        }

        EffectRequest::FinishTurnBegin { player } => {
            put_u8(h, 72);
            put_u32(h, player.raw());
        }

        EffectRequest::AdvanceOpeningExtraPlay { player } => {
            put_u8(h, 73);
            put_u32(h, player.raw());
        }

        EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage {
            player,
            card,
            skill,
        } => {
            put_u8(h, 74);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::CommitAddCurrentShaTarget {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 76);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::CommitStoreHandShaInSkillPile {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 79);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::CommitSpendSkillPileForDyingJiu {
            player,
            dying,
            card,
            skill,
        } => {
            put_u8(h, 80);
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::GuanxingTop {
            player,
            count,
            round: _,
            flipped,
            top: _,
            bottom: _,
            skill,
        } => {
            put_u8(h, 132);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::GuanxingTopByFaluMarks {
            player,
            marks,
            skill,
        } => {
            put_u8(h, 134);
            put_u32(h, player.raw());
            for def in marks.iter() {
                put_u32(h, def.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::GiveSelectedThenRecoverSelf {
            player,
            target,
            cards,
            amount,
            skill,
        } => {
            put_u8(h, 130);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            for card in cards.iter() {
                put_u32(h, card.raw());
            }
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::AddBoundSkill {
            player,
            skill_def,
            bound_target,
        } => {
            put_u8(h, 82);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
            put_u32(h, bound_target.raw());
        }

        EffectRequest::CommitAddSkillBoundCardTarget {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 84);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::AddCardColorLockSkill {
            player,
            source,
            color_mask,
            skill_def,
        } => {
            put_u8(h, 86);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u8(h, *color_mask);
            put_u32(h, skill_def.raw());
        }

        EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, skill } => {
            put_u8(h, 87);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::AddCardUsePolicySkill {
            player,
            card,
            policy_mask,
            skill_def,
        } => {
            put_u8(h, 89);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *policy_mask);
            put_u32(h, skill_def.raw());
        }

        EffectRequest::UseVirtualShaDrawIfNoDamage {
            player,
            target,
            skill,
        } => {
            put_u8(h, 93);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 94);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::FinishLinkIfTargetWounded {
            player,
            target,
            toggle_link,
            skill,
        } => {
            put_u8(h, 96);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            if *toggle_link {
                put_u8(h, 1);
            }
        }

        EffectRequest::ClearPhaseGiftTarget { player, skill } => {
            put_u8(h, 99);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::EmitShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        } => {
            put_u8(h, 100);
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, response_cards.iter().map(|card| card.raw()));
            put_u8(h, *card_flags);
        }

        EffectRequest::SkipCurrentPhase { player } => {
            put_u8(h, 103);
            put_u32(h, player.raw());
        }

        EffectRequest::RevealUniformHandDrawSelfChooseOthers { player, skill } => {
            put_u8(h, 105);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::CommitTransferSavedCardUser {
            previous,
            new_user,
            card,
            skill,
        } => {
            put_u8(h, 107);
            put_u32(h, previous.raw());
            put_u32(h, new_user.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
            player,
            target,
            skill,
        } => {
            put_u8(h, 110);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::IncreaseCurrentDrawCount { player, count } => {
            put_u8(h, 114);
            put_u32(h, player.raw());
            put_u32(h, *count);
        }

        EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors {
            player,
            targets,
            index,
            selected,
            skill,
        } => {
            put_u8(h, 118);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ContinueIncreaseBoundDamage {
            player,
            damage_id,
            amount,
            skill,
        } => {
            put_u8(h, 120);
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ContinueChooseCompareTargetHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 121);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, win_skill_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::RandomizeCurrentHandCompareOpponent { player, skill } => {
            put_u8(h, 122);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::EmitHandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        } => {
            put_u8(h, 123);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, player_card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, winner.raw());
        }

        EffectRequest::GainHighestRankShaFromEventCards {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 124);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::GainHandCompareResultCard {
            player,
            initiator,
            target,
            winner,
            cards,
            skill,
        } => {
            put_u8(h, 125);
            put_u32(h, player.raw());
            put_u32(h, initiator.raw());
            put_u32(h, target.raw());
            put_u32(h, winner.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::GainDiscardCardByKind { player, kind } => {
            put_u8(h, 130);
            put_u32(h, player.raw());
            put_u8(h, *kind);
        }

        EffectRequest::RevealUniformHandChooseDamage {
            player,
            amount,
            skill,
        } => {
            put_u8(h, 131);
            put_u32(h, player.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveOlJieDanshou { player, skill } => {
            put_u8(h, 0);
            h.update(&771u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        _ => unreachable!("effect routed to a different hash group"),
    }
}
