//! Golden for SHIJI 崔琰「压军 / 尊帝」语义（ABI 428 / 429）。
//!
//! 压军：出牌阶段开始，手牌中有本回合获得的牌时，选一张新获牌与一名其他角色
//! 拼点；赢则可将拼点牌置于牌堆顶，输则添加“戒”标记（手牌上限减标记数）。
//! 尊帝：出牌阶段限一次，弃置一张手牌交给一名其他角色，然后判定：黑色其摸三张，
//! 红色其移动一张场上牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPYAJUN: u32 = 3_556_581_945;
const SPYAJUN_LESS: u32 = 3_131_511_516;
const SPZUNDI: u32 = 4_234_395_954;
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
                rng_seed: 20_260_812,
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

#[allow(clippy::too_many_arguments)]
fn inject_card(
    tx: &mut Transaction,
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank,
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

fn skill_instance(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == def && s.owner == owner)
        .map(|s| s.id)
        .unwrap()
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

fn into_play_phase(tx: &mut Transaction, owner: PlayerId) {
    // 主公回合开始的 phaseDraw 摸牌也计入“本回合获得”；先清空以获得干净触发环境。
    tx.state_mut().turn.gained_hand_cards.clear();
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
}

/// 压军：赢家可把拼点牌置于牌堆顶（选择链自动推进：新获牌 → 目标 →
/// 目标拼点牌 → 置顶）。
#[test]
fn spyajun_win_places_card_on_top() {
    let (mut tx, owner, seats) = setup(SPYAJUN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    into_play_phase(&mut tx, owner);
    // 拥有者一张本回合新获的 3 点牌，目标一张 2 点牌。
    let owner_card = inject_card(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        3,
    );
    tx.state_mut()
        .turn
        .gained_hand_cards
        .insert((owner, owner_card));
    let target_card = inject_card(
        &mut tx,
        other,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        2,
    );
    let skill = skill_instance(&tx, owner, SPYAJUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyajun {
            player: owner,
            less_def: SkillDefId::new(SPYAJUN_LESS),
            card: CardId::NONE,
            target: PlayerId::NONE,
            target_card: CardId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开新获牌选择");

    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "应推进到目标选择");
    let state = resolve_choice(state, owner, 2, vec![other.raw()], false);
    assert_ne!(
        state.resolution.pending_choice_id, 0,
        "应推进到目标拼点牌选择"
    );
    let state = resolve_choice(state, other, 3, vec![target_card.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "赢家应可选置顶");
    let state = resolve_choice(state, owner, 4, vec![owner_card.raw()], false);

    assert_eq!(state.resolution.pending_choice_id, 0, "结算完成");
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last().copied(),
        Some(owner_card),
        "拼点牌应置入牌堆顶"
    );
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .all(|(m, _)| *m != SPYAJUN_LESS),
        "赢家不应有戒标记"
    );
}

/// 压军：败方获得戒标记（手牌上限 - 标记数），且没有置顶选择。
#[test]
fn spyajun_loss_adds_less_mark() {
    let (mut tx, owner, seats) = setup(SPYAJUN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    into_play_phase(&mut tx, owner);
    let owner_card = inject_card(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        3,
    );
    tx.state_mut()
        .turn
        .gained_hand_cards
        .insert((owner, owner_card));
    let target_card = inject_card(
        &mut tx,
        other,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        10,
    );
    let skill = skill_instance(&tx, owner, SPYAJUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyajun {
            player: owner,
            less_def: SkillDefId::new(SPYAJUN_LESS),
            card: CardId::NONE,
            target: PlayerId::NONE,
            target_card: CardId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    let state = resolve_choice(state, owner, 2, vec![other.raw()], false);
    // 目标选 10 点牌：目标胜 → 败分支直接结算，无置顶选择。
    let state = resolve_choice(state, other, 3, vec![target_card.raw()], false);

    assert_eq!(state.resolution.pending_choice_id, 0, "败分支不应打开选择");
    assert_eq!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(m, _)| *m == SPYAJUN_LESS)
            .map(|(_, n)| *n),
        Some(1),
        "败方应获得一枚戒标记"
    );
    // 两拼点牌均进弃牌堆。
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let in_discard = |card: CardId| {
        state
            .zones
            .get(discard)
            .is_some_and(|zone| zone.cards.contains(&card))
    };
    assert!(in_discard(owner_card) && in_discard(target_card));
}

/// 尊帝：黑色判定 → 目标摸三张。
#[test]
fn spzundi_black_draws_three() {
    let (mut tx, owner, seats) = setup(SPZUNDI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    // 牌堆顶放一张黑桃牌（黑）。
    inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Spade,
        7,
    );
    let skill = skill_instance(&tx, owner, SPZUNDI);
    let hand_before = tx.state().hand_count(target);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpzundi {
            player: owner,
            target,
            card: CardId::NONE,
            destination: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(target),
        hand_before + 3,
        "黑色判定目标应摸三张"
    );
    assert_eq!(tx.state().resolution.pending_choice_id, 0, "黑色分支无选择");
}

/// 尊帝：红色判定 → 目标移动一张场上牌（目标选牌与落点，选择链自动推进）。
#[test]
fn spzundi_red_moves_field_card() {
    let (mut tx, owner, seats) = setup(SPZUNDI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    // 牌堆顶放一张红桃牌（红）。
    inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Tao,
        Suit::Heart,
        7,
    );
    // 目标持一件武器（可移到拥有者的空武器栏）。
    let weapon = inject_card(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        7,
    );
    let skill = skill_instance(&tx, owner, SPZUNDI);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpzundi {
            player: owner,
            target,
            card: CardId::NONE,
            destination: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "红色分支应开选牌");
    let state = resolve_choice(tx.commit().state, target, 1, vec![weapon.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "应推进到落点选择");
    let state = resolve_choice(state, target, 2, vec![owner.raw()], false);

    assert_eq!(state.resolution.pending_choice_id, 0, "结算完成");
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert!(
        state
            .zones
            .get(owner_equip)
            .is_some_and(|zone| zone.cards.contains(&weapon)),
        "武器应移入拥有者装备区"
    );
}
