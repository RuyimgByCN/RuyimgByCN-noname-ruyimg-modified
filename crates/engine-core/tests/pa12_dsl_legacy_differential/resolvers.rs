//! PA12 批⑮～批⑰具名复合结算器、视为技与跨文件技能在旧线 `engine_core::vm_host` 上的编译与执行差分测试。

use super::*;

#[test]
fn pa12_batch15_resolvers_and_view_as_compile_and_execute() {
    let view_as_cases: &[(&str, &str, &str)] = &[
        (
            "kanpo",
            include_str!("../../../../content/original/ported/shenhua/kanpo.skill"),
            "wuxie",
        ),
        (
            "jiuchi",
            include_str!("../../../../content/original/ported/shenhua/jiuchi.skill"),
            "jiu",
        ),
        (
            "jxwusheng",
            include_str!("../../../../content/original/ported/offline/jxwusheng.skill"),
            "sha",
        ),
    ];
    for (id, src, want_as) in view_as_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译 view_as .skill");
        assert!(
            !pkg.view_as_index.is_empty(),
            "{id} 应生成非空 view_as_index"
        );
        assert_eq!(pkg.view_as_index[0].as_kind, *want_as);
    }

    let trigger_cases: &[(&str, &str, usize, &[&str])] = &[
        (
            "pot_weiyan",
            include_str!("../../../../content/original/ported/bingshi/pot_weiyan.skill"),
            0,
            &["ResolvePotweiyan"],
        ),
        (
            "pot_caoshuang",
            include_str!("../../../../content/original/ported/bingshi/pot_caoshuang.skill"),
            0,
            &["ResolvePotcaoshuang"],
        ),
        (
            "pot_lvyi",
            include_str!("../../../../content/original/ported/bingshi/pot_lvyi.skill"),
            0,
            &["ResolvePotlvyi"],
        ),
        (
            "pot_yuji",
            include_str!("../../../../content/original/ported/bingshi/pot_yuji.skill"),
            0,
            &["ResolvePotyuji"],
        ),
        (
            "pot_zhouyu",
            include_str!("../../../../content/original/ported/bingshi/pot_zhouyu.skill"),
            0,
            &["ResolvePotzhouyu"],
        ),
        (
            "sp_zhonghui",
            include_str!("../../../../content/original/ported/bingshi/sp_zhonghui.skill"),
            0,
            &["ResolveMbzhonghui"],
        ),
        (
            "ddd_liuhong",
            include_str!("../../../../content/original/ported/ddd/liuhong.skill"),
            0,
            &["ResolveDddLiuhong"],
        ),
        (
            "twguohuai",
            include_str!("../../../../content/original/ported/tw/twguohuai.skill"),
            0,
            &["ResolveTwguohuai"],
        ),
        (
            "ddd_xuelingyun",
            include_str!("../../../../content/original/ported/ddd/xuelingyun.skill"),
            0,
            &["ResolveDddXuelingyun"],
        ),
        (
            "pot_taishici",
            include_str!("../../../../content/original/ported/bingshi/pot_taishici.skill"),
            0,
            &["ResolvePottaishici"],
        ),
        (
            "ddd_xujing",
            include_str!("../../../../content/original/ported/ddd/xujing.skill"),
            0,
            &["ResolveDddXujing"],
        ),
        (
            "ddd_liuba",
            include_str!("../../../../content/original/ported/ddd/liuba.skill"),
            0,
            &["ResolveDddLiuba"],
        ),
        (
            "ddd_liuye",
            include_str!("../../../../content/original/ported/ddd/liuye.skill"),
            0,
            &["ResolveDddLiuye"],
        ),
        (
            "ddd_zhangkai",
            include_str!("../../../../content/original/ported/ddd/zhangkai.skill"),
            0,
            &["ResolveDddZhangkai"],
        ),
        (
            "extra_jilin",
            include_str!("../../../../content/original/ported/extra/jilin.skill"),
            0,
            &["ResolveHiddenSkillPile"],
        ),
        (
            "twfuman",
            include_str!("../../../../content/original/ported/tw/twfuman.skill"),
            0,
            &["DrawCardsFromSkill", "ResolveTwfuman"],
        ),
        (
            "twkujian",
            include_str!("../../../../content/original/ported/tw/twkujian.skill"),
            0,
            &["ResolveTwkujian"],
        ),
        (
            "twxiawei",
            include_str!("../../../../content/original/ported/tw/twxiawei.skill"),
            0,
            &["ResolveTwxiawei"],
        ),
        (
            "caoyi",
            include_str!("../../../../content/original/ported/xianding/caoyi.skill"),
            0,
            &["ResolveCaoyi"],
        ),
        (
            "dc_sb_lusu",
            include_str!("../../../../content/original/ported/xianding/dc_sb_lusu.skill"),
            0,
            &["ResolveDcSbLusu"],
        ),
        (
            "dc_sb_zhouyu",
            include_str!("../../../../content/original/ported/xianding/dc_sb_zhouyu.skill"),
            0,
            &["ResolveDcSbZhouyu"],
        ),
        (
            "mb_sunjun",
            include_str!("../../../../content/original/ported/bingshi/mb_sunjun.skill"),
            0,
            &["ResolveMbxiongtu"],
        ),
        (
            "ddd_caoshuang",
            include_str!("../../../../content/original/ported/ddd/caoshuang.skill"),
            0,
            &["ResolveDddCaoshuang"],
        ),
    ];
    for (id, src, trig_idx, want_prefixes) in trigger_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 85, "identity-standard").expect("开局");
        let _ = std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&pkg);
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        let entry = &pkg.trigger_index[*trig_idx];
        let module = tx
            .state()
            .content
            .module(&entry.module)
            .cloned()
            .unwrap_or_else(|| {
                let m = pkg
                    .modules
                    .iter()
                    .find(|m| m.name == entry.module)
                    .expect("模块");
                rule_vm::VerifiedModule::verify(m.clone()).expect("校验")
            });
        let out = engine_core::vm_host::run_module_with_event_cards(
            tx.state(),
            &module,
            entry.skill_def_id,
            [owner.raw(), other.raw(), other.raw(), 1],
            1,
            &[1],
            &[2],
            0,
            0,
            0,
            other,
            0,
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert!(
            out.effects.len() >= want_prefixes.len(),
            "{id} 效果数不足: {:?}",
            out.effects
        );
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }
}

#[test]
fn pa12_batch16_resolvers_compile_and_execute() {
    let trigger_cases: &[(&str, &str, usize, &str)] = &[
        ("stddechong", include_str!("../../../../content/original/ported/sixiang/stddechong.skill"), 0, "ResolveStddechong"),
        ("ddd_yujin", include_str!("../../../../content/original/ported/ddd/yujin.skill"), 0, "ResolveDddYujin"),
        ("ddd_zhenji", include_str!("../../../../content/original/ported/ddd/zhenji.skill"), 0, "ResolveDddZhenji"),
        ("zhengjian", include_str!("../../../../content/original/ported/mobile/zhengjian.skill"), 0, "ResolveZhengjian"),
        ("zhanghe", include_str!("../../../../content/original/ported/shenhua/zhanghe.skill"), 0, "ResolveQiaobian"),
        ("twfeiyi", include_str!("../../../../content/original/ported/tw/twfeiyi.skill"), 0, "ResolveTwfeiyi"),
        ("zhengsu_bianzhen", include_str!("../../../../content/original/ported/shiji/zhengsu_bianzhen.skill"), 0, "ResolveZhengsu"),
        ("twcaoang", include_str!("../../../../content/original/ported/tw/twcaoang.skill"), 0, "ResolveTwcaoang"),
        ("twcaozhao", include_str!("../../../../content/original/ported/tw/twcaozhao.skill"), 0, "ResolveTwcaozhao"),
        ("twchengpu", include_str!("../../../../content/original/ported/tw/twchengpu.skill"), 0, "ResolveTwchengpu"),
        ("twchenxun", include_str!("../../../../content/original/ported/tw/twchenxun.skill"), 0, "ResolveTwchenxun"),
        ("twchenzhen", include_str!("../../../../content/original/ported/tw/twchenzhen.skill"), 0, "ResolveTwchenzhen"),
        ("twyizhu", include_str!("../../../../content/original/ported/tw/twyizhu.skill"), 0, "ResolveTwyizhu"),
        ("dcjiudun", include_str!("../../../../content/original/ported/xianding/dcjiudun.skill"), 0, "ResolveRuanji"),
        ("qiaojian", include_str!("../../../../content/original/ported/xianding/qiaojian.skill"), 0, "ResolveQiaojian"),
        ("zhupeilan", include_str!("../../../../content/original/ported/xianding/zhupeilan.skill"), 0, "ResolveZhupeilan"),
        ("mb_huangzu", include_str!("../../../../content/original/ported/bingshi/mb_huangzu.skill"), 0, "ResolveMbduanyang"),
        ("mb_luyusheng", include_str!("../../../../content/original/ported/bingshi/mb_luyusheng.skill"), 0, "ResolveMbrunwei"),
        ("pangxi", include_str!("../../../../content/original/ported/bingshi/pangxi.skill"), 0, "ResolveMbkuangxiang"),
        ("pot_caozhen", include_str!("../../../../content/original/ported/bingshi/pot_caozhen.skill"), 0, "ResolvePotsifeng"),
        ("pot_chendao", include_str!("../../../../content/original/ported/bingshi/pot_chendao.skill"), 0, "ResolvePotwanglie"),
        ("pot_chengpu", include_str!("../../../../content/original/ported/bingshi/pot_chengpu.skill"), 0, "ResolvePotchengpu"),
        ("pot_dengai_t", include_str!("../../../../content/original/ported/bingshi/pot_dengai.skill"), 0, "ResolvePottuntian"),
        ("pot_dengai_j", include_str!("../../../../content/original/ported/bingshi/pot_dengai.skill"), 2, "ResolvePotjixi"),
        ("pot_lougui", include_str!("../../../../content/original/ported/bingshi/pot_lougui.skill"), 0, "ResolvePotjiyu"),
        ("pot_sunchen", include_str!("../../../../content/original/ported/bingshi/pot_sunchen.skill"), 0, "ResolvePotnigu"),
        ("pot_zhangren", include_str!("../../../../content/original/ported/bingshi/pot_zhangren.skill"), 0, "ResolvePotfuan"),
        ("ddd_guanning", include_str!("../../../../content/original/ported/ddd/guanning.skill"), 0, "ResolveDddGuanning"),
        ("ddd_jianshuo", include_str!("../../../../content/original/ported/ddd/jianshuo.skill"), 0, "ResolveDddJianshuo"),
        ("shen_simayi", include_str!("../../../../content/original/ported/extra/shen_simayi.skill"), 0, "ResolveShenSimayi"),
    ];
    for (id, src, _trig_idx, want_prefix) in trigger_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 86, "identity-standard").expect("开局");
        let _ = std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&pkg);
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        let mut matched = false;
        let candidates: Vec<(u32, &str)> = pkg
            .trigger_index
            .iter()
            .map(|e| (e.skill_def_id, e.module.as_str()))
            .chain(
                pkg.active_index
                    .iter()
                    .map(|e| (e.skill_def_id, e.module.as_str())),
            )
            .collect();
        for (skill_def_id, mod_name) in candidates {
            let module = tx
                .state()
                .content
                .module(mod_name)
                .cloned()
                .unwrap_or_else(|| {
                    let m = pkg
                        .modules
                        .iter()
                        .find(|m| m.name == mod_name)
                        .expect("模块");
                    rule_vm::VerifiedModule::verify(m.clone()).expect("校验")
                });
            let out = engine_core::vm_host::run_module_with_event_cards(
                tx.state(),
                &module,
                skill_def_id,
                [owner.raw(), other.raw(), other.raw(), 1],
                1,
                &[1],
                &[2],
                0,
                0,
                0,
                other,
                0,
                10_000,
            )
            .expect("执行");
            assert!(out.paused.is_none(), "{id} 无挂起");
            if out
                .effects
                .iter()
                .any(|eff| format!("{eff:?}").starts_with(want_prefix))
            {
                matched = true;
                break;
            }
        }
        assert!(matched, "{id} 期望包含 {want_prefix}");
    }
}

#[test]
fn pa12_batch17_resolvers_view_as_and_cross_file_skills_compile_and_execute() {
    for (id, src) in [
        (
            "luanji",
            include_str!("../../../../content/original/ported/shenhua/luanji.skill"),
        ),
        (
            "jiushi1",
            include_str!("../../../../content/original/ported/yijiang/jiushi1.skill"),
        ),
        (
            "chixueqingfeng",
            include_str!("../../../../content/original/ported/sp/chixueqingfeng.skill"),
        ),
        (
            "rebiluan",
            include_str!("../../../../content/original/ported/sp/rebiluan.skill"),
        ),
        (
            "wangzun",
            include_str!("../../../../content/original/ported/sp/wangzun.skill"),
        ),
        (
            "decadexushen",
            include_str!("../../../../content/original/ported/sp2/decadexushen.skill"),
        ),
    ] {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译批⑰ .skill");
        assert!(!pkg.skill_index.is_empty(), "{id} 应含非空 skill_index");
    }
    let trigger_cases: &[(&str, &str, &str)] = &[
        (
            "dclinghui",
            include_str!("../../../../content/original/ported/xianding/dclinghui.skill"),
            "ResolveBailingyun",
        ),
        (
            "dcxiace",
            include_str!("../../../../content/original/ported/xianding/dcxiace.skill"),
            "ResolveBailingyun",
        ),
        (
            "dili_yuheng",
            include_str!("../../../../content/original/ported/extra/dili_yuheng.skill"),
            "ResolveDiliYuheng",
        ),
        (
            "xinlirang",
            include_str!("../../../../content/original/ported/shiji/xinlirang.skill"),
            "ResolveJieliangGain",
        ),
        (
            "jsrgchaozheng",
            include_str!("../../../../content/original/ported/jsrg/jsrgchaozheng.skill"),
            "ResolveJsrgchaozheng",
        ),
        (
            "starcuxia",
            include_str!("../../../../content/original/ported/sp2/starcuxia.skill"),
            "ResolveStarcuxiaWeiming",
        ),
        (
            "reanxu",
            include_str!("../../../../content/original/ported/mobile/reanxu.skill"),
            "ResolveReanxu",
        ),
        (
            "rebingyi",
            include_str!("../../../../content/original/ported/refresh/rebingyi.skill"),
            "ResolveRebingyi",
        ),
        (
            "rejizhi",
            include_str!("../../../../content/original/ported/refresh/rejizhi.skill"),
            "ResolveRejizhi",
        ),
        (
            "houfeng",
            include_str!("../../../../content/original/ported/shiji/houfeng.skill"),
            "ResolveHoufeng",
        ),
        (
            "dczhantao",
            include_str!("../../../../content/original/ported/xianding/dczhantao.skill"),
            "ResolveHuzun",
        ),
        (
            "dcxiaoyin",
            include_str!("../../../../content/original/ported/xianding/dcxiaoyin.skill"),
            "ResolveMalingli",
        ),
        (
            "dcjichun",
            include_str!("../../../../content/original/ported/xianding/dcjichun.skill"),
            "ResolveZhugemengxue",
        ),
        (
            "twyuyan",
            include_str!("../../../../content/original/ported/tw/twyuyan.skill"),
            "ResolveTwyuyan",
        ),
        (
            "twzhibian",
            include_str!("../../../../content/original/ported/tw/twzhibian.skill"),
            "ResolveTwzhibian",
        ),
    ];
    for (id, src, want_prefix) in trigger_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 87, "identity-standard").expect("开局");
        let _ = std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&pkg);
        let (owner, other) = (PlayerId::new(1), PlayerId::new(2));
        let matched = pkg
            .trigger_index
            .iter()
            .map(|e| (e.skill_def_id, e.module.as_str()))
            .chain(
                pkg.active_index
                    .iter()
                    .map(|e| (e.skill_def_id, e.module.as_str())),
            )
            .any(|(skill_def_id, mod_name)| {
                let m = pkg
                    .modules
                    .iter()
                    .find(|m| m.name == mod_name)
                    .expect("模块");
                let module = rule_vm::VerifiedModule::verify(m.clone()).expect("校验");
                let out = engine_core::vm_host::run_module_with_event_cards(
                    tx.state(),
                    &module,
                    skill_def_id,
                    [owner.raw(), other.raw(), other.raw(), 1],
                    1,
                    &[1],
                    &[2],
                    0,
                    0,
                    0,
                    other,
                    0,
                    10_000,
                )
                .expect("执行");
                assert!(out.paused.is_none(), "{id} 无挂起");
                out.effects
                    .iter()
                    .any(|eff| format!("{eff:?}").starts_with(want_prefix))
            });
        assert!(matched, "{id} 期望包含 {want_prefix}");
    }
}
