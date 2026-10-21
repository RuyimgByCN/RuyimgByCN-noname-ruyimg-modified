//! Golden for SHIJI 江庆「上议 / 简议」语义（ABI 423 / 424）。
//!
//! 上议：出牌阶段限一次，展示目标手牌并获得其中一张。
//! 简议：其他角色结束阶段获得弃牌堆中的防具牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPSHANGYI: u32 = 3_102_075_230;
const SPJIANYI: u32 = 2_525_303_245;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_811,
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

fn inject_card(
    tx: &mut Transaction,
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
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
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 上议：展示目标手牌并获一张。
#[test]
fn spshangyi_gains_target_hand_card() {
    let (mut tx, owner, seats) = setup(SPSHANGYI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let target_card = inject_card(&mut tx, other, ZoneKind::Hand, CardKind::Tao, Suit::Heart);
    let hand_before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPSHANGYI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpshangyi {
            player: owner,
            target: other,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开选牌 choice");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![target_card.raw()], false);
    assert_eq!(
        state.hand_count(owner),
        hand_before + 1,
        "应获得目标一张手牌"
    );
    assert!(
        state.hand_cards(owner).contains(&target_card),
        "应获得指定那张牌"
    );
}

/// 简议：弃牌堆有防具时获得它们。
#[test]
fn spjianyi_gains_discard_armors() {
    let (mut tx, owner, seats) = setup(SPJIANYI, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 弃牌堆放两件防具。
    let armor1 = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::Armor,
        Suit::Club,
    );
    let armor2 = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::Armor,
        Suit::Spade,
    );
    let hand_before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPJIANYI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpjianyi {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "应获得全部防具"
    );
    let hand: Vec<_> = tx.state().hand_cards(owner);
    assert!(hand.contains(&armor1) && hand.contains(&armor2));
}
