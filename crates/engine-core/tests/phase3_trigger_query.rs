//! Phase 3 acceptance: trigger index, stable order, modifiers, cache equivalence.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::{
    canonical_state_hash, query, query_explain, CardKind, Engine, Phase, QueryContext,
    TriggerOrderKey, BOOTSTRAP_JIANXIONG, BOOTSTRAP_MASHU, BOOTSTRAP_QICAO,
};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

fn start(seed: u64, n: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: "bootstrap".into(),
        },
    }
}

fn cmd(actor: PlayerId, kind: CommandKind) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind,
    }
}

#[test]
fn add_remove_skill_updates_trigger_index() {
    let mut eng = Engine::new();
    eng.submit(start(1, 2));
    // Phase 5 characters may already carry skills/triggers
    let base = eng.state().triggers.len();

    use engine_core::effect::run_effects;
    use engine_core::transaction::Transaction;

    let mut tx = Transaction::begin(eng.state().clone());
    let p = PlayerId::new(1);
    let before_skills = tx.state().players.get(p).unwrap().skills.len();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: p,
            skill_def: SkillDefId::new(BOOTSTRAP_JIANXIONG),
        }],
    )
    .unwrap();
    let committed = tx.commit();
    assert_eq!(committed.state.triggers.len(), base + 1);

    // Remove the skill we just added (last in list)
    let sid = *committed
        .state
        .players
        .get(p)
        .unwrap()
        .skills
        .last()
        .unwrap();
    let mut tx = Transaction::begin(committed.state);
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill: sid }]).unwrap();
    let committed = tx.commit();
    assert_eq!(committed.state.triggers.len(), base);
    assert_eq!(
        committed.state.players.get(p).unwrap().skills.len(),
        before_skills
    );
}

#[test]
fn jianxiong_ignores_cardless_damage() {
    use engine_core::effect::{run_effects, schedule_damage};
    use engine_core::transaction::Transaction;

    let mut eng = Engine::new();
    eng.submit(start(2, 2));
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);

    let mut tx = Transaction::begin(eng.state().clone());
    for player in [p1, p2] {
        for skill in tx.state().players.get(player).unwrap().skills.clone() {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: p2,
            skill_def: SkillDefId::new(BOOTSTRAP_JIANXIONG),
        }],
    )
    .unwrap();
    let hand_before = tx.state().hand_count(p2);
    // Non-lethal damage so we don't hit dying choice
    schedule_damage(&mut tx, p1, p2, 1).unwrap();
    let hand_after = tx.state().hand_count(p2);
    assert_eq!(hand_after, hand_before);
}

#[test]
fn trigger_order_keys_are_total_and_stable() {
    let a = TriggerOrderKey::new(engine_core::TimingBand::Normal, 0, false, 0, 1, 1);
    let b = TriggerOrderKey::new(engine_core::TimingBand::Normal, 0, false, 1, 1, 1);
    let c = TriggerOrderKey::new(engine_core::TimingBand::LastDo, 0, false, 0, 1, 1);
    assert!(a < b);
    assert!(a < c);
    assert!(b < c);
    let mut v = vec![c, b, a];
    v.sort();
    assert_eq!(v, vec![a, b, c]);
}

#[test]
fn simultaneous_triggers_seat_order_deterministic() {
    use engine_core::effect::run_effects;
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{
        bootstrap_trigger_defs, seat_rel_to_current, ActiveTriggerIndex, EventKind,
    };

    // Two players both with jianxiong; only owner-is-target fires for one damage.
    // Verify collect_sorted order is deterministic for same inputs.
    let mut idx = ActiveTriggerIndex::new();
    let content = engine_core::ContentRegistry::new();
    idx.register_skill(
        engine_api::SkillId::new(1),
        SkillDefId::new(BOOTSTRAP_JIANXIONG),
        PlayerId::new(1),
        &content,
    );
    idx.register_skill(
        engine_api::SkillId::new(2),
        SkillDefId::new(BOOTSTRAP_JIANXIONG),
        PlayerId::new(2),
        &content,
    );
    let alive = vec![PlayerId::new(1), PlayerId::new(2)];
    let current = PlayerId::new(1);
    let ordered: Vec<_> = idx
        .collect_sorted(EventKind::Damage, |active| {
            let seat_rel = seat_rel_to_current(&alive, current, active.owner);
            TriggerOrderKey::new(
                engine_core::TimingBand::Normal,
                0,
                false,
                seat_rel,
                active.skill_id.raw(),
                active.trigger_def.raw(),
            )
        })
        .into_iter()
        .map(|(k, a)| (k, a.owner.raw()))
        .collect();
    let ordered2: Vec<_> = idx
        .collect_sorted(EventKind::Damage, |active| {
            let seat_rel = seat_rel_to_current(&alive, current, active.owner);
            TriggerOrderKey::new(
                engine_core::TimingBand::Normal,
                0,
                false,
                seat_rel,
                active.skill_id.raw(),
                active.trigger_def.raw(),
            )
        })
        .into_iter()
        .map(|(k, a)| (k, a.owner.raw()))
        .collect();
    assert_eq!(ordered, ordered2);
    // Current seat (rel 0) before the other
    assert_eq!(ordered[0].1, 1);
    assert_eq!(ordered[1].1, 2);
    let _ = bootstrap_trigger_defs();
    let _ = run_effects;
    let _ = Transaction::begin;
}

#[test]
fn qicao_increases_attack_range_with_explain() {
    use engine_core::effect::run_effects;
    use engine_core::transaction::Transaction;

    let mut eng = Engine::new();
    eng.submit(start(3, 2));
    let p1 = PlayerId::new(1);
    let mut tx = Transaction::begin(eng.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: p1,
            skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
        }],
    )
    .unwrap();
    let mut ctx = QueryContext::new();
    let exp = query_explain(
        tx.state(),
        &QueryRequest::AttackRange { player: p1 },
        &mut ctx,
    );
    assert_eq!(exp.base, Some(1));
    assert_eq!(exp.response, QueryResponse::Int(2));
    // Bootstrap label uses "qicao"; package overlay uses pkg.<target>.<skill_def_id>.
    assert!(exp
        .sources
        .iter()
        .any(|s| s.label_key.contains("qicao") || s.label_key.contains("attack_range")));
}

#[test]
fn mashu_distance_and_cache_equivalence() {
    use engine_core::effect::run_effects;
    use engine_core::transaction::Transaction;

    let mut eng = Engine::new();
    eng.submit(start(4, 4));
    let p1 = PlayerId::new(1);
    let p3 = PlayerId::new(3);
    let mut tx = Transaction::begin(eng.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: p1,
            skill_def: SkillDefId::new(BOOTSTRAP_MASHU),
        }],
    )
    .unwrap();
    let state = tx.state();
    let req = QueryRequest::Distance { from: p1, to: p3 };

    let mut off = QueryContext::with_cache(false);
    let a = query(state, &req, &mut off);
    let mut on = QueryContext::with_cache(true);
    let b = query(state, &req, &mut on);
    let c = query(state, &req, &mut on);
    assert_eq!(a, b);
    assert_eq!(b, c);
    on.clear_cache();
    let d = query(state, &req, &mut on);
    assert_eq!(a, d);
}

#[test]
fn card_usable_and_target_legal() {
    let mut eng = Engine::new();
    eng.submit(start(5, 2));
    let actor = eng.state().turn.current;
    // EndPhase Draw -> Play
    eng.submit(cmd(actor, CommandKind::EndPhase));
    assert_eq!(eng.state().turn.phase, Phase::Play);
    let hand = eng.state().hand_cards(actor);
    assert!(!hand.is_empty());
    let card = hand[0];
    let mut ctx = QueryContext::new();
    let usable = query(
        eng.state(),
        &QueryRequest::CardUsable {
            player: actor,
            card,
        },
        &mut ctx,
    );
    // Shan may be false; others often true
    assert!(matches!(usable, QueryResponse::Bool(_)));

    if eng.state().cards.get(card).map(|c| c.kind) == Some(CardKind::Sha) {
        let target = eng
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .find(|p| *p != actor)
            .unwrap();
        let legal = query(
            eng.state(),
            &QueryRequest::TargetLegal {
                user: actor,
                card,
                target,
            },
            &mut ctx,
        );
        assert_eq!(legal, QueryResponse::Bool(true));
    }
}

#[test]
fn snapshot_rebuilds_indexes() {
    use engine_core::effect::run_effects;
    use engine_core::transaction::Transaction;

    let mut eng = Engine::new();
    eng.submit(start(6, 2));
    let mut tx = Transaction::begin(eng.state().clone());
    // Roster-independent: booted standard characters may carry triggers;
    // the snapshot must contain exactly the granted jianxiong skill.
    let booted: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if !booted.is_empty() {
        run_effects(&mut tx, booted).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: PlayerId::new(1),
            skill_def: SkillDefId::new(BOOTSTRAP_JIANXIONG),
        }],
    )
    .unwrap();
    let committed = tx.commit();
    eng.restore_snapshot(&{
        let mut s = eng.snapshot();
        s.state = committed.state.clone();
        s.revision = committed.state.revision;
        s.state_hash = canonical_state_hash(&committed.state);
        s
    })
    .unwrap();
    assert_eq!(eng.state().triggers.len(), 1);
    assert_eq!(eng.state_hash(), canonical_state_hash(eng.state()));
}

#[test]
fn play_uses_modifier_aware_range() {
    // With qicao range +1, distance-2 target becomes legal if base range 1.
    // 4 players: seat distance between 1 and 3 is 2.
    use engine_core::effect::run_effects;
    use engine_core::transaction::Transaction;

    let mut eng = Engine::new();
    eng.submit(start(7, 4));
    let p1 = PlayerId::new(1);
    let mut tx = Transaction::begin(eng.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: p1,
            skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
        }],
    )
    .unwrap();
    let mut ctx = QueryContext::new();
    let range = query(
        tx.state(),
        &QueryRequest::AttackRange { player: p1 },
        &mut ctx,
    );
    assert_eq!(range, QueryResponse::Int(2));
}
