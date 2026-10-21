//! Canonical encoding for `ResolveTwluanchou` and related requests.

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ResolveTwluanchou {
            player,
            helper_skill_def,
            action,
            targets,
            source,
            target,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&398u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTianxiang {
            player,
            target,
            discard_card,
            source,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            draw_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&416u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, discard_card.raw());
            put_u32(h, source.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
            put_u32(h, draw_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&399u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *wangxing_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwbianfuren {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&400u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwfeiyi {
            player,
            action,
            stage,
            card,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&401u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwchenzhen {
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            stage,
            card_kind,
            card,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&402u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u32(h, *record_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u8(h, *card_kind);
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwqianju {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&403u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwqingxi {
            player,
            target,
            card,
            stage,
            count,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&404u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u8(h, *count);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwzaoli {
            player,
            stage,
            cards,
            slots,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&405u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, slots.len() as u32);
            h.update(slots);
            put_u8(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwzhengjian {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&406u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwtianyu {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&407u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwwujing {
            player,
            target_count,
            action,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&408u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *target_count);
            put_u8(h, *action);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwshanghe {
            player,
            stage,
            remaining,
            got_jiu,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&409u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_ids(h, remaining.iter().map(|target| target.raw()));
            put_bool(h, *got_jiu);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwwangchang {
            player,
            target,
            card,
            selected_card,
            action,
            stage,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&410u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, selected_card.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwcaozhao {
            player,
            target,
            source,
            card,
            selected_skill_def,
            twfuzuan_skill_def,
            twfeifu_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&411u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *selected_skill_def);
            put_u32(h, *twfuzuan_skill_def);
            put_u32(h, *twfeifu_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwguohuai {
            player,
            target,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&412u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *mark_def);
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwchengpu {
            player,
            target,
            source,
            card,
            chunlao_skill_def,
            action,
            stage,
            selected_card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&413u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *chunlao_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, selected_card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwcaoang {
            player,
            target,
            source,
            card,
            helper_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&415u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwniujin {
            player,
            limited_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&417u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, limited_skill_def.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRetianxiang {
            player,
            target,
            discard_card,
            branch,
            source,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&418u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, discard_card.raw());
            put_u8(h, *branch);
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLianzhou {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&419u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveStddechong {
            player,
            target,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&420u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::DiscardHandEquipCount {
            player,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&422u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJieliangGain {
            player,
            mover,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&423u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, mover.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveStdjujian {
            player,
            target,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&424u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLulve {
            player,
            target,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&427u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDctingxian {
            player,
            card,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&429u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHfjieying {
            player,
            target,
            grant_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&430u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, grant_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcnuchen {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&432u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcdanji { player, skill } => {
            put_u8(h, 0);
            h.update(&435u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDczhiheng { player, skill } => {
            put_u8(h, 0);
            h.update(&436u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveStdgaobian {
            player,
            chooser,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&437u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, chooser.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpjiedao { player, skill } => {
            put_u8(h, 0);
            h.update(&439u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSibian { player, skill } => {
            put_u8(h, 0);
            h.update(&440u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::SkipNextDiscardPhase { player } => {
            put_u8(h, 0);
            h.update(&441u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::ResolveSpfengyin {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&442u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinfuLianpian {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&443u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGongjian {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&444u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
            player,
            reveal_count,
            recipient_cap,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&450u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *reveal_count);
            put_u8(h, *recipient_cap);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { player, skill } => {
            put_u8(h, 0);
            h.update(&451u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRefuhai { player, skill } => {
            put_u8(h, 0);
            h.update(&456u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGainTargetHandCeilHalfDifference {
            player,
            target,
            card,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 0);
            h.update(&461u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u8(h, u8::from(*ready_to_choose));
        }

        EffectRequest::ResolveGzjinfa {
            player,
            target,
            cost_card,
            given_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&464u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, cost_card.raw());
            put_u32(h, given_card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveWnQifeng {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&466u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinkuangfu {
            player,
            target,
            sha_target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&467u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, sha_target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcxiangshu {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&469u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcxiangshuLottery {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&470u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcyingtu {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&471u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&468u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
            player,
            target,
            damage_id,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&472u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            player,
            source,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&473u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMobiledanshou {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&474u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJsrgchiying {
            player,
            target,
            started,
            remaining,
            accum,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&475u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_bool(h, *started);
            put_u32(h, remaining.len() as u32);
            for participant in remaining {
                put_u32(h, participant.raw());
            }
            put_u32(h, accum.len() as u32);
            for card in accum {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::EmitCardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        } => {
            put_u8(h, 0);
            h.update(&476u16.to_le_bytes());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *kind);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, damaged_targets.iter().map(|target| target.raw()));
            put_ids(h, source_cards.iter().map(|card| card.raw()));
            put_bool(h, *all_sources_were_hand);
        }

        EffectRequest::ResolveMbqingdao {
            player,
            source,
            card,
            damaged,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&477u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_bool(h, *damaged);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbxiugeng {
            player,
            helper_def,
            hand_limit_def,
            stage,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&478u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_def.raw());
            put_u32(h, hand_limit_def.raw());
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbchenshe {
            player,
            dying,
            source,
            stage,
            roles,
            suits,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&479u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_ids(h, roles.iter().map(|target| target.raw()));
            put_u32(h, suits.len() as u32);
            for suit in suits {
                put_u8(h, *suit);
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbquanchong {
            player,
            helper_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&480u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbrenxing {
            player,
            stage,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&481u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbchizhang {
            player,
            card,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&482u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbduanyang {
            player,
            target,
            card,
            mode,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&483u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *mode);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage,
            value,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&484u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *value);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbshuanghuai {
            player,
            target,
            stage,
            choice,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&485u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *choice);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&486u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbganggeng {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&487u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbsijian {
            player,
            target,
            stage,
            choice,
            helper_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&488u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *choice);
            put_u32(h, helper_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbjianji {
            player,
            source,
            target,
            stage,
            card,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&489u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbyuanmo {
            player,
            target,
            stage,
            card,
            value,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&490u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, *value);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbfeijing {
            player,
            target,
            card,
            stage,
            targets,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&491u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbxiaoge {
            player,
            target,
            card,
            stage,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&492u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShuangxiong { player, skill } => {
            put_u8(h, 0);
            h.update(&493u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhijian {
            player,
            card,
            stage,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&494u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGuzheng {
            player,
            event_player,
            stage,
            cards,
            return_card,
            take_rest,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&495u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().copied());
            put_u32(h, return_card.raw());
            put_u8(h, u8::from(*take_rest));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhiba {
            player,
            actor,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&496u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhibaSettle {
            player,
            actor,
            winner,
            player_card,
            target_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&497u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, winner.raw());
            put_u32(h, player_card.raw());
            put_u32(h, target_card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltCongjian {
            player,
            targets,
            card,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&498u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveNzryFeijun {
            player,
            target,
            card,
            branch,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&499u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltYongsi {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&500u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltWeidi {
            player,
            cards,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&501u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLieren {
            player,
            target,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&502u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, win_skill_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbxuye {
            player,
            target,
            stage,
            was_max,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&496u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, u8::from(*was_max));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&497u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage,
            damage_id,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&498u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotjugu { player, skill } => {
            put_u8(h, 0);
            h.update(&499u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotsifeng {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&500u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotchenqun {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&501u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotwanglie {
            player,
            card,
            helper_skill_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&502u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePothongyi {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&503u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotguansha {
            player,
            helper_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&504u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&505u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbzhuji {
            player,
            choice,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&506u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *choice);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotlibing {
            player,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&507u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotpoxi {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&508u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, cards.len() as u32);
            for card in cards {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotfuan {
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&509u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, range_helper_def.raw());
            put_u32(h, yinxian_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotyinxian {
            player,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&510u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePothuanjie {
            player,
            target,
            stage,
            qice_def,
            kanpo_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&511u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, qice_def.raw());
            put_u32(h, kanpo_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotnigu {
            player,
            target,
            card,
            targets,
            index,
            declined,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&512u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *index);
            put_u8(h, *declined);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        _ => unreachable!("effect routed to a different hash group"),
    }
}
