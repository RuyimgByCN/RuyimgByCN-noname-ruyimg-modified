//! Golden for XIANDING 势张绣【附袭/豪义】（ABI 601）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBFUXI: u32 = 3_319_521_189;
const DCSBHAOYI: u32 = 2_398_354_936;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/xianding/dc_sb_zhangxiu.skill");

fn setup(
    def: u32,
    phase: Phase,
    current_index: usize,
) -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("dc-sb-zhangxiu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
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
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[current_index];
    tx.state_mut().turn.phase = phase;
    tx.state_mut().turn.entered_discard_cards.clear();
    tx.state_mut().turn.damage_card_materials.clear();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id;
    (tx.commit().state, owner, target, skill)
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
        def: CardDefId::new(99_601 + u32::from(rank)),
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

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn fuxi_gives_one_card_then_draws_two() {
    let (state, owner, target, skill) = setup(DCSBFUXI, Phase::Play, 1);
    let mut tx = Transaction::begin(state);
    let gift = alloc(&mut tx, owner, ZoneKind::Hand, 1, CardKind::Other);
    alloc(&mut tx, target, ZoneKind::Hand, 4, CardKind::Other);
    alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        2,
        CardKind::Other,
    );
    alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        3,
        CardKind::Other,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhangxiu {
                player: owner,
                target,
                action: 0,
                cards: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx.commit().state, owner, vec![0], 1);
    let state = resolve(state, owner, vec![gift.raw()], 2);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.cards.get(gift).unwrap().owner, target);
}

#[test]
fn fuxi_discards_then_uses_a_distance_free_virtual_sha() {
    let (state, owner, target, skill) = setup(DCSBFUXI, Phase::Play, 1);
    let mut tx = Transaction::begin(state);
    let target_card = alloc(&mut tx, target, ZoneKind::Hand, 6, CardKind::Other);
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhangxiu {
                player: owner,
                target,
                action: 0,
                cards: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx.commit().state, owner, vec![1], 5);
    let state = resolve(state, owner, vec![target_card.raw()], 6);
    assert_eq!(
        state.cards.get(target_card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn haoyi_gains_only_non_damaging_damage_cards_then_can_distribute_them() {
    let (state, owner, target, skill) = setup(DCSBHAOYI, Phase::End, 0);
    let mut tx = Transaction::begin(state);
    let eligible = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        4,
        CardKind::Sha,
    );
    let excluded = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        5,
        CardKind::TrickJuedou,
    );
    tx.state_mut()
        .turn
        .record_entered_discard_cards([eligible, excluded]);
    tx.state_mut().turn.record_damage_card_materials([excluded]);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhangxiu {
                player: owner,
                target: PlayerId::NONE,
                action: 10,
                cards: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().cards.get(eligible).unwrap().owner, owner);
    assert_eq!(
        tx.state().cards.get(excluded).unwrap().owner,
        PlayerId::NONE
    );
    let state = resolve(tx.commit().state, owner, vec![eligible.raw()], 3);
    let state = resolve(state, owner, vec![target.raw()], 4);
    assert_eq!(state.cards.get(eligible).unwrap().owner, target);
    assert!(
        !state.has_pending_choice(),
        "unexpected continuation {:?}: {:?}",
        state.resolution.continuation_kind,
        state.resolution.continuation_payload
    );
}
