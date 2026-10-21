//! Golden for 3D精选 辛宪英【洞察/智解】（ABI 625）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DONGCHA: u32 = 3_036_718_178;
const ZHIJIE: u32 = 1_168_942_044;
const SOURCE: &str = include_str!("../../../../content/original/ported/ddd/xinxianying.skill");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_625_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-xinxianying", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
    effects.extend([DONGCHA, ZHIJIE].map(|def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn place_two_jian(
    mut state: GameState,
    owner: PlayerId,
    actor: PlayerId,
    first: CardId,
    second: CardId,
) -> GameState {
    let skill = skill_id(&state, owner, DONGCHA);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXinxianying {
                player: owner,
                actor,
                action: 0,
                skill,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    state = tx.commit().state;
    let (state, _) = resolve(state, owner, vec![first.raw()], 1);
    let (state, done) = resolve(state, actor, vec![second.raw()], 2);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn dongcha_stores_then_owner_and_turn_player_take_one_jian_each() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    let first = inject(&mut state, owner, CardKind::Other, Suit::Heart, 1);
    let second = inject(&mut state, actor, CardKind::Other, Suit::Club, 2);
    let state = place_two_jian(state, owner, actor, first, second);
    assert_eq!(state.skill_pile_cards(owner, DONGCHA).len(), 2);
    let skill = skill_id(&state, owner, DONGCHA);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXinxianying {
                player: owner,
                actor,
                action: 1,
                skill,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, _) = resolve(state, owner, vec![first.raw()], 3);
    let (state, done) = resolve(state, actor, vec![second.raw()], 4);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(state.skill_pile_cards(owner, DONGCHA).is_empty());
    assert_eq!(state.cards.get(first).unwrap().owner, owner);
    assert_eq!(state.cards.get(second).unwrap().owner, actor);
}

#[test]
fn zhijie_same_color_jian_respond_as_shan_and_draw_two() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    let first = inject(&mut state, owner, CardKind::Other, Suit::Heart, 10);
    let second = inject(&mut state, actor, CardKind::Other, Suit::Diamond, 11);
    let attack = inject(&mut state, actor, CardKind::Sha, Suit::Spade, 12);
    for serial in 20..22 {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(99_625_000 + serial),
            kind: CardKind::Other,
            suit: Suit::Club,
            rank: 7,
            weapon_range: 0,
            zone: draw,
            owner: PlayerId::NONE,
            face_up: true,
        });
        state.zones.get_mut(draw).unwrap().cards.push(card);
    }
    let mut state = place_two_jian(state, owner, actor, first, second);
    state.turn.current = actor;
    state.turn.phase = Phase::Play;
    let hp = state.players.get(owner).unwrap().hp;
    let (state, response) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        10,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let conversion = response
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .find(|token| !matches!(*token, 0 | 1))
        .unwrap();
    let (state, source_choice) = resolve(state, owner, vec![conversion], 11);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let offered: Vec<_> = source_choice
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(offered, vec![first.raw(), second.raw()]);
    let (state, done) = resolve(state, owner, vec![first.raw(), second.raw()], 12);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(owner), 2);
    assert!(state.skill_pile_cards(owner, DONGCHA).is_empty());
}
