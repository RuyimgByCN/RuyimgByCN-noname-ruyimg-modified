# 0452 yijiang 批次三（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

本批新增 1 个技能（`zhongyong` 忠勇,关平）,纯 DSL 表达,复用 ABI 562 环境,
零新 ABI。

## 变更

| 技能 | DSL 表达 | 未建模/近似 |
|---|---|---|
| `zhongyong`（忠勇） | `trigger sha_dodged owner_source_sha_dodged optional` → `choose_other_gain_event_cards(self)` + `use_virtual_card_on(self, sha, event_player)` | 「交给目标不可为杀目标」约束未建模（choose_other_gain_event_cards 仅限其他角色）;「可对闪避者使用杀」以无条件虚拟杀近似（略强） |

这是 `sha_dodged` 事件与 `choose_other_gain_event_cards`（举荐管线）组合的首个
用例,验证了响应牌事件 → 事件牌转交 → 虚拟杀使用的完整链路。

## 落地

- yijiang 123 技能 / 27 武将 / ABI 562。
- `extensions/yijiang/` 同步（package.npack.json 同源,README 更新）。

## 备注

expansion（技能牌堆）链路的父技能（quanji/zyexin/chuyuan/xiansi 等）在 yijiang
均未移植,「触发存牌/取回/觉醒判定」原语需父技能齐备后才能闭环,已保留在
E 段缺口登记,后续批次处理。

## 验收

- content-cli pack 编译通过;zhongyong 触发索引正确（sha_dodged, optional）。
