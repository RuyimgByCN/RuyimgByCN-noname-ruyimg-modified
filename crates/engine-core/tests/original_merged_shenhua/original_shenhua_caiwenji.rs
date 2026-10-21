//! Golden coverage for 神话蔡琰「悲歌」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BEIGE: u32 = 2_536_833_640;
const TEST_REWRITE: u32 = 9_253_001;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/beige.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    target: PlayerId,
    source: PlayerId,
    skill: engine_api::SkillId,
}

fn character(id: u32) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("beige_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "female".into(),
        group: "qun".into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn setup(rewriter: bool) -> Setup {
    let source = if rewriter {
        format!(
            "{SOURCE}\nskill test_rewrite({TEST_REWRITE}) locked {{ modifier judgment_rewrite add 1; }}"
        )
    } else {
        SOURCE.to_owned()
    };
    let mut package = content_compiler::compile_source_named("beige-test", &source).unwrap();
    package.character_index = (0..3).map(|index| character(9_253_010 + index)).collect();
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_727_01, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target, source) = (seats[0], seats[1], seats[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(BEIGE),
    });
    if rewriter {
        effects.push(EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(TEST_REWRITE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == BEIGE)
        .unwrap()
        .id;
    Setup {
        tx,
        owner,
        target,
        source,
        skill,
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_253_100 + rank as u32),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    inject(tx, player, hand, suit, rank)
}

fn inject_judgment(tx: &mut Transaction, suit: Suit, rank: u8) -> CardId {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(tx, PlayerId::NONE, draw, suit, rank)
}

fn begin(mut setup: Setup, suit: Suit) -> (GameState, CardId, Result<(), EffectError>) {
    let cost = inject_hand(&mut setup.tx, setup.owner, Suit::Club, 1);
    inject_judgment(&mut setup.tx, suit, 7);
    let result = run_effects(
        &mut setup.tx,
        vec![EffectRequest::ResolveBeige {
            player: setup.owner,
            target: setup.target,
            source: setup.source,
            stage: 0,
            result_card: CardId::NONE,
            suit: Suit::None,
            rank: 0,
            rewriters: Vec::new(),
            skill: setup.skill,
        }],
    )
    .map(|_| ());
    (setup.tx.commit().state, cost, result)
}

fn resolve_first(state: GameState, key: u64) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let selection = engine_core::query::discardable_target_cards(
        &state,
        actor,
        actor,
        rule_api::TargetCardScope::HandEquip,
    )[0]
    .raw();
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_contract_is_abi219_while_committed_package_is_abi221() {
    let package = content_compiler::compile_source_named("beige-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 219);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == BEIGE)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty"
    );
    assert!(trigger.optional);
    let module = package
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    assert_eq!(module.code, [83, 84, 85, 80, 219, 1]);
    assert_eq!(
        module.logic_hash,
        [
            7, 31, 230, 177, 202, 60, 16, 99, 81, 198, 147, 57, 182, 167, 33, 230, 169, 90, 120,
            247, 146, 82, 145, 248, 69, 9, 44, 166, 133, 242, 19, 11,
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert_eq!(committed.skill_index.len(), 201);
    assert_eq!(committed.character_index.len(), 48);
    let caiwenji = committed
        .character_index
        .iter()
        .find(|character| character.id == 2_362_677_836)
        .unwrap();
    assert_eq!(caiwenji.skills, [BEIGE, 2_554_836_650]);
}

#[test]
fn final_suit_applies_each_legacy_branch_after_paying_one_card() {
    let mut heart = setup(false);
    heart
        .tx
        .state_mut()
        .players
        .get_mut(heart.target)
        .unwrap()
        .hp = 2;
    let target = heart.target;
    let (state, cost, result) = begin(heart, Suit::Heart);
    result.unwrap();
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);

    let diamond = setup(false);
    let target = diamond.target;
    let hand = diamond.tx.state().hand_count(target);
    let (state, _, result) = begin(diamond, Suit::Diamond);
    result.unwrap();
    assert_eq!(state.hand_count(target), hand + 2);

    let mut club = setup(false);
    let source = club.source;
    inject_hand(&mut club.tx, source, Suit::Heart, 2);
    inject_hand(&mut club.tx, source, Suit::Diamond, 3);
    let (state, _, result) = begin(club, Suit::Club);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, first) = resolve_first(state, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, second) = resolve_first(state, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(source), 0);

    let spade = setup(false);
    let source = spade.source;
    let (state, _, result) = begin(spade, Suit::Spade);
    result.unwrap();
    assert!(state.players.get(source).unwrap().turned_over);
}

#[test]
fn judgment_rewrite_card_becomes_the_final_beige_result() {
    let mut setup = setup(true);
    let source = setup.source;
    let target = setup.target;
    setup.tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let rewrite = inject_hand(&mut setup.tx, source, Suit::Spade, 9);
    let (state, _, result) = begin(setup, Suit::Heart);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, pipeline) = apply_command(
        state.clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![rewrite.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert!(state.players.get(source).unwrap().turned_over);
    assert!(!state.has_pending_choice());
}
