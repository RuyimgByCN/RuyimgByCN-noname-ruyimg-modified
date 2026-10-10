//! Golden for 3D精选 罗宪【示烈】（ABI 620）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDSHILIE: u32 = 3_904_237_061;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/luoxian.skill");

fn setup(seed: u64) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-luoxian", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(DDDSHILIE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8, serial: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_300 + serial),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_draw(state: &mut GameState, rank: u8, serial: u32) -> CardId {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_300 + serial),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == DDDSHILIE)
        })
        .unwrap()
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
fn package_exposes_exact_abi_and_source_free_contract() {
    let package = content_compiler::compile_source_named("ddd-luoxian", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 620);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.view_as_index.len(), 1);
    assert_eq!(package.view_as_index[0].cost.as_deref(), Some("dddshilie"));
}

#[test]
fn shilie_exact_sum_supplies_shan_keeps_cards_public_and_draws() {
    let (mut state, players) = setup(20_260_811);
    let owner = players[0];
    let attacker = players[1];
    state.turn.current = attacker;
    let sha = inject(&mut state, attacker, CardKind::Sha, 7, 1);
    let threshold =
        (state.players.get(owner).unwrap().hp + state.players.get(attacker).unwrap().hp) as u8;
    let shown = inject(&mut state, owner, CardKind::Other, threshold, 2);
    inject_draw(&mut state, 3, 20);
    let hp = state.players.get(owner).unwrap().hp;
    let (state, offered) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        10,
    );
    let hand: std::collections::BTreeSet<_> = state
        .hand_cards(owner)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let conversion = offered
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .find(|token| !hand.contains(token) && !matches!(*token, 0 | 1))
        .unwrap();
    let (state, sources) = choose(state, owner, vec![conversion], 11);
    assert_eq!(sources.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![shown.raw()], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(state.hand_cards(owner).contains(&shown));
    assert!(state.cards.get(shown).unwrap().face_up);
    assert_eq!(state.hand_count(owner), 2);
}

#[test]
fn shilie_above_threshold_uses_virtual_sha_without_draw_and_is_once_per_turn() {
    let (mut state, players) = setup(20_260_812);
    let owner = players[0];
    let target = players[1];
    let threshold = (state.players.get(owner).unwrap().hp * 2) as u8;
    let shown = inject(&mut state, owner, CardKind::Other, threshold + 1, 3);
    let instance = skill(&state, owner);
    let hp = state.players.get(target).unwrap().hp;
    let (state, choice) = submit(
        state,
        owner,
        CommandKind::UseSkillCard {
            skill: instance,
            card_kind: CardKind::Sha.tag(),
            targets: vec![target],
            equip_slot: 0,
        },
        20,
    );
    assert_eq!(choice.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![shown.raw()], 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(state.hand_cards(owner), vec![shown]);
    assert!(state.cards.get(shown).unwrap().face_up);
    assert_eq!(state.skills.get(instance).unwrap().use_count, 1);

    let (.., repeated) = submit(
        state,
        owner,
        CommandKind::UseSkillCard {
            skill: instance,
            card_kind: CardKind::Sha.tag(),
            targets: vec![target],
            equip_slot: 0,
        },
        22,
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}
