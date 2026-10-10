//! Golden for 3D精选 刘巴【丰政/玉律】（ABI 632）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENGZHENG: u32 = 986_583_704;
const FENGZHENG_GLOBAL: u32 = 1_253_882_292;
const YULV: u32 = 694_263_366;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/liuba.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liuba", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 632);
    let global = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == FENGZHENG_GLOBAL)
        .unwrap();
    assert_eq!(global.uses_per_play_phase, Some(1));
    assert_eq!(
        global
            .selection
            .as_ref()
            .unwrap()
            .sum_effective_number_exact,
        Some(13)
    );

    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
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
    effects.extend([FENGZHENG, YULV].map(|skill_def| EffectRequest::AddSkill {
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

fn inject(state: &mut GameState, owner: PlayerId, serial: u32, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_632_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Heart,
        rank,
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
    cancelled: bool,
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
                cancelled,
            },
        },
    )
}

#[test]
fn fengzheng_installs_global_and_uses_exact_thirteen_as_complete_wuzhong() {
    let (mut state, players) = setup();
    for player in &players {
        assert!(state
            .players
            .get(*player)
            .unwrap()
            .skills
            .iter()
            .any(|skill| { state.skills.get(*skill).unwrap().def.raw() == FENGZHENG_GLOBAL }));
    }

    let user = players[1];
    let six = inject(&mut state, user, 1, 6);
    let seven = inject(&mut state, user, 2, 7);
    state.turn.current = user;
    state.turn.phase = Phase::Play;
    let global = skill_id(&state, user, FENGZHENG_GLOBAL);
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuba {
            player: user,
            actor: user,
            action: 0,
            cards: vec![six, seven],
            targets: Vec::new(),
            skill: global,
        }],
    );
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    assert_eq!(
        state.skills.get(global).unwrap().round_fire_round,
        state.turn.round
    );
    assert!(!state.hand_cards(user).contains(&six));
    assert!(!state.hand_cards(user).contains(&seven));
}

#[test]
fn fengzheng_counts_distinct_previous_round_users_and_loses_on_three_to_one() {
    let (mut state, players) = setup();
    let owner = players[0];
    let fengzheng = skill_id(&state, owner, FENGZHENG);
    for player in &players {
        let global = skill_id(&state, *player, FENGZHENG_GLOBAL);
        state.skills.get_mut(global).unwrap().round_fire_round = 1;
    }
    state.turn.round = 2;
    state.turn.current = owner;
    state.turn.phase = Phase::Prepare;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuba {
                player: owner,
                actor: owner,
                action: 1,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: fengzheng,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![1], false, 10);
    let subset: Vec<_> = result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(subset.len(), 3);
    let (state, _) = choose(state, owner, subset, false, 11);
    let (state, result) = choose(state, owner, vec![players[1].raw()], false, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(players[1]), 3);
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .skills
        .contains(&fengzheng));
}

#[test]
fn yulv_matches_physical_rank_and_exchanges_after_second_trigger_this_turn() {
    let (state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let yulv = skill_id(&state, owner, YULV);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuba {
                player: owner,
                actor: owner,
                action: 3,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: yulv,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let initial = committed.output.choice.as_ref().unwrap().options[0].token;
    let (mut state, _) = choose(committed.state, owner, vec![initial], false, 20);
    let old_yulv = state.skill_pile_cards(owner, YULV)[0];
    let rank = state.cards.get(old_yulv).unwrap().rank;
    state.turn.current = actor;
    state.turn.phase = Phase::Play;

    for key in [21u64, 24] {
        let card = inject(&mut state, actor, key as u32, rank);
        let mut tx = Transaction::begin(state);
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::DiscardCards {
                    player: actor,
                    cards: vec![card],
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        let (next, result) = choose(tx.commit().state, owner, vec![0], false, key + 1);
        state = next;
        if key == 24 {
            assert!(result.choice.is_some(), "第二次玉律后应继续要求交换");
            let replacement = result.choice.as_ref().unwrap().options[0].token;
            let (next, result) = choose(state, owner, vec![replacement], false, key + 2);
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            state = next;
        }
    }
    assert_ne!(state.skill_pile_cards(owner, YULV)[0], old_yulv);
    assert!(state.hand_cards(owner).contains(&old_yulv));
}
