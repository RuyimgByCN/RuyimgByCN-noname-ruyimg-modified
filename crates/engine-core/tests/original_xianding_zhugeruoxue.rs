//! Golden for XIANDING 诸葛若雪【琼英/暖惠】（ABI 605）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCQIONGYING: u32 = 2_641_115_727;
const DCNUANHUI: u32 = 2_308_165_295;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/zhugeruoxue.skill");

fn setup(def: u32) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("zhugeruoxue-test", SOURCE).unwrap();
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
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(605_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
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
        "{result:?}"
    );
    state
}

#[test]
fn qiongying_moves_field_card_then_discards_matching_suit_hand_card() {
    let (state, [owner, source, destination], skill) = setup(DCQIONGYING);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let weapon = alloc(
        &mut tx,
        source,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        1,
    );
    let cost = alloc(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        2,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveZhugeruoxue {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                action: 0,
                remaining: 0,
                used_mask: 0,
                repeated: false,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(tx.commit().state, owner, vec![weapon.raw()], false, 6051);
    let state = choose(state, owner, vec![destination.raw()], false, 6052);
    let state = choose(state, owner, vec![cost.raw()], false, 6053);
    let equip = state.zones.find(ZoneKind::Equip, destination).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, equip);
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
}

#[test]
fn nuanhui_snapshots_equipment_count_and_discards_equipment_after_repeated_name() {
    let (state, [owner, target, _], skill) = setup(DCNUANHUI);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let weapon = alloc(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        3,
    );
    let armor = alloc(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        4,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveZhugeruoxue {
                player: owner,
                target,
                card: CardId::NONE,
                action: 11,
                remaining: 0,
                used_mask: 0,
                repeated: false,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(
        tx.commit().state,
        target,
        vec![u32::from(CardKind::Jiu.tag())],
        false,
        6054,
    );
    let state = choose(
        state,
        target,
        vec![u32::from(CardKind::Jiu.tag())],
        false,
        6055,
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard);
    assert_eq!(state.cards.get(armor).unwrap().zone, discard);
}
