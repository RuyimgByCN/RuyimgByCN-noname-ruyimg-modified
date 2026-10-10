//! M1 谦逊: 陆逊 cannot be targeted by 顺手牵羊 / 乐不思蜀 (but 兵粮寸断 still works).

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content::skill_def, Engine};
use rule_api::EffectRequest;

fn start(seed: u64, n: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    hand: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(8400 + kind.tag() as u32 * 10 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    };
    let cid = tx.state_mut().cards.alloc(card);
    if let Some(z) = tx.state_mut().zones.get_mut(hand) {
        z.cards.push(cid);
    }
    if let Some(c) = tx.state_mut().cards.get_mut(cid) {
        c.zone = hand;
        c.owner = owner;
    }
    cid
}

/// Setup: actor holds `kind`; `other` (陆逊) has 谦逊 and a hand card. Returns rejection or accept.
fn play_trick_on_qianxun(
    seed: u64,
    kind: CardKind,
) -> (bool, engine_core::command::PipelineResult) {
    let mut eng = Engine::new();
    eng.submit(start(seed, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: other,
            skill_def: engine_api::SkillDefId::new(skill_def::QIANXUN),
        }],
    )
    .unwrap();
    // Ensure 陆逊 has a hand card (for 顺手 to have something to take).
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    if tx.state().hand_count(other) == 0 {
        inject(&mut tx, other, other_hand, CardKind::Shan, Suit::Club, 2);
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let card = inject(&mut tx, actor, hand, kind, Suit::Spade, 6);

    let (_st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    (matches!(pipe.result, CommandResult::Rejected { .. }), pipe)
}

#[test]
fn qianxun_blocks_shunshou() {
    let (rejected, pipe) = play_trick_on_qianxun(7, CardKind::TrickShunshou);
    assert!(rejected, "谦逊 should block 顺手牵羊: {:?}", pipe.result);
}

#[test]
fn qianxun_blocks_le() {
    let (rejected, pipe) = play_trick_on_qianxun(7, CardKind::TrickLe);
    assert!(rejected, "谦逊 should block 乐不思蜀: {:?}", pipe.result);
}

#[test]
fn qianxun_does_not_block_bingliang() {
    // 兵粮寸断 is NOT blocked by 谦逊.
    let (rejected, pipe) = play_trick_on_qianxun(7, CardKind::TrickBingliang);
    assert!(!rejected, "谦逊 must not block 兵粮寸断: {:?}", pipe.result);
}
