# ADR 0355：ABI 493 浮海私密选择与连续计数 + mobile 魏文诸葛直

手杀魏文诸葛直（`re_weiwenzhugezhi`，3812335636）仅有「浮海」
（`refuhai`，2797180006），以 `resolve_refuhai(player)` 完成武将闭包。

## 决策

- 保留旧 JavaScript 的实际执行顺序：从技能拥有者下家开始，其他存活角色依座次逐个
  私密选择「潮起」或「潮落」，所有人完成后用单个公开 `RuleNote` 发布结果。
- 结算只统计从下家开始的首段连续相同结果；人数大于一时摸等量牌。
- 复用 `luanqun` 的私密选择后统一公开模式，不增加持久技能状态；选择续体只保存参与者
  快照和已提交结果。

## 验证

- `original_mobile_refuhai.rs` 覆盖武将闭包、选择未完成前不公开，以及
  「潮起、潮起、潮落」最终摸两张。
- mobile npack 与 `extensions/mobile/` 两份发布包同步，catalog 标记武将和技能为
  `ported`；旧 JavaScript 不进入运行时。
