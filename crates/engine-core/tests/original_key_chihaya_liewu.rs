//! Golden for KEY「烈武」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::query::{
    card_distance_unlimited_for_card, card_target_limit_for_kind, card_use_unlimited_for_card,
};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const LIEWU: u32 = 3_721_191_175;
const LIEWU_HELPER: u32 = 2_978_088_269;
const HUAIROU: u32 = 3_695_539_048;
const CHIHAYA: u32 = 4_054_734_317;
const LIEWU_SOURCE: &str = include_str!("../../../content/original/ported/key/chihaya_liewu.skill");
const RUNTIME_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/key/chihaya_liewu.skill"),
    include_str!("../../../content/original/ported/key/chihaya_huairou.skill")
);
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn liewu_package() -> content_schema::ContentPackage {
    content_compiler::compile_source_named("key-liewu", LIEWU_SOURCE).unwrap()
}

fn isolated(player_count: u8) -> (GameState, PlayerId, SkillId) {
    let package =
        content_compiler::compile_source_named("key-liewu-runtime", RUNTIME_SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIEWU),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LIEWU)
        .unwrap();
    (tx.commit().state, owner, skill)
}

fn command(actor: PlayerId, key: u64, kind: CommandKind) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(7_100 + kind.tag() as u32 * 20 + rank as u32),
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

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let players: Vec<_> = tx
        .state()
        .players
        .iter_live()
        .map(|player| player.id)
        .collect();
    for player in players {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
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
    }
}

fn pass_wuxie(
    mut state: GameState,
    mut pipeline: PipelineResult,
    key: &mut u64,
) -> (GameState, PipelineResult) {
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipeline.choice.clone().expect("wuxie choice");
        *key += 1;
        (state, pipeline) = apply_command(
            state,
            command(
                choice.observer,
                *key,
                CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            ),
        );
        assert!(
            matches!(pipeline.result, CommandResult::Accepted { .. }),
            "{:?}",
            pipeline.result
        );
    }
    (state, pipeline)
}

fn resolve_target_card(
    state: GameState,
    pipeline: PipelineResult,
    expected: CardId,
    key: &mut u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.expect("target-card choice");
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == expected.raw()));
    *key += 1;
    let (state, pipeline) = apply_command(
        state,
        command(
            choice.observer,
            *key,
            CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![expected.raw()],
                cancelled: false,
            },
        ),
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipeline.result
    );
    (state, pipeline)
}

#[test]
fn committed_package_declares_liewu_helper_character_and_package_abi() {
    let compiled = liewu_package();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 19);
    assert_eq!(compiled.skill_index.len(), 2);
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LIEWU_HELPER));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == LIEWU
            && modifier.target == "card_target_limit"
            && modifier.selector.as_deref() == Some("ordinary_trick")
            && modifier.mode == "add"
            && modifier.value == 1
    }));
    assert_eq!(
        compiled.trigger_index[0].filter,
        "owner_is_event_player_no_enabled_equip_slot_once_per_game"
    );

    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    // 烈武本身最低 ABI 为 19；聚合 KEY 包 ABI 取包内最高（旅行箱 164、取用 651）。
    assert_eq!(committed.manifest.rule_vm_abi_version, 691);
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "chihaya_liewu_equip_slot_disabled")
        .unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == CHIHAYA)
        .unwrap();
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "key");
    assert_eq!(character.skills, vec![LIEWU, 2_842_065_522]);
}

#[test]
fn sha_is_unlimited_and_ordinary_tricks_gain_one_optional_target() {
    let (state, owner, _) = isolated(4);
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 7);
    let guohe = inject(&mut tx, owner, CardKind::TrickGuohe, Suit::Club, 3);
    assert!(card_use_unlimited_for_card(tx.state(), owner, sha));
    assert!(card_distance_unlimited_for_card(tx.state(), owner, sha));
    assert_eq!(
        card_target_limit_for_kind(tx.state(), owner, CardKind::TrickGuohe, 1),
        2
    );

    let targets: Vec<_> = tx
        .state()
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    let first_card = inject(&mut tx, targets[0], CardKind::Shan, Suit::Heart, 2);
    let second_card = inject(&mut tx, targets[1], CardKind::Shan, Suit::Diamond, 4);
    inject(&mut tx, targets[2], CardKind::Shan, Suit::Club, 6);
    let state = tx.commit().state;

    let (_, rejected) = apply_command(
        state.clone(),
        command(
            owner,
            1,
            CommandKind::PlayCard {
                card: guohe,
                targets: targets.clone(),
                card_targets: vec![],
            },
        ),
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, pipeline) = apply_command(
        state,
        command(
            owner,
            2,
            CommandKind::PlayCard {
                card: guohe,
                targets: targets[..2].to_vec(),
                card_targets: vec![],
            },
        ),
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let mut key = 2;
    let (state, pipeline) = pass_wuxie(state, pipeline, &mut key);
    let (state, pipeline) = resolve_target_card(state, pipeline, first_card, &mut key);
    let (state, pipeline) = pass_wuxie(state, pipeline, &mut key);
    let (state, pipeline) = resolve_target_card(state, pipeline, second_card, &mut key);
    assert!(!state.has_pending_choice(), "{:?}", pipeline.choice);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(first_card).unwrap().zone, discard);
    assert_eq!(state.cards.get(second_card).unwrap().zone, discard);
}

#[test]
fn first_last_slot_disable_loses_max_hp_adds_huairou_once_and_never_repeats() {
    let (state, owner, liewu) = isolated(2);
    let mut tx = Transaction::begin(state);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 8;
        player.hp = 8;
    }
    let disabled = run_effects(
        &mut tx,
        vec![EffectRequest::DisableEquipSlots {
            player: owner,
            slots: vec![1, 2, 3, 4, 5],
        }],
    )
    .unwrap();
    assert_eq!(
        disabled
            .iter()
            .filter(|event| matches!(event, DomainEvent::EquipSlotDisabled { .. }))
            .count(),
        5
    );
    assert_eq!(tx.state().skills.get(liewu).unwrap().lifetime_fire_count, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 4);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    let huairou_count = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .filter(|skill| tx.state().skills.get(**skill).unwrap().def.raw() == HUAIROU)
        .count();
    assert_eq!(huairou_count, 1);

    let repeated = dispatch_for_events(&mut tx, &disabled, &mut TriggerFireRecord::new()).unwrap();
    assert!(repeated.is_empty());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HUAIROU),
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .filter(|skill| tx.state().skills.get(**skill).unwrap().def.raw() == HUAIROU)
            .count(),
        1
    );
}
