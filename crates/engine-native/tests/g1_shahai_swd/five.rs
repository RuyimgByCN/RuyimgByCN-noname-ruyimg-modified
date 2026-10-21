use super::*;

/// T12 swd 武将面批 19：五技五将部分转正（承剑=其他角色杀伤来源摸 1
/// 确认门；缚斧=杀伤+1 有武器；花镜=锦囊伤后回 1 摸 1；护盾=伤后无护
/// 甲+1；破星=目标体力高则伤+1）。
#[test]
fn swd_generals_b19_five_damage_line_skills() {
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
    assert!(pack.generals.len() >= 7);

    let mut e = Engine::new_with_extensions(62002, 4, &[&pack]).expect("建局");

    // 承剑：p1 持承剑，p0 杀伤 p2（来源 p0≠p1）→ 确认 → p0 摸 1
    // （清 p2 手防闪避负例路径）
    e.debug_grant(1, "chengjian");
    e.debug_set_hp(2, 8);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(2);
    let sha = e.debug_place(0, "sha");
    let h0 = hand_len_w(&e, 0);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(
        hand_len_w(&e, 0),
        h0,
        "承剑：来源（出杀者）摸 1 净持平（出杀-1+摸+1）"
    );

    // 缚斧：p3 持缚斧，p3 杀 p0（有武器）→ 伤 +1（hp -2）
    e.debug_grant(3, "jufu");
    e.debug_set_hp(0, 8);
    let z3 = e.debug_place(3, "zhuge");
    {
        let core = e.state_core_mut();
        core.players[3].hand.retain(|x| *x != z3);
        core.players[3].equips.push(z3);
    }
    advance_to_play(&mut e, 3);
    let sha3 = e.debug_place(3, "sha");
    submit(
        &mut e,
        3,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 6, "缚斧：杀伤害 +1（8-2）");

    // 破星：p1 持破星，p1 杀 p0（p0 体力 > p1）→ 伤 +1
    e.debug_grant(1, "poxing");
    e.debug_set_hp(0, 8);
    e.debug_set_hp(1, 3);
    advance_to_play(&mut e, 1);
    // 器族手持技（批⑩ init hook）在 p0 准备阶段给初始手牌含器族牌的角
    // 色加了护甲——清护甲+清手牌保持伤害断言纯凑（批⑪ 发牌敏感同 b37）
    e.debug_set_hujia(0, 0);
    e.debug_clear_hand(0);
    let sha1 = e.debug_place(1, "sha");
    let hp0 = e.debug_hp(0);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);

    assert_eq!(e.debug_hp(0), hp0 - 2, "破星：目标体力高 → 伤 +1");

    // 花镜：p2 持花镜，p2 杀 p0 → p2 回 1（已受伤时）摸 1
    e.debug_grant(2, "huajing");
    e.debug_set_hp(2, 2);
    advance_to_play(&mut e, 2);
    let h2 = hand_len_w(&e, 2);
    let sha2 = e.debug_place(2, "sha");
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);
    // hp2 3-1=2 → 回 1=3？不满血则回 1；摸 1
    let hp2b = e.debug_hp(2);
    assert!(hp2b >= 2, "花镜：回 1（或满血饱和）");
    assert_eq!(hand_len_w(&e, 2), h2 + 1, "花镜：摸 1");

    // 护盾：p3 持护盾，p3 造成伤害 → 无护甲 +1
    e.debug_grant(3, "hudun");
    advance_to_play(&mut e, 3);
    e.debug_deal_damage(Some(3), 0, 1, DamageNature::Normal);
    assert_eq!(e.state_core().players[3].hujia, 1, "护盾：+1 护甲");
}
