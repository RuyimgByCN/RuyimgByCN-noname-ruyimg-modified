# 2026-09-24 审查修复核验（主动无懈崩溃 + 装载器数据化 + 审查实验入档）

本证据描述 2026-09-24 晚对当日 P0 与 E02 改动审查后的三项跟进：修复出牌阶段主动
使用无懈导致的 panic；装载器改为包数据自描述（[ADR 0778](../../../adr/0778-native-c0-pack-data-keys.md)）；
把审查实验写入证据，并更正此前文档与证据中的表述。
上一轮证据：[E02 装载器核验](../2026-09-24-e02-pack-loader/README.md)。

## 检查与结果

| 检查 | 结果 |
|---|---|
| 源码版本 | STANDARD_RULESET_VERSION=60、ARCHIVE_FORMAT_VERSION=2（均未递增：没有合法对局轨迹、存档字节或哈希变化，golden 未变） |
| 基线校验 | `shasum -a 256 -c ../2026-09-24-e02-pack-loader/source-hashes.txt`：16 项中 10 项一致；不一致的 6 项均为本轮改动（装载器 lib 与验收测试、`host.rs`、`names.rs`、`g1_wudian.rs`、`g1_shahai_gujian.rs`）。文档主稿开工时与 E02 证据同源 |
| 无懈崩溃（修复前） | 旧代码出牌阶段主动打出无懈，无目标与有目标都在 `engine.rs:2296` 触发 `unreachable!("handled above")`；打出闪（对照）返回 `Rejected(WrongPhase)`（[experiments.log](experiments.log) 实验一） |
| 无懈修复 | `PlayCard` 牌种路由把 `Nullify` 与 `Shan` 同列为仅响应牌，返回 `WrongPhase`；删去被遮蔽的重复分支（原 6 处 unreachable-pattern 警告）；`lib.rs` 加 `#![deny(unreachable_patterns)]`，此后重复匹配臂编译失败 |
| 回归测试 | `native_kernel::nullify_rejected_when_played_actively`：无目标/有目标均为 `WrongPhase`，状态不变，恢复件同样拒绝。该测试在旧代码上先失败（同一 panic），修复后通过 |
| 数据键表 | `CardKind::key/from_key/ALL`（86）、`HostBehavior::key/from_key/ALL`（74）、`InitHook::key/from_key`、牌种伴随技能表（23 个牌种）。`a0_tag_registry.rs` 新增 3 项：两张键表双射且键唯一、伴随技能名全局唯一并覆盖固有装备技 |
| 装载器 | 入口改为 `bridge_pack(json)` / `bridge_generals(json)`，删除 `PackSpec`、`PresenceSkill`；新增错误 `InvalidField`、`UnknownFaction`、`UnknownGender`、`UnknownInitHook`、`UnknownEquipSkill`、`DuplicateSkill`；`HostVersionMismatch.declared` 改为 u64 |
| 包数据 | 玩点论杀、古剑的 `cards.json` 增加 `pack` 段，玩点论杀三尖增加 `range: 3` 与 `equip_skills`。重跑生成器后只有这两包的 `cards.json` 与 npack 的 `source_hash`/`asset_hash` 变化；先备份再生成的二次复核无差异（[generator-diff.txt](generator-diff.txt)） |
| 测试迁移 | `g1_wudian.rs` 删除 `wudian_spec` 与诱敌深入测试的手写桥接；`g1_shahai_gujian.rs` 删除 `gujian_spec`（30 项牌种表、18 项派生技能、装备技表）及未使用的 `CardJson`。新增断言：包元数据来自 cards.json；古剑 18 个伴随技能按牌首次出现序登记（与迁移前声明序一致，技能编号不变） |
| 装载器验收 | 样例覆盖包元数据、射程与装备技、古剑与 gwent 伴随技能、武将；错误断言 30 条，覆盖 `BridgeError` 全部 12 个变体；循环内每轮重新装载牌与武将，符号表计数不变，且牌名与武将技能名单的指针与预热时相同 |
| 指针断言先报红 | 临时把武将技能名单换回 `Box::leak`：新断言失败；同一改动下只保留计数断言则通过，说明计数看不到绕过符号表的分配（experiments.log）。临时改动已还原 |
| A0 先报红（补） | 当前 `standard_catalog_inventory.rs` 在缺陷源码（规则 59、拆牌循环缺陷还原）上 2/2 失败：`标准目录牌数 88 != 声明 80`、顺手牵羊 `8 != 4`（实验二） |
| golden 归因（实证） | 同一缺陷源码配第 48 轮 v59 golden 期望值：`replay_scenarios_match_golden` 通过，14 个场景哈希逐一复现 v59 golden（实验三）。v59→v60 的差异确由版本字段与牌堆修正构成，没有其他行为差异 |
| 定向测试 | `--test native_kernel nullify_rejected_when_played_actively` 1/1；`--test a0_tag_registry` 5/5；`--test g1_wudian` 7/7；`--test g1_shahai_gujian` 13/13；`-p content-native-loader` 1/1 |
| 全量验收 | `cargo test --offline -p engine-native --no-fail-fast`：**345 通过 / 0 失败 / 0 忽略**（E02 时 341，加无懈回归 1 项、键表测试 3 项），完整输出见 [native-tests-full.log](native-tests-full.log)；`-p content-native-loader` 1/1（[loader-tests.log](loader-tests.log)）；`-p content-runtime` 31/0（[content-runtime-tests.log](content-runtime-tests.log)）；`cargo check --offline --workspace --all-targets` 无错误 |
| 构建目录 | 全量日志在 `CARGO_TARGET_DIR=target/review-0924/iso-target`（上一步 shell 残留的设置）下编译工作区当前源码，其余在沙箱默认构建目录；均未使用共享 `target/`，未执行 cargo clean |

实验一至三在隔离副本上运行：副本是审查时 `crates/engine-native`（本轮修复前）的独立拷贝，
实验用的两处回退见 [iso-rollback.diff](iso-rollback.diff)，探针源码见
[review-probes.rs.txt](review-probes.rs.txt)。

## 更正此前的文档与证据

| 位置 | 原表述 | 实际 | 处理 |
|---|---|---|---|
| [E02 证据](../2026-09-24-e02-pack-loader/README.md) | 全量 362 通过（含 engine-native 341、装载器 1） | 341 + 1 = 342 | 原文加注 |
| E02 证据、[覆盖清单](../../../native-coverage-manifest.md) `pack_loader`、[演进提案 C0](../../../architecture/architecture-evolution-plan.md#c0-生产侧装载器与名称所有权) | 50 轮"装载/建局/启停"后符号表增量为 0 | 装载只在循环外做一次，循环内只建局与启停；装载器的武将技能名单仍用 `Box::leak`，计数看不到 | 装载器改为循环内重新装载并加指针断言；原文加注 |
| [N11 证据](../2026-09-24-n11-tag-registry/README.md)、[规则基准](../../../architecture/native-rules-baseline.md) | XiaoJi 挂起旧字节"显式 UnexpectedEof/BadTag" | tag 29 仍是 Dismantle，旧字节先按 Dismantle 错位解码，再在后续字段报错：单挂起与双 XiaoJi 为 `UnexpectedEof`，XiaoJi 后接 Dismantle 为 `BadTag("slot", 3)`；解码不校验载荷是否读尽，报错没有机制保证 | 规则基准改写；N11 证据加注 |
| [回放与目录证据](../2026-09-24-replay-golden-v60/README.md)、演进提案 A0 | A0 目录检查未在缺陷源码上先报红 | 实验二已报红 | 加注 |
| 回放与目录证据 | 哈希差异来源为版本字段与牌堆修正（静态推断） | 实验三实证 | 加注 |
| [进度主稿](../../../architecture/native-progress.md) | 最近一次全绿指向 N11（341）；engine.rs 6 处 unreachable-pattern 警告"非本批范围" | 最近全绿为本证据（345/0）；警告已清理并设为编译错误 | 主稿更新 |
| [扩展台账](../../../migration/upstream-extension-reconstruction.md)第 7 行、U05 | v60 完整回放仍有 1 项失败 | 当日已修复 | 改为链接主稿 |
| [ADR 索引](../../../adr/INDEX.md)顶部 | 最新核验为 9-23 第 16 轮（规则 v28、238 项） | 已过期 | 改为链接主稿 |

## 新发现（未修，登记为主稿待做）

1. **存档牌种编号解码 panic**：`archive.rs` 私有函数 `num_to_card_kind` 遇未知编号直接
   `panic!`，共 4 处调用（`declared_kinds`、`use_as`、两处 `need`）。存档校验和是无密钥
   blake3，伪造载荷后重算即可通过帧校验。探针把 `declared_kinds` 的牌种字节改成 255 并
   重算校验和，`decode_archive` panic：`archive: unknown card kind 255`（experiments.log）。
   应返回 `BadTag` 并补负例测试。
2. **解码不校验载荷读尽**：`decode_archive` 在 `dec_snapshot` 之后不检查剩余字节，错位
   数据若恰好可以解析会被接受。宜与第 1 项一起处理。

## 结论边界

- 已实现并验收：无懈崩溃修复与回归（先红后绿）、禁止 unreachable_patterns、数据键表与
  A0 测试、装载器数据化与严格校验、两包数据与测试迁移、审查实验入档；全量 345/0。
- 未做：其余 6 份 bridge 迁移（huanle、d3、hearth、cardpile、gwent、yunchou；炉石须把
  `leishajian`/`shenenshu` 改为规范键 `leisha`/`jihuocard`）；A0 第 3 点 `SkillDefId` 只追加；
  存档牌种编码表仍为私有函数、未行为级测试化；上面两项新发现；与上游规则等价验收；
  性能测量（本批无性能结论）。
- 规则语义：只把原本崩溃的非法命令改为拒绝，合法轨迹不变，故规则版本不递增。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（shasum -a 256，相对 noname-next 根）。
