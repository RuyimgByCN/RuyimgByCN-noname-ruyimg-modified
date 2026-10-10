//! Golden coverage for SHIJI 杜预「灭吴」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPMIEWU: u32 = 3_162_101_299;
const SPWUKU: u32 = 2_480_520_461;
const SOURCE: &str = include_str!("../../../content/original/ported/shiji/spsanchen.skill");

fn command(actor: PlayerId, key: u64, kind: CommandKind) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind,
    }
}

#[test]
fn spmiewu_consumes_wuku_uses_equipment_and_draws_afterward() {
    let package = content_compiler::compile_source_named("shiji-spmiewu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 502);
    assert_eq!(package.view_as_index.len(), 2);
    assert!(package
        .view_as_index
        .iter()
        .any(|rule| rule.as_kind == "trick"));

    let mut engine = Engine::new();
    engine.submit(command(
        PlayerId::NONE,
        1,
        CommandKind::StartGame {
            player_count: 2,
            rng_seed: 2_026_080_202,
            ruleset: "identity-standard".into(),
        },
    ));
    engine.load_package(&package).unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SPMIEWU),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(SPWUKU),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let material = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_502),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 9,
        weapon_range: 2,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(material);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SPMIEWU)
        .unwrap();
    let hand_before = tx.state().hand_count(owner);
    let state = tx.commit().state;

    let (state, result) = apply_command(
        state,
        command(
            owner,
            2,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        command(
            owner,
            3,
            CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![material.raw()],
                cancelled: false,
            },
        ),
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == SPWUKU && *count > 0 }));
    assert_eq!(
        state
            .zones
            .get(state.cards.get(material).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
}
