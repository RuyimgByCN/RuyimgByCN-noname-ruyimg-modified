# 2026-10-11 T6②：名称符号表总量上限与拒装策略（ADR 0779）

任务来源：[进度主稿 T6②](../../../architecture/native-progress.md)；开工 HEAD `fc992b98`（领取锁提交）。决策记录：[ADR 0779](../../../adr/0779-native-symbol-table-limit.md)。

## 结论

- **策略：总量上限 + 生产入口拒装（fail-fast），不回收。** 上限
  `content::names::SYMBOL_TABLE_LIMIT = 1_000_000`（唯一名称 + 唯一名单合计；已知全部内容总量 <1e5，约两个数量级余量）。三个生产入口 `bridge_pack`/`bridge_generals`/`bridge_pile` 解析前检查，超限返回新错误 `BridgeError::SymbolTableLimit { limit }`。
- **回收不取的理由**：表项是 `'static str`，该类型贯穿目录、快照编码与字节存档；改 `Rc`/owned 是全代码面改动并牵动存档语义与版本门禁，而唯一名称数本身有界。详见 ADR"为什么选上限拒装"。
- **收口点完整性**：`rg` 确认 `names::intern` 调用面只有装载器与 `content/host.rs::host_skill_def` 一处（入参来自装载器已 intern 的键，有界）；标准目录等用编译期字面量，不经运行时表。

## 改动内容

| 文件 | 改动 |
|---|---|
| `engine-native/src/content/names.rs` | `SYMBOL_TABLE_LIMIT` 常量 + `remaining_capacity()`；模块文档补上限策略 |
| `content-native-loader/src/lib.rs` | `BridgeError::SymbolTableLimit` 变体 + Display；`ensure_symbol_capacity()` 门接进三个生产入口；crate 头文档补一句 |
| `content-native-loader/tests/symbol_table_limit.rs` | 新增（独立测试目标=独立进程）：上限内装载通过 → 填充恰达 1e6 → 装载返回 `SymbolTableLimit` → `remaining==0` 且重复 intern 不增长。独占进程的原因：符号表是进程级全局，填充会污染同进程其他测试 |
| `docs/adr/0779-native-symbol-table-limit.md` + [INDEX](../../../adr/INDEX.md) + ABI-COORDINATION ADR 领取表 | 决策、理由、边界（单包一次 bridge 可逼近上限，受包文件大小界定，登记）、验收 |

## 已知边界（ADR 登记）

单包自身可在一次 bridge 内 intern 逼近上限（防单包爆量需预扫描去重计数，未做）——包来源可信度由安装审批/签名链承担。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p content-native-loader --test symbol_table_limit` | **1/1**（填充 1e6 实测 1.9s） |
| `cargo test --offline -p content-native-loader --test loader_acceptance` | 3/3（含既有 50 轮重复装载零增长界验收） |
| `cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast` | **66/0/0**（65+1） |
| `cargo test --offline -p engine-native --no-fail-fast` | **527/0/1** |
| `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib`（独立 target 目录，用完删） | **exit 0** |
| `cargo test --offline -p engine-api`（ABI-COORDINATION 加行后账面守护） | 4/4 |

## 边界

- 无行为变化：正常内容远低于上限，既有包装载路径结果不变（三包 66/0/0 与 engine-native 527/0/1）。
- ③（正式客户端/服务端接入）仍待领取，与本项无依赖。
