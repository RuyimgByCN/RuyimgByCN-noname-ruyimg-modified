# 2026-10-28 B8：Gate 1b 转绿（content-compiler ABI 755 死代码 + 被遮蔽的 28 个测试二进制 lint）

范围：剩余台账阻断 **B8 / P2**（CI Gate 1b `RUSTFLAGS=-D warnings cargo check -p engine-native --all-targets` 红）。本批**不改 ABI 窗口语义、不升规则／存档／ABI 版本、不改任何产品代码行为**：改动只有删除一个从未被引用的编译器常量及其导入，以及 28 个 engine-native 测试文件里的 35 条 lint（未用导入／变量／`mut`、`&str` 上的空操作 `.clone()`、孤儿辅助函数与宏）。开始 [start-time.txt](start-time.txt)，结束 [end-time.txt](end-time.txt)，源码基准 HEAD `95f51867`（[source-head.txt](source-head.txt)），规则 v325／存档 v105／ABI 753，工作树开工时干净。

## 1. 复现与归因

1. HEAD 上 Gate 1b 复现：exit 101，恰为扫描登记的 2 条 content-compiler 错误（[gate1b-before-head.log](gate1b-before-head.log)）。
2. **755 常量归因**：`ADDITIVE_RULE_VM_ABI_VERSION_755` 与其在 `effect_set_draw_phase_from_bottom.rs:4` 的导入均由 `da8c7ff0`（B2 批③，2026-10-26）一次引入；该提交在同文件新增的两个发射臂只用 756（`EffectChooseGiveOrRecastOneHand`）和 757（`EffectResolveOlJieDanshou`），755 从那时起无任何引用（`rg ABI_VERSION_755` 全工作区仅此两处）。B6（`72aa29cf`）只把 754 改名为 758 并更新引用，未动 755——扫描把两条错误归给 B6 不准确，但不影响判定。ABI-COORDINATION.md 没有 755 级编译器原语登记；754–757 含 tag 656 的包拒绝窗由 `engine-core/src/content/registry/new.rs:108` 的字面区间实现，不依赖该常量。**结论：死代码，删除**（台账 B8 完成条件"属死代码就删除"）。
3. **被遮蔽的下游错误**：content-compiler 通过后，cargo 才继续编译 engine-native 的 `--all-targets`，又暴露 **28 个测试二进制共 35 条** `-D warnings` 错误（[gate1b-before-masked-errors.log](gate1b-before-masked-errors.log)，`--keep-going` 完整枚举）。它们此前从未进入 CI 编译，各批"本批文件零警告"的说法只覆盖各自新文件。CI 注释写明 Gate 1b 的用意是抓**丢失 `#[test]` 的测试函数**（只报 dead_code），因此 4 条 dead_code 逐项审查：

| 位置 | 项 | 审查结论 |
|---|---|---|
| `nc08_upstream_ganglie.rs:88` | `fn place_by_prefix(e,&str,u8)` | 带参辅助，非测试签名；文件 3 个 `#[test]` 齐全；孤儿 |
| `nc08_upstream_xiuluo.rs:50` | `fn answer(e,Vec<ChoiceOption>)` | 同上 |
| `nc08_upstream_lingxue.rs:47` | `fn drive_to_quiescence(e)` | 同上 |
| `nc08_registered_defect_guards.rs:127` | `macro_rules! g14_guard` | 生成 `#[ignore]` 守卫；`bf2c1a60`（10-21）刻意删掉全部 12 个调用（G14 复判为非缺陷：Judge begin 即回合开始），宏定义遗留；其余 5 个守卫宏调用数 5/10/8/7/2 正常 |

   编译器已完成"零属性扫描"：无 `#[test]` 且未被调用的顶层函数只有上述 3 个；无 `duplicated attribute`（即无 `#[test]` 叠加）。3 条未用变量亦逐项核对无漏断言：`liutouge:49` 的 `other_loss` 是常量占位 `0i64`（"对他人"分支由独立的 `other` 断言 `-1`）；`pa12_batch3:222` 首个 `base` 被 235 行同名遮蔽（两分支均有断言）；`pa12_batch4_1:435` 的 `p` 在 `eprintln!` 调试闭包内。

## 2. 改动（30 文件，+29／−100）

- `crates/content-compiler/src/abi_versions.rs`：删 `ADDITIVE_RULE_VM_ABI_VERSION_755` 声明；`codegen/effect_set_draw_phase_from_bottom.rs`：删其导入。
- `crates/engine-native/tests/`（28 文件）：去 14 处未用导入（`CommandOutcome`×10、`Phase`×2、`advance_to_phase`、`BTreeSet`）；7 处 `c.spec.prompt.clone()`→`c.spec.prompt`（`&'static str`，类型不变）；6 处 `let mut`→`let`；3 处未用变量（`let (self_gain, other_loss)`→`let self_gain`、删首个 `base`、闭包 `p`→`_`）；删 3 个孤儿函数与 1 个孤儿宏（含各自 doc 注释）。全部修改由脚本按**逐行原文校验后替换**执行，任何不匹配即中止。

## 3. 验证（2026-10-28 23:30 – 10-29 00:05，HEAD `95f51867` + 本批工作树）

| 命令 | 结果 | 日志 |
|---|---|---|
| Gate 1b（独立 `CARGO_TARGET_DIR`，CI 原命令） | **exit 0，0 error／0 warning** | [gate1b-after.log](gate1b-after.log) |
| **阳性对照**：向 `nc08_upstream_xiuluo.rs` 追加 `fn gate1b_positive_control_unused(){}` 再跑 Gate 1b → exit 101 报该函数 never used；恢复后 exit 0 | 门禁确实在检查测试目标 | [gate1b-positive-control.log](gate1b-positive-control.log) |
| `cargo test --offline -p content-compiler --no-fail-fast` | **541/0/0**（基线 541/0/0） | [content-compiler.log](content-compiler.log) |
| `cargo test --offline -p engine-native --no-fail-fast` | **1624/0/73**，334 二进制（基线 1624/0/73、334） | [engine-native.log](engine-native.log) |
| 行数门禁单测 + `check-source-lines.py` | OK；**8657** 文件 ≤2000 行 | [source-line-tests.log](source-line-tests.log)、[source-lines.log](source-lines.log) |

阳性对照第一次失败是对照自身的错误：注入函数名以 `__` 开头，而 Rust `dead_code` 对下划线前缀名静默；`cargo -v` 证实 cargo 已判 Dirty 并重跑 rustc。改普通名后对照成立。本批未跑 wasm／loader／game-server／前端（无产品代码改动；content-compiler 与 engine-native 全量已覆盖本批全部改动文件）。

## 4. 兼容性与未覆盖

- 规则 v325、存档 v105、全局原生 ABI 753、Worker／WS 协议、ABI 拒绝窗（754–757 含 656）全部不变；golden 无变化。
- 测试计数不变（1624/0/73）：删除的是辅助函数／宏，不是测试。
- 本批之后，各批收口须以 Gate 1b 实跑 exit 0 为准，不得再以"既有警告、本批零新增"收口。
