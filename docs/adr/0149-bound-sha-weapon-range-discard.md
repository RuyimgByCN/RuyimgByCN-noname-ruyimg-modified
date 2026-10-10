# ADR 0149：绑定【杀】伤害的武器范围弃牌

## 决策

旧版 JavaScript 只作为技能语义证据，不进入内容包，也不在运行时执行。新增通用触发过滤器
`owner_source_sha_has_weapon`：只匹配拥有者作为伤害来源、事件玩家为存活目标、直接由逻辑【杀】
产生且拥有者装备区存在实体武器的 `damage_before`。卡牌关联伤害增加 `CHAIN_PROPAGATED` 来源位，
铁索传导继承卡牌和基础伤害但写入该位，因此不再伪装成直接卡牌父级事件。

新增 `choose_target_discard_weapon_range_hand_or_increase_damage(player, target)`。效果绑定当前引擎
伤害 ID，以实体武器的 `weapon_range`（最低一）作为精确手牌支付数；目标手牌不足时直接把待提交
伤害和 `base_amount` 各加一。手牌足够时向目标开放可取消的精确选牌屏障，载荷保存玩家、目标、
伤害、来源技能、支付数和实体手牌快照。恢复时重新校验技能有效、双方存活、直接【杀】伤害、武器
与范围未变、选择不重复且仍属于快照和目标手牌。取消令伤害与传导基数加一；支付成功按所选手牌、
来源当前全部实体武器的顺序进入普通弃置效果队列，移动事件和嵌套触发仍在伤害提交前完成。

伤害数值降至零不再借用 `prevented_damage_ids`：待提交 `amount` 直接保存零并在提交时无效果结束；
该集合只表示明确的防止伤害。来源侧加伤因此可正确作用于先前的数值减伤，同时不会撤销显式防止。
核心只实现通用伤害绑定、来源标志和实体选择，不识别「倾袭」、曹休或 YIJIANG 包。

## 兼容性

`EffectRequest`、`ContinuationKind`、触发过滤器、Rule IR 语句、VM host effect tag 与来源标志均为
末尾追加或空闲位扩展，保留既有序列化序号。Rule VM ABI 升至 59，Rule IR 升至 70，Engine
Semantics 升至 145；内容 schema 保持 3。旧包不获得新能力，仍按原最低 ABI 加载。
