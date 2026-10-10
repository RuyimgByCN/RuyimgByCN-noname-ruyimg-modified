//! Golden for SHIJI 朱儁「佯解」语义（ABI 430）。
//!
//! 佯解：出牌阶段限一次与一名其他角色拼点（点数小者胜）；拼点失败时可选
//! 另一名其他角色视为对拼点目标使用一张火【杀】。

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

const XINYANGJIE: u32 = 3_719_893_345;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(XINYANGJIE as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_813,
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
        skill_def: SkillDefId::new(XINYANGJIE),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, rank: u8) -> CardId {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + CardKind::Sha.tag() as u32),
        kind: CardKind::Sha,
        suit: Suit::Spade,
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

fn skill_instance(tx: &Transaction, owner: PlayerId) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == XINYANGJIE && s.owner == owner)
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
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
}

/// 佯解：点数小者胜——拥有者 3 点对目标 5 点，拥有者赢，直接结算无后续选择。
#[test]
fn xinyangjie_smaller_rank_wins_no_retaliation() {
    let (mut tx, owner, seats) = setup(2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    into_play_phase(&mut tx, owner);
    let owner_card = inject_hand(&mut tx, owner, 3);
    let target_card = inject_hand(&mut tx, target, 5);
    let skill = skill_instance(&tx, owner);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinyangjie {
            player: owner,
            target,
            card: CardId::NONE,
            target_card: CardId::NONE,
            source: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开拼点牌选择");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "应推进到目标拼点牌");
    let state = resolve_choice(state, target, 2, vec![target_card.raw()], false);
    assert_eq!(state.resolution.pending_choice_id, 0, "小者胜无火杀选择");
    // 两张拼点牌进入弃牌堆。
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

/// 佯解：拼点失败（拥有者 9 点对目标 3 点）→ 选一名其他角色对目标使用火【杀】。
#[test]
fn xinyangjie_failure_chooses_fire_sha_source() {
    let (mut tx, owner, seats) = setup(3);
    let target = seats[1];
    let source = seats[2];
    into_play_phase(&mut tx, owner);
    let owner_card = inject_hand(&mut tx, owner, 9);
    let target_card = inject_hand(&mut tx, target, 3);
    let skill = skill_instance(&tx, owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinyangjie {
            player: owner,
            target,
            card: CardId::NONE,
            target_card: CardId::NONE,
            source: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .unwrap_err();
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    let state = resolve_choice(state, target, 2, vec![target_card.raw()], false);
    assert_ne!(
        state.resolution.pending_choice_id, 0,
        "失败应打开火杀来源选择"
    );
    // 来源角色对目标用虚拟火杀：驱动目标响应（放弃出闪）后结算完成。
    let mut state = resolve_choice(state, owner, 3, vec![source.raw()], false);
    let mut key = 4u64;
    while state.resolution.pending_choice_id != 0 {
        let observer = PlayerId::new(state.resolution.pending_observer);
        let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
        let (next, result) = apply_command(
            state.clone(),
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(state.revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        state = next;
        key += 1;
    }
    let _ = source;
}

/// 佯解：拼点失败但取消火杀来源选择 → 直接结束。
#[test]
fn xinyangjie_failure_cancels_retaliation() {
    let (mut tx, owner, seats) = setup(3);
    let target = seats[1];
    into_play_phase(&mut tx, owner);
    let owner_card = inject_hand(&mut tx, owner, 9);
    let target_card = inject_hand(&mut tx, target, 3);
    let skill = skill_instance(&tx, owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinyangjie {
            player: owner,
            target,
            card: CardId::NONE,
            target_card: CardId::NONE,
            source: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .unwrap_err();
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    let state = resolve_choice(state, target, 2, vec![target_card.raw()], false);
    let state = resolve_choice(state, owner, 3, vec![], true);
    assert_eq!(state.resolution.pending_choice_id, 0, "取消后直接结束");
}
