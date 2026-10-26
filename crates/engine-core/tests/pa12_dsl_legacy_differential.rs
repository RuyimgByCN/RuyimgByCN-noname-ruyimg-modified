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
