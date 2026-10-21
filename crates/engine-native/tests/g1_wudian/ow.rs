use super::*;

/// 士兵76（U07 ow 纵切，规则 v118）：突击（回合内每用一张牌攻距-1）+
/// 目镜（黑牌当杀 declared-Sha）。力场暂缓（下准备阶段时点）。
#[test]
fn ow_shibing_tuji_and_mujing() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9101, 5, &[&pack]).expect("建局");
    // 目录：3 张卡（cards 空）但 general 元数据 1 名（士兵76）——
    // 扩展武将接在基础表后
    let gen_base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let mut shibing_skills = None;
    for idx in gen_base..(gen_base + 8) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "士兵76" {
                shibing_skills = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(
        shibing_skills.map(|v| v.to_vec()),
        Some(vec!["tuji", "mujing"]),
        "士兵76 目录技能=突击/目镜（lichang 暂缓未映射不入目录）"
    );

    advance_loop(&mut e, 0);
    e.debug_grant(0, "tuji");
    e.debug_grant(0, "mujing");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 4);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_clear_hand(4);

    // 基线：0↔2 对座 2
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        2,
        "对座基线"
    );

    // 突击：出牌阶段用 1 张牌（目镜当杀）→ 计数 1 → 攻距 -1
    // 目镜：黑牌（spade）当杀打 2 号
    let black = e.debug_place(0, "sha"); // spade/club sha 均黑
    let def_mj = e.catalog().skill_by_name("mujing").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_mj });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("目镜选牌挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_MUJING_CARD, 0)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(black)],
        },
    );
    let c = e.open_choice().expect("目镜目标挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_MUJING_TARGET, 0)
    );
    let (cid, obs) = (c.id, c.observer);
    e.debug_set_hp(2, 4);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hp(2), 3, "目镜当杀：2 号掉血（4→3）");
    // 突击计数：用 1 张 → 0↔2 距离 2-1=1
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        1,
        "突击：用 1 张攻距 -1"
    );

    // 计数清零：换回合后距离回基线
    let round0 = e.turn().round;
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        2,
        "换回合计数清零回基线"
    );
}

// ---------------------------------------------------------------------------
// 整包验收 T7（2026-10-10）：ow、xianjian 两个纯武将包补齐真实宿主全生命
// 周期（15/15）。与 `wudian_real_host_full_lifecycle` 同款五步：独立安装
// +激活 → 安装包→原生建局（目录元数据）→ 对局中卸载 → 在局锁定副本续玩
// → 撤销后重装/重新 import 被拒。
// ---------------------------------------------------------------------------

/// ow（shaai-ow-upstream-noname，24 名武将元数据、0 张卡）全生命周期。
#[test]
fn ow_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shaai-ow")
        .expect("shaai-ow 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(
        store.read_active().unwrap().packages.contains_key(&slot),
        "激活集"
    );

    // ② 安装包 → 原生建局（0 卡注入；士兵76=突击/目镜接线生效）
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert_eq!(pack.card_count(), 0, "ow 为纯武将包");
    assert_eq!(pack.generals.len(), 24, "24 名武将元数据");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6105, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards, "纯武将包不注入卡牌");
    let gen_base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let mut shibing_skills = None;
    for idx in gen_base..(gen_base + 40) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "士兵76" {
                shibing_skills = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(
        shibing_skills.map(|v| v.to_vec()),
        Some(vec!["tuji", "mujing"]),
        "士兵76 突击/目镜接线生效（力场暂缓不入目录）"
    );
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "shaai-ow", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store
        .read_revoke()
        .unwrap()
        .is_slot_revoked(&meta.namespace, "shaai-ow"));
    assert!(
        !store.read_active().unwrap().packages.contains_key(&slot),
        "移出激活集"
    );

    // ④ 在局锁定副本续玩
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "在局续玩: {out:?}"
    );

    // ⑤ 重装守卫
    let bytes = std::fs::read(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("package.npack.json"),
    )
    .expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(
        matches!(err, Err(content_runtime::RuntimeError::Revoked(_))),
        "撤销后重装应被拒: {err:?}"
    );

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}
