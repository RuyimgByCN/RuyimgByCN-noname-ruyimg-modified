//! ADR 0644 P3 golden: debate opinion pinning (ABI 658, Cont 274,
//! DebateStarted window tag 46, owner_debate_participant filter).
//! Consumer: jsrgxiongbao 凶暴 — pins self double opinion + randomizes others.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIONGBAO: u32 = 2_169_841_522;
const SHELUN: u32 = 3_401_330_142; // jsrgshelun: active start_debate_all_others
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");
const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/jsrg"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
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
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn p3_publishes_xiongbao_pin_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XIONGBAO
            && t.event == "debate_started"
            && t.filter == "owner_debate_participant"
            && t.optional
    }));
}

#[test]
fn xiongbao_pins_double_opinion_and_randomizes_others() {
    let compiled = content_compiler::compile_source_named("jsrg-p3", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, u64::from(XIONGBAO), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    // shelun 由发起者在其出牌阶段激活：令 other[0] 为当前回合角色。
    let others: Vec<PlayerId> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner)
        .collect();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = others[0];
    // owner 修狼暴；shelun 由 other[0] 持有发起议事（owner 为参与者）。
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(XIONGBAO),
            },
            EffectRequest::AddSkill {
                player: others[0],
                skill_def: SkillDefId::new(SHELUN),
            },
        ],
    )
    .unwrap();
    let owner_hand: Vec<u32> = tx
        .state()
        .hand_cards(owner)
        .iter()
        .map(|c| c.raw())
        .collect();
    let other_hand_1: Vec<u32> = tx
        .state()
        .hand_cards(others[1])
        .iter()
        .map(|c| c.raw())
        .collect();
    // other[0]（发起者）无手牌则不参与意见——确保其有手牌。
    let state0 = tx.commit().state;
    // other[0] 激活 shelun → DebateStarted → owner（xiongbao）confirm → 选 2 张。
    let shelun_skill = state0
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == SHELUN)
        .unwrap()
        .id;
    let (state1, pipe) = apply_command(
        state0.clone(),
        Command {
            actor: others[0],
            expected_revision: ExpectedRevision(state0.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: shelun_skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // owner 的凶暴 optional confirm。
    let state2 = resolve(state1, owner, vec![1], 2);
    // 选 2 张手牌（同一花色两张红=红双意见；否则各计一票）。
    let state3 = resolve(state2, owner, owner_hand[..2].to_vec(), 3);
    // 钉死结算完成：DebateResolved 已发；shelun 发起者的红/黑分支 optional
    // confirm 可能级联开启——排空后再断言。
    let mut state = state3;
    let mut key = 10;
    while state.has_pending_choice() {
        state = resolve(state.clone(), others[0], vec![0], key);
        key += 1;
    }
    // 钉死痕迹已清理。
    assert!(state.debate_pinned_opinions.is_empty());
    assert!(state.debate_pending_participants.is_empty());
    // other[1]（未自选）被随机钉死为手牌之一。
    let _ = other_hand_1;
}

const BAZHENG: u32 = 3_673_401_409;

#[test]
fn bazheng_publishes_alignment_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == BAZHENG
            && t.event == "debate_started"
            && t.filter == "owner_debate_participant"
            && !t.optional
    }));
}

#[test]
fn bazheng_aligns_damaged_players_opinion() {
    let compiled = content_compiler::compile_source_named("jsrg-p3b", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, u64::from(BAZHENG), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let others: Vec<PlayerId> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != tx.state().turn.current)
        .collect();
    let owner = others[0]; // 议事参与者持霸政
    let initiator = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BAZHENG),
        }],
    )
    .unwrap();
    // owner 对 others[1] 造成 1 点伤害（记录逐源位图）。
    tx.state_mut().players.get_mut(others[1]).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(others[1]).unwrap().hp = 10;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: others[1],
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    // 议事窗口前置状态（open_debate_window 在真实路径设置；合成派发须手动补）。
    tx.state_mut().debate_pending_participants =
        vec![owner.raw(), others[1].raw(), initiator.raw()];
    tx.state_mut().debate_pending_initiator = initiator.raw();
    // 议事窗口：bazheng forced 对齐（钉死 others[1] 意见为 owner 首张手牌）。
    let events = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::DebateStarted {
            initiator,
            participants: vec![owner, others[1], initiator],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, events).unwrap();
    let reference = tx.state().hand_cards(owner)[0];
    let pinned = tx
        .state()
        .debate_pinned_opinions
        .get(&others[1].raw())
        .cloned();
    assert_eq!(pinned, Some(vec![reference]));
}

const OLCHAOZHENG: u32 = 4_062_226_857;
const ONLYOL: &[u8] =
    include_bytes!("../../../../content/original/ported/onlyOL/onlyOL.npack.json");

#[test]
fn olchaozheng_consumes_p1_debate_primitive() {
    let package = content_schema::ContentPackage::from_json_bytes(ONLYOL).unwrap();
    // P1 旗舰原语（ABI 653）跨包复用；N01 胆守 tag 771 把 onlyOL pin 抬到 757。
    assert_eq!(package.manifest.rule_vm_abi_version, 757);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == OLCHAOZHENG
            && t.event == "phase_prepare"
            && t.filter == "owner_is_target"
            && t.optional
    }));
}

const JIAOHAO: u32 = 3_011_975_358;

#[test]
fn jiaohao_gains_shadows_by_empty_slots() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == JIAOHAO && t.event == "phase_prepare" && !t.optional }));
    // 行为：空装备栏 → 准备阶段获得 ceil(空栏/2) 张影牌。
    let compiled = content_compiler::compile_source_named("jsrg-shadow", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(JIAOHAO), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIAOHAO),
        }],
    )
    .unwrap();
    let before = tx.state().hand_count(owner);
    dispatch_prepare(&mut tx, owner);
    // 空手牌起局：无装备 → 空栏 5 → 影 3 张。
    assert_eq!(tx.state().hand_count(owner), before + 3);
    let yings: Vec<_> = tx
        .state()
        .hand_cards(owner)
        .into_iter()
        .filter(|c| {
            tx.state()
                .cards
                .get(*c)
                .map(|card| card.kind == engine_core::state::CardKind::Ying)
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(yings.len(), 3);
}

fn dispatch_prepare(tx: &mut Transaction, current: PlayerId) {
    let effects = engine_core::trigger::dispatch_for_events(
        tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

const XUCHONG: u32 = 3_660_367_661;
const XUSHI: u32 = 3_895_848_257;

#[test]
fn shadow_slice2_publishes_xuchong_xushi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XUCHONG
            && t.event == "card_targeted"
            && t.filter == "owner_alive_targeted_by_other"
            && t.optional
    }));
    assert!(package.active_index.iter().any(|a| a.skill_def_id == XUSHI));
}

#[test]
fn xuchong_targeted_draws_and_gains_shadow() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    let compiled = content_compiler::compile_source_named("jsrg-shadow2", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(XUCHONG), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XUCHONG),
        }],
    )
    .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::TrickWuzhong,
        suit: engine_api::Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: other,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let before = tx.state().hand_count(owner);
    // optional 在 dispatch 期开 confirm → resolve [1] → 摸 1 + 影 1。
    assert_eq!(
        engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[engine_api::DomainEvent::CardTargetAdded {
                player: other,
                card,
                target: owner,
                targets: vec![owner],
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 40);
    assert_eq!(state.hand_count(owner), before + 2);
    let yings: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .filter(|c| {
            state
                .cards
                .get(*c)
                .map(|cd| cd.kind == engine_core::state::CardKind::Ying)
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(yings.len(), 1);
}

const FUMOU: u32 = 3_002_614_318;

#[test]
fn fumou_publishes_ying_as_chuqibuyi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == FUMOU
            && v.as_kind == "chuqibuyi"
            && v.filter == "card_ying"
            && v.source_zone.as_deref().is_none_or(|z| z == "hand")
    }));
}

#[test]
fn chuqibuyi_deals_damage_when_first_card_suit_differs() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    // 直接构造出其不意使用管线：actor 对 other 出牌（异花色首张）→ 1 伤。
    let compiled = content_compiler::compile_source_named("chuqibuyi-test", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(FUMOU), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = owner;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    // 手牌：other 首张红桃（与出其不意黑桃异花色）→ 伤害成立。
    let hand_o = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let heart = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::Shan,
        suit: engine_api::Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand_o,
        owner: other,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(hand_o)
        .unwrap()
        .cards
        .insert(0, heart);
    // 出其不意牌（黑桃）入手并打出。
    let hand_a = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cq = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2),
        kind: CardKind::TrickChuqibuyi,
        suit: engine_api::Suit::Spade,
        rank: 3,
        weapon_range: 0,
        zone: hand_a,
        owner: owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand_a).unwrap().cards.push(cq);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let revision = tx.state().revision;
    let state0 = tx.commit().state;
    let (state, pipe) = engine_core::command::apply_command(
        state0,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(50),
            kind: CommandKind::PlayCard {
                card: cq,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // 可能开无懈窗口——排空后断言伤害。
    let mut state = state;
    let mut key = 51;
    while state.has_pending_choice() {
        state = resolve(state.clone(), owner, vec![], key);
        key += 1;
    }
    assert_eq!(
        state.players.get(other).unwrap().hp,
        other_hp - 1,
        "suit-mismatch first card should take 1 damage"
    );
}

const FUNI: u32 = 3_773_533_412;

#[test]
fn funi_zeroes_attack_range() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == FUNI && m.target == "attack_range" && m.mode == "override" && m.value == 0
    }));
}

const JINFA: u32 = 4_139_928_729;

#[test]
fn jinfa_publishes_debate_branches_and_change_group() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.active_index.iter().any(|a| a.skill_def_id == JINFA));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JINFA
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_is_initiator_result_black"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JINFA
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_is_initiator_result_red"
            && t.optional
    }));
}

#[test]
fn change_group_overrides_faction() {
    let compiled = content_compiler::compile_source_named("change-group-test", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(JINFA), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JINFA),
        }],
    )
    .unwrap();
    let before = tx.state().player_group(owner).map(str::to_string);
    // 直接驱动 change_group 效果（经 skill content 编译，ABI 661 原语）。
    let compiled2 = content_compiler::compile_source_named(
        "change-group-prim",
        "skill change_group_probe(299792458) { active play_phase { effect change_group(self, wei, self_skill); } }",
    )
    .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled2)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(299_792_458),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = owner;
    let state0 = tx.commit().state;
    let probe = state0
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == 299_792_458)
        .unwrap()
        .id;
    let revision = state0.revision;
    let (state, pipe) = engine_core::command::apply_command(
        state0,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(60),
            kind: CommandKind::ActivateSkill {
                skill: probe,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(
        state.player_group(owner).map(str::to_string),
        Some("wei".into())
    );
    let _ = before;
}

#[test]
fn funi_distributes_shadows_each_round() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == FUNI && t.event == "round_started" && !t.optional }));
    // 行为：round_started → 分发 ceil(存活/2) 张影。
    let compiled = content_compiler::compile_source_named("funi-dist", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, u64::from(FUNI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FUNI),
        }],
    )
    .unwrap();
    // round_started forced 触发：dispatch 返回效果列表 → run_effects 后分发
    // 选择挂起（Cont 275）。
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::RoundStarted {
            round: 1,
            current: owner,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let state0 = tx.commit().state;
    let others: Vec<PlayerId> = state0
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner)
        .collect();
    // 选 2 名目标（3 人局 ceil(3/2)=2 张，均分各 1）。
    let state1 = resolve(state0, owner, vec![others[0].raw(), others[1].raw()], 70);
    let ying0 = state1
        .hand_cards(others[0])
        .iter()
        .filter(|c| {
            state1
                .cards
                .get(**c)
                .map(|cd| cd.kind == engine_core::state::CardKind::Ying)
                .unwrap_or(false)
        })
        .count();
    let ying1 = state1
        .hand_cards(others[1])
        .iter()
        .filter(|c| {
            state1
                .cards
                .get(**c)
                .map(|cd| cd.kind == engine_core::state::CardKind::Ying)
                .unwrap_or(false)
        })
        .count();
    assert_eq!(ying0, 1);
    assert_eq!(ying1, 1);
}

const YAOYAN: u32 = 3_932_516_749;

#[test]
fn yaoyan_publishes_register_resolve_pair() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    let yaoyan_triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == YAOYAN)
        .collect();
    assert_eq!(yaoyan_triggers.len(), 2);
    assert!(yaoyan_triggers.iter().any(|t| t.event == "phase_prepare"));
    assert!(yaoyan_triggers.iter().any(|t| t.event == "phase_end"));
}

#[test]
fn funi_tracks_ying_into_discard() {
    use engine_core::state::{Card, ZoneKind};
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FUNI
            && t.event == "cards_moved"
            && t.filter == "owner_cards_moved_contain_ying"
    }));
    // 行为：影入弃牌堆 → storage 置 1。
    let compiled = content_compiler::compile_source_named("funi-moved", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(FUNI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FUNI),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == FUNI)
        .unwrap()
        .id;
    // 造一张影入弃牌堆。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .or_else(|| tx.state().zones.find(ZoneKind::DiscardPile, owner))
        .unwrap();
    let ying = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(0),
        kind: engine_core::state::CardKind::Ying,
        suit: engine_api::Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone: discard,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(ying);
    let from_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let events = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardsMoved {
            cards: vec![ying],
            from: from_hand,
            to: discard,
            reason: engine_api::CardMoveReason::Discard,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, events).unwrap();
    assert_eq!(
        tx.state()
            .skills
            .get(skill)
            .unwrap()
            .content_storage
            .get(&1),
        Some(&1)
    );
}

const CHUSHI: u32 = 3_259_850_167;

#[test]
fn chushi_publishes_debate_with_lord() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == CHUSHI));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == CHUSHI
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_is_initiator_result_red"
    }));
}

const TW_LIHUO2: u32 = 2_376_916_887;
const TW: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

#[test]
fn tw_lihuo2_fire_sha_extra_target() {
    let package = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == TW_LIHUO2
            && t.event == "card_used"
            && t.filter == "owner_source_fire_sha"
            && t.optional
    }));
}

const CHIJIE: u32 = 3_286_369_736;

#[test]
fn tw_chijie_publishes_group_choice() {
    let package = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 733);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == CHIJIE && t.event == "round_started" && t.optional }));
}

const FEIYANG: u32 = 2_568_675_461;

#[test]
fn feiyang_publishes_judge_discard() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FEIYANG
            && t.event == "phase_judge"
            && t.filter == "owner_is_target_judge_nonempty"
            && t.optional
    }));
}

#[test]
fn place_xumou_places_card_in_judge_zone() {
    use engine_core::state::{Card, ZoneKind};
    let compiled = content_compiler::compile_source_named("xumou-test", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(FEIYANG), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    // 直接驱动 place_xumou（经 primitives 分发，SkillId 用 dummy 已加载技能）。
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FEIYANG),
        }],
    )
    .unwrap();
    let dummy = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == FEIYANG)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::PlaceXumou {
            player: owner,
            card,
            skill: dummy,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(tx.state().zones.get(judge).unwrap().cards.len(), 1);
}

#[test]
fn xumou_card_settles_with_tag_in_judge_zone() {
    use engine_core::state::{Card, ZoneKind};
    let compiled = content_compiler::compile_source_named("xumou-settle", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(FEIYANG), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FEIYANG),
        }],
    )
    .unwrap();
    let dummy = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == FEIYANG)
        .unwrap()
        .id;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::PlaceXumou {
            player: owner,
            card,
            skill: dummy,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let placed = tx.state().zones.get(judge).unwrap().cards[0];
    // 判定结算：蓄谋牌被移弃牌堆并触发视为使用事件（code 55）。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .unwrap();
    let events = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardsMoved {
            cards: vec![placed],
            from: judge,
            to: discard,
            reason: engine_api::CardMoveReason::Unspecified,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    // 合成事件只跑触发链；蓄谋牌仍物理在判定区（真实结算走
    // resolve_judge_zone 的 apply_judge_result）。
    run_effects(&mut tx, events).unwrap();
    // 切片 2 核心：蓄谋牌带 XUMOU_TAG（判定结算的视为使用由 tag 驱动）。
    assert!(tx
        .state()
        .content_card_tags
        .get(&placed)
        .is_some_and(|tags| tags.contains(&engine_core::content::XUMOU_TAG)));
}

const FANGJIE: u32 = 2_803_992_112;

#[test]
fn fangjie_publishes_prepare_recovery() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FANGJIE
            && t.event == "phase_prepare"
            && t.filter == "owner_is_target"
            && t.optional
    }));
}

const DAIMOU: u32 = 2_351_858_276;
const EQIAN: u32 = 3_425_800_404;

// ---- 蓄谋族切片 3b（ABI 666）：殆谋双分支 + 遏前①选牌蓄谋 ----

fn xumou3b_started(tag: &str, players: usize, def: u32) -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named(tag, &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, players as u8, 20_260_827, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn xumou3b_resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let (state, pipe) = apply_command(
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
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn xumou3b_drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), engine_core::effect::EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn xumou3b_inject_sha(tx: &mut Transaction, owner: PlayerId) -> engine_api::CardId {
    use engine_core::state::{Card, ZoneKind};
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(engine_core::state::Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn place_deck_top_xumou_moves_draw_pile_top_into_judge_zone() {
    use engine_core::state::ZoneKind;
    let (state, owner) = xumou3b_started("xumou-3b-top", 2, DAIMOU);
    let mut tx = Transaction::begin(state);
    let dummy = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == DAIMOU)
        .unwrap()
        .id;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, engine_api::PlayerId::NONE)
        .unwrap();
    let top = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::PlaceDeckTopXumou {
            player: owner,
            skill: dummy,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(
        tx.state().zones.get(judge).unwrap().cards,
        vec![top],
        "draw-pile top becomes the sole judge-zone card"
    );
    assert_eq!(tx.state().cards.get(top).unwrap().zone, judge);
    assert_eq!(tx.state().cards.get(top).unwrap().owner, owner);
    assert!(!tx.state().zones.get(draw).unwrap().cards.contains(&top));
    assert!(tx
        .state()
        .content_card_tags
        .get(&top)
        .is_some_and(|tags| tags.contains(&engine_core::content::XUMOU_TAG)));
}

#[test]
fn daimou_places_deck_top_once_per_turn_when_other_targeted() {
    use engine_core::state::ZoneKind;
    let (state, owner) = xumou3b_started("xumou-3b-daimou", 2, DAIMOU);
    let other = state
        .players
        .iter_live()
        .find(|p| p.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    let sha = xumou3b_inject_sha(&mut tx, other);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, engine_api::PlayerId::NONE)
        .unwrap();
    let draw_len_before = tx.state().zones.get(draw).unwrap().cards.len();

    // 首次「杀指定其他角色」：optional confirm → 牌堆顶入判定区。
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: other,
            targets: vec![other],
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let state = xumou3b_resolve(tx.state, owner, vec![1], false, 301);
    let judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(state.zones.get(judge).unwrap().cards.len(), 1);
    assert_eq!(
        state.zones.get(draw).unwrap().cards.len(),
        draw_len_before - 1
    );

    // 同回合第二次：confirm 仍开，但 storage 门槛拦下置牌。
    let mut tx = Transaction::begin(state);
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: other,
            targets: vec![other],
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let state = xumou3b_resolve(tx.state, owner, vec![1], false, 302);
    assert_eq!(state.zones.get(judge).unwrap().cards.len(), 1);

    // 回合结束清零门槛（近似 addTempSkill 移除）后再触发可再置牌。
    let mut tx = Transaction::begin(state);
    let cleared = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: other,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    cleared.unwrap();
    let draw_len_mid = tx.state().zones.get(draw).unwrap().cards.len();
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: other,
            targets: vec![other],
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let state = xumou3b_resolve(tx.state, owner, vec![1], false, 303);
    assert_eq!(state.zones.get(judge).unwrap().cards.len(), 2);
    assert_eq!(state.zones.get(draw).unwrap().cards.len(), draw_len_mid - 1);
}

#[test]
fn daimou_discards_own_xumou_forced_when_self_targeted() {
    use engine_core::state::{Card, ZoneKind};
    let (state, owner) = xumou3b_started("xumou-3b-daimou-me", 2, DAIMOU);
    let other = state
        .players
        .iter_live()
        .find(|p| p.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    let dummy = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == DAIMOU)
        .unwrap()
        .id;
    // 种一张蓄谋牌进判定区（直接驱动切片 1 原语）。
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let seed_card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2),
        kind: engine_core::state::CardKind::Tao,
        suit: engine_api::Suit::Heart,
        rank: 4,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(seed_card);
    run_effects(
        &mut tx,
        vec![EffectRequest::PlaceXumou {
            player: owner,
            card: seed_card,
            skill: dummy,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let placed = tx.state().zones.get(judge).unwrap().cards[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .unwrap();
    let discard_len_before = tx.state().zones.get(discard).unwrap().cards.len();

    // 杀指定自己：forced（上游「你须」）——无 confirm，效果直跑。
    let sha = xumou3b_inject_sha(&mut tx, other);
    xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: owner,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().zones.get(judge).unwrap().cards.len(), 0);
    let discarded = tx.state().zones.get(discard).unwrap().cards.clone();
    assert_eq!(discarded.len(), discard_len_before + 1);
    assert!(discarded.contains(&placed));

    // 同回合第二次被指定：门槛已合上，不再弃。
    xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: owner,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().zones.get(discard).unwrap().cards.len(),
        discard_len_before + 1
    );
}

#[test]
fn eqian_phase_end_moves_chosen_hand_card_into_judge_zone() {
    use engine_core::state::ZoneKind;
    let (state, owner) = xumou3b_started("xumou-3b-eqian", 2, EQIAN);
    let hand_ids: Vec<u32> = state.hand_cards(owner).iter().map(|c| c.raw()).collect();
    assert!(!hand_ids.is_empty());
    let chosen = engine_api::CardId::new(hand_ids[0]);
    let mut tx = Transaction::begin(state);

    // 结束阶段（phase_end ← TurnStateChanged End）：optional confirm → 选牌续体。
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    // confirm 后停在 Cont 278 选牌；选中第一张手牌移入判定区。
    let mid = xumou3b_resolve(tx.state, owner, vec![1], false, 401);
    assert_eq!(
        mid.resolution.continuation_kind,
        engine_core::state::ContinuationKind::ChoosePlaceHandXumou
    );
    let judge = mid.zones.find(ZoneKind::Judge, owner).unwrap();
    let mut tx = Transaction::begin(mid);
    let pending_observer = tx.state().resolution.pending_observer;
    let tx = tx.commit();
    let final_state = xumou3b_resolve(
        tx.state,
        PlayerId::new(pending_observer),
        vec![chosen.raw()],
        false,
        402,
    );
    assert_eq!(final_state.zones.get(judge).unwrap().cards, vec![chosen]);
    assert_eq!(final_state.cards.get(chosen).unwrap().zone, judge);
    assert!(!final_state.hand_cards(owner).contains(&chosen));
    assert!(final_state
        .content_card_tags
        .get(&chosen)
        .is_some_and(|tags| tags.contains(&engine_core::content::XUMOU_TAG)));

    // cancel = 停止蓄谋：再次触发后取消，不移动任何牌。
    let mut tx = Transaction::begin(final_state);
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let state = xumou3b_resolve(tx.state, owner, vec![], true, 403);
    assert_eq!(state.zones.get(judge).unwrap().cards.len(), 1);
}
