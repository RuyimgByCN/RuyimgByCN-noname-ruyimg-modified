# 架构索引

本页只说明模块关系和阅读顺序。**最新源码版本、测试和待办见[进度主稿](native-progress.md)**；这里不重复易过期的计数。目标架构见[性能架构 v3.3](../../noname-performance-architecture.md)，不能把设计目标当作现有交付。

## 阅读顺序

1. [重写进度](native-progress.md)：当前通过、失败、扩展整包状态与下一步；按批次的历史流水与 2026-10-10 事故恢复记录在[批次流水存档](native-progress-history.md)。
2. [原生架构收敛执行方案](native-convergence-plan.md)：已采纳的 NC01–NC12、依赖和验收门槛；方案本身不表示交付完成。
3. [覆盖清单](../native-coverage-manifest.md)和[玩法规则基准](native-rules-baseline.md)：原生机制、规则版本、上游语义差异。
4. [扩展宿主契约](native-extensions.md)和[两仓 37 来源台账](../migration/upstream-extension-reconstruction.md)：独立扩展能力与逐来源验收。
5. [测试与性能证据](../operations/phase17-test-perf-infrastructure.md)和[原生基准](../operations/phase19-native-baseline.md)：分清旧线、原生线和历史负载。
6. [架构演进提案](architecture-evolution-plan.md)：保留历史论证和未采纳的技术候选，不另设执行清单。
7. [ADR 索引](../adr/INDEX.md)和[旧实现参考](legacy-reference.md)：按日期读取历史，勿把旧轮次的“当前”当作现状。

## 执行路径

    本地原生：Vue / Tauri → engine.native.worker → engine-native-wasm → engine-native
    原生联机：Vue → networkClient → WebSocket → game-server 原生房间 actor → engine-native
    原生 HTTP：game-server /native 端点 → 原生内容装配与会话

    包装配：已验证安装副本 / 本局固定副本 → content-native-loader → 本局只读目录
    规则与恢复：engine-native → Rust 技能机器 / rule-vm / 顺序事件栈
                            → NGSA 字节存档 / 座位视图 / 规划器

    旧线回退：Vue / Tauri → engine.worker → engine-wasm → engine-core
              game-server 旧房间 / engine-cli / benchmark-runner → engine-core

    上游对照：archive/upstream/noname、archive/upstream/noname-liuli

上述路径于 2026-10-24 按源码核对：客户端默认原生 Worker，旧后端由 `VITE_ENGINE_BACKEND` 显式选择；WebSocket 房间是否走原生规则由服务端配置决定。`content-native-loader` 已接入原生 WASM 和服务端，正式包开局与固定副本恢复的已验范围见主稿。房间 actor 负责串行处理与保存后发布，磁盘 store/journal 负责提交决定和恢复判定；这些接线不代表所有包类型、目标平台或第三方 SDK 已完成验收。

原生与旧产品线的规则、AI、WASM、UI 和性能数据必须分开统计。上游扩展旧目录需要转换/适配后才能装入原生宿主；卡牌子包的张数不代表来源扩展整包验收。客户端、服务端、联机与目标平台的原生切换按进度主稿验收。

## 历史资料

[ADR 0703 起的决策](../adr/INDEX.md)、[第 17–48 轮及更早证据](../operations/evidence/)保存实现过程和当时的反例。旧线[共享目录/事务改造](../migration/content-registry-arc-split.md)与[内容注释台账](../content-fidelity-ledger.md)是参考，不作为原生规则正确率。
