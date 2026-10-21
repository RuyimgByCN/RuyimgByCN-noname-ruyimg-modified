//! Golden for 联动「龙宫」：每回合一次防伤并令存活来源获得装备。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCLONGGONG: u32 = 3_209_573_631;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");

fn prepared(seed: u64) -> (Transaction, PlayerId, PlayerId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: tx.state().turn.current,
            skill_def: SkillDefId::new(DCLONGGONG),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let owner = tx.state().turn.current;
    let source = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 10;
    player.max_hp = 10;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let equipment = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_165_001),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 1,
        weapon_range: 2,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .push(equipment);
    (tx, owner, source, equipment)
}

fn resolve(tx: Transaction, owner: PlayerId, accept: bool, key: u64) -> GameState {
    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn damage(tx: &mut Transaction, source: PlayerId, owner: PlayerId) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

#[test]
fn accepted_use_prevents_damage_grants_equipment_and_consumes_turn_limit() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DCLONGGONG
            && trigger.event == "damage_before"
            && trigger.filter == "owner_is_target_source_alive_once"
            && trigger.optional
    }));

    let (mut tx, owner, source, equipment) = prepared(2_026_071_651);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let source_hand = tx.state().hand_count(source);
    assert_eq!(
        damage(&mut tx, source, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx, owner, true, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(source), source_hand + 1);
    let hand = state.zones.find(ZoneKind::Hand, source).unwrap();
    assert!(state.zones.get(hand).unwrap().cards.contains(&equipment));

    let mut tx = Transaction::begin(state);
    damage(&mut tx, source, owner).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn decline_keeps_the_use_and_source_free_damage_never_prompts() {
    let (mut tx, owner, source, _) = prepared(2_026_071_652);
    assert_eq!(
        damage(&mut tx, source, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx, owner, false, 2);
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        damage(&mut tx, source, owner),
        Err(EffectError::ChoiceRequested)
    );

    let (mut tx, owner, _, _) = prepared(2_026_071_653);
    damage(&mut tx, PlayerId::NONE, owner).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
}
