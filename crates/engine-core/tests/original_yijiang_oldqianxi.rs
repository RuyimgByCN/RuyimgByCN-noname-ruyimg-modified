//! Golden coverage for YIJIANG「旧潜袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDQIANXI: u32 = 3_282_621_718;
const MADAI: u32 = 2_878_015_596;
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/oldqianxi.skill");

fn setup(player_count: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("oldqianxi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_718, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDQIANXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    (tx, owner, seats)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn open_sha_damage(tx: &mut Transaction, source: PlayerId, target: PlayerId, card: CardId) {
    assert_eq!(
        run_effects(
            tx,
            vec![EffectRequest::DealShaDamage {
                source,
                target,
                card,
                amount: 1,
                nature: 0,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

fn resolve_optional(tx: Transaction, actor: PlayerId, accepted: bool, key: u64) -> GameState {
    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        committed.state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: accepted.then_some(1).into_iter().collect(),
                cancelled: !accepted,
            },
        },
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "unexpected optional-trigger result: {:?}",
        pipeline.result
    );
    assert!(pipeline.choice.is_none());
    state
}

#[test]
fn packages_publish_exact_skill_character_and_dependencies() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    let compiled = content_compiler::compile_source_named("oldqianxi", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = yijiang
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    let trigger = yijiang
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLDQIANXI)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_target_alive_distance_le_one"
    );
    assert!(trigger.optional);

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    for dependency in ["standard", "yijiang"] {
        assert!(old.manifest.dependencies.iter().any(|entry| {
            entry.namespace == "official"
                && entry.package_id == dependency
                && entry.version == "0.1.0"
        }));
    }
    let madai = old
        .character_index
        .iter()
        .find(|character| character.id == MADAI)
        .unwrap();
    assert_eq!(madai.name_key, "madai");
    assert_eq!(madai.max_hp, 4);
    assert_eq!(madai.skills, [2, OLDQIANXI]);
}

#[test]
fn nonheart_judgment_prevents_damage_and_reduces_max_hp() {
    let (mut tx, owner, seats) = setup(2);
    let target = seats[1];
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    let judgment = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Spade,
        97_000,
    );
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        97_001,
    );
    open_sha_damage(&mut tx, owner, target, sha);
    let state = resolve_optional(tx, owner, true, 1);
    let target_state = state.players.get(target).unwrap();
    assert_eq!((target_state.hp, target_state.max_hp), (3, 3));
    assert!(state.resolution.prevented_damage_ids.is_empty());
    assert_eq!(state.resolution.current_damage_id, 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&judgment));
}

#[test]
fn heart_judgment_keeps_damage_and_max_hp_unchanged() {
    let (mut tx, owner, seats) = setup(2);
    let target = seats[1];
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Heart,
        97_010,
    );
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        97_011,
    );
    open_sha_damage(&mut tx, owner, target, sha);
    let state = resolve_optional(tx, owner, true, 2);
    let target_state = state.players.get(target).unwrap();
    assert_eq!((target_state.hp, target_state.max_hp), (2, 4));
}

#[test]
fn declining_the_optional_trigger_keeps_normal_sha_damage() {
    let (mut tx, owner, seats) = setup(2);
    let target = seats[1];
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        97_020,
    );
    open_sha_damage(&mut tx, owner, target, sha);
    let state = resolve_optional(tx, owner, false, 3);
    let target_state = state.players.get(target).unwrap();
    assert_eq!((target_state.hp, target_state.max_hp), (2, 4));
}

#[test]
fn far_sha_and_non_sha_damage_do_not_open_the_trigger() {
    let (mut tx, owner, seats) = setup(4);
    let far = seats[2];
    tx.state_mut().players.get_mut(far).unwrap().hp = 3;
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        97_030,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: owner,
            target: far,
            card: sha,
            amount: 1,
            nature: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(far).unwrap().hp, 2);

    let near = seats[1];
    tx.state_mut().players.get_mut(near).unwrap().hp = 3;
    let duel = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Club,
        97_031,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: owner,
            target: near,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: duel,
            card_kind: CardKind::TrickJuedou.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(near).unwrap().hp, 2);
}
