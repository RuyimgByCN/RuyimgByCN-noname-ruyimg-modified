# O6 集成测试分组编译 + dev 调试信息（2026-10-29）

## 任务来源与环境

用户 2026-10-29 问：Rust 部分能否像 openidea（33 个小 crate、每 crate 0–4 个测试二进制）那样拆分，避免每次修改编译很久。先测量定位（下节），再经用户确认实施「测试分组 + `debug = "line-tables-only"`」；crate 拆分未做（见「未做」）。

环境：提交基线 HEAD `95f51867` + 未提交工作树（含其他会话在途改动，本批只动下列文件）；规则 **v328**、存档 **v105**（本批不变）；rustc/cargo 1.95.0；Apple M3 Max 14 核 / 36GB；Darwin 27.2。每项为单次测量。

## 定位（改动前）

| 项 | 实测 |
|---|---|
| 改 lib 一行（新增私有函数）→ 只编 lib | engine-native **2s**；engine-core **5s** |
| 改 lib → 重建全部测试（干净 target，`cargo test -p <crate> --no-run`） | engine-native **17.9s**（335 个测试二进制）；engine-core **24.6s**（176 个） |
| 冷构建全部测试（干净 target） | engine-native **44.5s**；engine-core **52.0s** |
| 主工作树 target（42GB；deps 23.1 万文件、增量缓存 17GB）同一 touch → engine-native 全部测试 | **70s / 77s**（两次）；同操作干净 target **19s** |

结论：lib 本身不是瓶颈，耗时在「每个 `tests/*.rs` 单独成 crate、单独链接」与 target 膨胀。O2（2026-10-14）已把 engine-core 1,078→176 个链接目标；engine-native 未做过。原始记录：[timing-before.txt](timing-before.txt)。

## 改动

1. **测试分组**（engine-native、engine-core）：`Cargo.toml` 设 `autotests = false`，`tests/groups/<组>.rs` 以 `#[path = "../<文件>.rs"] mod <文件>;` 登记顶层测试文件，每组一个 `[[test]]`。测试文件本身零改动（第 2 项除外），`include_str!`/嵌套 `mod`/`super::` 解析不变。
   - engine-native 335 文件 → 10 组：`kernel`、`g1`、`g1_shahai`、`swd`、`nc08`、`nc08_upstream_a_k`、`nc08_upstream_l_z`、`nc_rounds`、`l_rounds`、`misc`。
   - engine-core 176 文件 → 10 组：`original_sp`、`original_shenhua`、`original_yijiang`、`original_offline`、`original_mobile`、`original_key`、`original_ol`、`original_misc`、`phase`、`misc`。
   - 两 crate 各加守卫 `test_registry`：顶层每个 `.rs` 恰登记一次、登记项存在、每个分组文件都有 `[[test]]`。
2. **自启子进程测试**：`t8_rvm_bridge.rs`、`t12_g5_relays.rs`、`g1_huanle/huanle.rs` 以 `--exact <名>` 重跑自身；分组后全名带模块前缀，改为本文件内 `libtest_name()`（`module_path!` 去首段 crate 名 + 函数名），旧/新布局均正确。t12 的建局种子仍用原 `test_name`，不变。
3. **workspace `[profile.dev] debug = "line-tables-only"`**：panic 回溯保留文件与行号；需看局部变量时临时 `CARGO_PROFILE_DEV_DEBUG=true`。
4. **命令消费者**：`scripts/check-native-wasm-cross-target.py` 的 `--test nc08_cross_target_real` → `--test nc08 nc08_cross_target_real::`；进度主稿验证表 `--test replay_scenarios` → `--test kernel replay_scenarios::`。其余脚本/CI 无按文件名的 `--test` 调用。

## 守恒核验

| 项 | 改动前 | 改动后 |
|---|---|---|
| engine-native 集成测试全名 | 1,699（335 个二进制） | **1,699**（10 组），排序列表 sha256 同为 `3c8c52bd8a690797…`——[test-list-engine-native.txt](test-list-engine-native.txt) |
| engine-core 集成测试全名 | 3,758（176 个二进制） | **3,758**（10 组），sha256 同为 `8f7ae3a0049b23bf…`——[test-list-engine-core.txt](test-list-engine-core.txt) |
| engine-core 警告 | lib 107 + 测试 402 | 同，逐类计数一致——[warning-kinds-engine-core.txt](warning-kinds-engine-core.txt) |
| engine-native 警告 | 0 | 0；Gate 1b（独立 target，`RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --all-targets`）exit 0、23s——[gate1b.log](gate1b.log) |

全名口径：`<原文件名>::<函数名>`；改动前按 `cargo test -- --list` 的 `Running tests/<文件>.rs` 行拼接前缀。子串过滤行为不变。

全套（改动后，主工作树）：`cargo test --offline -p engine-native --no-fail-fast` **1644/0/73** exit 0、30s——[suite-engine-native.txt](suite-engine-native.txt)，含 3 个自启子进程测试（父进程读回子进程回写存档，子进程确实执行）与登记守卫；`cargo test --offline -p engine-core --no-fail-fast` **3847/0/1** exit 0、52s——[suite-engine-core.txt](suite-engine-core.txt)。行数门禁 `python3 -B scripts/check-source-lines.py` 通过（8,683 文件）；`python3 -B scripts/preflight.py` **20/20** 通过（含全部 docs 本地链接与脚本单测）。改动前全套本批未跑。

## 测量（改动后）

干净 rsync 副本、独立 target；「完整调试信息」= `CARGO_PROFILE_DEV_DEBUG=true`。原始记录：[timing-after.txt](timing-after.txt)。

| 场景 | 改动前 | 分组 + 完整调试信息 | 分组 + line-tables-only |
|---|---|---|---|
| engine-native 改 lib → 全部测试 | 17.9s | **3.3s / 3.4s** | **3.2s / 3.3s** |
| engine-core 改 lib → 全部测试 | 24.6s | **5.3s / 5.2s** | **5.9s / 5.7s** |
| engine-native 冷构建全部测试 | 44.5s | 31.8s | **30.2s** |
| engine-core 冷构建全部测试 | 52.0s | 36.5s | **30.3s** |
| 两 crate 测试构建后 target | — | 8.2GB | **6.6GB** |
| 改一个测试文件 → 其所在组（swd） | 0.4s（单文件二进制） | — | 2.1s |
| 主工作树 touch → 全部测试 | engine-native 70s / 77s | — | engine-native **9.1s**；engine-core **9.9s**（改动前主树未测 engine-core） |

## 结论

- 主要收益来自分组：改 lib 后重建全部测试 engine-native 约 5 倍、engine-core 约 4 倍提速；冷构建减少约 30–40%。
- `line-tables-only` 在分组后对增量几乎无影响（差值在单次测量噪声内），收益在冷构建（engine-core 36.5→30.3s）与磁盘（8.2→6.6GB）；分组前 344 个目标时曾测得 38.6→18.3s。代价：调试器看不到局部变量。
- 代价：改单个测试文件需重编整组（2.1s，旧布局 0.4s）。
- 主工作树 target 膨胀未处理（`cargo clean` 待用户决定）；主树数字受其影响，不与干净 target 混比。

## 未做与边界

- **crate 拆分（openidea 式）未立项**：lib 增量 2–5s 不是瓶颈；engine-native 核心模块→`content` 756 处引用（`HostMachine` 进 machine 分派、kernel 直接按技能名判定），`content`→核心约 1,000 处；engine-core `effect`↔`command` 366/930 处。拆分需先做依赖反转并核对存档/哈希，收益有限。
- 历史证据与 ADR 中的 `--test <文件名>` 命令保留原样（日期语境）。新写法：`cargo test -p <crate> --test <组> <文件名>::`，例如 `cargo test -p engine-native --test swd swd_moyan::`。
- 新增测试文件须在 `tests/groups/` 选一组登记 `#[path]` + `mod`；漏登记时 `cargo test -p <crate> --test test_registry` 报出文件名。
