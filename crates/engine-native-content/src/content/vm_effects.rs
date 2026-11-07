//! Content-owned VM commit machines. The kernel only stores and pumps their continuations.
mod baoshu;
mod batch13_cards;
mod batch13_targets;
mod batch14_cards;
mod batch14_targets;
mod batch15_resolvers;
mod batch16_resolvers;
mod danshou;
mod dcbihuo;
mod dcdichou;
mod fenghe;
mod pingkou;
mod tiqi;

use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageWindowKind, EventData};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;
use rule_api::EffectRequest;

pub(crate) fn step(
    ctx: &mut Ctx<'_>,
    skill: &str,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    if let Some(flow) = batch13_targets::step(ctx, effect, state) {
        return Some(flow);
    }
    if let Some(flow) = batch13_cards::step(ctx, effect, state) {
        return Some(flow);
    }
    if let Some(flow) = batch14_targets::step(ctx, skill, effect, state) {
        return Some(flow);
    }
    if let Some(flow) = batch14_cards::step(ctx, skill, effect, state) {
        return Some(flow);
    }
    if let Some(flow) = batch15_resolvers::step(ctx, skill, effect, state) {
        return Some(flow);
    }
    if let Some(flow) = batch16_resolvers::step(ctx, skill, effect, state) {
        return Some(flow);
    }
    if let Some(flow) = dcdichou::step(ctx, effect, state) {
        return Some(flow);
    }
    match effect {
        EffectRequest::ChooseGiveOrRecastOneHand {
            player,
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(fenghe::step(
                ctx,
                owner,
                *require_skill_def,
                *grant_skill_def,
                *grant_mark_def,
                state,
            ))
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
            player, amount, ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(pingkou::step(ctx, owner, *amount, state))
        }
        EffectRequest::ResolveOlJieDanshou { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(danshou::step(ctx, skill, owner, state))
        }
        EffectRequest::ResolveTiqi { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(tiqi::step(ctx, skill, owner, state))
        }
        EffectRequest::ResolveBaoshu { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(baoshu::step(ctx, skill, owner, state))
        }
        EffectRequest::ResolveDcbihuo { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(dcbihuo::step(ctx, skill, owner, state))
        }
        // R05-E（合同 v3 §2）：通用 AddMark/RemoveMark 碰派生 helper 键
        // 即拒——语义槽 (I,role,target) 只有与执行绑定匹配的 compound 能
        // 写。提交前复检（挂起队列恢复走同一泵路径，同口径复检）。
        EffectRequest::AddMark { mark, .. } | EffectRequest::RemoveMark { mark, .. } => {
            if is_r05_helper(ctx.catalog, mark.raw()) {
                r05_fault(
                    ctx,
                    skill,
                    "R05HelperKeyRejected",
                    "generic mark effects cannot touch derived r05 helper keys",
                );
                return Some(Flow::Done);
            }
            None
        }
        _ => None,
    }
}

pub(crate) fn active_ask(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if let Some(expected) = batch13_targets::active_ask(effect, state, core) {
        return Some(expected);
    }
    if let Some(expected) = batch13_cards::active_ask(effect, state, core, catalog) {
        return Some(expected);
    }
    if let Some(expected) = batch14_targets::active_ask(effect, state, core, catalog) {
        return Some(expected);
    }
    if let Some(expected) = batch14_cards::active_ask(effect, state, core, catalog) {
        return Some(expected);
    }
    if let Some(expected) = batch15_resolvers::active_ask(effect, state, core, catalog) {
        return Some(expected);
    }
    if let Some(expected) = batch16_resolvers::active_ask(effect, state, core, catalog) {
        return Some(expected);
    }
    if let Some(expected) = dcdichou::active_ask(effect, state, core) {
        return Some(expected);
    }
    match effect {
        EffectRequest::ChooseGiveOrRecastOneHand { player, .. } => {
            fenghe::active_ask(core, catalog, seat(*player, core)?, state)
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw { player, .. } => {
            pingkou::active_ask(core, seat(*player, core)?, state)
        }
        EffectRequest::ResolveOlJieDanshou { player, skill } => {
            let owner = seat(*player, core)?;
            let name = catalog.skills.get(skill.raw() as usize)?.name;
            danshou::active_ask(core, owner, name, state)
        }
        EffectRequest::ResolveTiqi { player, .. } => {
            tiqi::active_ask(core, seat(*player, core)?, state)
        }
        EffectRequest::ResolveBaoshu { player, .. } => {
            baoshu::active_ask(core, seat(*player, core)?, state)
        }
        EffectRequest::ResolveDcbihuo { player, .. } => {
            dcbihuo::active_ask(core, seat(*player, core)?, state)
        }
        _ => None,
    }
}

fn seat(player: rule_api::PlayerId, core: &GameStateCore) -> Option<u8> {
    let seat = player.raw().checked_sub(1)?;
    (seat < core.players.len() as u32).then_some(seat as u8)
}

fn cards(core: &GameStateCore, player: u8) -> Vec<ChoiceOption> {
    core.player(player)
        .hand
        .iter()
        .chain(&core.player(player).equips)
        .map(|c| ChoiceOption::Card(*c))
        .collect()
}

// ---- R05-E（裁决 28 合同 v3 §2/§5）：三固定 compound 的共享基建 ----

/// 四角色固定名后缀（装载器按同一模板派生；顺序 Limit/Combs/Next/Current）。
pub(super) const R05_ROLES: [&str; 4] = ["limit", "combs", "next", "current"];

/// 派生 helper 的目录名（`{逻辑技能名}#r05:{role}`）。
fn r05_helper_name(skill: &str, role: &str) -> String {
    format!("{skill}#r05:{role}")
}

/// 语义槽 (I,role,target) 的目录 def 解析；未派生（绕过装载器的包）即 None。
fn r05_helper_def(catalog: &Catalog, skill: &str, role: &str) -> Option<u32> {
    catalog
        .skill_by_name(&r05_helper_name(skill, role))
        .map(|def| def as u32)
}

/// 派生 helper 键判定：目录名带固定模板后缀。`#` 不出现在任何既有目录
/// 技能名中（装载器模板独占），故名字匹配即身份匹配。
fn is_r05_helper(catalog: &Catalog, mark: u32) -> bool {
    catalog
        .skills
        .get(mark as usize)
        .is_some_and(|def| def.name.contains("#r05:"))
}

/// 有符号累加且**含 0 与负数均保留条目**（§5：抵消 Some0 不删；与通用
/// AddMark 的 0 删除语义不同）。溢出返回 None（调用方按 Fault 拒绝）。
fn r05_mark_add(core: &mut GameStateCore, seat: u8, def: u32, delta: i32) -> Option<i32> {
    let key = (seat, def);
    let base = core.vm_aux.marks.get(&key).copied().unwrap_or(0);
    let next = base.checked_add(delta)?;
    core.vm_aux.marks.insert(key, next);
    Some(next)
}

/// compound 机故障：技能/类别/详情可定位（与 rvm_bridge 的 VmFault 同形）。
fn r05_fault(ctx: &mut Ctx<'_>, skill: &str, kind: &str, detail: &str) {
    if ctx.res.fault.is_none() {
        ctx.res.fault = Some(crate::machine::VmFault {
            skill: skill.to_string(),
            owner: 0,
            event: "R05".into(),
            kind: kind.into(),
            detail: detail.into(),
        });
    }
}

/// 目标装默认 helper（enabled；§6 生命周期随 F 批接线）。
fn r05_install_helper(ctx: &mut Ctx<'_>, seat: u8, def: u32) {
    if let Ok(id) = u16::try_from(def) {
        ctx.ensure_skill(
            (seat, id),
            crate::state::SkillInstance {
                owner: seat,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }
}

/// 当前触发帧的最近真实 D 祖先（live 读 n）。D 帧随 After 窗口仍在栈上。
fn current_draw_phase(ctx: &Ctx<'_>) -> Option<(u8, u32)> {
    for frame in ctx.res.stack.iter().rev() {
        if let EventData::DrawPhase {
            player, actual_gain, ..
        } = &frame.data
        {
            return Some((*player, *actual_gain));
        }
    }
    None
}

/// 当前帧是否为伤害 Source/End 窗口（返回窗口种与权威伤害根帧号）。
fn current_damage_window(ctx: &Ctx<'_>) -> Option<(DamageWindowKind, u32)> {
    let id = ctx.current_frame_id();
    match &ctx.res.frame(id)?.data {
        EventData::DamageWindow { root, window } => Some((*window, *root)),
        _ => None,
    }
}

/// 当前帧的权威伤害载荷（窗口根帧）。
fn damage_root_data(ctx: &Ctx<'_>, root: u32) -> Option<crate::event::DamageData> {
    match &ctx.res.frame(root)?.data {
        EventData::Damage(d) => Some(d.clone()),
        _ => None,
    }
}

/// 当前帧是否为真实回合开始窗（Next→Current take 的授权面）。
fn current_turn_begin_player(ctx: &Ctx<'_>) -> Option<u8> {
    let id = ctx.current_frame_id();
    match &ctx.res.frame(id)?.data {
        EventData::TurnBegin { player } => Some(*player),
        _ => None,
    }
}

/// 当前帧是否为本人的真实准备阶段开始（宝梳授权面：Prepare 且
/// owner=W.player，合同 v3 §7）。
fn at_own_prepare(ctx: &Ctx<'_>, owner: u8) -> bool {
    let id = ctx.current_frame_id();
    matches!(
        &ctx.res.frame(id).map(|f| f.data.clone()),
        Some(EventData::Phase {
            player,
            phase: crate::event::Phase::Prepare,
            begin: true
        }) if *player == owner
    )
}

fn alive_options(core: &GameStateCore) -> Vec<ChoiceOption> {
    core.alive_players().map(ChoiceOption::Player).collect()
}

fn wait(ask: ExpectedAsk) -> Flow {
    Flow::WaitChoice {
        observer: ask.observer,
        spec: ChoiceSpec {
            kind: ask.kind,
            prompt: ask.prompt,
            options: ask.options,
            min: ask.min,
            max: ask.max,
            allow_cancel: ask.allow_cancel,
        },
    }
}

fn discard(ctx: &mut Ctx<'_>, owner: u8, picked: &[u32]) {
    for card in picked {
        if ctx.core.player(owner).hand.contains(card) {
            ctx.take_hand_card(owner, *card);
        } else {
            ctx.take_owned_card(owner, *card);
        }
    }
    ctx.core.discard.extend_from_slice(picked);
    ctx.out.push(crate::machine::LogEvent::CardDiscarded {
        player: owner,
        cards: picked.to_vec(),
    });
    ctx.emit(crate::event::EventData::CardsDiscarded {
        player: owner,
        cards: picked.to_vec(),
    });
}

/// Validate content continuation shapes before any machine can resume them.
pub(crate) fn validate(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    card_count: usize,
) -> bool {
    if state.is_empty() {
        return true;
    }
    let seats = core.players.len();
    match effect {
        EffectRequest::ResolveOlJieDanshou { player, .. } => {
            player.raw() > 0
                && player.raw() as usize <= seats
                && state.len() == 3
                && (1..=7).contains(&state[0])
                && (state[0] == 1 || (1..=4).contains(&state[1]))
                && (state[0] <= 2 || (state[2] as usize) < seats)
        }
        EffectRequest::ChooseGiveOrRecastOneHand { player, .. } => {
            player.raw() > 0
                && player.raw() as usize <= seats
                && state.len() == 3
                && (1..=5).contains(&state[0])
                && (state[0] == 1 || (state[1] as usize) < card_count)
                && state[2] as usize <= seats
                && (state[0] != 2
                    || core
                        .player((player.raw() - 1) as u8)
                        .hand
                        .contains(&state[1]))
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw { player, .. } => {
            player.raw() > 0
                && player.raw() as usize <= seats
                && state.len() >= 4
                && state.len() <= seats + 4
                && (1..=3).contains(&state[0])
                && (state[1] as usize) <= state.len() - 4
                && state[3] < (1 << seats)
                && state[4..]
                    .iter()
                    .all(|target| (*target as usize) < seats && *target + 1 != player.raw())
                && state[4..]
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == state.len() - 4
        }
        // R05-E：三 compound 续行形状。Tiqi [stage,n,target+1]；Baoshu
        //[stage,m,k,index,targets...]；Dcbihuo [stage,mode]。
        EffectRequest::ResolveTiqi { player, .. } => {
            player.raw() > 0
                && player.raw() as usize <= seats
                && state.len() == 3
                && (1..=3).contains(&state[0])
                && state[1] != 2
                && state[2] >= 1
                && (state[2] as usize) <= seats
        }
        EffectRequest::ResolveBaoshu { player, .. } => {
            player.raw() > 0
                && player.raw() as usize <= seats
                && state.len() >= 4
                && state.len() <= seats + 4
                && (1..=6).contains(&state[0])
                && ((state[0] == 1 && state.len() == 4 && state[2] == 0 && state[3] == 0)
                    || (state[0] >= 2
                        && state.len() > 4
                        && state[2] as usize == state.len() - 4
                        && (state[3] as usize) < state.len() - 4))
                && (state[0] == 1 || state[4..].iter().all(|t| (1..=seats as u32).contains(t)))
                && state[4..]
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == state.len() - 4
        }
        EffectRequest::ResolveDcbihuo { player, .. } => {
            player.raw() > 0
                && player.raw() as usize <= seats
                && state.len() == 2
                && state[0] == 1
                && state[1] <= 1
        }
        EffectRequest::ChooseOtherDamage { .. }
        | EffectRequest::ChooseTargetDraw { .. }
        | EffectRequest::ChooseOtherRecover { .. }
        | EffectRequest::ChooseDrawOrRecover { .. }
        | EffectRequest::ChooseOtherAddMark { .. }
        | EffectRequest::ResolveHfjieying { .. } => batch13_targets::validate(effect, state, core),
        EffectRequest::ChooseDiscardOneHandOrEquip { .. }
        | EffectRequest::DiscardBasicDrawEqual { .. }
        | EffectRequest::ChooseUseVirtualSha { .. }
        | EffectRequest::ChooseOtherDiscardTargetCard { .. }
        | EffectRequest::ChooseDiscardTargetCard { .. }
        | EffectRequest::ChooseOtherGiveOneHandEquip { .. } => {
            batch13_cards::validate(effect, state, core, card_count)
        }
        EffectRequest::ResolveDcdichou { .. } => dcdichou::validate(effect, state, core),
        _ => {
            batch14_targets::validate(effect, state, core)
                || batch14_cards::validate(effect, state, core, card_count)
                || batch15_resolvers::validate(effect, state, core)
                || batch16_resolvers::validate(effect, state, core)
        }
    }
}

pub(crate) fn active_eligible(
    module: &rule_vm::BytecodeModule,
    skill: &str,
    owner: u8,
    core: &GameStateCore,
) -> bool {
    use rule_vm::bytecode::{effect_tag, op};
    let tag = effect_tag::RESOLVE_OL_JIE_DANSHOU.to_le_bytes();
    // Only a verified single-resolver body has the resolver's intrinsic activation precondition.
    let single = [
        op::HOST_SELF,
        op::HOST_SKILL,
        op::HOST_EFFECT_EXT,
        tag[0],
        tag[1],
        op::HALT,
    ];
    if module.entry_pc == 0 && module.code == single {
        return !danshou::options(core, owner, skill).is_empty();
    }
    true
}
