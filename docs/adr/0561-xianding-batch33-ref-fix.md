# ADR 0561：xianding characters.json 引用修复（batch33 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch33 为 D 段 opencode 领取，0560）

## 背景

全库 npack 级扫描发现 xianding 包 2 名武将引用失效 id（全库检索无）。batch33
在 22:00 定义 dcsbshijin 等（.skill），但 characters.json 引用未同步——与
batch27/28/29/32 遗留同模式（opencode 定义技能 → 重打 npack → 更新 README 后
转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| dc_sb_dengai（界邓艾） | dcsbzhouxi, dcsbshijin | 3927470045 | dcsbshijin=3506294264 |
| dc_sb_dongcheng（董成） | dcsbbaojia, dcsbdouwei | 2918478934 | dcsbdouwei=2217274823 |

各武将另一技能引用已正确（dcsbzhouxi=3028459952 / dcsbbaojia=3272096659）。

## 变更

1. `content/original/ported/xianding/characters.json`：2 武将第二技能引用修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（504 技能 /
   196 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- 全库 31 包 npack 级扫描 ALL CLEAN
- Hermes 累计修复 xianding 15 武将（dc_shen_liubei/dc_sb_liuzhang/dongxu/
  zhangjian/leitong/shie/shen_xuzhu/dc_sp_machao/chengbing/dc_duyu/zerong/
  dc_xujing/yuanji/dc_sb_dengai/dc_sb_dongcheng）+ diy 61 角色
- extensions md5 三份一致
