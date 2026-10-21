//! Golden coverage for BINGSHI 势曹爽「典易/奢权/渐专/溺尊」.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, Command, CommandKind, CommandResult,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DIANYI: u32 = 4_200_000_573;
const SHEQUAN: u32 = 4_200_000_574;
const JIANZHUAN: u32 = 4_200_000_575;
const NIZUN: u32 = 4_200_000_576;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/bingshi/pot_caoshuang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

#[test]
fn package_publishes_complete_pot_caoshuang_at_abi568() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    // 定稿 npack（2026-08-04 16:23 重打包）＝ 33 名上游武将 + ABI 521 孙韶 = 34；
    // 技能 99 项；上游 bingshi/card.js 无卡牌定义，card_index 为 0。
    assert_eq!(package.character_index.len(), 37);
    assert_eq!(package.skill_index.len(), 105);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_caoshuang")
            .unwrap()
            .skills,
        [DIANYI, SHEQUAN, JIANZHUAN]
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NIZUN));
}

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 568);
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
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [DIANYI, SHEQUAN, JIANZHUAN].map(|def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    tx.state_mut().turn.round = 1;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, serial: u32) -> CardId {
    let zone_owner = if matches!(
        zone_kind,
        ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge
    ) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = tx.state().zones.find(zone_kind, zone_owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(568_000 + serial),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
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

fn request(
    player: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolvePotcaoshuang {
        player,
        target: PlayerId::NONE,
        card,
        action,
        skill,
    }
}

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

fn assign(mut state: GameState, owner: PlayerId, action: u8, def: u32, key: u64) -> GameState {
    let jianzhuan = skill(&state, owner, JIANZHUAN);
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, CardId::NONE, action, jianzhuan)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    state = tx.commit().state;
    let (state, result) = resolve(state, vec![def], key);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn jianzhuan_assigns_all_timings_then_awakens_once() {
    let (state, owner, _) = setup(5680);
    let state = assign(state, owner, 0, DIANYI, 5681);
    let state = assign(state, owner, 1, SHEQUAN, 5682);
    let mut state = assign(state, owner, 2, DIANYI, 5683);
    state.players.get_mut(owner).unwrap().hp = 3;
    let jianzhuan = skill(&state, owner, JIANZHUAN);
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, CardId::NONE, 0, jianzhuan)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, result) = resolve(tx.commit().state, vec![DIANYI], 5684);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .all(|instance| instance.owner != owner || instance.def.raw() != DIANYI));
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == owner && instance.def.raw() == NIZUN));
    assert_eq!(state.players.get(owner).unwrap().max_hp, 5);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);
}

#[test]
fn shequan_freezes_private_choices_and_rewrites_hand_numbers() {
    let (state, owner, order) = setup(5685);
    let state = assign(state, owner, 0, SHEQUAN, 5686);
    let mut tx = Transaction::begin(state);
    let first = inject(&mut tx, order[1], ZoneKind::Hand, 1);
    let second = inject(&mut tx, order[2], ZoneKind::Hand, 2);
    let shequan = skill(tx.state(), owner, SHEQUAN);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, CardId::NONE, 6, shequan)]),
        Err(EffectError::ChoiceRequested)
    ));
    let pending = tx.commit().state;
    let first_observer = PlayerId::new(pending.resolution.pending_observer);
    let first_card = if first_observer == order[1] {
        first
    } else {
        second
    };
    let (pending, _) = resolve(pending, vec![first_card.raw()], 5687);
    let second_observer = PlayerId::new(pending.resolution.pending_observer);
    let second_card = if second_observer == order[1] {
        first
    } else {
        second
    };
    let (mut state, _) = resolve(pending, vec![second_card.raw()], 5688);
    assert!(state.card_has_content_tag(first, SHEQUAN));
    assert!(state.card_has_content_tag(second, SHEQUAN));
    assert_eq!(
        engine_core::query::effective_card_number(
            &state,
            order[1],
            state.cards.get(first).unwrap()
        ),
        1
    );
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let old_hand = state.zones.find(ZoneKind::Hand, order[1]).unwrap();
    state
        .zones
        .get_mut(old_hand)
        .unwrap()
        .cards
        .retain(|card| *card != first);
    state.zones.get_mut(owner_hand).unwrap().cards.push(first);
    let card = state.cards.get_mut(first).unwrap();
    card.zone = owner_hand;
    card.owner = owner;
    assert_eq!(
        engine_core::query::effective_card_number(&state, owner, state.cards.get(first).unwrap()),
        13
    );
}

#[test]
fn dianyi_uses_deterministic_random_gain_count_and_hp_threshold() {
    let (state, owner, order) = setup(5689);
    let state = assign(state, owner, 0, DIANYI, 5690);
    let state = assign(state, owner, 1, DIANYI, 5691);
    let state = assign(state, owner, 2, DIANYI, 5692);
    let mut tx = Transaction::begin(state);
    for serial in 10..=15 {
        let zone = if serial % 2 == 0 {
            ZoneKind::DiscardPile
        } else {
            ZoneKind::Hand
        };
        let card = inject(&mut tx, order[1], zone, serial);
        tx.state_mut().add_content_card_tag(card, SHEQUAN);
    }
    let dianyi = skill(tx.state(), owner, DIANYI);
    for action in 3..=5 {
        run_effects(&mut tx, vec![request(owner, CardId::NONE, action, dianyi)]).unwrap();
    }
    assert_eq!(tx.state().hand_count(owner), 6);
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        3,
        "典易状态={:?}",
        tx.state().skills.get(dianyi).unwrap().state_blob
    );
}

#[test]
fn nizun_collects_tagged_discard_once_per_turn_and_increases_damage() {
    let (state, owner, order) = setup(5693);
    let mut tx = Transaction::begin(state);
    let dianyi_skill = skill(tx.state(), owner, DIANYI);
    let jianzhuan_skill = skill(tx.state(), owner, JIANZHUAN);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill {
                skill: dianyi_skill,
            },
            EffectRequest::RemoveSkill {
                skill: jianzhuan_skill,
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NIZUN),
            },
        ],
    )
    .unwrap();
    let first = inject(&mut tx, PlayerId::NONE, ZoneKind::DiscardPile, 20);
    let second = inject(&mut tx, PlayerId::NONE, ZoneKind::DiscardPile, 21);
    tx.state_mut().add_content_card_tag(first, SHEQUAN);
    tx.state_mut().add_content_card_tag(second, SHEQUAN);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::card_used_by_player(owner), 1);
    tx.state_mut().turn.current = order[1];
    tx.state_mut().turn.phase = Phase::End;
    let nizun = skill(tx.state(), owner, NIZUN);
    run_effects(&mut tx, vec![request(owner, CardId::NONE, 9, nizun)]).unwrap();
    assert!(tx.state().playable_hand_cards(owner).contains(&first));
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == NIZUN)
        .unwrap()
        .zone;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![first],
            to: discard,
            from: Some(pile),
            reason: CardMoveReason::Use,
        }],
    )
    .unwrap();
    run_effects(&mut tx, vec![request(owner, first, 11, nizun)]).unwrap();
    assert!(!tx.state().playable_hand_cards(owner).contains(&second));
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: order[1],
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);
}
