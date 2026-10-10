//! Golden for 3D精选 张闿【劫行/拜泪】（ABI 639）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JIEXING: u32 = 3_377_566_778;
const BAILEI: u32 = 2_689_073_031;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/zhangkai.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-zhangkai", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 639);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap())
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_804, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
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
    effects.extend([JIEXING, BAILEI].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 4;
        participant.hp = 3;
    }
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, kind: CardKind) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap_or_else(|| {
        state
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .unwrap()
    });
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_639_000),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn jiexing_steals_recovery_then_only_a_later_recovery_resets_it() {
    let (mut state, players) = setup();
    let owner = players[0];
    let other = players[1];
    state.players.get_mut(owner).unwrap().hp = 1;
    state.players.get_mut(other).unwrap().hp = 1;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RecoverHp {
                source: other,
                player: other,
                amount: 2,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(other).unwrap().hp, 1);
    let jiexing = skill_id(&state, owner, JIEXING);
    assert_eq!(state.skills.get(jiexing).unwrap().lifetime_fire_count, 1);
    assert_eq!(state.skills.get(jiexing).unwrap().content_flags, 0);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().skills.get(jiexing).unwrap().lifetime_fire_count,
        0
    );
}

#[test]
fn jiexing_cancels_equipment_use_equips_material_and_later_use_resets() {
    let (mut state, players) = setup();
    let owner = players[0];
    let other = players[1];
    let old = inject(&mut state, owner, ZoneKind::Equip, CardKind::Armor);
    let card = inject(&mut state, other, ZoneKind::Hand, CardKind::Armor);
    let jiexing = skill_id(&state, owner, JIEXING);
    state.turn.current = other;
    state.turn.phase = Phase::Play;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![1], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(equip).unwrap().cards.contains(&card));
    assert!(state.zones.get(discard).unwrap().cards.contains(&old));
    assert_eq!(state.skills.get(jiexing).unwrap().lifetime_fire_count, 1);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: owner,
            card,
            kind: CardKind::Armor.tag(),
            targets: Vec::new(),
            damaged_targets: Vec::new(),
            source_cards: vec![card],
            all_sources_were_hand: true,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state().skills.get(jiexing).unwrap().lifetime_fire_count,
        0
    );
}

#[test]
fn bailei_damage_that_kills_removes_bailei() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[1];
    state.players.get_mut(target).unwrap().hp = 1;
    let bailei = skill_id(&state, owner, BAILEI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhangkai {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                cards: Vec::new(),
                amount: 0,
                action: 4,
                skill: bailei,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![target.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(target).unwrap().alive);
    assert!(state.skills.get(bailei).is_none());
}

#[test]
fn bailei_gains_from_the_unique_most_equipped_target() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[1];
    let equipment = inject(&mut state, target, ZoneKind::Equip, CardKind::Weapon);
    let bailei = skill_id(&state, owner, BAILEI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhangkai {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                cards: Vec::new(),
                amount: 0,
                action: 4,
                skill: bailei,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = choose(tx.commit().state, owner, vec![target.raw()], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![equipment.raw()], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&equipment));
    assert!(state.skills.get(bailei).is_some());
}
