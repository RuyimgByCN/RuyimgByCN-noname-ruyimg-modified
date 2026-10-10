//! Golden for XIANDING 新杀谋鲁肃【明势/盟谋】（ABI 603）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBMINGSHI: u32 = 4_288_768_048;
const DCSBMENGMOU: u32 = 2_188_556_454;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/dc_sb_lusu.skill");

fn setup(def: u32) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("dc-sb-lusu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
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
        player: players[0],
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == def)
        .unwrap()
        .id;
    (tx.commit().state, players, skill)
}

fn alloc(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone_owner = if matches!(zone_kind, ZoneKind::Hand) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = tx.state().zones.find(zone_kind, zone_owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_603 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice {key}: {result:?}"
    );
    state
}

#[test]
fn mingshi_draws_two_then_reveals_three_and_gives_the_chosen_card() {
    let (state, [owner, target, _], skill) = setup(DCSBMINGSHI);
    let mut tx = Transaction::begin(state);
    alloc(&mut tx, owner, ZoneKind::Hand, CardKind::Other, 1);
    alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        2,
    );
    alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        3,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbLusu {
                player: owner,
                source: owner,
                card: CardId::NONE,
                action: 0,
                cards: Vec::new(),
                amount: 0,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let cards = tx.state().hand_cards(owner);
    assert_eq!(cards.len(), 3);
    let state = resolve(
        tx.commit().state,
        owner,
        cards.iter().map(|card| card.raw()).collect(),
        false,
        6031,
    );
    let state = resolve(state, owner, vec![target.raw()], false, 6032);
    let gained = cards[1];
    let state = resolve(state, target, vec![gained.raw()], false, 6033);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 1);
    assert_eq!(state.cards.get(gained).unwrap().owner, target);
}

#[test]
fn mengmou_yin_counts_physical_sha_responses_then_loses_remaining_hp() {
    let (state, [owner, target, _], skill) = setup(DCSBMENGMOU);
    let mut tx = Transaction::begin(state);
    let transferred = alloc(&mut tx, target, ZoneKind::Hand, CardKind::Other, 1);
    let sha = alloc(&mut tx, target, ZoneKind::Hand, CardKind::Sha, 2);
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 2;
    tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1;
    let hp = tx.state().players.get(target).unwrap().hp;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbLusu {
                player: owner,
                source: owner,
                card: transferred,
                action: 11,
                cards: vec![transferred],
                amount: 1,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx.commit().state, target, vec![sha.raw()], false, 6034);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        hp - (max_hp - 1) as i32
    );
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 0);
    assert!(instance.state_blob.is_empty());
}

#[test]
fn mengmou_yang_uses_normal_sha_and_recovers_for_exact_damage() {
    let (state, [owner, target, victim], skill) = setup(DCSBMENGMOU);
    let mut tx = Transaction::begin(state);
    let transferred = alloc(&mut tx, target, ZoneKind::Hand, CardKind::Other, 1);
    let sha = alloc(&mut tx, target, ZoneKind::Hand, CardKind::Sha, 2);
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 1;
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbLusu {
                player: owner,
                source: owner,
                card: transferred,
                action: 11,
                cards: vec![transferred],
                amount: 1,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx.commit().state, target, vec![sha.raw()], false, 6035);
    let state = resolve(state, target, vec![victim.raw()], false, 6036);
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp - 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp + 1);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 1);
    assert!(instance.state_blob.is_empty());
}
