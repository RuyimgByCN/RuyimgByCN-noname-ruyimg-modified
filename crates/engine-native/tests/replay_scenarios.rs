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
    let _sha3 = e.debug_place(3, "sha");
    play(&mut e, savage, vec![]);
    answer(&mut e, ChoiceOption::Card(sha1)); // 1 免伤；2 跳过；询问 3
    answer(&mut e, ChoiceOption::Pass); // 3 受伤 → 南蛮完结
    // 万箭：仅 1 持闪（Pass 受伤），2/3 无闪直接跳过受伤
    e.debug_clear_hand(1);
    let _shan1 = e.debug_place(1, "shan");
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
/// 2026-10-11 重录依据（v196 机关怨，T12 切片）：规则版本 195→196
/// （机关怨=新机关牌切片+存档 v43 PlayerState.jiguanyuan_cards）。版本
/// 回退探针（196→195 复跑 golden 全绿）证明位移唯一来源是版本字段；
/// 机关怨分支仅经 swd 牌种，标准场景不可达，逐场景命令与断言不变。
/// 证据见 docs/operations/evidence/2026-10-11-t12-jiguanyuan-v196/。
/// 2026-10-11 重录依据（v197 机关妖术，T12 切片）：规则版本 196→197
/// （机关妖术=新机关牌切片，仅经 swd 牌种可达）。版本回退探针（197→196
/// 复跑 golden 全绿）证明位移唯一来源是版本字段；标准场景不可达，逐场
/// 景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t12-jiguanyaoshu-v197/。
/// 2026-10-11 重录依据（v198 机关术批，T12）：规则版本 197→198（灵剑池
/// 7 张+机关妖术补偿+CardKind::Hslingjian）+ 存档格式 43→44（灵剑状态
/// 四字段）。双版本回退探针（198/44→197/43 复跑 golden 全绿）证明位移
/// 唯一来源是版本字段；灵剑/补偿分支仅经 swd 牌种，标准场景不可达，逐
/// 场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t12-jiguanshu-v198/。
/// 2026-10-11 重录依据（v199 机关术×2，T18）：规则版本 198→199（机关术
/// 转正+CardMod 伴随表）+ 存档格式 44→45（card_mods）。双版本回退探针
///（199/45→198/44 复跑 golden 全绿）证明位移唯一来源是版本字段；机关术
/// 分支仅经 swd 牌种，标准场景不可达，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t18-jiguanshu-primitive-v199/。
/// 2026-10-11 重录依据（v200 阵法三张，T19a）：规则版本 199→200（天覆/
/// 七星/鸟翔阵 + 国战派生判定原语）。版本回退探针（200→199 复跑 golden
/// 全绿）证明位移唯一来源是版本字段；阵法分支仅经 zhenfa 牌种，标准场
/// 景不可达，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t19-zhenfa-v200/。
/// 2026-10-11 重录依据（v201 长蛇阵，T19b）：规则版本 200→201（长蛇阵
/// 转正+swapSeat 座位环）+ 存档格式 45→46（GameStateCore.seat_ring）。
/// 双版本回退探针（201/46→200/45 复跑 golden 全绿）证明位移唯一来源是
/// 版本字段；长蛇阵分支仅经 zhenfa 牌种，标准场景不可达，逐场景命令与
/// 断言不变。证据见
/// docs/operations/evidence/2026-10-11-t19b-swapseat-v201/。
/// 2026-10-11 重录依据（v202 标准差异消除批，T11）：规则版本 201→202
/// （倾国改黑手牌、救援补主公资格+用桃者吴势力、结姻目标改受伤男性、
/// 急救限持有者回合外——均为标准技能条件修正）。版本回退探针（202→201
/// 复跑 golden 全绿）证明位移唯一来源是版本字段；14 个标准场景均不触
/// 发这四个技能的修改分支，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-standard-diff-v202/。
/// 2026-10-11 重录依据（v203 反间对齐，T11）：规则版本 202→203（反间补
/// 目标选花色询问+花色相符不受伤）+ 存档格式 46→47（FanJian 机器
/// chosen_suit 字段与 SuitAsk 阶段）。版本回退探针（203→202 复跑 golden
/// 全绿）证明位移唯一来源是版本字段；反间分支不在标准场景可达路径，逐
/// 场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-fanjian-v203/。
/// 2026-10-11 重录依据（v204 护驾对齐，T11）：规则版本 203→204（护驾
/// 询问改持牌者自选一张闪打出——SelectCards 含倾国转化黑手牌，空选=
/// 拒绝；撤销「弃置+计数」登记差异的自动弃牌面）。版本回退探针（204→
/// 203 复跑 golden 全绿）证明位移唯一来源是版本字段；护驾分支不在标准
/// 场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-hujia-v204/。
/// 2026-10-11 重录依据（v205 斗地主 binglin 变体，T11）：规则版本
/// 204→205（GameMode 增 DoudizhuVariant 建局面：binglin 地主 5/农民 4，
/// 变体不入模式编码）。版本回退探针（205→204 复跑 golden 全绿）证明
/// 位移唯一来源是版本字段；斗地主分支不在标准场景可达路径，逐场景命
/// 令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-doudizhu-binglin-v205/。
/// 2026-10-11 重录依据（v206 versus 替补，T11）：规则版本 205→206
/// （Versus 增替补武将池：死亡座位从本队池弹队首换将复活+摸 4，死亡
/// 不结算；池空照常死亡）。版本回退探针（206→205 复跑 golden 全绿）
/// 证明位移唯一来源是版本字段；versus 分支不在标准场景可达路径，逐场
/// 景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-versus-substitute-v206/。
/// 2026-10-11 重录依据（v207 Boss 关卡骨架，T11）：规则版本 206→207
/// （Boss 关卡开局面：gameDraw boss/挑战者分列发牌数+loopFirst 回合起
/// 点座位，OpeningPlan 建局面、变体不入模式编码）。版本回退探针
/// （207→206 复跑 golden 全绿）证明位移唯一来源是版本字段；Boss 分支
/// 不在标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-stage-v207/。
/// 2026-10-11 重录依据（v208 boss 内容落包首批，T11）：规则版本
/// 207→208（BOSS_GENERALS 烛阴+共有技凶兽入目录——杀伤条件+1/距离-1/
/// 不可翻面三面；技能表尾追加）。版本回退探针（208→207 复跑 golden
/// 全绿）证明位移唯一来源是版本字段；Boss 分支不在标准场景可达路径，
/// 逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-zhuyin-v208/。
/// 2026-10-11 重录依据（v209 boss 内容第二批首片：混沌无脏，T11）：
/// 规则版本 208→209（BOSS_GENERALS 增混沌 hp25+无脏两面——摸牌基数
/// max(5,hp/2) 与手牌上限基数 0；hand_limit 签名增 catalog）。版本回
/// 退探针（209→208 复跑 golden 全绿）证明位移唯一来源是版本字段；
/// Boss 分支不在标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-hundun-wuzang-v209/。
/// 2026-10-11 重录依据（v210 boss 内容第二批续片：相德+隐贼，T11）：
/// 规则版本 209→210（混沌补全四技——相德 DamageBefore 触发机（新
/// MachineSnapshot::BossXiangde 变体 tag61/哈希 62）+隐贼激活门控与伤
/// 害后弃牌双臂）。版本回退探针（210→209 复跑 golden 全绿）证明位移
/// 唯一来源是版本字段；Boss 分支不在标准场景可达路径，逐场景命令与
/// 断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-xiangde-yinzei-v210/。
/// 2026-10-11 重录依据（v211 boss 内容第三批首片：穷奇逐厄，T11）：
/// 规则版本 210→211（BOSS_GENERALS 增穷奇 hp20+逐厄单技——global
/// damageEnd 穷奇与来源双摸；穷奇 maxHp25 登记差异暂=hp20）。版本回
/// 退探针（211→210 复跑 golden 全绿）证明位移唯一来源是版本字段；
/// Boss 分支不在标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-qiongqi-zhue-v211/。
/// 2026-10-11 重录依据（v212 boss 内容第三批续片：穷奇覆泰，T11）：
/// 规则版本 211→212（覆泰——准备阶段[判定阶段开始锚点]全体受伤者按
/// 座位序各回 1；新 MachineSnapshot::BossFutai 变体 tag62/哈希 63）。
/// 版本回退探针（212→211 复跑 golden 全绿）证明位移唯一来源是版本字
/// 段；Boss 分支不在标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-qiongqi-futai-v212/。
/// 2026-10-11 重录依据（v213 boss 内容第三批再续片：穷奇延妒，T11）：
/// 规则版本 212→213（延妒——体力首次≤一半激活[SkillInstance.enabled
/// 同隐贼先例]+其他角色无伤害统计的结束阶段穷奇获得其首张手牌[确定性
/// 近似]）。版本回退探针（213→212 复跑 golden 全绿）证明位移唯一来源
/// 是版本字段；Boss 分支不在标准场景可达路径，逐场景命令与断言不变。
/// 证据见 docs/operations/evidence/2026-10-11-t11-boss-yandu-v213/。
/// 2026-10-11 重录依据（v214 覆泰2 全场禁桃，T11）：规则版本 213→214
/// （覆泰2——穷奇持覆泰且非其回合时桃不可用[PlayCard 校验]/不可救
/// [is_tao_like]两面；穷奇回合禁令解除）。版本回退探针（214→213 复
/// 跑 golden 全绿）证明位移唯一来源是版本字段；Boss 分支不在标准场景
/// 可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-futai2-v214/。
/// 2026-10-11 重录依据（v215 boss 内容第四批首片：饕餮贪欲，T11）：
/// 规则版本 214→215（贪欲两面——跳过弃牌阶段+结束阶段全场最多手牌
/// 失 1 体[is_max 判定抽函数]；BOSS_GENERALS 增饕餮 hp20）。版本回退
/// 探针（215→214 复跑 golden 全绿）证明位移唯一来源是版本字段；Boss
/// 分支不在标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-taotie-v215/。
/// 2026-10-11 重录依据（v216 boss 内容第四批续片：饕餮藏目+积财，
/// T11）：规则版本 215→216（藏目——摸牌基数=存活角色数[Draw 臂无脏
/// 同位]；积财——激活门控[延妒并列]+回复结算后双摸[新 commit_recover
/// 臂；debug_recover 测试面]）。版本回退探针（216→215 复跑 golden 全
/// 绿）证明位移唯一来源是版本字段；Boss 分支不在标准场景可达路径，逐
/// 场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-cangmu-jicai-v216/。
/// 2026-10-11 重录依据（v217 boss 内容第五批首片：梼杌拟态，T11）：
/// 规则版本 216→217（拟态两面——回合内受伤防止[commit 入口直读]与
/// 回合外火伤 +1[amount 链]；BOSS_GENERALS 增梼杌 hp25）。版本回退探
/// 针（217→216 复跑 golden 全绿）证明位移唯一来源是版本字段；Boss
/// 分支不在标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-taowu-nitai-v217/。
/// 2026-10-11 重录依据（v218 boss 内容第五批续片：梼杌乱常，T11）：
/// 规则版本 217→218（乱常——激活门控[SkillInstance.enabled 同隐贼先
/// 例]+准备阶段虚拟南蛮/结束阶段虚拟万箭[use_virtual_card 载体 AOE；
/// 新 MachineSnapshot::BossLuanchang 变体 tag63/哈希 64]）。版本回退
/// 探针（218→217 复跑 golden 全绿）证明位移唯一来源是版本字段；Boss
/// 分支不在标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-luanchang-v218/。
/// 2026-10-11 重录依据（v223 相柳批末片：毒妒+蛇毒链+邪酬，T11）：
/// 规则版本 222→223（毒妒——相柳受击来源获蛇毒标记；邪酬——hp≤半激
/// 活+他人桃/酒获标记；蛇毒标记=PlayerState.shedu 新字段[存档
/// 48→49→50]+其回合开始弃 X 或失 X 结算机[Shedu 触发技，新快照变体
/// tag64/哈希 65]）。版本回退探针（223→222 复跑 golden 全绿）证明位
/// 移唯一来源是版本字段；Boss 分支不在标准场景可达路径，逐场景命令与
/// 断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-duqu-v223/。
/// **版本断链审计（v224 批发现）**：v220–v223 四批的 bump/restore 脚
/// 本链存在断链（restore 锚失效致运行时版本停留 222，四批「重录」实
/// 为 222 版+各批代码的哈希——自洽但名义版本与运行时不符）。本批修正
/// 至 224 并如实登记：各批行为面回归不受影响（以各自代码状态运行并录
/// 制），仅名义版本号断层。
/// 2026-10-11 重录依据（v224 朱厌批首片：兵先，T11）：规则版本
/// 222→**224**（运行时版本从实际 222 修正至 224+兵先——他人无出杀回
/// 合结束被虚拟杀）。探针形态变化：223 版运行≠v223 批哈希（目录表序
/// 已含各批 def 追加）——探针口径自本批起以「223 版+本批代码」的 14
/// 全位移佐证（Boss 分支不可达，场景命令与断言不变）。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-bingxian-v224/。
/// 2026-10-11 重录依据（v227 毕方批首片：招祸，T11）：规则版本
/// 222→227（断链批合并录——v225 虚失+门控七臂/v226 巨源+存档 v51/
/// v227 招祸两面[毕方伤害视为火 DamageBefore 机+免疫火伤]；毕方条目
/// 插入使书精/相柳/朱厌 def 索引重排，测试同步）。版本回退探针：
/// 224 版+当前代码含 v225–v227 def/臂≠v224 批哈希，探针以 14 全位移
/// 佐证 Boss 分支不可达；场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-bifang-v227/。
/// 2026-10-11 重录依据（v230 应昭批首片：封冻，T11）：规则版本
/// 227→229（断链批合并录——v228 红莲；v229 炎狱+TurnState.yanyu_count
/// [存档 51→52]；v230 封冻+**PlayerState.fengyin_active 新字段，存档
/// 52→53**）。**字段新增批探针口径**：PlayerState 字段入 canonical_hash
/// 使全部场景哈希位移（含版本字段），「位移唯一来源=版本字段」的探针
/// 形态对本批不适用——以 229 版+当前代码 14 全位移佐证 Boss 分支不可
/// 达；场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-fengdong-v230/。
/// 2026-10-11 重录依据（v231 应昭批续片：巡游，T11）：规则版本
/// 229→231（运行时版本从实际 230 修正+巡游单技——他人回合开始获座位
/// 序首名有手牌者一张手牌[确定性近似登记]，装备牌自动使用[同栏替换]）。
/// 版本回退探针：231 版+当前代码全位移佐证 Boss 分支不可达（探针口径
/// 延续字段批登记）；场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-xunyou-v231/。
/// 2026-10-11 重录依据（v228 毕方批续片：红莲，T11）：规则版本
/// 227→228（红莲两面——红色牌不计入手牌上限[Discard 统计与选项双排
/// 除，shamolvzhou 先例]+回合开始顺序获前缀红牌至多 3/对座位序前 3 名
/// 各 1 火伤[随机近似登记]）。版本回退探针（228→227 复跑 golden 全绿）
/// 证明位移唯一来源是版本字段；Boss 分支不在标准场景可达路径，逐场景
/// 命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-honglianx-v228/。
/// 2026-10-11 重录依据（v229 毕方批末片：炎狱，T11）：规则版本
/// 228→229（炎狱两面——激活门控第八并列臂+其他角色回合开始判定红=1
/// 火伤[Judge begin 段内联确定性判定铁骑先例；TurnState.yanyu_count 回
/// 合级计数器封顶 3，存档 51→52]）。版本回退探针（229→228 复跑 golden
/// 全绿）证明位移唯一来源是版本字段；Boss 分支不在标准场景可达路径，
/// 逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-yanyu-v229/。
/// 2026-10-11 重录依据（v225 朱厌批续片：虚失，T11）：规则版本
/// 224→225（虚失两面——出牌阶段结束[弃牌阶段开始锚]自翻面+翻回正面
/// 全场其他角色各受 1[随机恒 1 近似]；激活门控第七并列臂）。版本回退
/// 探针（225→224 复跑 golden 全绿）证明位移唯一来源是版本字段；Boss
/// 分支不在标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-xushi-v225/。
/// 2026-10-11 重录依据（v226 朱厌批末片：巨源，T11）：规则版本
/// 225→226（巨源两面——回合结束体力记录[PlayerState.juyuan_storage
/// 新字段，存档 50→51]+出牌阶段体力<记录时杀额外 1 目标[max_targets
/// 飞刃同款]）。版本回退探针（226→225 复跑 golden 全绿）证明位移唯一
/// 来源是版本字段；Boss 分支不在标准场景可达路径，逐场景命令与断言不
/// 变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-juyuan-v226/。
/// 2026-10-11 重录依据（v219 梼杌批末片：冥顽，T11）：规则版本
/// 218→219（冥顽三面——伤害后锁定集[TurnState.minwan_targets 回合级
/// 新字段，存档 48→49]+每用牌摸 1+PlayCard 目标校验 mod）。版本回退
/// 探针（219→218 复跑 golden 全绿）证明位移唯一来源是版本字段；Boss
/// 分支不在标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-minwan-v219/。
/// 2026-10-11 重录依据（v220 书精批：催毒+中毒，T11）：规则版本
/// 219→220（BOSS_GENERALS 增书精 hp2——催毒[伤害授毒 runtime 授技]；
/// 中毒[回合开始判定：非红桃伤 1/黑桃保留/方片梅花失去]——Judge begin
/// 段内联确定性判定[铁骑先例]）。版本回退探针（220→219 复跑 golden 全
/// 绿）证明位移唯一来源是版本字段；Boss 分支不在标准场景可达路径，逐
/// 场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-shujing-v220/。
/// 2026-10-11 重录依据（v221 四神兽系首片：相柳妖兽，T11）：规则版本
/// 220→221（BOSS_GENERALS 增相柳 hp25+妖兽单技——computed_distance 距
/// 离 -2[凶兽同款距离链]；相柳毒妒[蛇毒标记链]/兽九[手牌补 9]/邪酬
/// (+switch) 登记下批）。版本回退探针（221→220 复跑 golden 全绿）证
/// 明位移唯一来源是版本字段；Boss 分支不在标准场景可达路径，逐场景命
/// 令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-xiangliu-yashou-v221/。
/// 2026-10-11 重录依据（v222 相柳续片：兽九，T11）：规则版本 221→222
/// （兽九三面——手牌上限恒 9[hand_limit 函数分支]+出牌/结束补至 9+
/// 跳过摸牌[阶段臂]）。版本回退探针（222→221 复跑 golden 全绿）证明
/// 位移唯一来源是版本字段；Boss 分支不在标准场景可达路径，逐场景命令
/// 与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-jiushou-v222/。
/// 2026-10-13 重录依据（v269 转轮王批，T11 十殿阎罗）：**字段批**——规
/// 则版本 268→269（轮回/往生/反噬三技+转轮王条目[魔道共用]）且
/// PlayerState 新增 zlfanshi_terra 入 canonical hash（反噬 terra 旗，
/// 存档 v62）。字段批口径：v268 提交时（无字段+268）全绿；268+字段与
/// 269+字段两态全差且 actual 互异（版本位移可见），按字段批全量重录。
/// 证据见 docs/operations/evidence/2026-10-13-t11-boss-zhuanlunwang-v269/。
/// 2026-10-13 重录依据（v268 泰山王批，T11 十殿阎罗）：普通批——规则版
/// 本 267→268（服毒/苦酒/热恼三技+泰山王条目；显示名勘误 boss_fudu=服
/// 毒/boss_kujiu=苦酒）。服毒/苦酒=内联臂（CardUsed/Judge begin global
/// 扫持有者），热恼=Death 遗言单发机（BossRenao 入重建表）。探针：267
/// 回退全绿/268 全差=位移唯一来源是版本字段。证据见
/// docs/operations/evidence/2026-10-13-t11-boss-taishanwang-v268/。
/// 2026-10-13 重录依据（v267 卞城王批，T11 十殿阎罗）：普通批——规则版
/// 本 266→267（雷舟/雷斧/雷诛三技+卞城王条目）。雷诛=内容侧首个可挂
/// 起机（逐发雷伤 Yield+BossLeizhu 快照含 emitted 进度，入重建表首例）。
/// 探针：266 回退全绿/267 全差=位移唯一来源是版本字段。证据见
/// 2026-10-14 重录依据（v270 孟婆批，T11 十殿阎罗）：**字段批**——规
/// 则版本 269→270（拾忧/往事/忘魂三技+孟婆条目）且 PlayerState 新增
/// wangshi_ban 入 canonical hash（往事封禁旗，存档 v62→v63）。字段批
/// 口径：269 提交时（无字段）全绿；269+字段与 270+字段两态全差且
/// actual 互异（版本位移可见），按字段批全量重录。证据见
/// 2026-10-14 重录依据（v271 阎罗王批，T11 十殿阎罗）：普通批——规则版
/// 本 270→271（铁面/铡刀/诛心三技+阎罗王条目）。铁面/铡刀=sha 机仁王/
/// 青釭两链扩展（查询型），诛心=Death 遗言单发机（BossZhuxin 入重建
/// 表）。版本回退探针（271→270 复跑 golden 全绿）证明位移唯一来源是版
/// 本字段。证据见
/// 2026-10-14 重录依据（v272 平等王批，T11 十殿阎罗）：普通批——规则版
/// 本 271→272（锁足/阿鼻/平等三技+平等王条目）。锁足=Prepare-begin 臂
/// linked 直写，阿鼻=Damage 尾臂（恒定火近似），平等=Death 遗言两波机
///（BossPingdeng 入重建表）。版本回退探针（272→271 复跑 golden 全绿）
/// 证明位移唯一来源是版本字段。证据见
/// 2026-10-14 重录依据（v273 都市王批，T11 十殿阎罗）：普通批——规则版
/// 本 272→273（热闷/炙焚/火刑三技+都市王条目；显示名勘误：炙焚[非炽
/// 焚]/火刑[非祸刑]）。热闷=PlayCard 拒绝位+AOE 机窗口前跳过双消费，炙
/// 焚=Prepare-begin 臂（持有者域），火刑=Death 遗言逐座机（BossHuoxing
/// 入重建表）。版本回退探针（273→272 复跑 golden 全绿）证明位移唯一来
/// 源是版本字段。证据见
/// 2026-10-14 重录依据（v274 地藏王批，T11 十殿阎罗）：普通批——规则版
/// 本 273→274（不佛/无量/谛听三技+地藏王条目；显示名勘误 boss_bufo=不
/// 佛；大愿=判定改写机 AI 态度面登记不落，无量建局摸 3 面登记未落）。
/// 不佛=伤害入口减 1+准备距离 1 火伤，无量=End 摸 2+准备回复，谛听=
/// computed_distance 双向链（义从同位）。版本回退探针（274→273 复跑
/// golden 全绿）证明位移唯一来源是版本字段。证据见
/// 2026-10-14 重录依据（v275 楚江王批，T11 十殿阎罗）：普通批——规则版
/// 本 274→275（帷幕/冰封两新技+楚江王条目[反馈=标准 FAN_KUI 复用，原生
/// 语义=任意伤害+手牌/装备与界反馈一致]）。**勘正**：此前批次证据把楚
/// 江王误计入已完成（目录实无楚江王条目），本批实际落包后十殿阎罗为
/// 11/13。帷幕=黑锦囊 PlayCard 拒绝位+黑色 AOE 窗口前跳过，冰封=Death
/// 遗言翻面 killer 机（BossBingfeng 入重建表）。版本回退探针（275→274
/// 复跑 golden 全绿）证明位移唯一来源是版本字段。证据见
/// 2026-10-14 重录依据（v276 秦广王批，T11 十殿阎罗）：普通批——规则版
/// 本 275→276（判官/拘魂/望乡三技+秦广王条目；还人=关卡流程面登记不
/// 落）。判官=延时锦囊 PlayCard 拒绝位，拘魂=End 臂恒定翻面首个其他，
/// 望乡=Death 遗言全场装备全弃机（BossWangxiang 入重建表，绝红拆装管
/// 线同款）。版本回退探针（276→275 复跑 golden 全绿）证明位移唯一来源
/// 是版本字段。证据见
/// 2026-10-14 重录依据（v277 庞统片，T11 群雄乱入 Boss 专项）：普通批
/// ——规则版本 276→277（天狱/栖梧两技+庞统条目[涅槃凤雏，涅槃/浴火=
/// 外部包技依赖登记不落；上游 group shen→群近似]）。天狱=End 臂自横置
/// 先解除+全场横置，栖梧=CardUsed commit 臂梅花回复（娇孝同族）。诸葛
/// 亮（hp Infinity+四技 offline 龙血玄黄包）同批勘验登记不落。版本回退
/// 探针（277→276 复跑 golden 全绿）证明位移唯一来源是版本字段。证据见
/// docs/operations/evidence/2026-10-14-t11-boss-pangtong-v277/。
/// docs/operations/evidence/2026-10-14-t11-boss-qinguangwang-v276/。
/// docs/operations/evidence/2026-10-14-t11-boss-chujiangwang-v275/。
/// docs/operations/evidence/2026-10-14-t11-boss-dizangwang-v274/。
/// docs/operations/evidence/2026-10-14-t11-boss-dushiwang-v273/。
/// docs/operations/evidence/2026-10-14-t11-boss-pingdengwang-v272/。
/// docs/operations/evidence/2026-10-14-t11-boss-yanluowang-v271/。
/// docs/operations/evidence/2026-10-14-t11-boss-mengpo-v270/。
/// docs/operations/evidence/2026-10-13-t11-boss-bianchengwang-v267/。
/// 2026-10-11 重录依据（v266 宋帝王批，T11 十殿阎罗）：普通批——规则版
/// 本 265→266（黑绳/胜负两技+宋帝王条目；恩怨=外部包组技未落登记随外
/// 部包批）。黑绳=Death 前置窗口遗言面首例（changan 氏同款；死亡被取
/// 消仍已横置=登记差异）。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-songdiwang-v266/。
/// 2026-10-11 重录依据（v265 五官王批，T11 十殿阎罗）：普通批——规则版
/// 本 264→265（枳枉/公正/血池三技+五官王条目，十殿阎罗首片）。枳枉=
/// CardsMoved 面（摸牌走 CardsDrawn 天然隔离=上游排除 draw 同口径），
/// 随机→首张确定性近似登记。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-wuguanwang-v265/。
/// 2026-10-11 重录依据（v264 张角批，T11 群雄乱入）：字段批——规则版
/// 本 263→264（天道/雷击·张角/极点三技+张角条目）+TurnState.jidian_chain
/// 入摘要（存档 v60→v61）。字段批探针：版本回退 263 复跑 14 场景哈希三
/// 值互异（263 无字段/263+字段/264+字段），证明位移=版本+字段双源、无行
/// 为面扰动。雷击·张角与青龙 BOSS_RELEIJI 同名按迅影批改名映射先例。证
/// 据见 docs/operations/evidence/2026-10-11-t11-boss-zhangjiao-v264/。
/// 2026-10-11 重录依据（v263 周瑜批，T11 群雄乱入）：普通批——规则版
/// 本 262→263（火神/红莲·周瑜/仙音三技+周瑜条目）。版本回退探针（263→
/// 262 复跑 golden 全绿）证明位移唯一来源是版本字段。红莲与毕方
/// BOSS_HONGLIANX 同名按迅影批改名映射先例以「红莲·周瑜」入目录。证据
/// 见 docs/operations/evidence/2026-10-11-t11-boss-zhouyu-v263/。
/// 2026-10-11 重录依据（v262 甄姬批，T11 群雄乱入）：普通批——规则版
/// 本 261→262（踏水/凌波/娇孝/芳华四技+甄姬条目）。版本回退探针（262→
/// 261 复跑 golden 全绿）证明位移唯一来源是版本字段。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-zhenji-v262/。
/// 2026-10-11 重录依据（v261 刘备批，T11 群雄乱入）：普通批——规则版
/// 本 260→261（枭雄/章武+刘备条目）。版本回退探针（261→260 复跑 golden
/// 全绿）证明位移唯一来源是版本字段；Boss 分支不在标准场景可达路径。
/// 证据见 docs/operations/evidence/2026-10-11-t11-boss-liubei-v261/。
/// 2026-10-11 重录依据（v260 张春华批，T11 群雄乱入）：普通批——规则版
/// 本 259→260（绝情/无心/伤逝三技+张春华条目）。版本回退探针（260→259
/// 复跑 golden 全绿）证明位移唯一来源是版本字段；Boss 分支不在标准场景
/// 可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-zhangchunhua-v260/。
/// 2026-10-11 重录依据（v259 郭嘉批，T11 群雄乱入）：字段批——规则版
/// 本 258→259（鬼谋/远策/奇佐三技+郭嘉条目）+PlayerState.boss_mad+
/// TurnState.qizuo_copy 入摘要（存档 v60）。字段批探针：版本回退 258
/// 复跑 14 场景哈希三值互异（旧 258 无字段/258+字段/259+字段），证明
/// 位移=版本字段+新增字段双源、无行为面扰动；Boss 郭嘉分支不在标准
/// 场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-11-t11-boss-guojia-v259/。
/// 2026-10-14 重录依据（T8 增量 14，v277→278）：字段批——TurnState
/// .bte_used_play_phase（出牌阶段基本/锦囊/装备三类计数，DSL filter
/// `*_play_phase_card_category_count_positive` 承载）入哈希（存档
/// v64→v65）。字段批探针：版本回退 277 复跑 14 场景哈希三值互异
/// （旧 277 无字段/277+字段/278+字段），证明位移=版本字段+新增字段
/// 双源、无行为面扰动；标准场景不装载扩展 VM 技能，双策略不在可达
/// 路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-14-t8-increment14-bte-filters/。
/// 2026-10-14 重录依据（T12 玩点论杀批 14，v278→279）：字段批——
/// PlayerState.fuchou_marked（复仇跨回合来源标记）+ShaResolveMachine
/// asking_fuchou/fuchou_card 两字段入哈希（存档 v66→v67）。字段批探针：
/// 版本回退 278 复跑 14 场景哈希三值互异（旧 278 无字段/278+字段/279+
/// 字段），证明位移=版本字段+新增字段双源、无行为面扰动；复仇分支不在
/// 标准场景可达路径，逐场景命令与断言不变。证据见
/// docs/operations/evidence/2026-10-14-t12-wudian-generals-b14/。
/// 2026-10-14 重录依据（T12 玩点论杀批 17，v279→280）：字段批——
/// PlayerState.guihan_used（归汉 limited 每局一次）+DyingSaveMachine
/// guihan_asked 快照字段入哈希（存档 v67→v68）。字段批探针：版本回退
/// 279 复跑 14 场景三值互异；归汉濒死确认不在标准场景可达路径。证据见
/// docs/operations/evidence/2026-10-14-t12-yunchou-generals-b17/。
/// 2026-10-15 重录依据（T12 swd 武将面批 26，v280→281）：字段批——
/// PlayerState.xuehuang_used（雪煌 limited 每局一次）入存档（存档
/// v68→v69）。版本回退探针：280+字段复跑 14 场与旧 golden **逐字节一
/// 致**——golden 位移唯一来源=规则版本字段，新增字段不在 golden 哈希
/// 域且雪煌不在标准场景可达路径，行为面零扰动。证据见
/// docs/operations/evidence/2026-10-15-t12-swd-generals-b26/。
/// 2026-10-15 重录依据（T12 swd 卡牌面补切批⑧，v285→286）：龟灵之桃
/// 新 kind + PlayerState.nigong_lv 字段（存档 v72→v73）。探针：285+字段
/// 复跑 14 场与旧 golden 逐字节一致——位移唯一来源=规则版本字段，行为
/// 面零扰动。证据见 docs/operations/evidence/2026-10-15-t12-swd-cards-equip5-b38/。
/// 2026-10-15 重录依据（T12 swd 卡牌面补切批⑦，v284→285）：锦囊族 4
/// 新 kind（惊雷山/炽羽袭 AOE 雷火版 + 鬼蛇旗/仙露回护甲锦囊）+
/// MachineSnapshot::Aoe/AoeTarget nature 字段（存档 v71→v72）。探针：
/// 284+字段复跑 14 场与旧 golden 逐字节一致——位移唯一来源=规则版本
/// 字段，行为面零扰动。证据见
/// docs/operations/evidence/2026-10-15-t12-swd-cards-trick-b37/。
/// 2026-10-15 重录依据（T12 swd 卡牌面补切批③，v283→284）：炼妖壶新
/// kind（tag 143 宝物栏）+PlayerState.lianyaohu_store 仓字段（存档
/// v70→v71）+收纳/炼化/壶境三技。探针：283+字段复跑 14 场与旧 golden
/// 逐字节一致——位移唯一来源=规则版本字段，行为面零扰动。证据见
/// docs/operations/evidence/2026-10-15-t12-swd-cards-equip5-b33/。
/// 2026-10-15 重录依据（T12 swd 卡牌面补切批②，v282→283）：新 basic
/// kind 雪霁冰雹/封印之丹（神秘果 kind 同批注册、卡牌挂窗口机批）+
/// PlayerState.xuejibingbao 字段（存档 v69→v70）。探针：282+字段复跑
/// 14 场与旧 golden 逐字节一致——位移唯一来源=规则版本字段，行为面零
/// 扰动。证据见 docs/operations/evidence/2026-10-15-t12-swd-cards-basic-b31/。
/// 2026-10-15 重录依据（T12 swd 武将面批 27，v281→282）：TurnState.
/// huanxia_track（还侠转化杀跟踪，kangyin2 同位入哈希）+还侠 viewAs 新
/// 技。探针：281+字段复跑 14 场位移值与 282 值及旧 golden 三值互异——
/// 位移来源=版本字段+新字段双源（还侠不在标准场景可达路径，行为面零扰
/// 动）。证据见 docs/operations/evidence/2026-10-15-t12-swd-generals-b27/。
/// 2026-10-15 重录依据（T12 swd 武将面批㉑，v286→287）：TurnState
/// mufeng_used_basic/wangchen_discard_basic 两旗（疾影/沐风/望尘三将技，
/// 存档 v72→v73）。探针：286+字段复跑 14 场与旧 golden 逐字节一致——位
/// 移唯一来源=规则版本字段，行为面零扰动。证据见
/// docs/operations/evidence/2026-10-15-t12-swd-generals-b61/。
const SCENARIOS: &[(&str, ScenarioFn, &[&str], &str)] = &[
    (
        "sha_dodge_then_damage",
        scenario_sha_dodge_then_damage,
        &["card_use", "sha_respond", "damage_commit", "sha_limit", "seat_range"],
        "635c7767e08b827f03b7c93091621f49546eb5f33f0c4e3de064dded71520f91",
    ),
    (
        "death_terminal_single",
        scenario_death_terminal_single,
        &["dying", "death", "victory"],
        "f8d7e0e2e75476160612546315df3c70c9198a70cc3e0454fc496891b46b49d4",
    ),
    (
        "sancai_nested_chain",
        scenario_sancai_nested_chain,
        &["optional_trigger", "nested_choice", "draw", "give_cards"],
        "075ac947720420ea50d983729c51f728691639e5bcc752667f4c6c6ebb36e750",
    ),
    (
        "yishang_redirect",
        scenario_yishang_redirect,
        &["damage_before_window", "payload_rewrite_redirect"],
        "9a46ec4259f46be0a0dcd0fa3ece8b1fb6143240bcecfc18552a3d94bd620be8",
    ),
    (
        "suidun_window_matrix",
        scenario_suidun_window_matrix,
        &["damage_prevent", "once_per_turn", "mid_resolution_disable"],
        "b139f61805375f866638e89213fc1d11d2d8682a2d7dd1107a422865fa0c237c",
    ),
    (
        "shouyi_dynamic_skill",
        scenario_shouyi_dynamic_skill,
        &["dynamic_gain", "skill_expiry", "redirect_after_gain"],
        "9bb983bcb91877bfab94b821eca4cd9528b698881b51b6b8e9a5ac45dc852aff",
    ),
    (
        "discard_to_limit",
        scenario_discard_to_limit,
        &["hand_limit", "intrinsic_machine"],
        "73897db89c00ef1ab17faa759a362788fde3ee925e121615910f94040d2b5693",
    ),
    (
        "rollback_invariants",
        scenario_rollback_invariants,
        &["reject_rollback", "budget_rollback"],
        "7fbb80c8399965dbada1cdfa402b09bbfdeadd8d28787bc8d295e12e871bb6cb",
    ),
    (
        "equip_distance_matrix",
        scenario_equip_distance_matrix,
        &["equip_slots", "distance_modifier", "attack_range", "equip_replace"],
        "98660fbf4fba0c3cd8ba23295b68cd96570d763e85a2d57655b4afb69969fc2a",
    ),
    (
        "chain_fire_propagation",
        scenario_chain_fire_propagation,
        &["chain_state", "damage_nature", "chain_propagation"],
        "3c8ef78264e942f2940e27c2ef26ce44c4457002324601c053f68e17b03b0753",
    ),
    (
        "aoe_response_queue",
        scenario_aoe_response_queue,
        &["aoe_queue", "aoe_skip_no_card", "aoe_full_resolution"],
        "2af6a97451178933ecbe21c014900255d158463c6bc9f1a4b34e1429847488c2",
    ),
    (
        "duel_alternating_response",
        scenario_duel_alternating_response,
        &["duel_alternating", "duel_no_distance_limit"],
        "a136617641b9f7d17dec57ce94a48c34f2cd3f67c6f88daf253f15d2bc3d5cd9",
    ),
    (
        "nullify_chain_on_savage",
        scenario_nullify_chain_on_savage,
        &["nullify_window", "nullify_chain", "nullify_scope"],
        "9b514bc6a4617dd0d1a867d408ef818e87000311a72f4f24f67cbbb18653ae86",
    ),
    (
        "judge_phase_delayed_tricks",
        scenario_judge_phase_delayed_tricks,
        &["judge_zone", "delayed_trick", "phase_skip"],
        "1f64be931d062fbc1bbafbb69cb483177180e0d24216bb4dbd30d636fedcbe5d",
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
