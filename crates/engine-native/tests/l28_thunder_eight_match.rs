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
    core.detached_cards.retain(|&(c, _)| c != card);
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
    // ---- th_caiyi 彩翼（`extension.js:4420`）：仅结束阶段开始触发，目标选阳/阴未移除项（X=剩余项数），移去所选项并转阴阳 ----
    let mut e = fresh(&p, 2821, "th_caohua");
    e.debug_set_hp(1, 1);
    let (h0, h1) = (hand(&e, 0), hand(&e, 1));
    assert!(
        matches!(activate(&mut e, 0, "th_caiyi"), CommandOutcome::Rejected(_)),
        "④ th_caiyi has no enable:'phaseUse'"
    );
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(
        rejected(&mut e, vec![pl(9)]),
        "③ seat 9 is not a legal target"
    );
    answer(&mut e, "th_caiyi:choose_target", Vec::new());
    drain(&mut e);
    assert_eq!((hand(&e, 0), hand(&e, 1), hp(&e, 1)), (h0, h1, 1));
    assert_eq!(flag(&e, 0, "th_caiyi_state"), 0);
    e.debug_set_turn(0, Phase::Discard, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
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
    e.debug_set_turn(0, Phase::Play, 2);
    assert!(
        matches!(activate(&mut e, 0, "th_caiyi"), CommandOutcome::Rejected(_)),
        "④ th_caiyi cannot be activated manually"
    );
    let h1 = hand(&e, 1);
    e.debug_set_turn(0, Phase::Discard, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    answer(&mut e, "th_caiyi:choose_option", vec![tag(1)]);
    let disc4: Vec<_> = open(&e, "th_caiyi:discard_cards")
        .spec
        .options
        .iter()
        .take(4)
        .copied()
        .collect();
    assert_eq!(disc4.len(), 4);
    answer(&mut e, "th_caiyi:discard_cards", disc4);
    drain(&mut e);
    assert_eq!(
        hand(&e, 1),
        h1 - 4,
        "① yin opt 1 with X=4 remaining options discards 4 chosen cards"
    );
    assert_eq!(
        (flag(&e, 0, "th_caiyi_yin_removed"), flag(&e, 0, "th_caiyi_state")),
        (0b0010, 0),
        "① yin opt 1 removed and state flipped back to Yang"
    );
    e.debug_set_turn(0, Phase::Discard, 4);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
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
    let eq_own = card_by(&e, |c| c.kind.equip_slot() == Some(EquipSlot::Weapon) && !c.off_deck);
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
    while e
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "th_shengong:assist")
    {
        answer(&mut e, "th_shengong:assist", Vec::new());
    }
    let forged = open(&e, "th_shengong:choose_card").spec.options[0];
    assert!(rejected(&mut e, Vec::new()), "③ choose_card mandatory");
    snapshot_roundtrip(&e, &p, vec![forged]);
    answer(&mut e, "th_shengong:choose_card", vec![forged]);
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
    let shown = e.state_core().table.clone();
    assert_eq!(
        shown.len(),
        3,
        "excavated classics wait in the processing area"
    );
    snapshot_roundtrip(&e, &p, vec![pl(0)]);
    let h0 = hand(&e, 0);
    answer(&mut e, "th_zhengjing:choose_target", vec![pl(0)]);
    snapshot_roundtrip(&e, &p, vec![cd(shown[0])]);
    answer(&mut e, "th_zhengjing:choose_cards", vec![cd(shown[0])]);
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
        "① chosen classic on target, rest gained"
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
        "④ chengye is passive/phaseUseBegin (no active phaseUse)"
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
    let mut e = fresh(&p, 2829, "th_mamidi");
    for (idx, c) in [
        card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Duel),
        card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Sha),
        card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Nullify),
        card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Wuzhong),
        card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Le),
        card_by(&e, |c| c.kind.equip_slot().is_some()),
    ]
    .into_iter()
    .enumerate()
    {
        take_out(&mut e, c);
        e.state_core_mut()
            .vm_aux
            .piles
            .entry((0, "th_liujing".to_string()))
            .or_default()
            .push(c);
        let next_mask = flag(&e, 0, "th_chengye_mask") | (1 << idx);
        set_flag(&mut e, 0, "th_chengye_mask", next_mask);
    }
    set_flag(&mut e, 0, "th_chengye_liujing_count", 6);
    e.debug_set_turn(0, Phase::Draw, 2);
    let h0 = hand(&e, 0);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(hand(&e, 0), h0 + 6, "① phaseUseBegin th_chengye_gain gains all 6 classics");
    assert_eq!(flag(&e, 0, "th_chengye_mask"), 0);
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
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    snapshot_roundtrip(&e, &p, vec![pl(2)]);
    answer(&mut e, "th_tianshu:choose_target", vec![pl(2)]);
    snapshot_roundtrip(&e, &p, vec![cd(spade)]);
    answer(&mut e, "th_tianshu:discard_card", vec![cd(spade)]);
    drain(&mut e);
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
        "④ th_tianshu has no enable:'phaseUse'"
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
    // ④ 即使到下一轮，只要【太平要术】仍在场上装备区，天书进入出牌阶段也不触发
    next_turn(&mut e, 2);
    give(&mut e, 0, red);
    e.debug_set_turn(0, Phase::Draw, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(
        !matches!(
            activate(&mut e, 0, "th_tianshu"),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ) && e
            .open_choice()
            .as_ref()
            .is_none_or(|c| c.spec.prompt != "th_tianshu:choose_target"),
        "④ tianshu must not trigger at phaseUseBegin while th_taipingyaoshu is already in play"
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

/// Batch 1.6 L28 真实对局回归：补叙 / 整经 / 天书 / 太平要术。
#[test]
fn l28_batch16_buxu_zhengjing_tianshu_taipingyaoshu_real_match() {
    let p = pack();

    // ---- 1. th_buxu：按所选六经类别先后搜牌堆/弃牌堆，均无时退还次数 ----
    let mut e = fresh(&p, 2861, "th_mamidi");
    // 牌堆清空“无懈可击”，仅弃牌堆放 1 张 wuxie（槽 2 `礼`）
    let wuxies: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == engine_native::catalog::CardKind::Nullify)
        .map(|(i, _)| i as u32)
        .collect();
    for &c in &wuxies[1..] {
        give(&mut e, 3, c);
    }
    take_out(&mut e, wuxies[0]);
    e.state_core_mut().discard.push(wuxies[0]);
    let cost0 = e.state_core().player(0).hand[0];
    activate(&mut e, 0, "th_buxu");
    assert!(
        matches!(
            e.plan_action(0, &BoundedPlanner::new(4)).action,
            GameAction::Answer { .. }
        ),
        "AI must plan th_buxu:discard_cards"
    );
    answer(&mut e, "th_buxu:discard_cards", vec![cd(cost0)]);
    answer(&mut e, "th_buxu:choose_chamber", vec![tag(2)]);
    let liujing = e.state_core().vm_aux.piles[&(0, "th_liujing".to_string())].clone();
    assert!(
        liujing.len() == 1
            && e.catalog().card_kind(liujing[0]) == engine_native::catalog::CardKind::Nullify,
        "th_buxu(2) must find Nullify from discard pile when deck has none"
    );

    // 清空“无中生有”（槽 3 `易`）：牌堆与弃牌堆均无时不得添经、不得置 mask[3]、且退还使用次数
    let wuzhongs: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == engine_native::catalog::CardKind::Wuzhong)
        .map(|(i, _)| i as u32)
        .collect();
    for &c in &wuzhongs {
        give(&mut e, 3, c);
    }
    let mask_before = flag(&e, 0, "th_chengye_mask");
    let used_before = flag(&e, 0, "th_buxu_used_turn");
    let (c1, c2) = (
        e.state_core().player(0).hand[0],
        e.state_core().player(0).hand[1],
    );
    activate(&mut e, 0, "th_buxu");
    answer(&mut e, "th_buxu:discard_cards", vec![cd(c1), cd(c2)]);
    answer(&mut e, "th_buxu:choose_chamber", vec![tag(3)]);
    assert_eq!(
        (
            e.state_core().vm_aux.piles[&(0, "th_liujing".to_string())].len(),
            flag(&e, 0, "th_chengye_mask"),
            flag(&e, 0, "th_buxu_used_turn"),
        ),
        (1, mask_before, used_before),
        "missing matching card in both deck & discard must refund buxu stat and add no classic"
    );
    // 退还后再次发动补叙，费用仍为 2（而非 3），并选槽 5（装备）验证先牌堆后弃牌堆
    let extra_eq = card_by(&e, |c| c.kind.equip_slot().is_some());
    let deck_eq = card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Qilin);
    let extra_cost = card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Sha && c.suit == engine_native::catalog::Suit::Heart);
    give(&mut e, 0, extra_cost);
    take_out(&mut e, extra_eq);
    take_out(&mut e, deck_eq);
    e.state_core_mut().discard.push(extra_eq);
    e.state_core_mut().deck.push(deck_eq);
    let need_two: Vec<u32> = e.state_core().player(0).hand.iter().take(2).copied().collect();
    activate(&mut e, 0, "th_buxu");
    assert_eq!(open(&e, "th_buxu:discard_cards").spec.min, 2);
    answer(
        &mut e,
        "th_buxu:discard_cards",
        need_two.into_iter().map(cd).collect(),
    );
    snapshot_roundtrip(&e, &p, vec![tag(5)]);
    answer(&mut e, "th_buxu:choose_chamber", vec![tag(5)]);
    assert_eq!(
        e.state_core().vm_aux.piles[&(0, "th_liujing".to_string())].last().copied(),
        Some(deck_eq),
        "th_buxu(5) must search deck before discard pile and pick equip"
    );

    // ---- 2. th_zhengjing：0 张不添经不摸牌；>1 张自选任意非空子集作经 ----
    let mut e = fresh(&p, 2862, "th_zhengxuan");
    let h0 = hand(&e, 0);
    let deck0 = e.state_core().deck.len();
    activate(&mut e, 0, "th_zhengjing");
    answer(&mut e, "th_zhengjing:choose_count", vec![tag(0)]);
    assert_eq!(
        (
            hand(&e, 0),
            e.state_core().deck.len(),
            e.state_core()
                .vm_aux
                .piles
                .get(&(0, "th_jing".to_string()))
                .map_or(0, |v| v.len()),
            has_skill(&e, 0, "th_zhengjing2"),
        ),
        (h0, deck0, 0, false),
        "zhengjing count=0 must neither add jing nor draw nor grant th_zhengjing2"
    );
    // 亮出 3 张牌：验证非重名、AI 选敌作 target 并仅选 1 张给敌留 2 张给自己，且支持自选子集与快照往返
    next_turn(&mut e, 1);
    advance_to(&mut e, Phase::Play);
    let h0 = hand(&e, 0);
    activate(&mut e, 0, "th_zhengjing");
    assert_eq!(
        e.plan_action(0, &BoundedPlanner::new(4)).action,
        GameAction::Answer {
            choice_id: open(&e, "th_zhengjing:choose_count").id,
            selection: vec![tag(5)],
        },
        "AI should cut max classics"
    );
    answer(&mut e, "th_zhengjing:choose_count", vec![tag(3)]);
    let shown = e.state_core().table.clone();
    assert_eq!(shown.len(), 3);
    let names: std::collections::BTreeSet<&str> =
        shown.iter().map(|&c| e.catalog().cards[c as usize].name).collect();
    assert_eq!(names.len(), 3, "th_zhengjing shown cards must have distinct names");
    let ai_tgt = e.plan_action(0, &BoundedPlanner::new(4));
    assert!(
        matches!(ai_tgt.action, GameAction::Answer { ref selection, .. } if selection != &[pl(0)]),
        "AI should target an enemy rather than self for th_zhengjing2 draw/judge skip: {ai_tgt:?}"
    );
    answer(&mut e, "th_zhengjing:choose_target", vec![pl(0)]);
    let alloc = open(&e, "th_zhengjing:choose_cards");
    assert_eq!((alloc.spec.min, alloc.spec.max), (1, 3));
    assert_eq!(
        e.plan_action(0, &BoundedPlanner::new(4)).action,
        GameAction::Answer {
            choice_id: alloc.id,
            selection: vec![cd(shown[0])],
        },
        "AI puts exactly 1 card into th_jing and keeps the rest"
    );
    snapshot_roundtrip(&e, &p, vec![cd(shown[1]), cd(shown[2])]);
    answer(&mut e, "th_zhengjing:choose_cards", vec![cd(shown[1]), cd(shown[2])]);
    assert_eq!(
        e.state_core().vm_aux.piles[&(0, "th_jing".to_string())],
        vec![shown[1], shown[2]],
        "owner can put chosen non-empty subset (2 of 3) onto target as jing"
    );
    assert!(e.state_core().player(0).hand.contains(&shown[0]));
    assert_eq!(hand(&e, 0), h0 + 1);
    // 单张牌直接置作经，不开 allocate 选牌窗
    next_turn(&mut e, 2);
    advance_to(&mut e, Phase::Play);
    activate(&mut e, 0, "th_zhengjing");
    answer(&mut e, "th_zhengjing:choose_count", vec![tag(1)]);
    answer(&mut e, "th_zhengjing:choose_target", vec![pl(1)]);
    assert!(
        e.open_choice().is_none(),
        "single-card th_zhengjing must not open th_zhengjing:choose_cards"
    );
    assert_eq!(e.state_core().vm_aux.piles[&(1, "th_jing".to_string())].len(), 1);

    // ---- 3. th_tianshu：phaseUseBegin 触发 + 先选目标后扣任意 he 成本 + 实体牌路径 + 禁用防具栏降级入手 ----
    let mut e = fresh(&p, 2863, "th_nanhualaoxian");
    let tp = card_named(&e, "th_taipingyaoshu");
    take_out(&mut e, tp);
    // 太平要术不在牌堆/弃牌堆（游离在桌面）：消耗成本后不得强行把虚牌或桌面牌装进装备区/授技
    e.state_core_mut().table.push(tp);
    let spade = card_by(&e, |c| c.suit == engine_native::catalog::Suit::Spade && c.kind == engine_native::catalog::CardKind::Sha);
    give(&mut e, 0, spade);
    let enter_seat0_play = |e: &mut Engine, round: u32| {
        let _ = hand_turn_to(e, 0, round);
        drain(e);
        while (e.state_core().turn.current, e.state_core().turn.phase) != (0, Phase::Draw) {
            let cur = e.state_core().turn.current;
            let _ = submit(e, cur, GameAction::NextPhase);
            drain(e);
        }
        let _ = submit(e, 0, GameAction::NextPhase);
    };
    enter_seat0_play(&mut e, 2);
    // phaseUseBegin 应先询问 th_tianshu:choose_target（支持 AI、可取消、快照往返）
    let ask_t = open(&e, "th_tianshu:choose_target");
    assert_eq!(ask_t.observer, 0);
    let ai_ts = e.plan_action(0, &BoundedPlanner::new(4));
    assert!(matches!(ai_ts.action, GameAction::Answer { .. }), "AI handles th_tianshu:choose_target");
    snapshot_roundtrip(&e, &p, vec![pl(2)]);
    answer(&mut e, "th_tianshu:choose_target", vec![pl(2)]);
    // 第二个询问可取消且接受黑桃牌，取消不弃牌
    let h0_before = hand(&e, 0);
    snapshot_roundtrip(&e, &p, vec![cd(spade)]);
    answer(&mut e, "th_tianshu:discard_card", Vec::new());
    assert_eq!(hand(&e, 0), h0_before, "cancel discard refunds without losing cost");
    // 再走一次：选目标取消也无损；接受黑桃后因实体牌缺失不授技
    enter_seat0_play(&mut e, 3);
    let h0_round3 = hand(&e, 0);
    answer(&mut e, "th_tianshu:choose_target", Vec::new());
    assert_eq!(hand(&e, 0), h0_round3, "cancel choose_target at phaseUseBegin loses no card");
    enter_seat0_play(&mut e, 4);
    answer(&mut e, "th_tianshu:choose_target", vec![pl(2)]);
    answer(&mut e, "th_tianshu:discard_card", vec![cd(spade)]);
    drain(&mut e);
    assert!(
        !has_skill(&e, 2, "th_taipingyaoshu") && !e.state_core().player(2).equips.contains(&tp),
        "missing physical th_taipingyaoshu must not fake-grant skill"
    );

    // ---- 4. th_taipingyaoshu：禁用防具栏留在手牌 / 青釭剑穿透 / Equipped 日志序 / Snatch & Dismantle 外层结算 + unequip2 ----
    let mut e = fresh(&p, 2864, "th_nanhualaoxian");
    let tp = card_named(&e, "th_taipingyaoshu");
    take_out(&mut e, tp);
    e.state_core_mut().discard.push(tp);
    // 防具栏禁用时：天书使目标获得实体牌到手牌但不强行装备
    set_flag(&mut e, 2, "equip_slot_disabled:2", 1);
    let cost_c0 = e.state_core().player(0).hand[0];
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "th_tianshu:choose_target", vec![pl(2)]);
    answer(&mut e, "th_tianshu:discard_card", vec![cd(cost_c0)]);
    drain(&mut e);
    assert!(
        e.state_core().player(2).hand.contains(&tp) && !e.state_core().player(2).equips.contains(&tp),
        "disabled armor slot keeps th_taipingyaoshu in target hand"
    );
    set_flag(&mut e, 2, "equip_slot_disabled:2", 0);
    // 从弃牌堆重放天书正常装备至 seat 1
    take_out(&mut e, tp);
    e.state_core_mut().discard.push(tp);
    enter_seat0_play(&mut e, 2);
    let cost_c = e.state_core().player(0).hand[0];
    answer(&mut e, "th_tianshu:choose_target", vec![pl(1)]);
    answer(&mut e, "th_tianshu:discard_card", vec![cd(cost_c)]);
    drain(&mut e);
    assert!(e.state_core().player(1).equips.contains(&tp) && has_skill(&e, 1, "th_taipingyaoshu"));
    // 无 unequip2 时，青釭剑 + 火杀（source unequip）穿透太平要术防属性伤保护
    let qinggang = card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Qinggang);
    give(&mut e, 0, qinggang);
    play(&mut e, 0, qinggang, Vec::new());
    drain(&mut e);
    let fire_sha = card_by(&e, |c| c.kind == engine_native::catalog::CardKind::FireSha);
    give(&mut e, 0, fire_sha);
    let hp1_qg = hp(&e, 1);
    play(&mut e, 0, fire_sha, vec![1]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1_qg - 1, "Qinggang FireSha bypasses th_taipingyaoshu elemental guard");
    e.state_core_mut().player_mut(0).equips.retain(|&c| c != qinggang);
    e.state_core_mut().discard.push(qinggang);
    e.state_core_mut().turn.sha_used_play_phase = false;
    // unequip2（l25_hubu_unequip2）在场：火伤害不被防止
    set_flag(&mut e, 1, "l25_hubu_unequip2", 1);
    give(&mut e, 0, fire_sha);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, fire_sha, vec![1]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1 - 1, "unequip2 suppresses th_taipingyaoshu elemental guard");
    // unequip2 在场时拆走太平要术：撤销技能但不摸2/不失1血
    let dismantle = card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Dismantle);
    give(&mut e, 0, dismantle);
    let dropped1 = std::mem::take(&mut e.state_core_mut().player_mut(1).hand);
    e.state_core_mut().discard.extend(dropped1);
    e.debug_set_hp(1, 3);
    let h1 = hand(&e, 1);
    play(&mut e, 0, dismantle, vec![1]);
    drain(&mut e);
    assert_eq!(
        (hand(&e, 1), hp(&e, 1), has_skill(&e, 1, "th_taipingyaoshu")),
        (h1, 3, false),
        "unequip2 suppresses th_taipingyaoshu onLose draw2/loseHp1"
    );
    // 顺手牵羊夺走太平要术：外层顺手完全结算后，再触发摸2后失1血（seat 3 距离 1）
    e.state_core_mut().discard.retain(|&c| c != tp);
    e.state_core_mut().player_mut(3).equips.push(tp);
    let tp_def = e.catalog().skill_by_name("th_taipingyaoshu").unwrap();
    e.state_core_mut().skills.insert(
        (3, tp_def),
        engine_native::state::SkillInstance {
            owner: 3,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    let dropped3 = std::mem::take(&mut e.state_core_mut().player_mut(3).hand);
    e.state_core_mut().discard.extend(dropped3);
    e.debug_set_hp(3, 3);
    let snatch = card_by(&e, |c| c.kind == engine_native::catalog::CardKind::Snatch);
    give(&mut e, 0, snatch);
    e.state_core_mut().turn.sha_used_play_phase = false;
    play(&mut e, 0, snatch, vec![3]);
    drain(&mut e);
    assert_eq!(
        (hand(&e, 3), hp(&e, 3), e.state_core().player(0).hand.contains(&tp)),
        (2, 2, true),
        "snatching th_taipingyaoshu triggers onLose after snatch: draw 2 then lose 1 HP"
    );
    // 装备其他防具顶替太平要术：日志严格满足 Equipped -> CardsDrawn -> HpLost
    let other_armor = card_by(&e, |c| c.kind.equip_slot() == Some(EquipSlot::Armor) && c.name != "th_taipingyaoshu");
    give(&mut e, 0, other_armor);
    e.debug_set_hp(0, 3);
    play(&mut e, 0, tp, Vec::new());
    drain(&mut e);
    let log_start = e.log().len();
    play(&mut e, 0, other_armor, Vec::new());
    drain(&mut e);
    let tail = &e.log()[log_start..];
    let p_eq = tail.iter().position(|ev| matches!(ev, engine_native::LogEvent::Equipped { player: 0, card, .. } if *card == other_armor)).expect("Equipped log");
    let p_dr = tail.iter().position(|ev| matches!(ev, engine_native::LogEvent::CardsDrawn { player: 0, cards } if cards.len() == 2)).expect("CardsDrawn 2 log");
    let p_hp = tail.iter().position(|ev| matches!(ev, engine_native::LogEvent::HpLost { player: 0, amount: 1 })).expect("HpLost 1 log");
    assert!(p_eq < p_dr && p_dr < p_hp, "armor replace onLose order: Equipped({p_eq}) -> Draw2({p_dr}) -> LoseHp1({p_hp})");
}

/// 承业六经规则对齐、L28 AI 策略与太平要术 unequip2 手牌上限真实对局回归。
#[test]
fn l28_chengye_liujing_ai_and_tpys_unequip2_hand_limit_real_match() {
    use engine_native::catalog::{CardKind, Suit};

    let clear_hand = |e: &mut Engine, s: u8| {
        let dropped = std::mem::take(&mut e.state_core_mut().player_mut(s).hand);
        e.state_core_mut().discard.extend(dropped);
    };

    let cards_matching = |e: &Engine, f: fn(&engine_native::catalog::CardDef) -> bool| -> Vec<u32> {
        e.catalog()
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| f(c))
            .map(|(i, _)| i as u32)
            .collect()
    };

    let p = pack();

    // ---- 1. th_chengye（`雷霆万钧/extension.js:6186-6328`）----
    // 1a) 他人弃牌阶段弃基本牌不触发承业；他人使用牌结算进弃牌堆（useCardAfter）才按六经收录，且不含非伤害普通锦囊（拆）
    let mut e = fresh(&p, 2871, "th_mamidi");
    e.debug_set_turn(1, Phase::Play, 1);
    clear_hand(&mut e, 1);
    let shas = cards_matching(&e, |c| c.kind == CardKind::Sha);
    let sha_a = shas[0];
    let sha_b = shas[1];
    let dismantle = card_by(&e, |c| c.kind == CardKind::Dismantle);
    let duel = card_by(&e, |c| c.kind == CardKind::Duel);
    let wuzhong = card_by(&e, |c| c.kind == CardKind::Wuzhong);
    let wuxie = card_by(&e, |c| c.kind == CardKind::Nullify);
    let lebu = card_by(&e, |c| c.kind == CardKind::Le);
    let horse = card_by(&e, |c| c.kind == CardKind::HorsePlus);
    // 座位 1 弃牌阶段弃置两张 Sha：upstream loseAfter 只收 delay/equip，基本牌不得触发承业
    for &c in &shas[2..7] {
        give(&mut e, 1, c);
    }
    give(&mut e, 1, sha_a);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let disc_ask = open(&e, "discard_to_limit");
    let drop_cards: Vec<_> = e
        .state_core()
        .player(1)
        .hand
        .iter()
        .take(disc_ask.spec.min)
        .copied()
        .map(cd)
        .collect();
    answer(&mut e, "discard_to_limit", drop_cards);
    drain(&mut e);
    assert_eq!(
        flag(&e, 0, "th_chengye_mask"),
        0,
        "discarding basic Sha in phaseDiscard must not trigger th_chengye (extension.js:6212-6219)"
    );

    // 1b) 他人使用非伤害非六经锦囊（Dismantle）不入六经；使用 Wuzhong（无中 3）、拆装备触发 loseAfter（5）
    e.debug_set_turn(1, Phase::Play, 2);
    give(&mut e, 1, horse);
    play(&mut e, 1, horse, Vec::new());
    drain(&mut e);
    give(&mut e, 1, dismantle);
    // 让 seat 2 仅持装备供 dismantle 消耗进入弃牌堆（清手牌避免无懈或随机抽中手牌）
    clear_hand(&mut e, 2);
    let target_eq = card_by(&e, |c| c.kind == CardKind::HorseMinus);
    take_out(&mut e, target_eq);
    e.state_core_mut().player_mut(2).equips.push(target_eq);
    play(&mut e, 1, dismantle, vec![2]);
    drain(&mut e);
    // 场上 seat 2 弃装备入弃牌堆：loseAfter 收入 5(equip)，但 dismantle 本身不在六经 0..=5
    assert_eq!(
        flag(&e, 0, "th_chengye_mask"),
        1 << 5,
        "only equip in discard triggers loseAfter chamber 5; Dismantle matches no chamber"
    );

    give(&mut e, 1, wuzhong);
    play(&mut e, 1, wuzhong, Vec::new());
    drain(&mut e);
    assert_eq!(
        flag(&e, 0, "th_chengye_mask") & (1 << 3),
        1 << 3,
        "useCardAfter Wuzhong fills chamber 3"
    );

    // 1c) th_chengye 与 th_buxu 共享六经槽：槽 3/5 已满，补续不得再选 3/5；激活 th_chengye 被拒绝（无主动伤害）
    e.debug_set_turn(0, Phase::Play, 3);
    assert!(
        matches!(activate(&mut e, 0, "th_chengye"), CommandOutcome::Rejected(_)),
        "th_chengye has no active phaseUse ability (extension.js:6182-6370)"
    );
    let c0 = e.state_core().player(0).hand[0];
    activate(&mut e, 0, "th_buxu");
    answer(&mut e, "th_buxu:discard_cards", vec![cd(c0)]);
    assert!(
        rejected(&mut e, vec![tag(3)]) && rejected(&mut e, vec![tag(5)]),
        "shared th_chengye_mask forbids th_buxu from picking chambers already filled by th_chengye"
    );
    answer(&mut e, "th_buxu:choose_chamber", vec![tag(1)]); // 补基本牌

    // 1d) 集齐六经（补入 0 伤害锦囊、2 无懈、4 乐不思蜀），于自己出牌阶段开始自动收入手牌且清零
    let mut e = fresh(&p, 2872, "th_mamidi");
    for (idx, c) in [duel, sha_b, wuxie, wuzhong, lebu, horse].into_iter().enumerate() {
        take_out(&mut e, c);
        e.state_core_mut()
            .vm_aux
            .piles
            .entry((0, "th_liujing".to_string()))
            .or_default()
            .push(c);
        let mask = flag(&e, 0, "th_chengye_mask") | (1 << idx);
        set_flag(&mut e, 0, "th_chengye_mask", mask);
    }
    set_flag(&mut e, 0, "th_chengye_liujing_count", 6);
    e.debug_set_turn(0, Phase::Draw, 2);
    let hp1_before = hp(&e, 1);
    let h0_before = hand(&e, 0);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(
        (hand(&e, 0), flag(&e, 0, "th_chengye_mask"), hp(&e, 1)),
        (h0_before + 6, 0, hp1_before),
        "th_chengye_gain at phaseUseBegin moves all 6 classics to hand and deals no damage"
    );

    // ---- 2. L28 AI（`extension.js:6113, 6142-6147, 8646-8663`）----
    // 2a) th_zhengjing:choose_target 避开持 xinfu_pdgyingshi 的敌人（AI return 0），选无该技的敌人
    let mut e = fresh(&p, 2873, "th_zhengxuan");
    e.assign_general_named(1, "th_pangdegong").expect("pangdegong has xinfu_pdgyingshi");
    activate(&mut e, 0, "th_zhengjing");
    answer(&mut e, "th_zhengjing:choose_count", vec![tag(2)]);
    let ai_target = e.plan_action(0, &BoundedPlanner::new(4));
    assert!(
        matches!(ai_target.action, GameAction::Answer { ref selection, .. } if selection != &[pl(1)] && selection != &[pl(0)]),
        "th_zhengjing AI should score xinfu_pdgyingshi=0 and pick another enemy: {ai_target:?}"
    );
    answer(&mut e, "th_zhengjing:choose_target", vec![pl(2)]);
    // 2b) th_zhengjing:choose_cards 排序取最低价值 1 张置经（若首张为桃、次张为杀，须选杀置经而自留桃）
    let mut e = fresh(&p, 2874, "th_zhengxuan");
    clear_hand(&mut e, 0);
    let tao = card_by(&e, |c| c.kind == CardKind::Tao);
    let sha_c = card_by(&e, |c| c.kind == CardKind::Sha);
    take_out(&mut e, tao);
    take_out(&mut e, sha_c);
    e.state_core_mut().deck.push(sha_c);
    e.state_core_mut().deck.push(tao);
    activate(&mut e, 0, "th_zhengjing");
    answer(&mut e, "th_zhengjing:choose_count", vec![tag(2)]);
    assert_eq!(e.state_core().table, vec![tao, sha_c]);
    answer(&mut e, "th_zhengjing:choose_target", vec![pl(1)]);
    let ai_cards = e.plan_action(0, &BoundedPlanner::new(4));
    assert_eq!(
        ai_cards.action,
        GameAction::Answer {
            choice_id: open(&e, "th_zhengjing:choose_cards").id,
            selection: vec![cd(sha_c)],
        },
        "th_zhengjing processAI must pick lowest-value card (Sha) for target expansion and keep Tao"
    );

    // 2c) th_yeyan:discard_cards AI 面对两张黑桃排前的 5 张手牌时，必须选出 4 张不同花色而非重复黑桃
    let mut e = fresh(&p, 2875, "th_shen_zhouyu");
    clear_hand(&mut e, 0);
    let spades = cards_matching(&e, |c| c.suit == Suit::Spade);
    let s1 = spades[0];
    let s2 = spades[1];
    let h1 = card_by(&e, |c| c.suit == Suit::Heart);
    let c1 = card_by(&e, |c| c.suit == Suit::Club);
    let d1 = card_by(&e, |c| c.suit == Suit::Diamond);
    for c in [s1, s2, h1, c1, d1] {
        give(&mut e, 0, c);
    }
    activate(&mut e, 0, "th_yeyan");
    answer(&mut e, "th_yeyan:choose_mode", vec![tag(1)]);
    answer(&mut e, "th_yeyan:choose_targets", vec![pl(1)]);
    answer(&mut e, "th_yeyan:choose_damage", vec![tag(2)]);
    let ai_disc = e.plan_action(0, &BoundedPlanner::new(4));
    let GameAction::Answer { selection, .. } = &ai_disc.action else {
        panic!("expected Answer for th_yeyan:discard_cards: {ai_disc:?}");
    };
    assert_eq!(selection.len(), 4, "AI must provide 4 distinct-suit cards when available: {ai_disc:?}");
    let mut suits = Vec::new();
    for o in selection {
        if let ChoiceOption::Card(cid) = o {
            let s = e.catalog().cards[*cid as usize].suit;
            if !suits.contains(&s) {
                suits.push(s);
            }
        }
    }
    assert_eq!(suits.len(), 4, "th_yeyan AI discard must contain all 4 distinct suits");

    // ---- 3. 太平要术 unequip2 屏蔽手牌上限加成（`extension.js:8968`）----
    let mut e = fresh(&p, 2876, "th_nanhualaoxian");
    let tp = card_named(&e, "th_taipingyaoshu");
    give(&mut e, 1, tp);
    e.debug_set_turn(1, Phase::Play, 1);
    play(&mut e, 1, tp, Vec::new());
    drain(&mut e);
    let limit_with = e.state_core().hand_limit(e.catalog(), 1);
    let base_hp = hp(&e, 1);
    assert!(limit_with > base_hp as usize, "th_taipingyaoshu raises hand limit by factions - 1");
    set_flag(&mut e, 1, "l25_hubu_unequip2", 1);
    let limit_suppressed = e.state_core().hand_limit(e.catalog(), 1);
    assert_eq!(
        limit_suppressed, base_hp as usize,
        "unequip2 must suppress th_taipingyaoshu hand limit bonus"
    );

    // ---- 4. charlotte 与死亡 onremove 清回 th_jing / th_liujing 至弃牌堆（`extension.js:6165-6170, 6273-6276`）----
    let zj2_def = e.catalog().skill_by_name("th_zhengjing2").unwrap();
    assert!(
        e.catalog().is_charlotte_skill(zj2_def),
        "th_zhengjing2 must be marked charlotte (extension.js:6165)"
    );
    let mut e = fresh(&p, 2877, "th_mamidi");
    e.debug_grant(0, "th_zhengjing2");
    let c_jing = sha_b;
    let c_liu = wuzhong;
    take_out(&mut e, c_jing);
    take_out(&mut e, c_liu);
    e.state_core_mut()
        .vm_aux
        .piles
        .entry((0, "th_jing".to_string()))
        .or_default()
        .push(c_jing);
    e.state_core_mut()
        .vm_aux
        .piles
        .entry((0, "th_liujing".to_string()))
        .or_default()
        .push(c_liu);
    set_flag(&mut e, 0, "th_chengye_mask", 1 << 3);
    e.debug_set_hp(0, 1);
    let kill_sha = shas[3];
    e.debug_set_turn(1, Phase::Play, 2);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 2);
    clear_hand(&mut e, 3);
    give(&mut e, 1, kill_sha);
    play(&mut e, 1, kill_sha, vec![0]);
    drain(&mut e);
    assert!(!e.state_core().player(0).alive);
    assert!(
        e.state_core().discard.contains(&c_jing) && e.state_core().discard.contains(&c_liu),
        "holder death must return th_jing & th_liujing cards to discard pile"
    );
    assert!(
        !e.state_core().vm_aux.piles.contains_key(&(0, "th_jing".to_string()))
            && !e.state_core().vm_aux.piles.contains_key(&(0, "th_liujing".to_string())),
        "holder death must clear th_jing & th_liujing piles"
    );
    assert_eq!(flag(&e, 0, "th_chengye_mask"), 0);
}
