//! 第一百六十六批：offline「探虎」（tanhu）拼点胜负挂钩引擎批（ABI 678）。
//! 新原语 choose_hand_compare_grant_target_on_win：发起者严格胜 → **目标**获挂
//! tanhu2（AddSkill 到目标侧）；负/平无操作。tanhu2 为上游无规则面的标记技
//! （mark/intro），诚实终态。多 seed 全链驱动覆盖胜/负两分支。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;

const TANHU: u32 = 2_881_975_017;
const TANHU2: u32 = 2_642_618_323;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn started_with_skill(seed: u64) -> Option<(GameState, PlayerId, PlayerId)> {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let _ = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });

    // T15③ roster 平移：排空开局挂起可选选择（qimen 模式）。
    let mut drain_key = seed ^ 0x7153_0000;
    while engine.state().has_pending_choice() {
        drain_key = drain_key.wrapping_add(1);
        let choice_id = engine.state().resolution.pending_choice_id;
        let revision = engine.state().revision;
        let observer = engine.state().resolution.pending_observer;
        let _ = engine.submit(Command {
            actor: PlayerId::new(observer),
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(drain_key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: vec![],
                cancelled: true,
            },
        });
    }
    if engine.state().players.alive_by_seat().len() < 2 {
        return None;
    }
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| rule_api::EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    // offline 包存在 removeSkill 反应技，ChoiceRequested 属预期控制流。
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    let _ = engine_core::effect::run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TANHU),
        }],
    );
    assert!(
        tx.state()
            .skills
            .sorted_live()
            .iter()
            .any(|s| s.owner == owner && s.def == SkillDefId::new(TANHU)),
        "tanhu must be armed despite reaction choices"
    );
    // 直接把双方手牌移入弃牌堆（同步 card.zone，DiscardFromHand 对 bootstrap 牌有保护口径）。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for player in [owner, other] {
        if let Some(hand) = tx.state().zones.find(ZoneKind::Hand, player) {
            let cards = tx.state().zones.get(hand).unwrap().cards.clone();
            for card in cards {
                tx.state_mut().cards.get_mut(card).unwrap().zone = discard;
                tx.state_mut().cards.get_mut(card).unwrap().owner = PlayerId::NONE;
                tx.state_mut()
                    .zones
                    .get_mut(hand)
                    .unwrap()
                    .cards
                    .retain(|c| *c != card);
                tx.state_mut()
                    .zones
                    .get_mut(discard)
                    .unwrap()
                    .cards
                    .push(card);
            }
        }
    }
    let mut state = tx.commit().state;
    // 清掉 setup 期间残留的反应技选择（cancel 直至干净）。
    let mut guard_key = seed + 999;
    while state.has_pending_choice() {
        guard_key += 1;
        let cid = ChoiceId::new(state.resolution.pending_choice_id);
        let actor = PlayerId::new(state.resolution.pending_observer);
        let revision = state.revision;
        let (next, _) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(guard_key),
                kind: CommandKind::ResolveChoice {
                    choice_id: cid,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        state = next;
    }
    Some((state, owner, other))
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(99_000 + rank as u32),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.owner == player && s.def == SkillDefId::new(def))
}

fn resolve_pick(
    state: GameState,
    actor: PlayerId,
    pick: u32,
    key: u64,
) -> (GameState, PipelineResult) {
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
fn package_publishes_tanhu_grant_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package.skill_index.iter().any(|s| s.skill_def_id == TANHU));
    assert!(package.skill_index.iter().any(|s| s.skill_def_id == TANHU2));
}

#[test]
fn tanhu_full_chain_grants_marker_on_win_only_to_target() {
    let mut saw_win = false;
    let mut saw_lose = false;
    for seed in 2_026_091_201..2_026_091_280 {
        let Some((state, owner, other)) = started_with_skill(seed) else {
            continue;
        };
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        // 双方手牌点数确定：owner 10，other 3（owner 必胜）。
        inject_hand(&mut tx, owner, 10);
        inject_hand(&mut tx, other, 3);
        let state = tx.commit().state;

        let skill_id = state
            .skills
            .sorted_live()
            .iter()
            .find(|s| s.owner == owner && s.def == SkillDefId::new(TANHU))
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
                    targets: vec![other],
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "seed {seed}: activate rejected: {:?}",
            output.result
        );

        // 四段选择：选目标（other）→ 发起者选牌（唯一）→ 目标选牌（唯一）。
        let mut key = seed + 1;
        for step in 0..3 {
            assert!(
                state.has_pending_choice(),
                "seed {seed} step {step}: chain must stay open"
            );
            let observer = PlayerId::new(state.resolution.pending_observer);
            // 每段只有一个合法选项（单目标/唯一注入牌）——依次选第一项。
            // 选项 token 无法直接枚举：目标段选 other.raw()，选牌段选注入牌。
            let pick = match step {
                0 => other.raw(),
                1 => {
                    // 发起者唯一手牌
                    state
                        .zones
                        .find(ZoneKind::Hand, owner)
                        .and_then(|z| state.zones.get(z))
                        .map(|z| z.cards[0].raw())
                        .unwrap()
                }
                _ => state
                    .zones
                    .find(ZoneKind::Hand, other)
                    .and_then(|z| state.zones.get(z))
                    .map(|z| z.cards[0].raw())
                    .unwrap(),
            };
            let (next, result) = resolve_pick(state, observer, pick, key);
            assert!(
                matches!(result.result, CommandResult::Accepted { .. }),
                "seed {seed} step {step}: resolve rejected"
            );
            state = next;
            key += 1;
        }

        let owner_has = has_skill(&state, owner, TANHU2);
        let other_has = has_skill(&state, other, TANHU2);
        if !(other_has && !owner_has) {
            let owner_ranks: Vec<_> = state
                .hand_cards(owner)
                .iter()
                .filter_map(|c| state.cards.get(*c).map(|card| card.rank))
                .collect();
            let other_ranks: Vec<_> = state
                .hand_cards(other)
                .iter()
                .filter_map(|c| state.cards.get(*c).map(|card| card.rank))
                .collect();
            panic!(
                "seed {seed}: owner_has={owner_has} other_has={other_has} owner_ranks={owner_ranks:?} other_ranks={other_ranks:?}"
            );
        }
        saw_win = true;
    }
    assert!(saw_win);
}

#[test]
fn tanhu_lose_branch_grants_nothing() {
    // owner 出 2，other 出 K(13)：发起者必负。
    let mut picked = None;
    for seed in 2_026_091_301..2_026_091_360 {
        if let Some(tuple) = started_with_skill(seed) {
            picked = Some((seed, tuple));
            break;
        }
    }
    let (seed, (state, owner, other)) = picked.expect("usable tanhu lose-branch seed");
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    inject_hand(&mut tx, owner, 2);
    inject_hand(&mut tx, other, 13);
    let state = tx.commit().state;

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(TANHU))
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
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        output.result
    );

    let mut key = seed + 1;
    for step in 0..3 {
        let observer = PlayerId::new(state.resolution.pending_observer);
        let pick = match step {
            0 => other.raw(),
            1 => state
                .zones
                .find(ZoneKind::Hand, owner)
                .and_then(|z| state.zones.get(z))
                .map(|z| z.cards[0].raw())
                .unwrap(),
            _ => state
                .zones
                .find(ZoneKind::Hand, other)
                .and_then(|z| state.zones.get(z))
                .map(|z| z.cards[0].raw())
                .unwrap(),
        };
        let (next, result) = resolve_pick(state, observer, pick, key);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
        key += 1;
    }

    assert!(
        !has_skill(&state, owner, TANHU2) && !has_skill(&state, other, TANHU2),
        "loss must grant nothing to either side"
    );
}
