use super::*;

/// T12 swd 武将面批 21：三技四将部分转正——流虹（拓跋玉儿：使用杀后
/// 频率技摸 1，无询问窗）；墨剑（水镜/屈娴：自己使用杀指定目标后确认，
/// 目标摸 1 + 自己回 1，上游 shaBegin 等价接入 CardUsed 帧）；道法（壶
/// 中仙：其他角色造成伤害后确认，来源弃 1 张手牌——登记差异上游来源
/// 自选弃哪张，原生确定性弃首张）。
#[test]
fn swd_generals_b21_liuhong_mojian_daofa() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62004, 4, &[&pack]).expect("建局");

    // ① 流虹：p0 使用杀 → 频率技自动摸 1（无确认窗）
    e.debug_grant(0, "swdliuhong");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 1, "流虹：使用杀自动摸 1（用 1 摸 1）");
    assert!(e.open_choice().is_none(), "流虹：频率技无询问窗");

    // 反例 ①：非杀（桃）不摸
    e.debug_set_hp(0, 3);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let tao = e.debug_place(0, "tao");
    e.debug_set_hp(0, 3);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "流虹反例：用桃不摸");

    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    // ② 墨剑：p0（受伤）使用杀指定 p1 → 确认 → p1 摸 1、p0 回 1
    e.debug_grant(0, "mojian");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0 - 1);
    let sha2 = e.debug_place(0, "sha");
    let h1 = hand_len_w(&e, 1);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_order") {
        let c = e.open_choice().unwrap().clone();
        let def = e.catalog().skill_by_name("mojian").unwrap();
        submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Skill(def)] });
    }
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("墨剑确认窗");
    let conf: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Confirm))
            .cloned()
            .collect()
    };
    assert_eq!(conf.len(), 1, "墨剑确认面含 Confirm");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: conf,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 1), h1 + 1, "墨剑：目标摸 1");
    assert_eq!(e.debug_hp(0), max0, "墨剑：自己回 1 至满");

    // 反例 ②a：拒绝 → 目标不摸、自己不回
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, max0 - 1);
    let sha3 = e.debug_place(0, "sha");
    let (hp0, h1b) = (e.debug_hp(0), hand_len_w(&e, 1));
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_order") {
        let c = e.open_choice().unwrap().clone();
        let def = e.catalog().skill_by_name("mojian").unwrap();
        submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Skill(def)] });
    }
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("墨剑拒绝窗");
    // allow_cancel 取消 = 空选择（确认面仅 [Confirm]，拒绝走取消）
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 1), h1b, "墨剑反例：拒绝 → 目标不摸");
    assert_eq!(
        e.debug_hp(0),
        hp0,
        "墨剑反例：拒绝 → 自己不回（攻击者本不受伤）"
    );

    // 反例 ②b：满血确认 → 目标仍摸、自己不回（recover 饱和）
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, max0);
    e.debug_set_hp(1, 4);
    let sha4 = e.debug_place(0, "sha");
    let h1c = hand_len_w(&e, 1);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha4,
            targets: vec![1],
        },
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("墨剑满血确认窗");
    let conf: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Confirm))
            .cloned()
            .collect()
    };
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: conf,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 1), h1c + 1, "墨剑满血：目标仍摸 1");
    assert_eq!(e.debug_hp(0), max0, "墨剑满血：回复饱和不溢出");

    // ③ 道法：p3 持道法；p0 杀伤 p1（p0 另持 1 张牌）→ p3 确认 →
    // p0 弃首张手牌（登记差异：上游来源自选弃哪张）。
    // 撤销流虹（①所授）以免 p0 手牌计数被摸牌扰动。
    {
        let def_lh = e.catalog().skill_by_name("swdliuhong").expect("流虹 def");
        let def_mj = e.catalog().skill_by_name("mojian").expect("墨剑 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_lh));
        snap.core.skills.remove(&(0, def_mj));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销流虹/墨剑");
    }
    e.debug_grant(3, "daofa");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _extra = e.debug_place(0, "tao");
    let sha5 = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha5,
            targets: vec![1],
        },
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("道法确认窗");
    assert_eq!(obs, 3, "道法窗归持有者 p3");
    let conf: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Confirm))
            .cloned()
            .collect()
    };
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: conf,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "道法：来源弃首张手牌（tao）");

    // 反例 ③a：拒绝 → 来源保留手牌
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _extra = e.debug_place(0, "tao");
    let sha6 = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha6,
            targets: vec![1],
        },
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("道法拒绝窗");
    // allow_cancel 取消 = 空选择（确认面仅 [Confirm]，拒绝走取消）
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 1, "道法反例：拒绝 → 来源保留手牌");

    // 反例 ③b：来源=持有者 → 无窗（filter d.source != owner）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _p3card = e.debug_place(3, "tao");
    e.debug_set_hp(1, 4);
    e.debug_deal_damage(Some(3), 1, 1, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "道法反例：自己造成伤害不触发");

    // 反例 ③c：来源空手 → 无窗
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "道法反例：来源无手牌不触发");
}
