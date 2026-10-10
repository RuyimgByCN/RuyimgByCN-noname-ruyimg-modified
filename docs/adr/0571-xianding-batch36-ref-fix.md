# ADR 0571：xianding characters.json 引用修复（batch36 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch36 为 D 段 opencode 领取）

## 背景

全库 npack 级扫描发现 xianding 包 4 名武将引用失效 id（全库检索无）。batch36
在 22:50-23:00 定义 dchuayi/dcdulu/dccuxi/dcsbkongwu 等（.skill），但
characters.json 引用未同步——与 batch27-35 遗留同模式（opencode 定义技能 →
重打 npack → 更新 README 后转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| dc_shen_zhangliao（神张辽） | dccuxi, dcduorui, dczhiti | 3767673053 | dccuxi=4093257724 |
| liufuren（刘夫人） | dcdulu, dcfuji | 3688714569 | dcdulu=2779276477 |
| dc_sb_hucheer（界虎车儿） | dcsbkongwu | 3316374594 | dcsbkongwu=3614891392 |
| duanqiaoxiao（段巧笑） | dccaizhuang, dchuayi | 3042801245 | dchuayi=2287394739 |

各武将其余技能引用已正确。

## 变更

1. `content/original/ported/xianding/characters.json`：4 武将引用修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（506 技能 /
   196 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- 全库 31 包 npack 级扫描 ALL CLEAN（不含 opencode 后续批次活跃暂态）
- Hermes 累计修复 xianding 22 武将 + diy 61 角色
- extensions md5 三份一致

## 附注

pack 期间两次被 Codex 引擎开发半成品阻塞（content-compiler E0425 / rule-vm
E0433），待其编译恢复后重 pack 成功。
