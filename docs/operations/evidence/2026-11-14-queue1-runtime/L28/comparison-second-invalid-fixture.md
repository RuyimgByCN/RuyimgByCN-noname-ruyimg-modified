# L28 逐场采集对照（2026-11-14）

这是采集者对声明投影的重算，待另一人独立复核；不签署整技能等价。初态包含卡牌性质与区域内顺序。完整投影、差异字段和 39 次 NGSA 恢复见 [JSON](comparison-results.json)，逐场 raw/CDP 行号见 [索引](final-case-index.json)。

| 场景 | 初态投影 | 上游执行 | 结算区域投影 | 支付序列 | HP事件顺序 | 限定技消耗（原生/上游） |
| --- | --- | --- | --- | --- | --- | --- |
| buxu-basic | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| buxu-equip | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| buxu-nullify | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| buxu-cancel | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| buxu-empty | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| buxu-full | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| buxu-twice | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| buxu-invalid | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| zhengjing-one-self | 一致 | Spine入口阻塞 | 未验 | 未验 | 未验 | 未验 |
| zhengjing-three-target | 一致 | Spine入口阻塞 | 未验 | 未验 | 未验 | 未验 |
| zhengjing-zero | 一致 | Spine入口阻塞 | 未验 | 未验 | 未验 | 未验 |
| zhengjing-cancel | 一致 | Spine入口阻塞 | 未验 | 未验 | 未验 | 未验 |
| zhengjing-no-skill | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| yeyan-small-one | 一致 | 完成 | 有差异 | 一致 | 一致 | True/True |
| yeyan-small-three | 一致 | 完成 | 有差异 | 一致 | 有差异 | True/True |
| yeyan-great-one | 一致 | 完成 | 有差异 | 一致 | 有差异 | True/True |
| yeyan-great-split | 一致 | 完成 | 有差异 | 一致 | 有差异 | True/True |
| yeyan-cancel | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| yeyan-used | 一致 | 完成 | 有差异 | 一致 | 一致 | True/True |
| yeyan-no-four | 一致 | 完成 | 有差异 | 一致 | 一致 | True/True |
| yeyan-target-cancel | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| yeyan-great-discard-cancel | 一致 | 完成 | 有差异 | 一致 | 一致 | False/False |
| yeyan-great-one-default | 一致 | 完成 | 有差异 | 一致 | 有差异 | True/True |
| yeyan-great-split-default | 一致 | 完成 | 有差异 | 一致 | 有差异 | True/True |
| yeyan-great-two | 一致 | 原生无2火单目标选项 | 有差异 | 一致 | 有差异 | True/True |

所有历史轮次完整保留，没有从成功样本中删除差异。round6 的大业炎直接取消尝试在四牌已自动选择时只有 OK，重试只经原始头像监听器撤回三团火再取消；该失败尝试仍在索引列明。`yeyan-great-two` 的原生配对是最近的“大模式、单目标”而非相同火数输入，不计作同输入通过。

跨引擎不比较完整事件流或 RNG；39 次原生自身恢复则逐次比较完整快照（仅归一 auth_generation）、状态哈希、命令结果和新增日志。卡牌身份、名称、花色、点数、性质及区域顺序已进入初态投影；未执行的通用武器/坐骑语义仍未验。
