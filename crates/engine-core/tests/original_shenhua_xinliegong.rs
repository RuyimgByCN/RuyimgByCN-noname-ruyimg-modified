//! Golden coverage for SHENHUA「新烈弓」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINLIEGONG: u32 = 3_272_111_610;
const RE_HUANGZHONG: u32 = 2_306_877_807;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/xinliegong.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup(player_count: u8) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("xinliegong-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_727, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(XINLIEGONG),
    });
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

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play_sha(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    rank: u8,
    key: u64,
) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, CardKind::Sha, rank, 95_000 + key as u32);
    let state = tx.commit().state;
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
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
fn package_publishes_re_huangzhong_and_exact_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_HUANGZHONG
            && character.name_key == "re_huangzhong"
            && character.skills == [XINLIEGONG]
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINLIEGONG
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner"
            && trigger.optional
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == XINLIEGONG
            && modifier.target == "sha_distance_unlimited_if_target_distance_lte_rank"
    }));
}

#[test]
fn sha_rank_unlocks_only_targets_within_that_final_distance() {
    let (state, owner, seats) = setup(4);
    let far = seats[2];
    let (_, rejected) = play_sha(state, owner, far, 1, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, owner, seats) = setup(4);
    let far = seats[2];
    let (_, accepted) = play_sha(state, owner, far, 2, 2);
    assert_eq!(accepted.choice.unwrap().observer, owner);
}

#[test]
fn accepted_trigger_applies_each_current_target_branch_independently() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    inject(&mut tx, owner, CardKind::Other, 3, 96_001);
    inject(&mut tx, target, CardKind::Shan, 4, 96_002);
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, offered) = play_sha(tx.commit().state, owner, target, 7, 10);
    assert_eq!(offered.choice.unwrap().observer, owner);
    let (locked, resolved) = resolve(state, owner, vec![1], false, 11);
    assert!(resolved.choice.is_none());
    assert_eq!(locked.players.get(target).unwrap().hp, hp - 1);

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    inject(&mut tx, target, CardKind::Shan, 5, 96_003);
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, offered) = play_sha(tx.commit().state, owner, target, 7, 20);
    assert_eq!(offered.choice.unwrap().observer, owner);
    let (state, response) = resolve(state, owner, vec![1], false, 21);
    assert_eq!(response.choice.unwrap().observer, target);
    let (damaged, resolved) = resolve(state, target, vec![], true, 22);
    assert!(resolved.choice.is_none());
    assert_eq!(damaged.players.get(target).unwrap().hp, hp - 2);

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    inject(&mut tx, owner, CardKind::Other, 6, 96_004);
    inject(&mut tx, target, CardKind::Shan, 7, 96_005);
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, offered) = play_sha(tx.commit().state, owner, target, 7, 30);
    assert_eq!(offered.choice.unwrap().observer, owner);
    let (damaged, resolved) = resolve(state, owner, vec![1], false, 31);
    assert!(resolved.choice.is_none());
    assert_eq!(damaged.players.get(target).unwrap().hp, hp - 2);

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let shan = inject(&mut tx, target, CardKind::Shan, 8, 96_006);
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    let (_, response) = play_sha(tx.commit().state, owner, target, 7, 40);
    let choice = response.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
}
