# 2026-09-24 E02/G1 包生命周期核验（C0 生产侧装载器切片）

本证据描述台账 E02/G1 采用候选 C0（生产侧装载器与名称所有权）的落地切片：
新增宿主层装载器 crate 统一「已安装包 → `ExtensionPack`」转换（S11）、名称
符号表消除循环泄漏（S12）、g1 测试部分迁移且结果不变。择址决策见
[ADR 0777](../../../adr/0777-native-c0-native-loader.md)。
上一轮证据：[N11 字节恢复](../2026-09-24-n11-tag-registry/README.md)。

> 2026-09-24 审查补注（[审查修复证据](../2026-09-24-review-fixes/README.md)）：下表「全量验收」
> 的 362 应为 342（341 + 1）。「内存界验收」的循环只在循环外装载一次，循环内只建局与
> 启停，且装载器的武将技能名单仍用 `Box::leak`（符号表计数看不到），"50 轮装载"不成立。
> 审查修复后装载器改为包数据自描述（ADR 0778），验收改为循环内重新装载并加指针同一性
> 断言。其余内容保留当时语境。

## 检查与结果

| 检查 | 结果 |
|---|---|
| 源码版本 | STANDARD_RULESET_VERSION=60、ARCHIVE_FORMAT_VERSION=2（名称类型与编码字节不变，均不递增） |
| 基线校验 | `shasum -a 256 -c ../2026-09-24-n11-tag-registry/source-hashes.txt` 13/13 一致（开工时源码与该证据同源） |
| 新 crate | `crates/content-native-loader`（ADR 0777 择址：content-runtime 反向依赖 engine-native 会成环；engine-native 不引入 serde）。依赖 engine-native + content-runtime + serde_json，进 workspace members |
| 装载器 | `PackSpec` 数据驱动：牌种映射、宿主行为键映射、牌存在性派生技能、武器化/装备技规则、`host_api` 版本校验、deferred 未映射清单；`BridgeError` 六类明确错误 |
| 名称所有权 | `engine-native::content::names` 符号表（intern）；`host_skill_def` 的 Box::leak 改走符号表；泄漏上界 = 唯一名称数 |
| 内存界验收 | `loader_acceptance_bounded_names_and_clear_errors`：预热后 50 轮「装载 → 建局 → 禁用 → 启用」，符号表 `interned_count` 增量为 0 |
| 错误面验收 | 坏 JSON → BadJson；缺 cards → MissingField；未知牌种 → UnknownCardKind；未声明行为键 → UnknownBehaviorKey；host_api=999 → HostVersionMismatch{999, current}；不声明（旧包）视为兼容 |
| bridge 迁移 | gujian（30 牌种 + 18 派生技能 + 2 装备技）与 wudian（7 牌种 + 4 行为键 + 武器化/装备技 + 6 武将经 `bridge_generals`）改调装载器；kind_of/suit_of/Box::leak 散布点删除 |
| g1 不变性 | `--test g1_shahai_gujian` 13/13、`--test g1_wudian` 7/7——测试名、数量与结果与迁移前一致 |
| 全量验收 | `cargo test --offline -p engine-native` + `-p content-native-loader`：**362 通过 / 0 失败**（含 engine-native 341、装载器验收 1），完整输出见 [test-logs.txt](test-logs.txt) |
| CardKind 双射 | 上轮静态核查结论沿用（本批未改编码表） |

## 结论边界

- 已实现并验收：装载器 crate 与择址 ADR、名称符号表与循环内存界、错误面、
  两份 bridge 迁移（g1 不变性）、全量 362/0。
- 剩余切片（C0 第 3 点未完）：其余 6 份 bridge（huanle、d3、hearth、cardpile、
  gwent、yunchou）迁移装载器；`build_package`（npack 构建器，属包构建非装载）
  是否收敛另行评估。已登记台账 E02 行与演进提案 C0 小节。
- 名称类型的进一步收敛（`&'static str` → `Arc<str>`/符号表语义进入引擎核心
  类型）未做：当前 intern 方案已满足内存界验收且保持哈希/存档字节不变；
  类型迁移影响 canonical_hash/archive 全链路，须独立切片。
- C3 包管理：本批仅复用 content-runtime 既有发现/导入/内容锁/回滚，未新增
  包清单/签名/依赖解析能力。
- 不涉及规则语义变化（规则/格式版本未递增）；无性能测量（本批无性能结论）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（shasum -a 256，相对 noname-next 根）。
