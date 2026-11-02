use super::super::*;

impl HostMachine {
    pub(super) fn step_boss(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 青木：标记四象链 + 表现隐藏（座位交互门控近似=日志；无存档字段）
            HostBehavior::BossQingmu2 => {
                sixiang_mark_hide(ctx, self.owner, "青木", false);
                Flow::Done
            }
            // 青木·转阶：取消死亡执行 + 换勾芒 + 全场回满 + 两书精 + 挑战者重整
            HostBehavior::BossQingmu2x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "勾芒",
                        add_fellows: &["书精", "书精"],
                        side_reinits: &[],
                        heal_all: true,
                        chain_level: 2,
                    },
                );
                Flow::Done
            }
            // 青木3：三人隐藏（本体+环邻）
            HostBehavior::BossQingmu3 => {
                sixiang_mark_hide(ctx, self.owner, "青木3", true);
                Flow::Done
            }
            // 青木3·转阶：太昊 + 侧座勾芒/书精 + 挑战者重整
            HostBehavior::BossQingmu3x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "太昊",
                        add_fellows: &[],
                        side_reinits: &[(-1, "勾芒"), (1, "书精")],
                        heal_all: false,
                        chain_level: 3,
                    },
                );
                Flow::Done
            }
            // 白芒 / 赤炎 / 玄麟：同青木 dieBegin hide（D4/B2）
            HostBehavior::BossBaimang2 => {
                sixiang_mark_hide(ctx, self.owner, "白芒", false);
                Flow::Done
            }
            HostBehavior::BossChiyan2 => {
                sixiang_mark_hide(ctx, self.owner, "赤炎", false);
                Flow::Done
            }
            HostBehavior::BossXuanlin2 => {
                sixiang_mark_hide(ctx, self.owner, "玄麟", false);
                Flow::Done
            }
            // 白芒·转阶：蓐收 + 两明星 + 全场回满 + 挑战者重整
            HostBehavior::BossBaimang2x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "蓐收",
                        add_fellows: &["明星", "明星"],
                        side_reinits: &[],
                        heal_all: true,
                        chain_level: 2,
                    },
                );
                Flow::Done
            }
            // 赤炎·转阶：祝融 + 两焰灵 + 全场回满 + 挑战者重整
            HostBehavior::BossChiyan2x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "祝融",
                        add_fellows: &["焰灵", "焰灵"],
                        side_reinits: &[],
                        heal_all: true,
                        chain_level: 2,
                    },
                );
                Flow::Done
            }
            // 玄麟·转阶：共工 + 一玄冥 + 全场回满 + 挑战者重整
            HostBehavior::BossXuanlin2x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "共工",
                        add_fellows: &["玄冥"],
                        side_reinits: &[],
                        heal_all: true,
                        chain_level: 2,
                    },
                );
                Flow::Done
            }
            // 白芒3 / 赤炎3 / 玄麟3：同青木3 三人 hide（D4/B3）
            HostBehavior::BossBaimang3 => {
                sixiang_mark_hide(ctx, self.owner, "白芒3", true);
                Flow::Done
            }
            HostBehavior::BossChiyan3 => {
                sixiang_mark_hide(ctx, self.owner, "赤炎3", true);
                Flow::Done
            }
            HostBehavior::BossXuanlin3 => {
                sixiang_mark_hide(ctx, self.owner, "玄麟3", true);
                Flow::Done
            }
            // 白芒3·转阶：少昊 + 侧座蓐收/明星 + 一明星 fellow
            HostBehavior::BossBaimang3x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "少昊",
                        add_fellows: &["明星"],
                        side_reinits: &[(-1, "蓐收"), (1, "明星")],
                        heal_all: false,
                        chain_level: 3,
                    },
                );
                Flow::Done
            }
            // 赤炎3·转阶：炎帝 + 侧座祝融/焰灵
            HostBehavior::BossChiyan3x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "炎帝",
                        add_fellows: &[],
                        side_reinits: &[(-1, "祝融"), (1, "焰灵")],
                        heal_all: false,
                        chain_level: 3,
                    },
                );
                Flow::Done
            }
            // 玄麟3·转阶：颛顼 + 共工 fellow + 次座玄冥
            HostBehavior::BossXuanlin3x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "颛顼",
                        add_fellows: &["共工"],
                        side_reinits: &[(1, "玄冥")],
                        heal_all: false,
                        chain_level: 3,
                    },
                );
                Flow::Done
            }
            // 后援3/4：死亡前置隐藏（日志近似；不改 stage.kind）
            HostBehavior::BossBianshen3 => {
                bianshen_mark_hide(ctx, self.owner, "后援3");
                Flow::Done
            }
            HostBehavior::BossBianshen4 => {
                bianshen_mark_hide(ctx, self.owner, "后援4");
                Flow::Done
            }
            // 后援3·转阶：种子化 → 白无常/黑无常
            HostBehavior::BossBianshen3x => {
                bianshen_run_change_boss(ctx, self.owner, &["白无常", "黑无常"]);
                Flow::Done
            }
            // 后援4·转阶：种子化 → 夜叉/罗刹
            HostBehavior::BossBianshen4x => {
                bianshen_run_change_boss(ctx, self.owner, &["夜叉", "罗刹"]);
                Flow::Done
            }
            HostBehavior::BossHuanren => {
                huanren_on_game_start(ctx, self.owner);
                Flow::Done
            }
            HostBehavior::BossHuanrenDie => match self.stage {
                HostStage::Act => huanren_die_ask(ctx, self),
                HostStage::HuanrenPick => {
                    huanren_die_commit(ctx, self);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::BossQingyi => {
                wing_qing(ctx, self.owner);
                Flow::Done
            }
            HostBehavior::BossChiyi => {
                wing_chi(ctx, self.owner);
                Flow::Done
            }
            HostBehavior::BossChiyiMark => {
                if !ctx.modify_damage_amount(1) { return Flow::Done; }
                Flow::Done
            }
            HostBehavior::BossBaiyi => {
                wing_bai_discard(ctx, self.owner);
                Flow::Done
            }
            HostBehavior::BossBaiyiThunder => {
                ctx.current_damage_mut().prevented = true;
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
