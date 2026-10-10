# ADR 0587：xianding characters.json 引用修复（batch43 遗留）

- 日期：2026-08-04
- 领取者：E 段 Hermes
- 涉及包：xianding（batch43 为 D 段 opencode 领取，0583）

## 背景

全库 npack 级扫描发现 xianding 包武将 dc_sb_yangfeng（界杨奉，qun 3 体）
第三技能引用失效 id（全库检索无）。batch43（奇绝/毒逝·诅咒）定义 dcsbqijue
等（.skill），但 characters.json 引用未同步——与 batch27-42 遗留同模式
（opencode 定义技能 → 重打 npack → 更新 README 后转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| dc_sb_yangfeng（界杨奉） | dcsbzhubo, dcsbxieshi, dcsbqijue | 3506229050 | dcsbqijue=2527119333 |

前两技能引用已正确（dcsbzhubo=2323117459 / dcsbxieshi=2767618156）。

## 变更

1. `content/original/ported/xianding/characters.json`：dc_sb_yangfeng 第三技能修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（506 技能 /
   210 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- **全库 31 包 npack 级扫描 ALL CLEAN（零未解析引用）**
- Hermes 累计修复 xianding 26 武将（dc_shen_liubei 至 dc_sb_yangfeng）
  + diy 61 角色
- extensions md5 三份一致
