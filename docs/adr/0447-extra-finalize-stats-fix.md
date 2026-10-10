# 0445 extra 收尾与统计口径修正（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

extra 神包此前被按「文件数」误判为「33/85 已移植」。实际 extra 目录含多技能文件
（如 `shen_simayi.skill` 一个文件定义 renjie/renjie2/sbaiyin/lianpo/jilue 及极略五子，
共 10 个技能），按技能名维度统计实际已定义 65 个，覆盖上游全部 63 个顶层技能。

## 变更

1. 以 `content/original/ported/extra/characters.json` + `cards.json` 全量重打包
   `extra.npack.json`：65 技能 / 12 神将 / 6 张衍生卡 / ABI 240；68 个模块 bytecode 与
   `extensions/extra` 既有包逐字节一致，角色索引一致，并补齐了旧包缺失的 6 张衍生卡
   （card_index 0 → 6）。
2. `extensions/extra/` 同步（package.npack.json 与 extra.npack.json 同源）。
3. 修正 ABI-COORDINATION.md 认领表统计口径：已定义 = `.skill` 文件内 `skill name(id)`
   定义数（多技能文件按技能名计），并同步修正 newjiang/clan/onlyOL/key/yijiang 数字。

## 未建模项（已注明）

缺失的 10 个上游 key 均为辅助/折叠项：longhun1-4（折叠进 longhun）、dawu2/3 与
kuangfeng2/3（临时效果，与 qixing 技能牌堆联动）、qixing2（七星与手牌交换）、
duorui_clear（清理标记）、g_drlt_zhiti / g_nzry_jieying（global 辅助）。其中 longhun1-4
已按 README 折叠进父技能，其余属「无独立规则行为 / 与父技能联动省略」范畴。

## 验收

- `content-cli pack` 编译通过；65 技能 / 12 角色 / 6 卡 / ABI 240。
- extensions/extra 与重打包产物同源。
- 零新 ABI。
