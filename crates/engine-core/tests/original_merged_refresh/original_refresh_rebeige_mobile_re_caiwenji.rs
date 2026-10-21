//! Golden coverage for REFRESH Rebeige and MOBILE Re Caiwenji.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REBEIGE: u32 = 3_378_605_169;
const DUANCHANG: u32 = 2_554_836_650;
const RE_CAIWENJI: u32 = 2_656_544_451;
const TEST_REWRITE: u32 = 9_296_001;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/rebeige.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");
const MOBILE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");
const SHENHUA: &[u8] =
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
        name_key: format!("rebeige_test_{id}"),
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
    let mut package = content_compiler::compile_source_named("rebeige-test", &source).unwrap();
    package.character_index = (0..3).map(|index| character(9_296_010 + index)).collect();
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_730_01, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(REBEIGE),
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
        .find(|skill| skill.owner == owner && skill.def.raw() == REBEIGE)
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
        def: CardDefId::new(9_296_100 + rank as u32),
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

fn begin(
    mut setup: Setup,
    suit: Suit,
) -> (GameState, CardId, Result<Vec<DomainEvent>, EffectError>) {
    let cost = inject_hand(&mut setup.tx, setup.owner, Suit::Club, 1);
    inject_judgment(&mut setup.tx, suit, 7);
    let result = run_effects(
        &mut setup.tx,
        vec![EffectRequest::ResolveBeigeConfigured {
            player: setup.owner,
            target: setup.target,
            source: setup.source,
            stage: 0,
            result_card: CardId::NONE,
            suit: Suit::None,
            rank: 0,
            rewriters: Vec::new(),
            heart_recover_amount: 2,
            diamond_draw_count: 3,
            skill: setup.skill,
        }],
    );
    (setup.tx.commit().state, cost, result)
}

fn resolve_rewrite(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                cancelled,
            },
        },
    )
}

#[test]
fn compiled_contract_and_mobile_cross_package_character_are_closed() {
    let package = content_compiler::compile_source_named("rebeige-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 296);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REBEIGE)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty"
    );
    assert!(trigger.optional);

    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 298);
    assert!(refresh
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REBEIGE));
    let caiwenji = mobile
        .character_index
        .iter()
        .find(|character| character.id == RE_CAIWENJI)
        .unwrap();
    assert_eq!(caiwenji.name_key, "re_caiwenji");
    assert_eq!(caiwenji.sex, "female");
    assert_eq!(caiwenji.group, "qun");
    assert_eq!(caiwenji.max_hp, 3);
    assert_eq!(caiwenji.skills, [REBEIGE, DUANCHANG]);
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
    assert!(!mobile
        .skill_index
        .iter()
        .any(|skill| matches!(skill.skill_def_id, REBEIGE | DUANCHANG)));
    assert!(caiwenji.skills.iter().all(|wanted| refresh
        .skill_index
        .iter()
        .chain(shenhua.skill_index.iter())
        .any(|skill| skill.skill_def_id == *wanted)));
}

#[test]
fn configured_heart_uses_event_amount_and_diamond_draws_three_after_judgment() {
    let mut heart = setup(false);
    heart
        .tx
        .state_mut()
        .players
        .get_mut(heart.target)
        .unwrap()
        .hp = 1;
    let target = heart.target;
    let (state, _, events) = begin(heart, Suit::Heart);
    let events = events.unwrap();
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    let judgment = events
        .iter()
        .position(|event| matches!(event, DomainEvent::JudgmentResolved { player, .. } if *player == target))
        .unwrap();
    let recovered = events
        .iter()
        .position(|event| matches!(event, DomainEvent::HpRecovered { player, amount: 2, .. } if *player == target))
        .unwrap();
    assert!(judgment < recovered);

    let diamond = setup(false);
    let target = diamond.target;
    let hand = diamond.tx.state().hand_count(target);
    let (state, _, events) = begin(diamond, Suit::Diamond);
    let events = events.unwrap();
    assert_eq!(state.hand_count(target), hand + 3);
    let judgment = events
        .iter()
        .position(|event| matches!(event, DomainEvent::JudgmentResolved { player, .. } if *player == target))
        .unwrap();
    let drawn = events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsDrawn { player, cards } if *player == target && cards.len() == 3))
        .unwrap();
    assert!(judgment < drawn);
}

#[test]
fn configured_rewrite_payload_suffix_survives_replace_and_cancel_paths() {
    let mut replaced = setup(true);
    let source = replaced.source;
    let target = replaced.target;
    replaced.tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    let rewrite = inject_hand(&mut replaced.tx, source, Suit::Diamond, 9);
    let (state, _, result) = begin(replaced, Suit::Heart);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RewriteBeigeJudgment
    );
    let payload = &state.resolution.continuation_payload;
    let rewriter_end = 8 + payload[7] as usize;
    assert_eq!(&payload[rewriter_end..], [1, 2, 3]);
    let (state, pipeline) = resolve_rewrite(state, source, vec![rewrite.raw()], false, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 1);
    assert_eq!(state.hand_count(target), 3);
    let judgment = pipeline
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::JudgmentResolved { player, result_card, .. } if *player == target && *result_card == rewrite))
        .unwrap();
    let drawn = pipeline
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsDrawn { player, cards } if *player == target && cards.len() == 3))
        .unwrap();
    assert!(judgment < drawn);

    let mut cancelled = setup(true);
    let source = cancelled.source;
    let target = cancelled.target;
    cancelled.tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    inject_hand(&mut cancelled.tx, source, Suit::Spade, 10);
    let (state, _, result) = begin(cancelled, Suit::Heart);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let payload = &state.resolution.continuation_payload;
    let rewriter_end = 8 + payload[7] as usize;
    assert_eq!(&payload[rewriter_end..], [1, 2, 3]);
    let (state, pipeline) = resolve_rewrite(state, source, Vec::new(), true, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    let judgment = pipeline
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::JudgmentResolved { player, .. } if *player == target))
        .unwrap();
    let recovered = pipeline
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::HpRecovered { player, amount: 2, .. } if *player == target))
        .unwrap();
    assert!(judgment < recovered);
}
