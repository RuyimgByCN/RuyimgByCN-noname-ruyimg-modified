# 欢乐卡牌（huanle）归档扩展迁移

- 来源：`archive/upstream/noname/apps/core/extension/欢乐卡牌`（GNU GPL v3）。
- 本包登记 5 张装备卡（猴子偷桃/面具/苦无/血轮眼/九尾），依赖 standard 包。

## 已发布（真实运行时）

- 卡牌定义：`monkey`（宝物）、`jiuwei`（宝物）、`xuelunyang`（宝物）、`mianju`
  （防具）、`kuwu`（武器，攻击范围 1、进攻距离 -1）。
- `mianju`（面具）：装备时锁定技，你不能被翻面（`turn_over_cancel`）。
- `jiuwei`（九尾）：装备时结束阶段可回复 1 点体力或摸一张牌（旧 JS 按已受伤自动
  选回复，DSL 以自选近似）。

## 元数据登记（无运行时行为）

- `monkey`（猴子偷桃）：他人使用【桃】时弃此装备取消之并获牌——需要整张牌使用
  取消原语。
- `kuwu`（苦无）：【杀】造成伤害后目标弃一张牌——需要伤害后目标弃牌原语；武器
  距离部分已由卡牌定义表达。
- `xuelunyang`（血轮眼）：选择其他角色并复制其一项技能——需要动态复制技能机制。

## 未迁移

- `shoulijian`（手里剑）：自定义基本牌（方块 4 / 黑桃 4），引擎卡牌种类仅支持
  装备（weapon/armor/horse/treasure），自定义基本牌效果无法表达，登记为不可迁移；
  其效果（目标弃装备或受 1 点伤害）保留于译文。
- 卡牌进牌堆：引擎牌堆为标准 119 张配置，自定义装备卡不进默认牌堆（与
  wandianlunsha 派生卡处理一致），仅注册定义供规则层引用。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/huanle/huanle_impl.skill \
  content/original/ported/huanle/huanle_meta.skill \
  --cards content/original/ported/huanle/cards.json \
  --metadata content/original/ported/huanle/metadata.json \
  --out content/original/ported/huanle/huanle.npack.json \
  --package-id huanle
```
