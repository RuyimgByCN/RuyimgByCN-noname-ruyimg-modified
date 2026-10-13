//! O2 全量合并第二批（2026-10-14）：家族 damage 的 2 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_damage_choice_migration {
//! Goldens for OFFLINE「凌弱」and SHIJI「名士」rewritten from legacy JavaScript.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const GOBLIN_LINGRUO: u32 = 3_044_957_580;
const GOBLIN_SHIQIANG: u32 = 2_912_868_664;
const GOBLIN: u32 = 2_296_491_123;
const SPMINGSHI: u32 = 4_153_946_825;
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SHIJI: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn started(package: &[u8], skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    started_seeded(package, skill_def, 0)
}

fn started_seeded(package: &[u8], skill_def: u32, salt: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == skill_def && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: skill_def as u64 + salt,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    // offline 新将批 roster 平移：排空开局挂起可选选择（qimen 模式）。
    let mut drain_key = skill_def as u64 | 0xDEAD_0000_0000_0000;
    while engine.state().has_pending_choice() {
        drain_key += 1;
        let choice_id = engine.state().resolution.pending_choice_id;
        let revision = engine.state().revision;
        let observer = engine.state().resolution.pending_observer;
        let _ = engine.submit(Command {
            actor: PlayerId::new(observer),
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(drain_key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: vec![],
                cancelled: true,
            },
        });
    }
    let mut tx = Transaction::begin(engine.state().clone());
    let old_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old_skills).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    (tx, owner, other)
}

fn keep_hand(tx: &mut Transaction, player: PlayerId, count: usize) -> Vec<CardId> {
    let cards = tx.state().hand_cards(player);
    let kept = cards[..count].to_vec();
    if cards.len() > count {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: cards[count..].to_vec(),
                to: discard,
                from: Some(hand),
            }],
        )
        .unwrap();
    }
    kept
}

fn resolve(state: GameState, actor: PlayerId, card: CardId, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn shiqiang_damage(owner_hand: usize, other_hand: usize, owner_is_source: bool) -> u32 {
    let (mut tx, owner, other) = started_seeded(OFFLINE, GOBLIN_SHIQIANG, 1);
    keep_hand(&mut tx, owner, owner_hand);
    keep_hand(&mut tx, other, other_hand);
    let (source, target) = if owner_is_source {
        (owner, other)
    } else {
        (other, owner)
    };
    let hp = tx.state().players.get(target).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    hp.saturating_sub(tx.state().players.get(target).unwrap().hp) as u32
}

#[test]
fn offline_package_contains_complete_goblin_character() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GOBLIN)
        .expect("goblin character");
    assert_eq!(character.skills, vec![GOBLIN_SHIQIANG, GOBLIN_LINGRUO]);
}

#[test]
fn shiqiang_compares_hands_on_both_sides_without_stacking_on_equal_or_self_damage() {
    assert_eq!(shiqiang_damage(3, 1, true), 2);
    assert_eq!(shiqiang_damage(1, 3, true), 1);
    assert_eq!(shiqiang_damage(1, 3, false), 2);
    assert_eq!(shiqiang_damage(3, 1, false), 1);
    assert_eq!(shiqiang_damage(2, 2, true), 1);

    let (mut tx, owner, _) = started_seeded(OFFLINE, GOBLIN_SHIQIANG, 1);
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn lingruo_takes_one_card_only_when_the_living_damage_target_has_one() {
    let (mut tx, owner, target) = started(OFFLINE, GOBLIN_LINGRUO);
    let card = keep_hand(&mut tx, target, 1)[0];
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::PickTargetCard
    );
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());
    let state = resolve(tx.commit().state, owner, card, 1);
    assert_eq!(state.cards.get(card).unwrap().owner, owner);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state.zones.find(ZoneKind::Hand, owner).unwrap()
    );
    let (mut empty, owner, target) = started(OFFLINE, GOBLIN_LINGRUO);
    keep_hand(&mut empty, target, 0);
    run_effects(
        &mut empty,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(!empty.state().has_pending_choice());
}

#[test]
fn spmingshi_forces_one_discard_per_damage_point_and_ignores_self_damage() {
    let (mut tx, owner, source) = started(SHIJI, SPMINGSHI);
    let cards = keep_hand(&mut tx, source, 2);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::DiscardOneHandOrEquip
    );
    assert_eq!(tx.state().resolution.pending_observer, source.raw());
    let state = resolve(tx.commit().state, source, cards[0], 2);
    assert!(state.has_pending_choice());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardOneHandOrEquip
    );
    let state = resolve(state, source, cards[1], 3);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));
    assert!(!state.has_pending_choice());

    let (mut self_damage, owner, _) = started(SHIJI, SPMINGSHI);
    let hand = self_damage.state().hand_count(owner);
    run_effects(
        &mut self_damage,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(self_damage.state().hand_count(owner), hand);
}
}

mod original_damage_immunity {
//! Golden for target-side nature-specific damage cancellation.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::state::CardKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CLBSHUIYONG: u32 = 4_055_999_596;
const SHIXIN: u32 = 3_071_284_185;
const PSYUHUO: u32 = 2_658_654_582;
const YIRAN: u32 = 3_082_356_765;
const OLYOUJI: u32 = 2_389_723_895;
const COLLAB: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn isolated(package: &[u8], skill: u32, seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|active| EffectRequest::RemoveSkill { skill: active.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, other] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    (tx, owner, other)
}

fn assert_fire_only(mut tx: Transaction, owner: PlayerId, other: PlayerId) {
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 10);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 8);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, 9);
}

#[test]
fn clbshuiyong_cancels_only_fire_damage_received_by_the_owner() {
    let (tx, owner, other) = isolated(COLLAB, CLBSHUIYONG, 20_260_720);
    assert_fire_only(tx, owner, other);
}

#[test]
fn shixin_uses_the_same_target_side_fire_immunity() {
    let (tx, owner, other) = isolated(MOBILE, SHIXIN, 20_260_721);
    assert_fire_only(tx, owner, other);
}

#[test]
fn psyuhuo_also_excludes_fire_sha_and_huogong_from_discard_to_limit() {
    let (tx, owner, other) = isolated(OFFLINE, PSYUHUO, 20_260_722);
    assert_fire_only(tx, owner, other);

    let (mut tx, owner, _) = isolated(OFFLINE, PSYUHUO, 20_260_723);
    let cards = tx.state().hand_cards(owner);
    assert!(cards.len() >= 3);
    tx.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::ShaFire;
    tx.state_mut().cards.get_mut(cards[1]).unwrap().kind = CardKind::TrickHuogong;
    tx.state_mut().cards.get_mut(cards[2]).unwrap().kind = CardKind::Shan;

    assert!(engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        cards[0]
    ));
    assert!(engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        cards[1]
    ));
    assert!(!engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        cards[2]
    ));
    let discardable = engine_core::query::discard_to_limit_tokens(tx.state(), owner);
    assert!(!discardable.contains(&cards[0]));
    assert!(!discardable.contains(&cards[1]));
    assert!(discardable.contains(&cards[2]));
}

#[test]
fn yiran_adds_one_only_to_fire_damage_received_by_the_owner() {
    let (mut tx, owner, other) = isolated(COLLAB, YIRAN, 20_260_724);
    for (nature, lose_hp, expected_hp) in [
        (rule_api::damage_nature::NORMAL, false, 9),
        (rule_api::damage_nature::THUNDER, false, 8),
        (rule_api::damage_nature::FIRE, false, 6),
        (rule_api::damage_nature::FIRE, true, 5),
    ] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: lose_hp,
                nature,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().players.get(owner).unwrap().hp, expected_hp);
    }

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, 9);
}

#[test]
fn olyouji_adds_the_round_number_to_draws_with_a_cap_of_five() {
    let (mut tx, owner, _) = isolated(COLLAB, OLYOUJI, 20_260_725);
    for (round, expected) in [(0, 2), (1, 3), (4, 6), (5, 7), (9, 7)] {
        tx.state_mut().turn.round = round;
        assert_eq!(
            engine_core::query::draw_count_for(tx.state(), owner),
            expected
        );
    }
}
}
