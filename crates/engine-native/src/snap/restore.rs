use super::*;

// ---------------------------------------------------------------------------
// 机器重建（MachineSnapshot → Box<dyn SkillMachine>）
// ---------------------------------------------------------------------------

impl MachineSnapshot {
    /// 由机器快照重建机器实例（内核机器就地分派；内容机器——标准内容、
    /// Boss、宿主机器——经目录回调 `hooks.restore_machine` 先行重建）。
    pub fn restore(&self, catalog: &Catalog) -> Result<Box<dyn SkillMachine>, SnapshotError> {
        if let Some(m) = (catalog.hooks.restore_machine)(self, catalog) {
            return m;
        }
        match self {
            MachineSnapshot::Sha {
                card,
                source,
                target,
                nature,
                asked,
                armor_judged,
                dodges_used,
                tieji_red,
                liuli_done,
                liuli_stage,
                liuli_card,
                hujia_queue,
                hujia_holder,
                jiu_bonus,
                asking_qilin,
                qilin_horse,
                asking_hanbing,
                asking_fengxueren,
                asking_guanshi,
                guanshi_cards,
                asking_qinglong,
                asking_cixiong,
                cixiong_done,
                extra_targets,
                hanbing_pending,
                asking_xiandan,
                xiandan_done,
                xiandan_bonus,
                asking_lueying,
                lueying_victim,
                lueying_declined,
                asking_fuchou,
                fuchou_card,
                asking_sadeng,
                asking_sadeng_shan,
                shoulie_direct,
            } => Ok(Box::new(crate::kernel::ShaResolveMachine::new(
                *card,
                *source,
                *target,
                *nature,
                *asked,
                *armor_judged,
                *dodges_used,
                *tieji_red,
                *liuli_done,
                *liuli_stage,
                liuli_card.map(|c| c as crate::ids::CardId),
                hujia_queue.clone(),
                *hujia_holder,
                *jiu_bonus,
                *asking_qilin,
                qilin_horse.map(|c| c as crate::ids::CardId),
                *asking_hanbing,
                *asking_fengxueren,
                *asking_guanshi,
                guanshi_cards
                    .iter()
                    .copied()
                    .map(|c| c as crate::ids::CardId)
                    .collect(),
                *asking_qinglong,
                *asking_cixiong,
                *cixiong_done,
                extra_targets
                    .iter()
                    .copied()
                    .map(|c| c as crate::ids::PlayerId)
                    .collect(),
                *hanbing_pending,
                *asking_xiandan,
                *xiandan_done,
                *xiandan_bonus,
                *asking_lueying,
                lueying_victim.map(|c| c as crate::ids::PlayerId),
                *lueying_declined,
                *asking_fuchou,
                fuchou_card.map(|c| c as u32),
                *asking_sadeng,
                *asking_sadeng_shan,
                *shoulie_direct,
            ))),
            MachineSnapshot::DyingSave {
                dying,
                source,
                responders,
                asked,
                round_used,
                guihan_asked: _,
            } => Ok(Box::new(crate::kernel::DyingSaveMachine::new(
                *dying,
                *source,
                responders.clone(),
                *asked,
                *round_used,
            ))),
            MachineSnapshot::Duel {
                card,
                source,
                target,
                on_target,
                asked,
                second_pending,
            } => Ok(Box::new(crate::kernel::DuelMachine::new(
                *card,
                *source,
                *target,
                *on_target,
                *asked,
                *second_pending,
            ))),
            MachineSnapshot::NullifyWindow {
                card,
                source,
                nullified,
                queue,
                asked,
                window_open,
                owns_disposal,
                effect,
            } => {
                let effect = match effect {
                    Some(e) => match e.restore(catalog) {
                        Ok(m) => Some(m),
                        Err(_) => return Err(SnapshotError::UnknownMachine),
                    },
                    None => None,
                };
                Ok(Box::new(crate::kernel::NullifyWindowMachine::new(
                    *card,
                    *source,
                    *nullified,
                    queue.clone(),
                    *asked,
                    *window_open,
                    *owns_disposal,
                    effect,
                )))
            }
            MachineSnapshot::JudgePhase {
                owner,
                queue,
                current,
            } => Ok(Box::new(crate::kernel::JudgePhaseMachine::new(
                *owner,
                queue.clone(),
                current
                    .as_deref()
                    .map(crate::kernel::JudgementProgress::from_snap),
            ))),
            MachineSnapshot::Noop => Ok(Box::new(crate::kernel::NoopMachine::new())),
            MachineSnapshot::PlaceDelayed {
                card,
                target,
                placed,
            } => Ok(Box::new(crate::kernel::PlaceDelayedMachine::new(
                *card, *target, *placed,
            ))),
            MachineSnapshot::Aoe {
                card,
                source,
                need,
                targets,
                idx,
                inner,
                nature,
            } => {
                let inner = match inner {
                    Some(e) => match e.restore(catalog) {
                        Ok(m) => Some(m),
                        Err(_) => return Err(SnapshotError::UnknownMachine),
                    },
                    None => None,
                };
                Ok(Box::new(crate::kernel::AoeResolveMachine::new(
                    *card,
                    *source,
                    *need,
                    targets.clone(),
                    *idx,
                    inner,
                    *nature,
                )))
            }
            MachineSnapshot::AoeTarget {
                source,
                target,
                need,
                asked,
                card,
                nature,
            } => Ok(Box::new(crate::kernel::AoeTargetMachine::new(
                *source, *target, *need, *asked, *card, *nature,
            ))),
            MachineSnapshot::Chain {
                card,
                source,
                targets,
                idx,
                inner,
                dispatcher,
            } => {
                let inner = match inner {
                    Some(e) => match e.restore(catalog) {
                        Ok(m) => Some(m),
                        Err(_) => return Err(SnapshotError::UnknownMachine),
                    },
                    None => None,
                };
                Ok(Box::new(crate::kernel::ChainMachine::new(
                    *card,
                    *source,
                    targets.clone(),
                    *idx,
                    inner,
                    *dispatcher,
                )))
            }
            MachineSnapshot::DiscardToLimit {
                player,
                need,
                asked,
            } => Ok(Box::new(crate::kernel::DiscardToLimitMachine::new(
                *player, *need, *asked,
            ))),
            MachineSnapshot::Death { player, killer } => {
                Ok(Box::new(crate::kernel::DeathMachine::new(*player, *killer)))
            }
            MachineSnapshot::Rvm {
                owner,
                module,
                paused,
                skill_name,
                event,
                consume,
                count_hits,
                commit_pending,
                commit_pick,
            } => {
                let m = rule_vm::BytecodeModule::from_bytes(module)
                    .map_err(|_| SnapshotError::UnknownMachine)?;
                let machine = if let Some((player, count)) = commit_pick {
                    // J1（规则 v328）：提交期挂起——剩余队列+开窗参数重建，
                    // 问询由 active_ask 派生。
                    crate::rvm_bridge::VmMachine::restore_commit_paused(
                        *owner,
                        m,
                        commit_pending.clone(),
                        *player,
                        *count,
                        skill_name.clone(),
                        event.clone(),
                        consume.clone(),
                        *count_hits,
                    )
                    .map_err(|_| SnapshotError::UnknownMachine)?
                } else if let Some(mut p) = paused.clone() {
                    crate::rvm_bridge::VmMachine::restore_paused(
                        *owner,
                        m,
                        std::mem::take(&mut p.frames),
                        p.pc,
                        p.fuel_remaining,
                        p.id,
                        p.module_hash,
                        std::mem::take(&mut p.options),
                        std::mem::take(&mut p.pending),
                        skill_name.clone(),
                        event.clone(),
                        consume.clone(),
                        *count_hits,
                    )
                    .map_err(|_| SnapshotError::UnknownMachine)?
                } else {
                    crate::rvm_bridge::VmMachine::from_module(
                        *owner,
                        m,
                        skill_name.clone(),
                        event.clone(),
                        consume.clone(),
                        *count_hits,
                    )
                    .map_err(|_| SnapshotError::UnknownMachine)?
                };
                Ok(Box::new(machine))
            }
            MachineSnapshot::OptionalGate {
                owner,
                def,
                event,
                asked,
                inner,
            } => {
                let inner = match inner {
                    Some(e) => match e.restore(catalog) {
                        Ok(m) => Some(m),
                        Err(_) => return Err(SnapshotError::UnknownMachine),
                    },
                    None => None,
                };
                Ok(Box::new(crate::resolution::OptionalGate::new(
                    *owner,
                    *def,
                    event.clone(),
                    *asked,
                    inner,
                )))
            }
            _ => Err(SnapshotError::UnknownMachine),
        }
    }
}
