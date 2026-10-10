# ADR 0029：共享移出区与按牌种移出原语

## 决策

每局比赛初始化一个无拥有者的共享 `Removed` 区。内容 DSL 新增
`remove_cards_by_kind(card_kind)`：按 ZoneId、区内牌序和 CardId 的确定顺序，将摸牌堆、
弃牌堆、手牌区、装备区及判定区中指定规则牌种的全部实体牌公开移入该区；处理中和已移出区
不参与扫描。原语通过 Rule VM 产生带稳定牌种标签的 `EffectRequest`，技能不得直接遍历或修改
牌区。

## 原因

旧扩展常用 DOM 节点和 `ui.special` 移出某类牌。运行旧 JS 或暴露 UI 容器会破坏无头服务器、
Native/WASM 一致性和回放。共享区及类型化效果把这类行为收回权威状态机，同时允许不同技能
复用同一实现。首批迁移的联动包「骏魁」「骥冠」分别移出进攻坐骑和防御坐骑，再复用既有
【杀】次数与手牌上限 modifier。

## 兼容性

`EffectRequest`、`StmtIr` 与 VM effect tag 均只在末尾追加新变体/编号。新比赛多一个稳定共享区，
权威状态哈希会按预期变化；`EngineSemanticsId` 从修订 12 升至 13，Native、浏览器 WASM 和
skill-vm 必须重新通过跨目标一致性门禁。
