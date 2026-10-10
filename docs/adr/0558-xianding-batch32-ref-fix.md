# ADR 0558：xianding characters.json 引用修复（batch32 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch32 为 D 段 opencode 领取，0556）

## 背景

全库 npack 级扫描发现 xianding 包 2 名武将引用失效 id（全库检索无）。batch32
（防毒/赏誉）在 21:32 定义 dcshangyu/dcfangdu 等（.skill），但 characters.json
引用未同步——与 batch27/28/29 遗留同模式（opencode 定义技能 → 重打 npack →
更新 README 后转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| dc_xujing（徐靖） | dcshangyu, dccaixia | 2518690530 | dcshangyu=3428760598 |
| yuanji（袁姬） | dcfangdu, dcjiexing | 2441770008 | dcfangdu=3648264331 |

各武将另一技能引用已正确（dccaixia=2929523968 / dcjiexing=4014891370）。

## 变更

1. `content/original/ported/xianding/characters.json`：2 武将第一技能引用修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（504 技能 /
   196 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- 全库 31 包 npack 级扫描 ALL CLEAN
- Hermes 累计修复 xianding 13 武将（dc_shen_liubei/dc_sb_liuzhang/dongxu/
  zhangjian/leitong/shie/shen_xuzhu/dc_sp_machao/chengbing/dc_duyu/zerong/
  dc_xujing/yuanji）+ diy 61 角色
- extensions md5 三份一致
