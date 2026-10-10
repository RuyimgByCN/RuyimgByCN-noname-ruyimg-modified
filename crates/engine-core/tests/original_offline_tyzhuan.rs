//! Golden coverage for 桃陈式「驻岸」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TYZHUAN: u32 = 2_515_374_814;
const TY_CHENSHI: u32 = 3_071_008_696;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/tyzhuan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TYZHUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == TYZHUAN)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 3,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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

#[test]
fn package_publishes_abi158_contract_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 838);
    assert_eq!(package.character_index.len(), 382);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 128);
    assert_eq!(package.modules.len(), 724);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TYZHUAN)
        .unwrap();
    assert_eq!(trigger.filter, "owner_alive_equip_used");
    assert!(trigger.optional);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == TYZHUAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.has_equip);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (
            selection.zone.as_str(),
            selection.min,
            selection.max,
            selection.filter.as_deref()
        ),
        ("hand_equip", 1, 1, Some("card_sha"))
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TY_CHENSHI)
        .unwrap();
    assert_eq!(character.name_key, "ty_chenshi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [TYZHUAN]);

    let compiled = content_compiler::compile_source_named("tyzhuan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 158);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn sha_cost_is_atomic_then_owner_chooses_and_gains_target_equipment() {
    let mut setup = setup(2_026_072_701);
    let weapon = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        158_101,
        CardKind::Weapon,
    );
    let shan = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        158_102,
        CardKind::Shan,
    );
    let sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        158_103,
        CardKind::Sha,
    );
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(
        setup.state.clone(),
        setup.owner,
        setup.target,
        setup.skill,
        shan,
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, accepted) = activate(setup.state, setup.owner, setup.target, setup.skill, sha, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    assert_eq!(
        state
            .zones
            .get(state.cards.get(sha).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
    let (state, accepted) = resolve(state, setup.owner, vec![weapon.raw()], 3);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let gained = state.cards.get(weapon).unwrap();
    assert_eq!(gained.owner, setup.owner);
    assert_eq!(state.zones.get(gained.zone).unwrap().kind, ZoneKind::Hand);
}

#[test]
fn equipment_use_offers_optional_draw_but_non_equipment_does_not() {
    let mut setup = setup(2_026_072_702);
    let sha = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        158_201,
        CardKind::Sha,
    );
    let equip = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        158_202,
        CardKind::Armor,
    );
    let mut tx = Transaction::begin(setup.state);
    let hand = tx.state().hand_count(setup.owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: setup.target,
            card: sha,
            targets: vec![setup.owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: setup.target,
                card: equip,
                targets: vec![setup.target],
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, accepted) = resolve(tx.commit().state, setup.owner, vec![1], 4);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand + 1);
}
