# ADR 0066：判定后排除当前【杀】目标

- 状态：Adopted
- 日期：2026-07-15

## 背景

旧版 `olshenhou` 在拥有者成为其他角色使用【杀】的目标时允许判定；红色结果把拥有者加入该次用牌事件的 `excluded` 集合。这个语义必须在可选触发造成的 Choice 暂停之后继续保留，并且多目标【杀】只能排除判定成功的目标。

## 决策

新增通用过滤器 `owner_targeted_by_other_sha` 与类型化效果 `judge_cancel_current_sha_target_on_red(player, card, target)`。效果执行标准判定；有效花色为红桃或方片时，只把 `target` 记录为当前解析中【杀】的排除目标。

排除集合复用既有 `resolution.resume_payload`：单目标【杀】在固定六项后追加排除目标，多目标【杀】在目标列表后追加排除目标。实体牌与视为【杀】共用恢复入口，恢复时过滤排除目标，保留其他目标的原顺序与响应流程。

## 结果

- 可选触发跨 Choice 暂停后仍能精确恢复用牌解析。
- 多目标【杀】的判定和排除彼此隔离。
- DSL、Rule VM 和引擎只增加通用能力，不按技能 ID 分支。
- continuation payload 与效果语义变化，`EngineSemanticsId` 递增到 50。
