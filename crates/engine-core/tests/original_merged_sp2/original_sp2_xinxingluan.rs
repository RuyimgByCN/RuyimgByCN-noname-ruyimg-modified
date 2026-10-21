//! Golden coverage for 心星乱（sp2 樊稠，ABI 476）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINXINGLUAN: u32 = 3_806_437_757;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp2/xinxingluan.skill");

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    rank: u8,
    zone: ZoneId,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "play must be accepted: {:?}",
        result.result
    );
    next
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn setup(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINXINGLUAN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty the target's hand so the probe Sha cannot be dodged.
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
    run_effects(
        tx,
        cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(hand),
            })
            .collect(),
    )
    .unwrap();
}

#[test]
fn gain_branch_takes_rank_six_equipment() {
    let package = content_compiler::compile_source_named("xxl-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 476);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // A rank-6 weapon in the target's equip zone.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    let weapon = alloc_card(&mut tx, target, CardKind::Weapon, 96_100, 6, equip_zone);
    let owner_hand_before = tx.state().hand_count(owner);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, 5, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(
        state.resolution.pending_choice_id != 0,
        "branch choice pending"
    );
    let state = resolve_choice(state, owner, vec![2], false, 2); // gain branch
    assert!(state.has_pending_choice(), "gain target choice pending");
    let state = resolve_choice(state, owner, vec![target.raw()], false, 3);
    assert!(state.has_pending_choice(), "rank-6 card choice pending");
    let state = resolve_choice(state, owner, vec![weapon.raw()], false, 4);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained the rank-6 equipment"
    );
    let equip_len = state
        .zones
        .find(ZoneKind::Equip, target)
        .and_then(|id| state.zones.get(id))
        .map(|z| z.cards.len())
        .unwrap();
    assert_eq!(equip_len, 0, "target's equip zone emptied");
}

#[test]
fn discard_branch_target_discards_rank_six_hand() {
    let package = content_compiler::compile_source_named("xxl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let six = alloc_card(&mut tx, target, CardKind::Sha, 96_101, 6, target_hand);
    alloc_card(&mut tx, target, CardKind::Tao, 96_102, 5, target_hand);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, 5, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(state.resolution.pending_choice_id != 0);
    let state = resolve_choice(state, owner, vec![3], false, 2); // discard branch
    assert!(state.has_pending_choice(), "discard target choice pending");
    let state = resolve_choice(state, owner, vec![target.raw()], false, 3);
    assert!(state.has_pending_choice(), "target's rank-6 discard choice");
    let state = resolve_choice(state, target, vec![six.raw()], false, 4);

    assert_eq!(state.hand_count(target), 1, "target kept the rank-5 Tao");
}

#[test]
fn pile_branch_gains_rank_six_pile_card() {
    let package = content_compiler::compile_source_named("xxl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let six = alloc_card(&mut tx, PlayerId::NONE, CardKind::Sha, 96_200, 6, draw);
    let owner_hand_before = tx.state().hand_count(owner);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, 5, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(state.resolution.pending_choice_id != 0);
    let state = resolve_choice(state, owner, vec![1], false, 2); // pile branch
    assert!(state.has_pending_choice(), "pile card choice pending");
    let state = resolve_choice(state, owner, vec![six.raw()], false, 3);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained a rank-6 pile card"
    );
}

#[test]
fn once_per_turn() {
    let package = content_compiler::compile_source_named("xxl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_804, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    alloc_card(&mut tx, target, CardKind::Sha, 96_101, 6, target_hand);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha1 = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, 5, hand);
    let sha2 = alloc_card(&mut tx, owner, CardKind::Sha, 96_001, 5, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    // First Sha use: cancel the branch (still consumes the once-per-turn).
    let mut state = play(tx.commit().state, owner, sha1, vec![target], 1);
    assert!(state.resolution.pending_choice_id != 0);
    state = resolve_choice(state, owner, vec![], true, 2);
    // Second Sha use: no choice (already fired this turn).
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let mut tx = Transaction::begin(state);
    let state = play(tx.commit().state, owner, sha2, vec![target], 3);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "second card use offers no branch"
    );
}
