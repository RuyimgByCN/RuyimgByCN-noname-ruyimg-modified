//! Golden coverage for 贾诩「完杀」and the generic multi-rescuer dying-save queue.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIAXU: u32 = 3_399_795_312;
const LUANWU: u32 = 2_772_376_501;
const WANSHA: u32 = 2_730_287_699;
const WEIMU: u32 = 4_145_129_024;
const WANSHA_SOURCE: &str =
    include_str!("../../../../content/original/ported/shenhua/wansha.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    players: [PlayerId; 4],
}

fn setup(seed: u64, wansha: bool) -> Setup {
    let package = content_compiler::compile_source_named("wansha-test", WANSHA_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players: [PlayerId; 4] = tx.state().players.alive_by_seat().try_into().unwrap();
    let owner = players[0];
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
    if wansha {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WANSHA),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
        participant.dying = false;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        players,
    }
}

fn inject_tao(state: &mut GameState, owner: PlayerId, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_100 + rank as u32),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn open_save(state: GameState, dying: PlayerId, source: PlayerId) -> (GameState, PlayerId) {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RequestDyingSave {
                player: dying,
                source,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let observer = committed.output.choice.unwrap().observer;
    (committed.state, observer)
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_complete_jiaxu_and_exact_wansha_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == JIAXU)
        .unwrap();
    assert_eq!(character.name_key, "jiaxu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [LUANWU, WANSHA, WEIMU]);
    let modifier = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == WANSHA)
        .unwrap();
    assert_eq!(
        modifier.target,
        "tao_use_save_cancel_for_non_dying_others_during_owner_turn"
    );
    assert_eq!(modifier.mode, "override");
    assert_eq!(modifier.value, 1);
}

#[test]
fn dying_save_queue_starts_at_current_player_and_uses_multiple_rescuers() {
    let mut setup = setup(2_026_072_294, false);
    let [current, next, dying, _last] = setup.players;
    let first_tao = inject_tao(&mut setup.state, current, 2);
    let second_tao = inject_tao(&mut setup.state, next, 3);
    {
        let participant = setup.state.players.get_mut(dying).unwrap();
        participant.hp = -1;
        participant.dying = true;
    }
    let (state, observer) = open_save(setup.state, dying, next);
    assert_eq!(observer, current);
    let (state, first) = resolve(state, vec![1], false, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(dying).unwrap().hp, 0);
    assert_eq!(first.choice.as_ref().unwrap().observer, next);
    assert_eq!(
        state.cards.get(first_tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let (state, second) = resolve(state, vec![1], false, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let participant = state.players.get(dying).unwrap();
    assert_eq!(participant.hp, 1);
    assert!(participant.alive);
    assert!(!participant.dying);
    assert_eq!(
        state.cards.get(second_tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn wansha_blocks_other_non_dying_tao_but_keeps_owner_and_dying_exceptions() {
    let mut setup = setup(2_026_072_295, true);
    let [_owner, other, dying, last] = setup.players;
    let owner_tao = inject_tao(&mut setup.state, setup.owner, 4);
    let other_tao = inject_tao(&mut setup.state, other, 5);
    assert!(!engine_core::query::card_use_cancelled_for_card(
        &setup.state,
        setup.owner,
        owner_tao,
    ));
    assert!(engine_core::query::card_use_cancelled_for_card(
        &setup.state,
        other,
        other_tao,
    ));
    assert!(engine_core::query::card_save_cancelled_for_kind_to(
        &setup.state,
        other,
        dying,
        CardKind::Tao,
    ));

    setup.state.players.get_mut(other).unwrap().dying = true;
    assert!(!engine_core::query::card_use_cancelled_for_card(
        &setup.state,
        other,
        other_tao,
    ));
    assert!(engine_core::query::card_save_cancelled_for_kind_to(
        &setup.state,
        other,
        dying,
        CardKind::Tao,
    ));
    assert!(!engine_core::query::card_save_cancelled_for_kind_to(
        &setup.state,
        other,
        other,
        CardKind::Tao,
    ));
    setup.state.players.get_mut(other).unwrap().dying = false;
    setup.state.turn.current = last;
    assert!(!engine_core::query::card_use_cancelled_for_card(
        &setup.state,
        other,
        other_tao,
    ));
}

#[test]
fn wansha_save_queue_skips_blocked_helpers_and_still_allows_the_dying_player() {
    let mut setup = setup(2_026_072_296, true);
    let [_owner, blocked, dying, _last] = setup.players;
    let blocked_tao = inject_tao(&mut setup.state, blocked, 6);
    let dying_tao = inject_tao(&mut setup.state, dying, 7);
    {
        let participant = setup.state.players.get_mut(dying).unwrap();
        participant.hp = 0;
        participant.dying = true;
    }
    let (state, observer) = open_save(setup.state, dying, blocked);
    assert_eq!(observer, dying);
    assert_eq!(
        state.cards.get(blocked_tao).unwrap().zone,
        state.zones.find(ZoneKind::Hand, blocked).unwrap()
    );
    let (state, saved) = resolve(state, vec![1], false, 10);
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(dying).unwrap().hp, 1);
    assert!(!state.players.get(dying).unwrap().dying);
    assert_eq!(
        state.cards.get(dying_tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
