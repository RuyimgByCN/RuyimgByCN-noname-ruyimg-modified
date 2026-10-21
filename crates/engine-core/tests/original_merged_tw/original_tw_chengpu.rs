//! Golden coverage for TW 程普「疠火/醇醪」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWLIHUO: u32 = 2_910_350_044;
const TWCHUNLAO: u32 = 2_257_064_269;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twchengpu.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_001),
        kind,
        suit: Suit::Spade,
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
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );

    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

fn setup(skill_def: u32, phase: Phase) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("tw-chengpu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 389);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_208, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    });
    effects.push(EffectRequest::SetTurnState {
        current: players[0],
        phase: phase.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn converted_sha_costs_hp_only_when_it_causes_dying() {
    for (target_hp, expected_owner_hp) in [(4, 4), (1, 3)] {
        let (mut state, players) = setup(TWLIHUO, Phase::Play);
        let (owner, target) = (players[0], players[1]);
        for player in &players {
            state.players.get_mut(*player).unwrap().hp = 4;
        }
        state.players.get_mut(target).unwrap().hp = target_hp;
        for player in &players {
            let hand = state.hand_cards(*player);
            let mut tx = Transaction::begin(state);
            if !hand.is_empty() {
                run_effects(
                    &mut tx,
                    vec![EffectRequest::DiscardCards {
                        player: *player,
                        cards: hand,
                    }],
                )
                .unwrap();
            }
            state = tx.commit().state;
        }
        let sha = inject(&mut state, owner, CardKind::Sha);
        let (state, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(10 + target_hp as u64),
                kind: CommandKind::PlayCard {
                    card: sha,
                    targets: vec![target],
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        // 疠火在当前实现中无条件将普通杀转为火属性；加目标为唯一 choice，取消之。
        let state = choose(state, owner, Vec::new(), true, 20 + target_hp as u64);
        assert_eq!(state.players.get(owner).unwrap().hp, expected_owner_hp);
    }
}

#[test]
fn chunlao_stores_one_field_card_and_spends_it_to_rescue() {
    let (state, players) = setup(TWCHUNLAO, Phase::Idle);
    let (owner, target) = (players[0], players[1]);
    let skill = skill(&state, owner, TWCHUNLAO);
    let selected = state.hand_cards(target)[0];
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwchengpu {
                player: owner,
                target: owner,
                source: PlayerId::NONE,
                card: CardId::NONE,
                chunlao_skill_def: TWCHUNLAO,
                action: 1,
                stage: 0,
                selected_card: CardId::NONE,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], false, 41);
    let mut state = choose(state, owner, vec![selected.raw()], false, 42);
    assert_eq!(state.skill_pile_cards(target, TWCHUNLAO), vec![selected]);
    state.players.get_mut(target).unwrap().hp = 0;
    state.players.get_mut(target).unwrap().dying = true;
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwchengpu {
            player: owner,
            target,
            source: PlayerId::NONE,
            card: CardId::NONE,
            chunlao_skill_def: TWCHUNLAO,
            action: 3,
            stage: 0,
            selected_card: CardId::NONE,
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().skill_pile_cards(target, TWCHUNLAO).is_empty());
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 1);
}
