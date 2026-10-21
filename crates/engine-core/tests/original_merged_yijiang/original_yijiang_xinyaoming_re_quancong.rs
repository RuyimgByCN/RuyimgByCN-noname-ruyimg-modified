//! Golden coverage for 界全琮「邀名」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_QUANCONG: u32 = 387_181_137;
const XINYAOMING: u32 = 3_897_075_796;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/xinyaoming.skill");
const YIJIANG_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const REFRESH_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("xinyaoming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_730, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINYAOMING),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = RE_QUANCONG;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
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

fn open_trigger(state: GameState, owner: PlayerId, target: PlayerId, key: u64) -> GameState {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, result) = resolve(state, owner, vec![1], key);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveXinyaoming
    );
    state
}

#[test]
fn packages_publish_abi319_skill_and_complete_refresh_character() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINYAOMING));
    assert!(yijiang.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINYAOMING
            && trigger.event == "damage"
            && trigger.filter == "owner_is_source_or_target_xinyaoming_available"
            && trigger.optional
    }));
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH_PACKAGE).unwrap();
    let character = refresh
        .character_index
        .iter()
        .find(|character| character.id == RE_QUANCONG)
        .unwrap();
    assert_eq!(character.name_key, "re_quancong");
    assert_eq!(character.skills, [XINYAOMING]);
}

#[test]
fn three_actions_are_each_once_per_turn_and_reset_next_turn() {
    let mut setup = setup();
    for def in [99_401, 99_402, 99_403] {
        inject_hand(&mut setup.state, setup.target, def);
    }

    let state = open_trigger(setup.state, setup.owner, setup.target, 1);
    let (state, target_choice) = resolve(state, setup.owner, vec![setup.target.raw()], 2);
    assert_eq!(
        target_choice.choice.as_ref().unwrap().kind,
        ChoiceKind::Custom
    );
    assert_eq!(
        target_choice
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    let before_draw = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.owner, vec![1], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), before_draw + 1);

    let state = open_trigger(state, setup.owner, setup.target, 4);
    let (state, action_choice) = resolve(state, setup.owner, vec![setup.target.raw()], 5);
    assert_eq!(
        action_choice
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    let (state, discard_choice) = resolve(state, setup.owner, vec![2], 6);
    assert_eq!(
        discard_choice.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let discard_card = discard_choice.choice.as_ref().unwrap().options[0].token;
    let before_discard = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.owner, vec![discard_card], 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), before_discard - 1);

    let state = open_trigger(state, setup.owner, setup.target, 8);
    let (state, discard_draw_choice) = resolve(state, setup.owner, vec![setup.target.raw()], 9);
    assert_eq!(
        discard_draw_choice.choice.as_ref().unwrap().observer,
        setup.target
    );
    let cards: Vec<_> = discard_draw_choice
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .take(2)
        .map(|option| option.token)
        .collect();
    let before_exchange = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.target, cards, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), before_exchange);

    let mut tx = Transaction::begin(state);
    let fourth = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.owner,
            target: setup.target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert!(fourth.is_ok());
    assert!(!tx.state().has_pending_choice());
    tx.state_mut().turn.phase = Phase::End;
    run_effects(&mut tx, vec![EffectRequest::AdvanceTurn]).unwrap();
    let state = tx.commit().state;
    let state = open_trigger(state, setup.owner, setup.target, 11);
    assert!(state.has_pending_choice());
}

#[test]
fn uninvolved_damage_does_not_offer_xinyaoming() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.target,
            target: setup.other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert!(result.is_ok());
    assert!(!tx.state().has_pending_choice());
}
