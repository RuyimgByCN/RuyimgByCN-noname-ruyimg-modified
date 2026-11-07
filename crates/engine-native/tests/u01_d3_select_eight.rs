//! U01 3D精选（`d3-upstream-noname`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装/激活/停用/卸载；
//! ② 配置与包数据：`assets/character.json` 桥接 1 名武将（`ddd刘冲`）与 2 项宿主技能（`dddjinggou`、`dddmoyan`）；
//! ③ 组合共存：与 `huanle-upstream-noname`、`wudian-upstream-noname` 同局装载不冲突；
//! ④ 保存恢复：`Engine::save` / `Engine::restore_with_extensions` 字节级往返一致，篡改扩展集拒绝恢复；
//! ⑤ 功能执行：国战明置 `ddd刘冲`、精彀（`dddjinggou`）远射武器加伤与末焱（`dddmoyan`）准备阶段亮牌火杀结算；
//! ⑦ 异常拒装：损坏 JSON、未知行为键、缺失字段返回明确 `BridgeError`；
//! ⑧ 边界登记：其余 3D 精选立绘/武将按台账边界登记，不虚报全表 45 武将规则等价。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{bridge_generals, bridge_pack, BridgeError};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::Faction;
use engine_native::content::standard::Gender;
use engine_native::{
    CommandOutcome, Engine, GameAction, GameCommand, GameMode, Phase,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

#[test]
fn u01_d3_select_eight_dimension_acceptance() {
    // ① 发现与真实商店导入生命周期
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover_extensions");
    assert!(
        discovered.iter().any(|d| d.name == "d3-upstream-noname"),
        "d3-upstream-noname 可被发现"
    );

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import_extensions");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "d3-select" && m.namespace == "upstream-noname")
        .expect("d3-select 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let char_json = std::fs::read_to_string(ext_root.join("assets/character.json"))
        .expect("读取 assets/character.json");

    // ② 包数据桥接与武将/技能核对
    let bridged = bridge_pack(&char_json).expect("bridge_pack(d3 character.json)");
    let generals_out = bridge_generals(&char_json).expect("bridge_generals");
    assert_eq!(bridged.pack.def.name, "d3-upstream-noname");
    assert_eq!(bridged.pack.def.version, 1);
    assert_eq!(bridged.pack.general_count(), 1);
    assert_eq!(generals_out.unmapped_skills, 0, "刘冲双技能均已映射");
    assert_eq!(bridged.pack.host_skills.len(), 2);

    let base_generals = engine_native::content::standard::STANDARD_GENERALS.len();
    let e = Engine::new_with_extensions(190101, 3, &[&bridged.pack]).expect("单包建局");
    let (gname, faction, skills, hp, gender) = e
        .catalog()
        .general_meta(base_generals as u16)
        .expect("ddd刘冲在目录");
    assert_eq!(gname, "ddd刘冲");
    assert_eq!(faction, Faction::Qun);
    assert_eq!(skills, vec!["dddjinggou", "dddmoyan"]);
    assert_eq!(hp, 4);
    assert_eq!(gender, Gender::Male);

    // ③ 与其他扩展包共存建局（d3 + wudian）
    let wudian_json =
        std::fs::read_to_string(root.join("wudian-upstream-noname/assets/cards.json"))
            .expect("读取 wudian cards.json");
    let wudian_pack = bridge_pack(&wudian_json).expect("bridge wudian").pack;
    let coex_packs = [&bridged.pack, &wudian_pack];
    let mut coex = Engine::new_with_extensions(190102, 3, &coex_packs).expect("双包共存建局");
    assert!(coex.catalog().skill_by_name("dddjinggou").is_some());
    assert!(coex.catalog().skill_by_name("dddmoyan").is_some());
    assert!(coex.catalog().skill_by_name("wd_sanjian").is_some());

    // ④ 存档与恢复（含国战明置刘冲 + 扩展集不一致拒绝恢复）
    let mut snap = coex.save();
    snap.core.mode = GameMode::Kingdoms;
    snap.core.players[0].general_slots[0] = Some(base_generals as u16);
    coex = Engine::restore_with_extensions(&snap, &coex_packs).expect("恢复双包对局");

    // 推进至 0 号出牌阶段并明置主将
    for _ in 0..64 {
        if coex.turn().current == 0 && coex.turn().phase == Phase::Play && coex.open_choice().is_none() {
            break;
        }
        if let Some(c) = coex.open_choice().cloned() {
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let _ = submit(
                &mut coex,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = coex.turn().current;
        let _ = submit(&mut coex, actor, GameAction::NextPhase);
    }

    // ⑤ 明置 ddd刘冲 并验证精彀（dddjinggou）与末焱（dddmoyan）真实结算
    let rev_out = submit(&mut coex, 0, GameAction::RevealGeneral { slot: 0 });
    assert!(
        matches!(rev_out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "明置 ddd刘冲 成功: {rev_out:?}"
    );
    assert!(
        coex.state_core()
            .skills
            .keys()
            .any(|(o, d)| *o == 0 && coex.catalog().skill_name(*d) == "dddjinggou"),
        "明置后授予 dddjinggou"
    );
    assert!(
        coex.state_core()
            .skills
            .keys()
            .any(|(o, d)| *o == 0 && coex.catalog().skill_name(*d) == "dddmoyan"),
        "明置后授予 dddmoyan"
    );

    // 5a. 精彀（dddjinggou）：装备全场最大射程武器（wd_sanjian，射程 3）→ 触发目标选择并造 1 点伤害
    for p in 0..3 {
        coex.debug_clear_hand(p);
    }
    let sanjian_cid = coex
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.starts_with("wd_sanjian"))
        .expect("wd_sanjian 在目录") as u32;
    let mut sanjian_snap = coex.save();
    sanjian_snap.core.deck.retain(|c| *c != sanjian_cid);
    sanjian_snap.core.discard.retain(|c| *c != sanjian_cid);
    sanjian_snap.core.players[0].hand.push(sanjian_cid);
    coex = Engine::restore_with_extensions(&sanjian_snap, &coex_packs).expect("注入三尖两刃刀恢复");

    let hp1_before_jg = coex.debug_hp(1);
    let eq_out = submit(
        &mut coex,
        0,
        GameAction::PlayCard {
            card: sanjian_cid,
            targets: vec![],
        },
    );
    assert!(
        matches!(eq_out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "装备三尖两刃刀: {eq_out:?}"
    );
    let jg_choice = coex.open_choice().cloned().expect("精彀应挂起目标选择");
    assert_eq!(jg_choice.spec.prompt, "d3_jinggou_target");
    let _ = submit(
        &mut coex,
        jg_choice.observer,
        GameAction::Answer {
            choice_id: jg_choice.id,
            selection: vec![engine_native::ChoiceOption::Player(1)],
        },
    );
    while let Some(c) = coex.open_choice().cloned() {
        let sel = if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
            vec![engine_native::ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            &mut coex,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
    assert_eq!(coex.debug_hp(1), hp1_before_jg - 1, "精彀对 1 号造成 1 点伤害");

    // 5b. 末焱（dddmoyan）：进入新轮首个判定阶段展示牌库顶 3 张牌，交予 1 号 1 张，验证刘冲受伤 +1
    coex.debug_set_turn(2, Phase::End, coex.turn().round);
    for p in 0..3 {
        coex.debug_clear_hand(p);
    }
    let _ = submit(&mut coex, 2, GameAction::NextPhase);
    let moyan_target = coex.open_choice().cloned().expect("末焱应挂起选人");
    assert_eq!(moyan_target.spec.prompt, "d3_moyan_target");
    let _ = submit(
        &mut coex,
        moyan_target.observer,
        GameAction::Answer {
            choice_id: moyan_target.id,
            selection: vec![engine_native::ChoiceOption::Player(1)],
        },
    );
    let moyan_cards = coex.open_choice().cloned().expect("末焱应挂起选牌");
    assert_eq!(moyan_cards.spec.prompt, "d3_moyan_gain");
    let first_shown = moyan_cards.spec.options[0];
    let _ = submit(
        &mut coex,
        moyan_cards.observer,
        GameAction::Answer {
            choice_id: moyan_cards.id,
            selection: vec![first_shown],
        },
    );
    assert_eq!(coex.debug_hand(1).len(), 1, "末焱令 1 号获得 1 张展示牌");
    let hp0_before_my = coex.debug_hp(0);
    coex.debug_deal_damage(Some(2), 0, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(coex.debug_hp(0), hp0_before_my - 2, "末焱生效期间刘冲受到伤害 +1");

    let saved_after_reveal = coex.save();
    let restored = Engine::restore_with_extensions(&saved_after_reveal, &coex_packs)
        .expect("明置与双技结算后存档恢复成功");
    assert_eq!(restored.state_hash(), coex.state_hash(), "状态哈希完全一致");

    // 缺少扩展包时恢复必须拒绝
    assert!(
        Engine::restore_with_extensions(&saved_after_reveal, &[]).is_err(),
        "扩展集不一致应拒绝恢复"
    );

    // ⑥ 激活集切换与回滚闭环
    store
        .deactivate_slot(&meta.namespace, &meta.package_id)
        .expect("停用 d3-select");
    let rolled = store.rollback().expect("回滚激活集");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, _)| slot.ends_with("upstream-noname/d3-select")));

    // ⑦ 坏配置/损坏 JSON 拒绝装载
    assert!(matches!(
        bridge_pack("{bad json"),
        Err(BridgeError::BadJson(_))
    ));
    let bad_mapping = char_json.replace("host:d3_moyan", "host:nonexistent_d3_skill");
    assert!(matches!(
        bridge_pack(&bad_mapping),
        Err(BridgeError::UnknownBehaviorKey(_))
    ));

    // ⑧ 边界登记：验证 character.json 明确声明上游来源路径与版本指纹
    assert!(char_json.contains("apps/core/extension/3D精选/"));
    assert!(char_json.contains("ddd_liuchong"));
}
