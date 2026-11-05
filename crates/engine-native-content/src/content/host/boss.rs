use super::*;

/// 隐藏链转阶参数表（结构同构，避免八份拷贝）。
pub(super) struct SixiangChainParams {
    pub(super) next_boss: &'static str,
    pub(super) add_fellows: &'static [&'static str],
    /// 环相对位移 → 将名：-1=prevSeat，+1=nextSeat。
    pub(super) side_reinits: &'static [(i8, &'static str)],
    pub(super) heal_all: bool,
    pub(super) chain_level: u8,
}

pub(super) fn wing_bit(base: u8, round: u32) -> Option<u8> {
    let shift = match round {
        3 => 0,
        5 => 1,
        7 => 2,
        _ => return None,
    };
    Some(1u8 << (base + shift))
}

/// 青翼/赤翼轮次门（A2 H14/H15）：上游 player phaseZhunbeiBegin＝Boss
/// 本人**准备阶段开始**（原锚判定阶段开始）＋轮次 3/5/7＋armed 位未置。
pub(super) fn wing_round_gate(
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
    bit_base: u8,
) -> bool {
    let EventData::Phase {
        player,
        phase: crate::event::Phase::Prepare,
        begin: true,
    } = event
    else {
        return false;
    };
    if *player != owner || !matches!(ctx.state.turn.round, 3 | 5 | 7) {
        return false;
    }
    if !ctx
        .state
        .boss_mode()
        .map(|(b, _)| b == owner)
        .unwrap_or(false)
    {
        return false;
    }
    let bit = wing_bit(bit_base, ctx.state.turn.round).unwrap_or(0);
    let armed = ctx.state.boss_stage().map(|s| s.wing_armed).unwrap_or(0);
    armed & bit == 0
}

pub(super) fn with_boss_stage(
    ctx: &mut Ctx<'_>,
    f: impl FnOnce(&mut crate::state::BossStageState),
) {
    if let crate::state::GameMode::Boss { stage, .. } = &mut ctx.core.mode {
        f(stage);
    }
}

pub(super) fn general_def_first(catalog: &crate::catalog::Catalog, name: &str) -> Option<u16> {
    (0..catalog.general_count() as u16)
        .find(|&d| catalog.general_meta(d).map(|g| g.0) == Some(name))
}

pub(super) fn grant_named_skill(ctx: &mut Ctx<'_>, seat: PlayerId, name: &str) {
    let Some(def) = ctx.catalog.skill_by_name(name) else {
        return;
    };
    ctx.insert_skill(
        (seat, def),
        crate::state::SkillInstance {
            owner: seat,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
}

pub(super) fn seat_with_general(ctx: &Ctx<'_>, name: &str) -> Option<PlayerId> {
    let g = boss_general_def(ctx.catalog, name)?;
    (0..ctx.core.players.len() as u8).find(|&p| ctx.core.player(p).general_slots[0] == Some(g))
}

pub(super) fn forced_discard_n(ctx: &mut Ctx<'_>, seat: PlayerId, mut left: usize) {
    let hand: Vec<CardId> = ctx.core.player(seat).hand.clone();
    for c in hand {
        if left == 0 {
            break;
        }
        ctx.take_hand_card(seat, c);
        ctx.to_discard(c);
        left -= 1;
    }
    let equips: Vec<CardId> = ctx.core.player(seat).equips.clone();
    for c in equips {
        if left == 0 {
            break;
        }
        ctx.core.player_mut(seat).equips.retain(|x| *x != c);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, seat, c);
        ctx.to_discard(c);
        left -= 1;
    }
}

pub(super) fn bianshen_mark_hide(ctx: &mut Ctx<'_>, boss: PlayerId, skill: &'static str) {
    ctx.log(LogEvent::SkillTriggered { skill, owner: boss });
}

pub(super) fn bianshen_run_change_boss(ctx: &mut Ctx<'_>, boss: PlayerId, pool: &[&str]) {
    let defs: Vec<u16> = pool
        .iter()
        .filter_map(|n| boss_general_def(ctx.catalog, n))
        .collect();
    if defs.is_empty() {
        return;
    }
    ctx.core.player_mut(boss).death_cancelled = true;
    let j = ctx.core.rng.gen_range(defs.len() as u32) as usize;
    let g = defs[j];
    sixiang_reinit_seat(ctx, boss, g, false);
    sixiang_discard_zone(ctx, boss);
    ctx.start_draw(boss, 4);
    ctx.core.turn.current = boss;
    ctx.core.turn.phase = crate::event::Phase::Judge;
}

/// R05-C2：孟婆乱入。上游 boss.js:3439-3455（开局）＝game.addFellow 只
/// 入座**不发牌**；boss.js:3575-3577（转关，level 已 ≥1）＝fellow.
/// directgain(get.cards(4))。原生按此对齐：开局（draw4=false）零摸牌零
/// 扰动（旧「摸 4 再逐张取回压栈」已落 CardsDrawn 日志＝对日志面有扰
/// 动，删除）；转关卡（draw4=true）登记 4 张到 `draws`（由调用方统一
/// 逆序压 R 帧，保持与同步摸牌相同的执行次序）。
pub(super) fn huanren_spawn_mengpo(ctx: &mut Ctx<'_>, draw4: bool, draws: &mut Vec<(PlayerId, u32)>) {
    let Some(pid) = sixiang_add_fellow(ctx, "孟婆") else {
        return;
    };
    if draw4 {
        draws.push((pid, 4));
    }
    with_boss_stage(ctx, |stage| stage.mengpo_spawned = true);
}

pub(super) fn huanren_remove_mengpo(ctx: &mut Ctx<'_>) {
    let mengpo = boss_general_def(ctx.catalog, "孟婆");
    let seats = ctx.core.players.len() as u8;
    let mut gone = Vec::new();
    for t in 0..seats {
        if mengpo.is_some() && ctx.core.player(t).general_slots[0] == mengpo {
            let p = ctx.core.player_mut(t);
            p.alive = false;
            p.dying = false;
            p.hp = 0;
            gone.push(t);
        }
    }
    if let crate::state::GameMode::Boss { fellows, .. } = &mut ctx.core.mode {
        fellows.retain(|p| !gone.contains(p));
    }
}

pub(super) fn huanren_on_game_start(ctx: &mut Ctx<'_>, boss: PlayerId) {
    let banned: Vec<u16> = HUANREN_BANNED
        .iter()
        .filter_map(|n| general_def_first(ctx.catalog, n))
        .collect();
    with_boss_stage(ctx, |stage| {
        stage.kind = crate::state::BossStageKind::ShidianYanluo;
        stage.level = 0;
        stage.banned_generals = banned.clone();
    });
    let replacement = general_def_first(ctx.catalog, HUANREN_REPLACEMENT);
    let seats = ctx.core.players.len() as u8;
    for t in 0..seats {
        if t == boss {
            continue;
        }
        let g = ctx.core.player(t).general_slots[0];
        if g.is_some_and(|id| banned.contains(&id)) {
            if let Some(rep) = replacement {
                sixiang_reinit_seat(ctx, t, rep, false);
            }
        }
    }
    if ctx.core.rng.gen_range(100) < 40 {
        let mut draws = Vec::new();
        huanren_spawn_mengpo(ctx, false, &mut draws);
        // 开局臂上游不发牌（draws 恒空）——零 R 帧、零扰动。
    }
}

/// 十殿下一关候选。level==2 且孟婆已死且阶段数≤50 时改为地藏王。
/// 映种（四季映姬）按 ADR 0783 候选恒不可达，不进入此表。
pub(super) fn huanren_candidates(ctx: &Ctx<'_>, level: u8) -> Vec<u16> {
    let dizang = level == 2
        && ctx.core.turn.phase_number <= 50
        && ctx.core.boss_stage().is_some_and(|s| s.mengpo_died);
    let names: &[&str] = if dizang {
        &["地藏王"]
    } else {
        match level {
            0 => &["楚江王", "宋帝王", "五官王", "阎罗王"],
            1 => &["卞城王", "泰山王", "都市王", "平等王"],
            2 => &["转轮王"],
            _ => &[],
        }
    };
    names
        .iter()
        .filter_map(|n| boss_general_def(ctx.catalog, n))
        .collect()
}

pub(super) fn huanren_apply(ctx: &mut Ctx<'_>, boss: PlayerId, general: u16) {
    ctx.core.player_mut(boss).death_cancelled = true;
    let is_dizang = boss_general_def(ctx.catalog, "地藏王") == Some(general);
    sixiang_reinit_seat(ctx, boss, general, false);
    sixiang_discard_zone(ctx, boss);
    // R05-C2：摸牌按同步执行序登记（换将 4 → 地藏 3 → 孟婆 4），步末
    // 逆序压 R 帧＝后压先执行，保持与旧 draw_cards 相同的取牌次序与
    // CardsDrawn 次序（上游 boss.js:3558 changeBoss directgain(4)、
    // 3561-3563 地藏 draw(3)、3575-3577 孟婆 directgain(4) 同序）。
    let mut draws: Vec<(PlayerId, u32)> = Vec::new();
    draws.push((boss, 4));
    if is_dizang {
        draws.push((boss, 3));
    }
    grant_named_skill(ctx, boss, "还人");
    grant_named_skill(ctx, boss, "还人·转关");
    let spawned = ctx.core.boss_mengpo_spawned();
    with_boss_stage(ctx, |stage| {
        stage.level = stage.level.saturating_add(1);
        stage.pending = None;
        stage.kind = crate::state::BossStageKind::ShidianYanluo;
    });
    let level = ctx.core.boss_stage().map(|s| s.level).unwrap_or(0);
    if !spawned {
        let roll = ctx.core.rng.gen_range(100);
        if roll < 50 || level == 2 {
            huanren_spawn_mengpo(ctx, level != 0, &mut draws);
        }
    } else {
        huanren_remove_mengpo(ctx);
    }
    ctx.core.turn.current = boss;
    ctx.core.turn.phase = crate::event::Phase::Judge;
    push_draws_reversed(ctx, &draws);
}

/// R05-C2：把按同步执行序收集的摸牌请求逆序压成 R 帧（后压先执行＝
/// 收集序执行），供 boss 转阶/四象链等多点摸牌保持同步取牌次序。
pub(super) fn push_draws_reversed(ctx: &mut Ctx<'_>, draws: &[(PlayerId, u32)]) {
    for &(p, n) in draws.iter().rev() {
        ctx.start_draw(p, n);
    }
}

pub(super) fn huanren_die_ask(ctx: &mut Ctx<'_>, mach: &mut HostMachine) -> Flow {
    let level = ctx.core.boss_stage().map(|s| s.level).unwrap_or(0);
    let cands = huanren_candidates(ctx, level);
    if cands.is_empty() {
        return Flow::Done;
    }
    if cands.len() == 1 {
        huanren_apply(ctx, mach.owner, cands[0]);
        return Flow::Done;
    }
    with_boss_stage(ctx, |stage| {
        stage.pending = Some(crate::state::BossStagePending::SelectGeneral {
            candidates: cands.clone(),
        });
    });
    mach.stage = HostStage::HuanrenPick;
    Flow::WaitChoice {
        observer: mach.owner,
        spec: ChoiceSpec {
            kind: ChoiceKind::PickTag,
            prompt: HUANREN_PICK,
            options: (0..cands.len())
                .map(|i| ChoiceOption::Tag(i as u8))
                .collect(),
            min: 1,
            max: 1,
            allow_cancel: false,
        },
    }
}

pub(super) fn huanren_die_commit(ctx: &mut Ctx<'_>, mach: &HostMachine) {
    let cands = match ctx.core.boss_stage().and_then(|s| s.pending.clone()) {
        Some(crate::state::BossStagePending::SelectGeneral { candidates }) => candidates,
        None => return,
    };
    let idx = match ctx.answer().first() {
        Some(ChoiceOption::Tag(n)) => *n as usize,
        _ => 0,
    };
    let Some(&g) = cands.get(idx).or_else(|| cands.first()) else {
        return;
    };
    huanren_apply(ctx, mach.owner, g);
}

pub(super) fn wing_arm(ctx: &mut Ctx<'_>, bit: u8) {
    with_boss_stage(ctx, |stage| {
        if stage.kind == crate::state::BossStageKind::None {
            stage.kind = crate::state::BossStageKind::WingGate;
        }
        stage.wing_armed |= bit;
    });
}

pub(super) fn wing_qing(ctx: &mut Ctx<'_>, _owner: PlayerId) {
    let round = ctx.core.turn.round;
    let Some(bit) = wing_bit(0, round) else {
        return;
    };
    wing_arm(ctx, bit);
    let seats = ctx.core.players.len() as u8;
    let mut draws: Vec<(PlayerId, u32)> = Vec::new();
    match round {
        3 => {
            for t in 0..seats {
                if ctx.core.player(t).alive && ctx.core.is_boss_side(t) {
                    let p = ctx.core.player_mut(t);
                    if (p.hp as u32) < p.max_hp {
                        p.hp += 1;
                    }
                }
            }
        }
        5 => {
            for t in 0..seats {
                if ctx.core.player(t).alive
                    && !ctx.core.is_boss_side(t)
                    && ctx.core.player(t).hp > 1
                {
                    ctx.core.player_mut(t).hp -= 1;
                }
            }
        }
        7 => {
            for name in ["勾芒", "书精"] {
                if let Some(seat) = seat_with_general(ctx, name) {
                    let dead = !ctx.core.player(seat).alive;
                    {
                        let p = ctx.core.player_mut(seat);
                        p.max_hp += 1;
                        if dead {
                            p.alive = true;
                            p.dying = false;
                            p.hp = 3;
                        } else {
                            p.hp = (p.hp + 3).min(p.max_hp as i32);
                        }
                    }
                    draws.push((seat, 3));
                }
            }
        }
        _ => {}
    }
    push_draws_reversed(ctx, &draws);
}

pub(super) fn wing_chi(ctx: &mut Ctx<'_>, _owner: PlayerId) {
    let round = ctx.core.turn.round;
    let Some(bit) = wing_bit(3, round) else {
        return;
    };
    wing_arm(ctx, bit);
    let seats = ctx.core.players.len() as u8;
    match round {
        3 => {
            for t in 0..seats {
                if ctx.core.player(t).alive && !ctx.core.is_boss_side(t) {
                    grant_named_skill(ctx, t, "赤仪2");
                }
            }
        }
        5 => {
            for t in 0..seats {
                if ctx.core.player(t).alive && ctx.core.player(t).hp > 1 {
                    ctx.core.player_mut(t).hp -= 1;
                }
            }
        }
        7 => {
            if let Some(seat) = seat_with_general(ctx, "焰灵") {
                let p = ctx.core.player_mut(seat);
                p.hp -= 5;
                if p.hp < 1 {
                    p.hp = 1;
                }
            }
        }
        _ => {}
    }
}

pub(super) fn wing_bai_discard(ctx: &mut Ctx<'_>, owner: PlayerId) {
    wing_arm(ctx, WING_BAI_5);
    let seats = ctx.core.players.len() as u8;
    for t in 0..seats {
        if !ctx.core.player(t).alive || ctx.core.is_boss_side(t) {
            continue;
        }
        let _ = owner;
        forced_discard_n(ctx, t, 2);
    }
}

pub(super) fn boss_general_def(catalog: &crate::catalog::Catalog, name: &str) -> Option<u16> {
    let n = crate::content::standard::BOSS_GENERALS.len();
    let base = (catalog.general_count() - n) as u16;
    (base..catalog.general_count() as u16)
        .find(|&d| catalog.general_meta(d).map(|g| g.0) == Some(name))
}

pub(super) fn sixiang_mark_hide(
    ctx: &mut Ctx<'_>,
    boss: PlayerId,
    skill: &'static str,
    three: bool,
) {
    if let crate::state::GameMode::Boss { stage, .. } = &mut ctx.core.mode {
        stage.kind = crate::state::BossStageKind::SixiangChain;
    }
    ctx.log(crate::machine::LogEvent::SkillTriggered { skill, owner: boss });
    if three {
        let next = ctx.core.ring_next(boss);
        let prev = ctx.core.ring_prev(boss);
        for p in [next, prev] {
            if p != boss {
                ctx.log(crate::machine::LogEvent::SkillTriggered { skill, owner: p });
            }
        }
    }
}

pub(super) fn sixiang_reinit_seat(
    ctx: &mut Ctx<'_>,
    seat: PlayerId,
    general: u16,
    preserve_hp: bool,
) {
    if seat as usize >= ctx.core.players.len() {
        return;
    }
    let old_hp = ctx.core.player(seat).hp;
    let removed: Vec<crate::ids::SkillDefId> = ctx
        .core
        .skills
        .keys()
        .filter(|(o, _)| *o == seat)
        .map(|(_, d)| *d)
        .collect();
    for d in removed {
        ctx.remove_skill(seat, d);
    }
    ctx.core.player_mut(seat).general_slots[0] = Some(general);
    ctx.core.player_mut(seat).general_slots[1] = None;
    ctx.core.player_mut(seat).revealed_generals = [true, false];
    crate::state::apply_general_assembly(ctx.core, ctx.catalog, seat, 0);
    let p = ctx.core.player_mut(seat);
    if preserve_hp {
        p.hp = old_hp.min(p.max_hp as i32);
    } else {
        p.hp = p.max_hp as i32;
    }
    p.alive = true;
    p.dying = false;
    // 保留调用方已置的 death_cancelled（隐藏链转阶需拦 DeathMachine）
}

/// R05-C2：入座不再自发牌——发牌由调用方决定（上游 addFellow 不发牌：
/// apps/core/noname/game/index.js:9956-9968；addBossFellow 的
/// directgain(4)（boss.js:1689-1696）由调用方登记到 draws）。
pub(super) fn sixiang_add_fellow(ctx: &mut Ctx<'_>, name: &str) -> Option<PlayerId> {
    let g = boss_general_def(ctx.catalog, name)?;
    let hp = ctx.catalog.general_meta(g).map(|m| m.3).unwrap_or(2);
    let pid = ctx.core.players.len() as u8;
    ctx.core.players.push(crate::state::PlayerState::new(hp));
    ctx.core.seat_ring.push(pid);
    if let crate::state::GameMode::Boss { fellows, .. } = &mut ctx.core.mode {
        fellows.push(pid);
    }
    ctx.core.player_mut(pid).general_slots[0] = Some(g);
    ctx.core.player_mut(pid).revealed_generals = [true, false];
    crate::state::apply_general_assembly(ctx.core, ctx.catalog, pid, 0);
    let p = ctx.core.player_mut(pid);
    p.hp = p.max_hp as i32;
    p.alive = true;
    Some(pid)
}

pub(super) fn sixiang_mark_fellow(ctx: &mut Ctx<'_>, seat: PlayerId) {
    if let crate::state::GameMode::Boss { fellows, .. } = &mut ctx.core.mode {
        if !fellows.contains(&seat) {
            fellows.push(seat);
        }
    }
}

pub(super) fn sixiang_discard_zone(ctx: &mut Ctx<'_>, seat: PlayerId) {
    let hand: Vec<_> = ctx.core.player(seat).hand.clone();
    for c in hand {
        ctx.take_hand_card(seat, c);
        ctx.to_discard(c);
    }
    let equips: Vec<_> = ctx.core.player(seat).equips.clone();
    for c in equips {
        ctx.core.player_mut(seat).equips.retain(|x| *x != c);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, seat, c);
        ctx.to_discard(c);
    }
    let judge: Vec<_> = ctx.core.player(seat).judge.clone();
    for c in judge {
        ctx.core.player_mut(seat).judge.retain(|x| *x != c);
        ctx.to_discard(c);
    }
}

pub(super) fn sixiang_reset_challengers(ctx: &mut Ctx<'_>, draws: &mut Vec<(PlayerId, u32)>) {
    let seats = ctx.core.players.len() as u8;
    let mut dnum = 0u32;
    // 复活已死挑战者
    for t in 0..seats {
        if ctx.core.is_boss_side(t) {
            continue;
        }
        if !ctx.core.player(t).alive && ctx.core.player(t).max_hp > 0 {
            let p = ctx.core.player_mut(t);
            p.alive = true;
            p.dying = false;
            p.hp = p.max_hp as i32;
            p.boss_chongzheng = 0;
            ctx.log(crate::machine::LogEvent::BossRevived { player: t });
            dnum += 1;
        }
    }
    let draw_n = 4u32.saturating_sub(dnum);
    for t in 0..seats {
        if ctx.core.is_boss_side(t) || !ctx.core.player(t).alive {
            continue;
        }
        sixiang_discard_zone(ctx, t);
        let p = ctx.core.player_mut(t);
        p.hp = p.max_hp as i32;
        p.hujia = 0;
        p.face_down = false;
        p.linked = false;
        if draw_n > 0 {
            draws.push((t, draw_n));
        }
    }
}

pub(super) fn sixiang_run_chain(ctx: &mut Ctx<'_>, boss: PlayerId, params: &SixiangChainParams) {
    // 取消 DeathMachine 致死，由转阶重挂保活（空洞印同款基建）
    ctx.core.player_mut(boss).death_cancelled = true;
    let Some(next) = boss_general_def(ctx.catalog, params.next_boss) else {
        return;
    };
    // R05-C2：全链摸牌按同步执行序登记（换将 4 → 侧座各 4 → 新随从
    // 各 4 → 挑战者各 4-dnum），步末逆序压 R 帧保持同步取牌次序。
    let mut draws: Vec<(PlayerId, u32)> = Vec::new();
    // 登记差异：上游八座 changeSeat 观战位移不建模（seat_ring 保持）
    sixiang_reinit_seat(ctx, boss, next, false);
    // 上游 changeBoss 删旧建新+directgain(4)；原生座位保留，清区后摸 4
    sixiang_discard_zone(ctx, boss);
    draws.push((boss, 4));
    if let crate::state::GameMode::Boss { stage, .. } = &mut ctx.core.mode {
        stage.kind = crate::state::BossStageKind::SixiangChain;
        stage.level = params.chain_level;
    }
    if params.heal_all {
        let seats = ctx.core.players.len() as u8;
        for t in 0..seats {
            if ctx.core.player(t).alive {
                let p = ctx.core.player_mut(t);
                p.hp = p.max_hp as i32;
            }
        }
    }
    // 侧座 reinit 须先于 add_fellow：队尾插入会改变 ring_prev，
    // 否则白芒3 等「侧座+额外 fellow」会把新 fellow 误当 prev 盖掉。
    for &(rel, name) in params.side_reinits {
        let seat = if rel < 0 {
            ctx.core.ring_prev(boss)
        } else {
            ctx.core.ring_next(boss)
        };
        if seat == boss {
            continue;
        }
        if let Some(g) = boss_general_def(ctx.catalog, name) {
            sixiang_reinit_seat(ctx, seat, g, false);
            sixiang_discard_zone(ctx, seat);
            draws.push((seat, 4));
            sixiang_mark_fellow(ctx, seat);
        }
    }
    for name in params.add_fellows {
        if let Some(pid) = sixiang_add_fellow(ctx, name) {
            draws.push((pid, 4));
        }
    }
    sixiang_reset_challengers(ctx, &mut draws);
    // 回合环重置到 Boss（上游 phaseLoop 重锚定）
    ctx.core.turn.current = boss;
    ctx.core.turn.phase = crate::event::Phase::Judge;
    ctx.core.turn.round = 1;
    push_draws_reversed(ctx, &draws);
}
