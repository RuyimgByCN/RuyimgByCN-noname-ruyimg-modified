//! Golden coverage for SP张济「掳掠」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSLULVE: u32 = 2_729_205_545;
const NS_ZHANGJI: u32 = 3_174_048_491;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/nslulve.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    empty_target: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
        skill_def: SkillDefId::new(NSLULVE),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NSLULVE)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: others[0],
        empty_target: others[1],
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
    cards: Vec<CardId>,
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
                card_targets: cards,
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
fn package_publishes_abi157_contract_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 141);
    assert_eq!(package.modules.len(), 724);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == NSLULVE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.has_equip);
    assert!(targets.selected_cards_match_target_equip_count);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 1, 255)
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NS_ZHANGJI)
        .unwrap();
    assert_eq!(character.name_key, "ns_zhangji");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [NSLULVE]);

    let compiled = content_compiler::compile_source_named("nslulve", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 157);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn exact_equipment_count_is_an_atomic_cost_before_damage() {
    let mut setup = setup(2_026_072_601);
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        157_101,
        CardKind::Weapon,
    );
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        157_102,
        CardKind::Armor,
    );
    let first = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        157_103,
        CardKind::Sha,
    );
    let second = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        157_104,
        CardKind::Shan,
    );
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(
        setup.state.clone(),
        setup.owner,
        setup.target,
        setup.skill,
        vec![first],
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, accepted) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        vec![first, second],
        2,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(first).unwrap().zone, discard);
    assert_eq!(state.cards.get(second).unwrap().zone, discard);
    assert_eq!(state.skills.get(setup.skill).unwrap().use_count, 1);
}

#[test]
fn selection_barrier_uses_the_targets_live_equipment_count() {
    let mut setup = setup(2_026_072_602);
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        157_201,
        CardKind::Weapon,
    );
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        157_202,
        CardKind::Armor,
    );
    let first = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        157_203,
        CardKind::Sha,
    );
    let second = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        157_204,
        CardKind::Shan,
    );
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, offered) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        Vec::new(),
        1,
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );

    let (state, resolved) = resolve(state, setup.owner, vec![first.raw(), second.raw()], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn unequipped_or_unpayable_targets_are_rejected_atomically() {
    let mut empty = setup(2_026_072_603);
    let cost = inject(
        &mut empty.state,
        empty.owner,
        ZoneKind::Hand,
        157_301,
        CardKind::Sha,
    );
    let before = canonical_state_hash(&empty.state);
    let (same, rejected) = activate(
        empty.state,
        empty.owner,
        empty.empty_target,
        empty.skill,
        vec![cost],
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut unpayable = setup(2_026_072_604);
    for (offset, kind) in [CardKind::Weapon, CardKind::Armor, CardKind::HorseOff]
        .into_iter()
        .enumerate()
    {
        inject(
            &mut unpayable.state,
            unpayable.target,
            ZoneKind::Equip,
            157_310 + offset as u32,
            kind,
        );
    }
    inject(
        &mut unpayable.state,
        unpayable.owner,
        ZoneKind::Hand,
        157_320,
        CardKind::Sha,
    );
    let before = canonical_state_hash(&unpayable.state);
    let (same, rejected) = activate(
        unpayable.state,
        unpayable.owner,
        unpayable.target,
        unpayable.skill,
        Vec::new(),
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
