//! Golden for legacy CardUsed target/source draw skills.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const TSUMUGI_MUGYU: u32 = 3_191_570_097;
const YUI_JIANG: u32 = 2_788_570_580;
const JIANG: u32 = 2_402_221_407;
const NONAME_JIANG: u32 = 4_012_037_429;
const NSSHUANGXIONG: u32 = 2_353_402_107;
const AO_XISHI: u32 = 4_100_198_765;
const SPDENGLI: u32 = 3_419_124_885;
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SHIJI: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");
const DIY: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn setup(package: &[u8], skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_714 + skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch_card_used(
    tx: &mut Transaction,
    user: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn run_dispatched(tx: &mut Transaction, events: &[DomainEvent]) {
    let effects = dispatch_for_events(tx, events, &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

fn confirm(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let revision = tx.state().revision;
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn tsumugi_mugyu_requires_owner_target_and_hand_below_max_hp() {
    let (mut tx, owner, seats) = setup(KEY, TSUMUGI_MUGYU, 2);
    let source = seats.into_iter().find(|player| *player != owner).unwrap();
    let max_hp = tx.state().hand_count(owner).max(1);
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = max_hp;
    let discard_count = tx
        .state()
        .hand_count(owner)
        .saturating_sub(max_hp.saturating_sub(1));
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: discard_count,
        }],
    )
    .unwrap();
    let card = inject(&mut tx, source, CardKind::Other, Suit::Spade);
    let before = tx.state().hand_count(owner);
    assert!(before < max_hp);

    assert!(dispatch_card_used(&mut tx, source, card, vec![source])
        .unwrap()
        .is_empty());
    assert!(!tx.state().has_pending_choice());

    assert_eq!(
        dispatch_card_used(&mut tx, source, card, vec![owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(tx, owner, 1);
    assert_eq!(state.hand_count(owner), before + 1);
    assert_eq!(state.hand_count(owner), max_hp);

    let mut full = Transaction::begin(state);
    assert!(dispatch_card_used(&mut full, source, card, vec![owner])
        .unwrap()
        .is_empty());
    assert!(!full.state().has_pending_choice());
}

fn assert_jiang_semantics(package: &[u8], skill_def: u32) {
    let (mut negative, owner, seats) = setup(package, skill_def, 3);
    let source = seats
        .iter()
        .copied()
        .find(|player| *player != owner)
        .unwrap();
    let black_sha = inject(&mut negative, source, CardKind::Sha, Suit::Spade);
    let red_other = inject(&mut negative, source, CardKind::Other, Suit::Heart);
    assert!(
        dispatch_card_used(&mut negative, source, black_sha, vec![owner])
            .unwrap()
            .is_empty()
    );
    assert!(
        dispatch_card_used(&mut negative, source, red_other, vec![owner])
            .unwrap()
            .is_empty()
    );

    let (mut targeted, owner, seats) = setup(package, skill_def, 3);
    let source = seats
        .iter()
        .copied()
        .find(|player| *player != owner)
        .unwrap();
    let red_sha = inject(&mut targeted, source, CardKind::Sha, Suit::Diamond);
    let before = targeted.state().hand_count(owner);
    assert_eq!(
        dispatch_card_used(&mut targeted, source, red_sha, vec![owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(targeted, owner, 2);
    assert_eq!(state.hand_count(owner), before + 1);

    let (mut used, owner, seats) = setup(package, skill_def, 3);
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let juedou = inject(&mut used, owner, CardKind::TrickJuedou, Suit::Club);
    let before = used.state().hand_count(owner);
    assert_eq!(
        dispatch_card_used(&mut used, owner, juedou, targets).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(used, owner, 3);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn inherited_jiang_skills_match_juedou_and_red_sha_once_per_card() {
    assert_jiang_semantics(KEY, YUI_JIANG);
    assert_jiang_semantics(SHENHUA, JIANG);
    assert_jiang_semantics(DIY, NONAME_JIANG);
}

#[test]
fn nsshuangxiong_only_offers_for_owner_participating_in_juedou() {
    let (mut targeted, owner, seats) = setup(DIY, NSSHUANGXIONG, 3);
    let source = seats.into_iter().find(|player| *player != owner).unwrap();
    let red_sha = inject(&mut targeted, source, CardKind::Sha, Suit::Heart);
    assert!(
        dispatch_card_used(&mut targeted, source, red_sha, vec![owner])
            .unwrap()
            .is_empty()
    );

    let juedou = inject(&mut targeted, source, CardKind::TrickJuedou, Suit::Club);
    assert_eq!(
        dispatch_card_used(&mut targeted, source, juedou, vec![owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(targeted, owner, 6);
    assert!(state.players.get(owner).unwrap().turned_over);

    let (mut used, owner, seats) = setup(DIY, NSSHUANGXIONG, 3);
    let target = seats.into_iter().find(|player| *player != owner).unwrap();
    let juedou = inject(&mut used, owner, CardKind::TrickJuedou, Suit::Spade);
    assert_eq!(
        dispatch_card_used(&mut used, owner, juedou, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(used, owner, 7);
    assert!(state.players.get(owner).unwrap().turned_over);
}

#[test]
fn spdengli_fires_once_for_each_equal_hp_sha_target() {
    let (mut tx, owner, seats) = setup(SHIJI, SPDENGLI, 3);
    for player in &seats {
        tx.state_mut().players.get_mut(*player).unwrap().hp = 3;
    }
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_card_used(&mut tx, owner, sha, targets).unwrap_err(),
        EffectError::ChoiceRequested
    );

    let state = confirm(tx, owner, 4);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(state.has_pending_choice());
    let state = confirm(Transaction::begin(state), owner, 5);
    assert_eq!(state.hand_count(owner), before + 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn ao_xishi_covers_diamond_use_target_and_response_once_each() {
    let (mut tx, owner, seats) = setup(KEY, AO_XISHI, 3);
    let source = seats
        .iter()
        .copied()
        .find(|player| *player != owner)
        .unwrap();
    let others: Vec<_> = seats
        .iter()
        .copied()
        .filter(|player| *player != owner)
        .collect();

    let used = inject(&mut tx, owner, CardKind::Other, Suit::Diamond);
    let before = tx.state().hand_count(owner);
    run_dispatched(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: used,
            targets: others,
        }],
    );
    assert_eq!(tx.state().hand_count(owner), before + 1);

    let targeted = inject(&mut tx, source, CardKind::Other, Suit::Diamond);
    let before = tx.state().hand_count(owner);
    run_dispatched(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card: targeted,
            targets: vec![owner],
        }],
    );
    assert_eq!(tx.state().hand_count(owner), before + 1);

    let response = inject(&mut tx, owner, CardKind::Shan, Suit::Diamond);
    let before = tx.state().hand_count(owner);
    run_dispatched(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: response,
            response_kind: CardKind::Shan.tag(),
        }],
    );
    assert_eq!(tx.state().hand_count(owner), before + 1);

    let club = inject(&mut tx, owner, CardKind::Shan, Suit::Club);
    let before = tx.state().hand_count(owner);
    run_dispatched(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: club,
            response_kind: CardKind::Shan.tag(),
        }],
    );
    assert_eq!(tx.state().hand_count(owner), before);
}
