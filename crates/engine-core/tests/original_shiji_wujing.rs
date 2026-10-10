//! Golden for SHIJI 吴懿「合击 / 流冰」语义（ABI 426 / 427）。
//!
//! 合击：其他角色单目标决斗/红杀后，可对原目标视为使用杀或决斗。
//! 流冰：出牌阶段首次实体杀改为方块。

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

const LIUBING: u32 = 2_597_403_716;
const HEJI: u32 = 3_561_932_297;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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

/// 流冰：实体杀改为方块。
#[test]
fn liubing_changes_sha_to_diamond() {
    let (mut tx, owner, seats) = setup(LIUBING, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == LIUBING && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveLiubing {
            player: owner,
            card: sha,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().cards.get(sha).map(|c| c.suit),
        Some(Suit::Diamond),
        "杀花色应改为方块"
    );
}

/// 合击：打开二选一 choice（杀/决斗）。
#[test]
fn heji_offers_sha_or_juedou() {
    let (mut tx, owner, seats) = setup(HEJI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let card = inject_hand(&mut tx, other, CardKind::TrickJuedou, Suit::Club);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == HEJI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveHeji {
            player: owner,
            target: other,
            card,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开二选一 choice"
    );
    // 选"决斗"(option 1)。
    let state = resolve_choice(tx.commit().state, owner, 1, vec![1], false);
    assert!(state.players.get(owner).unwrap().alive);
}
