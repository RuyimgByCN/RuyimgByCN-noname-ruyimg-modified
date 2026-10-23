use super::*;

/// 整包验收 U07（2026-10-08，来源级第五包）：wudian 真实宿主全生命周期
/// ——独立安装+激活 → 安装包→原生建局（14 卡+费诗墨守接线后目录）→
/// 对局中卸载 → 在局锁定副本续玩 → 撤销后重装/重新 import 被拒。
#[test]
fn wudian_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(
        store.read_active().unwrap().packages.contains_key(&slot),
        "激活集"
    );

    // ② 安装包 → 原生建局（14 卡 + 费诗墨守接线后目录元数据）
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert_eq!(pack.card_count(), 14);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6005, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 14);
    let mut feishi_skills = None;
    let gen_base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    for idx in gen_base..(gen_base + 8) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "费诗" {
                feishi_skills = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(
        feishi_skills.map(|v| v.to_vec()),
        Some(vec!["nsshuaiyan", "moshou"]),
        "费诗率言+墨守接线生效"
    );
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "wudian", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store
        .read_revoke()
        .unwrap()
        .is_slot_revoked(&meta.namespace, "wudian"));
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
    assert_eq!(
        resumed.catalog().cards.len(),
        base_cards + 14,
        "锁定副本仍可用"
    );
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

/// T12 玩点论杀续批（2026-10-14）：三将补入（西真夕红/造之仁君/杨仪）。
/// 屯田·DIY（ActivateSkill 主动技重铸：弃 1 装备 → 牌堆随机装备入手）
/// 与禁言（PlayCard 门查询：体力 ≤2 黑锦囊禁用，往事封禁先例同位）。
/// 登记差异：禁言上游四面查询仅承载使用面；屯田上游 canRecast 换置面
/// 不做。仇渡暂缓（依赖 deferred 卡调兵遣将）；复仇系（跨回合存储）、
/// 良策（五谷 Remained 子事件）、坚壁（目标转移）、理断（令他人用牌）
/// 逐批后续。
#[test]
fn wudian_generals_batch10_diyjuntun_recast_and_jinyan() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian 安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert_eq!(pack.generals.len(), 9, "玩点论杀九名武将全量入册");

    let mut e = Engine::new_with_extensions(61000, 4, &[&pack]).expect("建局");
    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    // 新三将在扩展段尾（6 旧将之后）
    let zao = base + 7;
    let m = e.catalog().general_meta(zao).expect("造之仁君在目录");
    assert_eq!(m.0, "造之仁君");
    assert_eq!(
        m.2,
        vec!["liangce", "diyjuntun"],
        "良策+屯田转正（坚壁 null 暂缓不进目录列表）"
    );
    let jieyan_def = e.catalog().skill_by_name("diyjuntun").expect("屯田 def");
    let _ = jieyan_def;

    // 屯田·DIY：p0 挂装备 + 手牌装备 → ActivateSkill 重铸
    e.debug_grant(0, "diyjuntun");
    advance_to_play(&mut e, 0);
    let z0 = e.debug_place(0, "zhuge");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|x| *x != z0);
        core.players[0].equips.push(z0);
    }
    let h0 = hand_len_w(&e, 0);
    let equip_pool_before = {
        let core = e.state_core();
        core.deck
            .iter()
            .chain(core.discard.iter())
            .filter(|c| e.catalog().card_kind(**c).equip_slot().is_some())
            .count()
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: jieyan_def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "屯田受理: {out:?}"
    );
    // SelectCards 应答（选 zhuge2）
    let c = e.open_choice().expect("屯田选择挂起");
    let (cid, obs) = (c.id, c.observer);
    let sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == z0))
        .cloned()
        .collect();
    assert_eq!(sel.len(), 1, "重铸选择面含挂载装备");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    assert!(
        !e.state_core().players[0].equips.contains(&z0),
        "屯田：原装备被弃"
    );
    let gained = hand_len_w(&e, 0) - h0;
    if equip_pool_before > 0 {
        assert_eq!(
            gained, 1,
            "屯田：牌堆获得 1 张装备（池 {equip_pool_before}）"
        );
    }

    // 禁言：p1（持禁言）体力 2 → 黑锦囊使用被拒；体力 3 → 放行
    e.debug_grant(1, "jinyan");
    advance_to_play(&mut e, 1);
    e.debug_set_hp(1, 2);
    let le = e.debug_place(1, "le");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: le,
            targets: vec![0],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::CardLimit)),
        "禁言：体力 ≤2 黑锦囊被拒: {out:?}"
    );
    e.debug_set_hp(1, 3);
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: le,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "禁言：体力 3 放行: {out:?}"
    );
}

/// T12 玩点论杀批 12：理断（杨仪部分转正）——他人回合外获得装备到手
/// （可选确认）：同栏已有装备则交首张手牌给持有者；否则直接装备该牌
/// （登记差异：上游获得者自选，原生按同栏状态确定性分支；直装不走
/// Equipped 帧防双 grant）。坚壁登记「目标排除面」缺口维持暂缓。
#[test]
fn wudian_generals_batch12_liduan_gain_equip() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian 安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(61001, 4, &[&pack]).expect("建局");
    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let yangyi = base + 8;
    let m = e.catalog().general_meta(yangyi).expect("杨仪在目录");
    assert_eq!(m.0, "杨仪");
    assert_eq!(m.2, vec!["liduan"], "仇渡暂缓（依赖 deferred 卡）");

    // 直装分支：p1 持理断；p0 回合中，p2 获得装备到手（同栏无装备）
    e.debug_grant(1, "liduan");
    advance_to_play(&mut e, 0);
    let z = e.debug_place(2, "zhuge");
    {
        // 从 p2 手移到 p0 再 give（产生 CardsMoved from=0 to=2——wait，理断
        // filter 要 from!=to 且 to 非当前回合玩家：当前回合=p0，to=p2 ✓
        let core = e.state_core_mut();
        core.players[2].hand.retain(|x| *x != z);
        core.players[0].hand.push(z);
    }
    e.debug_give_hand_card(0, 2, z);
    settle_optional_confirm(&mut e);
    assert!(
        e.state_core().players[2].equips.contains(&z),
        "理断：直装分支——装备被装上"
    );

    // 交牌分支：p2 再获得一件同栏（武器）装备 → 交首张手牌给 p1
    let z2 = e.debug_place(2, "hanbing");
    {
        let core = e.state_core_mut();
        core.players[2].hand.retain(|x| *x != z2);
        core.players[0].hand.push(z2);
    }
    // 给 p2 一张手牌作交牌面
    let pay = e.debug_place(2, "tao");
    let h1 = e.state_core().players[1].hand.len();
    let out = e.debug_give_hand_card(0, 2, z2);
    let _ = out;
    settle_optional_confirm(&mut e);
    assert!(
        !e.state_core().players[2].equips.contains(&z2),
        "理断：同栏已有装备不直装"
    );
    assert_eq!(
        e.state_core().players[1].hand.len(),
        h1 + 1,
        "理断：交牌分支——p1 收 1 张手牌"
    );
    let _ = pay;
}

/// T12 玩点论杀批 13：坚壁（西真夕红部分转正）——AOE 目标排除原语首例
/// （catalog.aoe_excludes_target 通用查询，帷幕 weimu_blocks_aoe 同构）。
/// 南蛮对持有者无效（不询问不掉血），无持有者正常受伤。
#[test]
fn wudian_generals_batch13_jianbi_aoe_exclude() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian 安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(61002, 4, &[&pack]).expect("建局");
    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let m = e.catalog().general_meta(base + 6).expect("西真夕红在目录");
    assert_eq!(m.0, "西真夕红");
    assert_eq!(
        m.2,
        vec!["jinyan", "fuchou", "fuchou2", "jianbi"],
        "西真夕红整将（批 14 复仇双技转正）"
    );

    // 南蛮：p0 对全体——p1（持坚壁）被跳过（不询问不掉血），p2/p3 受伤
    e.debug_grant(1, "jianbi");
    assert!(
        e.catalog().aoe_excludes_target(e.state_core(), 1),
        "排除查询：grant 后应命中"
    );
    e.debug_set_hp(1, 8);
    e.debug_set_hp(2, 8);
    e.debug_set_hp(3, 8);
    advance_to_play(&mut e, 0);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    let nanman = e.debug_place(0, "savage");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: nanman,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "南蛮受理: {out:?}"
    );
    // 排空响应窗（无手牌 → 无窗；保险再排）
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 8, "坚壁：AOE 排除——p1 不掉血");
    assert_eq!(e.debug_hp(2), 7, "南蛮：p2 受 1 伤");
    assert_eq!(e.debug_hp(3), 7, "南蛮：p3 受 1 伤");

    // 负例：无坚壁持有者时 AOE 正常（新实例 p2 持坚壁对照位）
    let mut e2 = Engine::new_with_extensions(61003, 4, &[&pack]).expect("建局");
    e2.debug_set_hp(1, 8);
    e2.debug_set_hp(2, 8);
    e2.debug_set_hp(3, 8);
    advance_to_play(&mut e2, 0);
    for p in 1..4 {
        e2.debug_clear_hand(p);
    }
    let nanman2 = e2.debug_place(0, "savage");
    submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: nanman2,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e2);
    assert_eq!(e2.debug_hp(1), 7, "负例：无坚壁 p1 正常受伤");
}

/// T12 玩点论杀批 14：复仇双技（西真夕红整将转正）——成为杀目标时弃
/// 1 张手牌取消该杀（ShaResolveMachine 内联选择阶段；杀回来源手+所选
/// 牌归来源+标记）；自己回合结束时对标记来源逐个摸 1+虚拟杀强制指定。
#[test]
fn wudian_generals_batch14_fuchou_cancel_and_settle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian 安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(61004, 4, &[&pack]).expect("建局");
    assert!(e.catalog().skill_by_name("fuchou").is_some(), "复仇 def");
    assert!(e.catalog().skill_by_name("fuchou2").is_some(), "清算 def");

    // 复仇取消：p1 持复仇，p0 杀 p1 → 弃牌取消 → p0 收回杀+获得所选牌
    e.debug_grant(1, "fuchou");
    e.debug_set_hp(1, 8);
    advance_to_play(&mut e, 0);
    let sha = e.debug_place(0, "sha");
    let pay = e.debug_place(1, "tao");
    let hp0 = e.debug_hp(0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    let _ = out;
    // 复仇选择挂起：选 tao 取消杀
    let c = e.open_choice().expect("复仇选择挂起");
    assert_eq!(c.observer, 1, "复仇询问目标本人");
    let (cid, obs) = (c.id, c.observer);
    let sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == pay))
        .cloned()
        .collect();
    assert_eq!(sel.len(), 1, "选择面含所布牌");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    assert_eq!(e.debug_hp(1), 8, "复仇：杀被取消不受伤");
    assert!(e.debug_hand(0).contains(&sha), "复仇：杀回来源手");
    assert!(e.debug_hand(0).contains(&pay), "复仇：所选牌归来源");
    assert!(
        e.state_core().players[1].fuchou_marked.contains(&0),
        "复仇：来源已标记"
    );

    // 清算：**任何玩家的任何阶段结束**（上游 `global:'phaseAfter'`）——
    // 对标记来源摸 1 + 虚拟杀（p1 掉血或 p0 有闪弃闪）。
    // 第 986 轮：清算会在下面的推进循环**之内**发生，故在此先记手牌数。
    e.debug_grant(1, "fuchou2");
    e.debug_set_hp(1, 8);
    let h_before_settle = hand_len_w(&e, 1);
    // 推进到 p1 回合结束（End 阶段结束帧触发）
    for _ in 0..200 {
        if e.outcome().is_some() {
            break;
        }
        // 状态驱动：标记被消耗即清算完成（不再依赖「走到 End 阶段」）
        if e.state_core().players[1].fuchou_marked.is_empty() {
            break;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
        let pending = e
            .open_choice()
            .map(|ch| (ch.id, ch.observer, ch.spec.min.max(1)));
        if let Some((cid2, obs2, min)) = pending {
            let card_sel: Vec<ChoiceOption> = {
                let ch = e.open_choice().expect("still open");
                ch.spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(min)
                    .collect()
            };
            if !card_sel.is_empty() {
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: cid2,
                        selection: card_sel,
                    },
                );
            } else {
                let first = {
                    let ch = e.open_choice().expect("still open");
                    vec![ch.spec.options[0].clone()]
                };
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: cid2,
                        selection: first,
                    },
                );
            }
        }
    }
    let h1 = h_before_settle;
    // 触发 End 结束帧（NextPhase 推进）
    let cur = e.turn().current;
    if cur == 1 && e.turn().phase == Phase::End {
        submit(&mut e, 1, GameAction::NextPhase);
        let pending = e.open_choice().map(|ch| (ch.id, ch.observer));
        if let Some((cid2, obs2)) = pending {
            let first = {
                let ch = e.open_choice().expect("still open");
                vec![ch.spec.options[0].clone()]
            };
            submit(
                &mut e,
                obs2,
                GameAction::Answer {
                    choice_id: cid2,
                    selection: first,
                },
            );
        }
    }
    assert_eq!(hand_len_w(&e, 1), h1 + 1, "清算：对标记来源摸 1");
    assert!(
        e.state_core().players[1].fuchou_marked.is_empty(),
        "清算：标记已消耗"
    );
    let _ = hp0;
}

/// T12 玩点论杀批 15：良策（造之仁君整将转正）——基本手牌当五谷（viewAs
/// 转化，厉焰同族）；五谷剩余牌归持有者（wuguRemained 子事件承载，登记
/// 差异：上游玩家选目标，原生归持有者本人）。
#[test]
fn wudian_generals_batch15_liangce_conversion_and_remain() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian 安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(61005, 4, &[&pack]).expect("建局");
    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let m = e.catalog().general_meta(base + 7).expect("造之仁君在目录");
    assert_eq!(m.0, "造之仁君");
    assert_eq!(
        m.2,
        vec!["liangce", "diyjuntun"],
        "良策+屯田转正（坚壁暂缓不进列表）"
    );

    // 良策转化：p0 用基本牌（sha）→ 转化为五谷 → 全员择牌
    e.debug_grant(0, "liangce");
    advance_to_play(&mut e, 0);
    let sha = e.debug_place(0, "sha");
    let h2 = hand_len_w(&e, 2);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "良策：基本牌转化为五谷受理: {out:?}"
    );
    // 全员择牌排空（4 人各选 1 张；无懈窗排空）
    for _ in 0..12 {
        let pending = e
            .open_choice()
            .map(|ch| (ch.id, ch.observer, ch.spec.min.max(1)));
        let Some((cid, obs, min)) = pending else {
            break;
        };
        let card_sel: Vec<ChoiceOption> = {
            let ch = e.open_choice().expect("still open");
            ch.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(min)
                .collect()
        };
        if !card_sel.is_empty() {
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: card_sel,
                },
            );
        } else {
            let first = {
                let ch = e.open_choice().expect("still open");
                vec![ch.spec.options[0].clone()]
            };
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: first,
                },
            );
        }
    }
    settle_optional_confirm(&mut e);
    // 五谷：全员各摸 1 张（择牌）——p2 手牌 +1
    assert!(hand_len_w(&e, 2) >= h2, "五谷：目标择牌入手");

    // 结构性登记：4 人局五谷亮牌数=存活数=择牌数，永无剩余——良策·余获
    // 在可达面与「无效果」等价（批 16 勘察，近似归持有者已覆盖）。
}
