//! Canonical encoding for `ChooseGainDiscardCardByKind` and related requests.

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ChooseGainDiscardCardByKind {
            player,
            kind,
            skill,
        } => {
            put_u8(h, 132);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawForLifetimeCardActionDivisors { player, divisors } => {
            put_u8(h, 133);
            put_u32(h, player.raw());
            for divisor in divisors {
                put_u32(h, *divisor);
            }
        }

        EffectRequest::ChooseAddMissingSkill {
            player,
            skill_defs,
            skill,
        } => {
            put_u8(h, 134);
            put_u32(h, player.raw());
            for skill_def in skill_defs {
                put_u32(h, skill_def.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::AddFirstMissingSkill {
            player,
            skill_defs,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&530u16.to_le_bytes());
            put_u32(h, player.raw());
            for skill_def in skill_defs {
                put_u32(h, skill_def.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            skill,
        } => {
            put_u8(h, 135);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u8(h, *target_count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }

        EffectRequest::GainEventCards { player, cards } => {
            put_u8(h, 136);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
        }

        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, skill } => {
            put_u8(h, 137);
            put_u32(h, player.raw());
            put_u32(h, *cap);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, skill } => {
            put_u8(h, 138);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::EmitJudgmentResolved {
            player,
            result_card,
            suit,
            rank,
            cause_card,
        } => {
            put_u8(h, 139);
            put_u32(h, player.raw());
            put_u32(h, result_card.raw());
            put_u8(h, suit_tag(*suit));
            put_u8(h, *rank);
            put_u32(h, cause_card.raw());
        }

        EffectRequest::JudgeRecoverOtherOnSuit {
            source,
            judge_player,
            recover_player,
            suit,
            amount,
        } => {
            put_u8(h, 140);
            put_u32(h, source.raw());
            put_u32(h, judge_player.raw());
            put_u32(h, recover_player.raw());
            put_u8(h, suit_tag(*suit));
            put_u32(h, *amount);
        }

        EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
            player,
            remaining,
            started,
            skill,
        } => {
            put_u8(h, 141);
            put_u32(h, player.raw());
            put_ids(h, remaining.iter().map(|player| player.raw()));
            put_bool(h, *started);
            put_u32(h, skill.raw());
        }

        EffectRequest::ContinueDyingSave {
            player,
            source,
            responders,
        } => {
            put_u8(h, 142);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_ids(h, responders.iter().map(|player| player.raw()));
        }

        EffectRequest::RestoreCharacterState { player } => {
            put_u8(h, 143);
            put_u32(h, player.raw());
        }

        EffectRequest::DiscardAllHandEquipJudge { player } => {
            put_u8(h, 144);
            put_u32(h, player.raw());
        }

        EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
            source,
            card,
            target,
        } => {
            put_u8(h, 145);
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::UseVirtualCardOnSelectedTargets {
            player,
            kind,
            targets,
        } => {
            put_u8(h, 146);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_ids(h, targets.iter().map(|target| target.raw()));
        }

        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            skill,
        } => {
            put_u8(h, 147);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::SpjungongDiscardOrLoseThenSha {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&363u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
            put_u8(h, 0);
            h.update(&364u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::SpwanweiActive { player, target } => {
            put_u8(h, 0);
            h.update(&365u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::SpwanweiDying { player, target } => {
            put_u8(h, 0);
            h.update(&366u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player } => {
            put_u8(h, 0);
            h.update(&367u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::ChooseRecoverTarget {
            chooser,
            target,
            amount,
        } => {
            put_u8(h, 0);
            h.update(&368u16.to_le_bytes());
            put_u32(h, chooser.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
        }

        EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
            player,
            target,
            cards,
            targets,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&369u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&370u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
            player,
            card,
            targets,
            stage,
            damage_bonus,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&371u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, *damage_bonus);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpxizhan {
            player,
            phase_player,
            fangzong_skill_def,
        } => {
            put_u8(h, 0);
            h.update(&372u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, phase_player.raw());
            put_u32(h, fangzong_skill_def.raw());
        }

        EffectRequest::ResolveSpsongshu {
            player,
            phase_player,
            renku_skill_def,
            block_skill_def,
        } => {
            put_u8(h, 0);
            h.update(&384u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, phase_player.raw());
            put_u32(h, renku_skill_def.raw());
            put_u32(h, block_skill_def.raw());
        }

        EffectRequest::ResolveSpcunsi { player, target } => {
            put_u8(h, 0);
            h.update(&390u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::ResolveYuli {
            player,
            mode,
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
            h.update(&409u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTingwei {
            player,
            mode,
            card,
            targets,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&410u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|id| id.raw()));
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinmingshi {
            player,
            source,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&411u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXunyi {
            player,
            mode,
            source,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&412u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveBoming {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&413u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveEjian {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&414u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveYiyong {
            player,
            card,
            source,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&415u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShanxie { player, skill } => {
            put_u8(h, 0);
            h.update(&416u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSheyi {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&417u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTianyin { player, skill } => {
            put_u8(h, 0);
            h.update(&418u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDbzhuifeng { player, skill } => {
            put_u8(h, 0);
            h.update(&419u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDbquedi {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&420u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpyinju {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&421u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpchijie {
            player,
            source,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&422u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpshangyi {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&423u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpjianyi { player, skill } => {
            put_u8(h, 0);
            h.update(&424u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveFyjianyu {
            player,
            source,
            target,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&425u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_ids(h, targets.iter().map(|id| id.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLiubing {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&426u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHeji {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&427u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpyajun {
            player,
            less_def,
            card,
            target,
            target_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&428u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, less_def.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, target_card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpzundi {
            player,
            target,
            card,
            destination,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&429u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, destination.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinyangjie {
            player,
            target,
            card,
            target_card,
            source,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&430u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, source.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQingyu {
            player,
            action,
            xuancun_def,
            mbyongjue_def,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&431u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, u32::from(*action));
            put_u32(h, xuancun_def.raw());
            put_u32(h, mbyongjue_def.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhengsu {
            player,
            mode,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&432u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, u32::from(*mode));
            put_u32(h, u32::from(*stage));
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHoufeng {
            player,
            target,
            mode,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&433u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*mode));
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbdiancai {
            player,
            actor,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&434u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRechuhai {
            player,
            target,
            card,
            target_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&435u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRechuhai { .. } => {
            // 其他 agent 在建批次：落盘前暂以桩哈希。
            put_u8(h, 0);
            h.update(&999u16.to_le_bytes());
        }

        EffectRequest::ResolveXiongshu {
            player,
            target,
            card,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&436u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*action));
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXiongshu { .. } => {
            put_u8(h, 0);
            h.update(&997u16.to_le_bytes());
        }

        EffectRequest::ResolveJianhui {
            player,
            source,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&437u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpqiai {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&439u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpshanxi {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&440u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpshanxi { .. } => {
            put_u8(h, 0);
            h.update(&996u16.to_le_bytes());
        }

        EffectRequest::ResolveSpshanxiSuoming {
            player,
            actor,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&441u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpshanxiSuoming { .. } => {
            put_u8(h, 0);
            h.update(&995u16.to_le_bytes());
        }

        EffectRequest::ResolveTongduo {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&442u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveYuanqing { player, skill } => {
            put_u8(h, 0);
            h.update(&443u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShuchen {
            player,
            dying,
            renku_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&444u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, renku_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDuanbi {
            player,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&445u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
            for target in targets {
                put_u32(h, target.raw());
            }
        }

        EffectRequest::ResolveZaoli {
            player,
            mark,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&446u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *mark);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGuying {
            player,
            actor,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&447u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbzhonghui {
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&545u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, *amount);
            put_u32(h, mbsizi_def.raw());
            put_u32(h, mbkechang_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotjiejie {
            player,
            actor,
            used_def,
            suit_defs,
            qingshi_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&516u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, used_def.raw());
            for def in suit_defs {
                put_u32(h, def.raw());
            }
            put_u32(h, qingshi_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotqingshi { player, skill } => {
            put_u8(h, 0);
            h.update(&517u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&546u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotzhouyu {
            player,
            target,
            card,
            action,
            card_flags,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&547u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u8(h, *card_flags);
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotlvyi {
            player,
            target,
            card,
            action,
            event_cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&548u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, event_cards.len() as u32);
            for card in event_cards {
                put_u32(h, card.raw());
            }
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotcaoshuang {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&549u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotyuji {
            player,
            target,
            card,
            action,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&550u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, cards.len() as u32);
            for card in cards {
                put_u32(h, card.raw());
            }
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRehengjiang {
            player,
            actor,
            action,
            hand_limit_def,
            draw_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&551u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *action);
            put_u32(h, hand_limit_def.raw());
            put_u32(h, draw_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTuntian { player, skill } => {
            put_u8(h, 0);
            h.update(&503u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZaoxian {
            player,
            jixi_def,
            tuntian_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&504u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, jixi_def.raw());
            put_u32(h, tuntian_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTiaoxin {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&505u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhiji {
            player,
            guanxing_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&506u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, guanxing_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZaiqi {
            player,
            extra,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&507u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *extra);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwqiushou { player, skill } => {
            put_u8(h, 0);
            h.update(&508u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhengrong {
            player,
            candidates,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&509u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, candidates.iter().map(|candidate| candidate.raw()));
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHongju {
            player,
            qingjian_def,
            rong_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&510u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, qingjian_def.raw());
            put_u32(h, rong_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQingjian {
            player,
            rong_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&511u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, rong_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveReleiji {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&512u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHuangtian {
            player,
            actor,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&513u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHaoshi {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&514u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDimeng {
            player,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&515u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQiaobian {
            player,
            phase,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&516u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *phase);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ExcludeSavedShaTarget { card, target } => {
            put_u8(h, 0);
            h.update(&517u16.to_le_bytes());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::ResolveXiangle {
            player,
            source,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&518u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveFangquanSkip { player, skill } => {
            put_u8(h, 0);
            h.update(&519u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveFangquanGrant {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&520u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRuoyu {
            player,
            jijiang_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&521u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, jijiang_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZuilun {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&522u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveFuyin {
            player,
            source,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&523u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::BanUseAgainst { user, target } => {
            put_u8(h, 0);
            h.update(&524u16.to_le_bytes());
            put_u32(h, user.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::ResolveJuzhan {
            player,
            source,
            card,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&525u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQizhi {
            player,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&526u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJinqu {
            player,
            qizhi_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&527u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, qizhi_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJianxiang {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&528u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShenshi {
            player,
            source,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&529u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMingren {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&530u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhenliang {
            player,
            source,
            card,
            mingren_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&531u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, mingren_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveKuizhu {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&532u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhizheng {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&533u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLijun {
            player,
            source,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&534u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLiangyin {
            player,
            kongsheng_def,
            stage,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&535u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, kongsheng_def.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveKongsheng {
            player,
            stage,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&536u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&537u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, guixiu_def.raw());
            put_u32(h, yongjue_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::RecordCurrentShaAllTargets {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&538u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGiveOrDiscardHand {
            player,
            payment_count,
            scope,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&570u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *payment_count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }
        _ => unreachable!("effect routed to a different hash group"),
    }
}
