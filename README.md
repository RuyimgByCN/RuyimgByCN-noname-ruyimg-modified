# noname-next

无名杀原生重写工程。目标见[性能架构 v3.3](noname-performance-architecture.md)：Rust 原生规则、可暂停的顺序事件执行、独立 AI 计算和原生 GPU 牌桌。**当前实现、测试结果与失败项只在[进度主稿](docs/architecture/native-progress.md)维护**；目标架构、历史 ADR 和旧线成绩都不等于当前验收。

原有扩展机制属于必做范围。玩家应能独立安装、启停、升级、卸载，作者应能发布规则、武将、技能、卡牌、模式、AI、配置、资源和界面扩展。指定的 noname 与 noname-liuli 两仓全部 37 个来源实例、逐项状态和完成条件见[扩展台账](docs/migration/upstream-extension-reconstruction.md)，宿主能力见[扩展契约](docs/architecture/native-extensions.md)。子包卡牌转换与整包功能/规则等价验收分开统计。

## 阅读入口

| 文档 | 用途 |
|---|---|
| [架构索引](docs/architecture/overview.md) | 当前模块路径与阅读顺序 |
| [重写进度](docs/architecture/native-progress.md) | 最新源码/测试快照、已完成范围、失败项和待办 |
| [原生覆盖清单](docs/native-coverage-manifest.md) | 机制、场景与 golden 登记 |
| [玩法规则基准](docs/architecture/native-rules-baseline.md) | 规则版本、上游差异与语义边界 |
| [两仓扩展台账](docs/migration/upstream-extension-reconstruction.md) | 37 来源的功能清单、迁移路线和逐包验收 |
| [测试与性能证据](docs/operations/phase17-test-perf-infrastructure.md) | 旧线/原生线口径与优化假设复核 |
| [原生基准](docs/operations/phase19-native-baseline.md) | runner 场景与测量限制 |
| [ADR 索引](docs/adr/INDEX.md) | 按日期读取的历史决策 |
| [旧实现参考](docs/architecture/legacy-reference.md) | 旧模块、资料与清理记录 |

## 代码路径

| 路径 | 职责 |
|---|---|
| crates/engine-native | 新原生内核及切片；仍需正式客户端/服务端接入 |
| crates/content-native-loader | 已有生产包数据桥接；部分 G1 样本已复用，正式产品接线待验收 |
| packages/extensions | 已转换的独立包样本与卡牌子包 |
| crates/engine-core、engine-api、rule-api | 现有产品规则和接口、迁移对照 |
| crates/engine-ai、engine-wasm | 现有 AI 与 WebAssembly 路径 |
| apps/web-client、apps/desktop-client、apps/game-server | 现有客户端和服务端 |
| content、extensions | 旧线内容、DSL、包工具与资源，不能直接算原生规则验收 |

## 常用验证

    cargo test --offline -p engine-native
    cargo test --workspace
    cargo run -p conformance
    bash scripts/cross-target-hash.sh
    pnpm --dir apps/web-client build

这些命令覆盖不同执行线。原生当前失败、最近一次完整全绿证据和跨目标限制见[进度主稿](docs/architecture/native-progress.md)。
