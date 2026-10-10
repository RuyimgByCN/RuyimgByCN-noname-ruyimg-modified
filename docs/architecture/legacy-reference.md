# 旧实现参考与文档清理

更新：2026-10-06。当前目标见 [性能架构](../../noname-performance-architecture.md)，
实际进度见 [原生重写进度](native-progress.md)。

## 旧线仍承担的职责

现有客户端、服务端、AI 和内容工具仍依赖 `engine-core`。这些代码用于运行、规则研究和测试迁移；
旧 DSL/VM 与 WebView 并非已选定的新内核实现；**动态扩展能力已按 2026-09-22 用户要求恢复为目标**。
现有包发现/依赖/启停/资源导入应优先评估复用，旧扩展按能力适配，见 [原生扩展机制](native-extensions.md)。

| 范围 | 参考入口 | 当前边界 |
|---|---|---|
| Web / 桌面 | [Web 客户端记录](phase6-web-client.md)、[桌面说明](../../apps/desktop-client/README.md) | Vue + Worker/WASM + Tauri；未切换原生牌桌 |
| AI | [搜索入口](../../crates/engine-ai/src/submit.rs)、[fork](../../crates/engine-ai/src/fork.rs)、[评分](../../crates/engine-ai/src/search/mod.rs) | 完整权威状态仍进入搜索；投影函数存在不代表搜索信息隔离 |
| fork 实验 | [fork_strategy.rs](../../crates/engine-ai/src/fork_strategy.rs) | 策略类型和测试不证明零分配、零内存或真正的完整撤销日志；须按实际负载测量 |
| 渲染策略 | [renderer-runtime](../../packages/renderer-runtime/src/lib.rs) | DOM 预算、帧合并、动画策略；不是已集成的 GPU 渲染器 |
| 移动平台 | [历史 ADR 0702](../adr/0702-m7-mobile-blocking.md) | 旧壳可行性与清单记录，不是 Android/iOS 真机验收 |
| 联机 | [协议记录](../protocol/phase7-multiplayer.md)、[房间](../../apps/game-server/src/room.rs) | 旧内核单写房间，仍待新线集成与真实持久化 |
| 内容 | [注释台账](../content-fidelity-ledger.md)、[明细](../content-fidelity-ledger-detail.md)、[目录](../../content/original/catalog.json) | `ported` 和无近似注释均不等于语义正确 |
| 规则研究 | [规则词汇](../rule-semantics/vocabulary.md)、[历史批次](../migration/upstream-1.11.5-delta.md)、[ADR 索引](../adr/INDEX.md) | 保留规则取舍与来源证据，历史批次数字不作当前总进度 |
| 工具及包处理 | [迁移工具记录](../migration/phase9-legacy.md)、[作者说明](../extensions/author-guide.md) | 旧工具实现参考；包管理与扩展能力须保留，原生接入及旧 JS 兼容程度另验 |

保留的阶段文档和旧 ADR 中，M0–M8、阶段号、旧计划章节号及当时通过项均为历史背景。
新主线的发布标准由性能架构定义；有明确日期的旧测试记录不自动升级为当前复验结果。

## 2026-10-06 文档整理

| 处理 | 原因与替代入口 |
|---|---|
| 删除过期现状与重复待办 | v60 golden、牌堆循环、XiaoJi tag 已修；扩展契约、覆盖清单、规则基准、演进提案和台账中的旧阻断描述已校正。跨领域处理顺序合并回[进度主稿](native-progress.md) |
| 校正演进提案 S11–S13/C0 | 生产装载器已存在，两份桥接已迁移；同包重载复用不等于唯一名称可卸载回收；当前 P0 是异常存档解码。保留候选与已采用切片的区别 |
| 更新包台账与导航 | 玩点论杀当前为 14 张包数据、6 名武将元数据；武将声明仅思继映射、10 条未接入，纠正原“9 项”，莫守宿主样本不等于费诗装配。补装载器模块入口；逐包功能与整包验收仍归[台账](../migration/upstream-extension-reconstruction.md) |
| 保留历史文档 | 本次检查当前入口文档未发现内容完全相同的二级章节或失效本地文件链接；有独有决策/证据的文档保留日期语境，本次未删除整份文件 |

本次源码摘要、实跑与未修缺陷探针见[复核证据](../operations/evidence/2026-10-06-native-progress-audit/README.md)。

## 2026-09-24 文档整理

| 处理 | 原因与替代入口 |
|---|---|
| 删除 `docs/architecture/optimization-analysis-2026-09-24.md` | 原稿的 P0/P1 排序和收益百分比无同口径基准；可核实的静态结论已并入[测试与性能证据](../operations/phase17-test-perf-infrastructure.md#2026-09-24-优化假设复核)，O1–O5 待办归[进度主稿](native-progress.md#性能优化核验后的待办2026-09-24) |
| 去重 `docs/migration/upstream-1.11.5-delta.md` | 删除 99 个内容完全相同的二级章节（2,093 行），保留每组首次出现及同标题不同内容的历史记录 |
| 合并当前状态 | README 和[架构索引](overview.md)仅保留导航；[扩展宿主契约](native-extensions.md)只保留能力边界，E01–E06 状态归[37 来源台账](../migration/upstream-extension-reconstruction.md) |

正式 ADR、分轮证据和独有的旧线阶段文档仍保留历史语境。

## 2026-09-16 已删除的过期文档

本次删除 9 份文件，代码、素材、正式规则 ADR 和机器读取的 ABI/覆盖清单保留。
删除前的原文件副本保存在工作区外的临时压缩包，用于本次操作回退；不再作为项目文档入口。

| 已删除路径 | 原因 / 替代入口 |
|---|---|
| `noname-next-architecture-plan.md` | 旧 v3.0 总计划与现行原生目标冲突；改读性能架构和重写进度 |
| `docs/architecture/phase8-ai.md` | 将投影和完整状态 fork 连成已隔离路径；以本文 AI 边界及源码为准 |
| `docs/operations/ai-fork-strategies.md` | 零内存/零分配等结论没有同负载证据；保留策略源码和测试 |
| `docs/operations/phase25-game-content.md` | 早期占位技能、旧牌数和完成勾选失效；改读两条线各自的覆盖清单 |
| `docs/operations/phase28-mobile-shell.md` | WebView 固定路线被取代，清单测试不能代表真机通过；保留 ADR 0702 历史说明 |
| `docs/operations/phase32-battle-renderer.md` | 将 DOM 策略类型登记为完整渲染器；以源码和新线进度为准 |
| `docs/adr/notes/task1-continuation-analysis.md` | 一次性底稿已由 ADR 0596 固化 |
| `docs/adr/notes/task3-guozhan-investigation.md` | 一次性底稿已由 ADR 0598 固化 |
| `docs/adr/notes/task3-stage-c-investigation.md` | 阶段 C 调研由 ADR 0601–0604 的正式决策接替 |

正式 ADR 保留其历史决策，只修正文档导航和已失效的引用，不把新规则要求倒写成旧决策。
