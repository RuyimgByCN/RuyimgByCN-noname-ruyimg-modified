//! 效果原语分派（apply_effect）：通用效果原语。

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            skill,
        } => give_selected_then_target_may_use_draw_owner(
            tx, player, target, card, targets, stage, skill,
        ),

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
            player,
            target,
            damage_id,
            skill,
        } => choose_category_then_target_give_matching_or_damage_plus_one(
            tx, player, target, damage_id, skill,
        ),

        EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            player,
            source,
            card,
            stage,
            skill,
        } => choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(
            tx, player, source, card, stage, skill,
        ),

        EffectRequest::ContinueYoudishenru {
            action,
            protected,
            source,
            forced_card,
            saved,
        } => {
            crate::command::play::continue_youdishenru(
                tx,
                action,
                protected,
                source,
                forced_card,
                &saved,
            )?;
            Ok(Vec::new())
        }

        EffectRequest::ContinueWdbFuchou {
            action,
            player,
            target,
            remaining,
            skill,
        } => continue_wdb_fuchou(tx, action, player, target, remaining, skill),

        EffectRequest::ContinueDiaobingqianjiang {
            action,
            actor,
            source_card,
            targets,
            revealed,
        } => continue_diaobingqianjiang(tx, action, actor, source_card, targets, revealed),

        EffectRequest::GuozhanUseZhulianBihe { player } => {
            resolve_guozhan_use_zhulian_bihe(tx, player)
        }

        EffectRequest::CancelIncomingCardUse {
            player,
            source,
            card,
            skill,
        } => cancel_incoming_card_use(tx, player, source, card, skill),

        EffectRequest::ChooseOtherGainOneOfTwoSkills {
            player,
            target,
            first_def,
            second_def,
            skill,
        } => choose_other_gain_one_of_two_skills(tx, player, target, first_def, second_def, skill),

        EffectRequest::ContinueWugu {
            action,
            actor,
            source_card,
            targets,
            excluded,
            revealed,
        } => continue_wugu(tx, action, actor, source_card, targets, excluded, revealed),

        EffectRequest::ExcludeSavedShaTarget { card, target } => {
            exclude_saved_sha_target(tx, card, target)
        }

        _ => unreachable!("effect routed to a different primitive group"),
    }
}
