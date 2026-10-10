# ADR 0566：xianding characters.json 引用修复（batch35 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch35 为 D 段 opencode 领取，0563）

## 背景

全库 npack 级扫描发现 xianding 包武将 xizheng（希正，shu 3 体）引用失效 id
（全库检索无）。batch35（弹翼）在 22:36 定义 dcdanyi（.skill），但
characters.json 引用未同步——与 batch27-34 遗留同模式（opencode 定义技能 →
重打 npack → 更新 README 后转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| xizheng（希正） | dcdanyi, dcwencan | 3109120588 | dcdanyi=2264670487 |

另一技能引用已正确（dcwencan=2382200014）。

## 变更

1. `content/original/ported/xianding/characters.json`：xizheng 第一技能引用修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（506 技能 /
   196 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- 全库 31 包 npack 级扫描 ALL CLEAN
- Hermes 累计修复 xianding 22 武将 + diy 61 角色
- extensions md5 三份一致

## 附注

pack 期间被 Codex 引擎开发半成品阻塞（content-compiler E0425），待其编译恢复
后重 pack 成功。
