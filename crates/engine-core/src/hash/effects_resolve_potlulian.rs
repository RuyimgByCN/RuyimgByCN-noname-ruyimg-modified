//! 效果请求的规范哈希编码：通用效果原语。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
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
