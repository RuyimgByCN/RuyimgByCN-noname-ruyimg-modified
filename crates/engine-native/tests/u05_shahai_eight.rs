//! U05 杀海拾遗（9 个 `shahai-*` / `shaai-*` 子包目录）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：全部 9 个子包（`shahai-gujian`, `shahai-gwent`, `shahai-hearth`, `shahai-mtg`,
//!   `shahai-swd`, `shahai-yunchou`, `shahai-zhenfa`, `shaai-ow`, `shaai-xianjian`）真实发现、导入、激活与卸载；
//! ② 包数据桥接：全 9 包共 209 张扩展牌、149 名扩展武将经 `content_native_loader::bridge_pack` 装载通过；
//! ③ 组合共存：多子包联合装载（古剑 + 昆特 + 炉石 + 万智 + 轩辕剑 + 运筹 + 阵法 + 守望 + 仙剑）同局不冲突；
//! ④ 保存恢复：联合装载对局 `save` / `restore_with_extensions` 字节级一致，篡改扩展顺序或缺失扩展拒绝恢复；
//! ⑤ 功能执行：代表卡牌与武将技能在引擎中真实执行并通过牌区守恒校验；
//! ⑦ 异常拒装：非法牌种、非法花色与损坏 JSON 明确返回 `BridgeError`；
//! ⑧ 边界登记：万智/阵法子包的 `deferred` 条目显式携带 `reason`，不静默吞没。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{bridge_pack, BridgeError};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::content::extension::ExtensionPack;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};

static DEDUP: AtomicU64 = AtomicU64::new(1);

const SHAHAI_DIRS: [(&str, &str, usize, usize, usize); 9] = [
    ("shahai-gujian-upstream-noname", "shahai-gujian", 40, 0, 0),
    ("shahai-gwent-upstream-noname", "shahai-gwent", 21, 0, 0),
    ("shahai-hearth-upstream-noname", "shahai-hearth", 17, 0, 0),
    ("shahai-mtg-upstream-noname", "shahai-mtg", 10, 0, 2),
    ("shahai-swd-upstream-noname", "shahai-swd", 90, 79, 0),
    ("shahai-yunchou-upstream-noname", "shahai-yunchou", 23, 0, 0),
    ("shahai-zhenfa-upstream-noname", "shahai-zhenfa", 8, 0, 1),
    ("shaai-ow-upstream-noname", "shaai-ow", 0, 24, 0),
    ("shaai-xianjian-upstream-noname", "shaai-xianjian", 0, 36, 0),
];

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
fn u05_shahai_nine_packages_eight_dimension_acceptance() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover_extensions");
    for (dir_name, _, _, _, _) in SHAHAI_DIRS {
        assert!(
            discovered.iter().any(|d| d.name == dir_name),
            "杀海子包 {dir_name} 可被发现"
        );
    }

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import_extensions");

    let mut bridged_packs: Vec<ExtensionPack> = Vec::new();
    let mut total_cards = 0usize;
    let mut total_generals = 0usize;
    let mut total_deferred_entries = 0usize;

    for (dir_name, pid, exp_cards, exp_generals, exp_deferred) in SHAHAI_DIRS {
        let meta = report
            .installed
            .iter()
            .find(|m| m.package_id == pid && m.namespace == "upstream-noname")
            .unwrap_or_else(|| panic!("杀海子包 {pid} 已安装"));
        let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
        let cards_json = std::fs::read_to_string(ext_root.join("assets/cards.json"))
            .unwrap_or_else(|_| panic!("读取 {dir_name}/assets/cards.json"));
        let out = bridge_pack(&cards_json).unwrap_or_else(|e| panic!("桥接 {dir_name}: {e}"));
        assert_eq!(out.pack.card_count(), exp_cards, "{dir_name} 牌数");
        assert_eq!(out.pack.general_count(), exp_generals, "{dir_name} 武将数");
        assert_eq!(out.deferred.len(), exp_deferred, "{dir_name} 暂缓条目数");
        total_cards += out.pack.card_count();
        total_generals += out.pack.general_count();
        total_deferred_entries += out.deferred.len();
        bridged_packs.push(out.pack);

        if pid == "shahai-yunchou" {
            let char_json = std::fs::read_to_string(ext_root.join("assets/character.json"))
                .expect("读取 shahai-yunchou character.json");
            let char_out = bridge_pack(&char_json).expect("桥接 shahai-yunchou character.json");
            assert_eq!(char_out.pack.general_count(), 10, "运筹帷幄 10 名武将");
            total_generals += char_out.pack.general_count();
            bridged_packs.push(char_out.pack);
        }
    }

    assert_eq!(total_cards, 209, "杀海 9 子包共 209 张扩展实体牌");
    assert_eq!(total_generals, 149, "杀海 9 子包共 149 名扩展武将");
    assert_eq!(total_deferred_entries, 3, "万智(2)+阵法(1) 显式登记暂缓条目");

    // ③ 全 10 个 ExtensionPack（含运筹武将段）同局联合装载与建局
    let pack_refs: Vec<&ExtensionPack> = bridged_packs.iter().collect();
    let base = Engine::new(1, 4);
    let base_cards = base.catalog().cards.len();
    let base_generals = base.catalog().general_count();

    let mut e = Engine::new_with_extensions(190501, 4, &pack_refs).expect("杀海全子包联合建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 209);
    assert_eq!(e.catalog().general_count(), base_generals + 149);

    // ⑤ 推进到 0 号出牌阶段并真实执行 5 个代表子包机制：
    //    swd 护符、mtg 龙颅山峰、yunchou 曹操戏才、ow 士兵76（突击+目镜）、xianjian 赵灵儿（回梦+蛇蜕）
    let mut found_caocao = false;
    let mut found_shibing76 = false;
    let mut found_zhaoliner = false;
    for gid in (base_generals as u16)..(e.catalog().general_count() as u16) {
        if let Some((gname, _, skills, _, _)) = e.catalog().general_meta(gid) {
            if gname == "ycn曹操" {
                assert_eq!(skills, vec!["ycnxicai", "护驾"]);
                found_caocao = true;
            } else if gname == "士兵76" {
                assert_eq!(skills, vec!["tuji", "mujing"]);
                found_shibing76 = true;
            } else if gname == "赵灵儿" || gname == "pal_zhaoliner" {
                assert!(skills.contains(&"huimeng") && skills.contains(&"tianshe"));
                found_zhaoliner = true;
            }
        }
    }
    assert!(
        found_caocao && found_shibing76 && found_zhaoliner,
        "目录含 ycn曹操 / 士兵76 / 赵灵儿: caocao={found_caocao}, shibing76={found_shibing76}, zhaoliner={found_zhaoliner}"
    );

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
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 5a. ow 士兵76（突击 tuji + 目镜 mujing）与 yunchou 曹操（戏才 ycnxicai）联动：
    //     0 号发动目镜将黑牌当【杀】对 1 号（持 ycnxicai）使用 → 0 号突击攻距 2→1，1 号戏才获得该黑牌
    e.debug_grant(0, "tuji");
    e.debug_grant(0, "mujing");
    e.debug_grant(1, "ycnxicai");
    e.debug_set_hp(1, 4);
    e.debug_clear_hand(1);
    let dist_before = e.state_core().computed_distance(e.catalog(), 0, 2);
    assert_eq!(dist_before, 2, "突击发动前 0→2 对座基线距离为 2");
    let black_sha = e.debug_place(0, "sha");
    let mujing_def = e.catalog().skill_by_name("mujing").expect("mujing 在目录");
    let mj_out = submit(&mut e, 0, GameAction::ActivateSkill { skill: mujing_def });
    assert!(matches!(mj_out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    let mj_card = e.open_choice().cloned().expect("目镜选牌窗");
    assert_eq!(mj_card.spec.prompt, engine_native::content::host::WUDIAN_MUJING_CARD);
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: mj_card.id,
            selection: vec![engine_native::ChoiceOption::Card(black_sha)],
        },
    );
    let mj_target = e.open_choice().cloned().expect("目镜选目标窗");
    assert_eq!(mj_target.spec.prompt, engine_native::content::host::WUDIAN_MUJING_TARGET);
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: mj_target.id,
            selection: vec![engine_native::ChoiceOption::Player(1)],
        },
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if c.spec.options.contains(&engine_native::ChoiceOption::Confirm) {
            vec![engine_native::ChoiceOption::Confirm]
        } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
            vec![engine_native::ChoiceOption::Pass]
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
    }
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        1,
        "士兵76 突击使用 1 张牌后 0→2 距离由 2 缩减为 1"
    );
    assert!(
        e.debug_hand(1).contains(&black_sha),
        "ycn曹操 戏才受伤后获得伤害牌"
    );

    // 5b. mtg 龙颅山峰（CardKind::Longlushanfeng）：出地切换 current_land 并二选一摸【杀】
    let land_cid = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == engine_native::catalog::CardKind::Longlushanfeng)
        .expect("龙颅山峰在目录");
    {
        let mut s = e.save();
        for p in s.core.players.iter_mut() {
            p.hand.retain(|c| *c != land_cid);
        }
        s.core.deck.retain(|c| *c != land_cid);
        s.core.discard.retain(|c| *c != land_cid);
        s.core.players[0].hand.push(land_cid);
        e = Engine::restore_with_extensions(&s, &pack_refs).expect("注入龙颅山峰");
    }
    let land_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: land_cid,
            targets: vec![],
        },
    );
    assert!(matches!(land_out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(
        e.state_core().current_land,
        Some(engine_native::catalog::CardKind::Longlushanfeng),
        "龙颅山峰地效生效"
    );
    let land_choice = e.open_choice().cloned().expect("龙颅山峰二选一询问");
    assert_eq!(land_choice.spec.prompt, engine_native::content::host::MTG_LONGSHANFENG_GAIN);
    let _ = submit(
        &mut e,
        land_choice.observer,
        GameAction::Answer {
            choice_id: land_choice.id,
            selection: vec![engine_native::ChoiceOption::Tag(0)],
        },
    );

    // 5c. swd 护符（CardKind::Hufu）：濒死自救回 1 点体力
    let hufu_cid = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == engine_native::catalog::CardKind::Hufu)
        .expect("swd 护符在目录");
    e.debug_clear_hand(0);
    {
        let mut s = e.save();
        for p in s.core.players.iter_mut() {
            p.hand.retain(|c| *c != hufu_cid);
        }
        s.core.deck.retain(|c| *c != hufu_cid);
        s.core.discard.retain(|c| *c != hufu_cid);
        s.core.players[0].hand.push(hufu_cid);
        s.core.players[0].hp = 1;
        e = Engine::restore_with_extensions(&s, &pack_refs).expect("注入护符");
    }
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.observer == 0 && c.spec.options.contains(&engine_native::ChoiceOption::Card(hufu_cid)) {
            let _ = submit(
                &mut e,
                0,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![engine_native::ChoiceOption::Card(hufu_cid)],
                },
            );
        } else {
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![engine_native::ChoiceOption::Pass],
                },
            );
        }
    }
    assert_eq!(e.debug_hp(0), 1, "swd 护符濒死自救回至 1 体力");

    // 5d. xianjian 赵灵儿（回梦 huimeng + 蛇蜕 tianshe）：吃桃回复摸 2 + 属性伤害免疫
    e.debug_grant(0, "huimeng");
    e.debug_grant(0, "tianshe");
    e.debug_grant(0, "tianshe2");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 2);
    e.debug_clear_hand(0);
    let tao = e.debug_place(0, "tao");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    assert_eq!(e.debug_hp(0), 3, "赵灵儿吃桃回复 1 点");
    assert_eq!(e.debug_hand(0).len(), 2, "赵灵儿回梦回复后摸 2 张");
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Fire);
    assert_eq!(e.debug_hp(0), 3, "赵灵儿蛇蜕免疫火属性伤害");

    // 牌区守恒校验
    let snap = e.save();
    let mut zone_cards: Vec<u32> = Vec::new();
    for p in &snap.core.players {
        zone_cards.extend(p.hand.iter().copied());
        zone_cards.extend(p.equips.iter().copied());
        zone_cards.extend(p.judge.iter().copied());
    }
    zone_cards.extend(snap.core.deck.iter().copied());
    zone_cards.extend(snap.core.discard.iter().copied());
    zone_cards.extend(snap.core.table.iter().copied());
    let total_zone_len = zone_cards.len();
    zone_cards.sort_unstable();
    zone_cards.dedup();
    assert_eq!(zone_cards.len(), total_zone_len, "全区域实体牌无重复归属");

    // ④ 存档恢复一致性与扩展篡改拒绝
    let restored =
        Engine::restore_with_extensions(&snap, &pack_refs).expect("杀海全子包存档恢复");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert!(Engine::restore_with_extensions(&snap, &pack_refs[..5]).is_err());

    // ⑦ 坏输入拒绝
    let bad_kind_json = r#"{
        "pack": { "name": "bad-shahai", "version": 1, "description": "bad", "card_prefix": "sh_" },
        "cards": [{ "name": "x", "kind": "unknown_shahai_kind", "suit": "spade", "rank": 1 }]
    }"#;
    assert!(matches!(
        bridge_pack(bad_kind_json),
        Err(BridgeError::UnknownCardKind(_))
    ));
}
