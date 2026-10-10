# ADR 0590：xianding characters.json 引用修复（batch45 遗留）

- 日期：2026-08-04
- 领取者：E 段 Hermes
- 涉及包：xianding（batch45 为 D 段 opencode 领取，0587）

## 背景

全库 npack 级扫描发现 xianding 包武将 v_lvbu（吕布变体，qun 5 体）第一技能
引用失效 id（全库检索无）。batch45（dcxiaowu 笑武）在 14:21 定义 dcxiaowu
（.skill），但 characters.json 引用未同步——与 batch27-43 遗留同模式
（opencode 定义技能 → 重打 npack → 更新 README 后转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| v_lvbu（吕布变体） | dcxiaowu, dcbaguan | 2203577701 | dcxiaowu=4153740280 |

另一技能引用已正确（dcbaguan=3826078193）。

## 变更

1. `content/original/ported/xianding/characters.json`：v_lvbu 第一技能修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（507 技能 /
   210 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- **全库 31 包 npack 级扫描 ALL CLEAN（零未解析引用）**
- Hermes 累计修复 xianding 27 武将（dc_shen_liubei 至 v_lvbu）+ diy 61 角色
- extensions md5 三份一致
