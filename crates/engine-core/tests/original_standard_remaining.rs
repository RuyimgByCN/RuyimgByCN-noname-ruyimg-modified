//! Closure coverage for the last four legacy `standard` skills.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GANGLIE_THREE: u32 = 3_884_116_323;
const ZHONGYI: u32 = 2_818_590_263;
const ZHONGYI2: u32 = 3_789_127_080;
const ZHANSHEN: u32 = 3_834_202_888;
const MASHU: u32 = 2;
const SHENJI: u32 = 3_515_984_232;
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const EXTENSION: &[u8] = include_bytes!("../../../extensions/standard/package.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let ally = players[1];
    let enemy = players[2];
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for (player, identity) in [
        (owner, Identity::Lord),
        (ally, Identity::Loyalist),
        (enemy, Identity::Rebel),
    ] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.identity = identity;
        participant.hp = 10;
        participant.max_hp = 10;
    }
    (tx.commit().state, owner, ally, enemy)
}

fn add_skill(state: GameState, player: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.commit().state
}

fn skill_id(state: &GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == def)
        })
        .unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn put_judgment(state: &mut GameState, suit: Suit) {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(990_001),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
}

fn equip_first_hand_as_weapon(state: &mut GameState, player: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let equip = state.zones.find(ZoneKind::Equip, player).unwrap();
    let card = state.zones.get(hand).unwrap().cards[0];
    state
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|found| *found != card);
    state.zones.get_mut(equip).unwrap().cards.push(card);
    let card_state = state.cards.get_mut(card).unwrap();
    card_state.zone = equip;
    card_state.kind = CardKind::Weapon;
    card_state.weapon_range = 3;
    card
}

#[test]
fn package_and_drop_in_extension_publish_the_four_rewrites() {
    assert_eq!(PACKAGE, EXTENSION);
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert_eq!(package.skill_index.len(), 65);
    for def in [GANGLIE_THREE, ZHONGYI, ZHONGYI2, ZHANSHEN] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == def));
    }
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZHONGYI2
            && trigger.filter == "owner_alive_source_ally_sha_damage_before"
    }));
}

#[test]
fn ganglie_three_targets_only_an_enemy_and_accepts_equipment_as_discard() {
    let (mut state, owner, ally, enemy) = setup(261_001);
    state = add_skill(state, owner, GANGLIE_THREE);
    let weapon = equip_first_hand_as_weapon(&mut state, enemy);
    let hand = state.hand_cards(enemy);
    put_judgment(&mut state, Suit::Spade);
    let enemy_hp = state.players.get(enemy).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: ally,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveGanglieThree
    );
    let state = resolve(state, owner, vec![enemy.raw()], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GanglieSourceDiscardOrDamage
    );
    let state = resolve(state, enemy, vec![weapon.raw(), hand[0].raw()], false, 3);
    assert_eq!(state.players.get(enemy).unwrap().hp, enemy_hp);
    assert!(!state
        .zones
        .get(state.zones.find(ZoneKind::Equip, enemy).unwrap())
        .unwrap()
        .cards
        .contains(&weapon));
}

#[test]
fn zhongyi_round_lifecycle_and_zhanshen_awakening_resolve() {
    let (mut state, owner, ally, enemy) = setup(261_002);
    state = add_skill(state, owner, ZHONGYI);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let card = state.hand_cards(owner)[0];
    let active = skill_id(&state, owner, ZHONGYI);
    let (state, activated) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ActivateSkill {
                skill: active,
                targets: Vec::new(),
                card_targets: vec![card],
            },
        },
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let helper = skill_id(&state, owner, ZHONGYI2);
    assert_eq!(state.skill_pile_cards(owner, ZHONGYI2), [card]);

    let enemy_hp = state.players.get(enemy).unwrap().hp;
    let mut tx = Transaction::begin(state);
    tx.state_mut().resolution.current_damage_id = 900;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::CommitDamage {
            damage_id: 900,
            source: ally,
            target: enemy,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        });
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhongyi {
            player: owner,
            action: 1,
            helper_skill_def: SkillDefId::new(ZHONGYI2),
            card: CardId::NONE,
            skill: helper,
        }],
    )
    .unwrap();
    let tail = std::mem::take(&mut tx.state_mut().resolution.deferred_tail_effects);
    tx.state_mut().resolution.current_damage_id = 0;
    run_effects(&mut tx, tail).unwrap();
    assert_eq!(tx.state().players.get(enemy).unwrap().hp, enemy_hp - 2);
    tx.state_mut().turn.round += 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhongyi {
            player: owner,
            action: 2,
            helper_skill_def: SkillDefId::new(ZHONGYI2),
            card: CardId::NONE,
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().skill_pile_cards(owner, ZHONGYI2).is_empty());
    assert!(tx.state().skills.get(helper).is_none());

    let mut state = tx.commit().state;
    state.players.get_mut(ally).unwrap().alive = false;
    let weapon = equip_first_hand_as_weapon(&mut state, owner);
    state.players.get_mut(owner).unwrap().hp = 8;
    state = add_skill(state, owner, ZHANSHEN);
    let max_hp = state.players.get(owner).unwrap().max_hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp - 1);
    assert!(tx
        .state()
        .skills
        .get(skill_id(tx.state(), owner, MASHU))
        .is_some());
    assert!(tx
        .state()
        .skills
        .get(skill_id(tx.state(), owner, SHENJI))
        .is_some());
    assert!(!tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx
            .state()
            .skills
            .get(*skill)
            .is_some_and(|skill| skill.def.raw() == ZHANSHEN)));
    assert!(!tx
        .state()
        .zones
        .get(tx.state().zones.find(ZoneKind::Equip, owner).unwrap())
        .unwrap()
        .cards
        .contains(&weapon));
}
