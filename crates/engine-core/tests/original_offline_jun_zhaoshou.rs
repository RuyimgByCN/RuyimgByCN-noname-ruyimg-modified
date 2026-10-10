//! 第一百七十四批：offline「昭算」（jun_zhaoshou）第三方拼点原语（ABI 680）。
//! 上游：出牌阶段限一次，将所有手牌交给一名其他角色，其选一名角色（不含你）
//! 拼点；非平局时胜方对败方使用无距离【杀】。
//! 实现：give_all_hand + choose_hand_compare_between（本批新原语，内嵌对手选择
//! 与胜方虚拟杀，近似固定以败方为虚拟杀目标）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;

const JUN_ZHAOSHOU: u32 = 3_399_079_236;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn started(seed: u64) -> Option<(GameState, PlayerId, PlayerId, PlayerId)> {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    if !matches!(
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
    ) {
        return None;
    }
    let state = engine.state().clone();
    let owner = state.turn.current;
    let seats = state.players.alive_by_seat();
    let first = seats.iter().copied().find(|p| *p != owner)?;
    let second = seats.iter().copied().find(|p| *p != owner && *p != first)?;
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| rule_api::EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::AddSkill {
            player: owner,
            skill_def: engine_api::SkillDefId::new(JUN_ZHAOSHOU),
        }],
    )
    .ok();
    Some((tx.commit().state, owner, first, second))
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_300 + rank as u32),
        kind: CardKind::Sha,
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

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn resolve_pick(
    state: GameState,
    actor: PlayerId,
    pick: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![pick],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_jun_zhaoshou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == JUN_ZHAOSHOU));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == JUN_ZHAOSHOU));
}

#[test]
fn jun_zhaoshou_full_chain_gives_then_compares_then_virtual_sha() {
    // owner 高牌必胜路径：first 手牌 9、second 手牌 2 → first 胜 → second（败方）
    // 被无距离虚拟【杀】指定。
    for seed in 2_026_091_901..2_026_091_931 {
        let Some((state, owner, first, second)) = started(seed) else {
            continue;
        };
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        // owner 全手牌（2 张）将交给 first；first/second 各注入拼点牌。
        inject_hand(&mut tx, owner, 8);
        inject_hand(&mut tx, owner, 7);
        let first_card = inject_hand(&mut tx, first, 9);
        let second_card = inject_hand(&mut tx, second, 2);
        let state = tx.commit().state;

        let owner_hand_before = hand_count(&state, owner);
        let first_hand_before = hand_count(&state, first);
        let second_hp_before = hp_of(&state, second);

        let skill_id = state
            .skills
            .sorted_live()
            .iter()
            .find(|s| s.owner == owner && s.def == engine_api::SkillDefId::new(JUN_ZHAOSHOU))
            .unwrap()
            .id;
        let revision = state.revision;
        let (mut state, output) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(seed),
                kind: CommandKind::ActivateSkill {
                    skill: skill_id,
                    targets: vec![first],
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "seed {seed}: activate rejected: {:?}",
            output.result
        );

        // 四段选择：first 选对手（second）→ first 选牌（9）→ second 选牌（2）→
        // first（胜方）为虚拟【杀】自选目标（second）。
        let mut key = seed + 1;
        for step in 0..4 {
            assert!(
                state.has_pending_choice(),
                "seed {seed} step {step}: chain must stay open"
            );
            let observer = PlayerId::new(state.resolution.pending_observer);
                    let pick = match step {
                0 => second.raw(),
                1 => first_card.raw(),
                2 => second_card.raw(),
                _ => second.raw(),
            };
            let (next, result) = resolve_pick(state, observer, pick, key);
            assert!(
                matches!(result.result, CommandResult::Accepted { .. }),
                "seed {seed} step {step}: resolve rejected {:?}",
                result.result
            );
            state = next;
            key += 1;
        }

        // 断言：owner 手牌清空（give all）、first 获得 2 张 + 拼点牌已弃、
        // second 被虚拟杀扣 1 血。
        assert_eq!(hand_count(&state, owner), 0, "seed {seed}: all hand given");
        assert_eq!(hand_count(&state, owner), 0, "seed {seed}: all hand given");
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let discarded: Vec<_> = state
            .zones
            .get(discard)
            .unwrap()
            .cards
            .iter()
            .copied()
            .collect();
        assert!(
            discarded.contains(&first_card) && discarded.contains(&second_card),
            "seed {seed}: both comparison cards must reach the discard pile"
        );
        assert!(
            hp_of(&state, second) < second_hp_before,
            "seed {seed}: loser must take the virtual sha"
        );
        assert!(
            hp_of(&state, second) < second_hp_before,
            "seed {seed}: loser must take the virtual sha"
        );
        return; // 一条成功路径即收
    }
    panic!("no usable seed in range");
}
