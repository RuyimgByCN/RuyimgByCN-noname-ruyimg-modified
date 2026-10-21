//! Canonical encoding for `ResolvePotlulian` and related requests.

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ResolvePotlulian {
            player,
            card,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&513u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotdimeng {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&514u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePothaoshi {
            player,
            target,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&515u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpmiaolve {
            player,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&516u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpyingjia {
            player,
            action,
            helper_def,
            card,
            card_kind,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&517u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, helper_def.raw());
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMantianguohai {
            player,
            stage,
            payload,
        } => {
            put_u8(h, 0);
            h.update(&518u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_ids(h, payload.iter().copied());
        }

        EffectRequest::ResolveXinfuQinguo {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&445u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcmanzhi { player, skill } => {
            put_u8(h, 0);
            h.update(&446u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinxingluan { player, skill } => {
            put_u8(h, 0);
            h.update(&447u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDckuizhen { player, skill } => {
            put_u8(h, 0);
            h.update(&448u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDckuizhenResolve {
            player,
            winner,
            loser,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&449u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, winner.raw());
            put_u32(h, loser.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveNsdingzhou { player, skill } => {
            put_u8(h, 0);
            h.update(&454u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSxrmlanjiao {
            player,
            target,
            stage,
            view,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&455u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, view.len() as u32);
            for card in view {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRekuanshi { player, skill } => {
            put_u8(h, 0);
            h.update(&457u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRekuanshiResolve {
            player,
            damaged,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&458u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, damaged.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcmoyu { player, skill } => {
            put_u8(h, 0);
            h.update(&459u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXizhen { player, skill } => {
            put_u8(h, 0);
            h.update(&460u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXuxie { player, skill } => {
            put_u8(h, 0);
            h.update(&462u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcchiying {
            player,
            target,
            stage,
            accum,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&465u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, accum.len() as u32);
            for card in accum {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXuxieAdd { player, skill } => {
            put_u8(h, 0);
            h.update(&463u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwmoukui {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&421u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&425u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveCuijin {
            player,
            source,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&426u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwqingkou {
            player,
            winner,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&428u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, winner.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwchenxun {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&431u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwfenxian {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&452u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::SetSkillContentFlag {
            player,
            bit,
            value,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&453u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *bit);
            put_u8(h, u8::from(*value));
            put_u32(h, skill.raw());
        }

        EffectRequest::CommitDamageBanActiveIfDying {
            damage_id,
            source,
            target,
            amount,
            base_amount,
            nature,
            skill,
        } => {
            put_u8(h, 148);
            put_u32(h, *damage_id);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, *base_amount);
            put_u8(h, *nature);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            skill,
        } => {
            put_u8(h, 149);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            card,
            before,
            skill,
        } => {
            put_u8(h, 150);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *before);
            put_u32(h, skill.raw());
        }

        EffectRequest::FinishTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            before,
            skill,
        } => {
            put_u8(h, 151);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *before);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseUseVirtualCard {
            player,
            kind,
            skill,
        } => {
            put_u8(h, 152);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseUseVirtualNanman { player, skill } => {
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQiluan { player, skill } => {
            put_u8(h, 155);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::DiscardAllOwnJudge { player, skill } => {
            put_u8(h, 156);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::StoreEventCardInSkillPile {
            player,
            card,
            skill,
        } => {
            put_u8(h, 157);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveOllinjie {
            player,
            source,
            skill,
        } => {
            put_u8(h, 159);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetGainMaxHp { player, skill } => {
            // Reuses digest tag 159 with a distinct 2x u32 payload shape.
            put_u8(h, 159);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::GainTargetHandCard { player, skill } => {
            // Reuses digest tag 154 (existing uses are 4x u32) with a distinct
            // 2x u32 payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseClanNotMaxHandLimitGain { player, skill } => {
            // Reuses digest tag 154 with a distinct 3x u32 payload shape
            // (existing 154 uses are 4x u32 and 2x u32).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4D55_5949);
        }

        EffectRequest::GainWholeSkillPile {
            player,
            skill,
            pile_def,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt GWHP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, 0x4757_5048);
        }

        EffectRequest::ChooseOneFromSkillPileDiscard {
            player,
            skill,
            pile_def,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt CHOD).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, 0x4348_5044);
        }

        EffectRequest::PlaceHandCardsToDeckTop {
            player,
            skill,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt PHDT).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *count);
            put_u32(h, 0x5048_4454);
        }

        EffectRequest::RearrangePileToDeckTop {
            player,
            skill,
            pile_def,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt CHTP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, *count);
            put_u32(h, 0x5250_5444);
        }

        EffectRequest::ResolveQianlong { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt QLN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x51_4C_4E);
        }

        EffectRequest::ResolveJsrgzuozhan { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt ZUZH).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5A55_5A48);
        }

        EffectRequest::ResolveJsrgzuozhanGain {
            player,
            skill,
            event_player,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt ZUZG).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, event_player.raw());
            put_u32(h, 0x5A55_5A47);
        }

        EffectRequest::ResolveTwmouzhi {
            player,
            skill,
            mode,
            event_card,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt MOUZ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *mode);
            put_u32(h, *event_card);
            put_u32(h, 0x4D4F_555A);
        }

        EffectRequest::ResolveTwdaigui {
            player,
            skill,
            stage,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt TWDG).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, *count);
            put_u32(h, 0x5457_4447);
        }

        EffectRequest::ResolveTwmitu {
            player,
            skill,
            stage,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt TWMT).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, *count);
            put_u32(h, 0x5457_4D54);
        }

        EffectRequest::ResolveQiantun { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt QTN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5147_544E);
        }

        EffectRequest::ResolveYangming { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt YML).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x594D_4C);
        }

        EffectRequest::ResolveCanyun { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt CYY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4359_59);
        }

        EffectRequest::ResolveHengwu {
            player,
            skill,
            event_card,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt HENW).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *event_card);
            put_u32(h, 0x4845_4E57);
        }

        EffectRequest::ResolveBiaozhao3 { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt BZ3).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x425A_33);
        }

        EffectRequest::ResolveChuanyun {
            player,
            skill,
            event_player,
            event_card,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt CYN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, event_player.raw());
            put_u32(h, *event_card);
            put_u32(h, 0x4359_4E);
        }

        EffectRequest::ResolveBiaozhao2 {
            player,
            skill,
            event_player,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt BZ2).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, event_player.raw());
            put_u32(h, 0x425A_32);
        }

        EffectRequest::ResolveDcniji {
            player,
            skill,
            mode,
            event_card,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt NJI).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *mode);
            put_u32(h, *event_card);
            put_u32(h, 0x4E4A_49);
        }

        EffectRequest::ResolveDcmoukui {
            player,
            skill,
            stage,
            target,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt MKU).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, 0x4D4B_55);
        }

        EffectRequest::ResolveZhongyan {
            player,
            skill,
            stage,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt ZZY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, 0x5A5A_59);
        }

        EffectRequest::ResolveChumei {
            player,
            skill,
            stage,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt CHM).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, 0x4348_4D);
        }

        EffectRequest::ResolveShuaiyan {
            player,
            skill,
            stage,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt SHY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, 0x5348_59);
        }

        EffectRequest::CommitGaoyuanRedirect {
            player,
            source,
            card,
            target,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt GY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, 0x4747_59);
        }

        EffectRequest::ResolveTwxiongjun { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt XJ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x584A);
        }

        EffectRequest::ResolveYixiang { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt YX).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5958);
        }

        EffectRequest::ResolveXinfuQiai { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt FQ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x4651);
        }

        EffectRequest::ResolveSpolaige { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt PL).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x504C);
        }

        EffectRequest::ResolveOlzhijie { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt ZJ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5A4A);
        }

        EffectRequest::ResolveOlsuyi { player, target } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt SY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, 0x5359);
        }

        EffectRequest::ResolveOlqiaozhi { player, stage } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt QZ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *stage);
            put_u32(h, 0x515A);
        }

        EffectRequest::ResolveDczhee { player, mode } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt ZE).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *mode);
            put_u32(h, 0x5A45);
        }

        EffectRequest::ResolveDchaoxian { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt CX).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x4358);
        }

        EffectRequest::ResolveZhongjie { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt ZJ2).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5A4A);
        }

        EffectRequest::ResolveStarcuxiaWeiming {
            player,
            source,
            variant,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt XW).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, *variant);
            put_u32(h, 0x5857);
        }

        EffectRequest::ResolveSanshou { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt SS).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5353);
        }

        EffectRequest::ResolveNhlundao { player, source } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt LD).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, 0x4C44);
        }

        EffectRequest::ResolveXiangshu { player, targets } => {
            // Reuses digest tag 154 with a list + 1x u32 + salt payload shape (salt XS).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, 0x5853);
        }

        EffectRequest::ResolveOlwenren {
            player,
            targets,
            skill,
        } => {
            // Digest tag 158-shaped list payload (targets + skill, salt WR).
            put_u8(h, 158);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
            put_u32(h, 0x5752);
        }

        EffectRequest::ResolveDcqiaolve { player, target } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt QL).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, 0x514C);
        }

        EffectRequest::ResolveDingce { player, source } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt DCE).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, 0x4443_45);
        }

        EffectRequest::ResolveQiangyong {
            player,
            target,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt QY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5159);
        }

        EffectRequest::ResolveMeiyong { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt MY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x4D5_9);
        }

        EffectRequest::ResolveMbzhenann { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt ZNN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5A4E_4E);
        }

        EffectRequest::ResolveFriendbiwei {
            player,
            target,
            card,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt FBW).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4642_57);
        }

        EffectRequest::ResolveChulao { player, targets } => {
            // Reuses digest tag 154 with a distinct ids-list + salt payload shape (salt CLC).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, 0x434C_43);
        }

        EffectRequest::ResolveDcanxu {
            player,
            targets,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct ids-list + salt payload shape (salt DCX).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
            put_u32(h, 0x4443_58);
        }

        EffectRequest::ResolveDuoduan {
            player,
            source,
            card,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt DDN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4444_4E);
        }

        EffectRequest::ResolveGaoyuan {
            player,
            source,
            card,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt GY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, 0x4747_59);
        }

        EffectRequest::ResolveMbbeiming {
            player,
            skill,
            stage,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt BBM).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, 0x4242_4D);
        }

        EffectRequest::ResolveZhengjian {
            player,
            skill,
            stage,
            event_player,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt ZJZJ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, event_player.raw());
            put_u32(h, 0x5A4A_5A4A);
        }

        EffectRequest::ResolveReliangyin {
            player,
            skill,
            mode,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt RLY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *mode);
            put_u32(h, 0x524C_59);
        }

        EffectRequest::ResolveQinmian {
            player,
            skill,
            mode,
            amount,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt QMM).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *mode);
            put_u32(h, *amount);
            put_u32(h, 0x514D_4D);
        }

        EffectRequest::ResolveJsrgdaoren { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt JSDR).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4A53_4452);
        }

        EffectRequest::ResolveDuwang { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt DUW).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4455_57);
        }

        EffectRequest::ResolveQuanjia {
            player,
            skill,
            source,
            target,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt QJA).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, 0x514A_41);
        }

        EffectRequest::ResolvePxqixian {
            player,
            skill,
            stage,
            kind,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt PXQX).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, *kind);
            put_u32(h, 0x5058_5158);
        }

        EffectRequest::ResolveDcbingji {
            player,
            skill,
            stage,
            kind,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt DCBJ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, *kind);
            put_u32(h, 0x4443_424A);
        }

        EffectRequest::ChooseHandCardsToSkillPile {
            player,
            skill,
            min_count,
            max_count,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt CHSP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *min_count);
            put_u32(h, *max_count);
            put_u32(h, 0x4348_5350);
        }

        EffectRequest::ChooseTargetHandCardToPile {
            player,
            skill,
            pile_def,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt CHTP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, 0x4348_5450);
        }

        EffectRequest::PlaceTopDeckToExpansion {
            player,
            skill,
            pile_def,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt DPTP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, *count);
            put_u32(h, 0x4450_5450);
        }

        EffectRequest::DiscardSkillPile {
            player,
            skill,
            pile_def,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt DSKP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, 0x4453_4B50);
        }

        EffectRequest::GainFromSkillPile { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x474169_4e);
        }

        EffectRequest::DiscardBasicDrawEqual { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5943_4841);
        }

        EffectRequest::StartDebateAllOthers { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5344_4231);
        }

        EffectRequest::StartDebateTargets { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5344_4232);
        }

        EffectRequest::PinDebateOpinions { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5044_4F33);
        }

        EffectRequest::AlignDebateOpinions { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4144_4F34);
        }

        EffectRequest::GainShadowCards {
            player,
            count,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_u32(h, 0x4753_4335);
        }

        EffectRequest::ChangeGroup {
            player,
            group,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, group.len() as u32);
            put_u32(h, skill.raw());
            put_u32(h, 0x4347_5236);
        }

        EffectRequest::DistributeShadowCards {
            player,
            count,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_u32(h, 0x4453_4337);
        }

        EffectRequest::RegisterTurnEndDebate { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5244_4238);
        }

        EffectRequest::ResolveTurnEndDebates { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5254_4439);
        }

        EffectRequest::ChooseGroup { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4348_4750);
        }

        EffectRequest::ChooseGainOtherSkill { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4347_4F53);
        }

        EffectRequest::LoseGainedSkill { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4C47_534B);
        }

        EffectRequest::ResolvePotheyun { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5054_4859);
        }

        EffectRequest::PlaceXumou {
            player,
            card,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5058_4D31);
        }

        EffectRequest::PlaceDeckTopXumou { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5058_4D32);
        }

        EffectRequest::ChoosePlaceHandXumou { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5058_4D33);
        }

        EffectRequest::DiscardOneOwnXumou { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5058_4D34);
        }

        EffectRequest::PlaceDeckTopExpansion {
            player,
            count,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_u32(h, 0x4348_5431);
        }

        EffectRequest::CityConsume { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4348_5432);
        }

        EffectRequest::QueueExtraTurn {
            player,
            check,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *check);
            put_u32(h, skill.raw());
            put_u32(h, 0x4554_5232);
        }

        EffectRequest::ChooseOthersInsertExtraTurn {
            player,
            check,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *check);
            put_u32(h, skill.raw());
            put_u32(h, 0x4945_5431);
        }

        EffectRequest::QueueExtraPhases {
            player,
            phases_mask,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *phases_mask);
            put_u32(h, skill.raw());
            put_u32(h, 0x5048_4153);
        }

        EffectRequest::ResolveTwjieqiu {
            player,
            target,
            subaction,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *subaction);
            put_u32(h, skill.raw());
            put_u32(h, 0x4A51_4A31);
        }

        EffectRequest::SkipRemainingPhases {
            player,
            phases_mask,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *phases_mask);
            put_u32(h, skill.raw());
            put_u32(h, 0x5352_5031);
        }

        EffectRequest::ChooseOthersAddMark {
            player,
            mark,
            delta,
            max_count,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 5x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *mark);
            put_u32(h, *delta as u32);
            put_u32(h, *max_count);
            put_u32(h, skill.raw());
            put_u32(h, 0x4F41_4D31);
        }

        EffectRequest::ResolveJsrgchaozheng { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x435A_5A47);
        }

        EffectRequest::ResolveJsrgsheju {
            player,
            counterpart,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, counterpart.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x534A_4A55);
        }

        EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *mark);
            put_u32(h, *delta as u32);
            put_u32(h, skill.raw());
            put_u32(h, 0x4F41_4D4B);
        }

        EffectRequest::PlaceHandCardToExpansion { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x504C_4143);
        }

        EffectRequest::ExtraResolveCurrentTrick {
            player,
            card,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 payload shape
            // (existing 154 uses are 4x/2x/3x u32 with different salts/fields).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4558_5452);
        }

        EffectRequest::FinishPhaseEndVirtualCard => {
            put_u8(h, 153);
        }

        EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            skill,
        } => {
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::SkipNextPlayPhase { player } => {
            put_u8(h, 155);
            put_u32(h, player.raw());
        }

        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            skill,
        } => {
            put_u8(h, 156);
            put_u32(h, player.raw());
            put_u32(h, *draw_count);
            put_u32(h, *recover_count);
            put_u32(h, *times);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
            player,
            count,
            skill,
        } => {
            put_u8(h, 157);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawSelectedTargets {
            targets,
            count,
            skill,
        } => {
            put_u8(h, 158);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::RecoverIfMinCurrentHp { player, amount } => {
            put_u8(h, 159);
            put_u32(h, player.raw());
            put_u32(h, *amount);
        }

        EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            skill,
        } => {
            put_u8(h, 160);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            put_u8(h, 161);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            put_u8(h, 162);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        _ => unreachable!("effect routed to a different hash group"),
    }
}
