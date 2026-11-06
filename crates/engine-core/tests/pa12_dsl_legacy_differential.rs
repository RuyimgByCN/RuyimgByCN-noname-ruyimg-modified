//! PA12④ 差分（2026-10-26）：与 engine-native 原型
//! `tests/pa12_dsl_native_prototype.rs` 同一 `olyinji` 模块（同一 .skill 编译
//! 产物）经**旧线** vm_host 执行——挂起选项、两个续行分支的效果种类与落账
//! 逐面一致：`choose(1)` → `RecoverHp{amount:1}`（hp+1）、`choose(2)` →
//! `ChangeMaxHp{delta:1}`（max+1）。触发/滤镜面由原生整局证明；此处对照
//! 模块本体的执行语义（两条宿主线对同一字节码的等价性）。

use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_api::PlayerId;

const OLYINJI: &str = include_str!("../../../content/original/ported/sp/olyinji.skill");

fn module() -> rule_vm::VerifiedModule {
    let pkg =
        content_compiler::compile_source_named("sp-olyinji-prototype", OLYINJI).expect("编译");
    let name = pkg.trigger_index[0].module.clone();
    let raw = pkg
        .modules
        .iter()
        .find(|m| m.name == name)
        .unwrap_or_else(|| panic!("模块 {name} 不在包内"))
        .clone();
    rule_vm::VerifiedModule::verify(raw).expect("模块校验")
}

/// 旧线装载：包同时入 registry（正式路径），模块经装载点持有的已校验形态执行。
fn loaded_state(seed: u64) -> (Transaction, rule_vm::VerifiedModule) {
    let pkg =
        content_compiler::compile_source_named("sp-olyinji-prototype", OLYINJI).expect("编译");
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").expect("开局");
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&pkg)
        .expect("旧线 registry 装载同一包");
    let name = pkg.trigger_index[0].module.clone();
    let module = tx
        .state()
        .content
        .module(&name)
        .cloned()
        .expect("registry 应持有该模块（装载期已校验）");
    (tx, module)
}

#[test]
fn pa12_legacy_pause_options_match_native() {
    let (tx, module) = loaded_state(77);
    let owner = PlayerId::new(1);
    let out = engine_core::vm_host::run_module(
        tx.state(),
        &module,
        0,
        [owner.raw(), owner.raw(), owner.raw(), 0],
        0,
        10_000,
    )
    .expect("执行");
    let p = out.paused.expect("choose(1,2) 应挂起");
    assert_eq!(p.options, vec![1, 2], "与原生 open_choice 的 Tag 选项一致");
}

#[test]
fn pa12_legacy_branch_effects_match_native() {
    for (token, want_hp, want_max, want_effect) in [
        (1u32, 1i32, 0u32, "RecoverHp"),
        (2u32, 0i32, 1u32, "ChangeMaxHp"),
    ] {
        let (mut tx, module) = loaded_state(77);
        let owner = PlayerId::new(1);
        let ctx = [owner.raw(), owner.raw(), owner.raw(), 0];
        tx.state_mut().players.get_mut(owner).unwrap().hp = 3; // hp<max：回复分支可见
        let hp0 = tx.state().players.get(owner).unwrap().hp;
        let max0 = tx.state().players.get(owner).unwrap().max_hp;
        let out = engine_core::vm_host::run_module(tx.state(), &module, 0, ctx, 0, 10_000)
            .expect("执行");
        let p = out.paused.expect("挂起");
        let done = engine_core::vm_host::resume_module(
            tx.state(),
            &module,
            0,
            ctx,
            p.frames,
            p.pc,
            p.fuel_remaining,
            token,
        )
        .expect("续行");
        let kind = format!("{:?}", done.effects[0]);
        assert!(
            kind.starts_with(want_effect),
            "分支 {token} 效果应为 {want_effect}，实际 {kind}"
        );
        assert_eq!(done.effects.len(), 1, "单效果");
        run_effects(&mut tx, done.effects).expect("落账");
        assert_eq!(
            tx.state().players.get(owner).unwrap().hp,
            hp0 + want_hp,
            "分支 {token} hp 落账与原生一致"
        );
        assert_eq!(
            tx.state().players.get(owner).unwrap().max_hp,
            max0 + want_max,
            "分支 {token} max_hp 落账与原生一致"
        );
    }
}

/// PA12 批①差分：清励（`loop 5 { if hand<max_hp { if hand<5 { draw 1 } } }`）在旧线
/// 同样一次摸 5——两线 VM 宿主都在模块 Halt 后落账，模块内再查询看到执行前手牌，
/// 条件恒真。与原生 `pa12_batch1_owner_alive_triggers_on_others_end_phase` 的 5 张一致；
/// 这是 ported DSL 的保真度问题（上游：摸至体力上限、至多 5），登记于迁移计划 §2 H。
#[test]
fn pa12_legacy_clanqingli_requery_is_stale_draws_five() {
    const CLANQINGLI: &str =
        include_str!("../../../content/original/ported/clan/clanqingli.skill");
    let pkg = content_compiler::compile_source_named("clan-qingli", CLANQINGLI).expect("编译");
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 77, "identity-standard").expect("开局");
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&pkg)
        .expect("装载");
    let name = pkg.trigger_index[0].module.clone();
    let module = tx.state().content.module(&name).cloned().expect("模块");
    let owner = PlayerId::new(1);
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 9; // 手牌清空：0 < max 且 < 5
    for card in tx.state().hand_cards(owner) {
        let zone = tx.state().cards.get(card).expect("牌存在").zone;
        tx.state_mut()
            .zones
            .get_mut(zone)
            .expect("手牌区")
            .cards
            .retain(|c| *c != card);
    }
    assert_eq!(tx.state().hand_count(owner), 0);
    let out = engine_core::vm_host::run_module(
        tx.state(),
        &module,
        0,
        [owner.raw(), owner.raw(), owner.raw(), 0],
        0,
        10_000,
    )
    .expect("执行");
    assert!(out.paused.is_none());
    assert_eq!(out.effects.len(), 5, "再查询陈旧：loop 5 全部命中，摸 5");
    assert!(out
        .effects
        .iter()
        .all(|e| format!("{e:?}").starts_with("DrawCards")));
}

/// PA12 批②差分：五脉（`phase_begin owner_is_target optional` 摸 1）与蹙国
/// （`sha_dodged owner_is_source` 摸 1）的模块本体在旧线各产出一次 DrawCards——
/// 与原生整局（`pa12_batch2_phase_events.rs`）落账一致；触发时机（旧线 TurnBegan /
/// ShaDodged 域事件 vs 原生 Judge begin 帧 / ShaDodged 帧）由两侧各自的事件面保证。
#[test]
fn pa12_legacy_batch2_modules_draw_one() {
    for (id, src) in [
        ("key-wumai", include_str!("../../../content/original/ported/key/liyingxia_wumai.skill")),
        ("newjiang-cuguo", include_str!("../../../content/original/ported/newjiang/cuguo.skill")),
    ] {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 77, "identity-standard").expect("开局");
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&pkg)
            .expect("装载");
        let name = pkg.trigger_index[0].module.clone();
        let module = tx.state().content.module(&name).cloned().expect("模块");
        let owner = PlayerId::new(1);
        let out = engine_core::vm_host::run_module(
            tx.state(),
            &module,
            0,
            [owner.raw(), owner.raw(), owner.raw(), 0],
            0,
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), 1, "{id} 单效果");
        assert!(format!("{:?}", out.effects[0]).starts_with("DrawCards"), "{id} 摸牌");
    }
}

/// PA12 批③差分：抚（普通锦囊后摸 1）与军略（受伤/造成伤害 +标记=伤害值）模块本体
/// 在旧线的效果面——与原生整局（`pa12_batch3_filter_predicates.rs`）一致；谓词
/// （owner_source_ordinary_trick / owner_is_source_or_target）由各自事件面保证。
#[test]
fn pa12_legacy_batch3_modules() {
    for (id, src, want_prefix, amount_arg) in [
        ("collab-natufu", include_str!("../../../content/original/ported/collab/natufu.skill"), "DrawCards", 1),
        ("extra-junlve", include_str!("../../../content/original/ported/extra/nzry_junlve.skill"), "AddMark", 1),
    ] {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 77, "identity-standard").expect("开局");
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&pkg)
            .expect("装载");
        let name = pkg.trigger_index[0].module.clone();
        let module = tx.state().content.module(&name).cloned().expect("模块");
        let owner = PlayerId::new(1);
        let out = engine_core::vm_host::run_module(
            tx.state(),
            &module,
            0,
            [owner.raw(), owner.raw(), owner.raw(), 0],
            amount_arg,
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), 1, "{id} 单效果");
        assert!(format!("{:?}", out.effects[0]).starts_with(want_prefix), "{id} {want_prefix}");
    }
}

/// PA12 批⑥差分（2026-11-05）：`dcmouli`（phase_any_end）、`dcgangjian`（phase_any_end）、
/// `peersheng`（cards_drawn）、`haoxue`（cards_drawn）、`dcjiexing`（hp_recovered/hp_lost）、
/// `dcgangu`（hp_lost）、`xinjuxiang`（dying_after）与 `mingzhe`（HOST_EVENT_CARD_COUNT）
/// 在旧线 vm_host 的模块效果面——与原生整局（`pa12_batch6_events_and_cards.rs`）一致。
#[test]
fn pa12_legacy_batch6_modules_match_native() {
    for (id, src, want_prefixes) in [
        (
            "huicui-dcmouli",
            include_str!("../../../content/original/ported/huicui/dcmouli.skill"),
            &["RecoverHp", "ChangeMaxHp"][..],
        ),
        (
            "huicui-dcgangjian",
            include_str!("../../../content/original/ported/huicui/dcgangjian.skill"),
            &["DrawCards"][..],
        ),
        (
            "offline-peersheng",
            include_str!("../../../content/original/ported/offline/peersheng.skill"),
            &["DrawCards"][..],
        ),
        (
            "collab-haoxue",
            include_str!("../../../content/original/ported/collab/haoxue.skill"),
            &["RecoverHp"][..],
        ),
        (
            "xianding-dcjiexing",
            include_str!("../../../content/original/ported/xianding/dcjiexing.skill"),
            &["DrawCards"][..],
        ),
        (
            "huicui-dcgangu",
            include_str!("../../../content/original/ported/huicui/dcgangu.skill"),
            &["DrawCards", "DealDamage"][..],
        ),
        (
            "shiji-xinjuxiang",
            include_str!("../../../content/original/ported/shiji/xinjuxiang.skill"),
            &["RemoveSkill", "DealDamage"][..],
        ),
    ] {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 77, "identity-standard").expect("开局");
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&pkg)
            .expect("装载");
        let name = pkg.trigger_index[0].module.clone();
        let module = tx.state().content.module(&name).cloned().expect("模块");
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        let out = engine_core::vm_host::run_module(
            tx.state(),
            &module,
            pkg.trigger_index[0].skill_def_id,
            [owner.raw(), other.raw(), other.raw(), 0],
            1,
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }

    // 明哲（HOST_EVENT_CARD_COUNT + InPlayPhase）：出牌阶段外有事件牌→摸 1；无事件牌或出牌阶段内→0 效果
    let pkg = content_compiler::compile_source_named(
        "sp-mingzhe",
        include_str!("../../../content/original/ported/sp/mingzhe.skill"),
    )
    .expect("编译明哲");
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 77, "identity-standard").expect("开局");
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&pkg)
        .expect("装载");
    let module = tx
        .state()
        .content
        .module(&pkg.trigger_index[0].module)
        .cloned()
        .expect("模块");
    let owner = PlayerId::new(1);
    tx.state_mut().turn.phase = engine_core::state::Phase::End;
    let empty_cards = engine_core::vm_host::run_module_with_event_cards(
        tx.state(),
        &module,
        pkg.trigger_index[0].skill_def_id,
        [owner.raw(), owner.raw(), owner.raw(), 0],
        0,
        &[],
        &[],
        0,
        0,
        0,
        PlayerId::NONE,
        0,
        10_000,
    )
    .expect("空事件牌执行");
    assert!(empty_cards.effects.is_empty(), "event_card_count=0 时不摸牌");

    let with_card = engine_core::vm_host::run_module_with_event_cards(
        tx.state(),
        &module,
        pkg.trigger_index[0].skill_def_id,
        [owner.raw(), owner.raw(), owner.raw(), 1],
        1,
        &[1],
        &[],
        0,
        0,
        0,
        PlayerId::NONE,
        0,
        10_000,
    )
    .expect("非出牌阶段带事件牌执行");
    assert_eq!(with_card.effects.len(), 1, "event_card_count=1 且非出牌阶段摸 1");
    assert!(format!("{:?}", with_card.effects[0]).starts_with("DrawCards"));

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let in_play = engine_core::vm_host::run_module_with_event_cards(
        tx.state(),
        &module,
        pkg.trigger_index[0].skill_def_id,
        [owner.raw(), owner.raw(), owner.raw(), 1],
        1,
        &[1],
        &[],
        0,
        0,
        0,
        PlayerId::NONE,
        0,
        10_000,
    )
    .expect("出牌阶段带事件牌执行");
    assert!(in_play.effects.is_empty(), "出牌阶段内不摸牌");
}

/// PA12 批⑦ 旧线差分基线：验证 `sp/mingshi`、`key/shiroha_guying`、`sp/xiemu2`、
/// `xianding/dczhangji`、`onlyOL/olbuyi`、`offline/liangfan2` 在旧线 VM 宿主上的效果请求序与
/// 原生线 `pa12_batch7_codec_named_and_once` 一致。
#[test]
fn pa12_legacy_batch7_codec_named_and_once_skills_match_native() {
    for (id, src, trig_idx, target_cnt, want_prefixes) in [
        (
            "sp-mingshi",
            include_str!("../../../content/original/ported/sp/mingshi.skill"),
            0usize,
            1u32,
            &["DiscardHandEquipCount", "ModifyCurrentDamage"][..],
        ),
        (
            "key-shiroha-guying-target",
            include_str!("../../../content/original/ported/key/shiroha_guying.skill"),
            0,
            1,
            &["ModifyCurrentDamage"][..],
        ),
        (
            "key-shiroha-guying-source",
            include_str!("../../../content/original/ported/key/shiroha_guying.skill"),
            1,
            1,
            &["ModifyCurrentDamage"][..],
        ),
        (
            "sp-xiemu2",
            include_str!("../../../content/original/ported/sp/xiemu2.skill"),
            0,
            1,
            &["DrawCards"][..],
        ),
        (
            "xianding-dczhangji",
            include_str!("../../../content/original/ported/xianding/dczhangji.skill"),
            0,
            2,
            &["DrawCards"][..],
        ),
        (
            "onlyOL-olbuyi",
            include_str!("../../../content/original/ported/onlyOL/olbuyi.skill"),
            0,
            1,
            &["RecoverHp"][..],
        ),
        (
            "offline-liangfan2",
            include_str!("../../../content/original/ported/offline/liangfan2.skill"),
            0,
            1,
            &["DrawCards"][..],
        ),
    ] {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 77, "identity-standard").expect("开局");
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&pkg)
            .expect("装载");
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        // mingshi 需要 hp(event_source) > hp(self)
        tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
        tx.state_mut().players.get_mut(other).unwrap().hp = 4;
        let targets: Vec<u32> = (1..=target_cnt).collect();

        let entry = &pkg.trigger_index[trig_idx];
        let module = tx.state().content.module(&entry.module).cloned().expect("模块");
        let out = engine_core::vm_host::run_module_with_event_cards(
            tx.state(),
            &module,
            entry.skill_def_id,
            [owner.raw(), other.raw(), other.raw(), 1],
            1,
            &[1],
            &targets,
            0,
            0,
            0,
            other,
            0,
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }
}

/// PA12 批⑧ 旧线差分基线：验证 `sp/wuji`、`yijiang/zhichi`、`yijiang/jigong`、
/// `collab/dcqiusuo`、`mobile/friendmanjuan`、`shiji/spshiji`、`mobile/ly_piliche`、
/// `collab/dclonggong`、`yijiang/oldrenxin`、`sp/naman`、`collab/olhaizhu` 在旧线 VM 宿主上的
/// 效果请求序与原生线 `pa12_batch8_deterministic_effects` 一致。
#[test]
fn pa12_legacy_batch8_deterministic_effects_skills_match_native() {
    for (id, src, trig_idx, want_prefixes) in [
        (
            "sp-wuji",
            include_str!("../../../content/original/ported/sp/wuji.skill"),
            0usize,
            &[
                "RemoveSkillByDef",
                "ChangeMaxHp",
                "RecoverHp",
                "RemoveSkill",
            ][..],
        ),
        (
            "yijiang-zhichi",
            include_str!("../../../content/original/ported/yijiang/zhichi.skill"),
            0,
            &["AddSkillUntilTurnEnd"][..],
        ),
        (
            "yijiang-jigong",
            include_str!("../../../content/original/ported/yijiang/jigong.skill"),
            0,
            &["DrawCards", "AddSkillUntilTurnEnd"][..],
        ),
        (
            "collab-dcqiusuo",
            include_str!("../../../content/original/ported/collab/dcqiusuo.skill"),
            0,
            &["GainCardByKind"][..],
        ),
        (
            "mobile-friendmanjuan",
            include_str!("../../../content/original/ported/mobile/friendmanjuan.skill"),
            0,
            &["AddMark", "GainDiscardCardByKind"][..],
        ),
        (
            "shiji-spshiji",
            include_str!("../../../content/original/ported/shiji/spshiji.skill"),
            0,
            &["RevealHand"][..],
        ),
        (
            "mobile-ly-piliche",
            include_str!("../../../content/original/ported/mobile/ly_piliche.skill"),
            0,
            &["DiscardAllTargetEquipment"][..],
        ),
        (
            "collab-dclonggong",
            include_str!("../../../content/original/ported/collab/dclonggong.skill"),
            0,
            &["PreventDamage", "GainEquipmentFromDrawPile"][..],
        ),
        (
            "yijiang-oldrenxin",
            include_str!("../../../content/original/ported/yijiang/oldrenxin.skill"),
            0,
            &["TurnOver", "GiveAllHand", "RecoverHp"][..],
        ),
        (
            "sp-naman",
            include_str!("../../../content/original/ported/sp/naman.skill"),
            0,
            &["GainCardIfDiscard"][..],
        ),
        (
            "collab-olhaizhu",
            include_str!("../../../content/original/ported/collab/olhaizhu.skill"),
            0,
            &["GainEventCards"][..],
        ),
    ] {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 78, "identity-standard").expect("开局");
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&pkg)
            .expect("装载");
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);

        let entry = &pkg.trigger_index[trig_idx];
        let module = tx.state().content.module(&entry.module).cloned().expect("模块");
        let out = engine_core::vm_host::run_module_with_event_cards(
            tx.state(),
            &module,
            entry.skill_def_id,
            [owner.raw(), other.raw(), other.raw(), 1],
            1,
            &[1],
            &[1],
            0,
            0,
            0,
            other,
            0,
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }
}

/// 批⑨差分对表：10 个真实 `.skill` 模块在旧线 `engine_core::vm_host` 产出的 `EffectRequest` 序列
/// 与原生 `pa12_batch9_events_and_turn_effects` 完全一致。
#[test]
fn pa12_batch9_legacy_vm_effects_match_native_expectation() {
    for (id, src, trig_idx, want_prefixes) in [
        (
            "sixiang-stdfenyin",
            include_str!("../../../content/original/ported/sixiang/stdfenyin.skill"),
            0usize,
            &["IncreaseCurrentDrawCount", "AddSkill"][..],
        ),
        (
            "old-oldzishou",
            include_str!("../../../content/original/ported/old/oldzishou.skill"),
            0,
            &["IncreaseCurrentDrawCount", "SkipNextPlayPhase"][..],
        ),
        (
            "shiji-spyinju2",
            include_str!("../../../content/original/ported/shiji/spyinju2.skill"),
            0,
            &["SkipNextPlayPhase", "SkipNextDiscardPhase"][..],
        ),
        (
            "mobile-zhengjing2",
            include_str!("../../../content/original/ported/mobile/zhengjing2.skill"),
            0,
            &[
                "GainWholeSkillPile",
                "SkipNextJudgePhase",
                "RemoveSkillByDef",
            ][..],
        ),
        (
            "offline-zj-fuli",
            include_str!("../../../content/original/ported/offline/zj_fuli.skill"),
            0,
            &[
                "StorageSet",
                "RecoverHp",
                "DrawCards",
                "QueueExtraPhases",
            ][..],
        ),
        (
            "mobile-xinduodao",
            include_str!("../../../content/original/ported/mobile/xinduodao.skill"),
            0,
            &["GainTargetEquipment"][..],
        ),
        (
            "sp-oldhuxiao",
            include_str!("../../../content/original/ported/sp/oldhuxiao.skill"),
            0,
            &["RefundCurrentShaUse"][..],
        ),
        (
            "xianding-renzheng",
            include_str!("../../../content/original/ported/xianding/renzheng.skill"),
            0,
            &["DrawCards"][..],
        ),
        (
            "sp-pxyanmen",
            include_str!("../../../content/original/ported/sp/pxyanmen.skill"),
            0,
            &["GainCardByKind", "GainCardByKind"][..],
        ),
        (
            "offline-zj-hanci",
            include_str!("../../../content/original/ported/offline/zj_hanci.skill"),
            0,
            &["DrawCards", "DrawCards"][..],
        ),
    ] {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 79, "identity-standard").expect("开局");
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&pkg)
            .expect("装载");
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 1;

        let entry = &pkg.trigger_index[trig_idx];
        let module = tx
            .state()
            .content
            .module(&entry.module)
            .cloned()
            .expect("模块");
        let out = engine_core::vm_host::run_module_with_event_cards(
            tx.state(),
            &module,
            entry.skill_def_id,
            [owner.raw(), other.raw(), other.raw(), 1],
            1,
            &[1],
            &[1],
            0,
            0,
            0,
            other,
            0,
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }
}

/// PA12 批⑩ 旧线 `vm_host::run_module_with_event_cards` 差分锚点：
/// 验证 `huilei` / `qibie` / `duanchang` / `oldniepan` / `jysuozhou` / `dangxian` /
/// `redangxian` / `oljuejue` / `liegong` 等真实 ported `.skill` 在旧线 VM 宿主上
/// 产出的 `EffectRequest` 序列与原生 `rvm_ops` 批⑩接入的变体完全同构。
#[test]
fn pa12_batch10_combat_and_state_effects_legacy_differential() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "huilei",
            include_str!("../../../content/original/ported/yijiang/huilei.skill"),
            &["DiscardAllHandAndEquip"],
        ),
        (
            "qibie",
            include_str!("../../../content/original/ported/sp2/qibie.skill"),
            &["DiscardAllHandAndEquip", "RecoverHp", "DrawCards"],
        ),
        (
            "duanchang",
            include_str!("../../../content/original/ported/shenhua/duanchang.skill"),
            &["RemoveAllSkills"],
        ),
        (
            "oldniepan",
            include_str!("../../../content/original/ported/shenhua/oldniepan.skill"),
            &[
                "RemoveSkill",
                "DiscardAllHandEquipJudge",
                "RestoreCharacterState",
                "DrawCards",
                "RecoverHp",
            ],
        ),
        (
            "jysuozhou",
            include_str!("../../../content/original/ported/offline/jysuozhou.skill"),
            &["DrawLinkedPlayers"],
        ),
        (
            "dangxian",
            include_str!("../../../content/original/ported/yijiang/dangxian.skill"),
            &["QueueOpeningExtraPlayPhase"],
        ),
        (
            "redangxian",
            include_str!("../../../content/original/ported/mobile/redangxian.skill"),
            &["GainDiscardCardByKind", "QueueOpeningExtraPlayPhase"],
        ),
        (
            "oljuejue",
            include_str!("../../../content/original/ported/collab/oljuejue.skill"),
            &["IncreaseCurrentShaBaseDamage"],
        ),
        (
            "liegong",
            include_str!("../../../content/original/ported/shenhua/liegong.skill"),
            &["MakeCurrentShaTargetUnrespondable"],
        ),
    ];
    for (id, src, want_prefixes) in cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 80, "identity-standard").expect("开局");
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&pkg)
            .expect("装载");
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
        let entry = &pkg.trigger_index[0];
        let module = tx
            .state()
            .content
            .module(&entry.module)
            .cloned()
            .expect("模块");
        let out = engine_core::vm_host::run_module_with_event_cards(
            tx.state(),
            &module,
            entry.skill_def_id,
            [owner.raw(), other.raw(), other.raw(), 1],
            1,
            &[1],
            &[1],
            0,
            0,
            0,
            other,
            0,
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }
}

/// PA12 批⑪ 旧线 `vm_host::run_module_with_event_cards` 差分锚点：
/// 验证 `dcjianxiong` / `natulie` / `dcchenyong` / `wangxi` / `nzry_cunmu` / `duchi` /
/// `danlao` / `zhuzhou` / `nscangjian` / `stdyibing` 等真实 ported `.skill` 在旧线 VM 宿主上
/// 产出的 `EffectRequest` 序列与原生 `rvm_ops` 批⑪接入的变体完全同构。
#[test]
fn pa12_batch11_legacy_vm_effects_match_native_expectation() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "dcjianxiong",
            include_str!("../../../content/original/ported/collab/dcjianxiong.skill"),
            &["MoveCards", "DrawCards", "AddMark"],
        ),
        (
            "natulie",
            include_str!("../../../content/original/ported/collab/natulie.skill"),
            &["AdjustHandTo"],
        ),
        (
            "dcchenyong",
            include_str!("../../../content/original/ported/xianding/dcchenyong.skill"),
            &["DrawByPlayPhaseCardCategoryCount"],
        ),
        (
            "wangxi",
            include_str!("../../../content/original/ported/refresh/wangxi.skill"),
            &["DrawEachBySeat"],
        ),
        (
            "nzry_cunmu",
            include_str!("../../../content/original/ported/shenhua/nzry_cunmu.skill"),
            &["SetDrawPhaseFromBottom"],
        ),
        (
            "duchi",
            include_str!("../../../content/original/ported/newjiang/duchi.skill"),
            &["DrawCardsFromBottom", "RevealHand"],
        ),
        (
            "danlao",
            include_str!("../../../content/original/ported/sp/danlao.skill"),
            &["DrawCards", "CancelIncomingCardUse"],
        ),
        (
            "zhuzhou",
            include_str!("../../../content/original/ported/collab/zhuzhou.skill"),
            &["TakeOneCard"],
        ),
        (
            "nscangjian",
            include_str!("../../../content/original/ported/diy/nscangjian.skill"),
            &["TakeOneCard"],
        ),
        (
            "stdyibing",
            include_str!("../../../content/original/ported/sixiang/stdyibing.skill"),
            &["TakeOneHandCard"],
        ),
        (
            "wxdlqianqian",
            include_str!("../../../content/original/ported/offline/wxdlqianqian.skill"),
            &["DisableSkillByDefUntilTurnEnd"],
        ),
        (
            "rewansha",
            include_str!("../../../content/original/ported/refresh/rewansha.skill"),
            &["SetNonLockedSkillSuppression"],
        ),
    ];
    for (id, src, want_prefixes) in cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 81, "identity-standard").expect("开局");
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&pkg)
            .expect("装载");
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        let pz = tx
            .state()
            .zones
            .find(engine_core::state::ZoneKind::Processing, PlayerId::NONE)
            .expect("processing zone");
        tx.state_mut()
            .cards
            .get_mut(engine_api::CardId::new(1))
            .unwrap()
            .zone = pz;
        let entry = &pkg.trigger_index[0];
        let module = tx
            .state()
            .content
            .module(&entry.module)
            .cloned()
            .expect("模块");
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
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }
}

/// PA12 批⑫差分：验证旧线 `engine_core::vm_host` 对批⑫真实 ported `.skill`
/// （含 `selected_cards` / `selected_targets` 预选主动技与触发技）产出的
/// `EffectRequest` 序列与原生批⑫新增原语（tags 84..=108）逐项一致。
#[test]
fn pa12_batch12_legacy_vm_effects_match_native_expectation() {
    let active_cases: &[(&str, &str, &[&str])] = &[
        (
            "chongxin",
            include_str!("../../../content/original/ported/yingbian/chongxin.skill"),
            &["RecastCards"],
        ),
        (
            "rezhiheng",
            include_str!("../../../content/original/ported/refresh/rezhiheng.skill"),
            &["DiscardSelectedDrawAllHandBonus"],
        ),
        (
            "olweiyang",
            include_str!("../../../content/original/ported/collab/olweiyang.skill"),
            &["DiscardCards", "RecoverSelectedTargets"],
        ),
        (
            "shizhan",
            include_str!("../../../content/original/ported/refresh/shizhan.skill"),
            &["UseVirtualCardOn"],
        ),
        (
            "jylianhuan",
            include_str!("../../../content/original/ported/offline/jylianhuan.skill"),
            &["DealDamage", "UseVirtualCardOnSelectedTargets"],
        ),
        (
            "songci",
            include_str!("../../../content/original/ported/sp/songci.skill"),
            &["DrawSelectedTargets"],
        ),
        (
            "ziyuan",
            include_str!("../../../content/original/ported/sp/ziyuan.skill"),
            &["GiveSelectedThenRecoverTarget"],
        ),
        (
            "refenxun",
            include_str!("../../../content/original/ported/sp/refenxun.skill"),
            &["AddBoundSkill"],
        ),
        (
            "ol_wuqian",
            include_str!("../../../content/original/ported/extra/ol_wuqian.skill"),
            &["RemoveMark", "AddSkillUntilTurnEnd", "RecordActiveTarget"],
        ),
        (
            "eu_xiaoguo",
            include_str!("../../../content/original/ported/offline/eu_xiaoguo.skill"),
            &["DiscardCards", "DealDamageBanActiveIfDying"],
        ),
    ];
    for (id, src, want_prefixes) in active_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 82, "identity-standard").expect("开局");
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&pkg)
            .expect("装载");
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        let entry = &pkg.active_index[0];
        let module = tx
            .state()
            .content
            .module(&entry.module)
            .cloned()
            .expect("模块");
        let out = engine_core::vm_host::run_module_with_selected_inputs(
            tx.state(),
            &module,
            entry.skill_def_id,
            [owner.raw(), other.raw(), other.raw(), 1],
            1,
            &[1, 2],
            &[other.raw()],
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }

    let trigger_cases: &[(&str, &str, &[&str])] = &[
        (
            "quanji",
            include_str!("../../../content/original/ported/yijiang/quanji.skill"),
            &["DrawCards", "StoreSelectedInSkillPile"],
        ),
        (
            "dcxingwu",
            include_str!("../../../content/original/ported/xianding/dcxingwu.skill"),
            &["PlaceHandCardToExpansion"],
        ),
        (
            "zuici",
            include_str!("../../../content/original/ported/shiji/zuici.skill"),
            &["DisableEquipSlots", "RecoverHp"],
        ),
        (
            "kengo_guidui",
            include_str!("../../../content/original/ported/key/kengo_guidui.skill"),
            &[
                "EnableEquipSlot",
                "EnableEquipSlot",
                "EnableEquipSlot",
                "EnableEquipSlot",
                "EnableEquipSlot",
            ],
        ),
        (
            "jsrgjiaohao",
            include_str!("../../../content/original/ported/jsrg/jsrgjiaohao.skill"),
            &["GainShadowCards"],
        ),
        (
            "pxsizhou",
            include_str!("../../../content/original/ported/sp/pxsizhou.skill"),
            &["GainTargetHandCard"],
        ),
        (
            "qiaomeng",
            include_str!("../../../content/original/ported/refresh/qiaomeng.skill"),
            &["DiscardTargetCardGainMount"],
        ),
        (
            "scls_qinzheng",
            include_str!("../../../content/original/ported/offline/scls_qinzheng.skill"),
            &["DrawForLifetimeCardActionDivisors"],
        ),
        (
            "pszhonghun",
            include_str!("../../../content/original/ported/offline/pszhonghun.skill"),
            &["RevealTopGainIfColor"],
        ),
        (
            "jiezi",
            include_str!("../../../content/original/ported/shenhua/jiezi.skill"),
            &["DrawCards"],
        ),
        (
            "tiandu",
            include_str!("../../../content/standard/tiandu.skill"),
            &["GainEventCards"],
        ),
        (
            "botu",
            include_str!("../../../content/original/ported/refresh/botu.skill"),
            &["QueueExtraTurn"],
        ),
    ];
    for (id, src, want_prefixes) in trigger_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 82, "identity-standard").expect("开局");
        let _ = std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&pkg);
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        let pz = tx
            .state()
            .zones
            .find(engine_core::state::ZoneKind::DiscardPile, PlayerId::NONE)
            .expect("discard zone");
        tx.state_mut()
            .cards
            .get_mut(engine_api::CardId::new(1))
            .unwrap()
            .zone = pz;
        let entry = &pkg.trigger_index[0];
        let module = tx
            .state()
            .content
            .module(&entry.module)
            .cloned()
            .unwrap_or_else(|| rule_vm::VerifiedModule::verify(pkg.modules[0].clone()).expect("模块"));
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
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }
}

/// PA12 批⑬ + UP37 差分：验证旧线 `engine_core::vm_host` 对批⑬真实 ported `.skill`
/// 与 UP37 技能（`dcdichou`、`yingba`、`dcmingjie`、`dczigu`、`zzhenggong`）产出的
/// `EffectRequest` 序列与原生批⑬新增原语（tags 109..=128）逐项一致。
#[test]
fn pa12_batch13_legacy_vm_effects_match_native_expectation() {
    let active_cases: &[(&str, &str, &[&str])] = &[
        (
            "yingba",
            include_str!("../../../content/original/ported/shiji/yingba.skill"),
            &["DealDamage", "ChangeMaxHp", "AddSkill", "AddMark", "ChangeMaxHp"],
        ),
        (
            "dczigu",
            include_str!("../../../content/original/ported/huicui/dczigu.skill"),
            &["ChooseTargetDraw"],
        ),
        (
            "youyi",
            include_str!("../../../content/original/ported/shiji/youyi.skill"),
            &["ChooseOtherRecover"],
        ),
    ];
    for (id, src, want_prefixes) in active_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 83, "identity-standard").expect("开局");
        let _ = std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&pkg);
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        let entry = &pkg.active_index[0];
        let module = tx
            .state()
            .content
            .module(&entry.module)
            .cloned()
            .unwrap_or_else(|| {
                rule_vm::VerifiedModule::verify(pkg.modules[0].clone()).expect("模块")
            });
        let out = engine_core::vm_host::run_module_with_selected_inputs(
            tx.state(),
            &module,
            entry.skill_def_id,
            [owner.raw(), other.raw(), other.raw(), 1],
            1,
            &[],
            &[other.raw()],
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }

    let trigger_cases: &[(&str, &str, usize, &[&str])] = &[
        (
            "dcdichou-t0",
            include_str!("../../../content/original/ported/xianding/dcdichou.skill"),
            0,
            &["AddMark"],
        ),
        (
            "dcdichou-t1",
            include_str!("../../../content/original/ported/xianding/dcdichou.skill"),
            1,
            &["ResolveDcdichou"],
        ),
        (
            "dcdichou-t2",
            include_str!("../../../content/original/ported/xianding/dcdichou.skill"),
            2,
            &["DrawCards", "RemoveMark"],
        ),
        (
            "dcdichou-t3",
            include_str!("../../../content/original/ported/xianding/dcdichou.skill"),
            3,
            &["ResolveDcdichou"],
        ),
        (
            "dcmingjie-t0",
            include_str!("../../../content/original/ported/xianding/dcmingjie.skill"),
            0,
            &["DrawCards", "ChooseTargetDraw"],
        ),
        (
            "dcmingjie-t1",
            include_str!("../../../content/original/ported/xianding/dcmingjie.skill"),
            1,
            &["ChangeMaxHp"],
        ),
        (
            "zzhenggong",
            include_str!("../../../content/original/ported/yijiang/zzhenggong.skill"),
            0,
            &["TakeOneCard"],
        ),
        (
            "take_one_scoped_card_probe",
            "skill take_scoped_probe(91000123) { trigger damage owner_is_target { effect take_one_card(event_source, self, hand_equip_judge); } }",
            0,
            &["TakeOneScopedCard"],
        ),
        (
            "anyong",
            include_str!("../../../content/original/ported/huicui/anyong.skill"),
            0,
            &["ChooseOtherDamage"],
        ),
        (
            "xinkuanggu",
            include_str!("../../../content/original/ported/shenhua/xinkuanggu.skill"),
            0,
            &["ChooseDrawOrRecover"],
        ),
        (
            "kuanshi",
            include_str!("../../../content/original/ported/sp/kuanshi.skill"),
            0,
            &["ChooseOtherAddMark"],
        ),
        (
            "hfjieying",
            include_str!("../../../content/original/ported/sp2/hfjieying.skill"),
            0,
            &["ResolveHfjieying"],
        ),
        (
            "spmingshi",
            include_str!("../../../content/original/ported/shiji/spmingshi.skill"),
            0,
            &["ChooseDiscardOneHandOrEquip"],
        ),
        (
            "xiaoxi",
            include_str!("../../../content/original/ported/sp/xiaoxi.skill"),
            0,
            &["ChooseUseVirtualSha"],
        ),
        (
            "sijian",
            include_str!("../../../content/original/ported/sp/sijian.skill"),
            0,
            &["ChooseOtherDiscardTargetCard"],
        ),
        (
            "mengjin",
            include_str!("../../../content/original/ported/shenhua/mengjin.skill"),
            0,
            &["ChooseDiscardTargetCard"],
        ),
        (
            "fengjie",
            include_str!("../../../content/original/ported/shiji/fengjie.skill"),
            0,
            &["ChooseOtherGiveOneHandEquip"],
        ),
        (
            "dcwushen",
            include_str!("../../../content/original/ported/xianding/dcwushen.skill"),
            0,
            &["RecordCurrentShaAllTargets"],
        ),
        (
            "clanxsyingxiang",
            include_str!("../../../content/original/ported/clan/clanxsyingxiang.skill"),
            0,
            &["DrawAllEventTargets"],
        ),
        (
            "clanzhongliu",
            include_str!("../../../content/original/ported/clan/clanzhongliu.skill"),
            0,
            &["RefreshSkill"],
        ),
        (
            "rebaobian",
            include_str!("../../../content/original/ported/sp/rebaobian.skill"),
            0,
            &["AddFirstMissingSkill"],
        ),
        (
            "zhanshen",
            include_str!("../../../content/standard/zhanshen.skill"),
            0,
            &[
                "DiscardEquipmentSlot",
                "ChangeMaxHp",
                "AddSkill",
                "AddSkill",
                "RemoveSkill",
            ],
        ),
        (
            "olddanshou",
            include_str!("../../../content/original/ported/yijiang/olddanshou.skill"),
            0,
            &["DrawCards", "FinishCurrentPhaseAbortResolution"],
        ),
    ];
    for (id, src, trig_idx, want_prefixes) in trigger_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 83, "identity-standard").expect("开局");
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
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }
}

/// PA12 批⑭差分：验证旧线 `engine_core::vm_host` 对批⑭ 25 个真实 ported `.skill`
/// 产出的 `EffectRequest` 序列与原生批⑭新增原语（tags 129..=153）逐项一致。
#[test]
fn pa12_batch14_legacy_vm_effects_match_native_expectation() {
    let active_cases: &[(&str, &str, &[&str])] = &[
        (
            "xianzhen",
            include_str!("../../../content/original/ported/yijiang/xianzhen.skill"),
            &["ChooseCompareSelectedHandCard"],
        ),
        (
            "choose_one_from_skill_pile_discard_probe",
            "skill pile_discard_probe(91400141) { active play_phase { effect choose_one_from_skill_pile_discard(self, self_skill, 3615325273); } }",
            &["ChooseOneFromSkillPileDiscard"],
        ),
    ];
    for (id, src, want_prefixes) in active_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 84, "identity-standard").expect("开局");
        let _ = std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&pkg);
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        let entry = &pkg.active_index[0];
        let module = tx
            .state()
            .content
            .module(&entry.module)
            .cloned()
            .unwrap_or_else(|| {
                rule_vm::VerifiedModule::verify(pkg.modules[0].clone()).expect("模块")
            });
        let out = engine_core::vm_host::run_module_with_selected_inputs(
            tx.state(),
            &module,
            entry.skill_def_id,
            [owner.raw(), other.raw(), other.raw(), 1],
            1,
            &[1],
            &[other.raw()],
            10_000,
        )
        .expect("执行");
        assert!(out.paused.is_none(), "{id} 无挂起");
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }

    let trigger_cases: &[(&str, &str, usize, &[&str])] = &[
        (
            "shushen",
            include_str!("../../../content/original/ported/sp/shushen.skill"),
            0,
            &["ChooseOtherDrawOrRecover"],
        ),
        (
            "mieji",
            include_str!("../../../content/original/ported/yijiang/mieji.skill"),
            0,
            &["ChooseAddOrRemoveCurrentCardTarget"],
        ),
        (
            "xinjuece",
            include_str!("../../../content/original/ported/yijiang/xinjuece.skill"),
            0,
            &["ChooseTargetsDamage"],
        ),
        (
            "dangmo",
            include_str!("../../../content/original/ported/shiji/dangmo.skill"),
            0,
            &["ChooseAddCurrentShaTarget"],
        ),
        (
            "xinwangxi",
            include_str!("../../../content/original/ported/refresh/xinwangxi.skill"),
            0,
            &["DrawThenGiveOne"],
        ),
        (
            "qiaoshui",
            include_str!("../../../content/original/ported/yijiang/qiaoshui.skill"),
            0,
            &["ChooseHandCompare"],
        ),
        (
            "lizhan",
            include_str!("../../../content/original/ported/sp/lizhan.skill"),
            0,
            &["ChooseOptionalTargetsDraw"],
        ),
        (
            "stdduanbi",
            include_str!("../../../content/original/ported/sixiang/stdduanbi.skill"),
            0,
            &["DiscardFromHand", "ChooseTargetsDraw"],
        ),
        (
            "reguanxing",
            include_str!("../../../content/original/ported/refresh/reguanxing.skill"),
            0,
            &["GuanxingTop"],
        ),
        (
            "dcjieying",
            include_str!("../../../content/original/ported/xianding/dcjieying.skill"),
            0,
            &["ChooseLinkOther", "DrawLinkedPlayers"],
        ),
        (
            "fangzhu",
            include_str!("../../../content/original/ported/shenhua/fangzhu.skill"),
            0,
            &["ChooseOtherDrawOwnerLostHpTurnOver"],
        ),
        (
            "duanbing",
            include_str!("../../../content/original/ported/sp/duanbing.skill"),
            0,
            &["ChooseAddCurrentShaTargetWithinDistance"],
        ),
        (
            "dcsbqijue",
            include_str!("../../../content/original/ported/xianding/dcsbqijue.skill"),
            0,
            &["ChooseTargetDraw", "ChooseRecoverTarget"],
        ),
        (
            "jieming",
            include_str!("../../../content/original/ported/shenhua/jieming.skill"),
            0,
            &["ChooseTargetDrawToMaxHpCap"],
        ),
        (
            "olfeibian",
            include_str!("../../../content/original/ported/sp/olfeibian.skill"),
            0,
            &["ChooseDiscardHandRange"],
        ),
        (
            "kuiwei-t1",
            include_str!("../../../content/original/ported/sp/kuiwei.skill"),
            1,
            &["ChooseDiscardHandEquipThenRemoveSelfSkill"],
        ),
        (
            "tunchu",
            include_str!("../../../content/original/ported/mobile/tunchu.skill"),
            0,
            &["DrawCards", "ChooseHandCardsToSkillPile"],
        ),
        (
            "renxin",
            include_str!("../../../content/original/ported/yijiang/renxin.skill"),
            0,
            &["ChooseDiscardEquipmentPreventCurrentDamage", "TurnOver"],
        ),
        (
            "dcshouze",
            include_str!("../../../content/original/ported/xianding/dcshouze.skill"),
            0,
            &["RemoveMark", "ChooseGainDiscardCardByKind", "DealDamage"],
        ),
        (
            "zhengnan",
            include_str!("../../../content/original/ported/sp/zhengnan.skill"),
            0,
            &["DrawCards", "ChooseAddMissingSkill"],
        ),
        (
            "shebian",
            include_str!("../../../content/original/ported/refresh/shebian.skill"),
            0,
            &["ChooseMoveFieldCardTo"],
        ),
        (
            "tieji",
            include_str!("../../../content/standard/tieji.skill"),
            0,
            &["JudgeMakeCurrentShaTargetUnrespondableOnRed"],
        ),
        (
            "dcwuhun-t1",
            include_str!("../../../content/original/ported/xianding/dcwuhun.skill"),
            1,
            &["ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan"],
        ),
    ];
    for (id, src, trig_idx, want_prefixes) in trigger_cases {
        let pkg = content_compiler::compile_source_named(id, src).expect("编译");
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 84, "identity-standard").expect("开局");
        let _ = std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&pkg);
        let owner = PlayerId::new(1);
        let other = PlayerId::new(2);
        tx.state_mut().players.get_mut(other).unwrap().hp = 1;
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
        assert_eq!(out.effects.len(), want_prefixes.len(), "{id} 效果数");
        for (eff, want) in out.effects.iter().zip(want_prefixes.iter()) {
            assert!(
                format!("{eff:?}").starts_with(want),
                "{id} 期望 {want}，实际 {eff:?}"
            );
        }
    }
}



