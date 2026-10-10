//! 英姿 / 连营 / 枭姬.

use engine_api::{
    CardDefId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, Suit,
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
        def: CardDefId::new(9400 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: kind.is_equip(),
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

#[test]
fn roster_has_yingzi_lianying_xiaoji_paoxiao() {
    use engine_core::content::STANDARD_CHARACTERS;
    assert!(STANDARD_CHARACTERS
        .iter()
        .any(|c| c.name_key == "zhouyu" && c.skills.contains(&skill_def::YINGZI)));
    assert!(STANDARD_CHARACTERS
        .iter()
        .any(|c| c.name_key == "luxun" && c.skills.contains(&skill_def::LIANYING)));
    assert!(STANDARD_CHARACTERS
        .iter()
        .any(|c| c.name_key == "sunshangxiang" && c.skills.contains(&skill_def::XIAOJI)));
    assert!(STANDARD_CHARACTERS
        .iter()
        .any(|c| c.name_key == "zhangfei" && c.skills.contains(&skill_def::PAOXIAO)));
}

#[test]
fn yingzi_draws_three_in_draw_phase() {
    let mut eng = Engine::new();
    eng.submit(start(100, 2));
    let actor = eng.state().turn.current;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    grant(&mut tx, actor, skill_def::YINGZI);
    // Start at Judge so EndPhase → Draw + draw cards
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Judge;
    // Clear skip flags / empty judge
    let before = tx.state().hand_count(actor);

    let (st, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(st.turn.phase, Phase::Draw);
    // DEFAULT_DRAW=2 + 英姿 1
    assert_eq!(st.hand_count(actor), before + 3);
}

#[test]
fn lianying_draws_when_last_hand_lost() {
    let mut eng = Engine::new();
    eng.submit(start(101, 2));
    let actor = eng.state().turn.current;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    // Clear first so 连营不会在清空时触发
    clear_hand(&mut tx, actor);
    grant(&mut tx, actor, skill_def::LIANYING);
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let only = inject(&mut tx, actor, hand, CardKind::Sha, Suit::Spade, 1);
    assert_eq!(tx.state().hand_count(actor), 1);

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![only],
            to: discard,
            from: Some(hand),
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    // lost last card → optional 连营 accepted → draw 1
    assert_eq!(state.hand_count(actor), 1);
}

#[test]
fn xiaoji_offers_once_per_lost_equipment_card() {
    let mut eng = Engine::new();
    eng.submit(start(102, 2));
    let actor = eng.state().turn.current;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    // Roster-independent: drop every booted skill before granting 天香, so the
    // equipment-loss prompt below fires only from the granted skill.
    {
        let booted: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        if !booted.is_empty() {
            run_effects(&mut tx, booted).unwrap();
        }
    }
    grant(&mut tx, actor, skill_def::XIAOJI);

    clear_hand(&mut tx, actor);
    let equip = tx.state().zones.find(ZoneKind::Equip, actor).unwrap();
    let weapon = inject(&mut tx, actor, equip, CardKind::Weapon, Suit::Spade, 6);
    let armor = inject(&mut tx, actor, equip, CardKind::Armor, Suit::Club, 2);
    let hand_before = tx.state().hand_count(actor);

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon, armor],
            to: discard,
            from: Some(equip),
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, first) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, second) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(actor), hand_before + 4);
}

#[test]
fn package_loads_new_skill_defs() {
    let mut eng = Engine::new();
    eng.submit(start(103, 4));
    // 英姿/咆哮：modifier；连营/枭姬：trigger
    let yz = eng
        .content()
        .modifier_defs_for_skill(engine_api::SkillDefId::new(skill_def::YINGZI));
    assert!(!yz.is_empty(), "yingzi modifier");
    assert_eq!(yz[0].value, 1);
    for id in [skill_def::LIANYING, skill_def::XIAOJI] {
        let defs = eng
            .content()
            .defs_for_skill(engine_api::SkillDefId::new(id));
        assert!(!defs.is_empty(), "skill {id} trigger should load");
    }
    let px = eng
        .content()
        .modifier_defs_for_skill(engine_api::SkillDefId::new(skill_def::PAOXIAO));
    assert_eq!(px.len(), 1, "paoxiao sha-use modifier");
    assert_eq!(
        px[0].target,
        engine_core::query::ModifierTarget::ShaUseLimit
    );
    assert_eq!(px[0].value, i32::MAX);
}

#[test]
fn jizhi_only_on_tricks() {
    let mut eng = Engine::new();
    eng.submit(start(105, 2));
    use engine_core::trigger::TriggerFilter;
    let defs = eng
        .content()
        .defs_for_skill(engine_api::SkillDefId::new(skill_def::JIZHI));
    assert!(
        defs.iter()
            .any(|d| d.filter == TriggerFilter::OwnerSourceOrdinaryTrick && d.optional),
        "jizhi should filter tricks only"
    );
}

#[test]
fn lianying_xiaoji_use_zone_filters() {
    let mut eng = Engine::new();
    eng.submit(start(104, 4));
    use engine_core::trigger::TriggerFilter;
    let ly = eng
        .content()
        .defs_for_skill(engine_api::SkillDefId::new(skill_def::LIANYING));
    assert!(ly
        .iter()
        .any(|d| d.filter == TriggerFilter::OwnerFromHandEmpty && d.optional));
    let xj = eng
        .content()
        .defs_for_skill(engine_api::SkillDefId::new(skill_def::XIAOJI));
    assert!(xj
        .iter()
        .any(|d| d.filter == TriggerFilter::OwnerFromEquipPerCard && d.optional));
}
