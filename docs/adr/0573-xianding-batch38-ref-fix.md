# ADR 0573：xianding characters.json 引用修复（batch37/38 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch37/38 为 D 段 opencode 领取，0568/0569）

## 背景

全库 npack 级扫描发现 xianding 包武将 shen_pangtong（神庞统，shen 1 体）
第三技能引用失效 id（全库检索无）。batch37/38 定义 luansuo/fengliao 等
（.skill），但 characters.json 引用未同步——与 batch27-36 遗留同模式
（opencode 定义技能 → 重打 npack → 更新 README 后转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| shen_pangtong（神庞统） | luansuo, fengliao, kunyu | 2478060157 | kunyu=3623529353 |

前两技能引用已正确（luansuo=3225755369 / fengliao=2687295950）。

## 变更

1. `content/original/ported/xianding/characters.json`：shen_pangtong 第三技能修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（506 技能 /
   196 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- 全库 31 包 npack 级扫描 ALL CLEAN（不含 opencode 后续批次活跃暂态）
- Hermes 累计修复 xianding 23 武将 + diy 61 角色
- extensions md5 三份一致
