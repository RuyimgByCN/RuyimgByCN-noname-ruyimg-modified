//! Golden coverage for SHENHUA「烈弓」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIEGONG: u32 = 2_510_589_015;
const TEST_MULTI_SHA: u32 = 2_510_589_016;
const HUANGZHONG: u32 = 3_528_195_720;
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/shenhua/liegong.skill"),
    "\nskill test_multi_sha(2510589016) locked { modifier sha_target_limit add 1; }\n"
);

fn setup(player_count: u8) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("liegong-test", SOURCE).unwrap();
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
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIEGONG),
    });
    if player_count > 2 {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TEST_MULTI_SHA),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, seats)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn add_target_hand(tx: &mut Transaction, target: PlayerId, count: usize) -> CardId {
    assert!(count > 0);
    let shan = inject(tx, target, ZoneKind::Hand, CardKind::Shan, 93_000);
    for index in 1..count {
        inject(
            tx,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            93_000 + index as u32,
        );
    }
    shan
}

fn play_sha(
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult, CardId) {
    let mut tx = Transaction::begin(state);
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        94_000 + key as u32,
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets,
                card_targets: vec![],
            },
        },
    );
    (state, result, sha)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
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
    )
}

#[test]
fn packages_publish_exact_skill_character_and_dependency() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(shenhua.manifest.rule_vm_abi_version, 671);
    let trigger = shenhua
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIEGONG)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_target_hand_gte_hp_or_lte_attack_range"
    );
    assert!(trigger.optional);

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert!(old.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shenhua"
            && dependency.version == "0.1.0"
    }));
    let huangzhong = old
        .character_index
        .iter()
        .find(|character| character.id == HUANGZHONG)
        .unwrap();
    assert_eq!(huangzhong.name_key, "huangzhong");
    assert_eq!(huangzhong.max_hp, 4);
    assert_eq!(huangzhong.skills, [LIEGONG]);
}

#[test]
fn eligible_target_can_be_locked_or_left_to_play_shan() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let shan = add_target_hand(&mut tx, target, 1);
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, offered, _) = play_sha(tx.commit().state, owner, vec![target], 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);

    let (locked, result) = resolve(state.clone(), owner, vec![1], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(locked.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(locked.hand_count(target), 1);

    let (declined, result) = resolve(state, owner, vec![], true, 3);
    let response = result.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    let (after_shan, result) = resolve(declined, target, vec![shan.raw()], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(after_shan.players.get(target).unwrap().hp, hp);
}

#[test]
fn filter_uses_both_hp_and_current_attack_range_boundaries() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let shan = add_target_hand(&mut tx, target, 2);
    let (_state, result, _) = play_sha(tx.commit().state, owner, vec![target], 10);
    let response = result.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    add_target_hand(&mut tx, target, 4);
    let (_, result, _) = play_sha(tx.commit().state, owner, vec![target], 11);
    assert_eq!(result.choice.unwrap().kind, ChoiceKind::Confirm);

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    add_target_hand(&mut tx, target, 2);
    inject(&mut tx, owner, ZoneKind::Equip, CardKind::Weapon, 93_100);
    let (_, result, _) = play_sha(tx.commit().state, owner, vec![target], 12);
    assert_eq!(result.choice.unwrap().kind, ChoiceKind::Confirm);
}

#[test]
fn multi_target_sha_locks_only_the_accepted_current_target() {
    let (state, owner, seats) = setup(3);
    let (locked_target, response_target) = (seats[1], seats[2]);
    let mut tx = Transaction::begin(state);
    add_target_hand(&mut tx, locked_target, 1);
    let shan = add_target_hand(&mut tx, response_target, 2);
    let locked_hp = tx.state().players.get(locked_target).unwrap().hp;
    let response_hp = tx.state().players.get(response_target).unwrap().hp;
    let (state, result, _) = play_sha(
        tx.commit().state,
        owner,
        vec![locked_target, response_target],
        20,
    );
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![1], false, 21);
    let response = result.choice.unwrap();
    assert_eq!(state.players.get(locked_target).unwrap().hp, locked_hp - 1);
    assert_eq!(state.players.get(response_target).unwrap().hp, response_hp);
    assert_eq!(response.observer, response_target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
}

#[test]
fn intent_rejects_a_stale_or_mismatched_sha_tuple() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 95_000);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MakeCurrentShaTargetUnrespondable {
                source: owner,
                card,
                target,
            }],
        ),
        Err(EffectError::Illegal(
            "no matching currently resolving Sha target".into()
        ))
    );
}
