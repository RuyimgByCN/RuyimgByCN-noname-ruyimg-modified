//! T8 后端对照原型（架构演进提案 C1）：以真实扩展 huanle-upstream-noname
//! 的技能「酒尾」（`huanle_jiuwei`，包数据 `mapping: host:phase_end_recover_or_draw`）
//! 为同一样本，实测规则 VM 字节码后端（候选 B）满足 C1 验收的能力项：
//! 数据驱动安装（不重编译）、选择挂起/恢复、内容锁（continuation 绑定
//! module hash）、超限（fuel）、故障回滚（半次效果不落账）、调用成本。
//!
//! 候选 A（现状：编译期 HostBehavior 枚举臂，kernel.rs）与候选 C（WASM 组件）
//! 不在本原型内实测：A 的执行路径在 engine-native 内（本例不依赖它，其
//! 「新技能必须改引擎重编译」以 host.rs/kernel.rs 源码位置为证）；C 的
//! wasmtime 不在离线依赖图内，仅做桌面评估（见 ADR 草稿）。
//!
//! **本例为 ADR 草稿输入（docs/adr/notes/draft-t8-backend-comparison.md），
//! 非定案；后端选择待 D2 决策。**
//!
//! 运行：`cargo run --offline -p content-compiler --example t8_backend_prototype`

use content_compiler::compile_source_named;
use rule_api::{CapabilitySet, EffectRequest, HostTrap, PlayerId, QueryRequest, QueryResponse};
use rule_vm::{Continuation, ExecResult, HostContext, RuleVm, VmError, VmHost, DEFAULT_FUEL};

/// 酒尾 T8 对照样本：语义逐句对齐 kernel.rs 的 PhaseEndRecoverOrDraw 臂
/// （hp < max_hp → 回 1 体力，否则摸 1）。
const JIUWEI_DSL: &str = "\
skill hnl_jiuwei_t8(990001) {
  trigger phase_end owner_is_target {
    if hp(self) < max_hp(self) {
      effect recover_hp(self, 1);
    } else {
      effect draw_cards(self, 1);
    }
  }
}
";

/// 选择挂起/恢复样本：同语义改为二选一（1=回血，2=摸牌）。
const JIUWEI_CHOICE_DSL: &str = "\
skill hnl_jiuwei_choice_t8(990002) {
  trigger phase_end owner_is_target optional {
    let option = choose(1, 2);
    if option == 1 {
      effect recover_hp(self, 1);
    }
    if option == 2 {
      effect draw_cards(self, 1);
    }
  }
}
";

/// 超限样本：先发一张效果再无限循环——证明故障回滚协议（已发效果不落账）。
const OVERRUN_DSL: &str = "\
skill hnl_overrun_t8(990003) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
    loop 1000000 {
      effect draw_cards(self, 1);
    }
  }
}
";

/// 改动版（回血量 2）：仅用于内容锁负例（logic_hash 必然不同）。
const JIUWEI_CHOICE_TAMPERED_DSL: &str = "\
skill hnl_jiuwei_choice_t8(990002) {
  trigger phase_end owner_is_target optional {
    let option = choose(1, 2);
    if option == 1 {
      effect recover_hp(self, 2);
    }
    if option == 2 {
      effect draw_cards(self, 1);
    }
  }
}
";

/// 原型宿主：可配置 hp/max_hp，按 C1 协议分「已排队」与「已落账」两层——
/// 只有 `Halted` 才把排队效果落账（引擎提交点）；trap 时排队层整批丢弃
/// （不留下半次效果）；Paused 时排队层保留给续跑路径。
struct HarnessHost {
    caps: CapabilitySet,
    ctx: HostContext,
    queued: Vec<EffectRequest>,
    committed: Vec<EffectRequest>,
    hp: i32,
    max_hp: i32,
}

impl HarnessHost {
    fn new(hp: i32, max_hp: i32) -> Self {
        Self {
            caps: CapabilitySet::standard_skill(),
            ctx: HostContext {
                skill_id: 42,
                self_id: 1,
                event_player: 1,
                event_source: 1,
                event_card: 0,
                event_amount: 0,
            },
            queued: Vec::new(),
            committed: Vec::new(),
            hp,
            max_hp,
        }
    }

    /// 引擎提交点：仅 Halted 落账；Err 回滚排队层。
    fn commit(&mut self, result: &Result<ExecResult, VmError>) {
        match result {
            Ok(ExecResult::Halted { .. }) => self.committed.append(&mut self.queued),
            Ok(ExecResult::Paused { .. }) => {} // 排队层保留至续跑后落账
            Err(_) => self.queued.clear(),       // 故障回滚：半次效果不落账
        }
    }
}

impl VmHost for HarnessHost {
    fn capabilities(&self) -> &CapabilitySet {
        &self.caps
    }
    fn context(&self) -> HostContext {
        self.ctx.clone()
    }
    fn emit_effect(&mut self, req: EffectRequest) -> Result<(), HostTrap> {
        self.queued.push(req);
        Ok(())
    }
    fn run_query(&mut self, req: QueryRequest) -> Result<QueryResponse, HostTrap> {
        match req {
            QueryRequest::Hp { .. } => Ok(QueryResponse::Int(self.hp)),
            QueryRequest::MaxHp { .. } => Ok(QueryResponse::Int(self.max_hp)),
            _ => Ok(QueryResponse::Int(0)),
        }
    }
}

/// PA06/B1：装载期校验一次（`VerifiedModule`），执行期不再重复 verify。
fn module_of(pkg: &content_schema::ContentPackage, skill: &str) -> rule_vm::VerifiedModule {
    let trig = pkg
        .trigger_index
        .iter()
        .find(|t| t.module.contains(skill))
        .unwrap_or_else(|| panic!("trigger for {skill} not found"));
    let module = pkg
        .modules
        .iter()
        .find(|m| m.name == trig.module)
        .unwrap_or_else(|| panic!("module {} not found", trig.module))
        .clone();
    rule_vm::VerifiedModule::verify(module).expect("装载期校验")
}

fn owner() -> PlayerId {
    PlayerId::new(1)
}

fn main() {
    println!("== T8 后端对照原型（C1，ADR 草稿输入，非定案）==");
    println!("样本：huanle-upstream-noname「酒尾」huanle_jiuwei");
    println!(
        "候选 A（现状）= 编译期 HostBehavior 枚举（host:phase_end_recover_or_draw，kernel.rs 直派臂；新技能须改引擎重编译）"
    );
    println!("候选 B（本例实测）= rule-vm 字节码（DSL→typecheck→codegen→verify→VM）");
    println!("候选 C（WASM 组件）= 未实测（离线依赖图无 wasmtime），桌面评估见 ADR 草稿\n");

    // ── 1. 数据驱动安装：DSL 即包数据，编译进 npack 形态的 ContentPackage ──
    let src = format!("{JIUWEI_DSL}{JIUWEI_CHOICE_DSL}{OVERRUN_DSL}");
    let pkg = compile_source_named("huanle-t8-prototype", &src)
        .expect("酒尾对照样本编译");
    assert_eq!(pkg.trigger_index.len(), 3, "三个触发技能全部入索引");
    let pkg_again = compile_source_named("huanle-t8-prototype", &src).expect("复编译");
    assert_eq!(
        pkg.manifest.logic_hash, pkg_again.manifest.logic_hash,
        "logic_hash 可复现（内容锁输入稳定）"
    );
    println!(
        "① 安装即数据：3 技能 DSL 编译为 npack 形态包（logic_hash 复现一致，rule_vm_abi={}）",
        pkg.manifest.rule_vm_abi_version
    );

    // 模块经 to_bytes/from_bytes 往返（npack 载体形态）后可执行
    let jiuwei = module_of(&pkg, "hnl_jiuwei_t8");
    let shipped = rule_vm::VerifiedModule::verify(
        rule_vm::BytecodeModule::from_bytes(&jiuwei.to_bytes()).expect("字节码往返"),
    )
    .expect("恢复载体装载期校验");
    assert_eq!(shipped.logic_hash, jiuwei.logic_hash, "往返后 hash 不变");
    println!("② 模块 to_bytes/from_bytes 往返一致（npack 载体可执行）");

    // ── 2. 语义一致：两分支分别对齐 kernel 臂 ──
    let mut vm = RuleVm::new();
    let mut host = HarnessHost::new(2, 4); // hp < max_hp
    let out = vm.execute(&shipped, &mut host, DEFAULT_FUEL);
    host.commit(&out);
    assert!(matches!(out, Ok(ExecResult::Halted { .. })), "{out:?}");
    assert_eq!(
        host.committed,
        vec![EffectRequest::RecoverHp {
            source: owner(), // DSL recover_hp(self, 1)：来源=拥有者
            player: owner(),
            amount: 1,
        }],
        "hp<max_hp 分支=回 1 体力（同 kernel 臂）"
    );

    let mut host = HarnessHost::new(4, 4); // hp == max_hp
    let out = vm.execute(&shipped, &mut host, DEFAULT_FUEL);
    host.commit(&out);
    assert!(matches!(out, Ok(ExecResult::Halted { .. })), "{out:?}");
    assert_eq!(
        host.committed,
        vec![EffectRequest::DrawCards {
            player: owner(),
            count: 1,
        }],
        "hp==max_hp 分支=摸 1（同 kernel 臂）"
    );
    println!("③ 语义一致：受伤时回 1 / 满血摸 1，两分支均与 kernel 臂等价");

    // ── 3. 选择挂起/恢复 + 内容锁 ──
    let choice = module_of(&pkg, "hnl_jiuwei_choice_t8");
    let mut host = HarnessHost::new(4, 4);
    let out = vm.execute(&choice, &mut host, DEFAULT_FUEL);
    let cont = match out.expect("choice 执行") {
        ExecResult::Paused { continuation, options } => {
            assert_eq!(options, vec![1, 2], "选项按声明序");
            continuation
        }
        other => panic!("期望 Paused，得 {other:?}"),
    };
    // 存档形态：continuation 可序列化（进入内容锁与存档）
    let saved = serde_json::to_string(&cont).expect("continuation 序列化");
    let restored: Continuation = serde_json::from_str(&saved).expect("continuation 反序列化");
    // 内容锁：continuation 绑定 module logic_hash，换模块即拒绝续跑
    let tampered = module_of(
        &compile_source_named("huanle-t8-tampered", JIUWEI_CHOICE_TAMPERED_DSL)
            .expect("改动版编译"),
        "hnl_jiuwei_choice_t8",
    );
    let locked = vm.resume(&tampered, &mut host, restored.clone(), 2);
    assert!(
        matches!(&locked, Err(VmError::InvalidBytecode(m)) if m.contains("mismatch")),
        "内容锁应拒绝跨模块续跑：{locked:?}"
    );
    let out = vm.resume(&choice, &mut host, restored, 2); // 选 2=摸牌
    host.commit(&out);
    assert!(matches!(out, Ok(ExecResult::Halted { .. })), "{out:?}");
    assert_eq!(
        host.committed,
        vec![EffectRequest::DrawCards {
            player: owner(),
            count: 1,
        }],
        "续跑后落账=所选分支效果"
    );
    println!("④ 挂起/恢复：Paused→continuation JSON 往返→resume 落账所选分支");
    println!("⑤ 内容锁：continuation 换模块（logic_hash 漂移）续跑被拒");

    // ── 4. 超限与故障回滚 ──
    let mut host = HarnessHost::new(4, 4);
    let out = vm.execute(&shipped, &mut host, 2); // fuel=2 必然超限
    assert_eq!(out, Err(VmError::OutOfFuel), "fuel 超限确定性 trap");
    host.commit(&out);
    assert!(host.committed.is_empty(), "trap 后无落账");

    let overrun = module_of(&pkg, "hnl_overrun_t8");
    let mut host = HarnessHost::new(4, 4);
    let out = vm.execute(&overrun, &mut host, DEFAULT_FUEL);
    assert_eq!(out, Err(VmError::OutOfFuel), "死循环被 fuel 截断");
    assert!(!host.queued.is_empty(), "trap 前已有排队效果（半次效果的原料）");
    host.commit(&out);
    assert!(host.committed.is_empty(), "提交点协议：trap 整批丢弃，不落半次账");
    println!("⑥ 超限/回滚：fuel 截断死循环；trap 时排队效果整批丢弃（不留下半次效果）");

    // ── 5. 调用成本（同机初步测量，非验收口径）──
    const N: u32 = 20_000;
    let total = std::time::Instant::now();
    for _ in 0..N {
        let mut host = HarnessHost::new(4, 4);
        let r = vm.execute(&shipped, &mut host, DEFAULT_FUEL);
        assert!(matches!(r, Ok(ExecResult::Halted { .. })));
    }
    let total_ns = total.elapsed().as_nanos();
    println!(
        "⑦ 成本（本机初步测量，{} 构建）：酒尾触发一次（装载期已校验，不含 verify）≈ {} ns 均值（N={N}；解释器量级参考值，非验收口径）",
        if cfg!(debug_assertions) { "debug" } else { "release" },
        total_ns / N as u128,
    );

    println!("\n结论（草稿）：候选 B 在 C1 必须项上全部可实测满足；成本为解释器量级、");
    println!("候选 A 为编译直派（未单独测量）；候选 C 隔离最强但离线不可测。定案待 D2。");
}
