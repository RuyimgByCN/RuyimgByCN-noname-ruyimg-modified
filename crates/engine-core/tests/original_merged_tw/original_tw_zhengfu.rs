//! Golden coverage for TW 凯撒「征服」.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHENGFU: u32 = 4_010_507_840;
const KAISA: u32 = 3_063_102_378;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/zhengfu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    shan: CardId,
    tao: CardId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_360_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("zhengfu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 360);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHENGFU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 7);
    let shan = inject(&mut tx, target, CardKind::Shan, Suit::Club, 8);
    let tao = inject(&mut tx, target, CardKind::Tao, Suit::Heart, 9);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        sha,
        shan,
        tao,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play_and_choose_basic(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, result) = submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    );
    // 当前实现：征赋在打杀后直接打开基本牌类别选择（无独立确认段）。
    let category = result.choice.unwrap();
    assert_eq!(category.kind, ChoiceKind::Custom);
    assert_eq!(
        category
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: category.choice_id,
            selections: vec![1],
            cancelled: false,
        },
        key + 1,
    )
}

#[test]
fn matching_gift_keeps_shan_live_while_decline_makes_sha_unrespondable() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 360);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == KAISA)
            .unwrap()
            .skills,
        [ZHENGFU]
    );

    let first = setup(20_260_801);
    let hp = first.state.players.get(first.target).unwrap().hp;
    let (state, result) = play_and_choose_basic(&first, 1);
    let gift = result.choice.unwrap();
    assert_eq!(gift.observer, first.target);
    assert_eq!(gift.kind, ChoiceKind::SelectCards);
    assert!(gift
        .options
        .iter()
        .any(|option| option.token == first.tao.raw()));
    let (state, result) = submit(
        state,
        first.target,
        CommandKind::ResolveChoice {
            choice_id: gift.choice_id,
            selections: vec![first.tao.raw()],
            cancelled: false,
        },
        4,
    );
    assert!(state.hand_cards(first.owner).contains(&first.tao));
    let shan = result.choice.unwrap();
    assert!(shan
        .options
        .iter()
        .any(|option| option.token == first.shan.raw()));
    let (state, result) = submit(
        state,
        first.target,
        CommandKind::ResolveChoice {
            choice_id: shan.choice_id,
            selections: vec![first.shan.raw()],
            cancelled: false,
        },
        5,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first.target).unwrap().hp, hp);

    let setup = setup(20_260_802);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, result) = play_and_choose_basic(&setup, 10);
    let gift = result.choice.unwrap();
    let (state, result) = submit(
        state,
        setup.target,
        CommandKind::ResolveChoice {
            choice_id: gift.choice_id,
            selections: Vec::new(),
            cancelled: true,
        },
        13,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
}
