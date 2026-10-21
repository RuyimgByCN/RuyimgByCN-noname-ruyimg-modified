//! Golden for 标岑昏「极奢」「无度」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::query::{query, QueryContext};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STDJISHE: u32 = 2_148_577_926;
const STDWUDU: u32 = 4_151_205_179;
const STD_CENHUN: u32 = 3_739_444_871;
const JISHE_SOURCE: &str =
    include_str!("../../../../content/original/ported/sixiang/stdjishe.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    source: PlayerId,
    jishe: engine_api::SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(empty_target_hand: bool, seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let source = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if empty_target_hand {
        effects.push(EffectRequest::DiscardFromHand {
            player: target,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDJISHE),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDWUDU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 3;
    participant.hp = 3;
    let jishe = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDJISHE)
        .unwrap()
        .id;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        source,
        jishe,
    }
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::default(),
    ) {
        QueryResponse::Int(limit) => limit,
        response => panic!("unexpected hand-limit response: {response:?}"),
    }
}

fn activate_jishe(setup: &Setup, state: GameState, key: u64) -> (GameState, CommandResult) {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.jishe,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    (state, result.result)
}

fn resolve_optional(state: GameState, owner: PlayerId, accept: bool, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_abi143_and_complete_std_cenhun() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDJISHE)
        .unwrap();
    assert!(active.hand_limit_positive);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == STDJISHE
            && modifier.target == "hand_limit"
            && modifier.value_source.as_deref() == Some("owner_skill_use_count")
            && modifier.value == -1
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDWUDU
            && trigger.event == "damage_before"
            && trigger.filter == "owner_alive_event_player_hand_empty"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_CENHUN)
        .unwrap();
    assert_eq!(character.name_key, "std_cenhun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDJISHE, STDWUDU]);
    assert_eq!(
        content_compiler::compile_source_named("stdjishe", JISHE_SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        143
    );
}

#[test]
fn stdjishe_draws_until_current_hand_limit_reaches_zero() {
    let setup = setup(false, 2_026_072_411);
    let initial_hand = setup.state.hand_count(setup.owner);
    let mut state = setup.state.clone();
    assert_eq!(hand_limit(&state, setup.owner), 3);
    for key in 1..=3 {
        let (next, result) = activate_jishe(&setup, state, key);
        assert!(matches!(result, CommandResult::Accepted { .. }));
        state = next;
        assert_eq!(hand_limit(&state, setup.owner), 3 - key as i32);
        assert_eq!(state.hand_count(setup.owner), initial_hand + key as u32);
    }
    let before = canonical_state_hash(&state);
    let (same, result) = activate_jishe(&setup, state, 4);
    assert!(matches!(result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn stdwudu_accept_prevents_empty_hand_target_damage_and_loses_owner_max_hp() {
    let setup = setup(true, 2_026_072_412);
    let target_hp = setup.state.players.get(setup.target).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve_optional(tx.commit().state, setup.owner, true, 10);
    assert_eq!(state.players.get(setup.owner).unwrap().max_hp, 2);
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp);
}

#[test]
fn stdwudu_decline_or_nonempty_hand_allows_damage_without_cost() {
    let declined = setup(true, 2_026_072_413);
    let target_hp = declined.state.players.get(declined.target).unwrap().hp;
    let mut tx = Transaction::begin(declined.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: declined.source,
                target: declined.target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve_optional(tx.commit().state, declined.owner, false, 20);
    assert_eq!(state.players.get(declined.owner).unwrap().max_hp, 3);
    assert_eq!(
        state.players.get(declined.target).unwrap().hp,
        target_hp - 1
    );

    let nonempty = setup(false, 2_026_072_414);
    let target_hp = nonempty.state.players.get(nonempty.target).unwrap().hp;
    let mut tx = Transaction::begin(nonempty.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: nonempty.source,
            target: nonempty.target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(nonempty.owner).unwrap().max_hp, 3);
    assert_eq!(
        tx.state().players.get(nonempty.target).unwrap().hp,
        target_hp - 1
    );
}
