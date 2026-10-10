# P2/N08 标准缺失将第四批：伊籍「机捷/急援」（7/8，2026-10-07）

主稿 P2/N08 行缺失将第四子切片：伊籍双技能。上游对照
`character/standard/skill.js`（xinfu_jijie / xinfu_jiyuan）。前置批次：
[第三批 6/8](../2026-10-07-n08-missing-generals-3/README.md)。

## 本批落地

| 将（势力/体力） | 技 | 原生实现 | 快照 |
|---|---|---|---|
| 伊籍（蜀/3） | 机捷 xinfu_jijie | 出牌阶段限一次**主动技**（`behavior.activate` 挂钩 + once_per_turn）：取牌堆底一张（deck 首位）交任意存活角色（含自选——上游 chooseTarget 无过滤器，AI 含自引加成）；无牌可授时不可发动 | JiJie（tag 52，card 挂起期入快照） |
| | 急援 xinfu_jiyuan | Dying 触发可选：任一角色濒死可令其摸 1。**登记差异：仅濒死臂**——上游赠牌（gainAfter，giver==伊籍）/失去牌（loseAsyncAfter）臂需要 giver 跟踪面，原生无此建模 | JiYuan（tag 53） |

a0 同步：技能编号声明表 55–56（机捷/急援表尾追加，四批累计 47–56
十技）、快照变体样例 52–53（测试内变体序 50–51）。

## 验收口径（分开陈述）

- **代码已实现**：2 技 + 1 将 + 快照/编解码/哈希臂 + 装配授予。
- **回归已加入**：`yiji_assembly_jijie_jiyuan`（装配/机捷授牌堆底牌+
  含自选选项/牌堆底移除/同回合限一次拒绝/急援濒死者摸 1 体力不变/
  无人救死亡后三人局继续）。
- **全量通过**：engine-native **368/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；golden 全绿（规则/格式版本不变 v61/v4）；a0 7/7；lib
  `-D warnings` 通过。

## 剩余（1 将 2 技，缺失将最后一批）

袁术「妄尊」（他人准备阶段锁定：血比你多摸 1，主公再摸 1+其手牌
上限 −1——模式相关主公判定）/「同疾」（杀目标可弃牌转移给你——
目标转移机制）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（9 文件，相对 noname-next 根）。
