use super::*;
use crate::event::Stage;

/// 由目录与座位数导出恢复边界。
pub fn bounds_from(catalog: &Catalog, seats: usize) -> RestoreBounds {
    RestoreBounds {
        vm_validate: catalog.hooks.vm_validate,
        seats,
        cards: catalog.cards.len(),
        skills: catalog.skills.len(),
        generals: catalog.general_count(),
        ruleset_version: catalog.ruleset_version,
    }
}

pub(super) fn oob(what: &'static str, got: u64) -> SnapshotError {
    SnapshotError::IdOutOfBounds { what, got }
}

pub(super) fn check_pid(p: PlayerId, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if p as usize >= b.seats {
        return Err(oob("player", p as u64));
    }
    Ok(())
}

pub(super) fn check_card(c: CardId, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if c as usize >= b.cards {
        return Err(oob("card", c as u64));
    }
    Ok(())
}

pub(super) fn check_def(d: SkillDefId, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if d as usize >= b.skills {
        return Err(oob("skill_def", d as u64));
    }
    Ok(())
}

pub(super) fn check_opt_pid(p: Option<PlayerId>, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if let Some(p) = p {
        check_pid(p, b)?;
    }
    Ok(())
}

pub(super) fn check_opt_card(c: Option<CardId>, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if let Some(c) = c {
        check_card(c, b)?;
    }
    Ok(())
}

pub(super) fn check_pids(ps: &[PlayerId], b: &RestoreBounds) -> Result<(), SnapshotError> {
    for p in ps {
        check_pid(*p, b)?;
    }
    Ok(())
}

pub(super) fn check_cards(cs: &[CardId], b: &RestoreBounds) -> Result<(), SnapshotError> {
    for c in cs {
        check_card(*c, b)?;
    }
    Ok(())
}

pub(super) fn check_event_data(data: &EventData, b: &RestoreBounds) -> Result<(), SnapshotError> {
    match data {
        EventData::OrdinaryJudge { owner, .. } => check_pid(*owner, b)?,
        EventData::SkillInvocation(d) => {
            check_pid(d.owner, b)?;
            check_def(d.def, b)?;
            check_opt_pid(d.source, b)?;
            check_pid(d.target, b)?;
        }
        EventData::SkillLog { owner, def, .. } => {
            check_pid(*owner, b)?;
            check_def(*def, b)?;
        }
        EventData::DamageWindow { .. } => {}
        EventData::Damage(d) => {
            super::damage_validation::check_settlement(d, b)?;
            check_opt_pid(d.source, b)?;
            check_pid(d.target, b)?;
            check_pids(&d.redirect_chain, b)?;
            check_opt_card(d.card, b)?;
        }
        EventData::ShaDodged {
            card,
            source,
            target,
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pid(*target, b)?;
        }
        EventData::CardsGiven {
            giver,
            recipient,
            cards,
        } => {
            check_pid(*giver, b)?;
            check_pid(*recipient, b)?;
            check_cards(cards, b)?;
        }
        EventData::CardsLost {
            player,
            cards,
            origins,
        } => {
            check_pid(*player, b)?;
            check_cards(cards, b)?;
            if cards.len() != origins.len() {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "cards_lost_origins",
                    got: origins.len() as u64,
                });
            }
        }
        EventData::LoseHp { player, .. } => {
            check_pid(*player, b)?;
        }
        EventData::Recover {
            player, by_card, ..
        } => {
            check_pid(*player, b)?;
            check_opt_card(*by_card, b)?;
        }
        EventData::CardsDrawn { player, cards } => {
            check_pid(*player, b)?;
            check_cards(cards, b)?;
        }
        EventData::CardsMoved { cards, from, to } => {
            check_cards(cards, b)?;
            check_pid(*from, b)?;
            check_pid(*to, b)?;
        }
        EventData::CardUsed(u) => {
            check_card(u.card, b)?;
            check_pid(u.player, b)?;
            check_pids(&u.targets, b)?;
        }
        EventData::CardResponded { card, player } => {
            check_card(*card, b)?;
            check_pid(*player, b)?;
        }
        EventData::Dying { player, source } => {
            check_pid(*player, b)?;
            check_opt_pid(*source, b)?;
        }
        EventData::DyingSaved { player, source } => {
            check_pid(*player, b)?;
            check_opt_pid(*source, b)?;
        }
        EventData::Death { player, killer } => {
            check_pid(*player, b)?;
            check_opt_pid(*killer, b)?;
        }
        EventData::Phase { player, .. } => check_pid(*player, b)?,
        EventData::TurnEnded { player } => check_pid(*player, b)?,
        EventData::SkillToggled { player, skill, .. } => {
            check_pid(*player, b)?;
            check_def(*skill, b)?;
        }
        EventData::Equipped {
            player,
            card,
            replaced,
        } => {
            check_pid(*player, b)?;
            check_card(*card, b)?;
            check_opt_card(*replaced, b)?;
        }
        EventData::Linked { player, .. } => check_pid(*player, b)?,
        EventData::CardsDiscarded { player, cards } => {
            check_pid(*player, b)?;
            check_cards(cards, b)?;
        }
        EventData::TurnOverBefore { player, .. } => {
            check_pid(*player, b)?;
        }
        EventData::TurnOvered { player, .. } => {
            check_pid(*player, b)?;
        }
    }
    Ok(())
}

pub(super) fn check_options(opts: &[ChoiceOption], b: &RestoreBounds) -> Result<(), SnapshotError> {
    for o in opts {
        match o {
            ChoiceOption::Card(c) => check_card(*c, b)?,
            ChoiceOption::Control(_) => {}
            ChoiceOption::Skill(d) => check_def(*d, b)?,
            ChoiceOption::Player(p) => check_pid(*p, b)?,
            _ => {}
        }
    }
    Ok(())
}

/// 核心状态校验：规则版本绑定、牌区/技能实例 ID 界、身份局结构、
/// 武将槽界、胜负与模式一致性。
pub fn validate_core(core: &GameStateCore, b: &RestoreBounds) -> Result<(), SnapshotError> {
    // 规则版本（核心副本与目录绑定；顶层一致不豁免核心篡改）
    if core.ruleset_version != b.ruleset_version {
        return Err(SnapshotError::RuleVersionMismatch {
            snapshot: core.ruleset_version,
            catalog: b.ruleset_version,
        });
    }
    // NC07：座位环引用校验（长度须与玩家数一致，取值须为合法座位且构成排列）
    if !core.seat_ring.is_empty() {
        if core.seat_ring.len() != core.players.len() {
            return Err(oob("seat_ring_len", core.seat_ring.len() as u64));
        }
        let mut seen = vec![false; core.players.len()];
        for seat in &core.seat_ring {
            if *seat as usize >= core.players.len() {
                return Err(oob("seat_ring_seat", *seat as u64));
            }
            if seen[*seat as usize] {
                return Err(oob("seat_ring_duplicate", *seat as u64));
            }
            seen[*seat as usize] = true;
        }
    }

    // 牌区 ID 界、国战武将槽界、五行属性界
    for p in &core.players {
        check_cards(&p.hand, b)?;
        check_cards(&p.equips, b)?;
        check_cards(&p.judge, b)?;
        check_cards(&p.off_court, b)?;
        for g in p.general_slots.iter().flatten() {
            if *g as usize >= b.generals {
                return Err(oob("general_slot_index", *g as u64));
            }
        }
        // 明置必须引用已配置的武将槽
        for slot in 0..2 {
            if p.revealed_generals[slot] && p.general_slots[slot].is_none() {
                return Err(oob("reveal_without_general", 0));
            }
        }
        if let Some(n) = p.wunature {
            if n > 4 {
                return Err(oob("wunature", n as u64));
            }
        }
    }
    // 回合当前玩家在界
    if core.turn.current as usize >= core.players.len() {
        return Err(oob("player", core.turn.current as u64));
    }
    // 技能实例：键在界、值 owner 与键一致
    for ((owner, def), inst) in &core.skills {
        check_pid(*owner, b)?;
        check_def(*def, b)?;
        if inst.owner != *owner {
            return Err(oob("skill_instance_owner", inst.owner as u64));
        }
    }
    // 对决模式结构：两队互斥、并集覆盖全部座位
    if let GameMode::Versus { teams, substitutes } = &core.mode {
        if teams.len() != 2 {
            return Err(oob("versus_team_count", teams.len() as u64));
        }
        let mut seen = vec![false; b.seats];
        for t in teams {
            for p in t {
                if (*p as usize) >= b.seats || seen[*p as usize] {
                    return Err(oob("versus_team_member", *p as u64));
                }
                seen[*p as usize] = true;
            }
        }
        if seen.iter().any(|s| !s) {
            return Err(oob("versus_team_coverage", 0));
        }
        // 替补池（v206）：武将 def 界校验（池数与队数一致，条目为目录将）
        if substitutes.len() != teams.len() {
            return Err(oob("versus_substitute_pools", substitutes.len() as u64));
        }
        for pool in substitutes {
            for g in pool {
                check_def(*g, b)?;
            }
        }
    }
    // Boss 结构：boss/助战座位有效
    if let GameMode::Boss { boss, fellows, .. } = &core.mode {
        if (*boss as usize) >= b.seats {
            return Err(oob("boss_seat", *boss as u64));
        }
        for f in fellows {
            if (*f as usize) >= b.seats {
                return Err(oob("boss_fellow_seat", *f as u64));
            }
        }
    }
    // KOF 结构：2 人局（会话将池内容为 GENERAL 下标，越界由运行期授予校验）
    if let GameMode::Kof { .. } = &core.mode {
        if b.seats != 2 {
            return Err(oob("kof_seats", b.seats as u64));
        }
    }
    // 斗地主结构：地主座位有效
    if let GameMode::Doudizhu { lord } = &core.mode {
        if (*lord as usize) >= b.seats {
            return Err(oob("doudizhu_lord", *lord as u64));
        }
    }
    // 身份局结构
    if let GameMode::Identity { roles } = &core.mode {
        if roles.len() != b.seats {
            return Err(oob("roles_len", roles.len() as u64));
        }
        if roles.first() != Some(&crate::state::Role::Lord) {
            return Err(oob("lord_seat", 0));
        }
        if !roles.contains(&crate::state::Role::Traitor) {
            return Err(oob("roles_composition", 0));
        }
        // 人数表：4–6 人局 1 忠，7–8 人局 2 忠
        let loyalists = if (4..=6).contains(&b.seats) { 1 } else { 2 };
        let got = roles
            .iter()
            .filter(|r| **r == crate::state::Role::Loyalist)
            .count();
        if got != loyalists {
            return Err(oob("roles_loyalist_count", got as u64));
        }
    }
    // 胜负与模式一致性
    if let Some(Outcome { winner }) = &core.outcome {
        let ok = match (&core.mode, winner) {
            (GameMode::Prototype, Winner::Player(_)) => true,
            (GameMode::Identity { .. }, Winner::LordSide | Winner::Rebels | Winner::Traitor) => {
                true
            }
            (GameMode::Kingdoms, Winner::Faction(_)) => true,
            (GameMode::Versus { .. }, Winner::Versus(_)) => true,
            (GameMode::Doudizhu { .. }, Winner::Doudizhu { .. }) => true,
            (GameMode::Boss { .. }, Winner::Boss { .. }) => true,
            (GameMode::Kof { .. }, Winner::Player(_)) => true,
            _ => false,
        };
        if !ok {
            return Err(oob("outcome_mode_mismatch", 0));
        }
    }
    Ok(())
}

/// 机器持牌收集（N04-f：判定进行态/观星顶牌由机器持有，不在任何牌区）。
pub fn machine_held_cards(m: &MachineSnapshot, out: &mut Vec<CardId>) {
    if let MachineSnapshot::JudgeContinuation {
        payment: Some(p), ..
    } = m
    {
        out.push(p.old);
    }
    match m {
        MachineSnapshot::JudgeContinuation { queue, current, .. }
        | MachineSnapshot::JudgePhase { queue, current, .. } => {
            out.extend(queue.iter().copied());
            if let Some(c) = current {
                // finalized（效果已套用）后锦囊已离机，trick 仅为簿记
                if !c.finalized {
                    out.extend(c.trick);
                }
                if c.judge_held {
                    if !c.finalized {
                        if let Some(cards) = &c.gaiming_cards {
                            out.extend(cards.iter().copied());
                            // Only the first candidate aliases judge_card; other duplicates remain errors.
                            if cards.first() != Some(&c.judge_card) {
                                out.push(c.judge_card);
                            }
                        } else {
                            out.push(c.judge_card);
                        }
                        if c.haotian_holder.is_some() {
                            out.extend(c.haotian_other);
                        }
                    } else {
                        out.push(c.judge_card);
                    }
                }
            }
        }
        MachineSnapshot::XunXun {
            stage: XunXunStageSnap::Arrange,
            top,
            ..
        } => {
            out.extend(top.iter().copied());
        }
        MachineSnapshot::JiJie {
            stage: JiJieStageSnap::Read,
            card,
            ..
        } => {
            out.extend(card);
        }
        MachineSnapshot::GuanXing { top, .. } => out.extend(top.iter().copied()),
        // 末焱展示牌已离开牌库，挂起期间只在宿主机器上。
        MachineSnapshot::Host {
            behavior, cards, ..
        } if *behavior == HostBehavior::D3Moyan.tag() => {
            out.extend(cards.iter().copied());
        }
        MachineSnapshot::OptionalGate { inner, .. }
        | MachineSnapshot::NullifyWindow { effect: inner, .. }
        | MachineSnapshot::Aoe { inner, .. }
        | MachineSnapshot::Jijiang { inner, .. }
        | MachineSnapshot::LiJian { inner, .. } => {
            if let Some(m) = inner {
                machine_held_cards(m, out);
            }
        }
        _ => {}
    }
}

/// N04-f 牌归属多重集核算：目录全集 ⊆ 恰一牌区 ∪ 机器暂离。
/// 尾端缺失 / 中段缺失 / 全量缺失 / 重复归属一律拒绝。
pub fn validate_card_ownership(
    core: &GameStateCore,
    resolution: &ResolutionSnapshot,
    b: &RestoreBounds,
) -> Result<(), SnapshotError> {
    let mut seen = vec![false; b.cards];
    fn mark(seen: &mut [bool], b: &RestoreBounds, c: CardId) -> Result<(), SnapshotError> {
        check_card(c, b)?;
        if seen[c as usize] {
            return Err(oob("card_in_multiple_zones", c as u64));
        }
        seen[c as usize] = true;
        Ok(())
    }
    let mut held = Vec::new();
    for park in &resolution.parks {
        machine_held_cards(&park.machine, &mut held);
    }
    for card in physical_cards(core, &held) {
        mark(&mut seen, b, card.id)?;
    }
    for (id, ok) in seen.iter().enumerate() {
        if !ok {
            return Err(oob("card_missing_from_all_zones", id as u64));
        }
    }
    Ok(())
}

/// 执行位置校验：帧父子链 / 帧号水位与去重 / 挂起深度与槽位帧引用 /
/// 选择应答 ID 水位 / 机器树 ID 界。
pub fn validate_resolution(
    resolution: &ResolutionSnapshot,
    b: &RestoreBounds,
    core: &GameStateCore,
) -> Result<(), SnapshotError> {
    if resolution.open_choice.is_some() && resolution.parks.is_empty() {
        return Err(oob("choice_without_machine", 0));
    }
    // 栈帧：唯一性、水位、父子链（父必须存在且是更早的帧）
    let mut ids: std::collections::BTreeMap<u32, usize> = Default::default();
    for (i, f) in resolution.stack.iter().enumerate() {
        if f.id as u64 >= resolution.next_event_id as u64 {
            return Err(oob("event_id", f.id as u64));
        }
        if ids.insert(f.id, i).is_some() {
            return Err(oob("event_id_dup", f.id as u64));
        }
        if let Some(p) = f.parent {
            match ids.get(&p) {
                Some(&pi) if pi < i => {}
                _ => return Err(oob("frame_parent", p as u64)),
            }
        }
        check_event_data(&f.data, b)?;
        if matches!(
            f.data,
            EventData::Damage(_)
                | EventData::DamageWindow { .. }
                | EventData::SkillInvocation(_)
                | EventData::SkillLog { .. }
        ) && f.kind != f.data.kind()
        {
            return Err(SnapshotError::BadTag("damage_frame_kind", 0));
        }
        super::damage_validation::check_frame(f, resolution)?;
        for q in &f.queue {
            check_def(q.def, b)?;
            check_pid(q.owner, b)?;
        }
    }
    // 挂起：深度与槽位帧引用、机器树 ID 界
    for p in &resolution.parks {
        if p.resume_depth > resolution.stack.len() {
            return Err(oob("resume_depth", p.resume_depth as u64));
        }
        let frame = match p.slot {
            Slot::Intrinsic { frame } => frame,
            Slot::Trigger { frame } => frame,
        };
        // 会话级挂起槽（NO_FRAME 哨兵）豁免帧存在性检查
        if frame != NO_FRAME && !ids.contains_key(&frame) {
            return Err(oob("park_frame", frame as u64));
        }
        check_machine(&p.machine, b, core)?;
        if let MachineSnapshot::DamageFlow { root } = p.machine {
            if !matches!(p.slot,Slot::Intrinsic{frame}if frame==root)
                || !resolution.stack.iter().enumerate().any(|(i, f)| {
                    f.id == root
                        && p.resume_depth == i + 1
                        && f.stage == Stage::Intrinsic
                        && matches!(f.data, EventData::Damage(_))
                })
            {
                return Err(SnapshotError::BadTag("damage_flow_binding", root as u64));
            }
        }
    }
    // 应答选项 ID 界
    if let Some(ans) = &resolution.answer {
        check_options(ans, b)?;
    }
    // 开放选择：观察者/选项在界、ID 水位、必须有承接机器
    if let Some(open) = &resolution.open_choice {
        check_pid(open.observer, b)?;
        check_options(&open.spec.options, b)?;
        if open.id == 0 || open.id >= resolution.next_choice_id {
            return Err(oob("choice_id", open.id));
        }
        if resolution.parks.is_empty() {
            return Err(oob("choice_without_machine", 0));
        }
    }
    Ok(())
}
