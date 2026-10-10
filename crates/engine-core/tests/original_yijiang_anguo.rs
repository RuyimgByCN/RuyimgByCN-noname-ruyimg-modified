//! Golden for 旧朱治「安国」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ANGUO: u32 = 3_231_604_344;
const OLD_ZHUZHI: u32 = 4_018_190_437;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/anguo.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];

    let remove: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    run_effects(
        &mut tx,
        players
            .iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player: *player,
                count: u32::MAX,
            })
            .collect(),
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ANGUO),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == ANGUO)
        .unwrap()
        .id;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject_equipment(
    state: &mut GameState,
    target: PlayerId,
    kind: CardKind,
    weapon_range: u8,
    def: u32,
) -> CardId {
    assert!(kind.is_equip());
    let equip = state.zones.find(ZoneKind::Equip, target).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range,
        zone: equip,
        owner: target,
        face_up: true,
    });
    state.zones.get_mut(equip).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    pipeline: PipelineResult,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.expect("equipment choice");
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 1, false)
    );
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == card.raw()));
    submit(
        state,
        choice.observer,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![card.raw()],
            cancelled: false,
        },
        key,
    )
}

#[test]
fn packages_publish_abi97_active_contract_and_old_zhuzhi_closure() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert_eq!(yijiang.skill_index.len(), 281);
    assert_eq!(yijiang.character_index.len(), 72);
    assert_eq!(old.character_index.len(), 78);

    let active = yijiang
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ANGUO)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(!targets.allow_self);
    assert!(targets.has_equip);

    let compiled = content_compiler::compile_source_named("anguo", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 97);
    assert_eq!(
        compiled.modules[0].logic_hash,
        yijiang
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash
    );

    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_ZHUZHI)
        .unwrap();
    assert_eq!(character.name_key, "old_zhuzhi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [ANGUO]);
}

#[test]
fn reclaiming_a_range_two_weapon_draws_only_after_the_range_shrinks() {
    let mut setup = setup(2_026_072_301);
    let weapon = inject_equipment(&mut setup.state, setup.target, CardKind::Weapon, 2, 97_301);
    let (state, activated) = activate(&setup, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, resolved) = resolve(state, activated, weapon, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));

    assert_eq!(state.hand_cards(setup.owner).len(), 1);
    assert_eq!(state.hand_cards(setup.target), [weapon]);
    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        state.zones.find(ZoneKind::Hand, setup.target).unwrap()
    );
    assert_eq!(state.skills.get(setup.skill).unwrap().use_count, 1);
}

#[test]
fn reclaiming_non_range_equipment_does_not_draw() {
    let mut setup = setup(2_026_072_302);
    let armor = inject_equipment(&mut setup.state, setup.target, CardKind::Armor, 0, 97_302);
    let (state, activated) = activate(&setup, 3);
    let (state, resolved) = resolve(state, activated, armor, 4);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.owner).is_empty());
    assert_eq!(state.hand_cards(setup.target), [armor]);
}

#[test]
fn empty_equipment_target_and_second_use_are_atomically_rejected() {
    let empty = setup(2_026_072_303);
    let before = canonical_state_hash(&empty.state);
    let (same, rejected) = activate(&empty, 5);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    assert_eq!(same.skills.get(empty.skill).unwrap().use_count, 0);

    let mut setup = setup(2_026_072_304);
    let armor = inject_equipment(&mut setup.state, setup.target, CardKind::Armor, 0, 97_303);
    let (state, activated) = activate(&setup, 6);
    let (mut state, resolved) = resolve(state, activated, armor, 7);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    inject_equipment(&mut state, setup.target, CardKind::Armor, 0, 97_304);
    let before = canonical_state_hash(&state);
    let (same, rejected) = submit(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        8,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn equipment_choice_revalidates_the_card_at_resume() {
    let mut setup = setup(2_026_072_305);
    let armor = inject_equipment(&mut setup.state, setup.target, CardKind::Armor, 0, 97_305);
    let (mut state, activated) = activate(&setup, 9);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));

    let equip = state.zones.find(ZoneKind::Equip, setup.target).unwrap();
    let hand = state.zones.find(ZoneKind::Hand, setup.target).unwrap();
    state
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .retain(|card| *card != armor);
    state.zones.get_mut(hand).unwrap().cards.push(armor);
    let card = state.cards.get_mut(armor).unwrap();
    card.zone = hand;
    card.face_up = false;

    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, activated, armor, 10);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
