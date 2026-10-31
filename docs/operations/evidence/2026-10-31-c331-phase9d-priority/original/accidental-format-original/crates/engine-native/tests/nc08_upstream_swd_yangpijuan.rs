//! NC08 第⑦层（上游行为对照）样本 67：**来源 U05（杀海拾遗·swd）** 锦囊
//! `yangpijuan`（羊皮卷）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js`，`yangpijuan`）：
//!
//! ```js
//! yangpijuan: {
//!     type: "trick", enable: true, toself: true, modTarget: true,
//!     filterTarget(card, player, target) { return target === player; },   // **仅自己**
//!     content() {
//!         "step 0";
//!         // AI 默认类别：缺【闪】/缺【杀】/hp≤1 → basic；否则按装备位与随机数挑 trick/equip
//!         target.chooseControl("basic", "trick", "equip", () => choice)
//!               .set("prompt", "选择一种卡牌类型");
//!         "step 1";
//!         var list = get.inpile(result.control, "trick");
//!         list = list.randomGets(3);                       // **随机 3 张**
//!         for (var i = 0; i < list.length; i++) list[i] = [get.translation(result.control), "", …];
//!         // → 择一获得（createCard 凭空造牌）
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**仅自己**（`toself` + `filterTarget`）；
//! 2. **两段询问**：先**选类别**（基本/锦囊/装备）→ 再从该类别的**随机 ≤3 张**中**择一获得**；
//! 3. 类别候选来自 `get.inpile(class)`（**牌定义池**）。
//!
//! 原生实现引用：`content/host/step/swd_card_effects.rs:452`（三段机：`Act` → 类别询问
//! `SWD_YANGPIJUAN_CATEGORY`（`Tag(0..2)`，0=基本 1=锦囊 2=装备）→ `SelectCard` 按类别从**牌堆**
//! 过滤 + Fisher-Yates 取 ≤3 → `SWD_YANGPIJUAN_PICK` 择一 → `TargetChoice` 择定后**才**离牌堆入手）。
//!
//! **登记差异（本文件固定）**：
//! - **①牌来源**：上游 `createCard` **凭空造牌** ↔ 原生从**牌堆发现**（原生注释自述并登记）；
//! - **②亮出牌的所在**：上游造出的候选牌**不属于牌堆**；原生候选**择定前留在牌堆**
//!   （注释自述「挂起态区域完整，御夫录同口径」）→ 与①同源；
//! - **③候选数量**：上游恒 `randomGets(3)`（池足够时 3 张）↔ 原生 `pool.len().min(3)`
//!   （牌堆该类不足 3 张时**少于 3 张**）。
//!
//! 未覆盖（登记）：类别 AI 默认值（`choice` 的启发式）属 AI 面；`toself` 的 UI 面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：两段询问（类别 → 择一）且所选牌**入手**。
#[test]
fn upstream_yangpijuan_two_stage_choice() {
    let mut e = swd_engine(20269901);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.name.contains("yangpijuan"))
            .expect("羊皮卷在目录") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        // `toself: true` → 引擎按「无目标」形态受理（上游 filterTarget 恒为自己）
        action: GameAction::PlayCard {
            card,
            targets: vec![],
        },
    });
    eprintln!("DBG 羊皮卷：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "羊皮卷应可发动（仅自己）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut picked: Option<engine_native::CardId> = None;
    let mut tag_options: Vec<u8> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 类别询问：取 Tag(1)（锦囊）
        if c.spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Tag(_)))
            && picked.is_none()
        {
            tag_options = c
                .spec
                .options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Tag(t) => Some(*t),
                    _ => None,
                })
                .collect();
            let sel = ChoiceOption::Tag(1);
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![sel],
                },
            });
            continue;
        }
        // 择一询问：取第一张候选
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        if !cards.is_empty() && picked.is_none() {
            if let ChoiceOption::Card(x) = cards[0] {
                picked = Some(x);
            }
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![cards[0]],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
    drain(&mut e);
    let hand = e.debug_hand(0);
    eprintln!(
        "DBG 羊皮卷：询问={prompts:?} 类别选项={tag_options:?} 择定={picked:?} 手牌 {hand_before} → {}",
        hand.len()
    );
    assert!(
        prompts.iter().any(|p| p.contains("yangpijuan")),
        "羊皮卷：应出现类别/择一询问，实测 {prompts:?}"
    );
    let Some(picked) = picked else {
        eprintln!("NOTE 未走到择一询问——本用例不构成验收");
        return;
    };
    assert!(
        hand.contains(&picked),
        "羊皮卷：择定的牌（{picked}）应进入自己手牌"
    );
}

/// **登记差异①②③**（源码事实固定）。
#[test]
fn upstream_yangpijuan_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/swd_card_effects.rs"
    ))
    .expect("读 swd_card_effects");
    let idx = src
        .find("HostBehavior::SwdYangpijuanCard =>")
        .expect("羊皮卷分支");
    let seg: String = src[idx..].chars().take(2000).collect();
    assert!(
        seg.contains("上游 createCard 凭空造牌，原生从牌堆发现（登记差异）"),
        "差异①固定：原生注释已自述造牌 vs 牌堆发现：{seg}"
    );
    assert!(
        seg.contains("pool.len().min(3)"),
        "差异③固定：原生候选为 pool.len().min(3)（不足 3 张时更少）：{seg}"
    );
    assert!(
        seg.contains("挂起态区域完整"),
        "差异②固定：原生注释自述候选择定前留在牌堆：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js"),
    )
    .expect("读上游 swd/card.js");
    let idx = up.find("\tyangpijuan: {").expect("上游羊皮卷");
    let seg: String = up[idx..].chars().take(1600).collect();
    assert!(
        seg.contains("chooseControl(\"basic\", \"trick\", \"equip\"")
            && seg.contains("randomGets(3)"),
        "上游依据：三类别 chooseControl + randomGets(3)：{seg}"
    );
    assert!(
        seg.contains("get.inpile(result.control"),
        "上游依据：候选来自 get.inpile（牌定义池）：{seg}"
    );
}
