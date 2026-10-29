//! Typed query system with modifiers, explain, cycle detection, optional cache.
//!
//! Queries never emit effects or consume RuleRng (architecture §9).

mod cache;
mod modifier;

pub use cache::{CacheKey, QueryCache};
pub use modifier::{
    alive_wounded_or_guiming_wu_count, apply_distance_modifiers, apply_int_modifiers,
    apply_int_modifiers_for_state, apply_int_modifiers_for_state_with_context,
    apply_int_modifiers_for_state_with_other, bootstrap_modifier_defs, optional_draw_bonuses_for,
    sources_by_label, ActiveModifier, ActiveModifierIndex, CardSelector, CombineMode,
    ModifierDefinition, ModifierTarget, ModifierValueSource,
};

use crate::state::{Card, CardKind, EquipSlot, GameState, Phase, ZoneKind};
// CardKind used for equip queries
use engine_api::{CardId, PlayerId, Suit};
use rule_api::{ModifierSource, QueryErrorCode, QueryRequest, QueryResponse, RelationTag};
use std::collections::{BTreeSet, HashSet};

pub(crate) const XINYAOMING_DRAW: u8 = 1;
pub(crate) const XINYAOMING_DISCARD_HAND: u8 = 2;
pub(crate) const XINYAOMING_DISCARD_DRAW: u8 = 3;

/// Explained query result for debugger / AI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryExplanation {
    /// Final response.
    pub response: QueryResponse,
    /// Base value before modifiers (if integer path).
    pub base: Option<i32>,
    /// Modifier contributions.
    pub sources: Vec<ModifierSource>,
}

/// Query context for recursion / cycle detection.
#[derive(Debug)]
pub struct QueryContext {
    /// Depth.
    pub depth: u32,
    /// Max depth.
    pub max_depth: u32,
    /// In-flight request keys (cycle detection).
    in_flight: BTreeSet<Vec<u8>>,
    /// Optional cache.
    cache: Option<QueryCache>,
}

impl Default for QueryContext {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryContext {
    /// Default context (no cache).
    ///
    /// # 缓存边界契约（ADR 0703 第②部分）
    ///
    /// 缓存键的依赖版本为 `GameState::query_dep_version()`：提交版本 +
    /// 结算内伤害/恢复/选择序号 + RNG 调用数。结算内这些哨兵推进后
    /// 同键查询不会再命中旧值。**残余边界**：不经过上述哨兵的静默
    /// 字段写入（如宿主直改状态字段）不使键失效——跨命令/跨事务复用
    /// 本上下文时，宿主必须以提交边界为准调用 [`QueryContext::clear`]
    /// （或按命令重建上下文）。内核自身（vm_host）按命令新建无缓存
    /// 上下文，不受影响。
    pub fn new() -> Self {
        Self {
            depth: 0,
            max_depth: 32,
            in_flight: BTreeSet::new(),
            cache: None,
        }
    }

    /// 缓存命中次数（宿主侧可观察缓存边界；无缓存时恒为 0）。
    pub fn cache_hits(&self) -> u64 {
        self.cache.as_ref().map(|c| c.stats().0).unwrap_or(0)
    }

    /// Context with cache control.
    pub fn with_cache(enabled: bool) -> Self {
        let mut c = QueryCache::new();
        c.set_enabled(enabled);
        Self {
            depth: 0,
            max_depth: 32,
            in_flight: BTreeSet::new(),
            cache: Some(c),
        }
    }

    /// Enable/disable cache.
    pub fn set_cache_enabled(&mut self, on: bool) {
        if let Some(c) = &mut self.cache {
            c.set_enabled(on);
        } else if on {
            let mut c = QueryCache::new();
            c.set_enabled(true);
            self.cache = Some(c);
        }
    }

    /// Clear cache.
    pub fn clear_cache(&mut self) {
        if let Some(c) = &mut self.cache {
            c.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::bootstrap_match;
    use crate::state::GameState;
    use crate::transaction::Transaction;
    use crate::trigger::{BOOTSTRAP_MASHU, BOOTSTRAP_QICAO};
    use engine_api::{CardDefId, SkillDefId, SkillId, ZoneId};
    use rule_api::EffectRequest;

    #[test]
    fn cache_on_off_same_result() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 3, 1, "bootstrap").unwrap();
        let state = tx.state().clone();
        let p1 = PlayerId::new(1);
        let p2 = PlayerId::new(2);
        let req = QueryRequest::Distance { from: p1, to: p2 };

        let mut ctx_off = QueryContext::with_cache(false);
        let a = query(&state, &req, &mut ctx_off);

        let mut ctx_on = QueryContext::with_cache(true);
        let b1 = query(&state, &req, &mut ctx_on);
        let b2 = query(&state, &req, &mut ctx_on);
        assert_eq!(a, b1);
        assert_eq!(b1, b2);
    }

    #[test]
    fn exact_number_subsets_bound_unsatisfiable_large_hand() {
        let mut state = GameState::empty();
        let cards: Vec<_> = (0..64)
            .map(|_| {
                state.cards.alloc(Card {
                    id: CardId::NONE,
                    def: CardDefId::NONE,
                    kind: CardKind::Sha,
                    suit: Suit::Spade,
                    rank: 1,
                    weapon_range: 0,
                    zone: ZoneId::NONE,
                    owner: PlayerId::NONE,
                    face_up: true,
                })
            })
            .collect();

        assert!(
            effective_number_sum_subsets(&state, PlayerId::NONE, &cards, 1, 12, 13, 64,).is_empty()
        );
    }

    #[test]
    fn unique_max_current_hp_requires_one_strictly_highest_living_player() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 3, 1, "bootstrap").unwrap();
        let p1 = PlayerId::new(1);
        let p2 = PlayerId::new(2);
        let p3 = PlayerId::new(3);
        tx.state_mut().players.get_mut(p1).unwrap().hp = 3;
        tx.state_mut().players.get_mut(p2).unwrap().hp = 3;
        tx.state_mut().players.get_mut(p3).unwrap().hp = 2;

        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHp { player: p1 },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(false)
        );

        tx.state_mut().players.get_mut(p2).unwrap().hp = 1;
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHp { player: p1 },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(true)
        );
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHpAt {
                    player: p2,
                    value: 4,
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(true)
        );

        tx.state_mut().players.get_mut(p1).unwrap().alive = false;
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHp { player: p1 },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(false)
        );
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::UniqueMaxCurrentHp {
                    player: PlayerId::new(99),
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            }
        );
    }

    #[test]
    fn lifetime_damage_dealt_is_saturating_and_validates_player() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
        let player = PlayerId::new(1);

        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::LifetimeDamageDealt { player },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(0)
        );
        tx.state_mut().damage_dealt_totals.insert(player, u32::MAX);
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::LifetimeDamageDealt { player },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(i32::MAX)
        );
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::LifetimeDamageDealt {
                    player: PlayerId::new(99),
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            }
        );
    }

    #[test]
    fn public_card_kind_count_reads_only_draw_and_discard() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
        let player = PlayerId::new(1);
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let card = tx
            .state()
            .zones
            .get(draw)
            .unwrap()
            .cards
            .iter()
            .copied()
            .find(|card| tx.state().cards.get(*card).unwrap().kind != CardKind::Tao)
            .unwrap();
        let before = public_card_kind_count(tx.state(), CardKind::Tao.tag()).unwrap();

        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Tao;
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::PublicCardKindCount {
                    kind: CardKind::Tao.tag(),
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(before + 1)
        );

        tx.state_mut()
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .retain(|candidate| *candidate != card);
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        let card_state = tx.state_mut().cards.get_mut(card).unwrap();
        card_state.zone = hand;
        card_state.owner = player;
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::PublicCardKindCount {
                    kind: CardKind::Tao.tag(),
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(before)
        );
        assert_eq!(
            query(
                tx.state(),
                &QueryRequest::PublicCardKindCount { kind: u8::MAX - 1 },
                &mut QueryContext::new(),
            ),
            QueryResponse::Error {
                code: QueryErrorCode::InvalidId,
            }
        );
    }

    #[test]
    fn mashu_reduces_distance_explain() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 4, 1, "bootstrap").unwrap();
        // Give p1 马术
        let p1 = PlayerId::new(1);
        let sid = tx.state_mut().skills.alloc(crate::state::SkillInstance {
            id: SkillId::NONE,
            def: SkillDefId::new(BOOTSTRAP_MASHU),
            owner: p1,
            enabled: true,
            use_count: 0,
            turn_fire_count: 0,
            content_flags: 0,
            play_phase_fire_count: 0,
            play_phase_use_count: 0,
            play_phase_category_mask: 0,
            active_turn_banned: false,
            turn_disabled: false,
            disabled_by_skills: Vec::new(),
            temporary_until_turn_end_of: PlayerId::NONE,
            round_fire_round: 0,
            lifetime_fire_count: 0,
            view_as_branch: 0,
            mark: 0,
            view_as_use_round: 0,
            view_as_used_branches: 0,
            state_blob: vec![],
            source_free_physical_card: CardId::NONE,
            general_slot: crate::state::GENERAL_SLOT_NONE,
            content_storage: std::collections::BTreeMap::new(),
        });
        tx.state_mut().players.get_mut(p1).unwrap().skills.push(sid);
        tx.state_mut().modifiers.register_skill(
            sid,
            SkillDefId::new(BOOTSTRAP_MASHU),
            p1,
            &crate::content::ContentRegistry::new(),
        );

        let p3 = PlayerId::new(3);
        let mut ctx = QueryContext::new();
        let base_only = {
            // without would be seat distance
            base_distance(tx.state(), p1, p3).unwrap()
        };
        let exp = query_explain(
            tx.state(),
            &QueryRequest::Distance { from: p1, to: p3 },
            &mut ctx,
        );
        assert_eq!(exp.base, Some(base_only));
        assert!(!exp.sources.is_empty());
        if let QueryResponse::Int(v) = exp.response {
            assert!(v <= base_only);
            assert!(v >= 1);
        } else {
            panic!("expected int");
        }
        let _ = EffectRequest::AddSkill {
            player: p1,
            skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
        };
    }
}

/// 缓存边界（ADR 0703 第②部分）：依赖版本覆盖结算内单调哨兵，
/// 防止「revision 未变但状态已变」的脏命中。
#[cfg(test)]
mod cache_boundary_tests {
    use super::*;
    use crate::engine::Engine;
    use crate::engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};

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

    /// 依赖版本哨兵：revision / 伤害序号 / 恢复序号 / 选择序号 / RNG 调用数
    /// 任一推进都必须改变依赖版本。
    #[test]
    fn dep_version_senses_intra_settlement_sentinels() {
        let mut e = started_engine();
        let state = e.state_mut_for_tests();
        let v0 = state.query_dep_version();
        state.resolution.next_damage_seq += 1;
        assert_ne!(state.query_dep_version(), v0, "伤害序号推进应改变依赖版本");
        state.resolution.next_recovery_seq += 1;
        assert_ne!(state.query_dep_version(), v0, "恢复序号推进应改变依赖版本");
        state.resolution.next_choice_seq += 1;
        assert_ne!(state.query_dep_version(), v0, "选择序号推进应改变依赖版本");
        state.rng.call_count += 1;
        assert_ne!(state.query_dep_version(), v0, "RNG 消耗应改变依赖版本");
        state.revision += 1;
        assert_ne!(state.query_dep_version(), v0, "提交版本推进应改变依赖版本");
    }

    /// 缓存脏读修复：结算内推进哨兵后，同键查询不得命中旧缓存。
    #[test]
    fn cached_query_recomputes_after_intra_settlement_progress() {
        let mut e = started_engine();
        let p1 = PlayerId::new(1);
        let mut ctx = QueryContext::with_cache(true);

        let state = e.state();
        let r1 = query_explain(state, &QueryRequest::Hp { player: p1 }, &mut ctx);
        assert_eq!(ctx.cache_hits(), 0, "首次查询未命中");

        let state = e.state();
        let r2 = query_explain(state, &QueryRequest::Hp { player: p1 }, &mut ctx);
        assert_eq!(ctx.cache_hits(), 1, "同键二次查询应命中缓存");
        assert_eq!(r1.response, r2.response);

        // 模拟结算内推进（revision 不变）：伤害序号 +1
        e.state_mut_for_tests().resolution.next_damage_seq += 1;

        let state = e.state();
        let _r3 = query_explain(state, &QueryRequest::Hp { player: p1 }, &mut ctx);
        assert_eq!(
            ctx.cache_hits(),
            1,
            "依赖版本推进后不得命中旧缓存（脏读边界）"
        );
    }
}

mod alive_wounded_count;
pub use alive_wounded_count::*;

mod query;
pub use query::*;
mod sha_rank_skill_mark_modifier_active;
pub use sha_rank_skill_mark_modifier_active::*;
mod discardable_target_cards;
pub use discardable_target_cards::*;
mod card_target_cancelled_for_logical_kind;
pub use card_target_cancelled_for_logical_kind::*;
mod duel_response_count_for;
pub use duel_response_count_for::*;
mod target_legal_explain;
pub(crate) use target_legal_explain::*;
