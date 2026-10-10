//! Golden coverage for 界鬼才's hand/equipment rewrite and ordered rewrite windows.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REGUICAI: u32 = 2_369_807_074;
const REFANKUI: u32 = 3_719_484_801;
const RE_SIMAYI: u32 = 4_084_361_848;
const CLASSIC_GUICAI: u32 = 2_000_000_114;
const RESPONSE_LOCK: u32 = 2_000_000_115;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/reguicai.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(12_000 + kind.tag() as u32 * 30 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: kind.is_delayed_trick(),
    };
    let card_id = tx.state_mut().cards.alloc(card);
    tx.state_mut()
        .zones
        .get_mut(zone)
        .unwrap()
        .cards
        .push(card_id);
    card_id
}

fn setup(source: &str, players: u8) -> Transaction {
    let package = content_compiler::compile_source_named("reguicai-test", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, players, 20_260_718, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for player in tx.state().players.alive_by_seat() {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            });
        }
    }
    run_effects(&mut tx, effects).unwrap();
    tx
}

fn add_skill(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

fn prepare_le_judgment(tx: &mut Transaction, judged: PlayerId) {
    let judge = tx.state().zones.find(ZoneKind::Judge, judged).unwrap();
    inject(tx, judged, judge, CardKind::TrickLe, Suit::Club, 6);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(tx, PlayerId::NONE, draw, CardKind::Other, Suit::Spade, 3);
    tx.state_mut().turn.current = judged;
    tx.state_mut().turn.phase = Phase::Judge;
}

fn end_judge(state: GameState, judged: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: judged,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

fn choose(
    state: GameState,
    result: &PipelineResult,
    actor: PlayerId,
    card: engine_api::CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = result.choice.as_ref().unwrap();
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

fn cancel(
    state: GameState,
    result: &PipelineResult,
    actor: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = result.choice.as_ref().unwrap();
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    )
}

#[test]
fn committed_package_declares_abi24_reguicai_and_re_simayi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 24);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == REGUICAI && modifier.target == "judgment_rewrite_hand_equip"
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_SIMAYI
            && character.max_hp == 3
            && character.skills == vec![REFANKUI, REGUICAI]
    }));
}

#[test]
fn reguicai_can_rewrite_with_an_equipment_card() {
    let mut tx = setup(SOURCE, 2);
    let judged = tx.state().turn.current;
    let rewriter = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != judged)
        .unwrap();
    add_skill(&mut tx, rewriter, REGUICAI);
    let equip = tx.state().zones.find(ZoneKind::Equip, rewriter).unwrap();
    let heart = inject(&mut tx, rewriter, equip, CardKind::Armor, Suit::Heart, 5);
    prepare_le_judgment(&mut tx, judged);

    let (state, first) = end_judge(tx.state().clone(), judged, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let choice = first.choice.as_ref().unwrap();
    assert_eq!(choice.observer, rewriter);
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].token, heart.raw());

    let (state, resolved) = choose(state, &first, rewriter, heart, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(heart).unwrap().zone, discard);
}

#[test]
fn classic_hand_only_rewrite_does_not_offer_equipment() {
    let source = format!(
        "{SOURCE}\nskill classic_guicai({CLASSIC_GUICAI}) {{ modifier judgment_rewrite add 1; }}"
    );
    let mut tx = setup(&source, 2);
    let judged = tx.state().turn.current;
    let rewriter = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != judged)
        .unwrap();
    add_skill(&mut tx, rewriter, CLASSIC_GUICAI);
    let equip = tx.state().zones.find(ZoneKind::Equip, rewriter).unwrap();
    inject(&mut tx, rewriter, equip, CardKind::Armor, Suit::Heart, 5);
    prepare_le_judgment(&mut tx, judged);

    let (state, result) = end_judge(tx.state().clone(), judged, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);
}

#[test]
fn response_lock_removes_all_reguicai_options() {
    let source = format!(
        "{SOURCE}\nskill response_lock({RESPONSE_LOCK}) locked {{ modifier card_response_cancel override 1; }}"
    );
    let mut tx = setup(&source, 2);
    let judged = tx.state().turn.current;
    let rewriter = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != judged)
        .unwrap();
    add_skill(&mut tx, rewriter, REGUICAI);
    add_skill(&mut tx, rewriter, RESPONSE_LOCK);
    let equip = tx.state().zones.find(ZoneKind::Equip, rewriter).unwrap();
    inject(&mut tx, rewriter, equip, CardKind::Armor, Suit::Heart, 5);
    prepare_le_judgment(&mut tx, judged);

    let (state, result) = end_judge(tx.state().clone(), judged, 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);
}

#[test]
fn multiple_rewriters_are_offered_in_seat_order_and_the_last_result_wins() {
    let mut tx = setup(SOURCE, 3);
    let judged = tx.state().turn.current;
    let order = tx.state().players.alive_by_seat();
    let start = order.iter().position(|player| *player == judged).unwrap();
    let second = order[(start + 1) % order.len()];
    add_skill(&mut tx, judged, REGUICAI);
    add_skill(&mut tx, second, REGUICAI);
    let judged_hand = tx.state().zones.find(ZoneKind::Hand, judged).unwrap();
    let second_hand = tx.state().zones.find(ZoneKind::Hand, second).unwrap();
    let first_card = inject(
        &mut tx,
        judged,
        judged_hand,
        CardKind::Other,
        Suit::Diamond,
        7,
    );
    let second_card = inject(
        &mut tx,
        second,
        second_hand,
        CardKind::Other,
        Suit::Heart,
        8,
    );
    prepare_le_judgment(&mut tx, judged);

    let (state, first) = end_judge(tx.state().clone(), judged, 30);
    assert_eq!(first.choice.as_ref().unwrap().observer, judged);
    let (state, second_choice) = choose(state, &first, judged, first_card, 31);
    assert!(matches!(
        second_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(second_choice.choice.as_ref().unwrap().observer, second);
    let (state, resolved) = choose(state, &second_choice, second, second_card, 32);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
}

#[test]
fn declining_one_rewrite_still_offers_the_next_player() {
    let mut tx = setup(SOURCE, 3);
    let judged = tx.state().turn.current;
    let order = tx.state().players.alive_by_seat();
    let start = order.iter().position(|player| *player == judged).unwrap();
    let second = order[(start + 1) % order.len()];
    add_skill(&mut tx, judged, REGUICAI);
    add_skill(&mut tx, second, REGUICAI);
    let judged_hand = tx.state().zones.find(ZoneKind::Hand, judged).unwrap();
    let second_hand = tx.state().zones.find(ZoneKind::Hand, second).unwrap();
    inject(
        &mut tx,
        judged,
        judged_hand,
        CardKind::Other,
        Suit::Diamond,
        7,
    );
    let heart = inject(
        &mut tx,
        second,
        second_hand,
        CardKind::Other,
        Suit::Heart,
        8,
    );
    prepare_le_judgment(&mut tx, judged);

    let (state, first) = end_judge(tx.state().clone(), judged, 40);
    assert_eq!(first.choice.as_ref().unwrap().observer, judged);
    let (state, second_choice) = cancel(state, &first, judged, 41);
    assert_eq!(second_choice.choice.as_ref().unwrap().observer, second);
    let (state, resolved) = choose(state, &second_choice, second, heart, 42);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
}
