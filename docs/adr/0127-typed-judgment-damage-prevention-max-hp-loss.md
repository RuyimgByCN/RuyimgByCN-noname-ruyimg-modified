# ADR 0127：类型化判定防伤与减体力上限

## 背景

旧「潜袭」订阅 `damageBegin2`：拥有者以距离不大于 1 的逻辑【杀】即将造成伤害时，可以判定；
非红桃则取消当前伤害，并令目标减 1 点体力上限。它同时依赖当前伤害身份、逻辑牌名、权威距离、
判定牌移动和伤害提交顺序，不能把旧 JavaScript 判定回调直接留到运行时执行。

## 决策

- 过滤器 `owner_source_sha_target_alive_distance_le_one` 只接受拥有者作为来源、目标仍存活、逻辑牌
  为【杀】且权威距离不大于 1 的 `damage_before`；触发保持可选。
- Rule VM ABI 37 追加
  `judge_prevent_damage_lose_max_hp_on_nonheart(player, target)`，host effect tag 为 55。host 将 VM 的
  零占位绑定到当前 `damage_id`，并拒绝在对应伤害前时机之外使用。
- 该原语翻开一张判定牌；红桃不改变伤害或体力上限，其他花色先标记当前伤害为已防止，再令目标
  减 1 点体力上限。两项状态变更处于同一事务，失败时共同回滚。
- 嵌套效果队列不得取得调用者已暂存的伤害尾队列。判定牌移动可以正常派发事件，但当前伤害只能
  在判定原语及其触发效果完成后由外层调度提交，避免“先掉血、后记录判定”的顺序倒置。

## 版本

- Rule VM ABI：37
- Rule IR：48
- Engine semantics：123
