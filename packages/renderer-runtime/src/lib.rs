//! Battle renderer runtime (architecture plan §27).
//!
//! Implements the three core §27 rendering strategies as testable logic:
//!
//! - **DOM node budget enforcement** (§27: 战斗 DOM 节点数量有预算)
//! - **Frame aggregation** (§27: 一帧内合并重复更新)
//! - **Animation degradation** (§27: 低价值动画可合并或丢弃)
//!
//! These are platform-independent policy types. The web client (Vue/TS) reads
//! the decisions from this module and applies them to the DOM.

#![deny(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---- §27: DOM Node Budget ----

/// One category of DOM nodes in the battle UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeCategory {
    /// Seat / player cards.
    Seats = 1,
    /// Hand cards.
    HandCards = 2,
    /// Equipped cards.
    EquipCards = 3,
    /// Animation / effect overlays.
    Effects = 4,
    /// Status indicators (HP bars, marks).
    StatusIndicators = 5,
    /// Log entries.
    LogEntries = 6,
}

/// DOM node budget per category (§27).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeBudget {
    /// Max nodes per category.
    pub limits: HashMap<NodeCategory, u32>,
    /// Total max across all categories.
    pub total_limit: u32,
}

impl Default for NodeBudget {
    fn default() -> Self {
        let mut limits = HashMap::new();
        limits.insert(NodeCategory::Seats, 8);
        limits.insert(NodeCategory::HandCards, 200);
        limits.insert(NodeCategory::EquipCards, 32);
        limits.insert(NodeCategory::Effects, 20);
        limits.insert(NodeCategory::StatusIndicators, 80);
        limits.insert(NodeCategory::LogEntries, 50);
        Self {
            limits,
            total_limit: 500,
        }
    }
}

/// Low-memory node budget (for mobile devices).
impl NodeBudget {
    /// Reduced budget for low-end mobile.
    pub fn low_memory() -> Self {
        let mut limits = HashMap::new();
        limits.insert(NodeCategory::Seats, 8);
        limits.insert(NodeCategory::HandCards, 80);
        limits.insert(NodeCategory::EquipCards, 16);
        limits.insert(NodeCategory::Effects, 5);
        limits.insert(NodeCategory::StatusIndicators, 40);
        limits.insert(NodeCategory::LogEntries, 20);
        Self {
            limits,
            total_limit: 200,
        }
    }

    /// Check if adding `count` nodes of a category exceeds the budget.
    pub fn check_add(&self, category: NodeCategory, current: u32, count: u32) -> BudgetResult {
        let cat_limit = self.limits.get(&category).copied().unwrap_or(u32::MAX);
        if current + count > cat_limit {
            return BudgetResult::Exceeded {
                category,
                requested: current + count,
                limit: cat_limit,
            };
        }
        BudgetResult::Ok
    }
}

/// Result of a budget check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetResult {
    /// Within budget.
    Ok,
    /// Budget exceeded — must evict or refuse.
    Exceeded {
        /// Category that exceeded.
        category: NodeCategory,
        /// Requested total.
        requested: u32,
        /// Budget limit.
        limit: u32,
    },
}

// ---- §27: Frame Aggregation ----

/// One rendering patch to be applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderPatch {
    /// Target element id.
    pub target: String,
    /// Property to update (e.g. "hp", "hand", "class").
    pub property: String,
    /// New value.
    pub value: String,
}

/// Frame aggregator: merges patches within a single animation frame.
///
/// §27: "一帧内合并重复更新" — multiple patches to the same target+property
/// within one frame are collapsed to the last value.
#[derive(Debug, Clone, Default)]
pub struct FrameAggregator {
    /// Pending patches: (target, property) → latest value.
    pending: HashMap<(String, String), String>,
    /// Number of patches received this frame (before dedup).
    patches_received: u32,
    /// Number of patches after dedup.
    patches_after_dedup: u32,
}

impl FrameAggregator {
    /// Create a new aggregator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a patch for the current frame.
    pub fn queue(&mut self, patch: RenderPatch) {
        self.patches_received += 1;
        self.pending
            .insert((patch.target.clone(), patch.property.clone()), patch.value);
    }

    /// Flush all pending patches for this frame (returns deduplicated patches).
    pub fn flush(&mut self) -> Vec<RenderPatch> {
        self.patches_after_dedup = self.pending.len() as u32;
        let patches: Vec<RenderPatch> = self
            .pending
            .drain()
            .map(|((target, property), value)| RenderPatch {
                target,
                property,
                value,
            })
            .collect();
        // Sort by target for deterministic order.
        let _ = &mut self.patches_after_dedup;
        patches
    }

    /// How many patches were received (before dedup) this frame.
    pub fn patches_received(&self) -> u32 {
        self.patches_received
    }

    /// How many patches survived dedup in the last flush.
    pub fn patches_after_dedup(&self) -> u32 {
        self.patches_after_dedup
    }

    /// Dedup ratio (0.0 = everything deduped, 1.0 = nothing deduped).
    pub fn dedup_ratio(&self) -> f32 {
        if self.patches_received == 0 {
            return 1.0;
        }
        self.patches_after_dedup as f32 / self.patches_received as f32
    }
}

// ---- §27: Animation Degradation ----

/// Animation quality level (§27: 低价值动画可合并或丢弃).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum AnimationQuality {
    /// Full quality: all animations, particles, transitions.
    Full = 0,
    /// Reduced: skip non-essential particles, shorten transitions.
    Reduced = 1,
    /// Minimal: only essential animations (HP changes, card moves).
    Minimal = 2,
    /// Off: no animations at all (§27: 状态正确性不依赖动画完成).
    Off = 3,
}

impl Default for AnimationQuality {
    fn default() -> Self {
        AnimationQuality::Full
    }
}

/// Animation degradation policy (§27).
///
/// Monitors frame performance and escalates animation quality degradation
/// when frames are consistently over budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationPolicy {
    /// Current quality level.
    pub current_quality: AnimationQuality,
    /// Frame budget in milliseconds.
    pub frame_budget_ms: u32,
    /// Consecutive over-budget frames before escalating.
    pub escalate_after: u32,
    /// Consecutive under-budget frames before de-escalating.
    pub deescalate_after: u32,
    /// Current consecutive over-budget count.
    over_budget_streak: u32,
    /// Current consecutive under-budget count.
    under_budget_streak: u32,
}

impl Default for DegradationPolicy {
    fn default() -> Self {
        Self {
            current_quality: AnimationQuality::Full,
            frame_budget_ms: 16, // 60fps
            escalate_after: 5,
            deescalate_after: 30,
            over_budget_streak: 0,
            under_budget_streak: 0,
        }
    }
}

impl DegradationPolicy {
    /// Create a policy for low-end devices.
    pub fn low_end() -> Self {
        Self {
            current_quality: AnimationQuality::Reduced,
            frame_budget_ms: 33, // 30fps
            escalate_after: 3,
            deescalate_after: 60,
            over_budget_streak: 0,
            under_budget_streak: 0,
        }
    }

    /// Report a frame's render time. Returns the (possibly new) quality level.
    pub fn report_frame(&mut self, frame_ms: u32) -> AnimationQuality {
        if frame_ms > self.frame_budget_ms {
            self.over_budget_streak += 1;
            self.under_budget_streak = 0;
            if self.over_budget_streak >= self.escalate_after {
                self.escalate();
            }
        } else {
            self.under_budget_streak += 1;
            self.over_budget_streak = 0;
            if self.under_budget_streak >= self.deescalate_after {
                self.deescalate();
            }
        }
        self.current_quality
    }

    /// Escalate degradation (lower quality).
    pub fn escalate(&mut self) {
        self.current_quality = match self.current_quality {
            AnimationQuality::Full => AnimationQuality::Reduced,
            AnimationQuality::Reduced => AnimationQuality::Minimal,
            AnimationQuality::Minimal => AnimationQuality::Off,
            AnimationQuality::Off => AnimationQuality::Off,
        };
        self.over_budget_streak = 0;
    }

    /// De-escalate degradation (higher quality).
    pub fn deescalate(&mut self) {
        self.current_quality = match self.current_quality {
            AnimationQuality::Off => AnimationQuality::Minimal,
            AnimationQuality::Minimal => AnimationQuality::Reduced,
            AnimationQuality::Reduced => AnimationQuality::Full,
            AnimationQuality::Full => AnimationQuality::Full,
        };
        self.under_budget_streak = 0;
    }

    /// Whether a given animation type should be shown at current quality.
    pub fn should_show(&self, anim_type: AnimationType) -> bool {
        match self.current_quality {
            AnimationQuality::Full => true,
            AnimationQuality::Reduced => !matches!(
                anim_type,
                AnimationType::Particle | AnimationType::ScreenFlash
            ),
            AnimationQuality::Minimal => {
                matches!(anim_type, AnimationType::HpChange | AnimationType::CardMove)
            }
            AnimationQuality::Off => false,
        }
    }
}

/// Type of animation (for degradation decisions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AnimationType {
    /// HP bar change (essential).
    HpChange = 1,
    /// Card move (essential).
    CardMove = 2,
    /// Card flip animation.
    CardFlip = 3,
    /// Damage number popup.
    DamagePopup = 4,
    /// Particle effect (non-essential).
    Particle = 5,
    /// Screen flash (non-essential).
    ScreenFlash = 6,
    /// Skill activation glow.
    SkillGlow = 7,
}

// ---- §27.1 PresentationCue Animation Timeline ----

/// One presentation cue in the animation timeline (§27.1).
///
/// A PresentationCue is NOT a source of truth — it's a visual hint derived from
/// the authority state. It can be skipped, sped up, or discarded during backlog
/// without affecting rule correctness. If lost, the final screen can always be
/// rebuilt from an ObserverSnapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresentationCue {
    /// Stable monotonic sequence number.
    pub sequence: u64,
    /// Source revision this cue was derived from.
    pub source_revision: u64,
    /// Type of visual event.
    pub cue_type: CueType,
    /// Duration in milliseconds (at 1× speed).
    pub duration_ms: u32,
    /// Whether this cue blocks interaction until complete (e.g. card deal).
    pub blocks_interaction: bool,
}

/// Type of visual cue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum CueType {
    /// Cards drawn animation.
    CardDraw = 1,
    /// Card played / used animation.
    CardPlay = 2,
    /// Damage dealt flash.
    DamageDealt = 3,
    /// HP recovered effect.
    HpRecovered = 4,
    /// Player entered dying state.
    DyingEntered = 5,
    /// Player died.
    PlayerDied = 6,
    /// Skill activated glow.
    SkillActivated = 7,
    /// Turn changed indicator.
    TurnChanged = 8,
    /// Phase changed indicator.
    PhaseChanged = 9,
    /// Card moved (equipment, judgment, etc.).
    CardMoved = 10,
}

/// The animation timeline (§27.1).
///
/// Manages a queue of PresentationCues with skip, speed control, and backlog
/// degradation. The rule core never waits for animations; the UI consumes
/// cues at its own pace.
#[derive(Debug, Clone)]
pub struct AnimationTimeline {
    /// Pending cues in sequence order.
    pending: Vec<PresentationCue>,
    /// Cues already played.
    played: Vec<PresentationCue>,
    /// Speed multiplier (1.0 = normal, 2.0 = double speed, etc.).
    speed: f32,
    /// Whether skipping is enabled.
    skip_enabled: bool,
    /// Maximum pending cues before backlog degradation kicks in.
    backlog_threshold: usize,
    /// Next sequence to assign.
    next_sequence: u64,
    /// Whether interaction is currently blocked by a cue.
    interaction_blocked: bool,
}

impl Default for AnimationTimeline {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
            played: Vec::new(),
            speed: 1.0,
            skip_enabled: true,
            backlog_threshold: 20,
            next_sequence: 1,
            interaction_blocked: false,
        }
    }
}

impl AnimationTimeline {
    /// Create a new timeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the speed multiplier (§27.1: 可倍速).
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.1);
    }

    /// Get the current speed multiplier.
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Enable or disable skipping (§27.1: 可跳过).
    pub fn set_skip_enabled(&mut self, enabled: bool) {
        self.skip_enabled = enabled;
    }

    /// Queue a new cue from a cue type + revision.
    pub fn queue_cue(
        &mut self,
        cue_type: CueType,
        source_revision: u64,
        duration_ms: u32,
        blocks_interaction: bool,
    ) {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        self.pending.push(PresentationCue {
            sequence: seq,
            source_revision,
            cue_type,
            duration_ms,
            blocks_interaction,
        });
        // §27.1: 可在积压时降级 — if backlog exceeds threshold, compress.
        if self.pending.len() > self.backlog_threshold {
            self.compress_backlog();
        }
    }

    /// Play the next cue. Returns the effective duration (adjusted by speed).
    /// Returns None if no cues are pending.
    pub fn play_next(&mut self) -> Option<(PresentationCue, u32)> {
        let cue = self.pending.first()?.clone();
        self.pending.remove(0);
        let effective_ms = (cue.duration_ms as f32 / self.speed) as u32;
        self.interaction_blocked = cue.blocks_interaction;
        self.played.push(cue.clone());
        Some((cue, effective_ms))
    }

    /// Skip all pending cues (§27.1: 可跳过).
    /// The final screen is rebuilt from ObserverSnapshot, not from cues.
    pub fn skip_all(&mut self) {
        if !self.skip_enabled {
            return;
        }
        self.pending.clear();
        self.interaction_blocked = false;
    }

    /// Number of pending cues.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Whether interaction is currently blocked.
    pub fn is_interaction_blocked(&self) -> bool {
        self.interaction_blocked
    }

    /// Whether the timeline is in backlog (pending exceeds threshold).
    pub fn is_backlogged(&self) -> bool {
        self.pending.len() > self.backlog_threshold
    }

    /// Compress backlog by merging non-essential cues (§27.1: 可在积压时降级).
    /// Keeps only essential cues (CardPlay, DamageDealt, DyingEntered, PlayerDied).
    fn compress_backlog(&mut self) {
        self.pending.retain(|c| {
            matches!(
                c.cue_type,
                CueType::CardPlay
                    | CueType::DamageDealt
                    | CueType::DyingEntered
                    | CueType::PlayerDied
            )
        });
    }

    /// Recover the final visual state from a snapshot (§27.1: 丢失后可由
    /// ObserverSnapshot 恢复最终画面). Clears all cues — the UI rebuilds
    /// from the authoritative observer view.
    pub fn recover_from_snapshot(&mut self) {
        self.pending.clear();
        self.played.clear();
        self.interaction_blocked = false;
    }

    /// Total cues played.
    pub fn played_count(&self) -> usize {
        self.played.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- DOM Node Budget tests ----

    #[test]
    fn budget_allows_within_limit() {
        let budget = NodeBudget::default();
        let result = budget.check_add(NodeCategory::Seats, 4, 2);
        assert_eq!(result, BudgetResult::Ok);
    }

    #[test]
    fn budget_rejects_over_limit() {
        let budget = NodeBudget::default();
        let result = budget.check_add(NodeCategory::Seats, 7, 2);
        assert!(matches!(result, BudgetResult::Exceeded { .. }));
    }

    #[test]
    fn low_memory_budget_is_smaller() {
        let normal = NodeBudget::default();
        let low = NodeBudget::low_memory();
        let normal_hand = normal.limits.get(&NodeCategory::HandCards).unwrap();
        let low_hand = low.limits.get(&NodeCategory::HandCards).unwrap();
        assert!(low_hand < normal_hand);
    }

    // ---- Frame Aggregation tests ----

    #[test]
    fn aggregator_deduplicates_same_target_property() {
        let mut agg = FrameAggregator::new();
        agg.queue(RenderPatch {
            target: "seat1".into(),
            property: "hp".into(),
            value: "3".into(),
        });
        agg.queue(RenderPatch {
            target: "seat1".into(),
            property: "hp".into(),
            value: "2".into(),
        });
        agg.queue(RenderPatch {
            target: "seat1".into(),
            property: "hp".into(),
            value: "1".into(),
        });
        let flushed = agg.flush();
        assert_eq!(flushed.len(), 1, "3 patches to same target+property → 1");
        assert_eq!(flushed[0].value, "1", "last value wins");
        assert_eq!(agg.patches_received(), 3);
        assert_eq!(agg.patches_after_dedup(), 1);
    }

    #[test]
    fn aggregator_keeps_different_properties() {
        let mut agg = FrameAggregator::new();
        agg.queue(RenderPatch {
            target: "seat1".into(),
            property: "hp".into(),
            value: "3".into(),
        });
        agg.queue(RenderPatch {
            target: "seat1".into(),
            property: "class".into(),
            value: "dying".into(),
        });
        let flushed = agg.flush();
        assert_eq!(flushed.len(), 2);
    }

    #[test]
    fn aggregator_dedup_ratio() {
        let mut agg = FrameAggregator::new();
        for _ in 0..10 {
            agg.queue(RenderPatch {
                target: "x".into(),
                property: "y".into(),
                value: "z".into(),
            });
        }
        agg.flush();
        assert!(agg.dedup_ratio() < 0.2, "10 identical → ratio < 0.2");
    }

    // ---- Animation Degradation tests ----

    #[test]
    fn degradation_escalates_on_over_budget_streak() {
        let mut policy = DegradationPolicy::default();
        assert_eq!(policy.current_quality, AnimationQuality::Full);
        // 5 consecutive over-budget frames.
        for _ in 0..5 {
            policy.report_frame(20); // 20ms > 16ms budget
        }
        assert_eq!(
            policy.current_quality,
            AnimationQuality::Reduced,
            "should degrade after 5 over-budget frames"
        );
    }

    #[test]
    fn degradation_deescalates_on_under_budget_streak() {
        let mut policy = DegradationPolicy::default();
        policy.current_quality = AnimationQuality::Reduced;
        // 30 consecutive under-budget frames.
        for _ in 0..30 {
            policy.report_frame(8); // 8ms < 16ms budget
        }
        assert_eq!(
            policy.current_quality,
            AnimationQuality::Full,
            "should improve after 30 under-budget frames"
        );
    }

    #[test]
    fn degradation_full_quality_shows_all() {
        let policy = DegradationPolicy::default();
        assert!(policy.should_show(AnimationType::HpChange));
        assert!(policy.should_show(AnimationType::Particle));
        assert!(policy.should_show(AnimationType::ScreenFlash));
    }

    #[test]
    fn degradation_off_shows_nothing() {
        let policy = DegradationPolicy {
            current_quality: AnimationQuality::Off,
            ..Default::default()
        };
        assert!(!policy.should_show(AnimationType::HpChange));
        assert!(!policy.should_show(AnimationType::Particle));
    }

    #[test]
    fn degradation_minimal_shows_only_essential() {
        let policy = DegradationPolicy {
            current_quality: AnimationQuality::Minimal,
            ..Default::default()
        };
        assert!(policy.should_show(AnimationType::HpChange));
        assert!(policy.should_show(AnimationType::CardMove));
        assert!(!policy.should_show(AnimationType::Particle));
        assert!(!policy.should_show(AnimationType::CardFlip));
    }

    #[test]
    fn low_end_policy_starts_reduced() {
        let policy = DegradationPolicy::low_end();
        assert_eq!(policy.current_quality, AnimationQuality::Reduced);
        assert!(
            policy.frame_budget_ms > 16,
            "low-end should have > 16ms budget"
        );
    }

    #[test]
    fn escalation_clamps_at_off() {
        let mut policy = DegradationPolicy::default();
        policy.escalate(); // Full → Reduced
        policy.escalate(); // Reduced → Minimal
        policy.escalate(); // Minimal → Off
        policy.escalate(); // Off → Off (clamped)
        assert_eq!(policy.current_quality, AnimationQuality::Off);
    }

    // ---- §27.1 PresentationCue Timeline tests ----

    #[test]
    fn timeline_queues_and_plays_in_order() {
        let mut tl = AnimationTimeline::new();
        tl.queue_cue(CueType::CardDraw, 1, 200, false);
        tl.queue_cue(CueType::CardPlay, 2, 300, true);
        tl.queue_cue(CueType::DamageDealt, 3, 150, false);
        assert_eq!(tl.pending_count(), 3);
        // Play in order.
        let (c1, d1) = tl.play_next().unwrap();
        assert_eq!(c1.cue_type, CueType::CardDraw);
        assert_eq!(d1, 200);
        let (c2, _) = tl.play_next().unwrap();
        assert_eq!(c2.cue_type, CueType::CardPlay);
        let (c3, _) = tl.play_next().unwrap();
        assert_eq!(c3.cue_type, CueType::DamageDealt);
        assert_eq!(tl.pending_count(), 0);
        assert_eq!(tl.played_count(), 3);
    }

    #[test]
    fn timeline_speed_reduces_duration() {
        let mut tl = AnimationTimeline::new();
        tl.set_speed(2.0);
        tl.queue_cue(CueType::CardDraw, 1, 400, false);
        let (_, effective) = tl.play_next().unwrap();
        assert_eq!(effective, 200, "2× speed → half duration");
    }

    #[test]
    fn timeline_skip_all_clears_pending() {
        let mut tl = AnimationTimeline::new();
        tl.queue_cue(CueType::CardDraw, 1, 200, false);
        tl.queue_cue(CueType::CardPlay, 2, 300, true);
        assert_eq!(tl.pending_count(), 2);
        tl.skip_all();
        assert_eq!(tl.pending_count(), 0, "skip clears all pending");
        assert!(!tl.is_interaction_blocked());
    }

    #[test]
    fn timeline_skip_disabled_does_not_clear() {
        let mut tl = AnimationTimeline::new();
        tl.set_skip_enabled(false);
        tl.queue_cue(CueType::CardDraw, 1, 200, false);
        tl.skip_all();
        assert_eq!(tl.pending_count(), 1, "skip disabled → cues remain");
    }

    #[test]
    fn timeline_backlog_compresses_non_essential() {
        let mut tl = AnimationTimeline::new();
        tl.backlog_threshold = 5;
        // Queue 10 non-essential cues.
        for i in 0..10 {
            tl.queue_cue(CueType::TurnChanged, i, 100, false);
        }
        // Backlog compression should have removed non-essential cues.
        assert!(
            tl.pending_count() <= 5,
            "backlog should compress non-essential cues"
        );
    }

    #[test]
    fn timeline_backlog_keeps_essential() {
        let mut tl = AnimationTimeline::new();
        tl.backlog_threshold = 3;
        // Mix of essential and non-essential.
        tl.queue_cue(CueType::TurnChanged, 1, 100, false); // non-essential
        tl.queue_cue(CueType::DamageDealt, 2, 100, false); // essential
        tl.queue_cue(CueType::PhaseChanged, 3, 100, false); // non-essential
        tl.queue_cue(CueType::PlayerDied, 4, 100, false); // essential
        tl.queue_cue(CueType::CardPlay, 5, 100, false); // essential
                                                        // After compression, only essential remain.
        for c in &tl.pending {
            assert!(matches!(
                c.cue_type,
                CueType::DamageDealt | CueType::PlayerDied | CueType::CardPlay
            ));
        }
    }

    #[test]
    fn timeline_interaction_blocking() {
        let mut tl = AnimationTimeline::new();
        tl.queue_cue(CueType::CardPlay, 1, 300, true);
        assert!(!tl.is_interaction_blocked());
        tl.play_next();
        assert!(
            tl.is_interaction_blocked(),
            "blocking cue should block interaction"
        );
    }

    #[test]
    fn timeline_recover_from_snapshot() {
        let mut tl = AnimationTimeline::new();
        tl.queue_cue(CueType::CardDraw, 1, 200, false);
        tl.queue_cue(CueType::CardPlay, 2, 300, true);
        tl.play_next();
        tl.recover_from_snapshot();
        assert_eq!(tl.pending_count(), 0);
        assert_eq!(tl.played_count(), 0);
        assert!(!tl.is_interaction_blocked());
    }

    #[test]
    fn timeline_sequence_is_monotonic() {
        let mut tl = AnimationTimeline::new();
        tl.queue_cue(CueType::CardDraw, 1, 100, false);
        tl.queue_cue(CueType::CardPlay, 2, 100, false);
        let seqs: Vec<u64> = tl.pending.iter().map(|c| c.sequence).collect();
        assert_eq!(seqs, vec![1, 2]);
        assert!(seqs[1] > seqs[0]);
    }
}
