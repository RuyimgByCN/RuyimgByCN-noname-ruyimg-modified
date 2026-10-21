//! Golden for 3D精选 刘晔【察势/齐策】（ABI 628）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CHASHI: u32 = 2_811_969_013;
const QICE: u32 = 4_207_915_436;
const SOURCE: &str = include_str!("../../../../content/original/ported/ddd/liuye.skill");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liuye", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 628);
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
    effects.extend([CHASHI, QICE].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    let mut state = tx.commit().state;
    state.turn.current = players[1];
    state.turn.phase = Phase::Play;
    (state, players)
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

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_628_000 + serial),
        kind,
        suit,
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

fn arm_chashi(
    mut state: GameState,
    owner: PlayerId,
    actor: PlayerId,
    discarded: CardId,
) -> GameState {
    let skill = skill_id(&state, owner, CHASHI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuye {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![discarded.raw()], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn chashi_waits_for_first_matching_card_then_draws_for_both() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    let cost = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Heart,
        1,
    );
    let mismatch = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        2,
    );
    let match_by_category = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Club,
        3,
    );
    let state = arm_chashi(state, owner, actor, cost);
    let skill = skill_id(&state, owner, CHASHI);
    let owner_before = state.hand_count(owner);
    let actor_before = state.hand_count(actor);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuye {
            player: owner,
            actor,
            card: mismatch,
            action: 1,
            skill,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .skills
        .get(skill)
        .unwrap()
        .state_blob
        .starts_with(b"DCS1"));
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuye {
            player: owner,
            actor,
            card: match_by_category,
            action: 1,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert_eq!(tx.state().hand_count(actor), actor_before + 1);
    assert!(tx.state().skills.get(skill).unwrap().state_blob.is_empty());
}

#[test]
fn chashi_pending_comparison_expires_with_the_play_phase() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    let cost = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Diamond,
        10,
    );
    let matching = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Diamond,
        11,
    );
    let state = arm_chashi(state, owner, actor, cost);
    let skill = skill_id(&state, owner, CHASHI);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 4,
                skill,
            },
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor,
                card: matching,
                action: 1,
                skill,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}

#[test]
fn qice_stays_locked_across_attempts_until_owner_uses_a_trick() {
    let (mut state, players) = setup();
    let owner = players[0];
    state.turn.current = owner;
    state.turn.phase = Phase::Prepare;
    let basic = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        20,
    );
    let trick = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickGuohe,
        Suit::Club,
        21,
    );
    let skill = skill_id(&state, owner, QICE);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuye {
            player: owner,
            actor: owner,
            card: CardId::NONE,
            action: 2,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                action: 2,
                skill,
            },
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: basic,
                action: 3,
                skill,
            },
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                action: 2,
                skill,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: trick,
                action: 3,
                skill,
            },
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                action: 2,
                skill,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 4);
}
