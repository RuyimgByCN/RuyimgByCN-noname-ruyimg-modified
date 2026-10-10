# ADR 0179：挂起伤害牌的通用基础伤害增量

## 状态

已接受（2026-07-23）。

## 背景

旧版「悍勇」在拥有者受伤时，可于使用【南蛮入侵】、【万箭齐发】或无属性黑桃【杀】时令
该牌的伤害值基数加一。ABI 99 的 `increase_current_sha_base_damage` 只认识被挂起的主动
【杀】，无法覆盖先经过 CardUsed 触发、无懈窗口和逐目标响应队列的两种 AOE。把增量保存为
全局临时状态会在嵌套用牌、Choice 暂停、断线恢复和回放中产生歧义。

## 决策

Rule VM ABI 104 追加 `increase_current_card_base_damage(player, card, amount)`：

1. 旧 ABI 99 效果与 tag 保持不变；新效果重新校验当前挂起结算的来源与实体牌，只接受主动
   【杀】、【南蛮入侵】和【万箭齐发】；
2. 【杀】继续复用既有保存伤害值；AOE 增量以带 magic 的追加字段写入当前无懈恢复载荷，并随
   CardUsed 可选触发、无懈链和响应队列确定性恢复；
3. AOE 开始逐目标结算时冻结同一基础伤害值，所有未成功响应的合法目标使用该值；
4. 新过滤器
   `owner_source_wounded_nanman_wanjian_or_normal_spade_sha` 使用逻辑牌类别、有效花色与有效伤害
   属性判断触发资格，避免物理牌/视为牌分歧；
5. 「悍勇」仍是可选触发；轮数条件通过现有 `round_number_capped` 与 `hp` 查询在已选择发动后
   判断是否增加「燃」标记。

## 后果

- 以后需要修改当前【杀】或 AOE 基础伤害的技能可复用同一挂起结算能力，不引入技能专用全局
  状态。
- Rule IR 升至 76，Engine semantics 升至 151；内容 schema、包格式、协议与快照 schema 不变。
- ABI 103 主机拒绝 ABI 104 包；ABI 104 主机继续加载 ABI 3–103 的加法式内容包。
- 旧 JavaScript 只作为语义合同，不进入 npack、回放、网络协议或运行时。
