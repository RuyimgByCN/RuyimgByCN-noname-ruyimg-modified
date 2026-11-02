use super::*;

#[derive(Clone)]
pub struct DiscardToLimitMachine {
    pub(super) player: PlayerId,
    pub(super) need: usize,
    pub(super) asked: bool,
}

impl DiscardToLimitMachine {
    pub(crate) fn new(player: PlayerId, need: usize, asked: bool) -> Self {
        Self {
            player,
            need,
            asked,
        }
    }
}

impl crate::catalog::SkillMachine for DiscardToLimitMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::DiscardToLimit {
            player: self.player,
            need: self.need,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            // 沙漠绿洲（mtg 第六批，规则 v105）：基本牌不计上限——弃牌
            // 选项排除基本牌（不计入即不会被要求弃）
            let shamolvzhou = ctx.core.current_land == Some(crate::catalog::CardKind::Shamolvzhou);
            // 红莲（Boss 毕方，规则 v228）：弃牌选项排除红色牌（不计入
            // 上限即不会被要求弃）
            let honglianx = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_HONGLIANX)
                .map(|def| ctx.core.has_skill(self.player, def))
                .unwrap_or(false);
            let options: Vec<ChoiceOption> = ctx
                .core
                .player(self.player)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    if honglianx
                        && matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    {
                        return false;
                    }
                    !shamolvzhou
                        || crate::vocabulary::card_skills::yangpijuan_class(
                            ctx.catalog.card_kind(*c),
                        ) != Some(0)
                })
                .map(ChoiceOption::Card)
                .collect();
            return Flow::WaitChoice {
                observer: self.player,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "discard_to_limit",
                    options,
                    min: self.need,
                    max: self.need,
                    allow_cancel: false,
                },
            };
        }
        let answer = ctx.answer().clone();
        let mut discarded: Vec<crate::ids::CardId> = Vec::new();
        for opt in answer {
            if let ChoiceOption::Card(c) = opt {
                // 望尘（swd 古月圣，批㉑）：弃牌阶段弃置基本牌置旗
                if ctx.catalog.card_kind(c).is_basic_kind() {
                    ctx.core.turn.wangchen_discard_basic = true;
                }
                ctx.discard_from_hand(self.player, c);
                discarded.push(c);
            }
        }
        if !discarded.is_empty() {
            // 水没墓穴计数（mtg 第三批，规则 v102）：弃牌阶段内弃牌
            ctx.core.turn.discard_phase_discarded += discarded.len() as u32;
            // 拾忧（十殿阎罗孟婆，规则 v270）：其他角色弃牌阶段弃置的牌
            // ——持有者获得其中花色各不相同的牌（上游 chooseCardButton
            // 可选花色互异→锁定化恒发动+按弃牌序贪心取首个未见花色确定
            // 性近似登记；牌自弃牌堆拨入持有者手牌）
            if ctx.core.turn.phase == crate::event::Phase::Discard {
                if let Some(sydef) = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::names::BOSS_SHIYOU)
                {
                    let seats = ctx.core.players.len() as PlayerId;
                    let holders: Vec<PlayerId> = (0..seats)
                        .filter(|&h| {
                            h != self.player
                                && ctx.core.player(h).alive
                                && ctx.core.has_skill(h, sydef)
                        })
                        .collect();
                    if let Some(holder) = holders.first() {
                        let mut seen: Vec<crate::catalog::Suit> = Vec::new();
                        let mut gain: Vec<crate::ids::CardId> = Vec::new();
                        for c in &discarded {
                            let suit = ctx.catalog.cards[*c as usize].suit;
                            if !seen.contains(&suit) {
                                seen.push(suit);
                                gain.push(*c);
                            }
                        }
                        if !gain.is_empty() {
                            for c in &gain {
                                ctx.core.discard.retain(|x| x != c);
                            }
                            ctx.core
                                .player_mut(*holder)
                                .hand
                                .extend(gain.iter().copied());
                            ctx.log(LogEvent::CardsGivenPublic {
                                from: self.player,
                                to: *holder,
                                cards: gain,
                            });
                        }
                    }
                }
            }
            // 弃牌事件：连营类「失去手牌」触发依赖此事件（ADR 0763）
            ctx.emit(EventData::CardsDiscarded {
                player: self.player,
                cards: discarded,
            });
        }
        Flow::Done
    }
}
