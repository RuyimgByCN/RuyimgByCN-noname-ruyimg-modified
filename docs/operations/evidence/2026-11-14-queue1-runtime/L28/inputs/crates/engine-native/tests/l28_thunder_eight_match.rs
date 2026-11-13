//! L28 雷霆万钧（`archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js`）
//! 14 将 / 33 技 / `th_taipingyaoshu` / 7 配置的 6 维真实对局验收（2026-11-12）。
//!
//! 与 `l28_thunder_eight.rs`（2026-11-06 包装载八维验收，`l_rounds` 组）互补：本文件
//! 经 `Engine::submit` 真实命令（`ActivateSkill` / `PlayCard` / `NextPhase` / `Answer`）驱动
//! `HostBehavior::L28ThunderBatchA/B`，逐技覆盖六维：
//! ① 正常执行；② 可取消询问取消后不消耗次数、不产生效果；③ 非法目标/牌/数量被拒绝；
//! ④ 条件不满足与 `once_per_turn` / 限定技 `th_yeyan` 次数用尽；⑤ 多技能与装备交互
//! （太平要术防属性伤害/手牌上限/失去效果、忠壮攻击范围、隐世免延时锦囊）；
//! ⑥ 询问挂起时 `encode_archive`/`decode_archive` 存档恢复，续答后状态哈希一致。

use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    decision34_closure_extension_pack, parse_thunder_config, ThunderError,
    L28_THUNDER_EXTRA_SKILLS, L28_THUNDER_GENERALS,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::EquipSlot;
use engine_native::content::extension::ExtensionPack;
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice};

static DEDUP: AtomicU64 = AtomicU64::new(2_800_000);

pub(crate) fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

pub(crate) fn pack() -> ExtensionPack {
    decision34_closure_extension_pack()
}

/// 新开 4 人局：座位 0 装配 `general`，置于第 1 轮出牌阶段。
pub(crate) fn fresh(p: &ExtensionPack, seed: u64, general: &str) -> Engine {
    let mut e = Engine::new_with_extensions(seed, 4, &[p]).expect("L28 engine");
    e.assign_general_named(0, general)
        .expect("assign L28 general");
    e.debug_set_turn(0, Phase::Play, 1);
    e
}

pub(crate) fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(crate) fn set_flag(e: &mut Engine, p: u8, key: &str, v: u32) {
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((p, key.to_string()), v);
}

pub(crate) fn once_used(e: &Engine, p: u8, skill: &str) -> bool {
    let def = e.catalog().skill_by_name(skill).expect("skill");
    e.state_core()
        .skills
        .get(&(p, def))
        .is_some_and(|i| i.once_used)
}

pub(crate) fn has_skill(e: &Engine, p: u8, skill: &str) -> bool {
    let def = e.catalog().skill_by_name(skill).expect("skill");
    e.state_core().skills.contains_key(&(p, def))
}

pub(crate) fn hand(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

pub(crate) fn hp(e: &Engine, p: u8) -> i32 {
    e.state_core().player(p).hp
}

pub(crate) fn card_named(e: &Engine, name: &str) -> u32 {
    e.catalog()
        .cards
        .iter()
        .position(|c| c.name == name)
        .unwrap_or_else(|| panic!("card {name}")) as u32
}

pub(crate) fn card_by(e: &Engine, pred: impl Fn(&engine_native::catalog::CardDef) -> bool) -> u32 {
    e.catalog()
        .cards
        .iter()
        .position(pred)
        .expect("matching card") as u32
}

pub(crate) fn take_out(e: &mut Engine, card: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|&c| c != card);
    core.discard.retain(|&c| c != card);
    for p in 0..core.players.len() as u8 {
        core.player_mut(p).hand.retain(|&c| c != card);
        core.player_mut(p).equips.retain(|&c| c != card);
        core.player_mut(p).judge.retain(|&c| c != card);
    }
}

pub(crate) fn give(e: &mut Engine, p: u8, card: u32) {
    take_out(e, card);
    e.state_core_mut().player_mut(p).hand.push(card);
}

pub(crate) fn activate(e: &mut Engine, p: u8, skill: &str) -> CommandOutcome {
    let def = e.catalog().skill_by_name(skill).expect("skill def");
    submit(e, p, GameAction::ActivateSkill { skill: def })
}

pub(crate) fn open(e: &Engine, prompt: &str) -> OpenChoice {
    let c = e
        .open_choice()
        .unwrap_or_else(|| panic!("expected open choice {prompt}"))
        .clone();
    assert_eq!(c.spec.prompt, prompt, "open prompt");
    c
}

pub(crate) fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> CommandOutcome {
    let c = open(e, prompt);
    let out = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "{prompt} answer rejected: {out:?}"
    );
    out
}

pub(crate) fn rejected(e: &mut Engine, sel: Vec<ChoiceOption>) -> bool {
    let c = e.open_choice().expect("open choice").clone();
    let before = e.state_hash();
    let out = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    );
    let ok = matches!(out, CommandOutcome::Rejected(_));
    assert_eq!(
        e.state_hash(),
        before,
        "rejected answer must not mutate state"
    );
    assert!(
        e.open_choice().is_some_and(|o| o.id == c.id),
        "choice stays open after reject"
    );
    ok
}

pub(crate) fn tag(t: u8) -> ChoiceOption {
    ChoiceOption::Tag(t)
}
pub(crate) fn pl(p: u8) -> ChoiceOption {
    ChoiceOption::Player(p)
}
pub(crate) fn cd(c: u32) -> ChoiceOption {
    ChoiceOption::Card(c)
}

/// 答完剩余询问（有 Pass 选 Pass，可取消则取消，否则取首项）。
pub(crate) fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
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
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

/// ⑥ 挂起询问时存档 → 解码恢复：状态哈希一致；恢复局与原局作同一应答后哈希仍一致。
pub(crate) fn snapshot_roundtrip(e: &Engine, p: &ExtensionPack, sel: Vec<ChoiceOption>) {
    let c = e
        .open_choice()
        .expect("WaitingForChoice for archive")
        .clone();
    let planned = e.plan_action(c.observer, &BoundedPlanner::new(4));
    assert!(
        matches!(planned.action, GameAction::Answer { choice_id, .. } if choice_id == c.id),
        "AI must answer open {}",
        c.spec.prompt
    );
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode mid-choice archive");
    let mut restored = Engine::restore_with_extensions(&decoded, &[p]).expect("restore");
    assert_eq!(
        restored.state_hash(),
        e.state_hash(),
        "restore hash {}",
        c.spec.prompt
    );
    assert_eq!(
        restored.open_choice().map(|o| o.spec.prompt),
        Some(c.spec.prompt)
    );
    let mut twin = Engine::restore_with_extensions(&e.save(), &[p]).expect("twin");
    for eng in [&mut restored, &mut twin] {
        let out = submit(
            eng,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel.clone(),
            },
        );
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "resume answer: {out:?}"
        );
    }
    assert_eq!(
        restored.state_hash(),
        twin.state_hash(),
        "post-resume hash {}",
        c.spec.prompt
    );
}

/// 1. 14 将（含管宁 3/7）、33 技全部为原生宿主技能、太平要术卡牌与 7 项配置。
#[test]
fn l28_thunder_eight_roster_card_and_configs() {
    let p = pack();
    let mut e = Engine::new_with_extensions(2811, 4, &[&p]).expect("engine");
    assert_eq!(L28_THUNDER_GENERALS.len(), 14);
    let mut top: Vec<&str> = Vec::new();
    for &(name, fac, gender, max_hp, skills) in &L28_THUNDER_GENERALS {
        let gid = (0..e.catalog().general_count() as u16)
            .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.0 == name))
            .unwrap_or_else(|| panic!("general {name}"));
        let (_, f, sk, mhp, g) = e.catalog().general_meta(gid).unwrap();
        assert_eq!((f, g, mhp), (fac, gender, max_hp), "{name} meta");
        for s in skills {
            assert!(sk.contains(s), "{name} lacks {s}");
            if !top.contains(s) {
                top.push(s);
            }
        }
    }
    for s in L28_THUNDER_EXTRA_SKILLS
        .iter()
        .chain(["th_yeyan", "th_taipingyaoshu"].iter())
    {
        let def = e
            .catalog()
            .skill_by_name(s)
            .unwrap_or_else(|| panic!("skill {s}"));
        let b = &e.catalog().skills[def as usize].behavior;
        assert!(
            !b.is_vm_skill() && b.as_host_behavior().is_some(),
            "{s} must be native host"
        );
    }
    assert_eq!(
        top.len(),
        27,
        "14 generals expose 27 distinct general-level skills"
    );

    // 管宁 `3/7`（`extension.js:3657`）：普通座位 3/7，主公 +1 上限且起始 4。
    e.assign_general_named(1, "th_guanning").unwrap();
    assert_eq!((hp(&e, 1), e.state_core().player(1).max_hp), (3, 7));

    // 太平要术：防具、红桃 3、装备技能 `th_taipingyaoshu`。
    let tp = card_named(&e, "th_taipingyaoshu");
    let def = &e.catalog().cards[tp as usize];
    assert_eq!(
        e.catalog().card_kind(tp).equip_slot(),
        Some(EquipSlot::Armor)
    );
    assert!(def.equip_skills.contains(&"th_taipingyaoshu"));

    // 7 配置项：正式包 `config.json` 解析，缺省取上游 `init`，非法项拒绝。
    let text = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/thunder-upstream-liuli/assets/config.json"),
    )
    .unwrap();
    let cfg = parse_thunder_config(&text).expect("config.json");
    assert_eq!((cfg.game_speed, cfg.game_speed_label()), (4, "常速"));
    assert!(cfg.replace_cha && cfg.replace_plan().pairs.len() == 12);
    assert!(cfg.hide_pack && cfg.hides_exclusive_pack());
    assert_eq!(cfg.ui_introduc, "intro");
    assert_eq!(
        (
            cfg.yuqi_theme.as_str(),
            cfg.effective_dialog_theme("shousha")
        ),
        ("follow", "shousha")
    );
    assert_eq!(
        (cfg.time_theme.as_str(), cfg.effective_time_theme("decade")),
        ("follow", "decade")
    );
    assert_eq!(
        cfg.stand_image_path("th_guanning"),
        "extension/雷霆万钧/image/stand/th_guanning.png"
    );
    let with = |pairs: &[(&str, serde_json::Value)]| {
        let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
        for (k, x) in pairs {
            v[*k] = x.clone();
        }
        parse_thunder_config(&v.to_string())
    };
    use serde_json::json;
    let custom = with(&[
        ("gameSpeed", json!(0)),
        ("replaceCha", json!(false)),
        ("hidePack", json!(false)),
        ("UIItroduc", json!("intro")),
        ("yuqiTheme", json!("decade")),
        ("timeTheme", json!("shousha")),
        ("standPath", json!("extension/千幻聆音/lihui/")),
    ])
    .expect("custom config");
    assert_eq!(custom.game_speed_label(), "极速");
    assert!(custom.replace_plan().pairs.is_empty() && !custom.hides_exclusive_pack());
    assert_eq!(custom.effective_dialog_theme("shousha"), "decade");
    assert_eq!(custom.effective_time_theme("decade"), "shousha");
    assert_eq!(
        custom.stand_image_path("x"),
        "extension/千幻聆音/lihui/x.png"
    );
    for (k, bad) in [
        ("gameSpeed", json!(9)),
        ("UIItroduc", json!("guide")),
        ("yuqiTheme", json!("neon")),
        ("timeTheme", json!("default")),
        ("standPath", json!("/etc/")),
    ] {
        assert_eq!(
            with(&[(k, bad)]),
            Err(ThunderError::BadField(k.into())),
            "{k}"
        );
    }
}

/// 座位 3 结束阶段推进到座位 0 新回合（真实 `NextPhase`，重置 `once_per_turn`），
/// 途中可选询问一律取消，最后落在座位 0 出牌阶段。
pub(crate) fn next_turn(e: &mut Engine, round: u32) {
    drain(e);
    e.debug_set_turn(3, Phase::End, round);
    let _ = submit(e, 3, GameAction::NextPhase);
    for _ in 0..12 {
        drain(e);
        let t = &e.state_core().turn;
        if t.current == 0 && t.phase == Phase::Play {
            break;
        }
        let cur = t.current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    assert_eq!(
        e.state_core().turn.phase,
        Phase::Play,
        "reached seat 0 play phase"
    );
}

/// 2. Batch A 主动技：彩翼/冲虚/妙剑/神工/评才/整经/补续/成业/御风/天书/遁世。
#[test]
fn l28_thunder_eight_batch_a_active_skills() {
    let p = pack();
    // ---- th_caiyi 彩翼（`extension.js:4420`）：目标选阳/阴未移除项（X=剩余项数），移去所选项并转阴阳 ----
    let mut e = fresh(&p, 2821, "th_caohua");
    e.debug_set_hp(1, 1);
    let (h0, h1) = (hand(&e, 0), hand(&e, 1));
    activate(&mut e, 0, "th_caiyi");
    assert!(once_used(&e, 0, "th_caiyi"));
    assert!(
        rejected(&mut e, vec![pl(9)]),
        "③ seat 9 is not a legal target"
    );
    answer(&mut e, "th_caiyi:choose_target", Vec::new());
    assert!(
        !once_used(&e, 0, "th_caiyi"),
        "② cancel refunds once_per_turn"
    );
    assert_eq!((hand(&e, 0), hand(&e, 1), hp(&e, 1)), (h0, h1, 1));
    assert_eq!(flag(&e, 0, "th_caiyi_state"), 0);
    activate(&mut e, 0, "th_caiyi");
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    let c = open(&e, "th_caiyi:choose_option");
    assert_eq!(c.observer, 1, "① target seat 1 chooses the caiyi option");
    assert!(rejected(&mut e, Vec::new()), "③ option prompt is mandatory");
    assert!(rejected(&mut e, vec![tag(4)]), "③ option 4 out of 0..4");
    snapshot_roundtrip(&e, &p, vec![tag(1)]);
    answer(&mut e, "th_caiyi:choose_option", vec![tag(1)]);
    assert_eq!(
        hand(&e, 1),
        h1 + 4,
        "① yang opt 1 with X=4 remaining options draws 4 cards"
    );
    assert_eq!(
        (flag(&e, 0, "th_caiyi_yang_removed"), flag(&e, 0, "th_caiyi_state")),
        (0b0010, 1),
        "① opt 1 removed and state flipped to Yin"
    );
    assert!(
        matches!(activate(&mut e, 0, "th_caiyi"), CommandOutcome::Rejected(_)),
        "④ once_per_turn"
    );
    next_turn(&mut e, 1);
    let h1 = hand(&e, 1);
    activate(&mut e, 0, "th_caiyi");
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    answer(&mut e, "th_caiyi:choose_option", vec![tag(1)]);
    assert_eq!(
        hand(&e, 1),
        h1 - 4,
        "① yin opt 1 with X=4 remaining options discards 4 cards"
    );
    assert_eq!(
        (flag(&e, 0, "th_caiyi_yin_removed"), flag(&e, 0, "th_caiyi_state")),
        (0b0010, 0),
        "① yin opt 1 removed and state flipped back to Yang"
    );
    next_turn(&mut e, 2);
    activate(&mut e, 0, "th_caiyi");
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    assert!(
        rejected(&mut e, vec![tag(1)]),
        "③ already-removed yang opt 1 cannot be chosen again"
    );
    answer(&mut e, "th_caiyi:choose_option", vec![tag(0)]);
    assert_eq!(hp(&e, 1), 4, "① yang opt 0 with X=3 recovers 3 HP (1 -> 4)");

    // ---- th_chongxu 冲虚（`extension.js:4354`）/ miaojian 妙剑 ----
    let mut e = fresh(&p, 2822, "th_sunhanhua");
    let h0 = hand(&e, 0);
    activate(&mut e, 0, "th_chongxu");
    answer(&mut e, "th_chongxu:choose_action", Vec::new());
    assert!(
        !once_used(&e, 0, "th_chongxu") && flag(&e, 0, "th_chongxu_marks") == 0,
        "②"
    );
    activate(&mut e, 0, "th_chongxu");
    assert!(rejected(&mut e, vec![tag(3)]), "③ action 3 undefined");
    snapshot_roundtrip(&e, &p, vec![tag(0)]);
    answer(&mut e, "th_chongxu:choose_action", vec![tag(0)]);
    assert_eq!(
        flag(&e, 0, "th_miaojian_level"),
        1,
        "① 3 marks - 2 → miaojian lv1"
    );
    assert_eq!((flag(&e, 0, "th_chongxu_marks"), hand(&e, 0)), (1, h0 + 1));
    assert!(
        matches!(
            activate(&mut e, 0, "th_chongxu"),
            CommandOutcome::Rejected(_)
        ),
        "④"
    );
    let (h0, hp1) = (hand(&e, 0), hp(&e, 1));
    activate(&mut e, 0, "miaojian");
    assert!(
        rejected(&mut e, vec![pl(0)]),
        "③ miaojian cannot target self"
    );
    snapshot_roundtrip(&e, &p, vec![pl(1)]);
    answer(&mut e, "miaojian:choose_target", vec![pl(1)]);
    assert_eq!(
        (hp(&e, 1), hand(&e, 0)),
        (hp1 - 1, h0 + 2),
        "① lv1: no discard, dmg 1, draw 2"
    );

    // ---- th_shengong 神工（`extension.js:5163`）：弃置一张装备牌，选锻造类别，为目标装备 ----
    let mut e = fresh(&p, 2823, "th_puyuan");
    let old_hand = std::mem::take(&mut e.state_core_mut().player_mut(0).hand);
    e.state_core_mut().discard.extend(old_hand);
    let basic = card_by(&e, |c| c.kind.is_basic_kind());
    give(&mut e, 0, basic);
    assert!(
        matches!(
            activate(&mut e, 0, "th_shengong"),
            CommandOutcome::Rejected(_)
        ),
        "④ shengong requires at least 1 equipment card in he"
    );
    let eq_own = card_by(&e, |c| c.kind.equip_slot() == Some(EquipSlot::Armor));
    give(&mut e, 0, eq_own);
    let foreign = e.state_core().player(1).hand[0];
    activate(&mut e, 0, "th_shengong");
    assert!(
        rejected(&mut e, vec![cd(basic)]),
        "③ cannot discard non-equipment card for shengong"
    );
    assert!(
        rejected(&mut e, vec![cd(foreign)]),
        "③ cannot discard another seat's card"
    );
    answer(&mut e, "th_shengong:discard_card", Vec::new());
    assert!(!once_used(&e, 0, "th_shengong"), "② cancel refunds shengong");
    activate(&mut e, 0, "th_shengong");
    answer(&mut e, "th_shengong:discard_card", vec![cd(eq_own)]);
    assert!(
        rejected(&mut e, Vec::new()) && rejected(&mut e, vec![tag(3)]),
        "③ forge category 0..=2 mandatory"
    );
    snapshot_roundtrip(&e, &p, vec![tag(0)]);
    answer(&mut e, "th_shengong:choose_forge", vec![tag(0)]);
    assert!(rejected(&mut e, Vec::new()), "③ target prompt mandatory");
    snapshot_roundtrip(&e, &p, vec![pl(2)]);
    answer(&mut e, "th_shengong:choose_target", vec![pl(2)]);
    assert!(e.state_core().discard.contains(&eq_own));
    assert_eq!(
        (
            flag(&e, 0, "th_shengong_category"),
            flag(&e, 0, "th_shengong_forged"),
            e.state_core().player(2).equips.len(),
        ),
        (1, 1, 1),
        "① forged weapon onto seat 2"
    );
    assert_eq!(
        e.catalog()
            .card_kind(e.state_core().player(2).equips[0])
            .equip_slot(),
        Some(EquipSlot::Weapon),
        "① forged category 0 equips a Weapon"
    );

    // ---- th_pingcai 评才（`extension.js:5181`）：卧龙 = 1 点火焰伤害 ----
    let mut e = fresh(&p, 2824, "th_pangdegong");
    activate(&mut e, 0, "th_pingcai");
    answer(&mut e, "th_pingcai:choose_relic", Vec::new());
    assert!(!once_used(&e, 0, "th_pingcai") && flag(&e, 0, "th_pingcai_relic_wiped") == 0);
    activate(&mut e, 0, "th_pingcai");
    answer(&mut e, "th_pingcai:choose_relic", vec![tag(0)]);
    snapshot_roundtrip(&e, &p, vec![pl(1)]);
    let hp1 = hp(&e, 1);
    answer(&mut e, "th_pingcai:choose_target", vec![pl(1)]);
    assert_eq!(hp(&e, 1), hp1 - 1, "① wolong fire damage");

    // ---- th_zhengjing 整经（`extension.js:5953`）+ th_zhengjing2：经置于武将牌上，准备阶段获得 ----
    let mut e = fresh(&p, 2825, "th_zhengxuan");
    activate(&mut e, 0, "th_zhengjing");
    assert!(rejected(&mut e, vec![tag(6)]), "③ count 0..=5");
    answer(&mut e, "th_zhengjing:choose_count", vec![tag(3)]);
    assert_eq!(
        e.state_core().table.len(),
        3,
        "excavated classics wait in the processing area"
    );
    snapshot_roundtrip(&e, &p, vec![pl(0)]);
    let h0 = hand(&e, 0);
    answer(&mut e, "th_zhengjing:choose_target", vec![pl(0)]);
    let jing = e
        .state_core()
        .vm_aux
        .piles
        .get(&(0, "th_jing".to_string()))
        .cloned()
        .unwrap();
    assert_eq!(
        (jing.len(), hand(&e, 0)),
        (1, h0 + 2),
        "① one classic on target, rest gained"
    );
    assert!(has_skill(&e, 0, "th_zhengjing2") && e.state_core().table.is_empty());
    next_turn(&mut e, 1);
    assert!(
        e.state_core().player(0).hand.contains(&jing[0]),
        "zhengjing2 claims at prepare"
    );
    assert_eq!(hand(&e, 0), h0 + 3, "judge and draw phases skipped");
    assert!(
        !has_skill(&e, 0, "th_zhengjing2"),
        "zhengjing2 removes itself"
    );

    // ---- th_buxu 补续（同回合递增弃牌成本 1, 2, 3...）/ th_chengye 承业（`extension.js:6182-6514`）----
    let mut e = fresh(&p, 2826, "th_mamidi");
    assert!(
        matches!(
            activate(&mut e, 0, "th_chengye"),
            CommandOutcome::Rejected(_)
        ),
        "④ chengye needs six classics"
    );
    let own0 = e.state_core().player(0).hand[0];
    activate(&mut e, 0, "th_buxu");
    answer(&mut e, "th_buxu:discard_cards", Vec::new());
    assert!(
        e.state_core().player(0).hand.contains(&own0),
        "② cancel leaves card in hand"
    );
    activate(&mut e, 0, "th_buxu");
    answer(&mut e, "th_buxu:discard_cards", vec![cd(own0)]);
    assert!(rejected(&mut e, vec![tag(6)]), "③ chamber 0..=5");
    snapshot_roundtrip(&e, &p, vec![tag(2)]);
    answer(&mut e, "th_buxu:choose_chamber", vec![tag(2)]);
    assert_eq!(flag(&e, 0, "th_chengye_mask"), 0b000100);
    assert_eq!(
        e.state_core().vm_aux.piles[&(0, "th_liujing".to_string())].len(),
        1,
        "① 1st buxu adds 1 classic for 1 card"
    );
    // 同回合第 2 次发动补续：需弃置 2 张牌，且不能选已补过的槽位 2
    let (c1, c2) = (
        e.state_core().player(0).hand[0],
        e.state_core().player(0).hand[1],
    );
    activate(&mut e, 0, "th_buxu");
    assert!(
        rejected(&mut e, vec![cd(c1)]),
        "③ 2nd buxu in same turn requires 2 cards (escalating cost)"
    );
    answer(&mut e, "th_buxu:discard_cards", vec![cd(c1), cd(c2)]);
    assert!(
        rejected(&mut e, vec![tag(2)]),
        "③ chamber 2 already filled in th_chengye_mask"
    );
    answer(&mut e, "th_buxu:choose_chamber", vec![tag(0)]);
    assert_eq!(flag(&e, 0, "th_chengye_mask"), 0b000101);
    // 此时手牌只剩 1 张，同回合第 3 次补续需要 3 张牌 → 发动被拒绝（④）
    assert!(
        matches!(activate(&mut e, 0, "th_buxu"), CommandOutcome::Rejected(_)),
        "④ 3rd buxu needs 3 cards, rejected when only 1 card remains"
    );
    set_flag(&mut e, 0, "th_chengye_mask", 0x3F);
    set_flag(&mut e, 0, "th_chengye_liujing_count", 6);
    let h0 = hand(&e, 0);
    activate(&mut e, 0, "th_chengye");
    let hp2 = hp(&e, 2);
    answer(&mut e, "th_chengye:choose_target", vec![pl(2)]);
    assert_eq!(
        (hand(&e, 0), hp(&e, 2)),
        (h0 + 6, hp2 - 1),
        "① claim all 6 classics + 1 damage"
    );
    assert_eq!(flag(&e, 0, "th_chengye_completed"), 1);

    // ---- th_yufeng 御风 / th_tianshu 天书（`extension.js:6515-6973`）----
    let mut e = fresh(&p, 2827, "th_nanhualaoxian");
    let h0 = hand(&e, 0);
    activate(&mut e, 0, "th_yufeng");
    assert!(rejected(&mut e, vec![tag(0)]), "③ flight score 1..=3");
    answer(&mut e, "th_yufeng:choose_score", vec![tag(2)]);
    assert!(
        rejected(&mut e, vec![pl(0)]) && rejected(&mut e, vec![pl(1), pl(2), pl(3)]),
        "③ yufeng targets 1..=score other players"
    );
    answer(&mut e, "th_yufeng:choose_target", vec![pl(1), pl(3)]);
    assert_eq!(
        (
            hand(&e, 0),
            flag(&e, 1, "th_yufeng_locked"),
            flag(&e, 3, "th_yufeng_locked"),
        ),
        (h0 + 2, 1, 1),
        "① yufeng locks chosen targets until their next Prepare judge"
    );
    let spade = card_by(&e, |c| c.suit == engine_native::catalog::Suit::Spade);
    let red = card_by(&e, |c| {
        c.suit != engine_native::catalog::Suit::Spade && c.name == "shan"
    });
    give(&mut e, 0, spade);
    give(&mut e, 0, red);
    activate(&mut e, 0, "th_tianshu");
    assert!(
        rejected(&mut e, vec![cd(spade)]),
        "③ tianshu requires a non-Spade card"
    );
    answer(&mut e, "th_tianshu:discard_card", vec![cd(red)]);
    snapshot_roundtrip(&e, &p, vec![pl(2)]);
    answer(&mut e, "th_tianshu:choose_target", vec![pl(2)]);
    let tp = card_named(&e, "th_taipingyaoshu");
    assert!(
        e.state_core().player(2).equips.contains(&tp),
        "① seat 2 equips 太平要术"
    );
    assert!(has_skill(&e, 2, "th_taipingyaoshu"));
    assert!(
        matches!(
            activate(&mut e, 0, "th_tianshu"),
            CommandOutcome::Rejected(_)
        ),
        "④ once_per_turn and taipingyaoshu in play"
    );
    // ⑤ 被御风锁定的座位 1 进入准备阶段时触发判定，跳过相应阶段并解除锁定
    let _ = hand_turn_to(&mut e, 1, 1);
    drain(&mut e);
    assert_eq!(
        flag(&e, 1, "th_yufeng_locked"),
        0,
        "⑤ yufeng lock consumed at seat 1 Prepare judge"
    );
    assert!(
        flag(&e, 1, "th_yufeng_skipped_play_discard") == 1
            || flag(&e, 1, "th_yufeng_skipped_draw") == 1,
        "⑤ yufeng Prepare judge skips Play+Discard (black) or Draw (red)"
    );
    // ④ 即使到下一轮，只要【太平要术】仍在场上装备区，天书不可发动
    next_turn(&mut e, 2);
    give(&mut e, 0, red);
    assert!(
        matches!(
            activate(&mut e, 0, "th_tianshu"),
            CommandOutcome::Rejected(_)
        ),
        "④ tianshu rejected while th_taipingyaoshu is already in play"
    );

    // ---- th_dunshi 遁世（`extension.js:3783`）：视为使用未移除基本牌 1/回合 ----
    let mut e = fresh(&p, 2828, "th_guanning");
    assert_eq!(hp(&e, 0), 3);
    activate(&mut e, 0, "th_dunshi");
    answer(&mut e, "th_dunshi:choose_basic", Vec::new());
    assert!(
        !once_used(&e, 0, "th_dunshi") && flag(&e, 0, "th_dunshi_used_this_turn") == 0,
        "②"
    );
    activate(&mut e, 0, "th_dunshi");
    snapshot_roundtrip(&e, &p, vec![tag(2)]);
    answer(&mut e, "th_dunshi:choose_basic", vec![tag(2)]);
    assert_eq!(hp(&e, 0), 4, "① virtual tao recovers 1");
    assert!(
        matches!(
            activate(&mut e, 0, "th_dunshi"),
            CommandOutcome::Rejected(_)
        ),
        "④"
    );
}

/// 回合交给 `seat`（由上家结束阶段 `NextPhase` 推进），停在第一个询问或 `seat` 的准备阶段之后。
pub(crate) fn hand_turn_to(e: &mut Engine, seat: u8, round: u32) -> CommandOutcome {
    let prev = (seat + 3) % 4;
    e.debug_set_turn(prev, Phase::End, round);
    submit(e, prev, GameAction::NextPhase)
}

/// 当前回合角色推进阶段直到 `phase`（途中询问一律取消/取首项）。
pub(crate) fn advance_to(e: &mut Engine, phase: Phase) {
    for _ in 0..12 {
        drain(e);
        if e.state_core().turn.phase == phase {
            return;
        }
        let cur = e.state_core().turn.current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    panic!("never reached {phase:?}");
}

pub(crate) fn play(e: &mut Engine, p: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let out = submit(e, p, GameAction::PlayCard { card, targets });
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "play rejected: {out:?}"
    );
    out
}
