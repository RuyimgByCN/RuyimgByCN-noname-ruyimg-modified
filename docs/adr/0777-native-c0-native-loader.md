# ADR 0777：C0 生产侧装载器择址 content-native-loader（宿主层独立 crate）

日期：2026-09-24　|　状态：已采纳
关联：[演进提案 C0](../architecture/architecture-evolution-plan.md)、[台账 E02/G1](../migration/upstream-extension-reconstruction.md)、[主稿 P1](../architecture/native-progress.md)

> 2026-09-24 补注：「装载器边界」中以 `PackSpec` 为输入、「后果」末条（武将技能
> 名单一次性分配）已由 [ADR 0778](0778-native-c0-pack-data-keys.md) 取代：包数据自描述，
> 牌种/行为键与伴随技能由引擎全局表解析，名单走符号表。择址决策不变。

## 背景

S11：`packages/extensions` 的包数据在运行时读取，但「已安装包 → 原生目录
`ExtensionPack`」的桥接只存在于测试——`tests/g1_*.rs` 中 8 份各自的
`bridge()`（加 `g1_cardpile.rs` 的 `build_package`），映射规则（牌种、宿主
行为键、装备技、武将）各写一份。S12：桥接与 `host_skill_def` 的名称经
`Box::leak` 成为 `'static`，重复装载/启停循环线性增长泄漏。

C0 要求生产侧装载只有一份实现，且名称不再依赖无界泄漏。候选择址两选一：
新 crate，或 `content-runtime` 中的原生适配模块。

## 决策

**新 crate `crates/content-native-loader`**（宿主层装载器），不选
content-runtime 内置模块：

1. 依赖方向：装载器需要 engine-native 的目录类型（`CardKind`、`CardDef`、
   `ExtensionPack`、`HostBehavior`）与 serde_json 解析。engine-native 已经
   依赖 content-runtime（dev + build 期工具链），若 content-runtime 再依赖
   engine-native 即成循环。content-runtime 现有发现/导入/内容锁/回滚 API
   （`discover_extensions`、`import_extensions`、`PackageStore`）保持原位，
   装载器作为其下游消费者，不并建第二套包仓库（C3 边界）。
2. 依赖纪律：engine-native 运行期只依赖 blake3（S11 的既有设计，装载需要
   serde_json，进 engine-native 即破坏）。
3. 消费方：g1 测试、未来客户端与服务端（宿主进程）直接依赖本 crate。

## 名称所有权

`engine-native::content::names` 符号表（`intern`）取代散布的 `Box::leak`：
同名名称全局只分配一次，泄漏上界 = 唯一名称数。`host_skill_def` 与装载器
产出的动态名称全部走符号表。名称类型保持 `&'static str`——名称进入目录、
canonical_hash 与字节存档的字节不变，规则/格式版本均不递增。
`interned_count()` 供重复「装载 / 建局 / 启停」循环的内存界验收读取。

## 装载器边界

- 输入：包的 `cards.json` + 数据化 `PackSpec`（牌种映射、宿主行为键映射、
  牌存在性派生技能、武器化/装备技规则、init_hook）。
- 职责：JSON 解析、必需字段校验、`host_api` 版本一致性校验
  （对照 `tooling_host_versions().engine_api`）、映射执行、未映射（暂缓）
  清单报告、产出 `ExtensionPack`。
- 错误：`BridgeError`（坏 JSON / 缺字段 / 未知牌种 / 未知行为键 /
  版本不符），坏包在装载期失败，不进对局。
- 不做：包发现/导入/签名/回滚（content-runtime 既有）、第三方规则执行
  （C1 另立项）。

## 后果

- g1 测试逐步迁移到装载器（本轮：gujian、wudian 两份；其余六份沿用各自
  bridge，属 C0 剩余切片）；迁移只动实现不动断言，测试名、数量与结果不变。
- 桥接期（安装时一次性）的武将技能名单切片仍为一次性分配，安装不是循环
  路径；运行期名称全部有界。
