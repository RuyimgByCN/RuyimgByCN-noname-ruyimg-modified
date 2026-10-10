//! Golden for 3D精选 甄宓【淼形/浮世】（ABI 630）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MIAOXING: u32 = 505_834_181;
const MIAOXING_1: u32 = 919_529_013;
const MIAOXING_2: u32 = 2_949_101_455;
const FUSHI: u32 = 1_483_827_339;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/zhenji.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-zhenji", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 630);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    tx.state_mut()
        .players
        .get_mut(players[0])
        .unwrap()
        .character_def = 25;
    tx.state_mut()
        .players
        .get_mut(players[1])
        .unwrap()
        .character_def = 1;
    tx.state_mut()
        .players
        .get_mut(players[2])
        .unwrap()
        .character_def = 2;
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
    effects.extend([MIAOXING, FUSHI].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
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

fn inject(state: &mut GameState, owner: PlayerId, serial: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_630_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
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

fn initialize(mut state: GameState, owner: PlayerId, count: u32) -> (GameState, Vec<CardId>) {
    let cards: Vec<_> = (0..count)
        .map(|serial| inject(&mut state, owner, serial))
        .collect();
    let skill = skill_id(&state, owner, MIAOXING);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhenji {
                player: owner,
                target: PlayerId::NONE,
                action: 0,
                stage: 0,
                piles: Vec::new(),
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![0], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    (state, cards)
}

#[test]
fn miaoxing_triples_initial_hand_then_keeps_one_group_and_hides_two() {
    let (state, players) = setup();
    let owner = players[0];
    assert_eq!(
        engine_core::query::initial_hand_count_for(&state, owner, 4),
        12
    );
    let (state, cards) = initialize(state, owner, 12);
    assert_eq!(state.hand_cards(owner), cards[..4]);
    assert_eq!(
        state.hidden_skill_pile_cards(owner, MIAOXING_1),
        cards[4..8]
    );
    assert_eq!(state.hidden_skill_pile_cards(owner, MIAOXING_2), cards[8..]);
}

#[test]
fn miaoxing_balances_both_selected_piles_to_current_hand_count() {
    let (state, players) = setup();
    let owner = players[0];
    let (mut state, _) = initialize(state, owner, 9);
    let pile_1 = state.skill_pile_cards(owner, MIAOXING_1);
    let pile_2_zone = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == MIAOXING_2)
        .unwrap()
        .zone;
    for card in &pile_1[..2] {
        let from = state.cards.get(*card).unwrap().zone;
        state
            .zones
            .get_mut(from)
            .unwrap()
            .cards
            .retain(|candidate| candidate != card);
        state.zones.get_mut(pile_2_zone).unwrap().cards.push(*card);
        state.cards.get_mut(*card).unwrap().zone = pile_2_zone;
    }
    let skill = skill_id(&state, owner, MIAOXING);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhenji {
                player: owner,
                target: PlayerId::NONE,
                action: 1,
                stage: 0,
                piles: Vec::new(),
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![1, 2], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = result.choice.as_ref().unwrap();
    assert_eq!(discard.observer, owner);
    assert_eq!(discard.options.len(), 5);
    let selected: Vec<_> = discard
        .options
        .iter()
        .take(2)
        .map(|option| option.token)
        .collect();
    let (state, result) = choose(state, owner, selected, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skill_pile_cards(owner, MIAOXING_1).len(), 3);
    assert_eq!(state.skill_pile_cards(owner, MIAOXING_2).len(), 3);
}

#[test]
fn fushi_exchanges_owner_then_optional_male_hand_with_the_chosen_water() {
    let (state, players) = setup();
    let (owner, male) = (players[0], players[1]);
    let (mut state, cards) = initialize(state, owner, 6);
    let male_cards = vec![inject(&mut state, male, 20), inject(&mut state, male, 21)];
    let skill = skill_id(&state, owner, FUSHI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhenji {
                player: owner,
                target: PlayerId::NONE,
                action: 2,
                stage: 0,
                piles: Vec::new(),
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, owner_target) = choose(tx.commit().state, owner, vec![1], 4);
    assert_eq!(state.hand_cards(owner), cards[2..4]);
    assert_eq!(state.skill_pile_cards(owner, MIAOXING_1), cards[..2]);
    assert_eq!(owner_target.choice.as_ref().unwrap().observer, owner);
    let (state, male_pile) = choose(state, owner, vec![male.raw()], 5);
    assert_eq!(male_pile.choice.as_ref().unwrap().observer, male);
    let (state, done) = choose(state, male, vec![1], 6);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(male), cards[..2]);
    assert_eq!(state.skill_pile_cards(owner, MIAOXING_1), male_cards);
}
