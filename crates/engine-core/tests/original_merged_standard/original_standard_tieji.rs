//! Golden coverage for Standard 铁骑 rewritten from the legacy JavaScript semantics.

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

const TIEJI: u32 = 3_561_072_419;
const SOURCE: &str = include_str!("../../../../content/standard/tieji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, judgment_suit: Suit) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = compile_source_named("standard-tieji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TIEJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        2,
    );
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        judgment_suit,
        7,
    );
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        8,
    );
    (tx.commit().state, owner, target, sha, shan)
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

fn play_and_accept(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
) -> (GameState, PipelineResult) {
    let (state, offered) = submit(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    );
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(choice.observer, owner);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![1],
            cancelled: false,
        },
        2,
    )
}

#[test]
fn package_closes_machao_with_exact_optional_sha_target_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TIEJI)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_source_sha");
    assert!(trigger.optional);
    let machao = package
        .character_index
        .iter()
        .find(|character| character.name_key == "machao")
        .unwrap();
    assert!(machao.skills.contains(&TIEJI));
}

#[test]
fn red_judgment_blocks_shan_but_black_judgment_keeps_the_response_window() {
    let (state, owner, target, sha, shan) = setup(20_260_725_51, Suit::Heart);
    let hp = state.players.get(target).unwrap().hp;
    let (red, result) = play_and_accept(state, owner, target, sha);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(red.players.get(target).unwrap().hp, hp - 1);
    assert!(red.hand_cards(target).contains(&shan));

    let (state, owner, target, sha, shan) = setup(20_260_725_52, Suit::Spade);
    let hp = state.players.get(target).unwrap().hp;
    let (state, result) = play_and_accept(state, owner, target, sha);
    let response = result.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: response.choice_id,
            selections: vec![shan.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp);
}
