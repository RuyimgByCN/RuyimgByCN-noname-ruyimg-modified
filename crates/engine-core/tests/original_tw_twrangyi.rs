//! Golden coverage for TW 马良「让驿」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWRANGYI: u32 = 2_884_981_024;
const TW_MALIANG: u32 = 2_410_358_039;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twrangyi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_359_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(
    seed: u64,
) -> (
    GameState,
    PlayerId,
    PlayerId,
    engine_api::SkillId,
    [CardId; 2],
) {
    let package = content_compiler::compile_source_named("twrangyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 359);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
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
        skill_def: SkillDefId::new(TWRANGYI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWRANGYI)
        .unwrap()
        .id;
    let cards = [
        inject(&mut tx, owner, CardKind::Weapon, 3),
        inject(&mut tx, owner, CardKind::Tao, 4),
    ];
    (tx.commit().state, owner, target, skill, cards)
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "key {key}: {:?}",
        result.result,
    );
    state
}

#[test]
fn chosen_card_is_used_and_the_rest_returns_or_decline_deals_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 359);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_MALIANG)
            .unwrap()
            .skills,
        [TWRANGYI, 3_304_922_519]
    );

    let (mut state, owner, target, skill, cards) = setup(20_260_801);
    state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(state.hand_count(owner), 0, "cards were not gifted");
    assert_eq!(state.hand_count(target), 2, "target did not receive cards");
    assert_ne!(state.resolution.pending_choice_id, 0);
    assert_eq!(state.resolution.pending_observer, target.raw());
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![cards[0].raw()],
            cancelled: false,
        },
        2,
    );
    assert!(state.hand_cards(owner).contains(&cards[1]));
    assert!(!state.hand_cards(target).contains(&cards[0]));
    let equip = state.zones.find(ZoneKind::Equip, target).unwrap();
    assert!(state.zones.get(equip).unwrap().cards.contains(&cards[0]));

    let (mut state, owner, target, skill, cards) = setup(20_260_802);
    let hp = state.players.get(target).unwrap().hp;
    state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        3,
    );
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: Vec::new(),
            cancelled: true,
        },
        4,
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(cards
        .iter()
        .all(|card| state.hand_cards(target).contains(card)));
}
