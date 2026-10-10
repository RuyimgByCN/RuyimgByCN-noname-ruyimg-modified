# ADR 0554：xianding characters.json 引用修复（batch29 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch29 为 D 段 opencode 领取，0551）

## 背景

全库 npack 级扫描发现 xianding 包 2 名武将引用失效 id（全库检索无）。batch29
（清识/残肆）在 20:20-20:38 将 dcdyqingshi/dccansi 等定义（.skill），但
characters.json 引用未同步——与 batch27/28 遗留同模式（opencode 定义技能 →
重打 npack → 更新 README 后转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| dc_duyu（杜预） | dcjianguo, dcdyqingshi | 2617108563 | dcdyqingshi=4110806243 |
| zerong（泽荣） | dccansi, dcfozong | 2171233362 | dccansi=3144382148 |

各武将另一技能引用已正确（dcjianguo=2229402743 / dcfozong=3409449847）。

## 变更

1. `content/original/ported/xianding/characters.json`：2 武将引用修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（504 技能 /
   196 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- 全库 31 包 npack 级扫描 ALL CLEAN
- Hermes 累计修复 xianding 11 武将（dc_shen_liubei/dc_sb_liuzhang/dongxu/
  zhangjian/leitong/shie/shen_xuzhu/dc_sp_machao/chengbing/dc_duyu/zerong）
- extensions md5 三份一致
