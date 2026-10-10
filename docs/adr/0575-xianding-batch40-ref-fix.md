# ADR 0575：xianding characters.json 引用修复（batch40 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch40 为 D 段 opencode 领取，0572）

## 背景

全库 npack 级扫描发现 xianding 包武将 liubian（刘辩，qun 3 体）第一技能
引用失效 id（全库检索无）。batch40（势援）在 23:37 定义 shiyuan（.skill），
但 characters.json 引用未同步——与 batch27-38 遗留同模式（opencode 定义技能
→ 重打 npack → 更新 README 后转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| liubian（刘辩） | shiyuan, dushi, yuwei | 3653071969 | shiyuan=4115301600 |

另两技能引用已正确（dushi=3887700670 / yuwei=3414942371）。

## 变更

1. `content/original/ported/xianding/characters.json`：liubian 第一技能修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（506 技能 /
   196 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- **全库 31 包 npack 级扫描 ALL CLEAN（零未解析引用）**——xianding/diy 全部
  修复闭环后首次达成
- Hermes 累计修复 xianding 24 武将 + diy 61 角色
- extensions md5 三份一致
