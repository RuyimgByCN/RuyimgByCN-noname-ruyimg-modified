//! PA12 Batch 17：`EffectRequest` 扩展编解码子表 5（tags 209..=238，其余委派 `rest`）。

mod rest;

use super::super::codec::{push_u32, Rd};
use super::super::codec_ext2::{push_card_vec, push_player_vec, rd_card_vec, rd_player_vec};
use super::super::*;

pub(super) fn enc_batch17_effect(w: &mut Vec<u8>, e: &EffectRequest) {
    match e {
        EffectRequest::ChooseCardUserOrAddTarget {
            player,
            card,
            skill,
        } => {
            w.push(209);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseCurrentCardUseEnhancements {
            player,
            card,
            helper_skill_def,
            skill,
        } => {
            w.push(210);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, helper_skill_def.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player,
            card,
            target,
            skill,
        } => {
            w.push(211);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseOptionalDiscardTargetCard {
            player,
            target,
            scope,
        } => {
            w.push(212);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(scope.tag());
        }
        EffectRequest::ChooseUseVirtualCard {
            player,
            kind,
            skill,
        } => {
            w.push(213);
            push_u32(w, player.raw());
            w.push(*kind);
            push_u32(w, skill.raw());
        }
        EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, skill } => {
            w.push(214);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseDiscardHandEquipThenRecover {
            player,
            count,
            amount,
            skill,
        } => {
            w.push(215);
            push_u32(w, player.raw());
            push_u32(w, *count);
            push_u32(w, *amount);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser,
            target,
            count,
            amount,
            skill,
        } => {
            w.push(216);
            push_u32(w, chooser.raw());
            push_u32(w, target.raw());
            push_u32(w, *count);
            push_u32(w, *amount);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseDiscardWeaponOrLoseHp { player, skill } => {
            w.push(217);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseMoveTargetEquipmentOrDraw {
            player,
            target,
            skill,
        } => {
            w.push(218);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseOtherHandStoreSkillPile {
            player,
            target,
            skill,
        } => {
            w.push(219);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseOtherYinghun {
            player,
            target,
            count,
            branch,
            ready_to_discard,
            skill,
        } => {
            w.push(220);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, *count);
            w.push(*branch);
            w.push(u8::from(*ready_to_discard));
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, skill } => {
            w.push(221);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
            skill,
        } => {
            w.push(222);
            push_u32(w, player.raw());
            w.push(*action);
            push_u32(w, card.raw());
            push_card_vec(w, event_cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
            player,
            amount,
            skill,
        } => {
            w.push(223);
            push_u32(w, player.raw());
            push_u32(w, *amount);
            push_u32(w, skill.raw());
        }
        EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
            player,
            target,
            skill,
        } => {
            w.push(224);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace,
        } => {
            w.push(225);
            push_u32(w, player.raw());
            push_u32(w, *damage_id);
            push_u32(w, *amount);
            push_u32(w, skill.raw());
            push_u32(w, replacement_skill_def.raw());
            w.push(u8::from(*ready_to_replace));
        }
        EffectRequest::DrawThenOptionallyDistributeHandCards {
            player,
            draw_count,
            give_cap,
            cards,
            targets,
            skill,
            ready_to_choose,
            finish,
        } => {
            w.push(226);
            push_u32(w, player.raw());
            push_u32(w, *draw_count);
            push_u32(w, *give_cap);
            push_card_vec(w, cards);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
            w.push(u8::from(*ready_to_choose));
            w.push(u8::from(*finish));
        }
        EffectRequest::ReplaceDrawWithDistinctSuitSelection {
            player,
            count,
            stage,
            cards,
            skill,
        } => {
            w.push(227);
            push_u32(w, player.raw());
            w.push(*count);
            w.push(*stage);
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::StartDebateAllOthers { player, skill } => {
            w.push(228);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::StartDebateTargets { player, skill } => {
            w.push(229);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveBailingyun {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => {
            w.push(230);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveJieliangGain {
            player,
            mover,
            cards,
            skill,
        } => {
            w.push(231);
            push_u32(w, player.raw());
            push_u32(w, mover.raw());
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveJsrgchaozheng { player, skill } => {
            w.push(232);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveStarcuxiaWeiming {
            player,
            source,
            variant,
        } => {
            w.push(233);
            push_u32(w, player.raw());
            push_u32(w, source.raw());
            push_u32(w, *variant);
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
            w.push(234);
            push_u32(w, player.raw());
            w.push(*stage);
            push_u32(w, first.raw());
            push_u32(w, second.raw());
            push_u32(w, card.raw());
            w.push(u8::from(*from_hand));
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveRebingyi { player, skill } => {
            w.push(235);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveRejizhi {
            player,
            stage,
            card,
            skill,
        } => {
            w.push(236);
            push_u32(w, player.raw());
            w.push(*stage);
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
            player,
            remaining,
            started,
            skill,
        } => {
            w.push(237);
            push_u32(w, player.raw());
            push_player_vec(w, remaining);
            w.push(u8::from(*started));
            push_u32(w, skill.raw());
        }
        EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
            player,
            target,
            skill,
        } => {
            w.push(238);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        other => rest::enc_batch17_rest(w, other),
    }
}

pub(super) fn dec_batch17_effect(tag: u8, rd: &mut Rd<'_>) -> Option<EffectRequest> {
    Some(match tag {
        209 => EffectRequest::ChooseCardUserOrAddTarget {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        210 => EffectRequest::ChooseCurrentCardUseEnhancements {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            helper_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        211 => EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        212 => EffectRequest::ChooseOptionalDiscardTargetCard {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            scope: rule_api::TargetCardScope::from_tag(rd.u8()?)?,
        },
        213 => EffectRequest::ChooseUseVirtualCard {
            player: rule_api::PlayerId::new(rd.u32()?),
            kind: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        214 => EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        215 => EffectRequest::ChooseDiscardHandEquipThenRecover {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            amount: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        216 => EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            amount: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        217 => EffectRequest::ChooseDiscardWeaponOrLoseHp {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        218 => EffectRequest::ChooseMoveTargetEquipmentOrDraw {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        219 => EffectRequest::ChooseOtherHandStoreSkillPile {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        220 => EffectRequest::ChooseOtherYinghun {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            branch: rd.u8()?,
            ready_to_discard: rd.u8()? != 0,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        221 => EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        222 => EffectRequest::ResolveDiliYuheng {
            player: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            card: rule_api::CardId::new(rd.u32()?),
            event_cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        223 => EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
            player: rule_api::PlayerId::new(rd.u32()?),
            amount: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        224 => EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        225 => EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player: rule_api::PlayerId::new(rd.u32()?),
            damage_id: rd.u32()?,
            amount: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
            replacement_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            ready_to_replace: rd.u8()? != 0,
        },
        226 => EffectRequest::DrawThenOptionallyDistributeHandCards {
            player: rule_api::PlayerId::new(rd.u32()?),
            draw_count: rd.u32()?,
            give_cap: rd.u32()?,
            cards: rd_card_vec(rd)?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
            ready_to_choose: rd.u8()? != 0,
            finish: rd.u8()? != 0,
        },
        227 => EffectRequest::ReplaceDrawWithDistinctSuitSelection {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u8()?,
            stage: rd.u8()?,
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        228 => EffectRequest::StartDebateAllOthers {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        229 => EffectRequest::StartDebateTargets {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        230 => EffectRequest::ResolveBailingyun {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        231 => EffectRequest::ResolveJieliangGain {
            player: rule_api::PlayerId::new(rd.u32()?),
            mover: rule_api::PlayerId::new(rd.u32()?),
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        232 => EffectRequest::ResolveJsrgchaozheng {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        233 => EffectRequest::ResolveStarcuxiaWeiming {
            player: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            variant: rd.u32()?,
        },
        234 => EffectRequest::ResolveReanxu {
            player: rule_api::PlayerId::new(rd.u32()?),
            stage: rd.u8()?,
            first: rule_api::PlayerId::new(rd.u32()?),
            second: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            from_hand: rd.u8()? != 0,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        235 => EffectRequest::ResolveRebingyi {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        236 => EffectRequest::ResolveRejizhi {
            player: rule_api::PlayerId::new(rd.u32()?),
            stage: rd.u8()?,
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        237 => EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
            player: rule_api::PlayerId::new(rd.u32()?),
            remaining: rd_player_vec(rd)?,
            started: rd.u8()? != 0,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        238 => EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        239..=254 => rest::dec_batch17_rest(tag, rd)?,
        _ => return None,
    })
}

pub(super) fn dec_ext_u16(ext_tag: u16, rd: &mut Rd<'_>) -> Option<EffectRequest> {
    rest::dec_ext_u16(ext_tag, rd)
}
