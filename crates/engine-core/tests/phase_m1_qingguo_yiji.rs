//! 倾国（黑当闪）、遗计（受伤摸 2）、结姻挂载.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
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
    zone: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9500 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    };
    let cid = tx.state_mut().cards.alloc(card);
    if let Some(z) = tx.state_mut().zones.get_mut(zone) {
        z.cards.push(cid);
    }
    if let Some(c) = tx.state_mut().cards.get_mut(cid) {
        c.zone = zone;
        c.owner = owner;
    }
    cid
}

fn grant(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: engine_api::SkillDefId::new(def),
        }],
    )
    .unwrap();
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

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards: Vec<_> = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
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

fn other_of(eng: &Engine, actor: PlayerId) -> PlayerId {
    eng.state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id
}

#[test]
fn roster_zhenji_qingguo_guojia_yiji_ssx_jieyin() {
    use engine_core::content::STANDARD_CHARACTERS;
    assert!(STANDARD_CHARACTERS
        .iter()
        .any(|c| c.name_key == "zhenji" && c.skills.contains(&skill_def::QINGGUO)));
    assert!(STANDARD_CHARACTERS
        .iter()
        .any(|c| c.name_key == "guojia" && c.skills.contains(&skill_def::YIJI)));
    assert!(STANDARD_CHARACTERS
        .iter()
        .any(|c| c.name_key == "sunshangxiang" && c.skills.contains(&skill_def::JIEYIN)));
}

#[test]
fn qingguo_black_as_shan_blocks_sha() {
    let mut eng = Engine::new();
    eng.submit(start(110, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    grant(&mut tx, other, skill_def::QINGGUO);
    let hand_a = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hand_b = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let sha = inject(&mut tx, actor, hand_a, CardKind::Sha, Suit::Heart, 1);
    let black = inject(&mut tx, other, hand_b, CardKind::Tao, Suit::Spade, 3);
    let hp = tx.state().players.get(other).unwrap().hp;

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
    assert!(st.has_pending_choice(), "should offer shan options");
    let choice = pipe.choice.unwrap();
    assert!(
        choice.options.iter().any(|o| o.token == black.raw()),
        "black card should be legal 倾国 response"
    );
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![black.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    assert_eq!(st2.players.get(other).unwrap().hp, hp);
    assert!(!st2.hand_cards(other).contains(&black));
}

#[test]
fn yiji_draws_two_on_damage() {
    let mut eng = Engine::new();
    eng.submit(start(111, 2));
    let actor = eng.state().turn.current;
    let other = other_of(&eng, actor);

    let mut tx = Transaction::begin(eng.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().revision = 1;
    grant(&mut tx, other, skill_def::YIJI);
    // Avoid dying: set high hp
    if let Some(p) = tx.state_mut().players.get_mut(other) {
        p.hp = 5;
        p.max_hp = 5;
    }
    let before = tx.state().hand_count(other);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: actor,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    let cards = pipe.choice.unwrap();
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: cards.choice_id,
                selections: cards.options.iter().map(|option| option.token).collect(),
                cancelled: false,
            },
        },
    );
    let recipient = pipe2.choice.unwrap();
    let (st3, pipe3) = apply_command(
        st2,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: recipient.choice_id,
                selections: vec![other.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe3.result, CommandResult::Accepted { .. }));
    assert_eq!(st3.hand_count(other), before + 2);
}

#[test]
fn jieyin_discards_two_and_recovers_both_once() {
    let mut eng = Engine::new();
    eng.submit(start(113, 8));
    let actor = eng.state().turn.current;
    let target = eng
        .state()
        .players
        .iter_live()
        .find(|player| {
            player.id != actor
                && eng
                    .state()
                    .content
                    .character(player.character_def)
                    .is_some_and(|character| character.sex == "male")
        })
        .unwrap()
        .id;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    grant(&mut tx, actor, skill_def::JIEYIN);
    tx.state_mut().players.get_mut(actor).unwrap().hp -= 1;
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;
    let actor_hp = tx.state().players.get(actor).unwrap().hp;
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let first = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 1);
    let second = inject(&mut tx, actor, hand, CardKind::Shan, Suit::Heart, 2);
    let skill = tx
        .state()
        .players
        .get(actor)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def::JIEYIN)
        .unwrap();
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![first, second],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(actor).unwrap().hp, actor_hp + 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp + 1);
    assert!(!state.hand_cards(actor).contains(&first));
    assert!(!state.hand_cards(actor).contains(&second));
}

#[test]
fn qingnang_discards_one_and_recovers_a_wounded_target_once() {
    let mut eng = Engine::new();
    eng.submit(start(114, 2));
    let actor = eng.state().turn.current;
    let target = other_of(&eng, actor);
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    grant(&mut tx, actor, skill_def::QINGNANG);
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;
    let hp = tx.state().players.get(target).unwrap().hp;
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let cost = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 1);
    let skill = tx
        .state()
        .players
        .get(actor)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def::QINGNANG)
        .unwrap();
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![cost],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp + 1);
    assert!(!state.hand_cards(actor).contains(&cost));
}

#[test]
fn package_has_qingguo_view_as() {
    let mut eng = Engine::new();
    eng.submit(start(112, 4));
    let vs = eng
        .content()
        .view_as_defs_for_skill(engine_api::SkillDefId::new(skill_def::QINGGUO));
    assert!(!vs.is_empty());
    assert_eq!(vs[0].as_kind, "shan");
    assert_eq!(vs[0].filter, "black");
}
