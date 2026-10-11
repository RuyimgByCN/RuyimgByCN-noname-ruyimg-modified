//! 阶段 1 交付：固化可重放场景（ADR 0703 "固定基线"）。
//!
//! 每个场景 = 命名 + 确定性命令脚本 + 终态 blake3 哈希（golden）。
//! golden 变化 = 内核语义变化，必须先登记行为差异/ADR 再更新。
//! 场景名必须在 `docs/native-coverage-manifest.md` 中登记（linkage 测试守护）。

use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

// ---------------------------------------------------------------------------
// 脚手架（与 native_kernel.rs 同约定：清手/放牌必须在推进之后，保证确定性）
// ---------------------------------------------------------------------------

/// 进程级去重键发生器：命令信任边界要求 dedup_id 严格递增。
static DEDUP: AtomicU64 = AtomicU64::new(1);

fn fresh_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn submit_cmd(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let expected_revision = e.revision();
    e.submit(GameCommand {
        actor,
        expected_revision,
        dedup_id: fresh_dedup(),
        action,
    })
}

fn next_phase(e: &mut Engine) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::NextPhase)
}

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..64 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play {
            return;
        }
        match next_phase(e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                answer_min_cards(e, &choice);
            }
            other => panic!("advance rejected: {other:?}"),
        }
    }
    panic!("advance_to_play did not converge");
}

fn answer_sel(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let choice = e.open_choice().expect("choice open").clone();
    submit_cmd(
        e,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection,
        },
    )
}

fn answer_min_cards(e: &mut Engine, choice: &engine_native::OpenChoice) -> CommandOutcome {
    let sel: Vec<ChoiceOption> = choice
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(_)))
        .copied()
        .take(choice.spec.min.max(1))
        .collect();
    answer_sel(e, sel)
}

fn answer(e: &mut Engine, opt: ChoiceOption) -> CommandOutcome {
    answer_sel(e, vec![opt])
}

fn play(e: &mut Engine, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::PlayCard { card, targets })
}

fn play_sha_at(e: &mut Engine, attacker: u8, target: u8) -> CommandOutcome {
    let sha = e
        .debug_hand(attacker)
        .iter()
        .copied()
        .find(|c| e.catalog().card_kind(*c) == CardKind::Sha)
        .expect("attacker holds a sha");
    let actor = e.turn().current;
    submit_cmd(
        e,
        actor,
        GameAction::PlayCard {
            card: sha,
            targets: vec![target],
        },
    )
}

// ---------------------------------------------------------------------------
// 场景（名称 = 覆盖清单登记键）
// ---------------------------------------------------------------------------

/// 出牌/闪/伤害/限一次/距离：闪避与受伤两回合。
fn scenario_sha_dodge_then_damage() -> [u8; 32] {
    let mut e = Engine::new(11, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let shan = e.debug_place(1, "shan");
    e.debug_place(0, "sha");
    play_sha_at(&mut e, 0, 1);
    answer(&mut e, ChoiceOption::Card(shan));
    // 回合交给 1：1 反击 0，0 无闪直接受伤
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    e.state_hash()
}

/// 濒死/无人救援/死亡/终局条件（单挑终局）。
fn scenario_death_terminal_single() -> [u8; 32] {
    let mut e = Engine::new(71, 2);
    e.debug_set_hp(0, 1);
    advance_to_play(&mut e, 1);
    for p in 0..2 {
        e.debug_clear_hand(p);
    }
    let _sha = e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    assert!(e.outcome().is_some(), "single mode must end after kill");
    e.state_hash()
}

/// 嵌套选择链（散财：确认→摸牌→目标→牌）。
fn scenario_sancai_nested_chain() -> [u8; 32] {
    let mut e = Engine::new(23, 4);
    e.debug_grant(0, "散财");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    answer(&mut e, ChoiceOption::Confirm);
    answer(&mut e, ChoiceOption::Player(2));
    let card = match e.open_choice().expect("card choice").spec.options[0] {
        ChoiceOption::Card(c) => c,
        _ => panic!("first option must be a card"),
    };
    answer(&mut e, ChoiceOption::Card(card));
    e.state_hash()
}

/// 伤害转移（移伤：前置窗口改写载荷）。
fn scenario_yishang_redirect() -> [u8; 32] {
    let mut e = Engine::new(31, 4);
    e.debug_grant(0, "移伤");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    answer(&mut e, ChoiceOption::Confirm);
    answer(&mut e, ChoiceOption::Player(2));
    e.state_hash()
}

/// 防止/每回合限一次/窗口内禁用（碎盾×肃清矩阵）。
fn scenario_suidun_window_matrix() -> [u8; 32] {
    let mut e = Engine::new(53, 4);
    e.debug_grant(0, "碎盾");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0); // 第一段被碎盾防止
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(0);
    e.debug_place(3, "sha");
    play_sha_at(&mut e, 3, 0); // 限一次已用 → 生效
    e.debug_grant(2, "肃清");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0); // 肃清禁用已入队碎盾 → 生效
    e.state_hash()
}

/// 动态获得/到期（授艺→移伤→回合结束移除）。
fn scenario_shouyi_dynamic_skill() -> [u8; 32] {
    let mut e = Engine::new(61, 4);
    e.debug_grant(0, "授艺");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    assert!(e.debug_has_skill(0, "移伤"));
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(0);
    play_sha_at(&mut e, 3, 0);
    answer(&mut e, ChoiceOption::Confirm);
    answer(&mut e, ChoiceOption::Player(1));
    // 推进到 0 的回合（临时技能应存续），再走到其回合结束到期
    advance_to_play(&mut e, 0);
    assert!(e.debug_has_skill(0, "移伤"));
    for _ in 0..5 {
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                answer_min_cards(&mut e, &choice);
            }
            other => panic!("{other:?}"),
        }
        if !e.debug_has_skill(0, "移伤") {
            break;
        }
    }
    assert!(!e.debug_has_skill(0, "移伤"));
    e.state_hash()
}

/// 弃牌上限。
fn scenario_discard_to_limit() -> [u8; 32] {
    let mut e = Engine::new(91, 4);
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 2);
    for _ in 0..4 {
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                answer_min_cards(&mut e, &choice);
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(e.debug_hand(0).len(), 2);
    e.state_hash()
}

/// 拒绝整段回滚（非法应答 + 预算耗尽后哈希不变）。
fn scenario_rollback_invariants() -> [u8; 32] {
    let mut e = Engine::new(81, 4);
    e.debug_grant(0, "散财");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    let (id, _) = {
        let c = e.open_choice().expect("choice open");
        (c.id, c.spec.prompt)
    };
    let before = e.state_hash();
    assert!(matches!(
        e.submit(GameCommand {
            actor: e.open_choice().unwrap().observer,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Card(9999)],
            },
        }),
        CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
    ));
    assert_eq!(e.state_hash(), before);
    // budget 字段入哈希：改预算后以其为新基准
    e.debug_set_budget(2);
    let before_small_budget = e.state_hash();
    assert!(matches!(
        e.submit(GameCommand {
            actor: e.open_choice().unwrap().observer,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Confirm],
            },
        }),
        CommandOutcome::Rejected(engine_native::RejectReason::BudgetExhausted)
    ));
    assert_eq!(e.state_hash(), before_small_budget);
    e.debug_set_budget(4096);
    answer(&mut e, ChoiceOption::Confirm);
    e.state_hash()
}

/// 装备栏与距离/攻击范围：±马修饰、武器射程、同栏替换、跨回合打击。
fn scenario_equip_distance_matrix() -> [u8; 32] {
    let mut e = Engine::new(101, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let sha = e.debug_place(0, "sha");
    // 基础距离 2 → 非法（拒绝不改状态）；-1 马 → 距离 1 → 命中
    play(&mut e, sha, vec![2]);
    let hm = e.debug_place(0, "horse_minus");
    play(&mut e, hm, vec![]);
    play(&mut e, sha, vec![2]);
    let w2 = e.debug_place(0, "weapon2");
    play(&mut e, w2, vec![]);
    // 2 号出 +1 马
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(0);
    let hp_card = e.debug_place(2, "horse_plus");
    play(&mut e, hp_card, vec![]);
    // 0 借武器射程 2 跨 +1 马打击
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let sha2 = e.debug_place(0, "sha");
    play(&mut e, sha2, vec![2]);
    // 同栏替换：weapon3 换下 weapon2
    e.debug_clear_hand(0);
    let w3 = e.debug_place(0, "weapon3");
    play(&mut e, w3, vec![]);
    e.state_hash()
}

/// 铁索连环与属性伤害：横置 → 火杀传导 → 解链；普通杀不传导。
fn scenario_chain_fire_propagation() -> [u8; 32] {
    let mut e = Engine::new(113, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    play(&mut e, chain, vec![1, 2]);
    let fsha = e.debug_place(0, "fire_sha");
    play(&mut e, fsha, vec![1]); // 传导给 2，随后解链
    // 普通杀不再传导
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    // 雷伤验证（无链者不再传导）
    e.debug_deal_damage(
        None,
        2,
        1,
        engine_native::event::DamageNature::Thunder,
    );
    e.state_hash()
}

/// AOE 多目标响应队列：南蛮（免伤/跳过/Pass）→ 万箭（全 Pass 受伤）。
fn scenario_aoe_response_queue() -> [u8; 32] {
    let mut e = Engine::new(131, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in [1u8, 2, 3] {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let sha1 = e.debug_place(1, "sha");
    let sha3 = e.debug_place(3, "sha");
    play(&mut e, savage, vec![]);
    answer(&mut e, ChoiceOption::Card(sha1)); // 1 免伤；2 跳过；询问 3
    answer(&mut e, ChoiceOption::Pass); // 3 受伤 → 南蛮完结
    // 万箭：仅 1 持闪（Pass 受伤），2/3 无闪直接跳过受伤
    e.debug_clear_hand(1);
    let shan1 = e.debug_place(1, "shan");
    let archery = e.debug_place(0, "archery");
    play(&mut e, archery, vec![]);
    answer(&mut e, ChoiceOption::Pass);
    e.state_hash()
}

/// 决斗交替出杀：目标先出、来源出、目标无杀即败；无距离限制（0→2）。
fn scenario_duel_alternating_response() -> [u8; 32] {
    let mut e = Engine::new(141, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    let sha_t1 = e.debug_place(2, "sha");
    let sha_s = e.debug_place(0, "sha");
    play(&mut e, duel, vec![2]);
    answer(&mut e, ChoiceOption::Card(sha_t1));
    answer(&mut e, ChoiceOption::Card(sha_s));
    // 目标再无杀：不询问直接受伤，决斗结束
    e.state_hash()
}

/// 无懈响应链：南蛮被无懈抵消 → 反无懈恢复生效（含交替询问）。
fn scenario_nullify_chain_on_savage() -> [u8; 32] {
    let mut e = Engine::new(153, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in [1u8, 2, 3] {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let nul1 = e.debug_place(1, "nullify");
    let nul2 = e.debug_place(2, "nullify");
    let sha1 = e.debug_place(1, "sha");
    play(&mut e, savage, vec![]);
    answer(&mut e, ChoiceOption::Card(nul1)); // 抵消
    answer(&mut e, ChoiceOption::Card(nul2)); // 反无懈 → 恢复
    // 南蛮生效：1 出杀免伤；2、3 跳过受伤
    answer(&mut e, ChoiceOption::Card(sha1));
    e.state_hash()
}

/// 判定区与延时锦囊：乐跳过出牌阶段（黑桃判定）、判定牌入弃堆。
fn scenario_judge_phase_delayed_tricks() -> [u8; 32] {
    let mut e = Engine::new(161, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 黑桃判定 → 乐生效 → 1 跳过出牌阶段（无人持无懈，窗口不开询问）
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && def.kind == CardKind::Sha);
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    // 推进 1 的回合：Draw 之后直接 Discard
    loop {
        let t = e.turn();
        if t.current == 1 && t.phase == Phase::Draw {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                answer_min_cards(&mut e, &choice);
            }
            other => panic!("{other:?}"),
        }
    }
    next_phase(&mut e);
    e.state_hash()
}

// ---------------------------------------------------------------------------
// golden 表 + 联动守护
// ---------------------------------------------------------------------------

type ScenarioFn = fn() -> [u8; 32];

/// (场景名, 运行函数, 覆盖清单机制键, golden 哈希 hex)
/// golden 变更 = 语义变化，须先在清单登记行为差异/ADR。
/// 2026-09-24 重录依据（v60）：规则版本 59→60（祖墨手铠封印，版本字段
/// 入 canonical_hash 必变全部哈希；v60 行为分支仅经 gwent 牌种，标准场景
/// 不可达）+ standard_catalog_parts 拆牌循环重复入目录修复（顺/酒/雷杀
/// 8/4/4 → 4/2/2，改变建局发牌）。轨迹语义未变，逐场景命令与断言不变；
/// 证据见 docs/operations/evidence/2026-09-24-replay-golden-v60/。
/// 2026-10-07 重录依据（v61/v4）：E03 翻面原语——PlayerState 尾部追加
/// face_down 编码（初始 false，行为无变化，纯状态扩展）+ 规则版本 60→61 +
/// 存档格式 3→4。既有场景无翻面事件，轨迹语义不变。
/// 2026-10-10 重录依据（v192 事故恢复收尾）：golden 原值为 v188 批重录；
/// 其后 v191 勾魂锣（gouhunluo 字段入 canonical_hash）与 v192 机关桶
///（版本 191→192）两批的重录因 host.rs 误清零事故未及完成，事故恢复后
/// 补齐。版本回退探针（192→188 复跑仍 mismatch）证明位移含 v191 哈希
/// 字段扩展，非单一版本号；场景轨迹语义经探针（杀→闪响应→弃牌序）与
/// 全量行为测试（524 passed）核验未变，逐场景命令与断言不变。
/// 2026-10-10 重录依据（v193 机关桶收尾）：规则版本 192→193（机关桶
/// 补上游 reverseOrder 逆序 + contentAfter 无人受伤摸一）。版本回退探
/// 针（193→192 复跑 golden 全绿）证明位移唯一来源是版本字段入
/// canonical_hash；jiguantong 行为分支仅经 swd 扩展牌种，标准场景不可
/// 达，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-10-t10-jiguantong-v193/。
/// 2026-10-11 重录依据（v194 掠影对齐，T11 子项）：规则版本 193→194
/// （掠影补"另有其他有牌角色"过滤与非强制确认）+ 存档格式 41→42
///（MachineSnapshot::Sha 尾部追加 lueying_declined）。双版本回退探针
///（194/42→193/41 复跑 golden 全绿）证明位移唯一来源是版本字段；
/// 掠影分支仅经 xianjian 扩展牌种，标准场景不可达，逐场景命令与断言
/// 不变。证据见 docs/operations/evidence/2026-10-11-t11-lueying-v194/。
/// 2026-10-11 重录依据（v195 机关凤，T12 切片）：规则版本 194→195
/// （机关凤=新机关牌切片，仅经 swd 牌种可达）。版本回退探针（195→194
/// 复跑 golden 全绿）证明位移唯一来源是版本字段；标准场景不可达，逐场
/// 景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t12-jiguanfeng-v195/。
const SCENARIOS: &[(&str, ScenarioFn, &[&str], &str)] = &[
    (
        "sha_dodge_then_damage",
        scenario_sha_dodge_then_damage,
        &["card_use", "sha_respond", "damage_commit", "sha_limit", "seat_range"],
        "734f4f515d2add6ac9860d3146c9fcd71c11fbcd1bf384d5551f1660fff87f7e",
    ),
    (
        "death_terminal_single",
        scenario_death_terminal_single,
        &["dying", "death", "victory"],
        "6b9703a7e3706adc12c68a72f928962521fd82c64592e18eb2281ccd2e8d137a",
    ),
    (
        "sancai_nested_chain",
        scenario_sancai_nested_chain,
        &["optional_trigger", "nested_choice", "draw", "give_cards"],
        "aa5265b3d0be9fc870ad5d13c4e25e75eed9cb8142f27025e1d76b40a7cdf5ed",
    ),
    (
        "yishang_redirect",
        scenario_yishang_redirect,
        &["damage_before_window", "payload_rewrite_redirect"],
        "8c3a96be94092af7905b87d9e45544b2cc1a64e0ba74606fbd8cbaf2fc6fe737",
    ),
    (
        "suidun_window_matrix",
        scenario_suidun_window_matrix,
        &["damage_prevent", "once_per_turn", "mid_resolution_disable"],
        "127d4f4eef8dcb378331141d1f66e426f25747ce9a875767e588ae9e64cef6eb",
    ),
    (
        "shouyi_dynamic_skill",
        scenario_shouyi_dynamic_skill,
        &["dynamic_gain", "skill_expiry", "redirect_after_gain"],
        "d3b40ea6329e92ed7e438e71c0c62c16081c33bd90a2d49c50e13bfb159a5d2f",
    ),
    (
        "discard_to_limit",
        scenario_discard_to_limit,
        &["hand_limit", "intrinsic_machine"],
        "878b2f31feda3c4403df283debc0245b48af389cb82503bbd71fa5552ce2ebe0",
    ),
    (
        "rollback_invariants",
        scenario_rollback_invariants,
        &["reject_rollback", "budget_rollback"],
        "59ca7c92eedce122d020a3c4438421d97f5bee2fbfd6d742cf5d494f60cf0057",
    ),
    (
        "equip_distance_matrix",
        scenario_equip_distance_matrix,
        &["equip_slots", "distance_modifier", "attack_range", "equip_replace"],
        "7d55f77612776e7210f8b1f878cf20aed58860f6eef5a7a6856b23c7f80915d0",
    ),
    (
        "chain_fire_propagation",
        scenario_chain_fire_propagation,
        &["chain_state", "damage_nature", "chain_propagation"],
        "378fff24f642b469c8dd0f50c59c9100b40307d4a7cd64a4f1f6fd9e6da15525",
    ),
    (
        "aoe_response_queue",
        scenario_aoe_response_queue,
        &["aoe_queue", "aoe_skip_no_card", "aoe_full_resolution"],
        "3a6da9c8718f4a96f4930a925d2380289980bb968c6933a05ec92d7b9f9749c2",
    ),
    (
        "duel_alternating_response",
        scenario_duel_alternating_response,
        &["duel_alternating", "duel_no_distance_limit"],
        "8be9fde76f63b0b9f4ec74d1e3bfff9268d5ba99b10289966163a1a86be764c3",
    ),
    (
        "nullify_chain_on_savage",
        scenario_nullify_chain_on_savage,
        &["nullify_window", "nullify_chain", "nullify_scope"],
        "d24eada7a39fa3998e085d28ba4a4f249b9fa433e239c7388234e5276674383b",
    ),
    (
        "judge_phase_delayed_tricks",
        scenario_judge_phase_delayed_tricks,
        &["judge_zone", "delayed_trick", "phase_skip"],
        "239e9c3f773bd39829cf6b09d1e0db7022df90e0ab8a44d8bc4d8cc840f033f6",
    ),
];

#[test]
fn replay_scenarios_match_golden() {
    let mut bad = Vec::new();
    for (name, run, _mechs, golden) in SCENARIOS {
        let hex = hex(&run());
        if hex != *golden {
            bad.push(format!("{name} actual={hex} expected={golden}"));
        }
    }
    assert!(bad.is_empty(), "golden mismatches:\n{}", bad.join("\n"));
}

/// 场景与覆盖清单联动：每个场景名与其机制键都必须登记在
/// docs/native-coverage-manifest.md（阶段 1 覆盖清单）。
#[test]
fn scenarios_are_registered_in_coverage_manifest() {
    let manifest = std::fs::read_to_string("../../docs/native-coverage-manifest.md")
        .expect("coverage manifest exists");
    for (name, _, mechs, _) in SCENARIOS {
        assert!(
            manifest.contains(&format!("replay:{name}")),
            "scenario {name} not registered in coverage manifest"
        );
        for m in *mechs {
            assert!(
                manifest.contains(&format!("`{m}`")),
                "mechanism key {m} not registered in coverage manifest"
            );
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
