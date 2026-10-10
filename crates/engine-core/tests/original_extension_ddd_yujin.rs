//! Golden for 3D精选 于禁【整军】（ABI 627）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHENGJUN: u32 = 2_986_915_915;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/yujin.skill");

fn setup() -> (GameState, Vec<PlayerId>, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("ddd-yujin", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 627);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
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
        player: players[0],
        skill_def: SkillDefId::new(ZHENGJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx.state().players.get(players[0]).unwrap().skills[0];
    let mut state = tx.commit().state;
    state.turn.current = players[0];
    state.turn.phase = Phase::Play;
    (state, players, skill)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_627_000 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 3,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
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

#[test]
fn hp_category_recovers_matching_player_and_is_recorded() {
    let (mut state, players, skill) = setup();
    let (owner, actor) = (players[0], players[1]);
    state.players.get_mut(owner).unwrap().hp = 3;
    state.players.get_mut(actor).unwrap().hp = 3;
    state.players.get_mut(actor).unwrap().max_hp = 4;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddYujin {
                player: owner,
                actor,
                target: PlayerId::NONE,
                move_mask: 0,
                action: 0,
                candidates: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![0], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(actor).unwrap().hp, 4);
    assert_eq!(state.skills.get(skill).unwrap().play_phase_category_mask, 1);
}

#[test]
fn one_move_event_can_resolve_hand_then_equipment_categories() {
    let (mut state, players, skill) = setup();
    let (owner, actor, destination) = (players[0], players[1], players[2]);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Shan, 1);
    inject(&mut state, actor, ZoneKind::Hand, CardKind::Tao, 2);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Armor, 3);
    let weapon = inject(&mut state, actor, ZoneKind::Equip, CardKind::Weapon, 4);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddYujin {
                player: owner,
                actor,
                target: actor,
                move_mask: 1 | 8,
                action: 1,
                candidates: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, owner, vec![actor.raw()], 10);
    let (state, _) = choose(state, owner, vec![0], 11); // actor draws one
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let (state, _) = choose(state, owner, vec![actor.raw()], 12);
    let (state, _) = choose(state, owner, vec![weapon.raw()], 13);
    let (state, result) = choose(state, owner, vec![destination.raw()], 14);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let destination_equip = state.zones.find(ZoneKind::Equip, destination).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, destination_equip);
    assert_eq!(
        state.skills.get(skill).unwrap().play_phase_category_mask,
        2 | 4
    );
}

#[test]
fn hand_discard_branch_is_chosen_by_owner_but_paid_by_changed_player() {
    let (mut state, players, skill) = setup();
    let (owner, actor) = (players[0], players[1]);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Shan, 20);
    let actor_card = inject(&mut state, actor, ZoneKind::Hand, CardKind::Tao, 21);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddYujin {
                player: owner,
                actor,
                target: PlayerId::NONE,
                move_mask: 1,
                action: 1,
                candidates: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, owner, vec![actor.raw()], 20);
    let (state, branch) = choose(state, owner, vec![1], 21);
    assert_eq!(branch.choice.unwrap().observer, actor);
    let (state, result) = choose(state, actor, vec![actor_card.raw()], 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(actor).contains(&actor_card));
    assert_eq!(state.skills.get(skill).unwrap().play_phase_category_mask, 2);
}

#[test]
fn cards_moved_trigger_captures_destination_owner_and_both_zone_categories() {
    let (mut state, players, _) = setup();
    let (owner, actor) = (players[0], players[1]);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Shan, 30);
    inject(&mut state, actor, ZoneKind::Hand, CardKind::Tao, 31);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Armor, 32);
    let weapon = inject(&mut state, actor, ZoneKind::Hand, CardKind::Weapon, 33);
    let from = state.zones.find(ZoneKind::Hand, actor).unwrap();
    let to = state.zones.find(ZoneKind::Equip, actor).unwrap();
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![weapon],
                to,
                from: Some(from),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, accepted) = choose(tx.commit().state, owner, vec![1], 30);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let choice = accepted.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(
        choice
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![actor.raw()]
    );
    assert!(state.has_pending_choice());
}
