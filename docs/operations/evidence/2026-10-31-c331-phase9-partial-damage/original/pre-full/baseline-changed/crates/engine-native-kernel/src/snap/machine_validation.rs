use super::*;

/// 机器快照树内全部 ID 的边界校验（含嵌套机器与判定进行态关系）。
pub(super) fn check_machine(
    m: &MachineSnapshot,
    b: &RestoreBounds,
    core: &GameStateCore,
) -> Result<(), SnapshotError> {
    match m {
        MachineSnapshot::TriggerOrder { owner, candidates, asked, cursor, done, .. } => {
            check_pid(*owner, b)?;
            for def in candidates { check_def(*def, b)?; }
            for (pid, def) in done { check_pid(*pid, b)?; check_def(*def, b)?; }
            if (*asked && candidates.len() < 2) || candidates.windows(2).any(|ds| ds[0] >= ds[1]) || *cursor > b.seats as u16 + 1 { return Err(SnapshotError::BadTag("trigger_order_candidates", 0)); }
        }
        MachineSnapshot::Sha {
            card,
            source,
            target,
            hujia_queue,
            hujia_holder,
            qilin_horse,
            asking_guanshi,
            guanshi_cards,
            extra_targets,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pid(*target, b)?;
            if let Some(q) = hujia_queue {
                check_pids(q, b)?;
            }
            check_opt_pid(*hujia_holder, b)?;
            if let Some(c) = qilin_horse {
                check_card(*c, b)?;
            }
            if *asking_guanshi {
                for c in guanshi_cards.iter() {
                    check_card(*c, b)?;
                }
            }
            for c in extra_targets.iter() {
                check_card(*c, b)?;
            }
        }
        MachineSnapshot::DyingSave {
            dying,
            source,
            responders,
            ..
        } => {
            check_pid(*dying, b)?;
            check_opt_pid(*source, b)?;
            check_pids(responders, b)?;
        }
        MachineSnapshot::Duel {
            card,
            source,
            target,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::NullifyWindow {
            card,
            source,
            queue,
            effect,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pids(queue, b)?;
            if let Some(e) = effect {
                check_machine(e, b, core)?;
            }
        }
        MachineSnapshot::JudgePhase {
            owner,
            queue,
            current,
        } => {
            check_pid(*owner, b)?;
            check_cards(queue, b)?;
            if let Some(c) = current {
                check_card(c.trick, b)?;
                check_card(c.judge_card, b)?;
                check_pids(&c.rewriters, b)?;
                check_pids(&c.tian_du, b)?;
                // N04-d：改判阶段 asked 与非空队列同置——asked=true 而队列
                // 已空会在恢复后应答时 panic，属非法快照。
                if c.in_rewrite && c.asked && c.rewriters.is_empty() {
                    return Err(oob("rewrite_asked_without_rewriter", 0));
                }
            }
        }
        MachineSnapshot::Noop => {}
        MachineSnapshot::PlaceDelayed { card, target, .. } => {
            check_card(*card, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::Aoe {
            card,
            source,
            targets,
            inner,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pids(targets, b)?;
            if let Some(e) = inner {
                check_machine(e, b, core)?;
            }
        }
        MachineSnapshot::AoeTarget {
            source,
            target,
            card,
            ..
        } => {
            check_pid(*source, b)?;
            check_pid(*target, b)?;
            check_card(*card, b)?;
        }
        MachineSnapshot::Chain {
            card,
            source,
            targets,
            inner,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pids(targets, b)?;
            if let Some(e) = inner {
                check_machine(e, b, core)?;
            }
        }
        MachineSnapshot::DiscardToLimit { player, .. } => check_pid(*player, b)?,
        MachineSnapshot::Death { player, killer } => {
            check_pid(*player, b)?;
            check_opt_pid(*killer, b)?;
        }
        MachineSnapshot::OptionalGate {
            owner,
            def,
            event,
            inner,
            ..
        } => {
            check_pid(*owner, b)?;
            check_def(*def, b)?;
            check_event_data(event, b)?;
            if let Some(e) = inner {
                check_machine(e, b, core)?;
            }
        }
        MachineSnapshot::SanCai { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
        }
        MachineSnapshot::YiShang { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::SuiDun => {}
        MachineSnapshot::SuQing { target } => check_pid(*target, b)?,
        MachineSnapshot::ShouYi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::FanKui { owner, source, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*source, b)?;
        }
        MachineSnapshot::JianXiong { owner, card, .. } => {
            check_pid(*owner, b)?;
            check_card(*card, b)?;
        }
        MachineSnapshot::BiYue { owner } => check_pid(*owner, b)?,
        MachineSnapshot::JiZhi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::ZhiHeng { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::KuRou { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::LuoShen { owner } => check_pid(*owner, b)?,
        MachineSnapshot::YingZi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Dismantle {
            card,
            source,
            target,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::MonkeySteal { card, user, .. } => {
            check_card(*card, b)?;
            check_pid(*user, b)?;
        }
        MachineSnapshot::XiaoJi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::QingNang { owner, healed, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*healed, b)?;
        }
        MachineSnapshot::JiJi { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
        }
        MachineSnapshot::GangLie { owner, source, .. } => {
            check_pid(*owner, b)?;
            check_pid(*source, b)?;
        }
        MachineSnapshot::XiaoGuo { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::KuangFu { owner, target } => {
            check_pid(*owner, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::YaoWu { owner, source, .. } => {
            check_pid(*owner, b)?;
            check_pid(*source, b)?;
        }
        MachineSnapshot::ShuShen { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
        }
        MachineSnapshot::ShenZhi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::XunXun { owner, top, .. } => {
            check_pid(*owner, b)?;
            for c in top {
                check_card(*c, b)?;
            }
        }
        MachineSnapshot::WangXi { owner, other } => {
            check_pid(*owner, b)?;
            check_pid(*other, b)?;
        }
        MachineSnapshot::JiJie { owner, card, .. } => {
            check_pid(*owner, b)?;
            if let Some(c) = card {
                check_card(*c, b)?;
            }
        }
        MachineSnapshot::JiYuan { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Zhangba { owner, cards, .. } => {
            check_pid(*owner, b)?;
            for c in cards {
                check_card(*c, b)?;
            }
        }
        MachineSnapshot::WangZun { owner, target } => {
            check_pid(*owner, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::TongJi { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::Wuzhong { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Taoyuan { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Wugu {
            owner,
            cards,
            pickers,
        } => {
            check_pid(*owner, b)?;
            for c in cards {
                check_card(*c, b)?;
            }
            check_pids(pickers, b)?;
        }
        MachineSnapshot::Jiedao {
            owner,
            holder,
            victim,
            ..
        } => {
            check_pid(*owner, b)?;
            check_pid(*holder, b)?;
            check_pid(*victim, b)?;
        }
        MachineSnapshot::TuXi { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::LuoYi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::GuanXing { owner, top, .. } => {
            check_pid(*owner, b)?;
            check_cards(top, b)?;
        }
        MachineSnapshot::LianYing { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Rende { owner, cards, .. } => {
            check_pid(*owner, b)?;
            check_cards(cards, b)?;
        }
        MachineSnapshot::Jijiang {
            owner,
            queue,
            helper,
            sha,
            inner,
            ..
        } => {
            check_pid(*owner, b)?;
            if let Some(q) = queue {
                check_pids(q, b)?;
            }
            check_opt_pid(*helper, b)?;
            check_opt_card(*sha, b)?;
            if let Some(e) = inner {
                check_machine(e, b, core)?;
            }
        }
        MachineSnapshot::Snatch {
            card,
            source,
            target,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::JieYin { owner, cards, .. } => {
            check_pid(*owner, b)?;
            check_cards(cards, b)?;
        }
        MachineSnapshot::FanJian { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
            // chosen_suit 为定界 tag（0..=3），非实体引用，无需界校验
        }
        MachineSnapshot::BossXiangde { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossFutai { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossLuanchang { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossZhaohuo { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossQizuo { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossXiaoxiong { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossLingbo { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossTashui { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossZjLeiji { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossHeisheng { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossLeizhu { owner, emitted, .. } => {
            check_pid(*owner, b)?;
            let _ = emitted;
        }
        MachineSnapshot::BossRenao { owner, emitted, .. } => {
            check_pid(*owner, b)?;
            let _ = emitted;
        }
        MachineSnapshot::BossWanghun { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossZhuxin { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossPingdeng { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossHuoxing { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossBingfeng { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossWangxiang { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::Rvm {
            owner,
            module,
            paused,
            skill_name,
            consume,
            commit_pending,
            commit_pick,
            following,
            commit_state,
            committing,
            ..
        } => {
            check_pid(*owner, b)?;
            if rule_vm::BytecodeModule::from_bytes(module).is_err() {
                return Err(SnapshotError::UnknownMachine);
            }
            // B7：技能身份字段不可空（空名档视为损坏拒绝）；scope 值域
            // 为编译器词表产出的 1=turn/2=round/3=game，越界视为损坏。
            if skill_name.is_empty() {
                return Err(SnapshotError::UnknownMachine);
            }
            for (_key, scope) in consume {
                if *scope > 4 {
                    return Err(SnapshotError::UnknownMachine);
                }
            }
            if let Some(p) = paused {
                if p.module_hash
                    != rule_vm::BytecodeModule::from_bytes(module)
                        .expect("checked")
                        .logic_hash
                {
                    return Err(SnapshotError::UnknownMachine);
                }
                let _ = &p.frames;
            }
            // J1（规则 v328）：提交期挂起——开窗座位界内；挂起时剩余队列
            // 必须非空且可解码（头元素即开窗效果）；未挂起时两者必须为空。
            match commit_pick {
                Some((player, _count)) => {
                    check_pid(*player, b)?;
                    if commit_pending.is_empty()
                        || crate::rvm_ops::dec_effects(commit_pending).is_none()
                    {
                        return Err(SnapshotError::UnknownMachine);
                    }
                }
                None => {
                    if !commit_pending.is_empty() && !committing {
                        return Err(SnapshotError::UnknownMachine);
                    }
                }
            }
            if *committing && crate::rvm_ops::dec_effects(commit_pending).is_none() {
                return Err(SnapshotError::UnknownMachine);
            }
            if !commit_state.is_empty() {
                if !committing {
                    return Err(SnapshotError::UnknownMachine);
                }
                let effects = crate::rvm_ops::dec_effects(commit_pending)
                    .ok_or(SnapshotError::UnknownMachine)?;
                let head = effects.first().ok_or(SnapshotError::UnknownMachine)?;
                if !(b.vm_validate)(head, commit_state, core, b.cards) {
                    return Err(SnapshotError::UnknownMachine);
                }
            }
            if let Some(p) = paused {
                if *committing || p.fuel_remaining > rule_vm::DEFAULT_FUEL {
                    return Err(SnapshotError::UnknownMachine);
                }
            }
            for next in following {
                check_machine(next, b, core)?;
            }
            if paused.is_some() && commit_pick.is_some() {
                return Err(SnapshotError::UnknownMachine);
            }
        }
        MachineSnapshot::Shedu { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossBuchun { owner, targets, .. } => {
            check_pid(*owner, b)?;
            for t in targets {
                check_pid(*t as u8, b)?;
            }
        }
        MachineSnapshot::BossXingxia { owner, actor, .. } => {
            check_pid(*owner, b)?;
            check_pid(*actor as u8, b)?;
        }
        MachineSnapshot::LiJian {
            owner, card, inner, ..
        } => {
            check_pid(*owner, b)?;
            check_opt_card(*card, b)?;
            if let Some(e) = inner {
                check_machine(e, b, core)?;
            }
        }
        MachineSnapshot::Host {
            owner,
            target,
            card,
            cards,
            targets,
            ..
        } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
            check_opt_card(*card, b)?;
            check_cards(cards, b)?;
            check_pids(targets, b)?;
        }
    }
    Ok(())
}
