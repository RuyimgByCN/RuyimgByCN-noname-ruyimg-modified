//! Golden coverage for 爻疑/谶纬 rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ObserverKind,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::projection::project_observer;
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YAO_YAOYI: u32 = 2_523_033_082;
const YAO_CHENWEI: u32 = 2_387_616_429;
const YAO_CHARACTER: u32 = 3_530_826_120;
const YAOYI_SOURCE: &str = include_str!("../../../content/original/ported/old/yao_yaoyi.skill");
const CHENWEI_SOURCE: &str = include_str!("../../../content/original/ported/old/yao_chenwei.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    yaoyi: SkillId,
    chenwei: SkillId,
}

fn setup() -> Setup {
    let mut package = content_compiler::compile_source_named(
        "yao-yuanshu-test",
        &format!("{YAOYI_SOURCE}\n{CHENWEI_SOURCE}"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    package.character_index.push(PackageCharacterEntry {
        id: YAO_CHARACTER,
        name_key: "x_yao_yuanshu".into(),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![YAO_YAOYI, YAO_CHENWEI],
        sex: "male".into(),
        group: "qun".into(),
        is_lord: false,
    clans: vec![],
        tags: Vec::new(),
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: Vec::new(),
    });
    package.manifest.logic_hash = package.compute_logic_hash();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_932, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YAO_YAOYI),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YAO_CHENWEI),
            },
        ])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let initial_cards = tx.state().hand_cards(owner);
    tx.state_mut()
        .face_down_hand_cards
        .extend(initial_cards.into_iter().take(4));
    let skill = |def| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
            .unwrap()
    };
    let yaoyi = skill(YAO_YAOYI);
    let chenwei = skill(YAO_CHENWEI);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        other,
        yaoyi,
        chenwei,
    }
}

fn choose(state: GameState, token: u32, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice key {key}: {:?}",
        result.result
    );
    state
}

fn use_as(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    kind: CardKind,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: kind.tag(),
                targets,
                equip_slot: 0,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "use key {key}: {:?}",
        result.result
    );
    state
}

fn pass_wuxie(mut state: GameState, mut key: u64) -> GameState {
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        key += 1;
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
    }
    state
}

#[test]
fn initial_cards_are_hidden_and_each_declared_name_is_once_per_turn() {
    let mut setup = setup();
    let owner_hand = setup.state.hand_cards(setup.owner);
    let tagged: Vec<_> = owner_hand
        .iter()
        .copied()
        .filter(|card| setup.state.face_down_hand_cards.contains(card))
        .collect();
    assert_eq!(tagged.len(), 4);
    let snapshot = project_observer(&setup.state, ObserverKind::Player(setup.owner));
    assert!(tagged
        .iter()
        .all(|card| snapshot.known_cards.iter().all(|view| view.id != *card)));

    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveSkill {
            skill: setup.chenwei,
        }],
    )
    .unwrap();
    setup.state = tx.commit().state;
    let source = tagged[0];
    let state = use_as(
        setup.state,
        setup.owner,
        setup.yaoyi,
        CardKind::TrickWuzhong,
        Vec::new(),
        1,
    );
    let state = pass_wuxie(choose(state, source.raw(), 2), 100);
    assert!(!state.face_down_hand_cards.contains(&source));
    assert_eq!(
        state
            .zones
            .get(state.cards.get(source).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
    let revision = state.revision;
    let (_, result) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::UseSkillCard {
                skill: setup.yaoyi,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: Vec::new(),
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn chenwei_alternates_toggle_and_gain_face_down_branches() {
    let setup = setup();
    let tagged = setup
        .state
        .hand_cards(setup.owner)
        .into_iter()
        .filter(|card| setup.state.face_down_hand_cards.contains(card))
        .collect::<Vec<_>>();
    let untagged = setup
        .state
        .hand_cards(setup.owner)
        .into_iter()
        .find(|card| !setup.state.face_down_hand_cards.contains(card))
        .unwrap();
    let state = use_as(
        setup.state,
        setup.owner,
        setup.yaoyi,
        CardKind::TrickWuzhong,
        Vec::new(),
        10,
    );
    let state = choose(state, tagged[0].raw(), 11);
    let state = choose(state, 1, 12);
    let state = choose(state, setup.other.raw(), 13);
    let state = pass_wuxie(choose(state, untagged.raw(), 14), 200);
    assert!(state.face_down_hand_cards.contains(&untagged));
    assert_eq!(state.skills.get(setup.chenwei).unwrap().view_as_branch, 1);

    let target_card = state.hand_cards(setup.other)[0];
    let source = state
        .hand_cards(setup.owner)
        .into_iter()
        .find(|card| state.face_down_hand_cards.contains(card))
        .unwrap();
    let state = use_as(
        state,
        setup.owner,
        setup.yaoyi,
        CardKind::Jiu,
        Vec::new(),
        15,
    );
    let state = choose(state, source.raw(), 16);
    let state = choose(state, 1, 17);
    let state = choose(state, setup.other.raw(), 18);
    let state = choose(state, target_card.raw(), 19);
    assert_eq!(state.cards.get(target_card).unwrap().owner, setup.owner);
    assert!(state.face_down_hand_cards.contains(&target_card));
    assert_eq!(state.skills.get(setup.chenwei).unwrap().view_as_branch, 0);
}

#[test]
fn package_publishes_yao_pair_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YAO_YAOYI));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YAO_CHENWEI));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == YAO_CHARACTER));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
