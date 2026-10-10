//! Golden for 3D精选 蹇硕【分野/逝潮】（ABI 633）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENYE: u32 = 1_190_748_172;
const SHICHAO: u32 = 219_219_058;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/jianshuo.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-jianshuo", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 633);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == FENYE)
        .unwrap();
    assert_eq!(active.uses_per_play_phase, Some(1));
    assert!(active.targets.as_ref().unwrap().can_compare);

    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([FENYE, SHICHAO].map(|skill_def| EffectRequest::AddSkill {
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
        def: CardDefId::new(99_633_000 + serial),
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

fn choice_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn fenye_uses_joined_cards_average_and_orders_winners_by_seat() {
    let (mut state, players) = setup();
    let owner = players[0];
    let skill = skill_id(&state, owner, FENYE);
    let cards = [
        inject(&mut state, players[0], 1, 10),
        inject(&mut state, players[1], 2, 4),
        inject(&mut state, players[2], 3, 12),
        inject(&mut state, players[3], 4, 2),
    ];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(100),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![players[1]],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(result.choice.is_some());
    let (state, _) = choose(state, players[0], vec![cards[0].raw()], false, 1);
    let (state, _) = choose(state, players[1], vec![cards[1].raw()], false, 2);
    let (state, _) = choose(state, players[2], vec![1], false, 3);
    let (state, _) = choose(state, players[2], vec![cards[2].raw()], false, 4);
    let (state, _) = choose(state, players[3], vec![2], false, 5);
    let (state, result) = choose(state, players[3], vec![cards[3].raw()], false, 6);
    assert_eq!(
        choice_tokens(&result),
        vec![players[1].raw(), players[3].raw()]
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));

    let loser_hp = state.players.get(players[1]).unwrap().hp;
    let (state, result) = choose(state, players[0], vec![players[1].raw()], false, 7);
    assert_eq!(state.players.get(players[1]).unwrap().hp, loser_hp - 1);
    assert_eq!(choice_tokens(&result), vec![players[3].raw()]);
    let (state, result) = choose(state, players[2], Vec::new(), true, 8);
    assert!(result.choice.is_none());
    let processing = state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(processing).unwrap().cards.is_empty());
}

#[test]
fn shichao_adjusts_to_rank_and_source_can_prevent_then_advance_rank() {
    let (mut state, players) = setup();
    let owner = players[0];
    let skill = skill_id(&state, owner, SHICHAO);
    inject(&mut state, players[1], 10, 8);
    inject(&mut state, players[1], 11, 9);
    inject(&mut state, players[1], 12, 10);
    inject(&mut state, players[2], 13, 6);
    inject(&mut state, players[2], 14, 7);
    inject(&mut state, players[3], 15, 5);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddJianshuo {
                player: owner,
                target: owner,
                action: 1,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![players[1].raw()], false, 20);
    assert!(result.choice.is_none());
    assert_eq!(state.hand_count(owner), 3);

    let mut state = state;
    state.resolution.current_damage_id = 633;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddJianshuo {
                player: owner,
                target: players[1],
                action: 2,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, players[1], vec![1], false, 21);
    assert!(state.resolution.prevented_damage_ids.contains(&633));
    assert_eq!(state.skills.get(skill).unwrap().mark, 1);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddJianshuo {
                player: owner,
                target: owner,
                action: 1,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![players[2].raw()], false, 22);
    let discard = choice_tokens(&result)[0];
    let (state, _) = choose(state, owner, vec![discard], false, 23);
    assert_eq!(state.hand_count(owner), 2);
}
