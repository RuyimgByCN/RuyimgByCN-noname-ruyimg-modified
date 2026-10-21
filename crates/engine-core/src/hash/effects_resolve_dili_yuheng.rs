//! Canonical encoding for `ResolveDiliYuheng` and related requests.

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&274u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, card.raw());
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::FinalizeTiaojiyanmei {
            actor,
            left,
            right,
            card,
        } => {
            put_u8(h, 0);
            h.update(&275u16.to_le_bytes());
            put_u32(h, actor.raw());
            put_u32(h, left.raw());
            put_u32(h, right.raw());
            put_u32(h, card.raw());
        }

        EffectRequest::BeginTiaojiyanmeiDiscard {
            actor,
            left,
            right,
            higher,
        } => {
            put_u8(h, 0);
            h.update(&276u16.to_le_bytes());
            put_u32(h, actor.raw());
            put_u32(h, left.raw());
            put_u32(h, right.raw());
            put_u32(h, higher.raw());
        }

        EffectRequest::DiscardSelectedDrawAllHandBonus {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&277u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherYinghun {
            player,
            target,
            count,
            branch,
            ready_to_discard,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&278u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u8(h, *branch);
            put_bool(h, *ready_to_discard);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherBizheng {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&279u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherXiashu {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&280u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGuanchaoRank {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&281u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGuanwei {
            player,
            target,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&282u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLanjiang {
            player,
            stage,
            index,
            target,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&283u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, *index);
            put_u32(h, target.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRebingyi { player, skill } => {
            put_u8(h, 0);
            h.update(&284u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRebotu { player, skill } => {
            put_u8(h, 0);
            h.update(&285u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveOldimeng {
            player,
            stage,
            first,
            second,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&286u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, first.raw());
            put_u32(h, second.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRejieyin {
            player,
            target,
            stage,
            card,
            equip,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&287u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_bool(h, *equip);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveReanxu {
            player,
            stage,
            first,
            second,
            card,
            from_hand,
            skill,
        } => {
            h.update(&288u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, first.raw());
            put_u32(h, second.raw());
            put_u32(h, card.raw());
            put_bool(h, *from_hand);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltJueyan {
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            skill,
        } => {
            h.update(&289u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u8(h, *slot);
            put_bool(h, *first);
            put_u32(h, sha_skill_def.raw());
            put_u32(h, weapon_skill_def.raw());
            put_u32(h, mount_skill_def.raw());
            put_u32(h, armor_skill_def.raw());
            put_u32(h, treasure_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRejizhi {
            player,
            stage,
            card,
            skill,
        } => {
            h.update(&290u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJiexun {
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            skill,
        } => {
            h.update(&291u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, *draw_count);
            put_u32(h, *discard_count);
            put_u32(h, funan_helper_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveWenguaSetup {
            player,
            helper_skill_def,
            skill,
        } => {
            h.update(&292u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveWengua {
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            skill,
        } => {
            h.update(&293u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, main_skill_def.raw());
            put_u32(h, marker_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill,
        } => {
            h.update(&294u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *index);
            put_u8(h, *remaining);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGanglieThree {
            player,
            target,
            skill,
        } => {
            h.update(&295u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhongyi {
            player,
            action,
            helper_skill_def,
            card,
            skill,
        } => {
            h.update(&296u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, helper_skill_def.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::DiscardEquipmentSlot { player, slot } => {
            h.update(&297u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *slot);
        }

        EffectRequest::ResolveOldAnxu {
            player,
            stage,
            giver,
            receiver,
            card,
            skill,
        } => {
            h.update(&298u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, giver.raw());
            put_u32(h, receiver.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::StoreSelectedInSkillPile {
            player,
            cards,
            skill,
        } => {
            h.update(&299u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::EndMatchForCampIfSkillPileAtLeast {
            player,
            pile_skill_def,
            count,
        } => {
            h.update(&300u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, pile_skill_def.raw());
            put_u32(h, *count);
        }

        EffectRequest::ResolveOldQinqing {
            player,
            target,
            lord,
            stage,
            skill,
        } => {
            h.update(&301u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, lord.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveOldMingjian {
            player,
            target,
            stage,
            skill,
        } => {
            h.update(&302u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveOldXuanfeng {
            player,
            target,
            card,
            stage,
            branch,
            skill,
        } => {
            h.update(&303u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJianrou {
            player,
            damage_id,
            skill,
        } => {
            h.update(&304u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveOldHuisheng {
            player,
            source,
            damage_id,
            stage,
            cards,
            skill,
        } => {
            h.update(&305u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, *damage_id);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveOldJiefan {
            player,
            dying,
            current,
            card,
            damage_id,
            stage,
            skill,
        } => {
            h.update(&306u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, current.raw());
            put_u32(h, card.raw());
            put_u32(h, *damage_id);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveChegu {
            player,
            card,
            targets,
            cards,
            branch,
            index,
            stage,
            skill,
        } => {
            h.update(&307u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, *branch);
            put_u8(h, *index);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJunkguixin {
            player,
            target,
            stage,
            group,
            skill,
        } => {
            h.update(&308u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *group);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveYaoChenwei {
            player,
            target,
            card,
            mode,
            stage,
            skill,
        } => {
            h.update(&309u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *mode);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            card,
            max_distance,
            skill,
        } => {
            h.update(&310u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, *max_distance);
            put_u32(h, skill.raw());
        }

        EffectRequest::RefundCurrentShaUse {
            player,
            card,
            skill,
        } => {
            h.update(&311u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResetLinkedSelectedTargets { source, targets } => {
            h.update(&374u16.to_le_bytes());
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
        }

        EffectRequest::DamageThenDrawSelectedTargets {
            source,
            targets,
            damage,
            draw,
            skill,
        } => {
            h.update(&312u16.to_le_bytes());
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *damage);
            put_u32(h, *draw);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveYoudi {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            h.update(&313u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            h.update(&[*stage]);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveKuangfu {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            h.update(&314u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            h.update(&[*stage]);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShanjia {
            player,
            count,
            stage,
            skill,
        } => {
            h.update(&315u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *count);
            h.update(&[*stage]);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveNewYajiao {
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill,
        } => {
            h.update(&316u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source_card.raw());
            put_u32(h, revealed.raw());
            put_u32(h, target.raw());
            h.update(&[*source_category, *stage]);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDingpin {
            player,
            target,
            skill,
        } => {
            h.update(&317u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::FinishCurrentPhaseAbortResolution { player, stage } => {
            h.update(&318u16.to_le_bytes());
            put_u32(h, player.raw());
            h.update(&[*stage]);
        }

        EffectRequest::GainTopCardToSkillPile { player, skill } => {
            h.update(&319u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player,
            target,
            helper_skill_def,
            selected,
            skill,
        } => {
            h.update(&320u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::GainOneHandFromSelectedTargets {
            player,
            targets,
            index,
            selected,
            skill,
        } => {
            h.update(&321u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseRepoluTargetsDraw { player, skill } => {
            h.update(&322u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinfuGuolun {
            player,
            target,
            player_card,
            target_card,
            skill,
        } => {
            h.update(&323u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, player_card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
            player,
            sha_user,
            card,
            restore_skill_def,
            skill,
        } => {
            h.update(&324u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, sha_user.raw());
            put_u32(h, card.raw());
            put_u32(h, restore_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill,
        } => {
            h.update(&325u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, result_card.raw());
            put_u8(h, crate::command::judge::suit_tag(*suit) as u8);
            put_u8(h, *rank);
            put_ids(h, rewriters.iter().map(|player| player.raw()));
            put_u32(h, *heart_recover_amount);
            put_u32(h, *diamond_draw_count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhuiyi {
            player,
            killer,
            skill,
        } => {
            h.update(&326u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, killer.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDecadezhenjun {
            player,
            target,
            stage,
            count,
            skill,
        } => {
            h.update(&327u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpXiaoguo {
            player,
            target,
            stage,
            skill,
        } => {
            h.update(&328u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            initialized,
            remaining_targets,
            skill,
        } => {
            h.update(&329u16.to_le_bytes());
            put_u32(h, player.raw());
            put_bool(h, *initialized);
            put_ids(h, remaining_targets.iter().map(|player| player.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards,
            amount,
            skill,
            ready_to_recover,
        } => {
            h.update(&330u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, *amount);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_recover);
        }

        EffectRequest::ResolveDingpan {
            player,
            target,
            stage,
            skill,
        } => {
            h.update(&331u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTanlu {
            player,
            target,
            amount,
            stage,
            skill,
        } => {
            h.update(&332u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::JudgeSameColorGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
            color,
        } => {
            h.update(&333u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
            put_u8(h, *color);
        }

        EffectRequest::ChooseCurrentTrickTargetsToNullify {
            player,
            source,
            card,
            skill,
        } => {
            h.update(&334u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
            player,
            card,
            skill,
            ready_to_choose,
        } => {
            h.update(&335u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }

        EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
        } => {
            h.update(&336u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, sha_card.raw());
            put_ids(h, shan_cards.iter().map(|card| card.raw()));
            put_ids(h, excluded_recipients.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
            player,
            targets,
            index,
            selected,
            draw_count,
            skill,
        } => {
            h.update(&337u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, *draw_count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
            source,
            card,
            target,
            bonus_skill_def,
        } => {
            h.update(&338u16.to_le_bytes());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, bonus_skill_def.raw());
        }

        EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            targets,
            target,
            selected,
            helper_skill_def,
            stage,
            skill,
        } => {
            h.update(&339u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, target.raw());
            put_u32(h, selected.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage,
            cards,
            accepted,
            skill,
        } => {
            h.update(&340u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, buff_skill_def.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_bool(h, *accepted);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRemoshi {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill,
        } => {
            h.update(&341u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinyaoming {
            player,
            target,
            action,
            skill,
        } => {
            h.update(&342u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRehuaiyi {
            player,
            stage,
            color,
            discarded_count,
            targets,
            index,
            gained_count,
            skill,
        } => {
            h.update(&343u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u8(h, *color);
            put_u32(h, *discarded_count);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, *gained_count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveReanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill,
        } => {
            h.update(&344u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *index);
            put_u8(h, *remaining);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
            player,
            target,
            damage_id,
            skill,
        } => {
            h.update(&345u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
            player,
            target,
            skill,
        } => {
            h.update(&346u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveBeizhu {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            h.update(&347u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbshanjia {
            player,
            lost_equipment_count,
            distance_skill_def,
            stage,
            reward_mask,
            skill,
        } => {
            h.update(&348u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *lost_equipment_count);
            put_u32(h, distance_skill_def.raw());
            put_u8(h, *stage);
            put_u8(h, *reward_mask);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDaoji {
            player,
            target,
            card,
            stage,
            tw,
            skill,
        } => {
            h.update(&349u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            if *tw {
                h.update(b"tw-daoji/v1");
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJiaohua {
            player,
            category,
            target,
            skill,
        } => {
            h.update(&350u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *category);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLuanqun {
            player,
            effect_skill_def,
            direct_hit_skill_def,
            stage,
            participants,
            cards,
            skill,
        } => {
            h.update(&351u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, effect_skill_def.raw());
            put_u32(h, direct_hit_skill_def.raw());
            put_u8(h, *stage);
            put_ids(h, participants.iter().map(|player| player.raw()));
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
            owner,
            giver,
            target,
            skill,
        } => {
            h.update(&352u16.to_le_bytes());
            put_u32(h, owner.raw());
            put_u32(h, giver.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbzhuguo {
            player,
            target,
            stage,
            skill,
        } => {
            h.update(&353u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbanxian {
            player,
            cards,
            card,
            stage,
            skill,
        } => {
            h.update(&354u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
            skill,
        } => {
            h.update(&355u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::TakeOneScopedCard { from, to, scope } => {
            h.update(&356u16.to_le_bytes());
            put_u32(h, from.raw());
            put_u32(h, to.raw());
            put_u8(h, scope.tag());
        }

        EffectRequest::RevealHand { player } => {
            h.update(&357u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::DrawLinkedPlayers { count } => {
            h.update(&358u16.to_le_bytes());
            put_u32(h, *count);
        }

        EffectRequest::RevealTopGainIfColor { player, color_mask } => {
            put_u8(h, 0);
            h.update(&362u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *color_mask);
        }

        EffectRequest::ResolveMuzhen { .. } => {
            put_u8(h, 0);
            h.update(&993u16.to_le_bytes());
        }

        EffectRequest::RefreshSkill { skill } => {
            put_u8(h, 0);
            h.update(&994u16.to_le_bytes());
            put_u32(h, skill.raw());
        }

        EffectRequest::TagCard { cards, tag } => {
            put_u8(h, 0);
            h.update(&995u16.to_le_bytes());
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, *tag);
        }

        EffectRequest::UntagCard { cards, tag } => {
            put_u8(h, 0);
            h.update(&996u16.to_le_bytes());
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, *tag);
        }

        EffectRequest::ClanshenjunReveal {
            player,
            event_card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&997u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ClanshenjunUse { player, skill } => {
            put_u8(h, 0);
            h.update(&998u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::StorageSet { skill, key, value } => {
            put_u8(h, 0);
            h.update(&999u16.to_le_bytes());
            put_u32(h, skill.raw());
            put_u32(h, *key);
            put_u32(h, *value);
        }

        EffectRequest::DrawAllEventTargets { players, count } => {
            put_u8(h, 0);
            h.update(&1000u16.to_le_bytes());
            put_u32(h, players.len() as u32);
            for player in players {
                put_u32(h, player.raw());
            }
            put_u32(h, *count);
        }

        EffectRequest::RoundEnded { round, current } => {
            put_u8(h, 0);
            h.update(&1001u16.to_le_bytes());
            put_u32(h, *round);
            put_u32(h, current.raw());
        }

        EffectRequest::RoundStarted { round, current } => {
            put_u8(h, 0);
            h.update(&1002u16.to_le_bytes());
            put_u32(h, *round);
            put_u32(h, current.raw());
        }

        EffectRequest::ClanyuzhiReveal { player, skill } => {
            put_u8(h, 0);
            h.update(&1003u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ClanyuzhiRoundEnd { player, skill } => {
            put_u8(h, 0);
            h.update(&1004u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::QieyiPhaseStart { player, skill } => {
            put_u8(h, 0);
            h.update(&1005u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::QieyiUseAfter {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&1006u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::JianzhiPhaseEnd { player, skill } => {
            put_u8(h, 0);
            h.update(&1007u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        _ => unreachable!("effect routed to a different hash group"),
    }
}
