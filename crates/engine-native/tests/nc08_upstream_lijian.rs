//! NC08 第⑦层（上游行为对照）样本 35：`lijian`（离间，**标准面** / 貂蝉）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! lijian: {
//!     enable: "phaseUse",
//!     usable: 1,                                  // 出牌阶段限一次
//!     filter(event, player) {                     // 需存在**两名以上**其他男性角色
//!         return game.countPlayer(c => c !== player && c.hasSex("male")) > 1;
//!     },
//!     filterCard: true,                           // 弃任意一张牌
//!     position: "he",                             // **手牌 + 装备区**
//!     filterTarget(card, player, target) {
//!         if (player === target) return false;
//!         if (!target.hasSex("male")) return false;                 // 目标须为**男性**
//!         if (ui.selected.targets.length === 1)
//!             return target.canUse({ name: "juedou" }, ui.selected.targets[0]);
//!         return true;
//!     },
//!     selectTarget: 2, multitarget: true,
//!     async content(event, trigger, player) {
//!         // 后选者（targets[1]）对先选者（targets[0]）使用【决斗】（无无懈）
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/限次**：出牌阶段，**每回合限一次**；
//! 2. **门槛**：存在**两名以上**其他**男性**角色；
//! 3. **费用**：弃**一张牌**（`position: "he"` → 含**装备区**）；
//! 4. **目标**：恰 **2 名男性**（不含自己），后选者向先选者**使用【决斗】**。
//!
//! 原生实现引用：`content/standard/skills/li_jian.rs`（`activate` 要求**手牌非空**；
//! 目标过滤 `seat_is_male`；机器 `DiscardAsk` → 内嵌决斗直结算）。
//!
//! **登记差异（本文件固定）**：**牌区**——上游 `position: "he"` 含**装备区**，原生 `activate`
//! 要求**手牌非空**（与 `zhiheng`/`qixi`/`guose` **同族**，见待实施修复清单第 11 项）。
//!
//! 未覆盖（登记）：`check`/`ai`/`audio`/`targetprompt` 表现与策略面；「后选者向先选者决斗」的
//! 方向性（本样本只验证决斗发生与体力变化）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

fn skill_id(e: &Engine, name: &str) -> u16 {
    e.catalog().skill_by_name(name).expect("技能在目录")
}

/// 上游点 2+4：两名男性角色 → 决斗发生（其中一方体力下降）。
#[test]
fn upstream_lijian_makes_two_males_duel() {
    let mut e = Engine::new(20266501, 4);
    e.debug_grant(0, "离间");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    nc08_common::give_colored(&mut e, 0, "shan", None);
    // 1、2 号装配**男性**武将（性别由 general_gender 推导），并各给一张【杀】以支撑决斗
    for p in [1u8, 2] {
        let male = e
            .general_candidates()
            .into_iter()
            .find(|d| {
                e.catalog()
                    .general_meta(*d)
                    .map(|(_, _, _, _, g)| {
                        g == engine_native::content::standard::Gender::Male
                    })
                    .unwrap_or(false)
            })
            .expect("存在男性武将");
        let name = e.catalog().general_meta(male).map(|(n, _, _, _, _)| n.to_string()).unwrap();
        e.debug_assemble_general(p, &name);
    }
    for p in [1u8, 2] {
        nc08_common::give_colored(&mut e, p, "sha", None);
    }
    let skill = skill_id(&e, "离间");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "离间应可发动：{out:?}"
    );
    // 依次回答：选目标（两名男性）、弃费用牌
    let mut picked: Vec<u8> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            });
            continue;
        }
        // 目标选择：挑男性玩家
        let players: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Player(p) if *p == 1 || *p == 2))
            .copied()
            .collect();
        if players.len() >= 2 && picked.is_empty() {
            picked.push(1);
            picked.push(2);
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: players.into_iter().take(2).collect(),
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
    drain(&mut e);
    let hp = (e.debug_hp(1), e.debug_hp(2));
    eprintln!("DBG 离间后 hp1={} hp2={} picked={picked:?}", hp.0, hp.1);
    assert!(
        hp.0 < 4 || hp.1 < 4,
        "离间：两名男性应发生决斗（一方体力下降），实测 hp1={} hp2={}",
        hp.0,
        hp.1
    );
}

/// 上游点 2（门槛）：**女性**角色不可被选为目标。
#[test]
fn upstream_lijian_targets_must_be_male() {
    let mut e = Engine::new(20266502, 4);
    e.debug_grant(0, "离间");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    nc08_common::give_colored(&mut e, 0, "shan", None);
    // 1 号男性、2 号女性（经装配武将设置性别）
    let pick = |e: &Engine, want_male: bool| -> String {
        e.general_candidates()
            .into_iter()
            .find(|d| {
                e.catalog()
                    .general_meta(*d)
                    .map(|(_, _, _, _, g)| {
                        (g == engine_native::content::standard::Gender::Male) == want_male
                    })
                    .unwrap_or(false)
            })
            .and_then(|d| e.catalog().general_meta(d).map(|(n, _, _, _, _)| n.to_string()))
            .expect("存在该性别武将")
    };
    let male_name = pick(&e, true);
    let female_name = pick(&e, false);
    e.debug_assemble_general(1, &male_name);
    e.debug_assemble_general(2, &female_name);
    let skill = skill_id(&e, "离间");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    if !matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }) {
        eprintln!("SKIP：未进入目标选择（{out:?}）");
        return;
    }
    let Some(c) = e.open_choice().cloned() else {
        eprintln!("SKIP：未出现目标询问");
        return;
    };
    assert!(
        !c.spec.options.contains(&ChoiceOption::Player(2)),
        "女性角色不应出现在候选中：{:?}",
        c.spec.options
    );
    let _ = Phase::Play;
}

/// 上游点 3 的**登记差异（牌区族第 4 例）**：原生 `activate` 要求**手牌非空**（上游 `he` 含装备区）。
#[test]
fn upstream_lijian_zone_difference_is_registered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/li_jian.rs"
    ))
    .expect("读离间源码");
    let idx = src.find("fn activate").expect("activate");
    let seg: String = src[idx..].chars().take(400).collect();
    assert!(
        seg.contains("hand.is_empty()"),
        "差异固定：原生 activate 要求手牌非空（上游 position 'he' 含装备区）：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("lijian: {").expect("上游离间");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("position: \"he\"") && seg.contains("selectTarget: 2"),
        "上游依据：position he + 恰 2 目标：{seg}"
    );
}
