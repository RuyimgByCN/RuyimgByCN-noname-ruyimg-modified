//! Golden coverage for SP 董卓「横征」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const HENGZHENG: u32 = 3_568_349_953;
const SP_DONGZHUO: u32 = 2_623_303_183;
const LIANYING: u32 = 21;
const GAIN_GUARD: u32 = 4_000_000_002;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/hengzheng.skill");
const GUIXIN_SOURCE: &str = include_str!("../../../../content/original/ported/extra/guixin.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");
const HENGZHENG_LOGIC_HASH: [u8; 32] = [
    51, 75, 221, 18, 16, 179, 8, 161, 79, 14, 244, 210, 172, 213, 57, 187, 250, 214, 132, 239, 137,
    187, 51, 206, 135, 234, 209, 55, 90, 193, 233, 250,
];
const SP_PACKAGE_LOGIC_HASH: [u8; 32] = [
    104, 11, 251, 180, 248, 79, 187, 16, 89, 76, 180, 14, 141, 177, 202, 117, 119, 248, 212, 56,
    232, 136, 9, 34, 34, 192, 63, 77, 154, 118, 188, 178,
];
const GUIXIN_LOGIC_HASH: [u8; 32] = [
    131, 229, 90, 4, 186, 198, 104, 70, 86, 232, 54, 203, 123, 216, 108, 106, 162, 185, 21, 128,
    76, 230, 51, 135, 225, 1, 120, 29, 62, 138, 32, 77,
];

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    last: PlayerId,
    first_hand: CardId,
    protected_armor: CardId,
    second_judge: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_600 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let source = format!(
        "{SOURCE}\nskill gain_guard({GAIN_GUARD}) locked {{\n  modifier card_gain_protected(armor_or_treasure) override 1;\n}}\n"
    );
    let package = compile_source_named("sp-hengzheng-test", &source).unwrap();
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&standard)
        .unwrap();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_729_01, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[2];
    let first = players[3];
    let second = players[0];
    let last = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HENGZHENG),
        },
        EffectRequest::AddSkill {
            player: first,
            skill_def: SkillDefId::new(GAIN_GUARD),
        },
        EffectRequest::AddSkill {
            player: first,
            skill_def: SkillDefId::new(LIANYING),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let first_hand = inject(tx.state_mut(), first, ZoneKind::Hand, CardKind::Other, 3);
    let protected_armor = inject(tx.state_mut(), first, ZoneKind::Equip, CardKind::Armor, 4);
    let second_judge = inject(
        tx.state_mut(),
        second,
        ZoneKind::Judge,
        CardKind::TrickLe,
        5,
    );
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        first,
        last,
        first_hand,
        protected_armor,
        second_judge,
    }
}

fn dispatch_draw(
    state: GameState,
    owner: PlayerId,
) -> (GameState, Option<engine_api::ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let dispatch = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    match dispatch {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) | Err(EffectError::ChoiceRequested) => {}
            Err(error) => panic!("draw effects failed: {error:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("draw dispatch failed: {error:?}"),
    }
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let command = Command {
        actor,
        expected_revision: ExpectedRevision(state.revision),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled,
        },
    };
    apply_command(state, command)
}

#[test]
fn package_and_module_freeze_hengzheng_without_changing_guixin_contract() {
    let compiled = compile_source_named("sp-hengzheng", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 300);
    let trigger = compiled.trigger_index.first().unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(
        trigger.filter,
        "owner_is_target_draw_not_fixed_hp_one_or_hand_empty"
    );
    assert!(trigger.optional);
    let module = compiled
        .modules
        .iter()
        .find(|module| module.name == "hengzheng_phase_draw")
        .unwrap();
    assert_eq!(module.code, [83, 90, 31, 1, 1]);
    assert_eq!(module.logic_hash, HENGZHENG_LOGIC_HASH);
    assert_eq!(
        rule_vm::bytecode::effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW,
        287
    );

    let package = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    assert_eq!(package.manifest.logic_hash, SP_PACKAGE_LOGIC_HASH);
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    assert_eq!(package.modules.len(), 295);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SP_DONGZHUO)
        .unwrap();
    assert_eq!(character.name_key, "sp_dongzhuo");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.skills, [HENGZHENG]);

    let guixin = compile_source_named("legacy-guixin", GUIXIN_SOURCE).unwrap();
    assert_eq!(guixin.manifest.rule_vm_abi_version, 237);
    assert_eq!(guixin.modules[0].code, [83, 80, 231, 1]);
    assert_eq!(guixin.modules[0].logic_hash, GUIXIN_LOGIC_HASH);
    assert_eq!(
        rule_vm::bytecode::effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER,
        231
    );
    assert_eq!(
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver.tag(),
        164
    );
}

#[test]
fn requires_hp_one_or_empty_hand_and_rejects_an_already_fixed_draw() {
    let setup = setup();
    let mut ineligible = setup.state.clone();
    inject(
        &mut ineligible,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        7,
    );
    let (_, choice) = dispatch_draw(ineligible, setup.owner);
    assert!(choice.is_none());

    let mut hp_one = setup.state.clone();
    inject(&mut hp_one, setup.owner, ZoneKind::Hand, CardKind::Other, 8);
    hp_one.players.get_mut(setup.owner).unwrap().hp = 1;
    let (_, choice) = dispatch_draw(hp_one, setup.owner);
    assert_eq!(choice.unwrap().kind, ChoiceKind::Confirm);

    let mut fixed = setup.state;
    fixed.players.get_mut(setup.owner).unwrap().hp = 1;
    fixed.turn.set_draw_phase_fixed_count(3);
    let (_, choice) = dispatch_draw(fixed, setup.owner);
    assert!(choice.is_none());
}

#[test]
fn gains_clockwise_gainable_cards_waits_for_nested_triggers_and_does_not_turn_over() {
    let setup = setup();
    let (state, optional) = dispatch_draw(setup.state, setup.owner);
    let optional = optional.unwrap();
    assert_eq!(optional.kind, ChoiceKind::Confirm);
    assert_eq!(optional.observer, setup.owner);

    let (state, first) = resolve(state, setup.owner, vec![1], false, 1);
    let first = first.choice.unwrap();
    assert_eq!(first.kind, ChoiceKind::SelectCards);
    assert_eq!(first.observer, setup.owner);
    assert_eq!(
        first
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.first_hand.raw()]
    );
    assert!(!first
        .options
        .iter()
        .any(|option| option.token == setup.protected_armor.raw()));

    let (state, nested) = resolve(state, setup.owner, vec![setup.first_hand.raw()], false, 2);
    let nested = nested.choice.unwrap();
    assert_eq!(nested.kind, ChoiceKind::Confirm);
    assert_eq!(nested.observer, setup.first);

    let (state, second) = resolve(state, setup.first, vec![1], false, 3);
    let second = second.choice.unwrap();
    assert_eq!(second.kind, ChoiceKind::SelectCards);
    assert_eq!(
        second
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.second_judge.raw()]
    );
    assert_eq!(state.hand_count(setup.first), 1);

    let (state, done) = resolve(state, setup.owner, vec![setup.second_judge.raw()], false, 4);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
    assert!(!state.players.get(setup.owner).unwrap().turned_over);
    assert!(state.hand_cards(setup.owner).contains(&setup.first_hand));
    assert!(state.hand_cards(setup.owner).contains(&setup.second_judge));
    assert_eq!(
        state.cards.get(setup.protected_armor).unwrap().owner,
        setup.first
    );
    assert!(state.hand_cards(setup.last).is_empty());
}
