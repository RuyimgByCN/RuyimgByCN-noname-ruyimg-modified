//! Golden for MOBILE 孙璋组件「自谋」的出牌阶段次数语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SCSZIMOU: u32 = 2_246_121_175;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

fn setup() -> (Transaction, PlayerId, engine_api::CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(SCSZIMOU as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: SCSZIMOU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let card = engine.state().hand_cards(owner)[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SCSZIMOU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, card)
}

fn use_card(tx: &mut Transaction, owner: PlayerId, card: engine_api::CardId) -> Vec<EffectRequest> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
}

fn gained_kinds(effects: &[EffectRequest]) -> Vec<u8> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            EffectRequest::GainCardByKind { kind, .. } => Some(*kind),
            _ => None,
        })
        .collect()
}

#[test]
fn package_declares_skill_without_publishing_the_hidden_component() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == SCSZIMOU && skill.name == "scszimou" && skill.locked
    }));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == SCSZIMOU)
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        vec![
            "owner_source_play_card_count_eq_2",
            "owner_source_play_card_count_eq_4",
            "owner_source_play_card_count_eq_6",
        ]
    );
    // 十常侍 roster: scs_sunzhang publishes with 子谋.
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "scs_sunzhang"
            && character.skills.contains(&SCSZIMOU)));
}

#[test]
fn second_fourth_and_sixth_uses_fire_and_each_play_phase_resets() {
    let (mut tx, owner, card) = setup();

    assert!(use_card(&mut tx, owner, card).is_empty());
    let responded = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card,
            response_kind: CardKind::Shan.tag(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(responded.is_empty());
    assert_eq!(
        gained_kinds(&use_card(&mut tx, owner, card)),
        vec![CardKind::Jiu.tag()]
    );
    assert!(use_card(&mut tx, owner, card).is_empty());
    assert_eq!(
        gained_kinds(&use_card(&mut tx, owner, card)),
        vec![CardKind::Sha.tag()]
    );
    assert!(use_card(&mut tx, owner, card).is_empty());
    assert_eq!(
        gained_kinds(&use_card(&mut tx, owner, card)),
        vec![CardKind::TrickJuedou.tag()]
    );

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(use_card(&mut tx, owner, card).is_empty());
    assert_eq!(
        gained_kinds(&use_card(&mut tx, owner, card)),
        vec![CardKind::Jiu.tag()]
    );
}
