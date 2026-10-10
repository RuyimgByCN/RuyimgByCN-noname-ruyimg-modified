//! Golden coverage for SHENHUA「行殇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINGSHANG: u32 = 2_937_826_132;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/xingshang.skill");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_722, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let victim = seats[1];
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
        skill_def: SkillDefId::new(XINGSHANG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, victim)
}

#[test]
fn package_exposes_exact_optional_death_contract() {
    let committed = package();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINGSHANG && skill.name == "xingshang"));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINGSHANG)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive_event_cards_nonempty");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("xingshang", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn accepting_gains_dead_players_hand_and_equipment_but_not_judgment() {
    let (mut tx, owner, victim) = setup();
    let hand_card = inject(&mut tx, victim, ZoneKind::Hand, CardKind::Sha, 9_820_001);
    let equip_card = inject(
        &mut tx,
        victim,
        ZoneKind::Equip,
        CardKind::Weapon,
        9_820_002,
    );
    let judge_card = inject(
        &mut tx,
        victim,
        ZoneKind::Judge,
        CardKind::TrickBingliang,
        9_820_003,
    );

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let state = tx.state().clone();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "xingshang choice rejected: {:?}",
        result.result
    );
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [hand_card, equip_card] {
        let card = state.cards.get(card).unwrap();
        assert_eq!(card.zone, owner_hand);
        assert_eq!(card.owner, owner);
    }
    let judge = state.cards.get(judge_card).unwrap();
    assert_eq!(judge.zone, discard);
    assert_eq!(judge.owner, PlayerId::NONE);

    let later_owner = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::GainEventCards {
            player: later_owner,
            cards: vec![hand_card, equip_card],
        }],
    )
    .unwrap();
    for card in [hand_card, equip_card] {
        let card = tx.state().cards.get(card).unwrap();
        assert_eq!(card.zone, owner_hand);
        assert_eq!(card.owner, owner);
    }
}

#[test]
fn empty_hand_and_equipment_do_not_offer_the_skill() {
    let (mut tx, owner, victim) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}
