# ADR 0148：回合目标历史与限定装备赠予

## 决策

旧版 JavaScript 只作为技能语义证据，不进入内容包，也不在运行时执行。回合状态新增稳定标志
`CARD_TARGETED_OTHER`：当前回合角色使用的牌只要存在非自身目标就写入；暂停中的牌后来追加目标时
同样写入。`PhaseEnded.card_targeted_other` 以反序列化缺省 `false` 的字段携带该快照，声明式过滤器
`owner_alive_other_turn_ended_no_card_targeted_other` 组合拥有者存活、其他存活角色回合结束与历史为假，
核心不扫描旧事件对象，也不识别「窃听」。

新增 `choose_move_target_equipment_or_draw(player, target)`。通用查询只枚举目标装备区内、拥有者对应
装备栏仍启用的实体牌；同栏已占用仍是合法候选。第一阶段提供移动、摸一张或取消，移动分支第二阶段
强制选择实体装备；恢复时重新校验角色、来源技能、快照候选、当前牌区和栏位。提交时先把拥有者同栏
旧装备移入弃牌堆，再把所选装备移动到拥有者装备区，全部沿用普通语义移动事件。

主动技契约增加缺省关闭的 `equip_nonempty` 与 `once_per_game`。服务器在运行 VM 前权威校验装备区；
限定次数使用既有 `SkillInstance.lifetime_fire_count`，只在合法发动时消费且不会随回合或阶段重置。
新增 `give_all_equipment_choose_recover_or_damage(player, target)`：快照发动时全部装备及数量，通过
`GiveCards` 交给目标手牌并等待嵌套移动触发完成；随后接收者可以取消目标选择，使仍存活的拥有者由
接收者作为来源回复快照数量体力，或选择一至该数量名当前位于接收者攻击范围内的不同存活角色，按
提交顺序各造成一点普通伤害。攻击范围查询统一排除查询者自身，因此无需技能专用目标特判。

所有选择续体都保存运行时玩家、技能和实体 ID，并在恢复时原子重验；伪造牌、重复目标、自身目标、
范围外目标或已经变化的上下文都会拒绝且不提交部分状态。核心只实现通用历史、主动技契约、装备移动
和范围内多目标效果，不识别「窃听」「献州」、蔡夫人或 YIJIANG 包。

## 兼容性

`DomainEvent` 字段、`EffectRequest`、`ContinuationKind`、触发过滤器、Rule IR 语句和 VM host effect
tag 均以缺省字段或末尾追加方式扩展，保留既有序列化序号。Rule VM ABI 升至 58，Rule IR 升至 69，
Engine Semantics 升至 144；内容 schema 保持 3。旧包不获得新能力，仍按其原最低 ABI 加载。
稳定回合目标历史进入规范状态哈希，因此三组完整身份局 Native/WASM Golden 在本 ADR 中有意重钉；
空 bootstrap 开局不建立该回合标志，既有三组开局 Golden 保持不变。
