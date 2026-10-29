use super::*;

// ---------------------------------------------------------------------------
// 星夏：出牌阶段主动技——对场上焰灵造成 2 点火焰伤害；随后可选一名敌
// 方：其弃一张红色牌（手牌+装备）或受 1 点火焰伤害（可放弃追击）。
// 间隔 ≥2 轮。祝融（规则 v242）。
// ---------------------------------------------------------------------------

/// 星夏阶段码（0=损焰灵已结算、1=敌方询问挂起、2=敌方已选、3=弃牌已答
/// 待结算——弃红询问由阶段 2 直接挂出，不经独立码）。
const XINGXIA_STAGE_DAMAGE: u8 = 0;
const XINGXIA_STAGE_ASK_ENEMY: u8 = 1;
const XINGXIA_STAGE_ENEMY_DONE: u8 = 2;
const XINGXIA_STAGE_DISCARD_DONE: u8 = 3;

pub(in super::super) struct XingxiaBehavior;

impl SkillBehavior for XingxiaBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }
    fn start(&self, _owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        unreachable!("星夏为主动技，不经触发总线")
    }
    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let used = ctx.state.players[owner as usize].xingxia_used_round;
        let round = ctx.state.turn.round;
        if used != 0 && round.saturating_sub(used) < 2 {
            return None;
        }
        // 焰灵定位：BOSS_GENERALS 段内按名查目录索引（表序重排稳健）
        let base =
            (ctx.catalog.general_count() - crate::content::standard::BOSS_GENERALS.len()) as u16;
        let yanling = (base..ctx.catalog.general_count() as u16)
            .find(|&d| ctx.catalog.general_meta(d).map(|g| g.0) == Some("焰灵"))?;
        let seats = ctx.state.players.len() as u8;
        let target = (0..seats).find(|&t| {
            ctx.state.players[t as usize].alive
                && ctx.state.players[t as usize].general_slots[0] == Some(yanling)
        })?;
        Some(Box::new(XingxiaMachine {
            owner,
            stage: XINGXIA_STAGE_DAMAGE,
            actor: target as u32,
        }))
    }
}

/// 星夏机器：损焰灵线→追击线（敌方弃红或 1 火伤）。
#[derive(Clone)]
pub(in super::super) struct XingxiaMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: u8,
    pub(in super::super) actor: u32,
}

impl XingxiaMachine {
    /// 焦点敌方的红色牌（手牌+装备）选项。
    fn red_options(ctx: &Ctx<'_>, t: PlayerId) -> Vec<ChoiceOption> {
        let mut options: Vec<ChoiceOption> = ctx
            .core
            .player(t)
            .hand
            .iter()
            .copied()
            .filter(|c| {
                matches!(
                    ctx.catalog.cards[*c as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                )
            })
            .map(ChoiceOption::Card)
            .collect();
        for c in ctx.core.player(t).equips.iter().copied() {
            if matches!(
                ctx.catalog.cards[c as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            ) {
                options.push(ChoiceOption::Card(c));
            }
        }
        options
    }
}

impl SkillMachine for XingxiaMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossXingxia {
            owner: self.owner,
            stage: self.stage,
            actor: self.actor,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            XINGXIA_STAGE_DAMAGE => {
                let t = self.actor as PlayerId;
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    2,
                    crate::event::DamageNature::Fire,
                    None,
                )));
                ctx.core.player_mut(self.owner).xingxia_used_round = ctx.core.turn.round;
                // 无敌方可追击 → 仅损焰灵即止
                let seats = ctx.core.players.len() as u8;
                let has_enemy = (0..seats).any(|t| {
                    ctx.core.player(t).alive
                        && ctx.core.is_boss_side(t) != ctx.core.is_boss_side(self.owner)
                });
                if !has_enemy {
                    return Flow::Done;
                }
                self.stage = XINGXIA_STAGE_ASK_ENEMY;
                Flow::Yield
            }
            XINGXIA_STAGE_ASK_ENEMY => {
                let seats = ctx.core.players.len() as u8;
                let candidates: Vec<ChoiceOption> = (0..seats)
                    .filter(|&t| {
                        ctx.core.player(t).alive
                            && ctx.core.is_boss_side(t) != ctx.core.is_boss_side(self.owner)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                self.stage = XINGXIA_STAGE_ENEMY_DONE;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "xingxia_enemy",
                        options: candidates,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            XINGXIA_STAGE_ENEMY_DONE => match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(t)) => {
                    self.actor = t as u32;
                    let options = Self::red_options(ctx, t);
                    self.stage = XINGXIA_STAGE_DISCARD_DONE;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "xingxia_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                _ => Flow::Done,
            },
            XINGXIA_STAGE_DISCARD_DONE => {
                let t = self.actor as PlayerId;
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Card(c)) => {
                        // he 位：手牌优先，装备区移除并撤技能
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else {
                            ctx.core.player_mut(t).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                        }
                    }
                    _ => {
                        // 放弃弃红 → 1 点火焰伤害
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        )));
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}
