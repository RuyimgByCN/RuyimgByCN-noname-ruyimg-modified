//! Golden for XIANDING 张臶【技能A/技能B】（ABI 604）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_ZJ_A: u32 = 2_792_442_500;
const DC_ZJ_B: u32 = 3_520_470_433;
const SOURCE: &str = include_str!("../../../../content/original/ported/xianding/zhangjian.skill");

fn setup(def: u32) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("zhangjian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
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

fn alloc(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_604 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selection: u32) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(604),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn skill_a_rewrites_physical_card_damage_to_rank_and_prevents_rankless_card_damage() {
    let (state, [owner, source, _], _) = setup(DC_ZJ_A);
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 20;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 20;
    let card = alloc(&mut tx, source, CardKind::Sha, 7);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 13);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 4,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 13);
}

#[test]
fn skill_b_discards_all_and_grants_skill_a_until_owners_next_turn() {
    let (state, [owner, target, previous], skill) = setup(DC_ZJ_B);
    let mut tx = Transaction::begin(state);
    alloc(&mut tx, owner, CardKind::Sha, 3);
    alloc(&mut tx, owner, CardKind::Tao, 4);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveZhangjian {
                player: owner,
                card: CardId::NONE,
                grant_skill_def: SkillDefId::new(DC_ZJ_A),
                action: 1,
                target: PlayerId::NONE,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(tx.commit().state, owner, target.raw());
    assert_eq!(state.hand_count(owner), 0);
    let granted = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == target && instance.def.raw() == DC_ZJ_A)
        .unwrap();
    assert_eq!(&granted.state_blob[..4], b"STB1");
    assert_eq!(
        u32::from_le_bytes(granted.state_blob[4..8].try_into().unwrap()),
        owner.raw()
    );

    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    tx.state_mut().turn.current = previous;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::TURN_END_DISPATCHED, 1);
    run_effects(&mut tx, vec![EffectRequest::AdvanceTurn]).unwrap();
    assert_eq!(tx.state().turn.current, owner);
    assert!(!tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == target && instance.def.raw() == DC_ZJ_A));
}
