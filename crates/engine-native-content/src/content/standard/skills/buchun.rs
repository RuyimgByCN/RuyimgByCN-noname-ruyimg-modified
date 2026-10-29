use super::*;

// ---------------------------------------------------------------------------
// 不臣：出牌阶段主动技两线——场上有已死亡的书精 → 失去 1 点体力并把
// 它们全部复活（回复至 1 点、摸 2）；否则令一名己方受伤角色回复 2 点
// 体力。发动后登记轮次（间隔 ≥2 轮可再发动）。勾芒（规则 v240）。
// ---------------------------------------------------------------------------

/// 不臣阶段码（快照显式编码；0/1=复活线，2/3=恢复线）。
const BUCHUN_STAGE_LOSE_HP: u8 = 0;
const BUCHUN_STAGE_REVIVE_NEXT: u8 = 1;
const BUCHUN_STAGE_ASK_TARGET: u8 = 2;
const BUCHUN_STAGE_WAIT_TARGET: u8 = 3;

pub(in super::super) struct BuchunBehavior;

impl SkillBehavior for BuchunBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        // 主动技：不经触发总线（ActivateSkill 直调 activate）
        false
    }
    fn start(&self, _owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        unreachable!("不臣为主动技，不经触发总线")
    }
    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        // 间隔门控：距上次发动 <2 轮 → 不可发动（上游 storage+roundNumber
        // 过滤同口径）
        let used = ctx.state.players[owner as usize].buchun_used_round;
        let round = ctx.state.turn.round;
        if used != 0 && round.saturating_sub(used) < 2 {
            return None;
        }
        // 书精将定位：BOSS_GENERALS 段内按名查目录索引（表序重排稳健）
        let base =
            (ctx.catalog.general_count() - crate::content::standard::BOSS_GENERALS.len()) as u16;
        let shujing = (base..ctx.catalog.general_count() as u16)
            .find(|&d| ctx.catalog.general_meta(d).map(|g| g.0) == Some("书精"))?;
        let seats = ctx.state.players.len() as u8;
        let dead_shujings: Vec<u32> = (0..seats)
            .filter(|&t| {
                let p = &ctx.state.players[t as usize];
                !p.alive && p.general_slots[0] == Some(shujing)
            })
            .map(|t| t as u32)
            .collect();
        if !dead_shujings.is_empty() {
            // 复活线：失 1 体 → 全部复活（回 1 体、各摸 2）
            return Some(Box::new(BuchunMachine {
                owner,
                stage: BUCHUN_STAGE_LOSE_HP,
                targets: dead_shujings,
            }));
        }
        // 恢复线：须存在同阵营受伤角色（上游 filterTarget isFriendOf+
        // isDamaged 同口径；is_boss_side 异侧=敌）
        let has_damaged_friend = (0..seats).any(|t| {
            let p = &ctx.state.players[t as usize];
            p.alive
                && p.hp < p.max_hp as i32
                && ctx.state.is_boss_side(t) == ctx.state.is_boss_side(owner)
        });
        if has_damaged_friend {
            return Some(Box::new(BuchunMachine {
                owner,
                stage: BUCHUN_STAGE_ASK_TARGET,
                targets: Vec::new(),
            }));
        }
        None
    }
}

/// 不臣机器：复活线两步（失体→逐个复活）、恢复线两步（询问→回 2）。
#[derive(Clone)]
pub(in super::super) struct BuchunMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: u8,
    pub(in super::super) targets: Vec<u32>,
}

impl SkillMachine for BuchunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossBuchun {
            owner: self.owner,
            stage: self.stage,
            targets: self.targets.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            BUCHUN_STAGE_LOSE_HP => {
                ctx.emit(EventData::LoseHp {
                    player: self.owner,
                    amount: 1,
                });
                self.stage = BUCHUN_STAGE_REVIVE_NEXT;
                Flow::Yield
            }
            BUCHUN_STAGE_REVIVE_NEXT => {
                while let Some(t) = self.targets.pop() {
                    let t = t as PlayerId;
                    if ctx.core.player(t).alive {
                        continue;
                    }
                    // 复活（上游 revive(1)+draw(2) 同口径；重整计数清零
                    // 与重整复活臂同款；轮次登记）
                    {
                        let p = ctx.core.player_mut(t);
                        p.alive = true;
                        p.dying = false;
                        p.hp = 1;
                        p.boss_chongzheng = 0;
                    }
                    ctx.log(LogEvent::BossRevived { player: t });
                    ctx.draw_cards(t, 2);
                    ctx.core.player_mut(self.owner).buchun_used_round = ctx.core.turn.round;
                    return Flow::Yield;
                }
                Flow::Done
            }
            BUCHUN_STAGE_ASK_TARGET => {
                let seats = ctx.core.players.len() as u8;
                let candidates: Vec<ChoiceOption> = (0..seats)
                    .filter(|&t| {
                        let p = ctx.core.player(t);
                        p.alive
                            && p.hp < p.max_hp as i32
                            && ctx.core.is_boss_side(t) == ctx.core.is_boss_side(self.owner)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                self.stage = BUCHUN_STAGE_WAIT_TARGET;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "buchun_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            BUCHUN_STAGE_WAIT_TARGET => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    ctx.recover(t, 2, None);
                    ctx.core.player_mut(self.owner).buchun_used_round = ctx.core.turn.round;
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}
