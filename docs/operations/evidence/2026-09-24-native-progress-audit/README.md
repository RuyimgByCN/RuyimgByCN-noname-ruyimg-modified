# 2026-09-24 当前原生进度独立复核

本证据只描述本次工作区快照。源码和卡牌数据摘要见[source-hashes.txt](source-hashes.txt)，原生命令输出见[native-tests.log](native-tests.log)。此前最近一次完整全绿是[第 48 轮 v59](../2026-09-23-native-review-48/README.md)。

| 检查 | 结果 |
|---|---|
| 源码 | STANDARD_RULESET_VERSION=60、ARCHIVE_FORMAT_VERSION=2；gwent cards.json 为 21/21 张、deferred 0 |
| 定向 | cargo test --offline -p engine-native --test g1_shahai_gwent：6/6 通过 |
| 测试目录 | cargo test --offline -q -p engine-native -- --list：列出 336 项 |
| 全量 | cargo test --offline -q -p engine-native：在 replay_scenarios_match_golden 失败；该测试内部 14 个场景哈希均与 golden 不符；此前 328 项通过，后续 7 项未执行 |
| 补跑 | view_isolation 单独 7/7 通过；因此可确认 335 项分别通过、1 项失败，不能声明 v60 全量通过 |
| 扩展范围 | packages/extensions 为 10 个转换包目录；两个上游来源清单重跑后与原 inventory.json 逐字节一致，整包原生验收仍 0/37 |

规则版本参与状态摘要，因此 v59→v60 本身会改变场景哈希；这不证明所有差异都合理。standard_catalog_parts 中顺手牵羊与酒/雷杀的入目录操作还位于拆牌两次循环内，可能额外生成实体牌。下一步应先核对标准牌集合、14 个场景的命令/事件/状态轨迹和祖墨手铠封印语义，再决定是否更新 golden，随后重跑全量并形成新一轮通过证据。

另有独立的字节存档问题：[archive.rs](../../../../crates/engine-native/src/archive.rs) 同时给 Dismantle 和 XiaoJi 机器快照编码 tag 29；解码先匹配 Dismantle，XiaoJi 分支不可达，编译器在本次测试输出中也报告 unreachable pattern。现有枭姬测试仅验证内存询问与摸牌，未覆盖挂起字节恢复。需修 tag 冲突并补往返/负例，不能因其他恢复用例通过而关闭 N11。

本轮只核验与整理文档；未修改内核或重录 golden。当前任务状态见[进度主稿](../../../architecture/native-progress.md)。
