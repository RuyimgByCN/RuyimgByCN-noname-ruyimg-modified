//! Resolution transaction segment with undo journal + baseline rollback.
//!
//! Architecture §6.2: mutations buffer until commit; on fault, roll back the
//! current segment only. Phase 2 records a compact journal for diagnostics and
//! property tests; authoritative rollback restores the baseline clone (equivalent
//! to replaying inverse ops).

use crate::hash::canonical_state_hash;
use crate::state::GameState;
use engine_api::{CardId, ChoiceRequest, DomainEvent, PlayerId, ZoneId};
use serde::{Deserialize, Serialize};

/// Compact inverse-friendly journal entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalEntry {
    /// Player HP changed.
    PlayerHp {
        /// Player.
        id: PlayerId,
        /// Previous HP.
        prev: i32,
    },
    /// Player max HP changed.
    PlayerMaxHp {
        /// Player.
        id: PlayerId,
        /// Previous max.
        prev: u32,
    },
    /// Alive / dying flags.
    PlayerLife {
        /// Player.
        id: PlayerId,
        /// Previous alive.
        prev_alive: bool,
        /// Previous dying.
        prev_dying: bool,
    },
    /// Card moved between zones.
    CardMove {
        /// Card.
        card: CardId,
        /// Previous zone.
        prev_zone: ZoneId,
        /// Previous owner.
        prev_owner: PlayerId,
        /// Previous face_up.
        prev_face_up: bool,
        /// Index within previous zone list before removal.
        prev_index: u32,
    },
    /// Marks vector replaced.
    PlayerMarks {
        /// Player.
        id: PlayerId,
        /// Previous marks.
        prev: Vec<(u32, i32)>,
    },
    /// Resolution blob checkpoint (choice / budgets / continuation).
    ResolutionCheckpoint,
    /// Turn state checkpoint.
    TurnCheckpoint,
    /// Outcome checkpoint.
    OutcomeCheckpoint,
    /// RNG checkpoint.
    RngCheckpoint,
    /// Generic opaque step marker (tests / content).
    Marker {
        /// Code.
        code: u32,
    },
}

/// Buffered outputs of a resolution segment.
#[derive(Debug, Clone, Default)]
pub struct SegmentOutput {
    /// Domain events in order.
    pub events: Vec<DomainEvent>,
    /// Choice request if the segment paused.
    pub choice: Option<ChoiceRequest>,
}

/// Open transaction: mutates a working copy; commit or rollback.
#[derive(Debug, Clone)]
pub struct Transaction {
    /// State at segment start (rollback target).
    baseline: GameState,
    /// Working state.
    working: GameState,
    /// Undo journal (diagnostic + future journal-only rollback).
    journal: Vec<JournalEntry>,
    /// Buffered output.
    output: SegmentOutput,
    /// Committed flag.
    committed: bool,
}

impl Transaction {
    /// Begin a segment on `state`.
    ///
    /// Resets per-segment effect/trigger budgets on the working copy.
    pub fn begin(state: GameState) -> Self {
        let mut working = state.clone();
        working.resolution.reset_segment_budgets();
        Self {
            baseline: state,
            working,
            journal: Vec::new(),
            output: SegmentOutput::default(),
            committed: false,
        }
    }

    /// Working state (mutable).
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.working
    }

    /// Working state (shared).
    pub fn state(&self) -> &GameState {
        &self.working
    }

    /// Record a journal entry before mutating.
    pub fn journal(&mut self, entry: JournalEntry) {
        self.journal.push(entry);
    }

    /// Journal length (tests / diagnostics).
    pub fn journal_len(&self) -> usize {
        self.journal.len()
    }

    /// Borrow journal.
    pub fn journal_entries(&self) -> &[JournalEntry] {
        &self.journal
    }

    /// Push domain event.
    pub fn push_event(&mut self, event: DomainEvent) {
        self.output.events.push(event);
    }

    /// Set choice barrier (also mirrors into working resolution).
    pub fn set_choice(&mut self, choice: ChoiceRequest) {
        self.output.choice = Some(choice);
    }

    /// Whether a choice was buffered this segment.
    pub fn has_buffered_choice(&self) -> bool {
        self.output.choice.is_some() || self.working.has_pending_choice()
    }

    /// Commit: bump revision, normalize budgets, return new state + output + hash.
    pub fn commit(mut self) -> CommitResult {
        if self.committed {
            panic!("double commit");
        }
        self.committed = true;
        self.working.revision = self.working.revision.saturating_add(1);
        // Budgets are per-segment; published state stores max remaining for next open.
        self.working.resolution.reset_segment_budgets();
        // Stamp choice revision to post-commit revision when a choice is open.
        if self.working.resolution.pending_choice_id != 0 {
            self.working.resolution.choice_revision = self.working.revision;
            if let Some(ref mut c) = self.output.choice {
                c.revision = self.working.revision;
            }
        }
        let hash = canonical_state_hash(&self.working);
        CommitResult {
            state: self.working,
            output: self.output,
            state_hash: hash,
            journal_len: self.journal.len(),
        }
    }

    /// Rollback to baseline (discard working mutations and journal).
    pub fn rollback(self) -> GameState {
        self.baseline
    }

    /// Baseline hash (for reject paths).
    pub fn baseline_hash(&self) -> [u8; 32] {
        canonical_state_hash(&self.baseline)
    }

    /// Journal-only rollback（ADR 0703 第②部分最小实现）：
    /// 逆序撤销 journal 中**可精确撤销**的条目，替代整份基线克隆。
    ///
    /// 当前可精确撤销：`PlayerHp` / `PlayerMaxHp` / `PlayerLife` /
    /// `PlayerMarks`。`CardMove` 与 4 个 Checkpoint 变体缺少可重放的
    /// 前值信息，返回错误交由调用方回退 `rollback()`（完整基线）——
    /// 完备性矩阵与扩面计划见
    /// docs/migration/content-registry-arc-split.md。
    pub fn rollback_via_journal(
        mut self,
    ) -> Result<GameState, JournalRollbackError> {
        if self.committed {
            panic!("double rollback");
        }
        self.committed = true;
        let entries = std::mem::take(&mut self.journal);
        for entry in entries.iter().rev() {
            match entry {
                JournalEntry::PlayerHp { id, prev } => {
                    let Some(p) = self.working.players.get_mut(*id) else {
                        return Err(JournalRollbackError::MissingEntity);
                    };
                    p.hp = *prev;
                }
                JournalEntry::PlayerMaxHp { id, prev } => {
                    let Some(p) = self.working.players.get_mut(*id) else {
                        return Err(JournalRollbackError::MissingEntity);
                    };
                    p.max_hp = *prev;
                }
                JournalEntry::PlayerLife {
                    id,
                    prev_alive,
                    prev_dying,
                } => {
                    let Some(p) = self.working.players.get_mut(*id) else {
                        return Err(JournalRollbackError::MissingEntity);
                    };
                    p.alive = *prev_alive;
                    p.dying = *prev_dying;
                }
                JournalEntry::PlayerMarks { id, prev } => {
                    let Some(p) = self.working.players.get_mut(*id) else {
                        return Err(JournalRollbackError::MissingEntity);
                    };
                    p.marks = prev.clone();
                }
                JournalEntry::CardMove {
                    card,
                    prev_zone,
                    prev_owner,
                    prev_face_up,
                    prev_index,
                } => {
                    // 精确撤销：从当前区域列表移除，恢复卡字段后按
                    // prev_index 插回原区域列表原位置。
                    let Some(c) = self.working.cards.get_mut(*card) else {
                        return Err(JournalRollbackError::MissingEntity);
                    };
                    let cur_zone = c.zone;
                    if !self.working.zones.remove_card(cur_zone, *card) {
                        return Err(JournalRollbackError::MissingEntity);
                    }
                    c.zone = *prev_zone;
                    c.owner = *prev_owner;
                    c.face_up = *prev_face_up;
                    let Some(z) = self.working.zones.get_mut(*prev_zone) else {
                        return Err(JournalRollbackError::MissingEntity);
                    };
                    let idx = (*prev_index as usize).min(z.cards.len());
                    z.cards.insert(idx, *card);
                }
                JournalEntry::ResolutionCheckpoint => {
                    return Err(JournalRollbackError::CheckpointEntry("resolution"));
                }
                JournalEntry::TurnCheckpoint => {
                    return Err(JournalRollbackError::CheckpointEntry("turn"));
                }
                JournalEntry::OutcomeCheckpoint => {
                    return Err(JournalRollbackError::CheckpointEntry("outcome"));
                }
                JournalEntry::RngCheckpoint => {
                    return Err(JournalRollbackError::CheckpointEntry("rng"));
                }
                JournalEntry::Marker { .. } => {}
            }
        }
        Ok(self.working)
    }
}

/// Journal-only rollback 的失败原因（调用方应回退完整基线回滚）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalRollbackError {
    /// Checkpoint 式条目未携带前值快照，无法重放。
    CheckpointEntry(&'static str),
    /// 卡牌移动撤销需要区列表恢复，尚未支持。
    CardMoveUndo,
    /// 条目引用的玩家/牌不存在（journal 与状态失配）。
    MissingEntity,
}

/// Successful commit payload.
#[derive(Debug, Clone)]
pub struct CommitResult {
    /// New authority state.
    pub state: GameState,
    /// Buffered outputs.
    pub output: SegmentOutput,
    /// Canonical hash after commit.
    pub state_hash: [u8; 32],
    /// How many journal entries were recorded (diagnostics).
    pub journal_len: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameState;

    #[test]
    fn rollback_restores_hash() {
        let base = GameState::empty();
        let h0 = canonical_state_hash(&base);
        let mut tx = Transaction::begin(base);
        tx.state_mut().revision = 99;
        tx.journal(JournalEntry::Marker { code: 1 });
        let restored = tx.rollback();
        assert_eq!(canonical_state_hash(&restored), h0);
        assert_eq!(restored.revision, 0);
    }
}

#[cfg(test)]
mod journal_equivalence_tests {
    use super::*;
    use crate::engine::Engine;
    use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey};

    fn started_engine() -> Engine {
        let mut e = Engine::new();
        e.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 42,
                ruleset: "identity-standard".into(),
            },
        });
        e
    }

    /// 对已被 journal 覆盖的字段（hp / marks / life），撤销日志重放与
    /// 完整基线回滚**必须等价**（同一终态哈希）——撤销日志替代
    /// 整份工作副本的等价性验收（ADR 0703 第②部分）。
    #[test]
    fn journal_rollback_equivalent_to_baseline_for_tracked_fields() {
        let e = started_engine();
        let state0 = e.state().clone();
        let p1 = PlayerId::new(1);
        let prev_hp = state0.players.get(p1).unwrap().hp;
        let prev_marks = state0.players.get(p1).unwrap().marks.clone();

        // 路径 A：完整基线回滚
        let mut tx1 = Transaction::begin(state0.clone());
        tx1.state_mut().players.get_mut(p1).unwrap().hp -= 1;
        tx1.journal(JournalEntry::PlayerHp {
            id: p1,
            prev: prev_hp,
        });
        tx1.state_mut().players.get_mut(p1).unwrap().marks = vec![(7, 3)];
        tx1.journal(JournalEntry::PlayerMarks {
            id: p1,
            prev: prev_marks.clone(),
        });
        let base = tx1.rollback();
        let h_base = canonical_state_hash(&base);

        // 路径 B：journal-only 回滚（不使用基线克隆）
        let mut tx2 = Transaction::begin(state0.clone());
        tx2.state_mut().players.get_mut(p1).unwrap().hp -= 1;
        tx2.journal(JournalEntry::PlayerHp {
            id: p1,
            prev: prev_hp,
        });
        tx2.state_mut().players.get_mut(p1).unwrap().marks = vec![(7, 3)];
        tx2.journal(JournalEntry::PlayerMarks {
            id: p1,
            prev: prev_marks.clone(),
        });
        let replayed = tx2.rollback_via_journal().expect("tracked rollback");
        let h_replay = canonical_state_hash(&replayed);

        assert_eq!(h_base, h_replay, "撤销重放必须等价于基线回滚");
        assert_eq!(replayed.players.get(p1).unwrap().hp, prev_hp);
        assert!(replayed.players.get(p1).unwrap().marks.is_empty());
    }

    /// 缺口检测：未被 journal 覆盖的字段写入（如静默改 hand_limit_bonus）
    /// 在 journal-only 回滚后必然造成哈希差异——该测试守护「等价性测试
    /// 能发现缺口」这一能力本身，防止框架退化为恒真。
    #[test]
    fn journal_rollback_detects_untracked_gap() {
        let e = started_engine();
        let state0 = e.state().clone();
        let p1 = PlayerId::new(1);
        let prev_hp = state0.players.get(p1).unwrap().hp;

        let mut tx1 = Transaction::begin(state0.clone());
        tx1.state_mut().players.get_mut(p1).unwrap().hp -= 1;
        tx1.journal(JournalEntry::PlayerHp {
            id: p1,
            prev: prev_hp,
        });
        let base = tx1.rollback();
        let h_base = canonical_state_hash(&base);

        let mut tx2 = Transaction::begin(state0.clone());
        tx2.state_mut().players.get_mut(p1).unwrap().hp -= 1;
        tx2.journal(JournalEntry::PlayerHp {
            id: p1,
            prev: prev_hp,
        });
        // 未跟踪写入：journal 缺口（revision 被哈希编码，缺口必然显形）
        tx2.state_mut().revision += 100;
        let replayed = tx2.rollback_via_journal().expect("tracked rollback");
        assert_ne!(
            canonical_state_hash(&replayed),
            h_base,
            "未跟踪字段的缺口必须被等价性测试发现"
        );
    }

    /// 卡牌移动（hand → discard）的精确撤销：区域列表、卡字段、
    /// 原列表位置全部还原，且与基线回滚哈希等价。
    #[test]
    fn journal_rollback_equivalent_for_card_move() {
        use crate::state::ZoneKind;
        let e = started_engine();
        let state0 = e.state().clone();
        let p1 = PlayerId::new(1);
        let hand = state0.zones.find(ZoneKind::Hand, p1).expect("hand zone");
        let discard = state0.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).expect("discard zone");
        let card = *state0
            .zones
            .get(hand)
            .and_then(|z| z.cards.first())
            .expect("hand has a card");

        // 路径 A：完整基线回滚
        let mut tx1 = Transaction::begin(state0.clone());
        {
            let (prev_owner, prev_face_up, prev_index) = {
                let c = tx1.state().cards.get(card).unwrap();
                let idx = tx1.state().zones.get(hand).unwrap().cards.iter().position(|x| *x == card).unwrap() as u32;
                (c.owner, c.face_up, idx)
            };
            tx1.journal(JournalEntry::CardMove {
                card,
                prev_zone: hand,
                prev_owner,
                prev_face_up: prev_face_up,
                prev_index,
            });
            assert!(tx1.state_mut().zones.remove_card(hand, card));
            assert!(tx1.state_mut().zones.push_card(discard, card));
            let c = tx1.state_mut().cards.get_mut(card).unwrap();
            c.zone = discard;
            c.owner = PlayerId::NONE;
            c.face_up = true;
        }
        let base = tx1.rollback();
        let h_base = canonical_state_hash(&base);

        // 路径 B：journal-only 回滚
        let mut tx2 = Transaction::begin(state0.clone());
        {
            let (prev_owner, prev_face_up, prev_index) = {
                let c = tx2.state().cards.get(card).unwrap();
                let idx = tx2.state().zones.get(hand).unwrap().cards.iter().position(|x| *x == card).unwrap() as u32;
                (c.owner, c.face_up, idx)
            };
            tx2.journal(JournalEntry::CardMove {
                card,
                prev_zone: hand,
                prev_owner,
                prev_face_up: prev_face_up,
                prev_index,
            });
            assert!(tx2.state_mut().zones.remove_card(hand, card));
            assert!(tx2.state_mut().zones.push_card(discard, card));
            let c = tx2.state_mut().cards.get_mut(card).unwrap();
            c.zone = discard;
            c.owner = PlayerId::NONE;
            c.face_up = true;
        }
        let replayed = tx2.rollback_via_journal().expect("card move undo");
        let h_replay = canonical_state_hash(&replayed);

        assert_eq!(h_base, h_replay, "卡牌移动的撤销重放必须等价于基线回滚");
        let c = replayed.cards.get(card).unwrap();
        assert_eq!(c.zone, hand);
        assert_eq!(c.owner, p1);
        let z = replayed.zones.get(hand).unwrap();
        assert_eq!(z.cards.iter().position(|x| *x == card), Some(0), "应恢复到原列表位置");
        assert!(replayed.zones.get(discard).unwrap().cards.is_empty());
    }

    /// Checkpoint 类条目未携带前值快照：journal-only 回滚必须显式报错
    /// （诚实降级），而不是静默漏撤销。
    #[test]
    fn checkpoint_entries_block_journal_rollback() {
        let e = started_engine();
        let state0 = e.state().clone();
        let mut tx = Transaction::begin(state0);
        tx.journal(JournalEntry::TurnCheckpoint);
        match tx.rollback_via_journal() {
            Err(JournalRollbackError::CheckpointEntry("turn")) => {}
            other => panic!("expected CheckpointEntry(turn), got {other:?}"),
        }
    }
}
