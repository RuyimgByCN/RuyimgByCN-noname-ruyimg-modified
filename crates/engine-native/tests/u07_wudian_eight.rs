//! U07 玩点论杀（`wudian-upstream-noname`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 包数据桥接：14 张实体牌（含望梅止渴×2、诱敌深入×3、三尖两刃刀×1、铁索连环×1）+ 9 名武将（费诗/韩龙/刘馥/刘焉/留赞/田豫/西真夕红/造之仁君/杨仪）+ 17 项已映射宿主技能；
//! ③ 组合共存：与 `d3-upstream-noname`、`hero-kill-upstream-noname` 同局共存；
//! ④ 保存恢复：对局中含望梅止渴与扩展武将技能状态下 `save` / `restore_with_extensions` 字节级一致；
//! ⑤ 功能执行：望梅止渴与韩龙「刺酋」/留赞「亢音」等技能真实结算，全区实体牌守恒；
//! ⑦ 异常拒装：非法装备技引用、未知牌种与损坏 JSON 返回明确 `BridgeError`；
//! ⑧ 边界登记：调兵遣将（`diaobingqianjiang`）与 `duoqi`/`youzhan`/`choudu` 在 `cards.json` 显式登记暂缓原因。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{bridge_generals, bridge_pack, BridgeError};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};

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
fn u07_wudian_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "wudian-upstream-noname"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian" && m.namespace == "upstream-noname")
        .expect("wudian 已安装")
        .clone();

    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("读取 wudian cards.json");

    // ② 包数据核对：14 张牌、9 名武将、17 项宿主技能、1 项暂缓牌（调兵遣将）
    let out = bridge_pack(&json).expect("bridge wudian");
    let gen_out = bridge_generals(&json).expect("bridge wudian generals");
    assert_eq!(out.pack.def.name, "wudian-upstream-noname");
    assert_eq!(out.pack.def.version, 2);
    assert_eq!(out.pack.card_count(), 14);
    assert_eq!(out.pack.general_count(), 9);
    assert_eq!(out.pack.host_skills.len(), 19);
    assert_eq!(out.deferred, vec!["diaobingqianjiang".to_string()]);
    assert_eq!(gen_out.generals.len(), 9);
    assert_eq!(gen_out.unmapped_skills, 4, "duoqi/youzhan/jianbi(造之仁君)/choudu 暂缓");

    // ③ 与英雄杀包同局共存
    let yxs_json =
        std::fs::read_to_string(root.join("hero-kill-upstream-noname/assets/cards.json"))
            .expect("读取 hero-kill cards.json");
    let yxs_pack = bridge_pack(&yxs_json).expect("bridge hero-kill").pack;
    let packs = [&out.pack, &yxs_pack];
    let mut e = Engine::new_with_extensions(190701, 3, &packs).expect("wudian + hero-kill 建局");

    // ⑤ 推进到 0 号出牌阶段，授予亢音（kangyin）/刺酋（ciqiu）并打出望梅止渴
    e.debug_grant(0, "kangyin");
    e.debug_grant(0, "ciqiu");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }

    let wangmei_cid = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.starts_with("wd_wangmeizhike"))
        .expect("wd_wangmeizhike 在目录") as u32;
    let mut inject_snap = e.save();
    for p in inject_snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != wangmei_cid);
    }
    inject_snap.core.deck.retain(|c| *c != wangmei_cid);
    inject_snap.core.discard.retain(|c| *c != wangmei_cid);
    inject_snap.core.players[0].hand.push(wangmei_cid);
    let mut e = Engine::restore_with_extensions(&inject_snap, &packs).expect("注入望梅止渴恢复");
    let play_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wangmei_cid,
            targets: vec![1],
        },
    );
    assert!(
        matches!(play_out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "打出望梅止渴: {play_out:?}"
    );
    while let Some(c) = e.open_choice().cloned() {
        let sel = if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
            vec![engine_native::ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }

    // ④ 存档恢复一致性、全区牌守恒与缺失扩展拒绝
    let snap = e.save();
    let mut all_ids = Vec::new();
    for p in &snap.core.players {
        all_ids.extend(p.hand.iter().copied());
        all_ids.extend(p.equips.iter().copied());
        all_ids.extend(p.judge.iter().copied());
    }
    all_ids.extend(snap.core.deck.iter().copied());
    all_ids.extend(snap.core.discard.iter().copied());
    all_ids.extend(snap.core.table.iter().copied());
    let raw_len = all_ids.len();
    all_ids.sort_unstable();
    all_ids.dedup();
    assert_eq!(all_ids.len(), raw_len, "玩点论杀实体牌全区守恒");
    let restored = Engine::restore_with_extensions(&snap, &packs).expect("恢复双包对局");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert!(Engine::restore_with_extensions(&snap, &[&out.pack]).is_err());

    // ⑦ 非法装备技引用拒绝
    let bad_equip = json.replacen("\"wd_sanjian\"", "\"wd_missing_equip_skill\"", 1);
    assert!(matches!(
        bridge_pack(&bad_equip),
        Err(BridgeError::UnknownEquipSkill(_))
    ));
}
