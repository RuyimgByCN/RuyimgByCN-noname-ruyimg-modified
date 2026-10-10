# ADR 0562：xianding characters.json 引用修复（batch34 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch34 为 D 段 opencode 领取）

## 背景

全库 npack 级扫描发现 xianding 包 2 名武将引用失效 id（全库检索无）。batch34
在 22:21 定义 dcduoyue/spzhuilie 等（.skill），但 characters.json 引用未同步——
与 batch27-33 遗留同模式（opencode 定义技能 → 重打 npack → 更新 README 后转走，
引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| v_caocao（曹操变体） | dcduoyue, dcjunhe, dcxiongwei | 2513963246 | dcduoyue=3729672764 |
| wangshuang（王双） | spzhuilie | 2196883215 | spzhuilie=3774269941 |

各武将其余技能引用已正确（dcjunhe=4081744783 / dcxiongwei=2688012572）。

## 变更

1. `content/original/ported/xianding/characters.json`：2 武将引用修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（506 技能 /
   196 武将 / ABI 607）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- 全库 31 包 npack 级扫描 ALL CLEAN
- Hermes 累计修复 xianding 17 武将（dc_shen_liubei/dc_sb_liuzhang/dongxu/
  zhangjian/leitong/shie/shen_xuzhu/dc_sp_machao/chengbing/dc_duyu/zerong/
  dc_xujing/yuanji/dc_sb_dengai/dc_sb_dongcheng/v_caocao/wangshuang）
  + diy 61 角色
- extensions md5 三份一致
