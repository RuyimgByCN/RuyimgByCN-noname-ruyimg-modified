# ADR 0779：名称符号表总量上限与拒装策略（不回收）

日期：2026-10-11　|　状态：已采纳
关联：[ADR 0777](0777-native-c0-native-loader.md)、[ADR 0778](0778-native-c0-pack-data-keys.md)、[主稿 T6②](../architecture/native-progress.md)、`crates/engine-native-content/src/content/names.rs`

## 背景

ADR 0777/0778 把目录与扩展的名称（技能键、牌定义名、武将名、名单切片）收敛到
进程级符号表（`content::names`）：同名只分配一次 `'static str`，重复
装载/开局/启停循环不增长（`loader_acceptance_bounded_names_and_clear_errors`
以 50 轮重装+建局+启停验收）。表项经 `Box::leak` 永不回收，总量上界 =
「唯一名称数 + 唯一名单数」。

遗留缺口（主稿 T6②）：上界成立但**无强制上限**。长期运行的宿主进程
（正式服务端，主稿 T6③/T13 的目标形态）反复安装含新唯一名称的任意来源包
时，表单调增长无停止条件；损坏或恶意包流可无限撑大内存。

## 决策

**总量上限 + 生产入口拒装（fail-fast），不做回收：**

1. 上限常量 `content::names::SYMBOL_TABLE_LIMIT = 1_000_000`（唯一名称数 +
   唯一名单数合计）。标准目录与全部已知来源的名称总量在 1e5 以下，上限留
   约两个数量级余量；正常内容永远触不到。
2. 三个生产入口 `content-native-loader::{bridge_pack, bridge_generals,
   bridge_pile}` 在解析前检查 `interned_count() >= LIMIT`，超限返回
   `BridgeError::SymbolTableLimit { limit }` 拒装。调用方（安装器/客户端/
   服务端）按既有坏包路径处理：明确报错、不静默、不 panic。
3. 引擎内唯一其它 intern 调用点 `host_skill_def`（`content/host.rs`）的
   入参来自装载器已 intern 的声明键，天然有界，不设第二道门。

**为什么选上限拒装而不是回收：**

- 回收要求放弃 `'static str`（改 `Rc`/owned 或世代句柄）。该类型贯穿
  目录（`CardDef::name`、`ExtensionGeneral`）、快照编码与字节存档路径
  ——换类型是全代码面改动，并牵动存档语义与版本门禁，收益只是"回收
  实际不可达的内存"（唯一名称数本身有界）。
- LRU/世代缓存同样要求非 `'static` 句柄 + 失效判定（名称被目录/存档
  引用期间不可逐出），复杂度更高。
- 拒装与"生产入口在装载器、坏包一律明确报错"的既有架构（0777）一致，
  是唯一零类型面改动的收口点。

## 边界（登记）

- 入口检查发生在解析前：**单个包**自身可在一次 bridge 内 intern 逼近上限
  （每名一次分配），该次贡献受包文件大小界定（JSON 全量在内存）。防单包
  爆量需预扫描去重计数，未做——登记为已知边界，包来源可信度模型里由
  安装审批/签名链（`signature_trust_chain`）承担。
- 上限数值为工程余量选择，非上游对齐项；调整只需改常量并重跑
  `symbol_table_limit` 验收。

## 验收

- `content-native-loader/tests/symbol_table_limit.rs`（独立进程，独占
  符号表）：上限内装载通过 → 填充恰达上限 → 装载返回
  `SymbolTableLimit` → `remaining_capacity()==0` 且重复 intern 不增长。
- 既有 `loader_acceptance_bounded_names_and_clear_errors`（50 轮重复
  装载/启停零增长）继续作为"重复循环不泄漏"的正反例。
- `cargo test --offline -p content-native-loader`、deps 三包、engine-native
  全量、Gate 1b 见[证据](../operations/evidence/2026-10-11-t6-2-symbol-table-limit/README.md)。
