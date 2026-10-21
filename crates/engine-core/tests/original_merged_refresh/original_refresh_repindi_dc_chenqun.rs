//! Golden coverage for 界陈群「界品第」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_CHENQUN: u32 = 2_755_954_297;
const REPINDI: u32 = 3_416_188_911;
const DCFAEN: u32 = 2_290_375_740;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/repindi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, u64::from(REPINDI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
            skill_def: SkillDefId::new(REPINDI),
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
        first: players[1],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn repindi_skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == REPINDI)
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
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = repindi_skill(&state, owner);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![card],
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn enter_new_play_phase(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
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
    tx.commit().state
}

fn is_linked(state: &GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

#[test]
fn package_publishes_complete_dc_chenqun_and_exact_abi314_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 314);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_CHENQUN)
        .unwrap();
    assert_eq!(character.skills, [REPINDI, DCFAEN]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REPINDI)
        .unwrap();
    let targets = active.targets.as_ref().unwrap();
    assert!(targets.allow_self);
    assert!(targets.unused_by_skill_this_play_phase);
    assert!(!targets.unused_by_skill_this_turn);
    assert!(
        active
            .selection
            .as_ref()
            .unwrap()
            .unused_category_by_skill_this_play_phase
    );
    let compiled = content_compiler::compile_source_named("repindi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 314);
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
fn phase_uniqueness_turn_count_and_link_toggle_match_legacy_js() {
    let mut setup = setup();
    {
        let owner = setup.state.players.get_mut(setup.owner).unwrap();
        owner.max_hp = 4;
        owner.hp = 3;
    }

    let first_cost = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_301);
    inject(&mut setup.state, setup.owner, CardKind::Tao, 99_300);
    let (state, opened) = activate(setup.state, setup.owner, setup.owner, first_cost, 1);
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded
    );
    let (mut state, drawn) = resolve(state, setup.owner, vec![1], 2);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert!(is_linked(&state, setup.owner));

    let repeated_target_cost = inject(&mut state, setup.owner, CardKind::TrickWuzhong, 99_302);
    let before = canonical_state_hash(&state);
    let (state, repeated_target) =
        activate(state, setup.owner, setup.owner, repeated_target_cost, 3);
    assert!(matches!(
        repeated_target.result,
        CommandResult::Rejected { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before);

    let mut state = state;
    let repeated_category_cost = inject(&mut state, setup.owner, CardKind::Shan, 99_303);
    let before = canonical_state_hash(&state);
    let (state, repeated_category) =
        activate(state, setup.owner, setup.first, repeated_category_cost, 4);
    assert!(matches!(
        repeated_category.result,
        CommandResult::Rejected { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before);

    let mut state = state;
    let second_cost = inject(&mut state, setup.owner, CardKind::TrickWuzhong, 99_304);
    let only_target_card = inject(&mut state, setup.first, CardKind::Tao, 99_305);
    let (state, second) = activate(state, setup.owner, setup.first, second_cost, 5);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    let (state, discard_branch) = resolve(state, setup.owner, vec![2], 6);
    let choice = discard_branch.choice.as_ref().unwrap();
    assert_eq!(choice.observer, setup.first);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let (state, discarded) = resolve(state, setup.first, vec![only_target_card.raw()], 7);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(is_linked(&state, setup.owner));
    assert_eq!(
        state
            .skills
            .get(repindi_skill(&state, setup.owner))
            .unwrap()
            .use_count,
        2
    );

    let mut state = enter_new_play_phase(state, setup.owner);
    let third_cost = inject(&mut state, setup.owner, CardKind::Sha, 99_306);
    let (state, third) = activate(state, setup.owner, setup.owner, third_cost, 8);
    assert!(matches!(third.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .get(repindi_skill(&state, setup.owner))
            .unwrap()
            .use_count,
        3
    );
    let (state, resolved) = resolve(state, setup.owner, vec![1], 9);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!is_linked(&state, setup.owner));
}
