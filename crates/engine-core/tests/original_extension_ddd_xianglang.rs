//! Golden for 3D精选 向朗【恰化/恂恂/腹笥/拓籍】（ABI 626）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QIAHUA: u32 = 891_363_178;
const XUNXUN: u32 = 1_673_829_014;
const FUSI: u32 = 1_632_230_116;
const FUSI_GLOBAL: u32 = 2_198_174_483;
const TUOJI: u32 = 1_700_500_735;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/xianglang.skill");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-xianglang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 626);
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
    effects.extend([QIAHUA, FUSI, TUOJI].map(|def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_626_000 + serial),
        kind,
        suit: Suit::Heart,
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

fn choose(
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

#[test]
fn qiahua_reveals_exact_hp_cards_and_grants_xunxun_for_turn() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    state.players.get_mut(actor).unwrap().hp = 2;
    state.turn.current = actor;
    let cards = [
        inject(&mut state, owner, CardKind::Shan, 1),
        inject(&mut state, owner, CardKind::Tao, 2),
    ];
    let qiahua = skill(&state, owner, QIAHUA);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXianglang {
                player: owner,
                actor,
                action: 0,
                skill: qiahua,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(
        tx.commit().state,
        owner,
        cards.iter().map(|card| card.raw()).collect(),
        1,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().face_up));
    assert_eq!(
        state
            .skills
            .get(skill(&state, actor, XUNXUN))
            .unwrap()
            .temporary_until_turn_end_of,
        actor
    );
}

#[test]
fn fusi_confirmed_foreign_shown_sha_is_used_and_tuoji_draws_three() {
    let (mut state, players) = setup();
    let (owner, borrower, target) = (players[0], players[1], players[2]);
    let shown = inject(&mut state, owner, CardKind::Sha, 10);
    state.cards.get_mut(shown).unwrap().face_up = true;
    let fusi = skill(&state, owner, FUSI);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXianglang {
            player: owner,
            actor: PlayerId::NONE,
            action: 1,
            skill: fusi,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    state.turn.current = borrower;
    state.turn.phase = Phase::Play;
    let helper = skill(&state, borrower, FUSI_GLOBAL);
    let target_hp = state.players.get(target).unwrap().hp;
    let (state, source_choice) = submit(
        state,
        borrower,
        CommandKind::UseSkillCard {
            skill: helper,
            card_kind: CardKind::Sha.tag(),
            targets: vec![target],
            equip_slot: 0,
        },
        10,
    );
    assert_eq!(source_choice.choice.unwrap().observer, borrower);
    let (state, approval) = choose(state, borrower, vec![shown.raw()], 11);
    assert_eq!(approval.choice.unwrap().observer, owner);
    let (state, tuoji) = choose(state, owner, vec![0], 12);
    assert_eq!(
        tuoji.choice.as_ref().map(|choice| choice.observer),
        Some(owner),
        "result={:?} continuation={:?}",
        tuoji.result,
        state.resolution.continuation_kind
    );
    let before = state.hand_count(owner);
    let (state, done) = choose(state, owner, vec![1], 13);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(owner), before + 3);
    assert!(!state.hand_cards(owner).contains(&shown));
}

#[test]
fn fusi_face_up_cards_ignore_limit_and_all_shown_owner_turn_locks_other_hands() {
    let (mut state, players) = setup();
    let (owner, other) = (players[0], players[1]);
    let shown = inject(&mut state, owner, CardKind::Tao, 20);
    state.cards.get_mut(shown).unwrap().face_up = true;
    state.turn.current = owner;
    assert!(engine_core::query::card_ignored_for_hand_limit(
        &state, owner, shown
    ));
    assert!(engine_core::query::hand_card_use_response_cancelled_for(
        &state, other
    ));
}
