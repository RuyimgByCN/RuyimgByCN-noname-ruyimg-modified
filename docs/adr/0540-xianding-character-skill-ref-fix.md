# ADR 0540：xianding characters.json 技能引用修复

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：xianding

## 背景

全包一致性终验（npack skill_index vs character_index 交叉核对）发现 xianding 包
characters.json 中 4 名武将引用了 **5 个不存在的技能 def_id**（全库检索均无）。
内容编译器 pack 时不校验 character→skill 引用，缺陷静默进入已发布 npack。

## 缺陷清单

| 武将 | 上游技能 | 错误引用 id | 修正为 |
|---|---|---|---|
| dc_shen_liubei | dclongnu, dcjieying | 3029944376 | dcjieying=3849335872 |
| dc_sb_liuzhang | dcsbjutu, dcsbrenshan, dcsbyizhi | 3234837342 | dcsbyizhi=3294916313 |
| dongxu | dcqingleng, dczhendu | 2872120335 | dcqingleng=2507120853 |
| zhangjian | dc_zj_a, dc_zj_b | 2238296709, 2540253792 | dc_zj_a=2792442500, dc_zj_b=3520470433 |
| leitong | kuiji | 3615171539 | kuiji=2248505393（后补：18:24 全库 npack 级扫描发现，随 ADR 0542 修复批次一并修正） |

错误 id 与正确 id 无数值关联（随机 u32 错位），判定为 characters.json 生成时
引用了占位/过期 id。

## 变更

1. `content/original/ported/xianding/characters.json`：4 武将 skills 引用修正
2. `content/original/ported/xianding/xianding.npack.json`：重打包（ABI 607 不变，
   504 技能 / 196 武将不变）
3. `extensions/xianding/{xianding,package}.npack.json`：同步（md5 三份一致）

## 验收

- npack character_index 全部技能引用可解析（未解析 = 0）
- skill_index 504、character_index 196 与修复前一致（仅引用修正，无内容增减）
- `md5 content/original/ported/xianding/xianding.npack.json extensions/xianding/xianding.npack.json extensions/xianding/package.npack.json` 一致

## 说明

diy 包存在同类现象（characters 引用 84 个 .skill 未定义 id），但经核实为 omp
ADR 0512 的**有意元数据登记设计**（README 声明"其余 ns_/junk_ 系技能为元数据
登记 + 文档化近似"），非缺陷，未改动。
