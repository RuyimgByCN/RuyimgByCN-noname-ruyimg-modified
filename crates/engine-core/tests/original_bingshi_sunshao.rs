//! Golden coverage for BINGSHI 孙韶「敢决」「筑墼」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBGANJUE: u32 = 3_789_971_232;
const MBZHUJI: u32 = 2_468_204_018;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/sunshao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64, players: u8, skill_def: u32) -> (GameState, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("sunshao_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(521_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) + 1,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_sunshao_at_abi521() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "sunshao")
            .unwrap()
            .skills,
        [MBGANJUE, MBZHUJI]
    );
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == MBGANJUE)
        .unwrap();
    assert!(view.ignore_distance && view.no_use_count);
    assert!(view.target_lacks_source_suit_unrespondable);
}

#[test]
fn ganjue_ignores_distance_and_sha_count_but_same_suit_hand_can_respond() {
    let (state, owner, skill) = setup(5211, 4, MBGANJUE);
    let seats = state.players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, far);
    let source = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        7,
    );
    let shan = inject(&mut tx, far, ZoneKind::Hand, CardKind::Shan, Suit::Heart, 8);
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 1);
    let hp = tx.state().players.get(far).unwrap().hp;
    let (state, output) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![far],
                card_targets: vec![source],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let state = choose(state, vec![shan.raw()], false, 2);
    assert_eq!(state.players.get(far).unwrap().hp, hp);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
}

#[test]
fn ganjue_target_without_matching_suit_cannot_respond() {
    let (state, owner, skill) = setup(5212, 2, MBGANJUE);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, target);
    let source = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        9,
    );
    inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        10,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, output) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![source],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn zhuji_discards_exact_suit_uses_shared_equip_and_hujia_absorbs_damage() {
    let (state, owner, skill) = setup(5213, 2, MBZHUJI);
    let source = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    let heart_a = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        3,
    );
    let heart_b = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        4,
    );
    let club = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        5,
    );
    let weapon = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        Suit::Heart,
        6,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbzhuji {
                player: owner,
                choice: 0,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![2], false, 4);
    assert!(state.has_pending_choice());
    assert!(!state.hand_cards(owner).contains(&heart_a));
    assert!(!state.hand_cards(owner).contains(&heart_b));
    assert!(state.hand_cards(owner).contains(&club));
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert!(state.zones.get(equip).unwrap().cards.contains(&weapon));
    let state = choose(state, vec![3], false, 5);
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_HUJIA && *count > 0));
}
