# P2/N08 缺牌种批①-1：无中生有/桃园结义（规则 v62，2026-10-07）

主稿 P2/N08「缺牌种补齐批①」第一子批：四缺牌种中的两个（另外两个
五谷丰登/借刀杀人需多段询问机器，批①-2 另批）。上游对照
`card/standard.js` list 块（花色点数原样落位）。前置：
[牌表逐张对照](../2026-10-07-n08-cardlist-parity/README.md)。

## 本批落地

| 牌种（编码） | 上游牌 | 语义 | 实现 |
|---|---|---|---|
| Wuzhong（90） | 无中生有 ♥7/8/9/11 ×4 | 使用后摸 2 | CardUsed 帧 → 无懈窗口包装（Dismantle 同款）→ WuzhongMachine（快照 tag 56） |
| Taoyuan（91） | 桃园结义 ♥1 ×1 | 所有其他存活角色回复 1（满血跳过，使用者不在范围） | 同包装 → TaoyuanMachine（快照 tag 57） |

- 声明表 80→**85 张**（inventory 测试同步），`STANDARD_RULESET_VERSION`
  61→**62**（既定流程：声明表变更 = 递增 + 重录）。
- **14 golden 全量重录**：牌堆构成变化（+5 张入洗牌池）使全部场景哈希
  变化；行为语义无变化，重录依据 = 确认语义后更新流程（v60/v61 先例）。

## 种子敏感级联修复（三处，测试侧零语义改动）

| 测试 | 症状 | 修复口径 |
|---|---|---|
| `luoshen_repeat_judgement_gains_black_cards` | 判定首张变红，「≥1 黑」种子属性失效 | 注入黑色判定首张去随机化（牌堆顶=末尾） |
| `propagated_damage_is_full_resolution` | 3 号起手摸到无懈：链子的无懈窗口挂起，火杀提交被 ChoicePending 拒绝 | 链子/火杀后各排空无懈窗口（放弃） |
| `identity_pure_command_match_reaches_outcome` | 双种子终局阵营翻转 | 按实跑复核更新快照值（v30/v60 先例同款注释），双种子现均主公方 |

并行会话冲突处理：a0 技能声明表曾出现「无中生有/桃园结义」两条——
锦囊牌种走牌种机器不经 SkillDef（拆/顺先例），已按落地设计修正并注明
「锦囊不入技能声明表」。

## 验收口径（分开陈述）

- **代码已实现**：2 牌种 + 5 张卡 + 结算机器 + 无懈窗口 + 声明表 +
  版本递增。
- **回归已加入**：`wuzhong_taoyuan_trick_behaviors`（摸 2/离场/AoE 回复
  含满血跳过与使用者排除）。
- **golden 已重录**：14 场景全量（重录依据如上）。
- **全量通过**：engine-native **370/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7、inventory 2/2、replay 2/2；content-runtime 32/0、
  engine-ai 29/0；lib `-D warnings` 通过。

## 剩余

五谷丰登/借刀杀人（批①-2，多段询问机器）；装备命名化（批②）；数量
对齐（批③）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
