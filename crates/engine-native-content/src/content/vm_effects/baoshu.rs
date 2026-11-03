//! R05-E 宝梳（合同 v3 §5）：成本窗＝选 1..=当前 maxHp 名在场角色（含
//! 自己、不重复、可取消）→ 成功后经既有 SkillLog 通道子结算并 yield
//!（登记差异：真实 logSkillBegin 子结算新事件随后续批次；此处 begin/end
//! 两帧近似）→ 读 maxHp、m=1+maxHp−k（可为 0/负）→ 按座次逐目标：
//! 在场→仅当前 linked 才发 link(false)→await 重查→仅当前 face_down 才
//! turnOver(false)（取消不重试、不 doubletoggle）→isIn 才装 helper 并
//! checked 加 delta=(m==0?1:m) 到 Combs 槽（含负数与 0 均保留条目）。
use super::*;

/// 状态：`[stage, m_bits, k, index, target+1 ...]`；stage 1=成本询问、
/// 2=SkillLog(begin) 子结算中、3=SkillLog(end) 子结算中、4=逐目标循环、
/// 5=link 子事件后、6=turnOver 子事件后。m 以 i32 位模式存 u32。
pub(super) fn step(ctx: &mut Ctx<'_>, skill: &str, owner: u8, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        // 授权面：本人的真实准备阶段开始（§7：Baoshu 为 Prepare 且
        // owner=W.player）。
        if !at_own_prepare(ctx, owner) {
            r05_fault(
                ctx,
                skill,
                "R05WindowMismatch",
                "baoshu requires the owner's real prepare phase begin window",
            );
            return Flow::Done;
        }
        let max_hp = ctx.core.player(owner).max_hp;
        if max_hp == 0 {
            return Flow::Done;
        }
        *state = vec![1, 0, 0, 0];
        return wait(cost_ask(ctx.core, owner));
    }
    match state[0] {
        1 => {
            if ctx.answer().is_empty() {
                return Flow::Done; // 成本取消：无任何状态变化
            }
            // 含自己、不重复（防御性去重——选项面本身不重复）。
            let mut picked: Vec<u8> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            picked.sort_unstable();
            picked.dedup();
            let alive = ctx.core.alive_players().collect::<Vec<_>>();
            if picked.is_empty()
                || picked.len() > ctx.core.player(owner).max_hp as usize
                || picked.iter().any(|p| !alive.contains(p))
            {
                r05_fault(ctx, skill, "R05CostInvalid", "baoshu cost selection invalid");
                return Flow::Done;
            }
            // 按座次取 targets（§5）。
            state.clear();
            state.extend_from_slice(&[2, 0, picked.len() as u32, 0]);
            state.extend(picked.iter().map(|p| *p as u32 + 1));
            skill_log(ctx, owner, skill, true);
            Flow::Yield
        }
        2 => {
            state[0] = 3;
            skill_log(ctx, owner, skill, false);
            Flow::Yield
        }
        3 => {
            // 子结算后读 maxHp（§4 反例：cost 与 content 之间 maxHp 可变，
            // 不在选择时冻结 m）。
            let k = state[2] as u32;
            let m = 1 + ctx.core.player(owner).max_hp as i32 - k as i32;
            state[1] = m as u32;
            state[3] = 0;
            state[0] = 4;
            self::step(ctx, skill, owner, state)
        }
        4 => {
            let index = state[3] as usize;
            if index >= state[2] as usize {
                return Flow::Done;
            }
            let target = (state[4 + index] - 1) as u8;
            if !ctx.core.player(target).alive {
                return advance(ctx, skill, owner, state);
            }
            // 仅当前 linked 才发 link(false)（commit 为翻转语义，故不
            // doubletoggle）；await 后重查 face_down。
            if ctx.core.player(target).linked {
                state[0] = 5;
                ctx.emit(crate::event::EventData::Linked {
                    player: target,
                    linked: false,
                });
                return Flow::Yield;
            }
            turnover_stage(ctx, skill, owner, state, target)
        }
        5 => {
            let target = (state[4 + state[3] as usize] - 1) as u8;
            turnover_stage(ctx, skill, owner, state, target)
        }
        6 => {
            let target = (state[4 + state[3] as usize] - 1) as u8;
            settle_target(ctx, skill, owner, state, target)
        }
        _ => Flow::Done,
    }
}

/// link 子事件后的翻转检查（await 重查：face_down 可能被子事件改变）。
fn turnover_stage(
    ctx: &mut Ctx<'_>,
    skill: &str,
    owner: u8,
    state: &mut Vec<u32>,
    target: u8,
) -> Flow {
    // 仅当前 face_down 才 turnOver(false)——翻转提交为 toggle，当前
    // face_down 时翻即朝上；取消翻面不重试（§5）。
    if ctx.core.player(target).face_down {
        state[0] = 6;
        ctx.emit(crate::event::EventData::TurnOverBefore {
            player: target,
            cancelled: false,
        });
        return Flow::Yield;
    }
    settle_target(ctx, skill, owner, state, target)
}

/// isIn 才装 helper 并 checked 加 delta（含负数与 0 均保留条目，不删）。
fn settle_target(
    ctx: &mut Ctx<'_>,
    skill: &str,
    owner: u8,
    state: &mut Vec<u32>,
    target: u8,
) -> Flow {
    if ctx.core.player(target).alive {
        let m = state[1] as i32;
        let delta = if m == 0 { 1 } else { m };
        let Some(def) = r05_helper_def(ctx.catalog, skill, R05_ROLES[1]) else {
            r05_fault(ctx, skill, "R05HelperMissing", "combs helper not derived");
            return Flow::Done;
        };
        r05_install_helper(ctx, target, def);
        if r05_mark_add(ctx.core, target, def, delta).is_none() {
            r05_fault(ctx, skill, "R05MarkOverflow", "combs slot i32 overflow");
            return Flow::Done;
        }
    }
    advance(ctx, skill, owner, state)
}

fn advance(ctx: &mut Ctx<'_>, skill: &str, owner: u8, state: &mut Vec<u32>) -> Flow {
    state[3] += 1;
    state[0] = 4;
    self::step(ctx, skill, owner, state)
}

/// 既有 SkillLog 通道近似（登记差异：非新增 SkillLogBegin 事件类型；
/// begin/end 两帧走普通窗口序，无子结算效果面）。
fn skill_log(ctx: &mut Ctx<'_>, owner: u8, skill: &str, begin: bool) {
    let def = ctx.catalog.skill_by_name(skill).unwrap_or(0);
    ctx.emit(crate::event::EventData::SkillLog {
        owner,
        def,
        begin,
    });
}

fn cost_ask(core: &GameStateCore, owner: u8) -> ExpectedAsk {
    let max = core.player(owner).max_hp as usize;
    let alive = core.alive_players().count();
    ask(
        owner,
        "rvm_baoshu_cost",
        ChoiceKind::SelectPlayers,
        alive_options(core),
        1,
        max.min(alive),
        true,
    )
}

pub(super) fn active_ask(core: &GameStateCore, owner: u8, state: &[u32]) -> Option<ExpectedAsk> {
    if !core.player(owner).alive {
        return None;
    }
    (state.first() == Some(&1)).then(|| cost_ask(core, owner))
}
