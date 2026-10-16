# 2026-10-16 维护：生产源码调试输出清理 + 机器守护（T2 口径补强）

核验日期：2026-10-16。源码：`noname-next`，规则 v290 / 存档 v77（本批**不改**规则、
存档与目录，无版本递增；只删打印语句与新增守护测试）。

## 1. 起因

T2（2026-10-10 事故恢复批）删除两个备份文件与 4 处**按环境变量开关**的 `eprintln`，
完成条件为「`rg 'env::var' crates/engine-native/src` 与 PT_DBG/XLY_DBG 均 0 命中」。
2026-10-16 复查该口径时发现多批内容开发（T12 swd/玩点论杀等）又引入了新残留：

| 位置 | 残留 | 性质 |
|---|---|---|
| `src/kernel.rs`（归汉 DyingSave） | `DBG guihan step2 entered`、`DBG guihan confirmed` | **无开关**调试打印（每次濒死经过都写 stderr） |
| `src/content/host.rs`（分舵 JiefenOwnerDiscard） | `DBG jiefen owner_discard entered`、`... no card`、`... gives card={c}` | 无开关注释级打印 |
| `src/content/host.rs`（破星 SwdPoxing） | `b41 dbg poxing step ...`、`b41 dbg hp ...`、`b41 dbg amount now ...` | 无开关打印（伤害帧直写路径） |
| `src/content/host.rs`（TB 过滤器） | `TB_DBG` 开关 + `DBG tb owner=... kind=... targets=... hit=...` | 环境变量开关打印 |
| `src/resolution.rs`（触发规划） | `TB_DBG` 开关 + `DBG plan entry def=... owner=... kind=...` | 环境变量开关打印（热路径，每次触发规划都查环境变量） |

即：T2 的完成条件在 2026-10-16 已**不再成立**（`TB_DBG` 命中 2 处），且存在无开关打印。

## 2. 处理

1. 删除上表全部 10 处打印语句与 2 处 `std::env::var` 开关（不含任何行为分支：打印均
   处于独立语句位置，删除不影响控制流）。`src/` 现对 `println!(` / `eprintln!(` /
   `env::var(` 三方零命中。
2. 新增机器守护 `tests/maintenance_guards.rs::production_sources_have_no_debug_output_or_env_switches`：
   递归扫描 `crates/engine-native/src/**/*.rs`，出现打印宏或环境变量开关即失败（提示诊断
   输出应放 `tests/`（`#[ignore]` 探针）、`examples/` 或日志/表现事件路径）。把 T2 的
   一次性 `rg` 口径变成回归守护，防止后续批次再引入。

## 3. 验证

| 口径 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **807 通过、0 失败、1 ignore**（上一批 806 + 本批守护 1；ignore 仍为既有诊断探针 `xly_state_growth_probe`）——[engine-native-tests.log](engine-native-tests.log) |
| Gate 1b（`RUSTFLAGS="-D warnings"`、独立 target 目录、`--all-targets`） | **exit 0**、0 warning——[gate1b-all-targets.log](gate1b-all-targets.log) |
| 规则/存档/golden | 不变（v290 / v77；本批不触语义与状态编码） |

全工作区未随本批重跑：改动仅有打印语句删除与新增测试文件，无 API、状态或存档变化；
上一批（同一源码基线，另有逐目标无懈语义变化）全工作区为 **5387/0/2**
（[日志](../2026-10-16-n06-chain-per-target-nullify/workspace-tests.log)）。

## 4. 边界

- 本批只覆盖 `crates/engine-native/src`；其他 crate 的 `src` 未纳入扫描域（如需泛化，
  另立批次并把扫描根参数化）。
- `examples/`、`tests/` 的打印仍允许（诊断探针用途），不在守护域内。
- 未删除 `unreachable!` 类断言（既有设计，另有 `deny(unreachable_patterns)` 约定）。
