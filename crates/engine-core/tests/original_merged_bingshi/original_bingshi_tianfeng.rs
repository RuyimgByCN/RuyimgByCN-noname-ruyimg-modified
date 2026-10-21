//! Golden coverage for BINGSHI 势田丰「刚鲠」「死谏」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBGANGGENG: u32 = 2_184_967_024;
const MBSIJIAN: u32 = 2_168_498_663;
const MBSIJIAN_DISCARD: u32 = 2_654_683_932;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

#[test]
fn package_publishes_complete_tianfeng_at_abi509() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let character = package
        .character_index
        .iter()
        .find(|character| character.name_key == "mb_tianfeng")
        .unwrap();
    assert_eq!(character.skills, [MBGANGGENG, MBSIJIAN]);
}

fn setup(skill_def: u32) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: u64::from(skill_def),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, others[0], others[1])
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

fn inject(tx: &mut Transaction, owner: PlayerId, serial: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(508_000 + serial),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn ganggeng_gives_selected_cards_and_draws_when_recipient_has_max_hand() {
    let (mut tx, owner, target, _) = setup(MBGANGGENG);
    let skill = skill(tx.state(), owner, MBGANGGENG);
    let cards = vec![inject(&mut tx, owner, 1), inject(&mut tx, owner, 2)];
    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbganggeng {
            player: owner,
            target,
            stage: 0,
            cards: cards.clone(),
            skill,
        }],
    )
    .unwrap();
    assert!(cards
        .iter()
        .all(|card| tx.state().hand_cards(target).contains(card)));
    assert_eq!(
        &tx.state().skills.get(skill).unwrap().state_blob[..4],
        b"GGG1"
    );
    let hand_after_gift = tx.state().hand_count(owner);
    assert_eq!(hand_after_gift, before - 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbganggeng {
            player: owner,
            target: owner,
            stage: 1,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_after_gift + 1);
    assert!(tx.state().skills.get(skill).unwrap().state_blob.is_empty());
}

#[test]
fn ganggeng_active_command_forwards_selected_hand_cards() {
    let (mut tx, owner, target, _) = setup(MBGANGGENG);
    let skill = skill(tx.state(), owner, MBGANGGENG);
    let cards = vec![inject(&mut tx, owner, 40), inject(&mut tx, owner, 41)];
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(40),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: cards.clone(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(cards
        .iter()
        .all(|card| state.hand_cards(target).contains(card)));
}

#[test]
fn ganggeng_discards_from_recipient_when_another_player_has_more_hand() {
    let (mut tx, owner, target, leader) = setup(MBGANGGENG);
    let skill = skill(tx.state(), owner, MBGANGGENG);
    let cards = vec![inject(&mut tx, owner, 3), inject(&mut tx, owner, 4)];
    for serial in 10..20 {
        inject(&mut tx, leader, serial);
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbganggeng {
            player: owner,
            target,
            stage: 0,
            cards,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbganggeng {
                player: owner,
                target: owner,
                stage: 1,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let selected = tx.state().hand_cards(target)[0];
    let state = resolve(tx.commit().state, vec![selected.raw()], false, 1);
    assert!(!state.hand_cards(target).contains(&selected));
}

#[test]
fn sijian_both_is_cumulative_and_grants_one_shot_discard() {
    let (mut tx, owner, target, _) = setup(MBSIJIAN);
    let skill = skill(tx.state(), owner, MBSIJIAN);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbsijian {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                choice: 0,
                helper_def: SkillDefId::new(MBSIJIAN_DISCARD),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![3], false, 2);
    let state = resolve(state, vec![target.raw()], false, 3);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(owner), hand + 2);
    assert!(state
        .skills
        .sorted_live()
        .into_iter()
        .any(|instance| { instance.owner == target && instance.def.raw() == MBSIJIAN_DISCARD }));

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbsijian {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                choice: 0,
                helper_def: SkillDefId::new(MBSIJIAN_DISCARD),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![3], false, 4);
    let state = resolve(state, vec![target.raw()], false, 5);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);

    let mut tx = Transaction::begin(state);
    let used = inject(&mut tx, target, 30);
    let discardable = inject(&mut tx, target, 31);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: target,
                card: used,
                targets: Vec::new(),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![discardable.raw()], false, 6);
    assert!(!state
        .skills
        .sorted_live()
        .into_iter()
        .any(|instance| { instance.owner == target && instance.def.raw() == MBSIJIAN_DISCARD }));
    assert!(!state.hand_cards(target).contains(&discardable));
}
