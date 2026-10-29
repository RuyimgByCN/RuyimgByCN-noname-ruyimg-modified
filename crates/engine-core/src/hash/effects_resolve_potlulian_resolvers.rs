//! 效果请求的规范哈希编码：具名技能结算器（Resolve*）。

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

        EffectRequest::ResolveQiluan { player, skill } => {
            put_u8(h, 155);
            put_u32(h, player.raw());
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

        EffectRequest::ResolveTurnEndDebates { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5254_4439);
        }

        EffectRequest::ResolvePotheyun { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5054_4859);
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

        _ => unreachable!("effect routed to a different hash group"),
    }
}
