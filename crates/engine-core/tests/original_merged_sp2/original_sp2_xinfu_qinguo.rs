//! Golden coverage for 勤国（sp2 吕岱，ABI 474）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_QINGUO: u32 = 2_648_676_099;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp2/xinfu_qinguo.skill");

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    zone: ZoneId,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 5,
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

fn drive_choice(
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

fn setup(tx: &mut Transaction, owner: PlayerId, others: &[PlayerId]) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_QINGUO),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &other in others {
        let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
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
}

#[test]
fn equipment_use_offers_virtual_sha() {
    let package = content_compiler::compile_source_named("qinguo-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 474);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    setup(&mut tx, owner, &[victim]);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let weapon = alloc_card(&mut tx, owner, CardKind::Weapon, 96_000, hand);
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    // Equip the weapon: 勤国 offers a virtual Sha.
    let state = play(tx.commit().state, owner, weapon, vec![], 1);
    assert!(
        state.resolution.pending_choice_id != 0,
        "virtual Sha choice pending"
    );
    let state = drive_choice(state, owner, vec![victim.raw()], false, 2);
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp_before - 1,
        "virtual Sha hit the victim"
    );
}

#[test]
fn non_equipment_use_offers_nothing() {
    let package = content_compiler::compile_source_named("qinguo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    setup(&mut tx, owner, &[victim]);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = alloc_card(&mut tx, owner, CardKind::Sha, 96_000, hand);

    let mut tx = Transaction::begin(tx.commit().state);
    let state = play(tx.commit().state, owner, sha, vec![victim], 1);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no qinguo choice for a Sha use"
    );
}

#[test]
fn recover_when_equip_count_equals_hp_after_equip_loss() {
    let package = content_compiler::compile_source_named("qinguo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, _other) = (seats[0], seats[1]);
    setup(&mut tx, owner, &[]);
    // Wound the owner to 3 HP; seed 4 equipment cards into the equip zone.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    for (kind, def) in [
        (CardKind::Weapon, 96_000),
        (CardKind::Armor, 96_001),
        (CardKind::HorseOff, 96_002),
        (CardKind::HorseDef, 96_003),
    ] {
        alloc_card(&mut tx, owner, kind, def, equip_zone);
    }
    let hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    // Move one equipment out: equip count 4 -> 3 == hp 3, wounded -> recover 1.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let moving = *tx
        .state()
        .zones
        .get(equip_zone)
        .unwrap()
        .cards
        .first()
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![moving],
            to: discard,
            from: Some(equip_zone),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before + 1,
        "recovered 1 HP when equip count matched HP after the loss"
    );
}
