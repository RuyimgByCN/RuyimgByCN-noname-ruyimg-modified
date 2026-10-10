//! Golden for MOBILE「据峻 / 壁险」的阶段历史与无实体普通用牌流程。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUJUN: u32 = 3_855_454_532;
const BIXIAN: u32 = 4_274_044_152;
const O_CHENLAN: u32 = 2_348_532_465;
const O_MEICHENG: u32 = 2_493_526_902;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64, skill: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
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
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|instance| EffectRequest::RemoveSkill { skill: instance.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill),
        },
        EffectRequest::SetTurnState {
            current: other,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn target_owner(tx: &mut Transaction, user: PlayerId, owner: PlayerId, rank: u8) {
    let hand = tx.state().zones.find(ZoneKind::Hand, user).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner: user,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    run_effects(
        tx,
        vec![EffectRequest::UseCard {
            player: user,
            card,
            targets: vec![owner],
        }],
    )
    .unwrap();
}

fn bit(player: PlayerId) -> i32 {
    1_i32 << (player.raw() - 1)
}

#[test]
fn package_declares_current_abi_skills_and_closed_characters() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    for (skill, event_filter) in [
        (
            JUJUN,
            "owner_alive_phase_targeted_hp_changed_can_use_wanjian",
        ),
        (
            BIXIAN,
            "owner_alive_phase_targeted_hp_unchanged_can_use_juedou",
        ),
    ] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill && entry.locked));
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == skill
                && trigger.event == "phase_any_end"
                && trigger.filter == event_filter
                && !trigger.optional
        }));
    }
    for (character, skill) in [(O_CHENLAN, JUJUN), (O_MEICHENG, BIXIAN)] {
        let character = package
            .character_index
            .iter()
            .find(|entry| entry.id == character)
            .unwrap();
        assert_eq!(character.group, "qun");
        assert_eq!(character.max_hp, 4);
        assert_eq!(character.skills, vec![skill]);
    }
}

#[test]
fn targeted_without_hp_change_forces_bixian_duel_target_choice() {
    let (mut tx, owner, other) = setup(20_260_723, BIXIAN);
    target_owner(&mut tx, other, owner, 3);
    assert_eq!(
        tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS) & bit(owner),
        bit(owner)
    );
    assert_eq!(
        tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) & bit(owner),
        0
    );

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Discard.to_u8(),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseUseVirtualCard
    );
    assert_eq!(tx.state().resolution.continuation_payload[2], 23);
    assert_eq!(tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS), 0);
    assert_eq!(tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS), 0);
}

#[test]
fn max_hp_change_without_actual_hp_change_still_forces_bixian() {
    let (mut tx, owner, other) = setup(20_260_726, BIXIAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    target_owner(&mut tx, other, owner, 5);
    run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: owner,
            delta: -1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(
        tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) & bit(owner),
        0
    );

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Discard.to_u8(),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseUseVirtualCard
    );
}

#[test]
fn targeted_with_hp_change_forces_jujun_wanjian() {
    let (mut tx, owner, other) = setup(20_260_724, JUJUN);
    target_owner(&mut tx, other, owner, 4);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) & bit(owner),
        bit(owner)
    );

    match run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: Phase::Discard.to_u8(),
        }],
    ) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("unexpected Wanjian resolution error: {error}"),
    }
    assert!(tx.state().cards.sorted_live().into_iter().any(|card| {
        card.def.is_none() && card.kind == CardKind::TrickWanjian && card.suit == Suit::None
    }));
    assert_eq!(tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS), 0);
    assert_eq!(tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS), 0);
}

#[test]
fn hp_change_without_becoming_a_card_target_fires_neither_skill() {
    let (mut tx, owner, other) = setup(20_260_725, JUJUN);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    assert!(!tx
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .any(|card| card.def.is_none() && card.kind == CardKind::TrickWanjian));
}
