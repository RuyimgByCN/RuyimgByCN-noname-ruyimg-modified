//! Golden coverage for BINGSHI 势孙綝「逆固」「戮连」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTNIGU: u32 = 3_220_807_381;
const POTLULIAN: u32 = 2_734_921_313;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_sunchen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(def: u32, seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects = Vec::new();
    for player in tx.state().players.alive_by_seat() {
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            });
        }
    }
    run_effects(tx, effects).unwrap();
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
    weapon_range: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(532_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

#[test]
fn potnigu_collects_optional_gifts_and_buffs_exactly_each_refused_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_sunchen")
            .unwrap()
            .skills,
        [POTNIGU, POTLULIAN]
    );
    let (state, owner, order) = setup(POTNIGU, 5321);
    let others: Vec<_> = order
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx);
    let cost_a = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Spade,
        1,
        0,
    );
    let cost_b = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Heart,
        2,
        0,
    );
    inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        3,
        8,
    );
    let gift = inject(
        &mut tx,
        others[0],
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        4,
        0,
    );
    inject(
        &mut tx,
        others[1],
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        5,
        0,
    );
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner, POTNIGU),
                targets: Vec::new(),
                card_targets: vec![cost_a, cost_b],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let first_observer = PlayerId::new(state.resolution.pending_observer);
    let first_gift = if first_observer == others[0] {
        gift
    } else {
        state.hand_cards(first_observer)[0]
    };
    let (state, output) = resolve(state, vec![first_gift.raw()], false, 2);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let (state, output) = resolve(state, Vec::new(), true, 3);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(state.hand_cards(owner).contains(&first_gift));

    let before = state.players.get(others[0]).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: others[0],
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(others[0]).unwrap().hp, before - 2);
}

#[test]
fn potlulian_links_targets_draws_and_forces_fire_damage_after_hand_card_resolves() {
    let (state, owner, order) = setup(POTLULIAN, 5322);
    let others: Vec<_> = order
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let target = others[0];
    let minimum = others[1];
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx);
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    tx.state_mut().players.get_mut(minimum).unwrap().hp = 1;
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        10,
        0,
    );
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == engine_core::content::MARK_CHAIN && *count > 0 }));
    assert_eq!(state.hand_count(owner), 1);
    let options: Vec<_> = output
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&target.raw()));
    assert!(!options.contains(&minimum.raw()));
    let before = state.players.get(target).unwrap().hp;
    let (state, output) = resolve(state, vec![target.raw()], false, 11);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, before - 1);
}
