//! Golden for XIANDING 卞玥【庇族/无胁】（ABI 598）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCBIZU: u32 = 2_914_157_062;
const DCWUXIE: u32 = 3_514_743_897;
const SOURCE: &str = include_str!("../../../../content/original/ported/xianding/bianyue.skill");

fn setup() -> (
    GameState,
    PlayerId,
    PlayerId,
    PlayerId,
    engine_api::SkillId,
    engine_api::SkillId,
) {
    let package = content_compiler::compile_source_named("bianyue-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, other) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        let cards = tx.state().hand_cards(*player);
        if !cards.is_empty() {
            effects.push(EffectRequest::DiscardCards {
                player: *player,
                cards,
            });
        }
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCBIZU),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCWUXIE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = |def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let bizu = skill(DCBIZU);
    let wuxie = skill(DCWUXIE);
    (tx.commit().state, owner, target, other, bizu, wuxie)
}

fn alloc(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    rank: u8,
    kind: CardKind,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap_or_else(|| {
        tx.state()
            .zones
            .find(zone, PlayerId::NONE)
            .expect("shared zone")
    });
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_800 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(state: GameState, owner: PlayerId, skill: engine_api::SkillId, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn bizu_repeated_exact_group_draws_then_bans_and_recovers() {
    let (state, owner, target, _other, bizu, _) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    alloc(&mut tx, owner, ZoneKind::Hand, 1, CardKind::Other);
    alloc(&mut tx, target, ZoneKind::Hand, 2, CardKind::Other);
    for rank in 3..=6 {
        alloc(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            rank,
            CardKind::Other,
        );
    }
    let state = activate(tx.commit().state, owner, bizu, 1);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 2);
    let state = activate(state, owner, bizu, 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(state.skills.get(bizu).unwrap().active_turn_banned);
}

#[test]
fn wuxie_swaps_whole_hands_then_places_new_damage_cards_at_draw_bottom() {
    let (state, owner, target, _other, _bizu, wuxie) = setup();
    let mut tx = Transaction::begin(state);
    let owner_card = alloc(&mut tx, owner, ZoneKind::Hand, 1, CardKind::Other);
    let damage = alloc(&mut tx, target, ZoneKind::Hand, 2, CardKind::Sha);
    let safe = alloc(&mut tx, target, ZoneKind::Hand, 3, CardKind::Other);
    let top = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        4,
        CardKind::Other,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveBianyue {
                player: owner,
                target: owner,
                action: 10,
                skill: wuxie,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(owner), vec![safe]);
    assert_eq!(state.hand_cards(target), vec![owner_card]);
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = &state.zones.get(draw).unwrap().cards;
    assert_eq!(cards.first(), Some(&damage));
    assert_eq!(cards.last(), Some(&top));
}
