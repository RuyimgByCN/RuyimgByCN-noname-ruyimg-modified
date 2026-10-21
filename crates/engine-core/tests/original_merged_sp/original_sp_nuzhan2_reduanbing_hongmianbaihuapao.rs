//! Golden for three SP skills rewritten from legacy JavaScript semantics:
//! - 怒斩2 (nuzhan2): locked — a Sha whose only material is one equipment card deals +1 base damage.
//! - 断兵 (reduanbing): optional — after using a Sha, add one legal target within distance 1.
//! - 红棉百花袍 (hongmianbaihuapao_skill): locked equip skill — chain-propagated damage is prevented.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_card_flag, damage_nature, EffectRequest};

const NUZHAN2: u32 = 3_186_808_257;
const REDUANBING: u32 = 3_958_105_675;
const HONGMIANBAIHUAPAO: u32 = 2_276_671_070;
const UNLIMITED_SHA: u32 = 4_290_000_277;
const NEW_REWUSHENG: u32 = 3_038_912_803;
const NUZHAN2_SOURCE: &str = include_str!("../../../../content/original/ported/sp/nuzhan2.skill");
const REDUANBING_SOURCE: &str =
    include_str!("../../../../content/original/ported/sp/reduanbing.skill");
const HONGMIAN_SOURCE: &str =
    include_str!("../../../../content/original/ported/sp/hongmianbaihuapao_skill.skill");
const REWUSHENG_SOURCE: &str =
    include_str!("../../../../content/original/ported/refresh/new_rewusheng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn skill_id(
    state: &engine_core::state::GameState,
    player: PlayerId,
    def: u32,
) -> engine_api::SkillId {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

/// Drive every pending choice (each target declines 闪) until no choice remains.
fn run_to_stable(
    mut state: engine_core::state::GameState,
    mut key: u64,
) -> (engine_core::state::GameState, u64) {
    while state.has_pending_choice() {
        let (next, r) = apply_command(
            state.clone(),
            Command {
                actor: PlayerId::new(state.resolution.pending_observer),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(r.result, CommandResult::Accepted { .. }),
            "{:?}",
            r.result
        );
        state = next;
        key += 1;
    }
    (state, key)
}

#[test]
fn package_declares_all_three_rewritten_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == NUZHAN2
            && t.event == "card_used"
            && t.filter == "owner_source_sha_single_equip_source_card"
            && !t.optional
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == REDUANBING
            && t.event == "card_used"
            && t.filter == "owner_source_sha"
            && t.optional
    }));
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == HONGMIANBAIHUAPAO
            && t.event == "damage_before"
            && t.filter == "owner_is_target_chain_damage_linked"
            && !t.optional
    }));
    // No stale noop modifier remains for these defs.
    assert!(!pkg.modifier_index.iter().any(|m| {
        m.skill_def_id == NUZHAN2
            || m.skill_def_id == REDUANBING
            || m.skill_def_id == HONGMIANBAIHUAPAO
    }));
}

#[test]
fn nuzhan2_equip_material_sha_boosts_base_damage() {
    let source = format!(
        "{NUZHAN2_SOURCE}\n{REWUSHENG_SOURCE}\nskill unlimited_sha(4290000277) {{ modifier card_distance_unlimited(sha) override 1; }}"
    );
    let compiled = content_compiler::compile_source_named("nuzhan2-test", &source).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&compiled).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 2_026_081_101,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NUZHAN2),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NEW_REWUSHENG),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(UNLIMITED_SHA),
            },
        ])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for seat in tx.state().players.alive_by_seat() {
        let participant = tx.state_mut().players.get_mut(seat).unwrap();
        participant.max_hp = 4;
        participant.hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    // A red equipment card converted into a Sha: the only use material is this equip.
    let equip = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_278),
        kind: CardKind::Weapon,
        suit: Suit::Heart,
        rank: 3,
        weapon_range: 2,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(equip);

    let view_as = skill_id(tx.state(), owner, NEW_REWUSHENG);
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: view_as,
                targets: vec![target],
                card_targets: vec![equip],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let (state, _) = run_to_stable(state, 2);
    // Base 1 + nuzhan2 +1 = 2 damage.
    assert_eq!(state.players.get(target).unwrap().hp, 2);
}

#[test]
fn reduanbing_adds_one_distance_one_target_to_used_sha() {
    let compiled =
        content_compiler::compile_source_named("reduanbing-test", REDUANBING_SOURCE).unwrap();
    let fixture = content_compiler::compile_source_named(
        "unlimited-sha-fixture",
        "skill unlimited_sha(4290000277) { modifier card_distance_unlimited(sha) override 1; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&compiled).unwrap();
    engine.load_package(&fixture).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 2_026_081_102,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let first = seats[(owner_index + 1) % seats.len()];
    let far = seats[(owner_index + 2) % seats.len()];
    let near = seats[(owner_index + 3) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(REDUANBING),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(UNLIMITED_SHA),
            },
        ])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for seat in &seats {
        let participant = tx.state_mut().players.get_mut(*seat).unwrap();
        participant.max_hp = 4;
        participant.hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_279),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);

    let (state, offered) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![first],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    // The optional trigger asks the owner to confirm first.
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let confirm = offered.choice.unwrap();
    assert_eq!(confirm.observer, owner);
    let (state, accepted) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: confirm.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentShaTarget
    );
    let choice = accepted.choice.unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == near.raw()));
    assert!(choice
        .options
        .iter()
        .all(|option| option.token != far.raw()));

    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![near.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { target, .. } if *target == near
    )));
}

#[test]
fn hongmianbaihuapao_prevents_only_chain_propagated_damage() {
    let compiled =
        content_compiler::compile_source_named("hongmian-test", HONGMIAN_SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&compiled).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 2_026_081_103,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HONGMIANBAIHUAPAO),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for seat in tx.state().players.alive_by_seat() {
        let participant = tx.state_mut().players.get_mut(seat).unwrap();
        participant.max_hp = 4;
        participant.hp = 4;
    }

    let initial = tx.state().players.get(owner).unwrap().hp;

    // Direct fire damage (not chain-propagated) must still apply.
    let direct = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::TrickHuogong.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(direct
        .iter()
        .any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, initial - 1);

    // Chain-propagated elemental damage is prevented.
    let chained = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::THUNDER,
            card: CardId::NONE,
            card_kind: CardKind::TrickShandian.tag(),
            card_flags: damage_card_flag::CHAIN_PROPAGATED,
        }],
    )
    .unwrap();
    assert!(!chained
        .iter()
        .any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, initial - 1);
}
