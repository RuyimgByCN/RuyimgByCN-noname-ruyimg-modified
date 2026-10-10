//! Golden coverage for 界许褚「裸衣」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_XUZHU: u32 = 4_251_485_779;
const NEW_RELUOYI: u32 = 3_935_960_913;
const RELUOYI2: u32 = 3_768_135_792;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/new_reluoyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    top: Vec<CardId>,
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, top_kinds: [CardKind; 3]) -> Setup {
    let package = content_compiler::compile_source_named("new-reluoyi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NEW_RELUOYI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    let top = top_kinds
        .into_iter()
        .enumerate()
        .map(|(index, kind)| {
            inject(
                &mut tx,
                PlayerId::NONE,
                ZoneKind::DrawPile,
                kind,
                [Suit::Spade, Suit::Heart, Suit::Club][index],
                10 + index as u8,
            )
        })
        .collect();
    Setup {
        state: tx.commit().state,
        owner,
        top,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn open_choice(setup: &Setup) -> (GameState, PipelineResult) {
    let (state, result) = submit(setup.state.clone(), setup.owner, CommandKind::EndPhase, 1);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    (state, result)
}

fn resolve_bool(
    state: GameState,
    owner: PlayerId,
    accepted: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![u32::from(accepted)],
            cancelled: false,
        },
        key,
    )
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_complete_re_xuzhu_and_abi317_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 317);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_XUZHU)
        .unwrap();
    assert_eq!(character.skills, [NEW_RELUOYI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NEW_RELUOYI)
        .unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target_draw_not_replaced");
    assert!(!trigger.optional);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == RELUOYI2 && modifier.target == "sha_damage" && modifier.value == 1
    }));

    let compiled = content_compiler::compile_source_named("new_reluoyi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 317);
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
fn accepting_gains_basic_duel_and_weapon_and_expires_buff_next_owner_turn() {
    let setup = setup(
        2_026_073_301,
        [CardKind::Sha, CardKind::TrickJuedou, CardKind::Weapon],
    );
    let (state, _) = open_choice(&setup);
    let (mut state, result) = resolve_bool(state, setup.owner, true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
    assert!(setup
        .top
        .iter()
        .all(|card| state.hand_cards(setup.owner).contains(card)));
    assert!(has_skill(&state, setup.owner, RELUOYI2));
    assert_eq!(
        engine_core::query::sha_damage_for(&state, setup.owner, 1),
        2
    );

    for key in 3..20 {
        if state.turn.current == setup.owner && state.turn.phase == Phase::Prepare {
            break;
        }
        let current = state.turn.current;
        let (next, advanced) = submit(state, current, CommandKind::EndPhase, key);
        assert!(matches!(advanced.result, CommandResult::Accepted { .. }));
        assert!(advanced.choice.is_none());
        state = next;
    }
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert!(!has_skill(&state, setup.owner, RELUOYI2));
    assert_eq!(
        engine_core::query::sha_damage_for(&state, setup.owner, 1),
        1
    );
}

#[test]
fn accepting_discards_cards_outside_the_eligible_categories() {
    let setup = setup(
        2_026_073_302,
        [CardKind::Tao, CardKind::Armor, CardKind::TrickWuzhong],
    );
    let eligible = setup.top[0];
    let rejected = [setup.top[1], setup.top[2]];
    let (state, _) = open_choice(&setup);
    let (state, _) = resolve_bool(state, setup.owner, true, 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.hand_cards(setup.owner).contains(&eligible));
    assert!(rejected
        .iter()
        .all(|card| state.zones.get(discard).unwrap().cards.contains(card)));
}

#[test]
fn declining_discards_all_revealed_cards_and_keeps_the_normal_draw() {
    let setup = setup(
        2_026_073_303,
        [CardKind::Sha, CardKind::TrickJuedou, CardKind::Weapon],
    );
    let (state, _) = open_choice(&setup);
    let (state, result) = resolve_bool(state, setup.owner, false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 2);
    assert!(!has_skill(&state, setup.owner, RELUOYI2));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(setup
        .top
        .iter()
        .all(|card| state.zones.get(discard).unwrap().cards.contains(card)));
}
