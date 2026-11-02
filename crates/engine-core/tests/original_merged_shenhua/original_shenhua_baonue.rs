//! Golden coverage for 神话「暴虐」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BAONUE: u32 = 3_659_854_073;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/baonue.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        voice_assets: Default::default(),
        id,
        name_key: format!("baonue_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

struct Setup {
    tx: Transaction,
    lord: PlayerId,
    qun_source: PlayerId,
    wei_source: PlayerId,
    target: PlayerId,
}

fn setup() -> Setup {
    let mut package = content_compiler::compile_source_named("baonue-test", SOURCE).unwrap();
    package.character_index = vec![
        character(9_288_010, "qun"),
        character(9_288_011, "qun"),
        character(9_288_012, "wei"),
        character(9_288_013, "shu"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 20_260_722_88, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let mut others = seats.iter().copied().filter(|player| *player != lord);
    let qun_source = others.next().unwrap();
    let wei_source = others.next().unwrap();
    let target = others.next().unwrap();
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_288_010;
    tx.state_mut()
        .players
        .get_mut(qun_source)
        .unwrap()
        .character_def = 9_288_011;
    tx.state_mut()
        .players
        .get_mut(wei_source)
        .unwrap()
        .character_def = 9_288_012;
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = 9_288_013;

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: lord,
        skill_def: SkillDefId::new(BAONUE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(lord).unwrap().hp = 3;
    Setup {
        tx,
        lord,
        qun_source,
        wei_source,
        target,
    }
}

fn inject_judgment(tx: &mut Transaction, suit: Suit) -> CardId {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_288_100 + suit as u32),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn deal(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
}

fn resolve_choice(
    tx: &Transaction,
    actor: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_exposes_damage_source_choice_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == BAONUE)
        .unwrap();
    assert!(skill.lord_only);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == BAONUE)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_alive_other_damage_source_group_qun");
    assert!(trigger.optional);
    assert_eq!(
        trigger.choice_actor,
        content_schema::TriggerChoiceActor::EventSource
    );
}

#[test]
fn only_another_living_qun_damage_source_gets_the_choice() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.state().clone());
    assert_eq!(
        deal(&mut tx, setup.qun_source, setup.target),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.pending_observer,
        setup.qun_source.raw()
    );

    let mut tx = Transaction::begin(setup.tx.state().clone());
    deal(&mut tx, setup.wei_source, setup.target).unwrap();
    assert!(!tx.state().has_pending_choice());

    let mut tx = Transaction::begin(setup.tx.state().clone());
    deal(&mut tx, setup.lord, setup.target).unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn accepted_spade_judges_for_source_and_recovers_lord() {
    let mut setup = setup();
    let judgment = inject_judgment(&mut setup.tx, Suit::Spade);
    assert_eq!(
        deal(&mut setup.tx, setup.qun_source, setup.target),
        Err(EffectError::ChoiceRequested)
    );
    let (state, pipeline) = resolve_choice(&setup.tx, setup.qun_source, true, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.lord).unwrap().hp, 4);
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        DomainEvent::JudgmentResolved {
            player,
            result_card,
            cause_card,
            ..
        } if *player == setup.qun_source && *result_card == judgment && cause_card.is_none()
    )));
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        DomainEvent::HpRecovered { source, player, amount: 1 }
            if *source == setup.lord && *player == setup.lord
    )));
}

#[test]
fn non_spade_does_not_recover_and_declining_does_not_judge() {
    let mut non_spade = setup();
    let club = inject_judgment(&mut non_spade.tx, Suit::Club);
    assert_eq!(
        deal(&mut non_spade.tx, non_spade.qun_source, non_spade.target,),
        Err(EffectError::ChoiceRequested)
    );
    let (state, pipeline) = resolve_choice(&non_spade.tx, non_spade.qun_source, true, 2);
    assert_eq!(state.players.get(non_spade.lord).unwrap().hp, 3);
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        DomainEvent::JudgmentResolved { player, result_card, .. }
            if *player == non_spade.qun_source && *result_card == club
    )));

    let mut declined = setup();
    let spade = inject_judgment(&mut declined.tx, Suit::Spade);
    assert_eq!(
        deal(&mut declined.tx, declined.qun_source, declined.target),
        Err(EffectError::ChoiceRequested)
    );
    let (state, pipeline) = resolve_choice(&declined.tx, declined.qun_source, false, 3);
    assert_eq!(state.players.get(declined.lord).unwrap().hp, 3);
    assert!(!pipeline.events.iter().any(|event| matches!(
        event,
        DomainEvent::JudgmentResolved { result_card, .. } if *result_card == spade
    )));
}
