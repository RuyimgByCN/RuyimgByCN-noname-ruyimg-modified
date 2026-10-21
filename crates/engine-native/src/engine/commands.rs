use super::*;

impl Engine {
    /// 提交命令：单一工作副本上结算，拒绝则整段丢弃。
    ///
    /// 信任边界（校验顺序即拒绝优先级）：
    /// 1. 去重键（`DuplicateCommand`：键必须为正且严格超过已接受高水位）；
    /// 2. 修订号（`StaleRevision`）；
    /// 3. actor 座位界（`UnknownActor`）；
    /// 4. 终局/选择挂起门禁；
    /// 5. 动作授权与参数校验（`apply_command`：actor 身份、ID 界、规则合法性）。
    /// 只有 Accepted/Pending 消耗去重键；拒绝路径不消耗，可同键重试。
    /// 设置宿主时间片（None = 不限）。默认不限片：命令要么完整结算、
    /// 要么暂停于选择、要么整体拒绝/回滚。
    pub fn set_time_slice(&mut self, slice: Option<u32>) {
        self.time_slice = slice;
    }

    /// 段是否处于时间片暂停中（状态已提交部分结算，须 resume 续行）。
    pub fn is_suspended(&self) -> bool {
        self.segment_open
    }

    /// 中止开放中的段（N07 失败契约）：回滚到段启动前的快照——状态、
    /// 修订号、去重水位、宿主日志全部恢复，段关闭。适用于
    /// 「暂停段遭遇 `BudgetExhausted`」等无法续行的失败：宿主可放弃该段、
    /// 以新命令继续对局（例如提高预算或绕开导致失败的玩法）。
    /// 未处于暂停段时返回 `NotSuspended`。
    pub fn abort_segment(&mut self) -> Result<(), RejectReason> {
        if !self.segment_open {
            return Err(RejectReason::NotSuspended);
        }
        // 无检查点的开放段不可静默关闭（N07-b：仅改标志并报成功是被
        // 禁止的契约违反）；检查点随快照延续，开放段必然携带。
        let Some((snapshot, log_mark)) = self.segment_checkpoint.take() else {
            return Err(RejectReason::NotSuspended);
        };
        let restored = Engine::restore(&snapshot).map_err(|_| RejectReason::NotSuspended)?;
        self.state = restored.state;
        self.last_dedup = restored.last_dedup;
        self.log.truncate(log_mark);
        self.segment_open = false;
        Ok(())
    }

    /// 续行时间片暂停的段：以新片继续推进，直至 Accepted/Pending/Rejected。
    pub fn resume(&mut self) -> CommandOutcome {
        if !self.segment_open {
            return CommandOutcome::Rejected(RejectReason::NotSuspended);
        }
        let log_mark = self.log.len();
        let work = self.state.clone();
        self.finish_segment(work, log_mark)
    }

    pub fn submit(&mut self, cmd: GameCommand) -> CommandOutcome {
        if self.segment_open {
            return CommandOutcome::Rejected(RejectReason::SegmentSuspended);
        }
        if cmd.dedup_id == 0 || cmd.dedup_id <= self.last_dedup {
            return CommandOutcome::Rejected(RejectReason::DuplicateCommand);
        }
        if cmd.expected_revision != self.state.core.revision {
            return CommandOutcome::Rejected(RejectReason::StaleRevision);
        }
        if cmd.actor as usize >= self.state.core.players.len() {
            return CommandOutcome::Rejected(RejectReason::UnknownActor);
        }
        if self.state.core.outcome.is_some() && !matches!(cmd.action, GameAction::Answer { .. }) {
            return CommandOutcome::Rejected(RejectReason::MatchOver);
        }
        // 补偿明置（ADR 0762）：成为牌的目标时的明置窗口与挂起询问
        // 并行——RevealGeneral 不受 ChoicePending 拦截（其余命令照旧）。
        if self.state.resolution.open_choice.is_some()
            && !matches!(cmd.action, GameAction::Answer { .. })
            && !matches!(cmd.action, GameAction::RevealGeneral { .. })
        {
            return CommandOutcome::Rejected(RejectReason::ChoicePending);
        }
        // N10 有界计算池：每座位每轮配额（拒绝不消耗配额；轮次变更自动归零）
        if let Some(q) = self.seat_quota {
            let round = self.state.core.turn.round;
            let used = self
                .seat_cmd_counts
                .get(&cmd.actor)
                .and_then(|(r, c)| (*r == round).then_some(*c))
                .unwrap_or(0);
            if used >= q {
                return CommandOutcome::Rejected(RejectReason::QuotaExhausted);
            }
        }
        // 补偿明置并行提交（ADR 0762）：挂起询问在场时，只应用明置的
        // 状态变更、不推进结算——原询问保持挂起，结算轨迹不变。
        if matches!(cmd.action, GameAction::RevealGeneral { .. })
            && self.state.resolution.open_choice.is_some()
        {
            let log_mark = self.log.len();
            let mut work = self.state.clone();
            if let Err(reason) = apply_command(&mut work, cmd.clone(), &self.catalog, &mut self.log)
            {
                self.log.truncate(log_mark);
                return CommandOutcome::Rejected(reason);
            }
            work.core.revision += 1;
            let choice = work
                .resolution
                .open_choice
                .clone()
                .expect("parallel reveal keeps open choice");
            self.state = work;
            self.segment_open = false;
            self.segment_checkpoint = None;
            self.last_dedup = cmd.dedup_id;
            if self.seat_quota.is_some() {
                let entry = self
                    .seat_cmd_counts
                    .entry(cmd.actor)
                    .or_insert((self.state.core.turn.round, 0));
                entry.1 += 1;
            }
            return CommandOutcome::Pending { choice };
        }
        let log_mark = self.log.len();
        let submit_round = self.state.core.turn.round;
        let mut work = self.state.clone();
        // 每段重置预算：硬上限只约束本段，不跨段累计（ADR 0708 缺陷修复）。
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        // 段检查点：供开放段失败时 `abort_segment` 回滚到段前（N07）。
        let checkpoint = self.save();
        if let Err(reason) = apply_command(&mut work, cmd.clone(), &self.catalog, &mut self.log) {
            self.log.truncate(log_mark);
            return CommandOutcome::Rejected(reason);
        }
        match self.finish_segment(work, log_mark) {
            CommandOutcome::Rejected(reason) => {
                self.segment_checkpoint = None;
                CommandOutcome::Rejected(reason)
            }
            outcome => {
                // 命令已被接受（含暂停于选择的 Pending）：消耗去重键，
                // 并计入该座位本轮配额（按提交时的轮次记账）。
                self.last_dedup = cmd.dedup_id;
                if self.seat_quota.is_some() {
                    let entry = self
                        .seat_cmd_counts
                        .entry(cmd.actor)
                        .or_insert((submit_round, 0));
                    if entry.0 == submit_round {
                        entry.1 += 1;
                    } else {
                        *entry = (submit_round, 1);
                    }
                }
                match outcome {
                    CommandOutcome::Suspended => {
                        // 段开放：保留检查点供失败中止（N07）。
                        self.segment_checkpoint = Some((checkpoint, log_mark));
                    }
                    _ => self.segment_checkpoint = None,
                }
                outcome
            }
        }
    }

    /// 结算工作状态并提交/回滚（submit/resume/调试入口共用）。
    ///
    /// 时间片语义（ADR 0707）：`Yielded` 时状态在步边界提交、宿主日志保留，
    /// 段保持开放等待续行；已分片提交过的段无法再整段回滚，故开放段上的
    /// 硬预算耗尽同样提交当前进度并返回 `Rejected(BudgetExhausted)`——
    /// 段会确定性地重复耗尽，宿主应放弃该段（原型契约，见进度文档）。
    pub(super) fn finish_segment(
        &mut self,
        mut work: GameState,
        log_mark: usize,
    ) -> CommandOutcome {
        match run(
            &mut work.core,
            &mut work.resolution,
            &self.catalog,
            &mut self.log,
            self.time_slice,
        ) {
            StepOutcome::BudgetExhausted => {
                if self.segment_open {
                    // 开放段（已分片提交过）：无法整段回滚，提交当前进度，
                    // 段保持开放——宿主应 `abort_segment()` 回滚到段前。
                    self.state = work;
                } else {
                    self.log.truncate(log_mark);
                    self.segment_checkpoint = None;
                }
                CommandOutcome::Rejected(RejectReason::BudgetExhausted)
            }
            StepOutcome::Pending => {
                work.core.revision += 1;
                self.state = work;
                self.segment_open = false;
                self.segment_checkpoint = None;
                let choice = self
                    .state
                    .resolution
                    .open_choice
                    .clone()
                    .expect("pending implies choice");
                CommandOutcome::Pending { choice }
            }
            StepOutcome::Yielded => {
                // 暂停不递增 revision：它是同一状态版本的执行延续，
                // 保证「分片续行」与「连续运行」的摘要轨迹完全一致（N07）。
                // （暂停态的内容仍随预算/执行位置变化而哈希不同。）
                self.state = work;
                self.segment_open = true;
                CommandOutcome::Suspended
            }
            StepOutcome::Fault => {
                // NC04：VM 故障——整段回滚（work 丢弃、日志截断、检查点清除），
                // 命令以 VmFault 拒绝；开放段则回滚到段前检查点。
                let fault = work.resolution.fault.take();
                self.log.truncate(log_mark);
                if self.segment_open {
                    if let Some((checkpoint, _)) = self.segment_checkpoint.take() {
                        if let Ok(restored) = Engine::restore_with_catalog(
                            &checkpoint,
                            self.catalog.clone(),
                        ) {
                            self.state = restored.state;
                        }
                    }
                }
                self.segment_open = false;
                self.segment_checkpoint = None;
                let (skill, detail) = match &fault {
                    Some(f) => (
                        f.skill.clone(),
                        format!("{}: {}（事件 {}）", f.kind, f.detail, f.event),
                    ),
                    None => (String::new(), "未知 VM 故障".to_string()),
                };
                CommandOutcome::Rejected(RejectReason::VmFault { skill, detail })
            }
            StepOutcome::Done => {
                work.core.revision += 1;
                self.state = work;
                self.segment_open = false;
                self.segment_checkpoint = None;
                CommandOutcome::Accepted {
                    state_hash: canonical_hash(&self.state),
                }
            }
        }
    }
}
