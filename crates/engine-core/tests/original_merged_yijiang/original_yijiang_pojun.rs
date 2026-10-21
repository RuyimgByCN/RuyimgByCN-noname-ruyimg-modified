//! Golden for 一将成名徐盛「破军」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POJUN: u32 = 2_171_911_670;
const OLD_XUSHENG: u32 = 3_544_872_927;
const YIJIANG_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

fn isolated(seed: u64) -> (Transaction, PlayerId, PlayerId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    let mut engine = Engine::new();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine.load_package(&package).unwrap();
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != source)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: target,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(POJUN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(82_171),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx, source, target, card)
}

fn accept(state: &GameState, source: PlayerId, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_packages_contain_pojun_and_old_xusheng_cross_package_closure() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == POJUN && !skill.locked));
    let trigger = yijiang
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == POJUN)
        .unwrap();
    assert_eq!(
        (trigger.event.as_str(), trigger.filter.as_str()),
        ("damage", "owner_source_sha")
    );
    assert!(trigger.optional);

    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    let xusheng = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_XUSHENG)
        .unwrap();
    assert_eq!(xusheng.name_key, "old_xusheng");
    assert_eq!(
        (xusheng.sex.as_str(), xusheng.group.as_str()),
        ("male", "wu")
    );
    assert_eq!(xusheng.max_hp, 4);
    assert_eq!(xusheng.skills, [POJUN]);
    assert!(old
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
}

#[test]
fn sha_damage_draws_up_to_five_from_current_hp_then_turns_target_over() {
    let (mut tx, source, target, card) = isolated(20_260_716_03);
    let player = tx.state_mut().players.get_mut(target).unwrap();
    player.max_hp = 6;
    player.hp = 6;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source,
                target,
                card,
                amount: 1,
                nature: rule_api::damage_nature::NORMAL,
                card_flags: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().players.get(target).unwrap().hp, 5);
    let state = accept(tx.state(), source, 1);
    assert_eq!(state.hand_count(target), 5);
    assert!(state.players.get(target).unwrap().turned_over);
}

#[test]
fn current_hp_controls_draw_count_and_turn_over_toggles() {
    let (mut tx, source, target, card) = isolated(20_260_716_04);
    let player = tx.state_mut().players.get_mut(target).unwrap();
    player.max_hp = 4;
    player.hp = 3;
    player.turned_over = true;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source,
                target,
                card,
                amount: 1,
                nature: rule_api::damage_nature::NORMAL,
                card_flags: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx.state(), source, 2);
    assert_eq!(state.hand_count(target), 2);
    assert!(!state.players.get(target).unwrap().turned_over);
}

#[test]
fn non_sha_damage_does_not_trigger_pojun() {
    let (mut tx, source, target, _) = isolated(20_260_716_05);
    let hand = tx.state().hand_count(target);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(target), hand);
    assert!(!tx.state().players.get(target).unwrap().turned_over);
    assert!(!tx.state().has_pending_choice());
}
