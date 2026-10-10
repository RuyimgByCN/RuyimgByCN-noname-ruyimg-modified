//! Golden coverage for 蒋钦「尚义」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZYSHANGYI: u32 = 2_778_115_845;
const JIANGQING: u32 = 2_799_920_397;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/zyshangyi.skill");
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
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
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
        skill_def: SkillDefId::new(ZYSHANGYI),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZYSHANGYI)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject_card(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: u8::from(matches!(kind, CardKind::Weapon)),
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

#[test]
fn package_publishes_jiangqing_and_the_exact_active_contract_at_abi153() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 838);
    assert_eq!(package.character_index.len(), 382);
    assert_eq!(package.active_index.len(), 137);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZYSHANGYI && skill.name == "zyshangyi"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ZYSHANGYI)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|target| (target.min, target.max, target.allow_self)),
        Some((1, 1, false))
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == JIANGQING)
        .unwrap();
    assert_eq!(character.name_key, "jiangqing");
    assert_eq!(character.skills, [ZYSHANGYI]);

    let compiled = content_compiler::compile_source_named("zyshangyi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 153);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn reveals_the_owner_hand_to_the_target_and_cancellation_keeps_target_cards() {
    let mut setup = setup(2_026_072_503);
    let owner_sha = inject_card(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Sha,
        153_001,
    );
    let owner_shan = inject_card(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Shan,
        153_002,
    );
    let target_hand = inject_card(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        CardKind::Tao,
        153_003,
    );
    let target_equip = inject_card(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        CardKind::Weapon,
        153_004,
    );
    let (state, output) = activate(setup.state, setup.owner, setup.target, setup.skill, 1);

    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[owner_sha, owner_shan] && to == &[setup.target]
    )));
    let choice = output.choice.unwrap();
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 1, true)
    );
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target_hand.raw()]
    );

    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(target_hand).unwrap().owner, setup.target);
    assert_eq!(state.cards.get(target_equip).unwrap().owner, setup.target);
}

#[test]
fn selected_hand_card_is_discarded_but_equipment_is_never_offered() {
    let mut setup = setup(2_026_072_504);
    let target_hand = inject_card(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        CardKind::Tao,
        153_011,
    );
    let target_equip = inject_card(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        CardKind::Weapon,
        153_012,
    );
    let (state, output) = activate(setup.state, setup.owner, setup.target, setup.skill, 10);
    let choice = output.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target_hand.raw()]
    );
    let (state, finished) = resolve(state, setup.owner, vec![target_hand.raw()], false, 11);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_hand).unwrap().zone, discard);
    assert_eq!(
        state.cards.get(target_equip).unwrap().zone,
        state.zones.find(ZoneKind::Equip, setup.target).unwrap()
    );
}

#[test]
fn empty_target_hand_is_a_noop_while_self_target_and_second_use_are_rejected() {
    let mut setup = setup(2_026_072_505);
    let owner_card = inject_card(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Sha,
        153_021,
    );
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(setup.state, setup.owner, setup.owner, setup.skill, 20);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, accepted) = activate(same, setup.owner, setup.target, setup.skill, 21);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.choice.is_none());
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[owner_card] && to == &[setup.target]
    )));
    let before = canonical_state_hash(&state);
    let (same, repeated) = activate(state, setup.owner, setup.target, setup.skill, 22);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
