//! NC08 第⑦层（上游行为对照）样本 60：`yaowu`（耀武，**标准面** / 华雄）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! yaowu: {
//!     trigger: { player: "damageBegin3" },             // 自己受到伤害时（生效前）
//!     forced: true,                                    // **锁定技**
//!     filter(event, player) {
//!         return event.card?.name === "sha"            // 由【杀】造成
//!             && get.color(event.card) === "red"       // **红色**
//!             && event.source?.isIn();                 // 来源**存活**
//!     },
//!     async content(event, trigger, player) {
//!         await trigger.source.chooseDrawRecover({ forced: true });   // 来源**二选**：摸 1 或回复 1
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/门槛**：自己受到**红色【杀】**伤害时（锁定）；来源须**存活**且**非自己**；
//! 2. **效果**：**伤害来源**在「**摸一张牌**」与「**回复 1 点体力**」间**二选**（`forced` → 不可取消）。
//!
//! 原生实现引用：`content/standard/skills/yao_wu.rs`（filter：`Damage` + 目标自己 + 未防止 +
//! `amount > 0` + 来源存在/非自己/存活 + 牌为【杀】且花色**红桃/方块**；机器中
//! `if (p.hp as u32) < p.max_hp` → **满血时无回复选项**）。
//!
//! **登记（注释可能滞后，待核对）**：`yao_wu.rs` 头注释写「**登记差异**：上游 `chooseDrawRecover`
//! 由 AI/玩家确认，原生为**强制二选**」——但上游 `content` 本身即
//! `chooseDrawRecover({ forced: true })`（**同样强制**）→ 该「差异」**疑不成立**（注释滞后，
//! 与本会话样本 57 仁德的注释滞后同类）；**本样本只登记该观察**，不据此判定差异。
//!
//! 未覆盖（登记）：来源**二选的行为面**（需构造来源为玩家且可观察其手牌/体力变化的场景）；
//! `ai.neg` 策略面。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 1（**红色【杀】**）：红杀造成伤害 → 出现来源二选询问。
#[test]
fn upstream_yaowu_red_sha_triggers_choice() {
    let mut e = Engine::new(20269201, 4);
    e.debug_grant(1, "耀武");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持**红色**【杀】打 1 号（耀武持有者）
    let red_sha = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| {
                c.name == "sha"
                    && matches!(
                        c.suit,
                        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                    )
            })
            .expect("红色杀") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: red_sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
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
    eprintln!(
        "DBG 耀武（红杀）：询问={prompts:?} 1 号 hp={}",
        e.debug_hp(1)
    );
    assert!(
        prompts.iter().any(|p| p.contains("yaowu")),
        "耀武：红色【杀】造成伤害后应出现来源二选询问，实测 {prompts:?}"
    );
    let _ = Phase::Play;
}

/// 上游点 1（**反例 · 黑色【杀】**）：黑杀**不**触发耀武。
#[test]
fn upstream_yaowu_black_sha_does_not_trigger() {
    let mut e = Engine::new(20269202, 4);
    e.debug_grant(1, "耀武");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let black_sha = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| {
                c.name == "sha"
                    && matches!(
                        c.suit,
                        engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                    )
            })
            .expect("黑色杀") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: black_sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
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
    eprintln!("DBG 耀武（黑杀）：询问={prompts:?}");
    assert!(
        !prompts.iter().any(|p| p.contains("yaowu")),
        "耀武：**黑色**【杀】不应触发（上游 `get.color(card) === 'red'`）：{prompts:?}"
    );
}

/// **登记（注释可能滞后）**：原生头注释所称的「差异」在上游同样存在 `forced: true`。
#[test]
fn upstream_yaowu_note_may_be_stale() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/yao_wu.rs"
    ))
    .expect("读耀武源码");
    let head: String = src.chars().take(400).collect();
    assert!(
        head.contains("登记差异") && head.contains("强制二选"),
        "登记：原生注释称『上游由 AI/玩家确认，原生为强制二选』：{head}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tyaowu: {").expect("上游耀武");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("chooseDrawRecover({ forced: true })"),
        "上游依据：`chooseDrawRecover` 亦为 **forced** → 该『差异』疑不成立：{seg}"
    );
    assert!(
        seg.contains("damageBegin3") && seg.contains("get.color(event.card) === \"red\""),
        "上游依据：damageBegin3 + 红色判定：{seg}"
    );
}
