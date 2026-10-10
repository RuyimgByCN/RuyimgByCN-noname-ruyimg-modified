//! Standard 裸衣: draw one fewer card to add +1 Sha/Duel damage for the turn.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage};
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
        def: CardDefId::new(8300 + kind.tag() as u32 * 10 + rank as u32),
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

/// Clear a player's hand so they cannot respond with 闪.
fn clear_hand(tx: &mut Transaction, p: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, p).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().hand_cards(p);
    if !cards.is_empty() {
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            }],
        )
        .unwrap();
    }
}

fn clear_skills(tx: &mut Transaction) {
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
}

#[test]
fn luoyi_sha_deals_two_damage() {
    let mut eng = Engine::new();
    eng.submit(start(7, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    // Grant 裸衣 and mark it active (as if 许褚 forwent his draw this turn).
    // Give a max_hp buffer so the target survives 2 damage and stays out of dying.
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::DRAW_REPLACED, 1);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: actor,
                skill_def: engine_api::SkillDefId::new(skill_def::LUOYI),
            },
            EffectRequest::ChangeMaxHp {
                player: other,
                delta: 3,
            },
            EffectRequest::RecoverHp {
                source: other,
                player: other,
                amount: 3,
            },
        ],
    )
    .unwrap();
    clear_hand(&mut tx, other);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 7);
    let hp_before = tx.state().players.get(other).unwrap().hp;

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(
        !st.has_pending_choice(),
        "target has no 闪, damage applies directly"
    );
    assert_eq!(
        st.players.get(other).unwrap().hp,
        hp_before - 2,
        "裸衣 杀 should deal 2 damage"
    );
}

#[test]
fn without_luoyi_sha_deals_one() {
    // Control: same setup without 裸衣 → 1 damage.
    let mut eng = Engine::new();
    eng.submit(start(7, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ChangeMaxHp {
                player: other,
                delta: 3,
            },
            EffectRequest::RecoverHp {
                source: other,
                player: other,
                amount: 3,
            },
        ],
    )
    .unwrap();
    clear_hand(&mut tx, other);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 7);
    let hp_before = tx.state().players.get(other).unwrap().hp;

    let (st, _pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert_eq!(st.players.get(other).unwrap().hp, hp_before - 1);
}

#[test]
fn luoyi2_rewrites_the_shared_sha_and_duel_bonus_without_touching_other_damage() {
    let mut eng = Engine::new();
    eng.submit(start(29, 2));
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    for player in [actor, other] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(2_999_929_189),
        }],
    )
    .unwrap();

    // `sha_damage` is the shared outgoing damage channel used by both Sha and Duel.
    assert_eq!(engine_core::query::sha_damage_for(tx.state(), actor, 1), 2);
    let before = tx.state().players.get(other).unwrap().hp;
    schedule_damage(&mut tx, actor, other, 1).unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, before - 1);
}

/// Drive 许褚 through the Judge→Draw transition and return the opened choice.
fn open_draw_choice(
    seed: u64,
) -> (
    engine_core::state::GameState,
    PlayerId,
    engine_core::command::PipelineResult,
) {
    let mut eng = Engine::new();
    eng.submit(start(seed, 2));
    let actor = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Judge;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::LUOYI),
        }],
    )
    .unwrap();
    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    (st, actor, pipe)
}

#[test]
fn luoyi_draws_one_fewer_and_activates_bonus() {
    let (st, actor, pipe) = open_draw_choice(7);
    assert!(
        st.has_pending_choice(),
        "裸衣 draw-phase choice should open"
    );
    let hand_before = st.hand_count(actor);
    let choice = pipe.choice.expect("choice");
    // Draw one fewer card (select token 1).
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    assert_ne!(
        st2.turn.flag(engine_core::state::turn_flag::DRAW_REPLACED),
        0,
        "reduced draw should activate 裸衣"
    );
    assert_eq!(
        st2.hand_count(actor),
        hand_before + 1,
        "裸衣 reduces the normal two-card draw by exactly one"
    );
    assert_eq!(st2.turn.phase, Phase::Play, "draw phase done → play");
}

#[test]
fn luoyi_draw_normally_no_bonus() {
    let (st, actor, pipe) = open_draw_choice(7);
    let hand_before = st.hand_count(actor);
    let choice = pipe.choice.expect("choice");
    // Decline (cancel) → draw normally, no bonus.
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    assert_eq!(
        st2.turn.flag(engine_core::state::turn_flag::DRAW_REPLACED),
        0,
        "declining must not activate 裸衣"
    );
    assert!(
        st2.hand_count(actor) > hand_before,
        "should have drawn cards"
    );
    assert_eq!(st2.turn.phase, Phase::Play);
}
