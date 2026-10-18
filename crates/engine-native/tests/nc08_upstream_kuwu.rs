//! NC08 第⑦层（上游行为对照）样本 5：`kuwu`（苦无，L19 欢乐卡牌）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js`）：
//!
//! ```js
//! kuwu: {
//!     audio: "ext:欢乐卡牌/audio/skill:true",
//!     trigger: {
//!         source: "damageSource",
//!     },
//!     forced: true,
//!     equipSkill: true,
//!     filter(event, player) {
//!         if (event._notrigger.includes(event.player)) {
//!             return false;
//!         }
//!         return event.card && event.card.name === "sha" && event.notLink() && event.player.countCards("he") > 0;
//!     },
//!     async content(event, trigger, player) {
//!         await trigger.player.chooseToDiscard(true, "he");
//!     },
//! },
//! ```
//!
//! 卡牌定义（`.../欢乐卡牌/card/card.js`，武器栏 `equip1`，`distance.attackFrom:-1`）：
//!
//! ```js
//! kuwu: { type: "equip", subtype: "equip1", skills: ["kuwu"], nomod: true,
//!         nopower: true, unique: true, distance: { attackFrom: -1 } },
//! ```
//!
//! 由该 JS 推导的规则点（本文件逐条验收）：
//! 1. **装备技**（`equipSkill: true` + 卡牌 `skills:["kuwu"]`）：装上苦无才持有，
//!    卸下即失去；
//! 2. **锁定触发**：来源造成伤害后（`trigger.source = "damageSource"`、
//!    `forced: true`）——无「是否发动」确认门；非【杀】伤害不触发；
//! 3. filter 要求受伤角色 `countCards("he") > 0`（无手牌/装备则不触发）；
//! 4. filter 要求 `event.notLink()`——**铁索连环传导伤害不触发**；
//! 5. filter 要求受伤角色不在 `event._notrigger`（上游 `untrigger(false, player)`
//!    标记的「本次伤害对该角色不触发」）；
//! 6. 效果：受伤角色**强制**弃置一张 `he` 牌（`chooseToDiscard(true, "he")`
//!    ——由受伤角色自选，手牌或装备区）。
//!
//! 逐条核对用的上游辅助函数（本文件引用的 `notLink`/`untrigger` 语义）：
//! `noname/library/element/gameEvent.ts:1058`（`notLink()` = 父事件非
//! `_lianhuan`/`_lianhuan2`）、`:646`（`untrigger(false, player)` 把该角色加入
//! `_notrigger`）、`noname/library/index.js:11137`（`_notrigger` 命中则跳过触发）。
//!
//! 原生实现引用：`content/host/filtering/common.rs:531-534`（damage 过滤）、
//! `content/host/metadata/common.rs:6-12`（锁定/时机）、
//! `content/host/step/skills_status.rs:666-681`（弃牌效果）、
//! `content/host/tags.rs:303`/`keys.rs:850`（tag 0 ↔ `host:sha_damage_force_discard`）、
//! `kernel/damage.rs:988-1015`（铁索传导伤害带 `from_chain: true`）。
//!
//! 已登记差异（详见证据 README，逐条以本文件断言固定）：
//! - 差异 K1：原生不判 `notLink()`——铁索传导伤害**仍触发**（上游不触发）；
//! - 差异 K2：原生无「受伤角色有牌」filter——仅持装备的目标不再被强制弃牌
//!   （上游会强制弃装备）；
//! - 差异 K3：原生无 `_notrigger` 概念（结构性差异，以源码事实固定）；
//! - 差异 K4：弃牌从「受伤角色自选 `he`」简化为「确定性弃首张手牌」。
//!
//! 对齐项（对照结论，勿误记为差异）：上游 `event.card.name === "sha"` 对火杀/
//! 雷杀**同样为真**——杀族牌的 `name` 恒为 `"sha"`、属性走 `nature` 字段
//! （`archive/upstream/noname/apps/core/card/standard.js:99-102` 的
//! `sha: { nature: ["thunder","fire","kami","ice"] }`），故原生
//! `CardKind::is_sha_like()`（Sha/FireSha/LeiSha，`catalog.rs:610-612`）与上游
//! 同口径。`g1_huanle/huanle.rs:1329-1334` 注释「上游过滤精确杀名」为**误读**，
//! 本样本登记勘误（不改他人测试，仅在证据记录）。
//!
//! 不在规则对照范围（表现/策略面）：`audio`、`ai.equipValue`。

use engine_native::ids::CardId;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn huanle_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("huanle-upstream-noname/assets/cards.json"))
        .expect("读欢乐卡牌包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

/// 应答所有开放选择（记录出现过的 prompt，供「无确认门」断言）。
fn drain_collect(e: &mut Engine) -> Vec<String> {
    let mut prompts = Vec::new();
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        prompts.push(c.spec.prompt.to_string());
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let out = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "应答被拒: {out:?}"
        );
    }
    prompts
}

fn drain(e: &mut Engine) {
    let _ = drain_collect(e);
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let out = submit(e, actor, GameAction::NextPhase);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "推进被拒: {out:?}"
        );
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 装苦无：按目录实体名取牌（同 Kind 多牌种时 `debug_place` 会误取标准武器）。
fn equip_kuwu(e: &mut Engine) -> CardId {
    let kuwu = e.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
    let out = submit(e, 0, GameAction::PlayCard {
        card: kuwu,
        targets: Vec::new(),
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备苦无: {out:?}"
    );
    drain(e);
    kuwu
}

fn discard_len(e: &Engine) -> usize {
    e.debug_discard_len()
}

/// 上游规则点 1：苦无为**装备技**——装上授予 `huanle_kuwu`，卸下撤销。
#[test]
fn upstream_kuwu_is_equip_skill() {
    let mut e = huanle_engine(20262701);
    advance_to_play(&mut e, 0);
    assert!(!e.debug_has_skill(0, "huanle_kuwu"), "未装备前不持有苦无");
    let kuwu = equip_kuwu(&mut e);
    assert!(
        e.debug_has_skill(0, "huanle_kuwu"),
        "装备苦无授予技能实例（上游 equipSkill + card.skills）"
    );
    e.debug_remove_equip(0, kuwu);
    assert!(
        !e.debug_has_skill(0, "huanle_kuwu"),
        "卸下苦无撤销技能实例（装备技随装备授予/撤销）"
    );
}

/// 上游规则点 2：锁定触发、仅【杀】伤害——非【杀】伤害（无牌 / 非杀牌）
/// 不触发；触发时**无**「是否发动」确认门（`forced: true`）。
#[test]
fn upstream_kuwu_requires_sha_damage_and_is_forced() {
    let mut e = huanle_engine(20262702);
    advance_to_play(&mut e, 0);
    equip_kuwu(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let a = e.debug_place(1, "tao");
    let b = e.debug_place(1, "jiu");
    // ① 无牌伤害（event.card 为空）：上游 `event.card &&` 为假 → 不触发
    let _ = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    drain(&mut e);
    assert!(
        e.debug_hand(1).contains(&a) && e.debug_hand(1).contains(&b),
        "无牌伤害不触发苦无（上游 filter 要求 event.card）"
    );
    // ② 有牌但**非杀**：上游 `event.card.name === 'sha'` 为假 → 不触发
    let non_sha = {
        let cat = e.catalog();
        let core = e.state_core();
        core.deck
            .iter()
            .copied()
            .find(|c| !cat.card_kind(*c).is_sha_like())
            .expect("牌堆有非杀牌")
    };
    let _ = e.debug_deal_damage_with_card(
        Some(0),
        1,
        1,
        engine_native::event::DamageNature::Normal,
        non_sha,
    );
    drain(&mut e);
    assert!(
        e.debug_hand(1).contains(&a) && e.debug_hand(1).contains(&b),
        "非杀牌伤害不触发苦无"
    );
}

/// 上游规则点 6（+ 差异 K4）：【杀】造成伤害后受伤角色**强制**弃一张牌；
/// 原生按「确定性弃首张手牌」落地——本断言固定该现状（改回 `he` 自选即失败）。
#[test]
fn upstream_kuwu_forces_target_discard() {
    let mut e = huanle_engine(20262703);
    advance_to_play(&mut e, 0);
    equip_kuwu(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let first = e.debug_place(1, "tao");
    let second = e.debug_place(1, "jiu");
    let sha = e.debug_place(0, "sha");
    let before = discard_len(&e);
    let prompts = {
        let out = submit(&mut e, 0, GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        });
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "出杀: {out:?}"
        );
        drain_collect(&mut e)
    };
    assert!(
        !prompts.iter().any(|p| p == "trigger_confirm"),
        "锁定技不得挂「是否发动」确认门：{prompts:?}"
    );
    assert_eq!(
        e.debug_hp(1),
        3,
        "杀命中（1 点伤害）后才轮到苦无弃牌"
    );
    assert!(
        !e.debug_hand(1).contains(&first),
        "受伤角色被强制弃牌（上游 chooseToDiscard(true,'he')）"
    );
    assert!(
        e.debug_hand(1).contains(&second),
        "差异 K4：原生固定弃**首张**手牌（上游由受伤角色自选）"
    );
    assert_eq!(
        discard_len(&e),
        before + 2,
        "弃牌堆 = 杀 + 苦无弃牌（各 1 张）"
    );
}

/// 差异 K1：上游 `event.notLink()` 排除铁索连环传导伤害；原生只按
/// 「来源 + 杀族牌」过滤，传导伤害（`from_chain: true`）**仍触发**苦无。
#[test]
fn upstream_kuwu_chain_damage_difference() {
    let mut e = huanle_engine(20262704);
    advance_to_play(&mut e, 0);
    equip_kuwu(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1、2 号横置；2 号持一张手牌（只应由「传导」决定是否被弃）
    let victim_card = e.debug_place(2, "tao");
    {
        let core = e.state_core_mut();
        core.players[1].linked = true;
        core.players[2].linked = true;
    }
    let fire_sha = e.debug_place(0, "fire_sha");
    let out = submit(&mut e, 0, GameAction::PlayCard {
        card: fire_sha,
        targets: vec![1],
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "出火杀: {out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 3, "火杀命中 1 号");
    assert_eq!(e.debug_hp(2), 3, "铁索传导火伤至 2 号");
    assert!(
        !e.debug_hand(2).contains(&victim_card),
        "差异 K1：原生对铁索传导伤害仍触发苦无（上游 notLink() 为假 → 不触发）"
    );
    assert!(!e.debug_linked(1) && !e.debug_linked(2), "传导后连环解除");
}

/// 差异 K2+K3：上游 filter 还要求「目标有 `he` 牌」与「目标不在 `_notrigger`」；
/// 原生无这两项条件（源码事实固定），且仅持装备的目标**不会**被强制弃牌
/// （上游会强制其弃掉装备）。
#[test]
fn upstream_kuwu_filter_gaps() {
    // 源码事实：伤害过滤臂只有「来源 + 未防止 + 有杀族牌」三项
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/filtering/common.rs"
    ))
    .expect("读宿主过滤源码");
    let idx = src
        .find("HostBehavior::ShaDamageForceDiscard => matches!")
        .expect("苦无过滤臂");
    let arm: String = src[idx..].chars().take(320).collect();
    assert!(
        arm.contains("is_sha_like"),
        "杀族判定在过滤臂内：{arm}"
    );
    assert!(
        !arm.contains("from_chain") && !arm.contains("_notrigger"),
        "差异 K1/K3：过滤臂缺 notLink/_notrigger 对应条件（若新增需更新证据）：{arm}"
    );
    // 行为事实：目标无手牌、只有装备 → 原生不弃任何牌（上游强制弃装备）
    let mut e = huanle_engine(20262705);
    advance_to_play(&mut e, 0);
    equip_kuwu(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let equip = {
        let cat = e.catalog();
        let core = e.state_core();
        core.deck
            .iter()
            .copied()
            .find(|c| cat.card_kind(*c).equip_slot().is_some())
            .expect("牌堆有装备")
    };
    {
        let core = e.state_core_mut();
        core.deck.retain(|c| *c != equip);
        core.players[1].equips.push(equip);
    }
    let sha = e.debug_place(0, "sha");
    let before = discard_len(&e);
    let out = submit(&mut e, 0, GameAction::PlayCard {
        card: sha,
        targets: vec![1],
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "出杀: {out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 3, "杀命中");
    assert!(
        e.debug_equips(1).contains(&equip),
        "差异 K2：原生只弃手牌——仅持装备的目标不被强制弃牌（上游会弃装备）"
    );
    assert_eq!(discard_len(&e), before + 1, "弃牌堆只有杀本身");
}
