# ADR 0550：xianding characters.json 引用修复（batch28 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch28 为 D 段 opencode 领取，0547）

## 背景

全库 npack 级扫描发现 xianding 包武将 chengbing（程秉，wu 3 体力）的第二个技能
引用失效 id（3359025792，全库检索无）。batch28（恩遇）在 19:34 将 dcenyu 定义
（dcenyu.skill），但 characters.json 引用未同步——与 batch27 的 shie 等遗留
同模式（opencode 定义技能 → 重打 npack → 转走，引用未更新）。

## 缺陷

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| chengbing | dcjingzao, dcenyu | 3359025792 | dcenyu=3753328944 |

第一个技能引用已正确（dcjingzao=4291328887）。

## 变更

1. `content/original/ported/xianding/characters.json`：chengbing 第二技能引用修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（504 技能 /
   196 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- 全库 31 包 npack 级扫描 ALL CLEAN
- xianding 9 个 Hermes 修复武将（dc_shen_liubei/dc_sb_liuzhang/dongxu/zhangjian/
  leitong/shie/shen_xuzhu/dc_sp_machao/chengbing）引用全部正确
- extensions md5 三份一致

## 附注

§37.10 待清理记录（0549）已闭环：工作区 target/ 已在编译空闲时清理。
