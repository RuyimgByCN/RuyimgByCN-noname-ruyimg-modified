//! Golden coverage for BINGSHI 国渊「清蹈」「修耕」「陈赦」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const GUOYUAN: u32 = 2_786_098_854;
const MBQINGDAO: u32 = 2_885_098_893;
const MBXIUGENG: u32 = 3_797_581_537;
const MBCHENSHE: u32 = 3_265_153_315;
const MBXIUGENG_EFFECT: u32 = 4_053_015_127;
const MBXIUGENG_HANDCARD: u32 = 2_492_312_223;
const MBQUNCHONG: u32 = 3_613_650_732;
const MBQUNCHONG_EFFECT: u32 = 3_254_288_451;
const MBRENXING: u32 = 3_029_832_657;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32, player_count: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: u64::from(skill_def),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
    (tx, owner, others)
}

fn skill(state: &GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == player && skill.def.raw() == def)
        .unwrap()
        .id
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, offset: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(503_000 + offset),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_contains_guoyuan_at_abi509() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let guoyuan = package
        .character_index
        .iter()
        .find(|character| character.id == GUOYUAN)
        .unwrap();
    assert_eq!(guoyuan.skills, [MBQINGDAO, MBXIUGENG, MBCHENSHE]);
}

#[test]
fn qingdao_waits_for_the_damage_card_to_finish() {
    let (mut tx, owner, others) = setup(MBQINGDAO, 2);
    let attacker = others[0];
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, attacker);
    let sha = inject(&mut tx, attacker, CardKind::Sha, 1);
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let hp = tx.state().players.get(owner).unwrap().hp;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: attacker,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert!(state.has_pending_choice());
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    let state = resolve(state, vec![1], 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn xiugeng_equal_hand_count_applies_both_results() {
    let (mut tx, owner, others) = setup(MBXIUGENG, 2);
    let target = others[0];
    clear_hand(&mut tx, target);
    let owner_skill = skill(tx.state(), owner, MBXIUGENG);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxiugeng {
                player: owner,
                helper_def: SkillDefId::new(MBXIUGENG_EFFECT),
                hand_limit_def: SkillDefId::new(MBXIUGENG_HANDCARD),
                stage: 0,
                targets: Vec::new(),
                skill: owner_skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![target.raw()], 3);
    let helper = skill(&state, target, MBXIUGENG_EFFECT);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiugeng {
            player: target,
            helper_def: SkillDefId::new(MBXIUGENG_EFFECT),
            hand_limit_def: SkillDefId::new(MBXIUGENG_HANDCARD),
            stage: 1,
            targets: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(target), 2);
    assert!(tx.state().skills.get(helper).is_none());
    assert!(tx
        .state()
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == MBXIUGENG_HANDCARD && *count == 1));
}

#[test]
fn chenshe_same_suit_recovers_to_max_and_removes_itself() {
    let (mut tx, owner, others) = setup(MBCHENSHE, 3);
    let dying = others[0];
    let source = others[1];
    for player in [owner, dying, source] {
        clear_hand(&mut tx, player);
    }
    let cards = [
        inject(&mut tx, owner, CardKind::Sha, 2),
        inject(&mut tx, dying, CardKind::Shan, 3),
        inject(&mut tx, source, CardKind::Tao, 4),
    ];
    let max_hp = tx.state().players.get(dying).unwrap().max_hp;
    let dying_state = tx.state_mut().players.get_mut(dying).unwrap();
    dying_state.hp = 0;
    dying_state.dying = true;
    let skill = skill(tx.state(), owner, MBCHENSHE);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbchenshe {
                player: owner,
                dying,
                source,
                stage: 0,
                roles: Vec::new(),
                suits: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    for (key, card) in cards.into_iter().enumerate() {
        state = resolve(state, vec![card.raw()], 10 + key as u64);
    }
    assert_eq!(state.players.get(dying).unwrap().hp, max_hp as i32);
    assert!(state.skills.get(skill).is_none());
    assert!(!state.has_pending_choice());
}

#[test]
fn quanchong_discards_all_and_queues_penalized_extra_turn() {
    let (mut tx, owner, others) = setup(MBQUNCHONG, 2);
    let hp = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .hp
        .min(tx.state().players.get(others[0]).unwrap().hp);
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    tx.state_mut().players.get_mut(others[0]).unwrap().hp = hp;
    let skill = skill(tx.state(), owner, MBQUNCHONG);
    assert!(!tx.state().hand_cards(owner).is_empty());
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbquanchong {
            player: owner,
            helper_def: SkillDefId::new(MBQUNCHONG_EFFECT),
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).is_empty());
    assert_eq!(tx.state().turn.extra_turn_queue, [owner]);
    assert!(tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == owner && instance.def.raw() == MBQUNCHONG_EFFECT));
}

#[test]
fn renxing_draw_branch_counts_against_the_round_limit() {
    let (mut tx, owner, _) = setup(MBRENXING, 2);
    let skill = skill(tx.state(), owner, MBRENXING);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbrenxing {
                player: owner,
                stage: 0,
                target: PlayerId::NONE,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![1], 30);
    assert_eq!(state.hand_count(owner), before + 1);
    assert_eq!(
        u32::from_le_bytes(
            state.skills.get(skill).unwrap().state_blob[8..12]
                .try_into()
                .unwrap()
        ),
        1
    );
}
