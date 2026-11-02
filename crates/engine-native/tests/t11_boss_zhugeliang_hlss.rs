//! T11 Boss 诸葛亮族（hlss 四技 + 无限体力）：禳星/隐月/隐月·判/起风/改命。
//!
//! 真实包 `packages/extensions/boss-zhugeliang-upstream-liuli`（来源
//! liuli `character/extra.js` + `mode/boss.js`）；本文件同时覆盖包装载、
//! 装配（无限体力 + 3 星）与五技行为面。

use engine_native::event::DamageNature;
use std::sync::atomic::{AtomicU64, Ordering};

/// 进程级去重键（命令信任边界要求严格递增）。
static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}
use engine_native::state::INFINITE_HP;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn general_def_of(e: &Engine, name: &str) -> u16 {
    for def in 0..=u16::MAX {
        if e.catalog().general_meta(def).map(|g| g.0) == Some(name) {
            return def;
        }
        if e.catalog().general_meta(def).is_none() && def > 64 {
            break;
        }
    }
    panic!("目录中无武将 {name}");
}

/// 从真实包建局并把 `boss_zhugeliang` 装到 0 号（无限体力 + 3 星）。
fn setup(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("boss-zhugeliang-upstream-liuli/assets/cards.json"),
    )
    .expect("读 Boss 诸葛亮包数据");
    let bridged = content_native_loader::bridge_pack(&json).expect("装载 Boss 诸葛亮包");
    let pack = bridged.pack;
    let mut e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "boss_zhugeliang");
    e.debug_assemble_general_def(0, def);
    e
}

/// 应答所有开放选择直到静止：可选类（含 Pass 或 allow_cancel）一律**放弃**
/// （避免起风/改命等可选技干扰被测路径），强制类取首 min 项。
fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel || c.spec.min == 0 {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

/// 装配初始化：3 星、计数 0、无受伤记录；隐月 +1 与禳星光回满并存。
#[test]
fn hlss_assembly_initializes_stars_and_light_refill() {
    let mut e = setup(92001);
    // 本测关注星系统与「光」的体力回满面：把无限体力改为有限（4）以便观察；
    // 无限体力形态由 `hlss_with_infinite_hp_keeps_stars` 覆盖。
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 4);
    assert_eq!(e.state_core().player(0).hlss_stars, 3, "装配 3 星");
    assert_eq!(e.state_core().player(0).hlss_count, 0);
    assert_eq!(e.state_core().player(0).hlss_damage, 0);
    let xd = e
        .catalog()
        .skill_by_name("hlss_xiangxing")
        .expect("禳星在目录");
    let yd = e
        .catalog()
        .skill_by_name("hlss_yueyin")
        .expect("隐月在目录");
    assert!(e.state_core().skills.contains_key(&(0, xd)), "禳星实例");
    assert!(e.state_core().skills.contains_key(&(0, yd)), "隐月实例");

    // 隐月：1 点伤害 +1 = 2 点；禳星光：体力回满（受 2 点仍 4/4）。
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 4, "光：受 2 点伤害后回满");
    assert_eq!(e.state_core().player(0).hlss_count, 2, "计数按结算值累加（1+隐月 1）");
    assert_eq!(e.state_core().player(0).hlss_stars, 3, "未满 3 不失星");

    // 再受 1 点 → 计数 4 ≥ 3 → 失星（3→2）并分配 3 点雷伤给其他角色。
    let hp_before: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hlss_stars, 2, "失一星");
    assert_eq!(e.state_core().player(0).hlss_count, 0, "失星清零计数");
    let hp_after: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    let dealt: i32 = hp_before
        .iter()
        .zip(hp_after.iter())
        .map(|(a, b)| a - b)
        .sum();
    assert_eq!(dealt, 3, "雷伤分配共 3 点（随机 1 + 随机敌 2）");
    assert!(hp_after.iter().zip(hp_before.iter()).any(|(a, b)| a < b), "至少一人受伤");
    assert_eq!(e.debug_hp(0), 4, "禳星持有者不吃自己的雷伤");
}

/// 隐月免除：当前星数对应伤害类型已在记录中 → 不失星、不放雷。
#[test]
fn hlss_yueyin_skip_prevents_star_loss() {
    let mut e = setup(92002);
    // 先受雷伤：记录雷位；计数 2（1 + 隐月 1）
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Thunder);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hlss_damage & 1,
        1,
        "记录雷属性伤害"
    );
    assert_eq!(e.state_core().player(0).hlss_count, 2);
    // 再受普通伤害：计数 4 ≥ 3 → 三星等级要求雷记录 → 免除
    let hp_before: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hlss_stars, 3, "免除：不失星");
    assert_eq!(e.state_core().player(0).hlss_count, 4, "免除：计数保留");
    let hp_after: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    assert_eq!(hp_before, hp_after, "免除：不放雷");
    // 无隐月时不免除（移除隐月实例后同样流程照常失星）。
    let yd = e.catalog().skill_by_name("hlss_yueyin").expect("隐月");
    e.state_core_mut().skills.remove(&(0, yd));
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hlss_stars, 2, "无隐月：照常失星");
}

/// 星归零后觉醒：不再回满体力，伤害可致死；记录位图在失星时清空。
#[test]
fn hlss_zero_stars_awakens_and_stops_refill() {
    let mut e = setup(92003);
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 4);
    // 三星耗尽：每次 3 点（1 + 隐月 1 + 2 点补足）
    for _ in 0..3 {
        // 普通伤害：三星等级要求雷记录、二星要求火、一星要求锦囊——普通
        // 伤害不记入任何一位，故三次都照常失星。
        let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
        drain(&mut e);
    }
    assert_eq!(e.state_core().player(0).hlss_stars, 0, "三星耗尽");
    assert_eq!(e.state_core().player(0).hlss_damage, 0, "失星清空记录");
    let xd = e
        .catalog()
        .skill_by_name("hlss_xiangxing")
        .expect("禳星");
    assert!(
        !e.state_core()
            .skills
            .get(&(0, xd))
            .map(|i| i.enabled)
            .unwrap_or(false),
        "觉醒：禳星禁用"
    );
    // 觉醒后不再回满：2 点伤害 + 隐月 1 = 3 → 体力 4→1
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 1, "觉醒后伤害照常（不再回满）");
}

/// 隐月·判：不能被翻面；判定区的牌对持有者不生效（乐不思蜀不跳阶段）。
#[test]
fn hlss_yueyin2_blocks_turnover_and_voids_delayed_trick() {
    let mut e = setup(92004);
    // 翻面被取消：debug_flip 返回是否发生翻面
    assert!(!e.debug_flip(0), "隐月·判：翻面被取消");
    assert!(!e.state_core().player(0).face_down, "未翻面");
    // 移除隐月·判后照常翻面（对照）
    let y2 = e
        .catalog()
        .skill_by_name("hlss_yueyin2")
        .expect("隐月·判");
    e.state_core_mut().skills.remove(&(0, y2));
    assert!(e.debug_flip(0), "对照：无隐月·判照常翻面");
    e.debug_set_face_down(0, false);
    // 复位隐月·判实例并验证判定区牌不生效
    e.debug_grant(0, "hlss_yueyin2");
    e.debug_place_judge(0, "le");
    // 判定牌设为黑桃（非红桃 → 照常会跳过错开阶段）
    e.debug_set_deck_top_by(|d| d.suit == engine_native::catalog::Suit::Spade);
    // 进入 0 号回合的判定阶段：把回合交给 3 号并结束其回合，NextPhase
    // 使回合转回 0 号 → 回合开始 → Judge 阶段结算判定区延时锦囊。
    e.debug_set_turn(3, Phase::End, e.turn().round);
    drain(&mut e);
    let out = e.submit(GameCommand {
        actor: 3,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(
        e.debug_judge(0).is_empty(),
        "判定区延时锦囊已结算（隐月·判：判定不生效）"
    );
    assert!(
        !e.state_core().turn.skip_play_phase,
        "隐月·判：乐不思蜀判定不生效（不跳阶段）"
    );
    // 对照：移除隐月·判后同一判定照常跳阶段
    let y2 = e.catalog().skill_by_name("hlss_yueyin2").expect("隐月·判");
    e.state_core_mut().skills.remove(&(0, y2));
    e.debug_place_judge(0, "le");
    e.debug_set_deck_top_by(|d| d.suit == engine_native::catalog::Suit::Spade);
    e.debug_set_turn(3, Phase::End, e.turn().round);
    drain(&mut e);
    let out = e.submit(GameCommand {
        actor: 3,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(
        e.debug_judge(0).is_empty(),
        "对照：判定区延时锦囊已结算"
    );
    assert!(
        e.state_core().turn.skip_play_phase,
        "对照：无隐月·判照常跳阶段"
    );
}

/// 无限体力哨兵与星系统并存（Boss 诸葛亮族完整形态）：体力不变、星照常掉。
#[test]
fn hlss_with_infinite_hp_keeps_stars() {
    let mut e = setup(92005);
    assert_eq!(e.state_core().player(0).max_hp, INFINITE_HP, "包 hp=Infinity 装配");
    assert_eq!(e.debug_hp(0), INFINITE_HP as i32);
    for _ in 0..3 {
        let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
        drain(&mut e);
    }
    assert_eq!(e.debug_hp(0), INFINITE_HP as i32, "无限体力不变");
    assert_eq!(e.state_core().player(0).hlss_stars, 0, "星照常耗尽");
}

/// 应答直到出现指定 prompt 并用给定选择应答（其余按可选放弃处理）。
fn answer_prompt(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "应答 {prompt} 被拒：{out:?}"
            );
            return true;
        }
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let fallback: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: fallback },
        });
    }
    false
}

/// 把回合交给 0 号（从 3 号结束阶段推进）并应答到静止。
fn start_turn_of_zero(e: &mut Engine) {
    e.debug_set_turn(3, Phase::End, e.turn().round);
    drain(e);
    let out = e.submit(GameCommand {
        actor: 3,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
}

/// 起风：准备阶段确认后从可用锦囊牌名中选一张并使用（用过的牌名不再出现）；
/// 放弃则不入账。
#[test]
fn hlss_fengqi_picks_unused_trick_and_records() {
    use engine_native::catalog::CardKind;

    let mut e = setup(93001);
    e.debug_clear_hand(0);
    let wuzhong = engine_native::archive::kind_code(CardKind::Wuzhong);
    // 起风窗口：准备阶段（Judge 开始）→ 确认 → 选牌名 → 使用
    start_turn_of_zero(&mut e);
    assert!(
        answer_prompt(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "起风确认窗"
    );
    let hand_before = e.debug_hand(0).len();
    assert!(
        answer_prompt(&mut e, "fengqi_card", vec![ChoiceOption::Kind(wuzhong)]),
        "起风选牌名窗"
    );
    drain(&mut e);
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before + 2,
        "无中生有：使用后摸 2（虚拟使用不消耗手牌）"
    );
    assert_eq!(
        e.state_core().player(0).hlss_fengqi_used,
        vec![wuzhong],
        "已用牌种入账"
    );

    // 第二次准备阶段：候选不再包含已用牌种（确认后读候选表再选首项）。
    start_turn_of_zero(&mut e);
    let c = e.open_choice().cloned().expect("起风确认窗");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    let pick = e.open_choice().cloned().expect("起风选牌名窗");
    assert!(
        !pick.spec.options.contains(&ChoiceOption::Kind(wuzhong)),
        "已用牌种不再出现：{:?}",
        pick.spec.options
    );
    let second = pick.spec.options[0];
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: pick.id, selection: vec![second] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    let mut expected = vec![wuzhong];
    if let ChoiceOption::Kind(k) = second {
        expected.push(k);
    }
    assert_eq!(
        e.state_core().player(0).hlss_fengqi_used,
        expected,
        "已用牌种累计（用过的牌名不再出现）"
    );
    assert!(
        e.state_core().skills.contains_key(&(
            0,
            e.catalog().skill_by_name("hlss_fengqi").expect("起风")
        )),
        "起风可多次发动（未限次）"
    );
}

/// 起风（放弃确认）：不发动、不使用、不入账。
#[test]
fn hlss_fengqi_decline_does_nothing() {
    let mut e = setup(93002);
    e.debug_clear_hand(0);
    start_turn_of_zero(&mut e);
    let c = e.open_choice().cloned().expect("起风确认窗");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
    });
    drain(&mut e);
    assert!(e.state_core().player(0).hlss_fengqi_used.is_empty());
    assert_eq!(e.debug_hand(0).len(), 0, "不摸牌");
}

/// 改命：判定前亮牌堆顶 7 张选 1 作为判定结果（其余弃置、跳过正常判定）；
/// 放弃则 7 张回牌堆顶、照常判定。
#[test]
fn hlss_gaiming_picks_judge_result_or_declines() {
    use engine_native::catalog::{CardKind, Suit};

    // 路径 A：选红桃（乐不思蜀不生效：不跳阶段），其余 6 张入弃堆。
    let mut e = setup(93003);
    e.debug_place_judge(0, "le");
    e.debug_set_deck_top_by(|d| d.suit == Suit::Spade);
    start_turn_of_zero(&mut e);
    // 先放弃起风确认门（可选触发），进入改命窗
    let gate = e.open_choice().cloned().expect("起风确认门");
    assert_eq!(gate.spec.prompt, "trigger_confirm");
    let _ = e.submit(GameCommand {
        actor: gate.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: gate.id, selection: Vec::new() },
    });
    let c = e.open_choice().cloned().expect("改命窗");
    assert_eq!(c.spec.prompt, "gaiming_pick");
    assert_eq!(c.spec.options.len(), 7.min(7), "亮 7 张候选");
    let heart: Option<ChoiceOption> = c.spec.options.iter().copied().find(|o| match o {
        ChoiceOption::Card(id) => e.catalog().cards[*id as usize].suit == Suit::Heart,
        _ => false,
    });
    let sel: Vec<ChoiceOption> = match heart {
        Some(h) => vec![h],
        None => vec![c.spec.options[0]],
    };
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c.id, selection: sel },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(
        !e.state_core().turn.skip_play_phase,
        "选中红桃：乐不思蜀不生效"
    );
    assert!(e.debug_judge(0).is_empty(), "判定区已结算");
    assert!(e.debug_discard_len() > 0, "未选中的判定候选入弃堆");

    // 路径 B：放弃 → 7 张回牌堆顶、照常判定（黑桃 → 乐生效跳阶段）。
    let mut e = setup(93004);
    e.debug_place_judge(0, "le");
    e.debug_set_deck_top_by(|d| d.suit == Suit::Spade && d.kind == CardKind::Sha);
    let deck_before = e.state_core().deck.len();
    start_turn_of_zero(&mut e);
    assert!(
        answer_prompt(&mut e, "gaiming_pick", Vec::new()),
        "改命窗（放弃）"
    );
    drain(&mut e);
    // 规则 332（裁决 24）：放弃后 7 张回牌堆顶，再由正常判定初始化抽走
    // 恰一张（原首候选）。修复前此处断言 deck_before 并把重复实体锁成
    // 期望（牌 65 既在牌堆又被结算弃置）；见 2026-11-03 红→绿证据。
    assert_eq!(
        e.state_core().deck.len(),
        deck_before - 1,
        "放弃：7 张回牌堆顶后正常判定消耗一张（登记差异：上游滞留处理区）"
    );
    assert!(e.debug_judge(0).is_empty(), "判定区已结算（照常判定）");
    assert!(
        !e.state_core().turn.skip_play_phase,
        "隐月·判在位：乐不思蜀判定不生效"
    );
}
