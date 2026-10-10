//! Golden coverage for TW 桥公「遗珠」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWYIZHU: u32 = 2_650_180_014;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twyizhu.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_373_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn planted_card_used_by_other_can_add_target_then_is_untracked_and_draws() {
    let package = content_compiler::compile_source_named("twyizhu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 373);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_114, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, user, target) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWYIZHU),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWYIZHU)
        .unwrap()
        .id;
    let sha = inject(&mut tx, owner, CardKind::Sha, 1);
    let pearl = inject(&mut tx, owner, CardKind::Tao, 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwyizhu {
            player: owner,
            source: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            stage: 1,
            event_cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert!(tx.state().zones.get(draw).unwrap().cards.contains(&sha));
    let user_hand = tx.state().zones.find(ZoneKind::Hand, user).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![sha],
            to: user_hand,
            from: Some(draw),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = user;
    tx.state_mut().turn.phase = Phase::Play;
    let mut state = tx.commit().state;
    let owner_hand_before = state.hand_count(owner);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(target).unwrap().hp;

    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: user,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state = next;
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![owner.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    // Drive the 杀: both targets decline 闪, then the damage commits.
    let mut state = state;
    let mut key = 3u64;
    while state.has_pending_choice() {
        let (next, r) = apply_command(
            state.clone(),
            Command {
                actor: PlayerId::new(state.resolution.pending_observer),
                expected_revision: ExpectedRevision(state.revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(r.result, CommandResult::Accepted { .. }), "{:?}", r.result);
        state = next;
        key += 1;
    }
    assert_eq!(state.hand_count(owner), owner_hand_before + 1);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(&instance.state_blob[..4], b"KJN1");
    assert_eq!(&instance.state_blob[4..], &pearl.raw().to_le_bytes());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![pearl],
            to: discard,
            from: Some(draw),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.skills.get(skill).unwrap().state_blob, b"KJN1");
}
