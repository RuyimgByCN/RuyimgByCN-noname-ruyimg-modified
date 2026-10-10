# ADR 0546：xianding characters.json 引用修复（batch27 遗留）

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding（batch27 为 D 段 opencode 领取，0543/0544）

## 背景

全库 npack 级扫描发现 xianding 包 3 名武将的**第二个技能引用失效 id**（全库检索无）。
batch27（施仇/壮魄/剑出）在 18:37 将 sejianchu/zhuangpo/twodcspshichou 从空壳占位
转为实现（.skill 定义 + 重打 npack），但 characters.json 的武将引用未同步，
随后 opencode 转去其他工作区（18:38 后 xianding 无修改，18+ 分钟停滞），
缺陷以暂态形式遗留。

## 缺陷清单

| 武将 | 上游技能 | 错误引用 | 修正为 |
|---|---|---|---|
| shie（施翳） | liren, sejianchu | 3355579768 | sejianchu=3620279118 |
| shen_xuzhu（神许褚） | zhengqing, zhuangpo | 3222350667 | zhuangpo=2849685408 |
| dc_sp_machao（SP 马超 twodc 版） | twodcspzhuiji, twodcspshichou | 2499124614 | twodcspshichou=3276628911 |

三个武将的**第一个技能引用已正确**（liren/zhengqing/twodcspzhuiji），仅第二技能
id 错位——与 leitong（ADR 0540 补记）同模式。

## 变更

1. `content/original/ported/xianding/characters.json`：3 武将第二技能引用修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（504 技能 /
   196 武将 / ABI 607 不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- 全库 31 包 npack 级扫描 ALL CLEAN（character→skill 引用全部可解析）
- xianding 504 技能 / 196 武将 / ABI 607 与修复前一致
- extensions md5 三份一致
