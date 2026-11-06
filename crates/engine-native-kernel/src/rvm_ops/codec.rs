//! VM 效果请求的字节编解码（入队前持久化与恢复）。

use super::*;

pub(super) fn push_u32(w: &mut Vec<u8>, v: u32) {
    w.extend_from_slice(&v.to_le_bytes());
}
pub(super) fn push_i32(w: &mut Vec<u8>, v: i32) {
    w.extend_from_slice(&v.to_le_bytes());
}

/// 排队效果编码。只覆盖 `accept_effect` 放行的变体。
pub fn enc_effects(w: &mut Vec<u8>, effects: &[EffectRequest]) {
    push_u32(w, effects.len() as u32);
    for e in effects {
        match e {
            EffectRequest::ChooseGiveOrRecastOneHand {
                player,
                skill,
                require_skill_def,
                grant_skill_def,
                grant_mark_def,
            } => {
                w.push(31);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
                push_u32(w, *require_skill_def);
                push_u32(w, *grant_skill_def);
                push_u32(w, *grant_mark_def);
            }
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
                player,
                amount,
                skill,
            } => {
                w.push(32);
                push_u32(w, player.raw());
                push_u32(w, *amount);
                push_u32(w, skill.raw());
            }
            EffectRequest::ResolveOlJieDanshou { player, skill } => {
                w.push(30);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            // R05-E（ABI 759）：三 compound 与胆守同形（player, skill）。
            EffectRequest::ResolveTiqi { player, skill } => {
                w.push(33);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::ResolveBaoshu { player, skill } => {
                w.push(34);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::ResolveDcbihuo { player, skill } => {
                w.push(35);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::RecoverHp {
                source,
                player,
                amount,
            } => {
                w.push(1);
                push_u32(w, source.raw());
                push_u32(w, player.raw());
                push_u32(w, *amount);
            }
            EffectRequest::DrawCards { player, count } => {
                w.push(2);
                push_u32(w, player.raw());
                push_u32(w, *count);
            }
            EffectRequest::DealDamage {
                source,
                target,
                amount,
                is_lose_hp,
                nature,
            } => {
                w.push(3);
                push_u32(w, source.raw());
                push_u32(w, target.raw());
                push_u32(w, *amount);
                w.push(u8::from(*is_lose_hp));
                w.push(*nature);
            }
            EffectRequest::ChangeMaxHp { player, delta } => {
                w.push(4);
                push_u32(w, player.raw());
                push_i32(w, *delta);
            }
            EffectRequest::AddMark {
                player,
                mark,
                delta,
            } => {
                w.push(5);
                push_u32(w, player.raw());
                push_u32(w, mark.raw());
                push_i32(w, *delta);
            }
            EffectRequest::RemoveMark {
                player,
                mark,
                amount,
            } => {
                w.push(6);
                push_u32(w, player.raw());
                push_u32(w, mark.raw());
                push_u32(w, *amount);
            }
            EffectRequest::DiscardFromHand { player, count } => {
                w.push(7);
                push_u32(w, player.raw());
                push_u32(w, *count);
            }
            EffectRequest::DiscardFromHandEquip { player, count } => {
                w.push(8);
                push_u32(w, player.raw());
                push_u32(w, *count);
            }
            EffectRequest::AddSkill { player, skill_def } => {
                w.push(9);
                push_u32(w, player.raw());
                push_u32(w, skill_def.raw());
            }
            EffectRequest::RemoveSkill { skill } => {
                w.push(10);
                push_u32(w, skill.raw());
            }
            EffectRequest::GainTopCardToSkillPile { player, skill } => {
                w.push(11);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::GainWholeSkillPile {
                player,
                skill,
                pile_def,
            } => {
                w.push(12);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
                push_u32(w, *pile_def);
            }
            EffectRequest::GainAllSkillPileCards { player, skill } => {
                w.push(13);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::GainFromSkillPile { player, skill } => {
                w.push(14);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::DiscardSkillPile {
                player,
                skill,
                pile_def,
            } => {
                w.push(15);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
                push_u32(w, *pile_def);
            }
            EffectRequest::StoreEventCardInSkillPile {
                player,
                card,
                skill,
            } => {
                w.push(16);
                push_u32(w, player.raw());
                push_u32(w, card.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::PlaceTopDeckToExpansion {
                player,
                skill,
                pile_def,
                count,
            } => {
                w.push(17);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
                push_u32(w, *pile_def);
                push_u32(w, *count);
            }
            EffectRequest::EnterDying { player } => {
                w.push(18);
                push_u32(w, player.raw());
            }
            EffectRequest::KillPlayer { player, killer } => {
                w.push(19);
                push_u32(w, player.raw());
                push_u32(w, killer.raw());
            }
            EffectRequest::MoveCards { cards, to, from } => {
                w.push(20);
                push_u32(w, cards.len() as u32);
                for c in cards {
                    push_u32(w, c.raw());
                }
                push_u32(w, to.raw());
                match from {
                    Some(z) => {
                        w.push(1);
                        push_u32(w, z.raw());
                    }
                    None => w.push(0),
                }
            }
            EffectRequest::RevealCards { cards, to } => {
                w.push(21);
                push_u32(w, cards.len() as u32);
                for c in cards {
                    push_u32(w, c.raw());
                }
                push_u32(w, to.len() as u32);
                for p in to {
                    push_u32(w, p.raw());
                }
            }
            EffectRequest::SetTurnState { current, phase } => {
                w.push(22);
                push_u32(w, current.raw());
                w.push(*phase);
            }
            EffectRequest::TakeFromHand { from, to } => {
                w.push(23);
                push_u32(w, from.raw());
                push_u32(w, to.raw());
            }
            EffectRequest::DiscardAllOwnJudge { player, skill } => {
                w.push(24);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::GainOtherSkillPileCards {
                player,
                target,
                skill_def,
                skill,
            } => {
                w.push(25);
                push_u32(w, player.raw());
                push_u32(w, target.raw());
                push_u32(w, *skill_def);
                push_u32(w, skill.raw());
            }
            // J1（规则 v328）：提交期挂起需要把开窗效果编码入档（此前该
            // 变体从未在挂起队列中出现，编码面未覆盖）。
            EffectRequest::DiscardHandEquipCount {
                player,
                count,
                skill,
            } => {
                w.push(26);
                push_u32(w, player.raw());
                push_u32(w, *count);
                push_u32(w, skill.raw());
            }
            // PA12 批⑦：补齐 accept_effect 已放行但编码面遗漏的五种变体
            //（TurnOver / PreventDamage / ModifyCurrentDamage / SkipCurrentPhase / StorageSet），
            // 解锁开窗或挂起期间队列含此类效果的真实技能（如 mingshi 等）。
            EffectRequest::TurnOver { player } => {
                w.push(27);
                push_u32(w, player.raw());
            }
            EffectRequest::PreventDamage { damage_id } => {
                w.push(28);
                push_u32(w, *damage_id);
            }
            EffectRequest::ModifyCurrentDamage {
                player,
                delta,
                skill,
            } => {
                w.push(29);
                push_u32(w, player.raw());
                push_i32(w, *delta);
                push_u32(w, skill.raw());
            }
            EffectRequest::SkipCurrentPhase { player } => {
                w.push(36);
                push_u32(w, player.raw());
            }
            EffectRequest::StorageSet { skill, key, value } => {
                w.push(37);
                push_u32(w, skill.raw());
                push_u32(w, *key);
                push_u32(w, *value);
            }
            // PA12 批⑧：10 种确定性非挂起效果编码（38..=48）。
            EffectRequest::AddSkillUntilTurnEnd { player, skill_def } => {
                w.push(38);
                push_u32(w, player.raw());
                push_u32(w, skill_def.raw());
            }
            EffectRequest::RemoveSkillByDef { player, skill_def } => {
                w.push(39);
                push_u32(w, player.raw());
                push_u32(w, skill_def.raw());
            }
            EffectRequest::RevealHand { player } => {
                w.push(40);
                push_u32(w, player.raw());
            }
            EffectRequest::RevealHandToPlayer { player, observer } => {
                w.push(41);
                push_u32(w, player.raw());
                push_u32(w, observer.raw());
            }
            EffectRequest::GainCardByKind { player, kind } => {
                w.push(42);
                push_u32(w, player.raw());
                w.push(*kind);
            }
            EffectRequest::GainDiscardCardByKind { player, kind } => {
                w.push(43);
                push_u32(w, player.raw());
                w.push(*kind);
            }
            EffectRequest::GainEquipmentFromDrawPile { player } => {
                w.push(44);
                push_u32(w, player.raw());
            }
            EffectRequest::GiveAllHand { player, target } => {
                w.push(45);
                push_u32(w, player.raw());
                push_u32(w, target.raw());
            }
            EffectRequest::DiscardAllTargetEquipment { actor, target } => {
                w.push(46);
                push_u32(w, actor.raw());
                push_u32(w, target.raw());
            }
            EffectRequest::GainEventCards { player, cards } => {
                w.push(47);
                push_u32(w, player.raw());
                push_u32(w, cards.len() as u32);
                for c in cards {
                    push_u32(w, c.raw());
                }
            }
            EffectRequest::GainCardIfDiscard { player, card } => {
                w.push(48);
                push_u32(w, player.raw());
                push_u32(w, card.raw());
            }
            // PA12 批⑨：9 种回合/摸牌/装备/杀次数确定性效果编码（49..=57）。
            EffectRequest::IncreaseCurrentDrawCount { player, count } => {
                w.push(49);
                push_u32(w, player.raw());
                push_u32(w, *count);
            }
            EffectRequest::SkipNextPlayPhase { player } => {
                w.push(50);
                push_u32(w, player.raw());
            }
            EffectRequest::SkipNextDiscardPhase { player } => {
                w.push(51);
                push_u32(w, player.raw());
            }
            EffectRequest::SkipNextJudgePhase { player } => {
                w.push(52);
                push_u32(w, player.raw());
            }
            EffectRequest::SkipJudgeAndDrawPhases { player, skill } => {
                w.push(53);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::SkipRemainingPhases {
                player,
                phases_mask,
                skill,
            } => {
                w.push(54);
                push_u32(w, player.raw());
                push_u32(w, *phases_mask);
                push_u32(w, skill.raw());
            }
            EffectRequest::QueueExtraPhases {
                player,
                phases_mask,
                skill,
            } => {
                w.push(55);
                push_u32(w, player.raw());
                push_u32(w, *phases_mask);
                push_u32(w, skill.raw());
            }
            EffectRequest::GainTargetEquipment {
                player,
                target,
                slot,
            } => {
                w.push(56);
                push_u32(w, player.raw());
                push_u32(w, target.raw());
                w.push(*slot);
            }
            EffectRequest::RefundCurrentShaUse {
                player,
                card,
                skill,
            } => {
                w.push(57);
                push_u32(w, player.raw());
                push_u32(w, card.raw());
                push_u32(w, skill.raw());
            }
            // PA12 批⑩：13 种区域整弃/复原/清技/连环摸/当先/装备/移出/杀修正确定性效果编码（58..=70）。
            EffectRequest::DiscardAllHandAndEquip { player } => {
                w.push(58);
                push_u32(w, player.raw());
            }
            EffectRequest::DiscardAllHandEquipJudge { player } => {
                w.push(59);
                push_u32(w, player.raw());
            }
            EffectRequest::RestoreCharacterState { player } => {
                w.push(60);
                push_u32(w, player.raw());
            }
            EffectRequest::RemoveAllSkills { player } => {
                w.push(61);
                push_u32(w, player.raw());
            }
            EffectRequest::DrawLinkedPlayers { count } => {
                w.push(62);
                push_u32(w, *count);
            }
            EffectRequest::RecoverIfMinCurrentHp { player, amount } => {
                w.push(63);
                push_u32(w, player.raw());
                push_u32(w, *amount);
            }
            EffectRequest::EquipWeaponFromDrawPile { player } => {
                w.push(64);
                push_u32(w, player.raw());
            }
            EffectRequest::RemoveCardsByKind { kind } => {
                w.push(65);
                w.push(*kind);
            }
            EffectRequest::QueueOpeningExtraPlayPhase { player, skill } => {
                w.push(66);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::IncreaseCurrentShaBaseDamage {
                player,
                card,
                amount,
                skill,
            } => {
                w.push(67);
                push_u32(w, player.raw());
                push_u32(w, card.raw());
                push_u32(w, *amount);
                push_u32(w, skill.raw());
            }
            EffectRequest::IncreaseCurrentCardBaseDamage {
                player,
                card,
                amount,
                skill,
            } => {
                w.push(68);
                push_u32(w, player.raw());
                push_u32(w, card.raw());
                push_u32(w, *amount);
                push_u32(w, skill.raw());
            }
            EffectRequest::IncreaseCurrentShaTargetDamage {
                source,
                card,
                target,
                amount,
            } => {
                w.push(69);
                push_u32(w, source.raw());
                push_u32(w, card.raw());
                push_u32(w, target.raw());
                push_u32(w, *amount);
            }
            EffectRequest::MakeCurrentShaTargetUnrespondable {
                source,
                card,
                target,
            } => {
                w.push(70);
                push_u32(w, source.raw());
                push_u32(w, card.raw());
                push_u32(w, target.raw());
            }
            // PA12 批⑪：13 种手牌调整/底牌摸/目标排除/取牌/禁用抑制确定性效果编码（71..=83）。
            EffectRequest::AdjustHandTo { player, count } => {
                w.push(71);
                push_u32(w, player.raw());
                push_u32(w, *count);
            }
            EffectRequest::DrawByPlayPhaseCardCategoryCount { player, skill } => {
                w.push(72);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::DrawEachBySeat {
                first,
                second,
                count,
                skill,
            } => {
                w.push(73);
                push_u32(w, first.raw());
                push_u32(w, second.raw());
                push_u32(w, *count);
                push_u32(w, skill.raw());
            }
            EffectRequest::SetDrawPhaseFromBottom { player } => {
                w.push(74);
                push_u32(w, player.raw());
            }
            EffectRequest::DrawCardsFromBottom { player, count } => {
                w.push(75);
                push_u32(w, player.raw());
                push_u32(w, *count);
            }
            EffectRequest::CancelIncomingCardUse {
                player,
                source,
                card,
                skill,
            } => {
                w.push(76);
                push_u32(w, player.raw());
                push_u32(w, source.raw());
                push_u32(w, card.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::ExcludeSavedShaTarget { card, target } => {
                w.push(77);
                push_u32(w, card.raw());
                push_u32(w, target.raw());
            }
            EffectRequest::TakeOneCard {
                from,
                to,
                equip_only,
            } => {
                w.push(78);
                push_u32(w, from.raw());
                push_u32(w, to.raw());
                w.push(u8::from(*equip_only));
            }
            EffectRequest::TakeOneHandCard { from, to } => {
                w.push(79);
                push_u32(w, from.raw());
                push_u32(w, to.raw());
            }
            EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def } => {
                w.push(80);
                push_u32(w, player.raw());
                push_u32(w, skill_def.raw());
            }
            EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
                w.push(81);
                push_u32(w, player.raw());
                push_u32(w, skill.raw());
            }
            EffectRequest::SetNonLockedSkillSuppression {
                player,
                enabled,
                skill,
            } => {
                w.push(82);
                push_u32(w, player.raw());
                w.push(u8::from(*enabled));
                push_u32(w, skill.raw());
            }
            EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player } => {
                w.push(83);
                push_u32(w, player.raw());
            }
            _ => unreachable!("accept_effect 已拒绝未编码变体"),
        }
    }
}

pub(super) struct Rd<'a> {
    r: &'a [u8],
}
impl Rd<'_> {
    fn u32(&mut self) -> Option<u32> {
        if self.r.len() < 4 {
            return None;
        }
        let (a, rest) = self.r.split_at(4);
        self.r = rest;
        Some(u32::from_le_bytes(a.try_into().ok()?))
    }
    fn i32(&mut self) -> Option<i32> {
        self.u32().map(|v| v as i32)
    }
    fn u8(&mut self) -> Option<u8> {
        let v = *self.r.first()?;
        self.r = &self.r[1..];
        Some(v)
    }
}

/// 与 `enc_effects` 对偶。未知标签或尾部残留返回 None。
pub fn dec_effects(bytes: &[u8]) -> Option<Vec<EffectRequest>> {
    let mut rd = Rd { r: bytes };
    let n = rd.u32()? as usize;
    let mut out = Vec::new();
    for _ in 0..n {
        let tag = rd.u8()?;
        let e = match tag {
            31 => EffectRequest::ChooseGiveOrRecastOneHand {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
                require_skill_def: rd.u32()?,
                grant_skill_def: rd.u32()?,
                grant_mark_def: rd.u32()?,
            },
            32 => EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
                player: rule_api::PlayerId::new(rd.u32()?),
                amount: rd.u32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            30 => EffectRequest::ResolveOlJieDanshou {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            33 => EffectRequest::ResolveTiqi {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            34 => EffectRequest::ResolveBaoshu {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            35 => EffectRequest::ResolveDcbihuo {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            1 => EffectRequest::RecoverHp {
                source: rule_api::PlayerId::new(rd.u32()?),
                player: rule_api::PlayerId::new(rd.u32()?),
                amount: rd.u32()?,
            },
            2 => EffectRequest::DrawCards {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
            },
            3 => EffectRequest::DealDamage {
                source: rule_api::PlayerId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
                amount: rd.u32()?,
                is_lose_hp: rd.u8()? == 1,
                nature: rd.u8()?,
            },
            4 => EffectRequest::ChangeMaxHp {
                player: rule_api::PlayerId::new(rd.u32()?),
                delta: rd.i32()?,
            },
            5 => EffectRequest::AddMark {
                player: rule_api::PlayerId::new(rd.u32()?),
                mark: rule_api::MarkDefId::new(rd.u32()?),
                delta: rd.i32()?,
            },
            6 => EffectRequest::RemoveMark {
                player: rule_api::PlayerId::new(rd.u32()?),
                mark: rule_api::MarkDefId::new(rd.u32()?),
                amount: rd.u32()?,
            },
            7 => EffectRequest::DiscardFromHand {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
            },
            8 => EffectRequest::DiscardFromHandEquip {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
            },
            9 => EffectRequest::AddSkill {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill_def: rule_api::SkillDefId::new(rd.u32()?),
            },
            10 => EffectRequest::RemoveSkill {
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            11 => EffectRequest::GainTopCardToSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            12 => EffectRequest::GainWholeSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
                pile_def: rd.u32()?,
            },
            13 => EffectRequest::GainAllSkillPileCards {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            14 => EffectRequest::GainFromSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            15 => EffectRequest::DiscardSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
                pile_def: rd.u32()?,
            },
            16 => EffectRequest::StoreEventCardInSkillPile {
                player: rule_api::PlayerId::new(rd.u32()?),
                card: rule_api::CardId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            17 => EffectRequest::PlaceTopDeckToExpansion {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
                pile_def: rd.u32()?,
                count: rd.u32()?,
            },
            18 => EffectRequest::EnterDying {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            19 => EffectRequest::KillPlayer {
                player: rule_api::PlayerId::new(rd.u32()?),
                killer: rule_api::PlayerId::new(rd.u32()?),
            },
            20 => {
                let n = rd.u32()? as usize;
                let mut cards = Vec::with_capacity(n);
                for _ in 0..n {
                    cards.push(rule_api::CardId::new(rd.u32()?));
                }
                let to = rule_api::ZoneId::new(rd.u32()?);
                let from = if rd.u8()? == 1 {
                    Some(rule_api::ZoneId::new(rd.u32()?))
                } else {
                    None
                };
                EffectRequest::MoveCards { cards, to, from }
            }
            21 => {
                let n = rd.u32()? as usize;
                let mut cards = Vec::with_capacity(n);
                for _ in 0..n {
                    cards.push(rule_api::CardId::new(rd.u32()?));
                }
                let m = rd.u32()? as usize;
                let mut to = Vec::with_capacity(m);
                for _ in 0..m {
                    to.push(rule_api::PlayerId::new(rd.u32()?));
                }
                EffectRequest::RevealCards { cards, to }
            }
            22 => EffectRequest::SetTurnState {
                current: rule_api::PlayerId::new(rd.u32()?),
                phase: rd.u8()?,
            },
            23 => EffectRequest::TakeFromHand {
                from: rule_api::PlayerId::new(rd.u32()?),
                to: rule_api::PlayerId::new(rd.u32()?),
            },
            24 => EffectRequest::DiscardAllOwnJudge {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            25 => EffectRequest::GainOtherSkillPileCards {
                player: rule_api::PlayerId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
                skill_def: rd.u32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            // J1（规则 v328）：提交期挂起队列可含开窗效果（编码臂 26）。
            26 => EffectRequest::DiscardHandEquipCount {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            27 => EffectRequest::TurnOver {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            28 => EffectRequest::PreventDamage {
                damage_id: rd.u32()?,
            },
            29 => EffectRequest::ModifyCurrentDamage {
                player: rule_api::PlayerId::new(rd.u32()?),
                delta: rd.i32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            36 => EffectRequest::SkipCurrentPhase {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            37 => EffectRequest::StorageSet {
                skill: rule_api::SkillId::new(rd.u32()?),
                key: rd.u32()?,
                value: rd.u32()?,
            },
            38 => EffectRequest::AddSkillUntilTurnEnd {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill_def: rule_api::SkillDefId::new(rd.u32()?),
            },
            39 => EffectRequest::RemoveSkillByDef {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill_def: rule_api::SkillDefId::new(rd.u32()?),
            },
            40 => EffectRequest::RevealHand {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            41 => EffectRequest::RevealHandToPlayer {
                player: rule_api::PlayerId::new(rd.u32()?),
                observer: rule_api::PlayerId::new(rd.u32()?),
            },
            42 => EffectRequest::GainCardByKind {
                player: rule_api::PlayerId::new(rd.u32()?),
                kind: rd.u8()?,
            },
            43 => EffectRequest::GainDiscardCardByKind {
                player: rule_api::PlayerId::new(rd.u32()?),
                kind: rd.u8()?,
            },
            44 => EffectRequest::GainEquipmentFromDrawPile {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            45 => EffectRequest::GiveAllHand {
                player: rule_api::PlayerId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
            },
            46 => EffectRequest::DiscardAllTargetEquipment {
                actor: rule_api::PlayerId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
            },
            47 => {
                let player = rule_api::PlayerId::new(rd.u32()?);
                let m = rd.u32()? as usize;
                let mut cards = Vec::with_capacity(m);
                for _ in 0..m {
                    cards.push(rule_api::CardId::new(rd.u32()?));
                }
                EffectRequest::GainEventCards { player, cards }
            }
            48 => EffectRequest::GainCardIfDiscard {
                player: rule_api::PlayerId::new(rd.u32()?),
                card: rule_api::CardId::new(rd.u32()?),
            },
            49 => EffectRequest::IncreaseCurrentDrawCount {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
            },
            50 => EffectRequest::SkipNextPlayPhase {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            51 => EffectRequest::SkipNextDiscardPhase {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            52 => EffectRequest::SkipNextJudgePhase {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            53 => EffectRequest::SkipJudgeAndDrawPhases {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            54 => EffectRequest::SkipRemainingPhases {
                player: rule_api::PlayerId::new(rd.u32()?),
                phases_mask: rd.u32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            55 => EffectRequest::QueueExtraPhases {
                player: rule_api::PlayerId::new(rd.u32()?),
                phases_mask: rd.u32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            56 => EffectRequest::GainTargetEquipment {
                player: rule_api::PlayerId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
                slot: rd.u8()?,
            },
            57 => EffectRequest::RefundCurrentShaUse {
                player: rule_api::PlayerId::new(rd.u32()?),
                card: rule_api::CardId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            58 => EffectRequest::DiscardAllHandAndEquip {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            59 => EffectRequest::DiscardAllHandEquipJudge {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            60 => EffectRequest::RestoreCharacterState {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            61 => EffectRequest::RemoveAllSkills {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            62 => EffectRequest::DrawLinkedPlayers { count: rd.u32()? },
            63 => EffectRequest::RecoverIfMinCurrentHp {
                player: rule_api::PlayerId::new(rd.u32()?),
                amount: rd.u32()?,
            },
            64 => EffectRequest::EquipWeaponFromDrawPile {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            65 => EffectRequest::RemoveCardsByKind { kind: rd.u8()? },
            66 => EffectRequest::QueueOpeningExtraPlayPhase {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            67 => EffectRequest::IncreaseCurrentShaBaseDamage {
                player: rule_api::PlayerId::new(rd.u32()?),
                card: rule_api::CardId::new(rd.u32()?),
                amount: rd.u32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            68 => EffectRequest::IncreaseCurrentCardBaseDamage {
                player: rule_api::PlayerId::new(rd.u32()?),
                card: rule_api::CardId::new(rd.u32()?),
                amount: rd.u32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            69 => EffectRequest::IncreaseCurrentShaTargetDamage {
                source: rule_api::PlayerId::new(rd.u32()?),
                card: rule_api::CardId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
                amount: rd.u32()?,
            },
            70 => EffectRequest::MakeCurrentShaTargetUnrespondable {
                source: rule_api::PlayerId::new(rd.u32()?),
                card: rule_api::CardId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
            },
            71 => EffectRequest::AdjustHandTo {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
            },
            72 => EffectRequest::DrawByPlayPhaseCardCategoryCount {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            73 => EffectRequest::DrawEachBySeat {
                first: rule_api::PlayerId::new(rd.u32()?),
                second: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            74 => EffectRequest::SetDrawPhaseFromBottom {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            75 => EffectRequest::DrawCardsFromBottom {
                player: rule_api::PlayerId::new(rd.u32()?),
                count: rd.u32()?,
            },
            76 => EffectRequest::CancelIncomingCardUse {
                player: rule_api::PlayerId::new(rd.u32()?),
                source: rule_api::PlayerId::new(rd.u32()?),
                card: rule_api::CardId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            77 => EffectRequest::ExcludeSavedShaTarget {
                card: rule_api::CardId::new(rd.u32()?),
                target: rule_api::PlayerId::new(rd.u32()?),
            },
            78 => EffectRequest::TakeOneCard {
                from: rule_api::PlayerId::new(rd.u32()?),
                to: rule_api::PlayerId::new(rd.u32()?),
                equip_only: rd.u8()? != 0,
            },
            79 => EffectRequest::TakeOneHandCard {
                from: rule_api::PlayerId::new(rd.u32()?),
                to: rule_api::PlayerId::new(rd.u32()?),
            },
            80 => EffectRequest::DisableSkillByDefUntilTurnEnd {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill_def: rule_api::SkillDefId::new(rd.u32()?),
            },
            81 => EffectRequest::BanActiveUntilTurnEnd {
                player: rule_api::PlayerId::new(rd.u32()?),
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            82 => EffectRequest::SetNonLockedSkillSuppression {
                player: rule_api::PlayerId::new(rd.u32()?),
                enabled: rd.u8()? != 0,
                skill: rule_api::SkillId::new(rd.u32()?),
            },
            83 => EffectRequest::SuppressNonLockedSkillsUntilTurnEnd {
                player: rule_api::PlayerId::new(rd.u32()?),
            },
            _ => return None,
        };
        out.push(e);
    }
    if rd.r.is_empty() {
        Some(out)
    } else {
        None
    }
}
