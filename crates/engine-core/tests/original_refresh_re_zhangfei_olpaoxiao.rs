//! Golden coverage for REFRESH「界张飞·咆哮」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLPAOXIAO: u32 = 2_829_854_661;
const OLPAOXIAO2: u32 = 3_946_229_744;
const OLTISHEN: u32 = 3_850_927_603;
const RE_ZHANGFEI: u32 = 2_213_648_390;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/olpaoxiao.skill"),
    include_str!("../../../content/original/ported/refresh/olpaoxiao2.skill"),
);

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("olpaoxiao-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_721, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
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
        skill_def: SkillDefId::new(OLPAOXIAO),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in seats {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, target)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
}

fn resolve_shan(
    state: GameState,
    target: PlayerId,
    shan: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: target,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    )
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn mark_count(state: &GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_publishes_complete_re_zhangfei_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHANGFEI)
        .unwrap();
    assert_eq!(character.name_key, "re_zhangfei");
    assert_eq!(character.skills, [OLPAOXIAO, OLTISHEN]);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OLPAOXIAO2
            && modifier.target == "damage_amount"
            && modifier.selector.as_deref() == Some("sha")
            && modifier.value_source.as_deref() == Some("owner_skill_mark")
    }));
}

#[test]
fn each_full_dodge_adds_one_mark_and_next_sha_damage_consumes_all_marks() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha1 = inject(&mut tx, owner, CardKind::Sha, 97_000);
    let sha2 = inject(&mut tx, owner, CardKind::Sha, 97_001);
    let sha3 = inject(&mut tx, owner, CardKind::Sha, 97_002);
    let sha4 = inject(&mut tx, owner, CardKind::Sha, 97_003);
    let shan1 = inject(&mut tx, target, CardKind::Shan, 97_004);
    let shan2 = inject(&mut tx, target, CardKind::Shan, 97_005);
    let state = tx.commit().state;

    assert_eq!(
        engine_core::query::sha_use_limit_for(&state, owner),
        i32::MAX as u32
    );
    let (state, result) = play(state, owner, sha1, target, 1);
    assert!(result.choice.is_some());
    let (state, result) = resolve_shan(state, target, shan1, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, owner, OLPAOXIAO2), 1);
    assert!(has_skill(&state, owner, OLPAOXIAO2));

    let (state, result) = play(state, owner, sha2, target, 3);
    assert!(result.choice.is_some());
    let (state, result) = resolve_shan(state, target, shan2, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, owner, OLPAOXIAO2), 2);

    let hp = state.players.get(target).unwrap().hp;
    let (state, result) = play(state, owner, sha3, target, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 3);
    assert_eq!(mark_count(&state, owner, OLPAOXIAO2), 0);
    assert!(!has_skill(&state, owner, OLPAOXIAO2));

    let hp = state.players.get(target).unwrap().hp;
    let (state, result) = play(state, owner, sha4, target, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn unused_marks_are_cleared_when_the_owners_turn_ends() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, CardKind::Sha, 97_010);
    let shan = inject(&mut tx, target, CardKind::Shan, 97_011);
    let (state, _) = play(tx.commit().state, owner, sha, target, 10);
    let (state, _) = resolve_shan(state, target, shan, 11);
    assert_eq!(mark_count(&state, owner, OLPAOXIAO2), 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: target,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), owner, OLPAOXIAO2), 0);
    assert!(!has_skill(tx.state(), owner, OLPAOXIAO2));
}
