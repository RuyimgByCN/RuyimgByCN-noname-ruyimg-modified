//! Golden coverage for 标羊祜「明伐」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STD_DC_YANGHU: u32 = 3_430_968_343;
const STDMINGFA: u32 = 2_715_469_127;
const STDMINGFA_USED: u32 = 2_582_921_343;
const DAMAGE_BOOST: u32 = 4_000_000_001;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdmingfa.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn setup(boost_damage: bool) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_408, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    if boost_damage {
        let boost = content_compiler::compile_source(
            "skill stdmingfa_test_damage_boost(4000000001) locked { modifier damage_amount add 1; }",
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&boost).unwrap();
    }
    let players = tx.state().players.alive_by_seat();
    let (owner, target, other) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDMINGFA),
    });
    if boost_damage {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DAMAGE_BOOST),
        });
    }
    effects.push(EffectRequest::SetTurnState {
        current: owner,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
        participant.dying = false;
    }
    let skill = skill_id(tx.state(), owner, STDMINGFA).unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

fn run(state: GameState, effects: Vec<EffectRequest>) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn inject_tao(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(132_001),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 3,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn assert_bound_target(state: &GameState, skill: SkillId, target: PlayerId) {
    let mut expected = b"BND1".to_vec();
    expected.extend_from_slice(&target.raw().to_le_bytes());
    assert_eq!(state.skills.get(skill).unwrap().state_blob, expected);
}

#[test]
fn package_publishes_abi132_skills_and_complete_std_dc_yanghu() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDMINGFA)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, None);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self, targets.hp_gt),
        (1, 1, true, Some(1))
    );
    for event in ["hp_recovered", "death"] {
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == STDMINGFA_USED
                && trigger.event == event
                && trigger.filter == "owner_alive_skill_bound_target_is_event_player"
                && !trigger.optional
        }));
    }
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_DC_YANGHU)
        .unwrap();
    assert_eq!(character.name_key, "std_dc_yanghu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDMINGFA]);

    let compiled = content_compiler::compile_source_named("stdmingfa", SOURCE).unwrap();
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn surviving_target_disables_skill_until_that_target_recovers_or_dies() {
    let mut first = setup(false);
    first.state.players.get_mut(first.target).unwrap().hp = 3;
    first.state.players.get_mut(first.other).unwrap().hp = 3;
    let (state, result) = activate(first.state, first.owner, first.target, first.skill, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        } if *source == first.owner && *target == first.target
    )));
    assert_eq!(state.players.get(first.target).unwrap().hp, 2);
    assert!(skill_id(&state, first.owner, STDMINGFA).is_none());
    let helper = skill_id(&state, first.owner, STDMINGFA_USED).unwrap();
    assert_bound_target(&state, helper, first.target);

    let before_retry = canonical_state_hash(&state);
    let (state, retry) = activate(state, first.owner, first.target, first.skill, 2);
    assert!(matches!(retry.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before_retry);

    let state = run(
        state,
        vec![EffectRequest::RecoverHp {
            source: first.other,
            player: first.other,
            amount: 1,
        }],
    );
    assert!(skill_id(&state, first.owner, STDMINGFA).is_none());
    assert_eq!(skill_id(&state, first.owner, STDMINGFA_USED), Some(helper));

    let state = run(
        state,
        vec![EffectRequest::RecoverHp {
            source: first.owner,
            player: first.target,
            amount: 1,
        }],
    );
    assert!(skill_id(&state, first.owner, STDMINGFA).is_some());
    assert!(skill_id(&state, first.owner, STDMINGFA_USED).is_none());

    let mut second = setup(false);
    second.state.players.get_mut(second.target).unwrap().hp = 3;
    let (state, result) = activate(second.state, second.owner, second.target, second.skill, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = run(
        state,
        vec![EffectRequest::KillPlayer {
            player: second.target,
            killer: second.owner,
        }],
    );
    assert!(skill_id(&state, second.owner, STDMINGFA).is_some());
    assert!(skill_id(&state, second.owner, STDMINGFA_USED).is_none());
}

#[test]
fn literal_hp_threshold_rejects_one_hp_but_allows_self() {
    let mut illegal = setup(false);
    illegal.state.players.get_mut(illegal.target).unwrap().hp = 1;
    let before = canonical_state_hash(&illegal.state);
    let (same, result) = activate(
        illegal.state,
        illegal.owner,
        illegal.target,
        illegal.skill,
        10,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let legal = setup(false);
    let owner_hp = legal.state.players.get(legal.owner).unwrap().hp;
    let (state, result) = activate(legal.state, legal.owner, legal.owner, legal.skill, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(legal.owner).unwrap().hp, owner_hp - 1);
    let helper = skill_id(&state, legal.owner, STDMINGFA_USED).unwrap();
    assert_bound_target(&state, helper, legal.owner);
}

#[test]
fn lethal_damage_keeps_skill_but_dying_save_finishes_before_disabling_it() {
    let mut lethal = setup(true);
    lethal.state.players.get_mut(lethal.target).unwrap().hp = 2;
    let (state, result) = activate(lethal.state, lethal.owner, lethal.target, lethal.skill, 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(lethal.target).unwrap().alive);
    assert!(skill_id(&state, lethal.owner, STDMINGFA).is_some());
    assert!(skill_id(&state, lethal.owner, STDMINGFA_USED).is_none());

    let mut saved = setup(true);
    saved.state.players.get_mut(saved.target).unwrap().hp = 2;
    inject_tao(&mut saved.state, saved.target);
    let (state, result) = activate(saved.state, saved.owner, saved.target, saved.skill, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DyingSave
    );
    assert!(skill_id(&state, saved.owner, STDMINGFA).is_some());
    assert!(skill_id(&state, saved.owner, STDMINGFA_USED).is_none());
    assert!(result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == 1));

    let (state, result) = resolve(state, vec![1], 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let target = state.players.get(saved.target).unwrap();
    assert!(target.alive);
    assert!(!target.dying);
    assert_eq!(target.hp, 1);
    assert!(skill_id(&state, saved.owner, STDMINGFA).is_none());
    let helper = skill_id(&state, saved.owner, STDMINGFA_USED).unwrap();
    assert_bound_target(&state, helper, saved.target);
}
