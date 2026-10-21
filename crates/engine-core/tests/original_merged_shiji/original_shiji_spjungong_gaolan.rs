//! Golden for SHIJI 高览（sp_gaolan）与「峻攻」弃牌/失体力/命中禁用语义（ABI 400）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SP_GAOLAN: u32 = 2_850_957_074;
const SPJUNGONG: u32 = 3_803_073_466;
const SPDANGLI: u32 = 3_419_124_885;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    for c in cards {
        engine_core::effect::move_one_card(tx, c, hand, discard, PlayerId::NONE, false).unwrap();
    }
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn spjungong_banned(state: &GameState, owner: PlayerId) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .filter_map(|skill| state.skills.get(*skill))
        .find(|instance| instance.def == SkillDefId::new(SPJUNGONG))
        .map_or(false, |instance| instance.active_turn_banned)
}

fn resolve_choice(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_closes_sp_gaolan_with_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 400);

    // 高览闭包包含「峻攻」「登历」且顺序完全一致。
    let character = package
        .character_index
        .iter()
        .find(|c| c.id == SP_GAOLAN)
        .expect("sp_gaolan character index entry");
    assert_eq!(character.name_key, "sp_gaolan");
    assert_eq!(character.skills, vec![SPJUNGONG, SPDANGLI]);
    assert_eq!(character.max_hp, 4);

    // 峻攻以 ABI 400 声明主动技与命中禁用触发器。
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == SPJUNGONG)
        .expect("spjungong damage trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_source_sha_damage_recorded_by_skill");
}

#[test]
fn spjungong_discards_card_uses_sha_and_bans_active_on_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_801,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;

    // 清空两人手牌，确保无法闪避。
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();

    let skill_id = add_skill(&mut tx, owner, SPJUNGONG);

    // 初始未禁用。
    assert!(!spjungong_banned(tx.state(), owner));

    // 给 owner 注入恰好 1 张成本手牌。
    let card = inject_hand(&mut tx, owner, CardKind::Tao, 5);

    // 发动峻攻：选择 1 张手牌弃置，对 target 使用虚拟【杀】。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::SpjungongDiscardOrLoseThenSha {
            player: owner,
            target,
            skill: skill_id,
        }],
    )
    .unwrap_err();

    // 应该弹出 SelectCards 屏障（选 0..1 张牌）。
    assert!(matches!(err, EffectError::ChoiceRequested));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseSpjungongDiscardOrLose
    );

    // 选择弃置 1 张手牌。
    let (state, result) = resolve_choice(tx.state().clone(), vec![card.raw()], false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    // 确认手牌已被弃置。
    assert_eq!(state.hand_count(owner), 0);

    // 确认峻攻因虚拟【杀】命中伤害而被标记为本回合禁用。
    assert!(spjungong_banned(&state, owner));
}

#[test]
fn spjungong_loses_hp_when_no_cards_selected() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_802,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;

    // 清空两人手牌，确保无法闪避。
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();

    let skill_id = add_skill(&mut tx, owner, SPJUNGONG);

    let initial_hp = tx.state().players.get(owner).unwrap().hp;

    // 发动峻攻（n=1）。
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::SpjungongDiscardOrLoseThenSha {
            player: owner,
            target,
            skill: skill_id,
        }],
    );

    // 选择 0 张牌 → 扣 1 点体力。
    let (state, result) = resolve_choice(tx.state().clone(), vec![], false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    assert_eq!(state.players.get(owner).unwrap().hp, initial_hp - 1);
    assert!(spjungong_banned(&state, owner));
}
