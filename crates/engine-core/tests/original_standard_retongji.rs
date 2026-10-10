//! Golden coverage for Standard 同疾 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RETONGJI: u32 = 2_773_091_493;
const RE_YUANSHU: u32 = 3_366_235_136;
const SOURCE: &str = include_str!("../../../content/standard/retongji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(with_owner_shan: bool) -> (GameState, PlayerId, PlayerId, PlayerId, CardId, CardId) {
    let package = compile_source_named("standard-retongji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_725_62, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let source = players[0];
    let target = players[1];
    let owner = players[2];
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
        skill_def: SkillDefId::new(RETONGJI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject(&mut tx, target, CardKind::Other, Suit::Club, 2);
    if with_owner_shan {
        inject(&mut tx, owner, CardKind::Shan, Suit::Heart, 3);
    }
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 4);
    (tx.commit().state, source, target, owner, sha, cost)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    sha: CardId,
) -> (GameState, PipelineResult) {
    submit(
        state,
        source,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    )
}

#[test]
fn package_closes_re_yuanshu_with_both_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RETONGJI)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_sha_target_can_redirect_to_owner"
    );
    assert!(!trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_YUANSHU)
        .unwrap();
    assert_eq!(character.skills, [3_817_957_374, RETONGJI]);
}

#[test]
fn target_may_discard_to_redirect_sha_to_owner_outside_source_range() {
    let (state, source, target, owner, sha, cost) = setup(true);
    assert!(!engine_core::query::player_in_attack_range(
        &state, source, owner
    ));
    assert!(engine_core::query::player_in_attack_range(
        &state, target, owner
    ));
    let target_hp = state.players.get(target).unwrap().hp;
    let owner_hp = state.players.get(owner).unwrap().hp;
    let (state, offered) = play(state, source, target, sha);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == cost.raw()));
    let (state, redirected) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![cost.raw()],
            cancelled: false,
        },
        2,
    );
    let response = redirected.choice.unwrap();
    assert_eq!(response.observer, owner);
    let shan = state
        .hand_cards(owner)
        .into_iter()
        .find(|card| state.cards.get(*card).unwrap().kind == CardKind::Shan)
        .unwrap();
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: response.choice_id,
            selections: vec![shan.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    assert!(!state.hand_cards(target).contains(&cost));
}

#[test]
fn declining_cost_keeps_the_original_sha_target() {
    let (state, source, target, owner, sha, cost) = setup(false);
    let target_hp = state.players.get(target).unwrap().hp;
    let owner_hp = state.players.get(owner).unwrap().hp;
    let (state, offered) = play(state, source, target, sha);
    let choice = offered.choice.unwrap();
    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: Vec::new(),
            cancelled: true,
        },
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    assert!(state.hand_cards(target).contains(&cost));
}
