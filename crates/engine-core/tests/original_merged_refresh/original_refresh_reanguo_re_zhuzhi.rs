//! Golden coverage for 界朱治「安国」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_ZHUZHI: u32 = 3_807_098_010;
const REANGUO: u32 = 3_231_751_411;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/reanguo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("reanguo-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REANGUO),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
        other: players[2],
    }
}

fn inject(
    state: &mut GameState,
    zone_kind: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == REANGUO)
        .unwrap()
        .id
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

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = skill(&state, owner);
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

fn resolve_recast(
    state: GameState,
    owner: PlayerId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: cards.into_iter().map(CardId::raw).collect(),
            cancelled: false,
        },
        key,
    )
}

fn equip_count(state: &GameState, player: PlayerId) -> usize {
    state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone))
        .map_or(0, |zone| zone.cards.len())
}

#[test]
fn package_publishes_complete_re_zhuzhi_and_abi321_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 321);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHUZHI)
        .unwrap();
    assert_eq!(character.name_key, "re_zhuzhi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [REANGUO]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REANGUO)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(!targets.allow_self);
    let compiled = content_compiler::compile_source_named("reanguo", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 321);
    assert_eq!(
        compiled.modules[0].logic_hash,
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash
    );
}

#[test]
fn target_completes_all_three_branches_then_owner_may_recast() {
    let mut setup = setup(32_117_001);
    let recast = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        99_601,
    );
    setup.state.players.get_mut(setup.target).unwrap().hp = 2;
    setup.state.players.get_mut(setup.owner).unwrap().hp = 3;
    setup.state.players.get_mut(setup.other).unwrap().hp = 4;

    let (state, activated) = activate(setup.state, setup.owner, setup.target, 1);
    assert_eq!(
        activated.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveReanguo
    );
    assert_eq!(state.hand_count(setup.target), 1);
    assert_eq!(state.players.get(setup.target).unwrap().hp, 3);
    assert_eq!(equip_count(&state, setup.target), 1);
    assert_eq!(equip_count(&state, setup.owner), 0);
    assert!(activated
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == recast.raw()));

    let (state, resolved) = resolve_recast(state, setup.owner, vec![recast], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 1);
    assert_ne!(state.hand_cards(setup.owner), [recast]);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(recast).unwrap().zone, discard);
}

#[test]
fn owner_executes_only_the_branches_the_target_did_not_complete() {
    let mut setup = setup(32_117_002);
    let recast = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        99_610,
    );
    inject(
        &mut setup.state,
        ZoneKind::Equip,
        setup.target,
        CardKind::Armor,
        99_611,
    );
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.other,
        CardKind::Shan,
        99_612,
    );
    setup.state.players.get_mut(setup.owner).unwrap().hp = 2;
    setup.state.players.get_mut(setup.target).unwrap().hp = 4;
    setup.state.players.get_mut(setup.other).unwrap().hp = 3;

    let (state, activated) = activate(setup.state, setup.owner, setup.target, 10);
    assert_eq!(
        activated.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(state.hand_count(setup.target), 1);
    assert_eq!(state.players.get(setup.target).unwrap().hp, 4);
    assert_eq!(equip_count(&state, setup.target), 1);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert_eq!(equip_count(&state, setup.owner), 1);
    assert!(activated
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == recast.raw()));
}

#[test]
fn missing_draw_pile_equipment_does_not_count_as_completed_or_open_recast() {
    let mut setup = setup(32_117_003);
    let retained = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        99_620,
    );
    setup.state.players.get_mut(setup.target).unwrap().hp = 2;
    setup.state.players.get_mut(setup.owner).unwrap().hp = 3;
    setup.state.players.get_mut(setup.other).unwrap().hp = 4;
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let equipment: Vec<_> = setup
        .state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .copied()
        .filter(|card| setup.state.cards.get(*card).unwrap().kind.is_equip())
        .collect();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: equipment,
            to: discard,
            from: Some(draw),
        }],
    )
    .unwrap();
    let state = tx.commit().state;

    let (state, activated) = activate(state, setup.owner, setup.target, 20);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(activated.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(equip_count(&state, setup.target), 0);
    assert_eq!(equip_count(&state, setup.owner), 0);
    assert_eq!(state.cards.get(retained).unwrap().owner, setup.owner);
}
