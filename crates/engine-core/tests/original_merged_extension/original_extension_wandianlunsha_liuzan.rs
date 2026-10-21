//! Golden for archived extension 玩点论杀 留赞【亢音】（ABI 612）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const KANGYIN: u32 = 2_250_380_873;
const KANGYIN2: u32 = 2_524_514_324;
const SOURCE: &str = include_str!("../../../../content/original/ported/wandianlunsha/liuzan.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_120 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
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
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

fn setup(kind: CardKind, hp: i32) -> (GameState, Vec<PlayerId>, engine_api::SkillId, CardId) {
    let package = content_compiler::compile_source_named("wandianlunsha-liuzan", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 612);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(KANGYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 4;
    participant.hp = hp;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let card = inject(tx.state_mut(), target, kind, 1);
    let kangyin = skill(tx.state(), owner, KANGYIN);
    (tx.commit().state, players, kangyin, card)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn kangyin_basic_draws_optional_targets_and_nonbasic_grants_turn_modifiers() {
    let (state, players, kangyin, basic) = setup(CardKind::Shan, 2);
    let owner = players[0];
    let target = players[1];
    let state = activate(state, owner, target, kangyin);
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    let state = choose(state, owner, vec![basic.raw()], 2);
    assert!(state.has_pending_choice());
    let owner_hand = state.hand_count(owner);
    let recipient_hand = state.hand_count(players[2]);
    let state = choose(state, owner, vec![owner.raw(), players[2].raw()], 3);
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(players[2]), recipient_hand + 1);

    let (state, players, kangyin, trick) = setup(CardKind::TrickWuzhong, 4);
    let owner = players[0];
    let target = players[1];
    let opposite = players[2];
    let base_target_limit = engine_core::query::sha_target_limit_for(&state, owner);
    assert!(!engine_core::query::player_in_attack_range(
        &state, owner, opposite
    ));
    let state = activate(state, owner, target, kangyin);
    let state = choose(state, owner, vec![trick.raw()], 4);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(
        engine_core::query::sha_target_limit_for(&state, owner),
        base_target_limit + 1
    );
    assert!(engine_core::query::player_in_attack_range(
        &state, owner, opposite
    ));
    let helper = skill(&state, owner, KANGYIN2);
    let instance = state.skills.get(helper).unwrap();
    assert_eq!(instance.state_blob, 1u32.to_le_bytes());
    assert_eq!(instance.temporary_until_turn_end_of, owner);
}
