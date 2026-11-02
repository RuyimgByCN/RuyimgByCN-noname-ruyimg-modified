//! Golden coverage for REFRESH 界曹操's rewritten 奸雄/护驾 contract.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NEW_REJIANXIONG: u32 = 3_449_956_505;
const REHUJIA: u32 = 3_691_667_990;
const RE_CAOCAO: u32 = 3_230_095_684;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/re_caocao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

fn character(id: u32, group: &str, is_lord: bool) -> PackageCharacterEntry {
    PackageCharacterEntry {
        voice_assets: Default::default(),
        id,
        name_key: format!("re_caocao_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

struct Setup {
    state: GameState,
    owner: PlayerId,
    wei: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let mut package = content_compiler::compile_source_named("refresh-re-caocao-test", SOURCE)
        .expect("compile re_caocao");
    package.character_index = vec![
        character(RE_CAOCAO, "wei", true),
        character(9_199_001, "wei", false),
        character(9_199_002, "shu", false),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_726_199, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let mut rest = seats.iter().copied().filter(|player| *player != owner);
    let wei = rest.next().unwrap();
    let other = rest.next().unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = RE_CAOCAO;
    tx.state_mut().players.get_mut(wei).unwrap().character_def = 9_199_001;
    tx.state_mut().players.get_mut(other).unwrap().character_def = 9_199_002;

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_REJIANXIONG),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REHUJIA),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 20;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 20;
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        wei,
        other,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

fn inject_processing_card(state: &mut GameState) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_199_100),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_abi232_complete_re_caocao() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_CAOCAO)
        .unwrap();
    assert!(character.is_lord);
    assert_eq!(character.skills, vec![NEW_REJIANXIONG, REHUJIA]);
    let hujia = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == REHUJIA)
        .unwrap();
    assert!(hujia.lord_only);
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == REHUJIA
            && view.as_kind == "shan"
            && view.source_zone.as_deref() == Some("group_wei")
    }));
}

#[test]
fn new_rejianxiong_gains_processing_damage_card_then_draws() {
    let mut setup = setup();
    let card = inject_processing_card(&mut setup.state);
    let hand_before = setup.state.hand_count(setup.owner);
    let mut tx = Transaction::begin(setup.state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: setup.other,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(tx.state().resolution.pending_observer, setup.owner.raw());
    let (state, resolved) = resolve(tx.commit().state, setup.owner, vec![1], 1);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand_before + 2);
    assert!(state.hand_cards(setup.owner).contains(&card));
}

#[test]
fn rehujia_draw_is_chosen_by_outside_turn_wei_player_once_per_turn() {
    let initial = setup();
    let hand_before = initial.state.hand_count(initial.owner);
    let mut tx = Transaction::begin(initial.state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: initial.wei,
            respond_to: initial.other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(tx.state().resolution.pending_observer, initial.wei.raw());
    let (state, resolved) = resolve(tx.commit().state, initial.wei, vec![1], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(initial.owner), hand_before + 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: initial.wei,
            respond_to: initial.other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(initial.owner), hand_before + 1);

    let mut own_turn = setup();
    own_turn.state.turn.current = own_turn.wei;
    let mut tx = Transaction::begin(own_turn.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: own_turn.wei,
            respond_to: own_turn.other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
