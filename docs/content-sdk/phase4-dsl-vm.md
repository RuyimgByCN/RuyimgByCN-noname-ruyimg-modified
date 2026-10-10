# 阶段 4 / M0：DSL、Rule VM 与统一技能表面

> 历史参考（2026-10-10 标注）：本页是旧线 `engine-core` 的 DSL 与 Rule VM 手册。原生线 `engine-native` 不使用这套 DSL/VM，技能由 Rust 实现，扩展执行后端尚未选定（见[进度主稿](../architecture/native-progress.md#后续待做按阻塞关系) T8）。最新入口见[旧实现参考](../architecture/legacy-reference.md)。

## 统一执行模型

官方与社区技能默认编译到 **同一 Rule VM**。触发技（`trigger`）与主动技（`active`）共用：

- 同一套 `EffectRequest` / `QueryRequest`
- 同一字节码与 Host 约定
- 同一 content package 装载路径

Rust 只提供 Primitive Effect / Query、回合与区域机制、VM 宿主。

## DSL 示例

```text
skill jianxiong(1) {
  trigger damage owner_is_target {
    effect draw_cards(self, 1);
  }
}

skill fankui(5) {
  trigger damage owner_is_target {
    effect take_hand(event_source, self);
  }
}

skill biyue(4) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
  }
}

skill zhiheng(10) {
  active play_phase {
    effect discard_hand(self, 1);
    effect draw_cards(self, 1);
  }
}

skill mashu(2) {
  modifier distance_from add -1;
}

skill shenji(3) {
  modifier sha_target_limit add 2;
  modifier sha_use_limit add 1;
}

skill dcbenshi(4) locked {
  modifier attack_range add 1;
  modifier card_targets_all(sha) override 1;
}

skill qianju(2828017083) locked {
  modifier distance_from add -owner_lost_hp;
}

skill zhuiji(3152159217) {
  modifier distance_from_if_hp_gte_target override 1;
}

skill duanliang2(2158464053) {
  modifier card_distance_limit(bingliang) override 2;
}
```

### 已支持

| 类别 | 内容 |
|---|---|
| 控制流 | `if`/`else`、有界 `loop n`、`let` |
| 表达式（玩家） | `self` / `event_player` / `event_source` |
| 表达式（牌） | `event_card`（事件绑定的牌实例，如使用的牌） |
| 表达式（伤害值） | `event_amount`（仅 `damage` trigger；当前不可与 `choose()` 同用） |
| 表达式（其它） | `self_skill`、整数、`+ - * == <` |
| Effects | `draw_cards` `draw_each_by_seat` `draw_selected_targets` `draw_then_give_one` `draw_then_take_one` `take_one_card` `gain_event_cards` `choose_target_draw` `choose_target_draw_max_hp_cap_then_discard_excess` `give_selected_then_target_choose_discard_same_suit_or_lose_hp` `recover_hp` `recover_if_min_current_hp` `deal_damage` `lose_hp` `emit_note` `change_max_hp` `add_mark` `remove_mark` `discard_hand` `adjust_hand_to` `recast_selected` `disable_all_equip_slots` `take_hand` `add_skill` `remove_skill` `enter_dying` `reveal_card` `use_card` `use_card_on` `use_virtual_card_on` `move_card` `set_turn_state` |
| Queries | `distance` `attack_range` `hand_limit` `hp` `hand_count` `max_hp` `relation` `card_usable` `target_legal` `card_attribute` `skill_uses_remaining` `zone` `card_zone` `mark_count` `alive_count` `alive_group_count` `unique_max_current_hp` `unique_max_current_hp_at` `lifetime_damage_dealt` `card_action_count` |
| 触发事件 | `damage` `hp_lost` `card_used` `dying` `death` `phase_begin` `phase_end` `cards_drawn` `cards_moved` `duel_resolved` `equip_slot_disabled` `character_state_changed` |
| 主动窗口 | `play_phase` `any`（`CommandKind::ActivateSkill`）；`uses_per_turn N\|owner_hp` 声明固定或按当前体力读取的次数限制；`targets min max [other] [lowest_hp] [unlinked] [hp_at_least_owner] [can_compare] [can_use_on_owner card_kind] [second_can_use_juedou_on_first]` 声明权威玩家目标约束，其中 `can_compare` 要求发动者和每名目标当前均有手牌，`can_use_on_owner` 会按无来源虚拟牌的普通用牌、目标与次数规则校验，`second_can_use_juedou_on_first` 保留双目标顺序并在费用前校验第二名能否对第一名使用【决斗】；`select_hand\|select_he min max [same_color] [card_equip]` 把手牌或手牌/装备区输入交给 VM，并可约束同色或装备类别 |
| 视为用牌 | 实体来源：`view_as <kind> <window> <filter> [from hand_equip] [up_to 2] [same_suit] [nature fire] [multi_card_bonus damage_1\|recover_1\|discard_current_hand_equip] [uses_per_turn N] [ignore_distance] [red_unrespondable] [black_no_use_count];`。无来源动态牌族：`view_as basic\|ordinary_trick any any from none branch N once_per_round next_branch N cost gain_max_hp_1\|disable_equip_slot;`。二者均进入 `view_as_index`；动态牌族由调用方提交当前上下文中的具体牌名，核心统一校验普通牌目标、距离、次数、响应、无懈和濒死自救。filter 支持颜色、花色、具体基本牌、`card_basic`、`card_equip`、`non_basic`、`black_basic_equip` 及拥有者状态组合；多牌来源与附加结果、`ignore_distance` 和来源颜色策略只作用于该次实体牌转换。 |
| Trigger 元数据 | `trigger <event> <filter> optional { ... }` 会开确认 Choice；`skill <name>(id) locked` 禁止 optional trigger；`card_used` 每张牌触发一次并保留有序目标列表，`card_targeted` 按声明顺序为每个目标触发一次；`owner_is_source_or_target_other_alive_per_damage_point` 对来源、目标均存活的异源伤害按伤害点数独立触发；`owner_is_target_source_alive_gainable_card_per_damage_point` 按伤害点数检查存活来源的可获得牌，自伤仅检查装备区、他伤检查手牌区和装备区；`owner_is_target_once` 每回合只接受第一次匹配拥有者的事件；`owner_alive_event_player_outside_turn_from_hand_empty_once` 只接受每回合第一次实际发动的回合外失去最后手牌事件；`owner_alive_event_player_from_equip_empty` 只接受事件角色本次确实从装备区失去至少一张牌且结算后装备区为空的移动事件；`character_state_changed` 可用 `owner_alive_event_player_turn_changed_or_became_linked` 匹配双向翻面或进入横置状态，也可用 `owner_alive_event_player_became_face_up_or_linked` 只匹配翻至正面或进入横置状态，两者都排除解除横置；`owner_is_event_player_no_enabled_equip_slot_once_per_game` 只在拥有者首次废除最后一个装备栏时匹配，整局不重置；`owner_is_target_use_response_discarded_at_least_hp` 在拥有者回合结束时统计本回合所有角色因使用或打出而仍进入弃牌堆的实体牌，排除虚拟牌、装备成功入装备区及延时锦囊成功进入判定区的牌；参数化 `owner_is_target_hp_below_N` 在事件处理后按当前体力严格小于正整数阈值过滤；其余目标、牌类和状态过滤器按文档组合 |
| Modifier / Intent | 原有距离、范围、手牌、摸牌、杀伤害、响应牌数量与免疫通道；modifier 值支持整数、`owner_hp`、`owner_max_hp`、`owner_hp_minus_one`、`owner_hp_gt_two`、`owner_hp_le_two`、`owner_lost_hp`、`owner_lost_hp_minus_one`、`owner_lost_hp_min_one`、`owner_lost_hp_at_least_two`、`owner_all_others_in_attack_range`（其他存活角色均在攻击范围内时为 1）、`owner_sha_used_zero`（本回合尚未使用【杀】时为 1）、`owner_skill_mark`、`alive_group_count`、`alive_female_count`、`alive_other_count_capped_four`、`alive_wounded_count_zero_or_at_least_four`（无人受伤或至少四名存活角色受伤时为 1）、`alive_mounted_player_count_min_one`（至少装备一匹坐骑的存活角色数，最少为 1）、`owner_equip_count`（装备区牌数）、`owner_equip_count_odd` / `owner_equip_count_even`（装备区牌数奇偶）、`owner_no_mount`（装备区无坐骑时为 1，否则为 0）、`owner_has_treasure`（装备区有宝物时为 1）、`owner_has_heart_equip`（装备区存在有效红桃牌时为 1）、`owner_non_damage_hand_type_count`（非伤害手牌中基本、锦囊、装备类别的去重数）、`owner_turn_damage_dealt`（拥有者本回合造成的伤害点数）及 `owner_hand_empty`（手牌为空时为 1，否则为 0；动态值可加 `-` 前缀），并在每次 Query 时读取当前状态，`draw_count` 也支持这些查询期数值源；`card_suit(from) override to` 与 `card_name(selector) override name` 在不改写实体牌的前提下提供拥有者视角的有效花色和有效牌名，判定、出牌、响应、救援、目标合法性、触发、AI、无头跑局、视为技过滤和牌选择器按各自语义使用统一入口；`distance_from_if_hp_gte_target override 1` 在拥有者体力值不小于目标时把最终距离固定为 1，`hand_limit_base_max_hp` 可把手牌上限基数切到体力上限，`card_distance_unlimited` / `card_use_unlimited` 统一全牌距离与次数豁免，二者的 `(selector)` 形式可把豁免收窄到类别、花色或颜色，`card_distance_limit(selector) override N` 可设置某类牌的有限目标距离，`card_distance_unlimited_if_target_hand_gte_owner(selector)` 则逐目标比较手牌数后决定是否放开距离；`card_targets_all(selector)` 强制匹配牌覆盖全部合法目标，`card_targets_self_only` 禁止以其他角色为目标，`card_target_cancel(selector)` 令技能拥有者不能成为匹配牌的目标，并可结合 `owner_hand_empty` 动态启停；选择器除牌名族、类别、花色与颜色外，也支持 `black_trick`、`damage_trick`、`sha_fire` 和 `huogong` 等稳定组合；`card_target_cancel_from_current(any)` 禁止当前回合角色指定技能拥有者；`card_use_cancel(selector?)` / `card_response_cancel` / `card_save_cancel(tao?)` 分离主动用牌、响应与救援禁用，其中救援通道可只禁用【桃】，主动用牌可按牌名族、类别、花色或颜色限制，`hand_card_use_response_cancel` 只禁用手牌作为使用、响应或救援输入并保留非手牌响应，`sha_use_cancel` / `sha_response_cancel` 保留【杀】专用通道；`card_ignored_hand_limit(selector)` 按选择器统一“不计入手牌上限 + 弃牌阶段不可选”，`sha_ignored_hand_limit` 仅保留为兼容别名；`damage_amount` / `damage_nature` / `damage_cancel` 在 Mutation/Event 前修改或 veto typed Intent，目标侧 `damage_taken_cancel(normal|fire|thunder)` 则在伤害性质结算后否决拥有者将受到的对应伤害 |

目标感知修正另支持 `card_target_cancel_if_target_lacks_matching_suit`，以及带选择器的
`card_target_cancel_if_other_user_hand_gte_target_and_armor_empty(selector)`；两者在目标合法性查询时
读取当前手牌、来牌花色、使用者手牌数与防具区，不把动态条件固化进包或客户端。

`card_target_limit(selector) add N` 调整匹配牌可独立选择的最大目标数。实体牌与无来源转换牌共用
同一查询；【过河拆桥】【顺手牵羊】【火攻】【决斗】按目标顺序分别打开无懈窗口并结算，
【铁索连环】直接扩展其目标集合。全体锦囊和【借刀杀人】等成对目标牌不使用该修正器。

观察者投影可声明 `observer_foreign_hand_faces_visible override 1` 与
`observer_hidden_identities_visible override 1`。权限只授予技能拥有者及其私有回放导出；其他玩家、
公开旁观与公开回放仍不接收隐藏牌面或身份。客户端不能通过 DSL 扩大自己的观察者权限。

防具技能使用独立的可绕过通道：`armor_card_target_cancel(selector)` 与
`armor_damage_taken_amount(normal|fire|thunder)` 只在防具有效时参与目标合法性和伤害值计算；
来源拥有 `ignore_armor`，或目标拥有 `armor_effect_disabled` 时，两类防具修正统一失效。
`wanjian`、`sha_normal` 与 `sha_heart` 分别精确选择【万箭齐发】、无属性【杀】和红桃【杀】，避免扩大旧技能的牌类范围。

`relation(a, b)` 返回整数编码：`0=self / 1=ally / 2=enemy / 3=unknown`
（见 `rule_vm::bytecode::relation_code`）。

### 实体引用表达式（slice 4 已建机制）

DSL 现有一个牌类型操作数 `event_card`，经 `HostContext.event_card` →
`op::HOST_EVENT_CARD` 贯通，并由触发派发从 `TriggerContext.card` 注入（如 `card_used`
事件的使用牌）。类型检查区分玩家操作数与牌操作数：`reveal_card` 只接受牌操作数，
传入 `self` 之类会编译失败。

牌参数 query（`card_usable` / `target_legal` / `card_attribute`）对牌操作数做类型检查：
牌位置只接受牌操作数（如 `event_card`），传玩家会编译失败。

### choose{}（已接通，M0 slice5）

DSL 表达式 `choose(opt0, opt1, ...)` 单选：编译为 `HOST_CHOICE(count)`，VM 暂停并把
`Continuation`（frames/pc/fuel）+ 选项交回引擎；引擎存入 `ResolutionState.vm_cont_*`
（进 State Hash + 快照），开 `ChoiceRequest`；`ResolveChoice` 把所选 token 压回 VM 栈
并 `resume`，续跑的技能体可 `let x = choose(...)` 后按 `x` 分支。链式 choose 会再次暂停。
`ContinuationKind::VmResume` 承载该状态。示例：

```text
skill xuanze(60) {
  active play_phase {
    let pick = choose(1, 2);
    if pick == 1 { effect draw_cards(self, 2); }
    if pick == 2 { effect recover_hp(self, 1); }
  }
}
```

ABI 92 增加 `choose_range(min, max)`，复用同一 Choice 与 VM 续体，但上下界可由查询动态计算；
运行时生成非负闭区间内的全部整数令牌，最多 255 个。它用于「奇谋」这类必须按当前状态提供
`1..=hp(self)` 全量选项的技能，避免为单个旧 JS 技能增加专用 Effect。

ABI 93 增加 `discard_all_hand_equip_judge(player)` 与 `restore_character_state(player)`。
前者以强制弃置语义清空手牌、装备区和判定区；后者将存活角色复原为未横置、正面朝上，且不会
切换已经正常的状态。两者可组合表达旧版「涅槃」等“清空区域并复原武将牌”的限定技。

ABI 94 增加 `choose_discard_target_card_resolve_current_sha(source, card, target)`。它仅用于
`card_targeted` 触发：打开目标手牌/装备牌 Choice，并绑定当前实体【杀】与当前目标；弃置装备牌
后令当前目标不能响应，弃置其他牌后由目标获得仍在事件区域或弃牌堆中的实体【杀】。Choice 提交
时会重新验证三者，避免把旧 JS 的可变事件对象带入 npack。

### Trigger Choice 与调度续体

主动技和触发技都使用 `run_module`。触发 VM 暂停时，引擎把 VM 帧、当前触发标记、
尚未执行的 Effect FIFO 和已经冻结排序的 Trigger FIFO 一并写入 `ResolutionState`；这些字段
进入快照与 canonical state hash。`ResolveChoice` 续跑当前 VM，随后恢复剩余队列。
`optional` trigger 使用同一机制先开 Confirm Choice，拒绝只跳过当前触发，不会丢后续触发器。

逐目标过滤器 `owner_alive_sha_target_no_other_red_card_this_turn` 使用按内容需求启用的有界回合
历史：每名角色只保存“无 / 唯一实体牌 ID / 至少两张不同牌”，并在当前 `CardTargeted` 筛选完成
后登记本次红牌。因此它精确排除当前实体牌，又不会保存完整事件日志；使用者要求 ABI 29。

`move_equipment_then_draw(player)` 把装备迁移封装为可暂停的类型化效果：引擎依次请求装备牌和
合法接收者，在两个选择屏障后都重新校验；技能拥有者和原持有者均不能成为接收者，同栏旧装备
先被替换，最后原持有者摸一张牌。对应过滤器
`owner_is_target_has_movable_equipment` 可避免无合法移动时发出询问；两者要求 ABI 30。

`choose_use_virtual_sha_or_draw(player)` 对应旧扩展中常见的
`chooseUseTarget({name: "sha"}, false)`：虚拟【杀】遵守普通距离和目标合法性，但不校验或消耗
普通【杀】次数；取消、用牌被禁或无合法目标时摸一张牌。过滤器
`owner_alive_used_sha_this_turn` 使用按内容启用的每角色回合计数，只接受技能拥有者本回合实际
使用过逻辑【杀】的情况。两者要求 ABI 31。

`targets 2 2 equipment_pair_with_armor` 声明一个通用双目标装备弃置合同：两名不同的存活目标
各有可弃置装备，且至少一方有可弃置防具。配套效果
`discard_equipment_pair_with_armor(player)` 分两次强制选择并在每个屏障恢复时重检候选，最终从
两名目标各弃置一张装备且至少一张为防具。主动技模块可读取完整已选目标列表；两者要求 ABI 32。

`owner_source_black_sha_target_alive_has_discardable_equip` 把实体黑色逻辑【杀】、存活受伤目标和
统一弃置保护检查合并为伤害后过滤器。配套效果
`discard_target_equipment_gain_mount(player, target)` 打开一次强制装备选择，并在恢复时重检牌的
所有权、装备区位置与弃置保护；坐骑先独立进入弃牌堆，仅在仍处于弃牌堆时再进入选择者手牌。
两者要求 ABI 33。

ABI 34 将上述装备专用效果扩为类型化作用域
`discard_target_card_gain_mount(player, target, equip|hand_equip|hand_equip_judge)`。三个作用域都走
同一选择与续体重检实现；旧 ABI 33 host effect 仍按 `equip` 解码。过滤器
`owner_source_sha_target_alive_has_discardable_region_card` 接受任意逻辑【杀】造成的伤害，并要求
存活目标的手牌区、装备区或判定区至少有一张当前可弃置牌。这样相邻扩展技能只声明作用域，不再
复制隐藏手牌、装备保护、判定区和坐骑回收状态机。

ABI 35 增加 `make_current_sha_target_unrespondable(source, card, target)`。它只接受
`card_targeted` 时机，并把来源、实体牌和当前目标与保存的【杀】续体重新核对；单目标关闭本次
响应位，多目标使用显式的逐目标响应位段。旧续体没有该段时仍按全部可响应解析，取消目标列表与
逐目标响应列表互不混用。过滤器
`owner_source_sha_target_hand_gte_hp_or_lte_attack_range` 通过统一 `attack_range` 查询比较当前目标
手牌数，因此装备、扩展 modifier 与后续模式全局加成都沿同一查询链生效。

ABI 36 增加类型化 `sha_dodged` 事件与
`choose_discard_target_card(player, target, equip|hand_equip|hand_equip_judge)`。事件只在一张
【杀】所需的全部【闪】都结算后按目标发出一次；普通【万箭齐发】响应不会冒充该时机。配套过滤器
`owner_source_sha_dodged_target_alive_has_discardable_hand_equip` 和效果共享统一弃置保护查询，
并在强制选牌续体恢复时重新校验存活、所有权与区域；本效果不会回收被弃置的坐骑。

ABI 37 增加
`judge_prevent_damage_lose_max_hp_on_nonheart(player, target)`。它只允许用于 `damage_before`，host
把当前伤害 ID 绑定到类型化效果；红桃判定保持原伤害，非红桃在同一事务中防止该伤害并令目标
减 1 点体力上限。过滤器 `owner_source_sha_target_alive_distance_le_one` 使用逻辑【杀】与权威距离
查询。判定牌移动触发的嵌套效果队列不会提前消费外层伤害提交尾队列。

ABI 38 增加主动技前置合同 `hand_gt_max_hp` 和
`choose_gain_top_cards_by_suit(player, count, spade|heart|club|diamond)`。合同在权威命令入口比较
手牌数与体力上限；效果只向选择者投影牌堆顶牌面，允许取消或公开获得任意非空匹配花色子集，
并在续体恢复时重新校验完整牌堆顶切片、唯一实体牌及有效花色。未选牌不移动，保持原相对顺序。

ABI 39 增加
`choose_other_draw_by_hand_count(player, normal_count, empty_count, times)`。它依次打开 `times` 个
可独立取消的其他存活角色选择；恢复时重新校验目标，并按目标当时是否空手选择摸牌数。取消当前
一次不会取消剩余次数，摸牌继续携带来源技能。典型用法是把 `hp_recovered` 的 `event_amount`
直接作为 `times`。

ABI 40 增加
`choose_discard_one_then_gain_target_equipment(player, target, slot)`，其中 `slot` 为
`weapon|armor|horse_def|horse_off|treasure`。效果把弃牌选择本身作为可取消的发动成本；只有恢复时
重新验证并成功弃置一张自己的手牌或装备后，才重新读取目标对应装备槽，并把仍在该槽的全部装备
移入玩家手牌。成本、装备槽和两次移动处于同一可回滚事务。配套过滤器
`owner_is_target_sha_source_has_discardable_hand_equip` 精确要求逻辑【杀】伤害、已识别来源及当前可弃牌。

ABI 41 增加主动选牌过滤器 `card_sha`、参数化触发过滤器
`owner_is_target_active_used_at_least_N_this_play_phase`，以及类型化效果
`choose_target_draw(player, count, other|male_other|female_other|any)`。`card_sha` 使用拥有者视角下的
逻辑牌名；发动次数按每个出牌阶段独立计数，并保留到该阶段结束触发结算完成。目标域候选只包含
当前存活且满足范围的角色，双性角色同时匹配男性和女性范围；续体恢复时会重新校验，防止客户端
伪造目标。

ABI 42 增加 `owner_is_target_play_phase_card_category_count_positive`、
`draw_by_play_phase_card_category_count(player)` 和动态修正值
`owner_distinct_hand_card_suits_used_this_turn`。引擎以固定 4-bit/3-bit 掩码保存每名玩家本回合从
实体手牌使用过的有效花色，以及本出牌阶段使用过的逻辑基本/锦囊/装备类别；进入新出牌阶段只清
类别，进入新回合清两者。历史仅在加载 ABI 42 内容时启用，不为旧包改变零状态向量。

ABI 43 增加 `owner_alive_play_phase_sha_user_has_discardable_hand_equip` 和
`choose_discard_one_for_sha_refund_and_draw_if_red(player, sha_user, card)`。引擎在当前主动【杀】的既有续体中
保存该次用牌是否真实计数及是否已退款；来源玩家与实体牌必须和续体一致。同一张【杀】即使触发
多个技能实例也至多退款一次，忽略次数的【杀】不会消耗此前计数。组合效果在开始和选择续体处
两次校验可弃牌，仅在成功支付一张手牌/装备牌后排入退款及按出牌者视角有效红色摸牌；若结算到该
触发时已无可弃牌，则整项无事发生。

ABI 44 增加 `queue_opening_extra_play_phase(player)`。它只能由当前玩家的回合开始触发排入一个
有界（最多八段）的开端额外出牌阶段；host 绑定技能来源，运行时在 `TurnBegan` 触发全部结算后
统一决定先进入额外出牌阶段还是准备阶段。额外阶段结束后恢复到准备、判定、摸牌和正常出牌阶段，
不会递归产生新的回合开始事件。引擎同时把【杀】的回合总计与当前出牌阶段额度分开保存，使每个
额外/正常出牌阶段独立获得一次基础额度；ABI 44 未加载时沿用原生命周期和状态哈希。

ABI 45 增加 `convert_current_sha_to_fire_lose_hp_if_damage(player, card)` 和
`choose_add_current_sha_target(player, card)`。当前【杀】续体保存转换来源、完整目标队列和本次是否
实际造成伤害；追加目标只发布 `CardTargetAdded`，不会重复发布 `CardUsed`。转换后的【杀】仅在
全部目标与嵌套濒死流程完成后、且至少造成过一点有效伤害时令使用者失去一点体力。天然火【杀】
可复用追加目标而不会支付转换代价。

ABI 46 增加 `choose_store_hand_sha_in_skill_pile(player)` 和
`choose_spend_skill_pile_for_dying_jiu(player, dying)`。公共扩展牌堆按 `(owner, skill definition)`
定位，牌仍走普通实体牌区域与公开投影；选牌恢复时重新校验位置、逻辑牌名和濒死状态。消耗牌后
通过普通用牌管线令濒死角色对自己使用虚拟【酒】。濒死/回合开始的可选触发及其嵌套选择会先于
原生求救或回合推进续体完成。

ABI 47 增加主动目标合同 `can_compare`、
`choose_compare_selected_hand_card(player, target, win_skill_def, lose_skill_def)` 和
`choose_add_skill_bound_card_target(player, card)`。拼点双方分别提交并公开、弃置一张当前手牌，
恢复选择时重新校验所有权和位置；只有发动者点数严格较大才获得胜利技能，平局归入失败。
胜利技能以版本化实例状态绑定目标，并可用
`card_distance_unlimited_for_skill_bound_target`、
`sha_use_unlimited_for_skill_bound_target`、`ignore_armor_for_skill_bound_target` 三个通用 modifier
逐目标放开距离、【杀】次数和防具效果。追加目标复用当前【杀】、单目标普通锦囊或单目标
【铁索连环】的保存续体，并在选择恢复时按该牌的目标规则再次校验；不会复制用牌事件。

ABI 48 增加 `choose_discard_matching_equip_color_add_skill(player, target, skill_def)`、
`use_virtual_sha_from_card_color_lock_source_if_no_sha(player)` 和
`card_action_cancel_for_skill_color_mask`。可取消成本只接受与当前装备颜色相同的非基本手牌/装备，
选择恢复时重新校验所有权、区域、类别与颜色。辅助技能用版本化实例状态保存来源和红黑掩码；
重复授予合并颜色并以最后来源覆盖。颜色锁统一过滤实体牌的主动使用、响应与濒死救援。若阶段角色
本出牌阶段没有使用【杀】，记录来源在阶段末以无花色虚拟【杀】反击，复用普通结算但忽略距离和
普通次数限制，随后移除辅助技能。

ABI 49 增加 `choose_current_card_use_enhancements(player, card, helper_skill_def)`、
`owner_current_turn_card_use_count`、`ignore_armor_for_skill_card_policy`、
`card_use_unrespondable_for_skill_card_policy`、
`owner_source_single_target_sha_or_ordinary_trick_current_turn` 和
`owner_source_event_card_policy_draw`。效果只接受当前单目标【杀】或普通锦囊，强制选择一至两项
强化，并在每个选择屏障后重新校验；追加目标复用原牌目标规则。其余策略写入版本化 `CUP1`
技能实例状态并按精确 `CardId` 生效：无视防具在伤害准备阶段检查，不可响应同时覆盖【闪】与
普通锦囊的原始无懈窗口，每次实际伤害分别触发摸牌。拥有者本回合用牌数值来源在其他角色回合
恒为零。

ABI 50 增加 `phase_draw_end` 和
`choose_draw_or_discard_add_skill(player, draw_skill_def, discard_skill_def)`。摸牌阶段离开时先发出
独立结束事件；此时权威阶段状态已切换到出牌阶段，但出牌阶段开始事件尚未派发。可选触发被接受后，
效果强制选择“先获得摸牌辅助技能并摸一张”或“先获得弃牌辅助技能并弃置一张手牌/装备”；没有
合法弃牌时不提供后者。分支与弃牌成本共用一个带阶段标记的有界续体，并在两次恢复时重新校验
拥有者、原技能实例、阶段、卡牌所有权和区域。核心不识别「将驰」或任何具体技能定义 ID。

ABI 51 为 `cards_moved` 触发上下文增加完整、有序的实体牌列表，并增加
`choose_discarded_black_basic_as_bingliang(player)` 与
`choose_discard_judge_use_virtual_sha_draw_if_no_damage(player, target)`。前者只接受本批仍在弃牌堆的
有效黑色基本牌，逐张选择后通过普通延时锦囊管线无距离使用，并在每次恢复时重新校验牌区、属性、
目标和重复延时牌；后者在判定阶段选择并弃置目标判定区的一张牌，再通过普通【杀】响应与伤害管线
结算一张无来源虚拟【杀】。伤害结果按该次精确 `CardId` 登记，无法使用或整次未造成伤害时才摸一张。
`TargetCardScope::Judge` 复用统一弃牌保护和判定区查询。两种效果都只接收运行时注入的稳定技能 ID，
不识别技能名、武将或包。

ABI 52 增加主动技目标约束 `unused_by_skill_this_turn`、选牌约束
`unused_category_by_skill_this_turn`，以及
`discard_selected_record_unique_category_target(player, target)` 和
`choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(player, target)`。技能实例用版本化
`AUC1` 状态保存本回合已用的基本/锦囊/装备类别位和有序目标 ID；命令入口在展示候选与提交时都
校验，支付成本时再次原子记录。后续效果按该主动技本回合已发动次数，让拥有者选择目标摸牌或由
目标强制选择等量可弃手牌/装备；分支及嵌套移动触发全部结束后，再检查目标是否受伤并令拥有者
进入横置状态。回合开始只清理 `AUC1` 形状，不影响其他技能状态。

ABI 53 增加 `choose_reveal_other_hand_card_remember_category(player)`、
`draw_then_give_exact_record_phase_target(player, target, count)` 与
`clear_phase_gift_target(player)`。前者用版本化 `RCG1` 技能实例状态保存当前出牌阶段展示牌的
有效基本/锦囊/装备类别，每次进入新的出牌阶段先按状态形状清理；后两者在摸牌及嵌套触发完成后，
强制从当时仍属于拥有者的手牌/装备中精确交牌，并用 `PGT1` 保存本阶段目标。死亡与阶段结束过滤器
只比较运行时技能实例和事件角色，可组合出“目标本阶段有击杀则清除，否则阶段末执行后果”，核心
不识别具体技能名或武将。

ABI 54 增加 `sha_resolved` 事件、完整有序的原目标/实体响应牌上下文，以及
`choose_gain_resolved_sha_or_shan_then_optional_use_sha(player, sha_card)`。引擎在整次【杀】及其嵌套
结算结束后才发出事件；赠牌组、接收者与红牌后续实体【杀】在每个选择屏障重新校验。后续用【杀】
沿用普通目标禁用与多目标上限，但不消耗普通次数，距离改由技能拥有者的攻击范围判定，且不能以该
拥有者为目标。核心只处理事件上下文与运行时 ID，不识别「忠勇」或武将。

ABI 55 增加 `skip_current_phase(player)` 与
`choose_other_targets_by_skipped_phase_count_damage(player, amount)`。前者只允许当前阶段角色在摸牌、
出牌或弃牌阶段跳过当前阶段，并通过统一阶段机继续推进；所有规则来源的跳阶段都会追加
`PhaseSkipped` 事件并累加回合级规范计数。摸牌阶段入口触发若暂停，阶段机会保存通用续体：拒绝技能
后恢复普通摸牌，接受跳过后恢复时保持幂等。后者只在拥有者的结束阶段按该计数开放一至 X 名其他
存活角色的强制选择，并在选择屏障后重新校验数量、去重、存活、排除自身、阶段与来源技能，再按
提交顺序逐一造成伤害。并列最大手牌、最大当前体力与最大非空装备数由声明式过滤器计算；核心不
识别「奋励」「平寇」或武将。

ABI 56 增加弃牌移动过滤器 `owner_discarded_has_hand_once_per_phase` 与
`reveal_uniform_hand_draw_self_choose_others(player)`。过滤器要求拥有者从手牌区或装备区因语义化弃置
失去牌、结算后仍有手牌且本阶段尚未接受过该触发；拒绝可选触发不消耗次数，真实阶段迁移统一重置。
效果公开展示当前全部手牌，只有有效颜色严格同为红色或同为黑色时才继续；拥有者固定摸一张，并可
取消或选择至多展示时手牌数名其他存活角色各摸一张。选择屏障重新校验来源技能、阶段、数量、去重、
存活和排除自身，参与者按绝对座次摸牌。核心只处理语义移动、运行时 ID 和通用选择上下文，不识别
「秉壹」、顾雍或包。

ABI 57 增加 `choose_card_user_or_add_target(player, card)` 与
`choose_discard_equipment_reduce_current_damage(player)`。前者用于暂停中的单目标【杀】/普通锦囊：
技能拥有者选择另一合法角色，该角色可交一张手牌并在移动触发完成后接管用牌者，或不交牌并成为
额外目标；选择恢复时验证来源技能、阶段、次数、暂停牌和目标，用牌者变化与追加目标分别发出
`CardUserChanged`、`CardTargetAdded`。后者仅用于 `damage_before`，从手牌/装备中强制选择一张有效
装备牌弃置并把绑定伤害减一，伤害 ID 与成本实体在屏障后重新校验。配套过滤器覆盖当前出牌阶段限次
的单目标【杀】/黑色普通锦囊，以及男性来源伤害与可弃有效装备牌。核心不识别具体技能、武将或包。

ABI 58 增加 `choose_move_target_equipment_or_draw(player, target)` 与
`give_all_equipment_choose_recover_or_damage(player, target)`，并为主动技增加 `equip_nonempty`、
`once_per_game` 声明式契约。前者根据目标当前装备和拥有者已启用的对应栏位提供移动/摸牌/取消分支，
移动时允许按普通装备规则替换同栏旧牌，并在每个选择屏障重验角色、技能和实体牌。后者快照并交出
拥有者当前全部装备，再由接收者取消以令拥有者回复等量体力，或选择其攻击范围内至多等量名不同
角色并按提交顺序各造成一点伤害。回合历史通过 `PhaseEnded.card_targeted_other` 和通用过滤器判定，
主动限定次数写入技能实例的比赛生命周期计数；核心不识别「窃听」「献州」、蔡夫人或包。

ABI 59 增加 `choose_target_discard_weapon_range_hand_or_increase_damage(player, target)` 与
`owner_source_sha_has_weapon`。过滤器仅匹配拥有者作为来源、直接由逻辑【杀】产生且拥有者装备
实体武器的伤害前事件；铁索传导段带独立来源标志而不匹配。效果以实体武器的 `weapon_range`
要求目标从手牌精确支付；手牌不足或取消则令绑定伤害及属性传导基数 +1，支付成功则按手牌、来源
当前武器的顺序进入普通弃置管线。所有选择恢复均重新校验伤害 ID、技能、武器、范围与手牌快照。

ABI 60 增加 `choose_hand_count_different_target_draw_or_random_discard(player)` 与
`owner_source_or_target_hand_count_differs_once_per_turn`。过滤器仅匹配拥有者作为伤害来源或目标、
本回合尚未接受该技能且存在手牌数不同的存活角色。接受可选触发后，效果要求从快照候选中选择一名
实时手牌数仍不同的角色；其手牌较少则摸一张，较多则由权威 `RuleRng` 选定并弃置一张实体手牌。
恢复时重新校验观察者、技能、双方存活、候选快照与手牌差异，非法选择不推进 RNG 或局面。

ABI 61 增加 `increase_current_draw_count(player, count)` 与通用 `owner_source` 过滤器。前者只允许在
`phase_draw` 触发器中使用，把非负增加量累加到当前存活摸牌阶段角色的待消费摸牌数；阶段机在实际
摸牌或进入摸牌替代选择时读取并清零，保证拒绝触发不写入、接受触发只消费一次，普通摸牌仍形成
一个 `CardsDrawn` 批次。后者只比较运行时技能拥有者与当前伤害来源，可和 `event_player == self`
分支、`prevent_current_damage()` 以及回合末移除组合出通用的“本回合不能伤害其他角色”临时效果。

ABI 62 增加 `choose_target_discard_excess_then_self_discard_or_target_draw(player)` 与
`owner_is_target_any_alive_hand_nonempty`。效果只允许在 `phase_prepare` 触发器中使用，以可序列化的
三阶段 continuation 选择目标、目标牌和可选的自身弃牌成本；目标弃牌产生的嵌套移动触发先完成，
再开放支付/摸牌分支。每个恢复点都会重新验证当前角色、准备阶段、来源技能、候选快照、实体牌区、
数量与去重，取消或成本不足时由原目标摸牌。

ABI 63 增加 `draw_then_choose_gain_one_from_damage_target_and_neighbors(player, target)`。
效果只允许在 `damage` 触发器中使用：先摸一张并等待其嵌套触发全部结束，再按受伤角色、其上家、
其下家的旧版顺序逐项选择并获得一张牌。二人局上下家重复项保留；序列遇到拥有者时只开放装备区。
每次移动完成且嵌套移动触发结束后才继续下一项，恢复点重验技能、存活状态、顺序快照、牌区与选项。

ABI 64 增加 `choose_prevent_damage_or_lose_max_hp_increase_damage(player, amount)`。
效果只允许在 `damage_before` 触发器中使用，并要求 `player` 是当前绑定伤害的来源。它开放不可取消的
强制二选一：防止同一次伤害，或先令来源失去 1 点体力上限，待体力上限变化及其嵌套触发全部完成后，
再将指定数值同时加到绑定伤害的当前值与传导基数。恢复点重验观察者、选择值、伤害 ID 与来源。

**禁止**：文件、网络、时钟、DOM、任意导入、裸指针、为单技能开 Rust 旁路。

## 编译流水线

```text
DSL → Lex → Parse → Type/Capability/Termination Check
    → Rule IR → Bytecode → Verify
    → Trigger / Modifier / Active Index → ContentPackage
```

当前内容 schema 为 v3、Rule IR 为 v129、Engine semantics 为
v255（label `noname-next/semantics/v1`）；Rule VM ABI 本页原写 v651，已过时，以 `crates/engine-api/src/version.rs` 的 `RULE_VM_ABI_VERSION` 为准（2026-10-10 为 734，账面未闭合项见[进度主稿](../architecture/native-progress.md#后续待做按阻塞关系) T15），`COMPILER_VERSION` 为
`0.5.0-active-selection`。模块 capability mask 由实际 opcode
推导；包 manifest 聚合模块所需权限，不再默认申请全部权限。相同编译器版本 + 源码 →
相同 `logic_hash`。

## Rule VM Host 约定

| effect_tag | 栈（底→顶） | EffectRequest |
|---|---|---|
| DRAW_CARDS | player, count | DrawCards |
| DEAL_DAMAGE | source, target, amount | DealDamage |
| RECOVER_HP | player, amount | RecoverHp |
| EMIT_NOTE | code | EmitRuleEvent |
| CHANGE_MAX_HP | player, delta | ChangeMaxHp |
| ADD_MARK | player, mark, delta | AddMark |
| REMOVE_MARK | player, mark, amount | RemoveMark |
| DISCARD_HAND | player, count | DiscardFromHand（手牌区顺序前 N 张） |
| ADJUST_HAND_TO | player, count | AdjustHandTo（不足则摸牌，超出则强制精确选择手牌弃置） |
| TAKE_HAND | from, to | TakeFromHand（from 手牌第一张 → to） |
| REMOVE_SKILL | skill_instance | RemoveSkill |
| MOVE_CARD | card, to_zone, from_zone_or_zero | MoveCards（单牌；可组合 `zone` / `card_zone`） |
| SET_TURN_STATE | current, phase_tag | SetTurnState |
| USE_CARD_ON | player, card, target | UseCard（单目标） |
| DISCARD_SELECTED | player | DiscardCards（主动技声明选中的手牌） |
| DRAW_SELECTED | player | DrawCards（数量等于主动技选牌数） |
| DRAW_EACH_BY_SEAT | first, second, count | DrawEachBySeat（两名角色按座次各摸等量牌） |
| DRAW_THEN_GIVE_ONE | player, target, count, drawn_only | DrawThenGiveOne（摸牌触发结算后强制交一张牌） |
| RECAST_SELECTED | player | RecastCards（本次主动技选牌进入弃牌堆并补摸等量牌） |
| CHOOSE_TARGET_DRAW | player, count, scope | ChooseTargetDraw（按类型化目标域选一名角色摸牌） |
| DRAW_BY_PLAY_PHASE_CARD_CATEGORY_COUNT | player | DrawByPlayPhaseCardCategoryCount（按本出牌阶段逻辑牌类别数摸牌） |
| CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED | player, sha_user, card | ChooseDiscardOneForShaRefundAndDrawIfRed（成功弃一张牌后，当前杀至多退款一次，红色则摸牌） |
| QUEUE_OPENING_EXTRA_PLAY_PHASE | player | QueueOpeningExtraPlayPhase（在当前回合开始窗口排入一个独立的额外出牌阶段） |
| CONVERT_CURRENT_SHA_TO_FIRE_LOSE_HP_IF_DAMAGE | player, card | ConvertCurrentShaToFireLoseHpIfDamage（绑定当前【杀】并登记整次结算后的伤害代价） |
| CHOOSE_ADD_CURRENT_SHA_TARGET | player, card | ChooseAddCurrentShaTarget（为当前【杀】选择一个仍合法的新目标） |
| CHOOSE_STORE_HAND_SHA_IN_SKILL_PILE | player | ChooseStoreHandShaInSkillPile（选择一至多张手牌逻辑【杀】存入本技能公开牌堆） |
| CHOOSE_SPEND_SKILL_PILE_FOR_DYING_JIU | player, dying | ChooseSpendSkillPileForDyingJiu（消耗一张本技能牌堆牌并令濒死角色使用虚拟【酒】） |
| DISABLE_ALL_EQUIP_SLOTS | player | DisableEquipSlots（废除当前仍启用的装备栏并弃置栏内装备） |
| CHOOSE_GAIN_TOP_CARDS_BY_SUIT | player, count, suit | ChooseGainTopCardsBySuit（私看牌堆顶并按花色任选获得） |
| CHOOSE_DRAW_OR_DISCARD_ADD_SKILL | player, draw_skill_def, discard_skill_def | ChooseDrawOrDiscardAddSkill（摸牌阶段结束时选择摸牌或支付一张手牌/装备成本，并授予对应临时技能） |
| CHOOSE_DISCARDED_BLACK_BASIC_AS_BINGLIANG | player | ChooseDiscardedBlackBasicAsBingliang（从当前移动事件批次中逐张把仍在弃牌堆的黑色基本牌无距离当【兵粮寸断】使用） |
| CHOOSE_DISCARD_JUDGE_USE_VIRTUAL_SHA_DRAW_IF_NO_DAMAGE | player, target | ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage（弃置目标判定区一张牌，使用虚拟【杀】，该次未造成伤害则摸牌） |
| DISCARD_SELECTED_RECORD_UNIQUE_CATEGORY_TARGET | player, target | DiscardSelectedRecordUniqueCategoryTarget（记录本回合唯一逻辑类别和目标，再弃置主动技所选牌） |
| CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_LINK_IF_WOUNDED | player, target | ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded（按本回合发动次数令目标摸或弃，随后按受伤状态横置拥有者） |
| CHOOSE_REVEAL_OTHER_HAND_CARD_REMEMBER_CATEGORY | player | ChooseRevealOtherHandCardRememberCategory（选择并展示他人手牌，记录当前出牌阶段逻辑类别） |
| DRAW_THEN_GIVE_EXACT_RECORD_PHASE_TARGET | player, target, count | DrawThenGiveExactRecordPhaseTarget（摸牌后精确交牌并记录阶段目标） |
| CLEAR_PHASE_GIFT_TARGET | player | ClearPhaseGiftTarget（清除当前技能实例的阶段赠牌目标） |
| CHOOSE_GAIN_RESOLVED_SHA_OR_SHAN_THEN_OPTIONAL_USE_SHA | player, sha_card | ChooseGainResolvedShaOrShanThenOptionalUseSha（选择合法接收者及整组实体【杀】或【闪】；若含红牌，再提示接收者使用一张实体或实体转化的【杀】） |
| SKIP_CURRENT_PHASE | player | SkipCurrentPhase（跳过当前角色正在进行的摸牌、出牌或弃牌阶段，并写入规范跳阶段历史） |
| CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE | player, amount | ChooseOtherTargetsBySkippedPhaseCountDamage（按本回合跳阶段数选择其他角色，并按提交顺序逐一造成伤害） |
| REVEAL_UNIFORM_HAND_DRAW_SELF_CHOOSE_OTHERS | player | RevealUniformHandDrawSelfChooseOthers（公开全部手牌；同色时自己与可选的其他角色按座次各摸一张） |
| CHOOSE_CARD_USER_OR_ADD_TARGET | player, card | ChooseCardUserOrAddTarget（交手牌后接管暂停用牌者，或成为额外目标） |
| CHOOSE_DISCARD_EQUIPMENT_REDUCE_CURRENT_DAMAGE | player | ChooseDiscardEquipmentReduceCurrentDamage（弃置有效装备牌并令绑定伤害减一） |
| INCREASE_CURRENT_DRAW_COUNT | player, count | IncreaseCurrentDrawCount（累加当前摸牌阶段待消费的摸牌数） |
| CHOOSE_TARGET_DISCARD_EXCESS_THEN_SELF_DISCARD_OR_TARGET_DRAW | player | ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw（准备阶段三段式目标弃牌与支付/摸牌分支） |
| DRAW_THEN_CHOOSE_GAIN_ONE_FROM_DAMAGE_TARGET_AND_NEIGHBORS | player, target | DrawThenChooseGainOneFromDamageTargetAndNeighbors（摸牌后按受伤者、上家、下家顺序逐项获得牌） |

`phase_end`：进入 `TurnPhaseTag::End` 时由 `TurnStateChanged` 映射为 `EventKind::PhaseEnd`（闭月等）。

## 包格式

`ContentPackage`：`manifest` + `modules[]` + `skill_index`（name/locked）+
`character_index` + `trigger_index`（含 optional）+ `modifier_index` + `active_index`
（含可选选牌契约）+ `view_as_index` + `compiler_version`。

## 测试

```bash
cargo test -p content-compiler
cargo test -p engine-core --test phase_m0_unified_skills
cargo test --workspace
```

## View-as

`view_as sha play_phase red;` 经 `ActivateSkill` + `card_targets` 出杀。
Filter: red/black/any/card_sha/card_shan/suits。

## 标准技能的通用策略接线

标准技能已不再通过 `skill_def` 分支选择 Rust 逻辑。无双组合 `response_count` 与
`duel_opponent_response_count`；克己、裸衣、观星和鬼才分别声明为
`skip_discard_if_no_sha`、`draw_replacement_damage`、
`prepare_scry_count`、`judgment_rewrite` modifier；界鬼才等可从手牌或装备牌改判的技能使用
`judgment_rewrite_hand_equip`。制衡使用 `select_hand` +
`discard_selected` / `draw_selected` 进入 VM。Rust 只解释这些可被任意内容包复用的规则通道
和 Choice continuation。

改判窗口按判定角色起始的座次顺序依次询问全部仍可改判的角色；前一名角色改判或放弃后，
后续角色仍可继续改判。候选牌会同时经过通用用牌与响应禁用查询。

剩余 M1 债务是逐技能语义精化与 Golden 覆盖（例如观星的完整上下牌自由排序），而不是
再引入标准技能 ID 旁路。画像发现新缺口时，仍须先补通用 Trigger / Intent / Effect / Query。

摸牌阶段的四牌二上二下整理使用 `draw_phase_scry_four_split_two`；动态阶段阈值可用
`alive_wounded_count_at_least_N` 配合 `max` 模式驱动该通道、`skip_judge_phase` 或
`skip_discard_phase`。装备牌进入或离开自己的装备区且至少两人受伤时，可使用
`cards_moved owner_equip_zone_changed_alive_wounded_at_least_two`，同一装备替换的离开与进入
按两个 `CardsMoved` 事件分别结算。

ABI 83 为已有的类型化选人摸牌效果追加 `min_hand` 目标域：
`choose_target_draw(player, count, min_hand)`。候选是所有并列最少手牌的存活角色，包含发动者本人；
选择提交时按最新手牌数重新校验，因此等待确认期间的嵌套摸牌或失牌不会留下过期候选。配套
`owner_alive_targeted_by_other` 只匹配其他角色用牌产生的逐目标 `card_targeted` 事件。

ABI 84 增加 `owner_is_target_per_damage_point`，为每一点实际伤害各生成一次独立触发，不要求伤害
来源存在或与目标不同。`choose_target_draw_to_max_hp_cap(player, cap)` 强制选择任意存活角色，
将其手牌补至 `min(max_hp, cap)`；目标和补牌数会在选择恢复时重新计算，已经达到上限的目标仍可
作为合法的无操作选择。

ABI 85 增加 `owner_is_target_other_alive`，只在拥有者是事件角色且存在其他存活角色时触发。
`choose_other_draw_owner_lost_hp_turn_over(player)` 强制选择一名其他存活角色，在选择恢复时按拥有者
最新的 `max_hp - hp` 计算摸牌数，先完成摸牌及其嵌套触发，再将目标翻面；候选失效会拒绝提交。

ABI 86 增加 `judgment_resolved`。它在所有改判窗口结束后只触发一次，`event_player` 是判定角色，
`event_card` 是延时锦囊或装备原因牌，最终实体判定牌位于事件牌列表首位。参数化过滤器
`owner_alive_other_judgment_black_event_player_group_<group>` 同时约束其他存活判定角色、势力与最终
黑色结果；改判 continuation 会随每次替换更新实体结果牌。延时锦囊在该触发队列结束后继续结算。

ABI 87 增加 `virtual_bagua_if_armor_slot_empty` modifier。非零值只在拥有者存活、防具栏可用且
没有实体防具时提供虚拟【八卦阵】响应；实体防具占用或防具栏废除后自动失效。虚拟与实体八卦
复用同一判定、改判和响应 continuation，虚拟判定的 `JudgmentResolved.cause_card` 为 `NONE`。

ABI 88 增加 `owner_alive_other_damage_source_group_<group>`，用于匹配另一名指定势力的存活伤害来源；
可选触发可配合 `by event_source` 让伤害来源确认。`judge_recover_other_on_suit(judge_player,
recover_player, suit, amount)` 为第一名角色执行判定，并只在最终有效花色匹配时回复第二名角色；判定会
照常经过改判并产生 `JudgmentResolved`，独立技能判定的 `cause_card` 为 `NONE`。

ABI 89 增加 `sha_response_count_if_other_female` modifier：技能拥有者作为【杀】使用者时检查当前
目标，作为当前目标时检查使用者；对方为女性或双性角色才修改所需【闪】数。多目标【杀】在进入每个
目标前重新查询。`owner_is_target_not_min_hp` 只匹配体力值不处于全场并列最低的存活事件角色。

ABI 90 增加 `force_others_use_sha_on_nearest_or_lose_hp(player)`：激活时按顺时针座次快照其他存活
角色，逐人选择实体或出牌阶段转化【杀】，目标只能是最终距离并列最小且满足普通【杀】规则的角色；
取消或无合法使用时失去一点体力。该【杀】不消耗普通次数，响应、触发和濒死结束后恢复后续队列。

ABI 91 增加 `tao_use_save_cancel_for_non_dying_others_during_owner_turn` modifier：拥有者的回合内，
其他非濒死角色不能主动使用【桃】或以【桃】救援。引擎的濒死流程从当前回合角色起按座次询问所有
合法救援者；每张【桃】/自救【酒】只回复一点，同一救援者可连续救援，全部放弃后才结算死亡。

ABI 92 增加 `HOST_CHOICE_RANGE` 与 DSL `choose_range(min, max)`：动态计算非负整数闭区间，
暂停时把完整选项和 VM 续体交给普通 Choice 屏障，恢复后把所选整数压回表达式栈。

ABI 93 增加两个稳定 HOST_EFFECT 标签：`DISCARD_ALL_HAND_EQUIP_JUDGE` 强制移动指定角色
手牌、装备区和判定区的全部实体牌至弃牌堆；`RESTORE_CHARACTER_STATE` 解除横置并仅在角色
背面朝上时翻回正面。两种状态变化继续发出普通领域事件并进入确定性哈希。

ABI 94 增加稳定 HOST_EFFECT 标签
`CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA`。宿主复用目标牌选择、当前【杀】续体、
弃牌保护、仅当前目标不可响应与事件实体牌获得原语；恢复阶段再次校验选择合法性，所有分支继续
进入普通领域事件、快照和确定性哈希。

ABI 95 增加稳定 HOST_EFFECT 标签 `USE_VIRTUAL_CARD_ON_SELECTED_TARGETS`。它只允许出现在声明
至少一个其他角色目标的主动技中，VM 将命令层已经验证的完整目标列表复制进类型化效果。当前卡牌
种类收窄为【决斗】：宿主生成一张无花色虚拟牌，主动技目标契约替代普通单目标数量上限，其余
目标策略、无懈、逐目标决斗、领域事件、快照和确定性哈希全部复用普通牌流程。

ABI 96 增加稳定 HOST_EFFECT 标签 `DEAL_DAMAGE_BAN_ACTIVE_IF_DYING`，以及主动选牌过滤器
`card_basic`。前者只允许用于主动技，并要求第四个参数为 `self_skill`；宿主保存带技能实例的
预提交伤害，使它可以跨越伤害前触发和 Choice 屏障。只有该提交实际发出目标的
`EnteredDying` 时才写入回合内主动禁用状态；下一回合开始统一清除。普通伤害修正、濒死救援、
领域事件、快照和确定性哈希继续复用现有流程。

ABI 97 增加主动目标约束 `has_equip` 与稳定 HOST_EFFECT 标签
`CHOOSE_TARGET_RECLAIM_EQUIPMENT_DRAW_IF_RANGE_REDUCED`。编译器要求该效果只用于
`targets 1 1 other has_equip` 的主动技；命令层先验证目标装备区非空，宿主随后打开不可取消的
装备牌 Choice。恢复时重新校验技能实例、目标和实体牌，把所选装备移入目标自己的手牌，并在普通
移牌事件及其嵌套触发完成后比较目标攻击范围内的存活角色数；只有数量减少时技能拥有者摸一张。
选择、提交、延后比较三段请求均进入序列化续体与确定性哈希。

ABI 98 增加 `phase_any_end` 事件、两个按需启用的阶段历史过滤器
`owner_alive_phase_targeted_hp_changed_can_use_wanjian` /
`owner_alive_phase_targeted_hp_unchanged_can_use_juedou`，以及稳定 HOST_EFFECT 标签
`CHOOSE_USE_VIRTUAL_CARD`。阶段历史用定长玩家位图保存“成为过用牌目标”和“当前体力实际变化”，
不保存事件对象；`CardTargetAdded` 同样计入，单纯改变体力上限但未裁剪当前体力不计入。编译器只
允许 `choose_use_virtual_card(self, wanjian|juedou, self_skill)` 出现在 `phase_any_end` 触发器。
宿主生成无花色、无来源实体的虚拟牌：【万箭齐发】立即取得完整合法目标集，【决斗】打开不可取消
的合法目标 Choice；两者随后都复用普通目标、无懈、响应、伤害、弃置、快照和确定性哈希流程。
阶段历史在结束事件快照后清理，内部延后尾请求确保这次阶段末用牌的嵌套结算不会进入下一阶段历史。

ABI 99 增加逻辑用牌类型过滤器 `owner_source_basic` / `owner_source_non_basic`，以及稳定
HOST_EFFECT 标签 `INCREASE_CURRENT_SHA_BASE_DAMAGE`。效果接收当前用牌者、实体牌与非负增量，
重新校验它们属于当前保存的主动【杀】续体，再以饱和加法只更新该次用牌的基础伤害；单目标和
多目标续体共享同一基础值。它不会读取或保存旧事件对象，也不会把临时伤害 modifier 延长到后续牌。

ABI 100 增加稳定 HOST_EFFECT 标签 `SKIP_NEXT_PLAY_PHASE` 与 DSL
`skip_next_play_phase(player)`。宿主只允许当前存活回合角色在出牌阶段之前设置既有
`SKIP_PLAY` 标记；权威阶段转换消费标记并统一产生 `PhaseSkipped(Play)`、跳过计数和
Draw → Discard 转换。该能力不保存旧事件对象，也不引入技能专用阶段状态。

ABI 101 增加稳定 HOST_EFFECT 标签 `CHOOSE_OTHER_DRAW_OR_RECOVER` 与 DSL
`choose_other_draw_or_recover(player, draw_count, recover_count, times)`。每次迭代先由
`player` 可取消地选择另一名存活角色，再由目标不可取消地选择摸牌或回复；目标满血时直接摸牌。
每一层屏障都在恢复时重新校验参与者，并通过定长整数载荷保存次数、数值和来源技能，不保存旧
JavaScript 事件或 Promise。

ABI 102 增加只读 HOST_QUERY `FIELD_EQUIPMENT_KIND_COUNT` 与 DSL
`field_equipment_kind_count(card_kind)`，只统计存活角色装备区内物理牌的规则类别；还增加稳定
HOST_EFFECT `CHOOSE_DISCARD_HAND_EQUIP_THEN_REMOVE_SELF_SKILL` 与 DSL
`choose_discard_hand_equip_then_remove_self_skill(player, count)`。后者强制选择
`min(count, 当前可弃手牌装备数)` 张牌，数量不足时直接全弃；来源一次性技能通过确定性延后尾
请求在失牌事件、触发器及其嵌套 Choice 全部完成后移除，不保存旧 JavaScript 事件或 Promise。

ABI 103 增加稳定 HOST_EFFECT `DRAW_SELECTED_TARGETS` 与 DSL
`draw_selected_targets(count)`。它只允许用于声明了至少一个目标的主动技，VM 从本次权威命令
快照目标 ID，引擎重验目标互异且存活后按座次逐一摸牌。ABI 103 同时增加
`RECOVER_IF_MIN_CURRENT_HP` 与 `recover_if_min_current_hp(player, amount)`；最低体力条件在效果
实际执行时读取当前存活角色，而不是在 VM 发出效果时提前固化。两项能力均不保存旧 JavaScript
对象，摸牌继续携带来源技能并进入普通触发队列。

ABI 104 增加稳定 HOST_EFFECT `INCREASE_CURRENT_CARD_BASE_DAMAGE` 与 DSL
`increase_current_card_base_damage(player, card, amount)`。它保留 ABI 99 的 Sha 专用接口，
并重新校验当前被 CardUsed 触发挂起的【杀】、【南蛮入侵】或【万箭齐发】来源与实体牌。AOE
增量作为带 magic 的追加字段随无懈恢复载荷保存，进入逐目标响应队列时冻结为统一基础伤害；
Choice、断线恢复与回放均不依赖旧 JavaScript 对象或进程全局临时状态。

ABI 105 增加触发过滤器
`owner_alive_other_play_phase_duplicate_card_category_has_hand_equip` 与
`owner_alive_other_phase_end_damage_gt_one_has_hand_equip`。前者只在加载内容实际请求时扩展
既有出牌阶段类别位集：低三位仍表示基本、锦囊、装备，第 4 位表示至少一个类别重复，阶段结束后
沿既有边界清理；未加载该过滤器的包集合不写新位，保持原有状态哈希。后者读取当前回合已提交的
伤害总量并要求大于 1。两者都排除事件角色本人、死亡角色和没有可弃手牌/装备牌的拥有者，不保存
旧 JavaScript 历史数组或事件树。该版本没有新增 Rule IR 或 HOST_EFFECT 标签。

ABI 106 增加触发过滤器 `owner_alive_is_target_current_play_phase`：要求拥有者存活、是事件
角色，且当前权威回合角色就是拥有者并处于出牌阶段。`event_amount` 同时开放给 `hp_lost`
事件，读取既有 `DamageDealt { is_lose_hp: true, amount }` 事件量；未新增 opcode、查询或
效果标签。「诈降」据此用 `event_amount * 3` 摸牌，并按同一事件量叠加临时技能标记。

ABI 107 增加目标侧修正器 `card_target_cancel_if_other_user_hp_gte_target`。当使用者不是技能
拥有者且其当前体力值大于等于拥有者时，该角色使用的牌不能选择拥有者为目标；双方体力在每次
目标合法性查询时重新读取。该版本没有新增 Rule IR、HOST_QUERY 或 HOST_EFFECT 标签。

ABI 108 增加主动技能专用 staged effect
`lose_hp_then_damage_other_lose_max_hp_if_dying(player, target, amount, self_skill)`。调用者必须
声明 `targets 1 1 other`；引擎原子判断这次体力流失是否首次令拥有者进入濒死，仅在该条件成立
时减少一点体力上限，并把目标的等量普通伤害延迟到 dying/save 完整结束后。该版本新增一个
Rule IR 语句、HOST_EFFECT 126，以及两个追加到 `EffectRequest` 尾部的公开/提交态变体。

ABI 109 增加主动选牌约束 `distinct_categories`、主动目标约束 `wounded` 与
`match_selected_card_count`，以及 `recover_selected_targets(amount)`。类别按有效牌类别折叠为
基本/锦囊/装备三类；匹配约束会把自动选牌 Choice 的上下限收紧到已选目标数，并在直接提交和
恢复时再次验证。逐目标回复使用主动命令的权威目标快照、去重并按座次结算。该版本新增一个
Rule IR 语句、HOST_EFFECT 127，以及追加到 `EffectRequest` 尾部的公开变体。

ABI 110 增加主动选牌约束 `unused_category_by_skill_this_play_phase` 和主动专用效果
`record_selected_play_phase_category(self_skill)`。技能实例以 3-bit 确定性位图记录当前
出牌阶段已经支付的有效基本/锦囊/装备类别；直接提交、自动 Choice 和 Choice 恢复都排除已记录
类别。记录效果要求精确选择一张牌，并在弃牌前重新验证牌仍属于技能拥有者且位于可弃置的
手牌/装备区。位图在每次进入 Play 阶段时清零，回合级主动技禁用不受影响。该版本新增一个
Rule IR 语句、HOST_EFFECT 128，以及追加到 `EffectRequest` 尾部的公开变体。

ABI 111 增加触发筛选器 `owner_alive_targeted_by_other_trick`，匹配活着的技能拥有者成为
其他角色使用的有效锦囊牌目标；增加 `turn_face_down_cancel override 1`，仅取消正面翻至背面，
不阻止已有背面状态翻回；增加 `restore_skipped_phase override 1`，在保留公开
`PhaseSkipped` 审计事件后恢复判定、摸牌、出牌或弃牌阶段。恢复策略同时覆盖回合推进中的
跳过标记/修饰器和主动 `skip_current_phase`，不执行旧 JavaScript 的阶段函数。该版本不新增
Rule IR 语句、HOST_EFFECT 或 `EffectRequest`；`ModifierTarget` 与 `TriggerFilter` 只在末尾追加。

ABI 112 增加 `owner_alive_other_discarded_black_to_discard`：只在活着的技能拥有者观察到其他
角色以 `Discard` 原因把原始黑色实体牌移入弃牌堆时触发。触发过滤阶段保留完整移动批次，
进入 VM 时只把该批匹配的黑牌暴露为 `event_cards`，因此既有 `gain_event_cards(player)` 能精确
取得仍留在弃牌堆的黑牌，不会误取同批红牌。自身弃牌、使用/打出后的普通移动及无拥有者来源
均不匹配。该版本不新增 Rule IR、HOST_EFFECT 或 `EffectRequest`，只在 `TriggerFilter` 尾部
追加一个通用契约。

ABI 113 增加主动目标约束 `hand_gt_owner`：目标必须存活、不是拥有者，且提交或 Choice 恢复时
手牌数仍严格大于拥有者。`take_target_hand_then_repeat_until_max(player, target, skill)` 编译为
新增 Rule IR 语句和 HOST_EFFECT 129；首次目标来自主动命令且不可取消，只暴露该目标当前手牌。
取得的实体牌移动完成后，内部 `ContinueTakeLargerHandUntilMax` 请求进入 deferred tail，待本次
移动及其嵌套触发完整结束才重新比较。若拥有者并列全场最多则终止，否则 continuation 89
先提供可取消的较多手牌目标 Choice，再提供不可取消的单张手牌 Choice。两阶段恢复都会重验
观察者、目标、牌所有权与区域。该版本将 Rule IR 76 提升至 77、Engine Semantics 159 提升至
160，并在 `EffectRequest` 尾部追加公开和内部提交态变体。

ABI 114 增加触发过滤器 `owner_alive_wounded_card_targets_gt_one`：只匹配存活、已受伤的技能
拥有者观察到一张声明了多个目标的用牌事件。它复用现有 `CardUsed.target_count`，不保存旧
JavaScript 事件对象，也不新增 Rule IR、HOST_EFFECT 或 `EffectRequest`。

ABI 115 增加查询时修正器值源 `owner_play_phase_card_use_count_zero`：仅在技能拥有者是当前
出牌阶段角色且尚未主动使用任何牌时返回一；增加触发过滤器
`owner_source_sha_or_trick_current_play_phase`：只匹配技能拥有者在自己的当前出牌阶段使用的
逻辑【杀】或锦囊。两者按需启用既有每角色出牌阶段计数，不新增 Rule IR、HOST_EFFECT 或
`EffectRequest`，也不执行旧 JavaScript 回调。

ABI 116 增加仅限主动技能的分段效果
`draw_then_choose_give_or_discard_hand(player, draw_count, payment_count)`：先摸牌并等待该批移动的
嵌套触发完整结束，再强制选择把指定数量的当前手牌交给一名其他存活角色，或弃置该数量的当前
手牌；若手牌不足，则弃置所有当前可弃置手牌。分支、目标、交牌和弃牌分别使用 continuation 91，
每次恢复都重新验证技能实例、观察者、角色存活、牌所有权与手牌区。该版本将 Rule IR 77 提升至
78、Engine Semantics 160 提升至 161，在 `StmtIr` 与 `EffectRequest` 尾部追加变体，并增加
HOST_EFFECT 131；旧 JavaScript 只作为语义对照，不进入包或运行时。

ABI 117 增加
`gain_target_equipment(player, target, slot)`：取得目标装备区中当前占用指定栏位的全部牌；
栏位支持 `weapon`、`armor`、`horse_def`、`horse_off`、`treasure`。效果结算时重新扫描
目标装备区，不保存旧事件对象或预先捕获的牌列表。同时增加
`owner_is_target_source_has_weapon` 伤害触发过滤器。该版本将 Rule IR 78 提升至 79、
Engine Semantics 161 提升至 162，并追加 HOST_EFFECT 132。

ABI 118 增加
`discard_all_target_equipment(actor, target)`：弃置 `actor` 当前可从 `target` 装备区弃置的
全部牌；执行时重新验证双方存活、装备区归属和弃置保护，无合法装备时空操作。同时增加
`owner_source_other_target_alive_has_discardable_hand_equip` 伤害触发过滤器。该版本将
Rule IR 79 提升至 80、Engine Semantics 162 提升至 163，并追加 HOST_EFFECT 133。

ABI 119 增加
`increase_current_sha_target_damage(source, card, target, amount)`：只提高当前结算中【杀】
对指定目标的伤害，不影响同一张【杀】的其他目标；执行时重新验证来源、实体牌和当前目标。
同时增加 `owner_source_sha_target_alive_owner_outside_target_attack_range` 指定目标触发过滤器。
该版本将 Rule IR 80 提升至 81、Engine Semantics 163 提升至 164，并追加 HOST_EFFECT 134。

ABI 120 增加
`choose_target_draw_then_owner_if_below_max_hp(player, count)`：选择任一存活角色摸指定数量的牌，
待该次摸牌及其嵌套触发完整结束后，若目标最终手牌数仍小于体力上限，拥有者摸一张牌。目标
Choice、技能实例和后置检查均在权威状态上重新验证；多点伤害可复用既有
`owner_is_target_per_damage_point` 分段调度。该版本将 Rule IR 81 提升至 82、Engine
Semantics 164 提升至 165，并追加 HOST_EFFECT 135 和 continuation 92。

ABI 121 增加主动目标约束 `hp_gt_owner`，以及
`choose_bound_source_attack_range_damage(player, source, amount)`：技能拥有者选择绑定伤害来源
攻击范围内的一名存活角色，再由该来源造成普通伤害。激活提交和 Choice 恢复均重新验证严格体力
关系、技能实例、绑定来源、双方存活、目标存活与当前攻击范围。该版本将 Rule IR 82 提升至 83、
Engine Semantics 165 提升至 166，并追加 HOST_EFFECT 136 和 continuation 93。

ABI 122 增加
`choose_target_draw_max_hp_cap_then_discard_excess(player, cap)`：允许技能拥有者选择一名体力
上限为正的存活角色，令其摸 `min(max_hp, cap)` 张牌；该次摸牌和嵌套触发结束后重新读取目标
体力上限，只在手牌超出新上限时强制目标精确选择弃牌。技能拥有者可以已经死亡，以支持死亡
事件上的 `forceDie` 等价语义，但目标选择恢复仍验证技能实例、观察者、目标存活和正体力上限。
该版本将 Rule IR 83 提升至 84、Engine Semantics 166 提升至 167，并追加 HOST_EFFECT 137、
continuation 94 和 `EffectRequest` 哈希标签 174。

ABI 123 增加
`give_selected_then_target_choose_discard_same_suit_or_lose_hp(player, target)`：读取主动技能
提交的唯一手牌输入，将该实体牌以 `GiveCards` 语义交给其他存活目标；移动和嵌套触发结束后，
目标强制选择公开当前手牌并弃置手牌区、装备区内所有与所给牌物理花色相同且可弃置的牌，或
失去一点体力。输入牌归属、目标存活与非自身约束在初始阶段验证，后置 Choice 重新验证目标
观察者和存活状态；分支开始后不再依赖拥有者继续存活或保有技能。该版本将 Rule IR 84 提升至
85、Engine Semantics 167 提升至 168，并追加 HOST_EFFECT 138、continuation 95 和
`EffectRequest` 哈希标签 175。

ABI 124 增加 `gain_equipment_from_draw_pile(player)`：从共享牌堆顶端方向扫描第一张装备牌，
并将其移动到指定角色手牌；牌堆没有装备时为空操作，绝不查询弃牌堆。该版本将 Rule IR 85
提升至 86、Engine Semantics 168 提升至 169，并追加 HOST_EFFECT 139 和 `EffectRequest`
哈希标签 176。

ABI 125 增加
`give_all_hand_recover_to_one(player, target)`：把拥有者当前全部手牌以 `Give` 原因交给另一名
存活角色，等待移动及其嵌套触发完整结束后，重新读取目标体力并只补至 1。同时增加限定触发
过滤器 `owner_alive_other_dying_hand_nonempty_once_per_game`；确认后由技能实例的
`lifetime_fire_count` 消耗次数，拒绝、空手、自己濒死或已发动时不会消耗或询问。该版本将
Rule IR 86 提升至 87、Engine Semantics 169 提升至 170，并追加 HOST_EFFECT 140 和
`EffectRequest` 哈希标签 177。

ABI 126 增加全局伤害过滤器
`owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty`。它只匹配逻辑
【杀】或【决斗】造成的 `damage`，要求拥有者、伤害来源和受伤角色均存活，且后两者合计至少有
一张手牌或装备牌。【决斗】伤害现在与【杀】一样把物理牌 ID 和逻辑牌类别传播到 DamageBefore /
Damage 触发上下文；内容可以复用既有 `choose_discard_target_card` 依次处理两个参与者。

ABI 127 增加结束阶段过滤器
`owner_is_target_any_alive_self_discardable_hand_equip`，以及分段效果
`choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(player)`。拥有者先选择一名能自行
弃置手牌或装备牌的存活目标，再由目标本人强制选择并弃置一张合法牌；移动和嵌套触发结束后
重新读取目标手牌数与体力值，不相等时令仍存活的拥有者失去一点体力。每个 Choice 屏障都重新
验证观察者、技能实例、目标存活和弃牌合法性。该版本追加 HOST_EFFECT 141、continuation 96
和 `EffectRequest` 哈希标签 178。

ABI 128 为主动技目标合同增加 `hand_lt N`：每名选中目标的当前手牌数必须严格小于字面量
`N`。该字段从 AST、Rule IR 到 npack `PackageTargetSpec` 都是可选追加字段，旧包省略时行为
不变；服务端在 `ActivateSkill` 提交时与存活、目标数量、是否允许自身等约束一起重新验证。
该版本不增加 HOST_EFFECT 或 continuation。

ABI 129 增加濒死过滤器
`owner_alive_source_alive_event_player_dying_once_per_round`，以及跨观察者效果
`choose_give_one_hand_equip_or_recover(owner, giver, target)`。技能拥有者确认发动后，由
`giver` 观察一个可取消的 0..1 张手牌/装备牌选择；选牌走普通赠予管线，取消、空选或起始
无牌时令 `target` 回复 1 点体力。Choice 恢复会重新验证技能归属、三名参与者、濒死状态、
原始选项快照和当前卡牌区域。该版本追加 HOST_EFFECT 142、continuation 97 与
`EffectRequest` 哈希标签 179。

ABI 130 为主动技 `select_he` 增加 `card_armor` 约束，只提供拥有者手牌区或装备区内可弃置的
物理防具牌；同时把既有权威效果
`add_bound_skill(player, skill_def, target)` 暴露给 DSL/Rule VM。该效果为 `player` 添加或
更新一个以 `target` 为绑定目标的技能实例，目标必须仍然存活；内容可组合
`card_distance_unlimited_for_skill_bound_target` 等现有 modifier，并用普通触发器清理临时技能。
该版本追加 HOST_EFFECT 143；沿用既有 `EffectRequest::AddBoundSkill`、哈希标签 82 和技能
实例 `BND1` 状态格式，不新增 Choice continuation。

ABI 131 增加卡牌使用过滤器
`owner_source_repeats_color_has_discardable_hand_equip`。内容注册时按需启用既有当前回合上一张
用牌元数据；触发时要求拥有者存活、正处于自己的回合、当前与上一张牌均有有效红黑颜色且颜色
相同，并至少有一张当前可弃的手牌或装备牌。过滤器在每个 `CardUsed` 的历史元数据写入之前
判断，因此比较对象始终是紧邻上一张牌；本版本不增加 HOST_EFFECT 或 continuation，内容可复用
`increase_current_draw_count`、`add_skill`、`discard_one_hand_or_equip` 和 `remove_skill`。

ABI 132 为主动技目标合同增加 `hp_gt N`，在 `ActivateSkill` 提交时权威重验目标当前体力值
严格大于字面量 `N`。同时增加
`owner_alive_skill_bound_target_is_event_player`，将全局事件角色与技能实例的共享 `BND1`
绑定目标比较；以及
`replace_skill_with_bound_after_resolution_if_target_alive(player, skill, skill_def, target)`。
后者先把引擎拥有的第二阶段放入 deferred-tail 屏障，等待当前效果、嵌套触发及濒死求桃全部
结束；目标仍存活才原子添加绑定助手并移除原技能，目标已死亡则为空操作。该版本追加
HOST_EFFECT 144 与 `EffectRequest` 哈希标签 180，不新增 Choice continuation。

ABI 133 增加
`choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(player)`。效果把手牌区和装备区建模为
两个稳定分支，只开放当前至少含一张可弃牌的分支；单分支自动提交，双分支使用强制 Choice。
选定后捕获并弃置该区全部合法牌，再把摸牌阶段放入 movement deferred-tail；此时重新读取当前
已损失体力并与实际弃牌数相加。摸牌完成后再通过第二个 deferred-tail 执行 1 点失去体力，
因此卡牌移动触发和摸牌触发均保持旧异步语义的先后关系。该版本追加 HOST_EFFECT 145、
continuation 98 与 `EffectRequest` 哈希标签 181。

ABI 134 增加伤害过滤器
`owner_is_target_source_group_qun_discardable_hand_equip_gte_two`，要求技能拥有者是受伤目标、
伤害来源当前势力为群，且拥有者至少有两张可弃手牌或装备牌；并增加
`choose_discard_hand_equip_then_recover(player, count, amount)`。该效果要求精确选择 `count`
张当前合法的手牌或装备牌并弃置，再通过 movement deferred-tail 在卡牌移动及其嵌套触发完成后
回复 `amount` 点体力；候选数恰好等于 `count` 时直接提交同一移动批次。该版本追加
HOST_EFFECT 146 与 `EffectRequest` 哈希标签 182，复用既有精确选牌 continuation 88。

ABI 135 为共享目标范围增加 `same_hp`：候选是当前体力等于选择者的所有存活角色，包含选择者
本人，并在 Choice 恢复时重新验证。新增
`choose_targets_damage(player, min_count, max_count, scope, amount)`，强制选择范围内去重角色，
数量必须落在静态边界内，再按座次依次由 `player` 造成 `amount` 点普通伤害。当前候选少于
`min_count` 时为空操作，实际选择上限裁剪到候选数。该版本追加 HOST_EFFECT 147、
continuation 99 与 `EffectRequest` 哈希标签 183。

ABI 136 为主动技目标合同增加 `has_discardable_equip`：每名选中目标必须有至少一张技能
拥有者当前可以弃置的装备牌；`ActivateSkill` 提交时会检查装备区归属与
`card_discard_protected(equip)` 等权威 modifier。新增
`discard_target_equipment_draw_weapon_range(player, target)`，让 `player` 强制选择并弃置
`target` 的一张合法装备；实体武器在移动及其嵌套触发完整结束后令原装备者摸其
`weapon_range` 张牌（零值按一），其他装备不摸牌。Choice 恢复时重新验证技能、观察者、角色、
原始选项快照和当前弃置合法性。该版本追加 HOST_EFFECT 148、continuation 100 与
`EffectRequest` 哈希标签 184。

ABI 137 增加 `choose_target_reveal_hand_damage_if_sha(player, amount)`。效果从当前所有有手牌的
存活角色中强制选择一名（允许选择者本人），公开该目标的完整当前手牌，并以目标为查询者读取
每张牌的有效牌名；至少一张是逻辑【杀】时，再由 `player` 对目标造成 `amount` 点普通伤害。
Choice 恢复时重新验证技能实例、观察者、参与者存活和目标非空手，公开事件先于伤害事件提交。
该版本追加 HOST_EFFECT 149、continuation 101 与 `EffectRequest` 哈希标签 185。

ABI 138 增加 `choose_other_give_one_hand_equip(player)`，先强制选择一名其他存活角色，再复用
既有 `DrawThenGiveOne { ready_to_give: true }`/`GiveOneHandOrEquip` 路径选择并交出一张当前
手牌或装备牌。配套过滤器 `owner_is_target_hand_equip_nonempty_other_alive` 只在正数伤害、
拥有者仍存活且有牌、并存在其他存活角色时开放发动窗口。目标 Choice 恢复时重验技能和双方
状态，卡牌 Choice 恢复时继续由共享赠牌续体重验卡牌归属与区域。该版本追加 HOST_EFFECT 150、
continuation 102 与 `EffectRequest` 哈希标签 186。

ABI 139 增加过滤器 `owner_alive_gained_three_this_turn_other_alive`。加载含该过滤器的包后，
引擎在共享 `move_one_card` 路径记录每名玩家本回合进入手牌的实体牌数量；计数使用
`TurnState` 的稳定玩家键，并随既有回合玩家计数统一清空。过滤器要求拥有者存活、累计至少
三张且存在其他存活角色。回复目标直接复用 ABI 101 的
`choose_other_draw_or_recover(player, 0, 1, 1)`，没有新增 HOST_EFFECT 或重复的回复续体。

ABI 140 增加 `choose_reveal_hand_card_bind_category_target(player, target)`。效果要求 `target`
是当前出牌阶段角色，强制由技能拥有者从其当前手牌选择一张并公开，随后在技能实例的 `BCT1`
状态中保存目标与有效基本牌/锦囊牌/装备牌类别。过滤器
`owner_alive_event_player_hand_nonempty_once_per_round` 提供每轮一次的非空手阶段开始窗口；
`owner_source_bound_category_first_three_play_phase` 只匹配绑定角色在该阶段使用的同类别牌，
并复用 `play_phase_fire_count` 将奖励限制为前三次。任意真实阶段切换清空绑定。该版本追加
HOST_EFFECT 151 与 `EffectRequest` 哈希标签 187；选牌继续复用 continuation 58。

ABI 141 增加 `owner_source_damage_card_once_per_turn`。过滤器要求事件为技能拥有者的
`card_used`，且有效牌种是逻辑【杀】或伤害类锦囊，并复用 `turn_fire_count` 将接受发动限制为
每回合一次。「铿锵」的分支直接组合既有 VM `choose`、`draw_cards`、`lose_hp` 与
`increase_current_card_base_damage`；本版本不增加 HOST_EFFECT、continuation 或
`EffectRequest` 变体。

ABI 142 增加
`recover_to_max_then_optional_exchange_skill(player, removable_skill_def)`（HOST_EFFECT 152）。
第一阶段验证并消费来源技能实例、回复拥有者至体力上限，再将第二阶段放入
`deferred_tail_effects`，因此回复事件、嵌套触发和 Choice 全部先结算。第二阶段仅在拥有者仍
存活且仍拥有可牺牲技能时打开确认 Choice；continuation 103 在提交时重新验证并原子执行移除
指定技能、添加来源技能定义。该版本追加 `EffectRequest` 哈希标签 188。

ABI 143 为主动技增加 `hand_limit_positive` 前置条件，并增加 modifier 值源
`owner_skill_use_count`。前者在每次 `ActivateSkill` 提交时通过统一 `HandLimit` 查询验证当前
手牌上限严格大于零；后者读取同一拥有者、同一定义主动技的本回合 `use_count`。二者共同表达
「极奢」的反复发动和手牌上限递减，不增加 HOST_EFFECT、continuation 或额外技能状态。

ABI 144 增加 `owner_alive_not_target_hand_equip_nonempty` 和
`reveal_hand_discard_kind(player, card_kind)`（HOST_EFFECT 153）。效果先公开目标的完整当前
手牌，再按目标视角的有效牌名筛选并弃置匹配牌；空手时为空操作。`draw_then_give_one` 在
`draw_count = 0` 时直接进入既有赠牌 Choice，不经抽牌尾部续接，因此同一技能中排在其后的效果
严格等待交牌完成。新 `EffectRequest` 的稳定哈希标签为 189，不增加 continuation。

ABI 145 增加 modifier 值源 `alive_wounded_or_guiming_wu_count_min_one`：统计存活受伤角色；
拥有者以主公身份启用「归命」时，再计入其他吴势力角色，并去重后至少返回 1。过滤器
`owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip` 只匹配拥有者
使用逻辑【杀】或普通锦囊指定受伤目标，且拥有者仍有可弃手牌或装备牌。该版本直接复用
`add_skill`、`remove_skill` 与 `discard_one_hand_or_equip`，不增加 HOST_EFFECT、
continuation 或 `EffectRequest` 变体。

ABI 146 增加回合结束过滤器
`owner_alive_other_turn_ended_card_targeted_other_no_damage_other`，以及
`choose_move_target_equipment_and_or_draw(player, target)`（HOST_EFFECT 154）。后者复用
ABI 58 的装备搬移 continuation，允许取消、单选移动/摸牌或双选，并在双选时先完成装备移动及
其嵌套事件，再摸一张牌；`PhaseEnded.current_dealt_damage_other` 带缺省值并精确区分自伤，
新增 `EffectRequest` 的稳定哈希标签为 190。

ABI 147 增加直连【杀】伤害前过滤器 `owner_source_sha_direct_damage_before`，排除铁索传导；
同时增加
`choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(player, target)`
（HOST_EFFECT 155）。无武器时目标选择弃置至多来源攻击范围张手牌/装备牌或令绑定伤害及传导
基数 +1；有实体武器时强制按弃牌、加伤、弃来源武器的顺序全部执行。continuation 104 在每个
屏障重新校验伤害 ID、技能、攻击范围、武器状态、候选快照和实体牌区；新增
`EffectRequest` 的稳定哈希标签为 191。

ABI 148 增加 `choose_draw_or_recast_add_skill(player, draw_skill_def, recast_skill_def)`
（HOST_EFFECT 156）。初始 Choice 可取消；摸牌分支添加指定临时技能后摸一张牌，重铸分支只在
当前手牌/装备区存在合法牌时提供，并强制选择一张后复用 `RecastCards`。两个 Choice 屏障都
重验当前回合、出牌阶段、来源技能和卡牌区域。continuation 105 保存阶段与两个稳定技能定义，
新增 `EffectRequest` 的稳定哈希标签为 192。

ABI 149 为回合开端额外出牌阶段队列补充来源技能实例，并增加
`choose_gain_sha_for_opening_extra_play(player)`（HOST_EFFECT 157）与
`finish_opening_extra_play_sha_gain(player)`（HOST_EFFECT 158）。前者只在当前额外阶段来源
等于来源技能时打开可取消确认，按牌堆、弃牌堆顺序获得第一张实体【杀】，并把该牌与拥有者当前
额外阶段开始时的累计伤害写入阶段实体牌策略；后者在阶段结束时清除策略，累计伤害未增加则通过普通伤害管线自伤
一点。`card_distance_unlimited_for_skill_card_policy` 只修正策略中的精确实体牌。
continuation 106 保存玩家、牌、原牌区、技能和伤害基线；新增 `EffectRequest` 的稳定哈希标签
为 193、194。

ABI 150 为主动技目标约束增加 `hand_nonempty`，并增加
`reveal_target_hand_gain_shan_armor_or_draw_two(player, target)`（HOST_EFFECT 159）。
效果公开目标当前完整手牌，获得其中有效牌名为【闪】或实体类别为防具的全部牌；没有匹配牌时
摸两张。获得牌复用 `GiveCards`，保留普通移动事件及 `Give` 原因；新增 `EffectRequest`
的稳定哈希标签为 195，不增加 continuation。

ABI 151 为主动技增加拥有者 `hand_nonempty`、`face_up` 前置条件和目标 `male` 约束；双性
角色按男性目标处理。新增 `give_all_hand(player, target)`（HOST_EFFECT 160）直接复用普通
赠予移动，以及 `gain_card_if_discard(player, card)`（HOST_EFFECT 161）只获得仍在共享弃牌区
的实体牌。过滤器
`owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn` 复用完整【杀】结算
事件及技能实例每回合计数。新增 `EffectRequest` 稳定哈希标签 196、197，不增加 continuation。

ABI 152 在既有 `ShaResolutionRecord` 增加带默认值的技能-目标绑定，并在 CardUsed 触发前
建立精确【杀】记录。新增 `choose_draw_or_discard_target_card_record_sha(player, card, target)`
（HOST_EFFECT 162）提供可取消的摸牌/弃目标牌分支并记录同一张【杀】；过滤器
`owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip` 只匹配同一技能、
同一目标的完全抵消事件。新增 continuation 107 和 `EffectRequest` 稳定哈希标签 198。

ABI 153 增加 `reveal_hand_to_player(player, observer)`（HOST_EFFECT 163）与
`choose_optional_discard_target_card(player, target, scope)`（HOST_EFFECT 164），并为目标牌
范围追加 `hand` 标签 4。前者只向指定存活观察者发送完整当前手牌；后者复用 continuation 36，
以兼容载荷末尾标志区分可取消选择，空候选为确定性空操作，提交时仍重验参与者、区域、所有权
及弃牌保护。新增 `EffectRequest` 稳定哈希标签 199、200。

ABI 154 增加 `reveal_hand_choose_discard_color(player)`（HOST_EFFECT 165）。效果公开玩家
完整当前手牌；单色时直接弃置该色全部可弃手牌，红黑均存在时由该玩家通过 continuation 108
强制选择一种颜色，提交时重验玩家、候选掩码、当前所有权、有效颜色与弃牌保护。过滤器
`owner_source_sha_target_alive_hand_nonempty` 精确约束逐目标【杀】事件和目标非空手牌。
新增 `EffectRequest` 稳定哈希标签 201。

ABI 155 为主动技目标增加 `in_attack_range` 与 `has_discardable_hand_equip`，分别权威重验
攻击范围和目标当前可弃手牌/装备。新增
`choose_discard_target_cards_recover_same_category(player, target, count, amount)`
（HOST_EFFECT 166）：强制弃置 `min(count, 当前候选数)` 张目标牌；只有恰弃 `count` 张且按
旧 `type2` 规则类别相同时，才在失牌触发与嵌套选择完成后回复。新增 continuation 109 和
`EffectRequest` 稳定哈希标签 202。

ABI 156 增加
`choose_compare_selected_hand_card_second_failure_seat_damage(player, target, draw_count, failure_count, lose_hp, damage)`
（HOST_EFFECT 167）。它要求
`targets 1 1 other can_compare unused_by_skill_this_turn select_hand 1 1`，复用既有拼点开始/完成
事件；严格胜利后摸牌或弃目标一张手牌/装备牌，第二次未赢后禁用来源主动技至回合结束、失去
体力，并在濒死链完成后选择等长座次路径或自动取较短路径。路径伤害通过内部有序队列逐个
提交，每次伤害的触发与求桃先于下一个目标。新增 continuation 110–112 和 `EffectRequest`
稳定哈希标签 203–208。

ABI 157 为主动技目标契约增加 `selected_cards_match_target_equip_count`。它要求
`targets 1 1 has_equip` 和卡牌选择声明，引擎在打开选择与提交时都以唯一目标当前装备数作为
精确选牌数；成本与后续效果继续组合 `discard_selected`、`deal_damage` 等既有 VM 效果。
该 ABI 未增加 HOST_EFFECT、continuation 或 `EffectRequest`。

ABI 158 增加 `owner_alive_equip_used` 触发过滤器：存活拥有者观察到任意逻辑装备牌的
`card_used` 事件时匹配，不附带使用者、标记或装备数条件。它继续复用既有可选触发与 VM
效果；未增加 HOST_EFFECT、continuation、`EffectRequest` 或包字段。

ABI 159 增加 `remove_all_skills(player)`（HOST_EFFECT 168）：对目标当前技能实例生成稳定
快照，再逐项走既有单技能移除路径，同时注销对应触发器和修正器。该效果不增加 continuation
或包字段，新增 `EffectRequest::RemoveAllSkills` 与稳定哈希标签 209。

ABI 160 增加 `choose_other_recover(player, amount)`（HOST_EFFECT 169）：强制玩家选择一名
其他存活角色，并令该目标回复指定点数；满体力目标仍可选择但结算为空操作。效果复用
`ChooseOtherDrawOrRecover` continuation 的追加阶段，不增加 continuation 或包字段，并新增
`EffectRequest::ChooseOtherRecover` 与稳定哈希标签 210。

ABI 161 增加 `enable_equip_slot(player, slot, source)`（HOST_EFFECT 170）：恢复 `player`
指定装备栏的一层废除计数，并以 `source` 作为归因发出 `equip_slot_enabled` 事件。栏位标签
为 1（武器）至 5（宝物）；无效标签是内容错误，未废除或已离场时为空操作。对应
`EffectRequest::EnableEquipSlot` 与稳定哈希标签 211。

ABI 162 为共享存活角色目标范围追加 `hand_empty`（稳定标签 6）：候选是当前没有手牌的
存活角色，包含选择者本人。`choose_target_draw`、`choose_targets_draw` 与
`choose_targets_damage` 统一复用该范围，并在 continuation 恢复时重新校验手牌状态。同时增加
结束阶段过滤器 `owner_is_target_any_alive_hand_empty`。该 ABI 不增加 HOST_EFFECT、
`EffectRequest`、continuation 或包字段。

ABI 163 增加规范状态 `draw_reshuffle_count`，仅在非空弃牌堆实际洗入摸牌堆后递增；并增加
准备阶段过滤器 `owner_is_target_draw_reshuffle_zero_wounded` 与
`owner_is_target_draw_reshuffle_positive`。两者可将同一锁定技拆成互斥分支，直接复用
`recover` 与 `lose_hp`，不增加 HOST_EFFECT、`EffectRequest`、continuation 或包字段。

ABI 164 增加 `turn_over_cancel` modifier；`override 1` 同时取消正面翻至背面和背面翻至正面。

ABI 165 增加 `owner_is_target_source_alive_once`：仅匹配拥有者受到的、来源仍存活的事件，
并按技能实例限制每回合实际发动一次。
既有 `turn_face_down_cancel` 继续只阻止正面翻至背面。新 modifier 复用权威 `turn_over` 入口，
不增加 HOST_EFFECT、`EffectRequest`、continuation 或包字段。

ABI 166 增加
`card_target_cancel_if_other_user_distance_gt_one(selector)` 目标侧 modifier：另一角色到技能
拥有者的最终距离大于一时，匹配 selector 的牌不能选择拥有者。距离复用统一距离查询，包含
座次、坐骑及技能修正；该 ABI 不增加 HOST_EFFECT、`EffectRequest`、continuation 或包字段。

ABI 167 增加 `duel_opponent_response_count` 参与者侧 modifier：每层令拥有者的【决斗】对手
每轮多打出一张【杀】，拥有者自己的响应数不变。【决斗】续效以 `need/have` 保存同一轮的
响应进度，实体【杀】与视为【杀】复用同一队列。

ABI 168 增加
`target_choose_suit_gain_owner_hand_damage_if_mismatch(player, target)`（HOST_EFFECT 171）：
目标先选择花色，再从 `player` 当前手牌中选择一张获得；以 `player` 视角计算该牌有效花色，
不符时由 `player` 对目标造成一点普通伤害。主动技要求
`hand_nonempty targets 1 1 other`。两段选择复用 continuation 113，新增
`EffectRequest` 稳定哈希标签 212。

ABI 169 增加 `owner_is_target_source_alive_gainable_card`：拥有者受到有存活来源的正数伤害，
且来源存在可获得牌时按伤害事件匹配一次；自伤仅暴露装备区，其他来源暴露手牌区和装备区。
它与 ABI 22 的逐伤害点版本共用 `take_one_card`，不增加 HOST_EFFECT、`EffectRequest` 或
continuation。

ABI 170 增加 `owner_is_target_card_in_processing`：拥有者受到伤害，且当前伤害实体牌载体仍在
公共处理区时匹配。技能可直接组合 `event_card`、`zone` 与 `move_card` 获得该载体；无实体牌
伤害不会打开触发选择，不增加 HOST_EFFECT、`EffectRequest` 或 continuation。

ABI 171 增加
`judge_source_discard_two_hand_or_damage(player, source)`（HOST_EFFECT 172）：`player`
判定，红桃直接结束；非红桃且 `source` 仍存活时，来源可弃置恰好两张可弃置手牌，否则受到
`player` 造成的一点普通伤害。选择复用 continuation 114，新增 `EffectRequest` 稳定哈希
标签 213。

ABI 172 增加
`give_selected_recover_after_phase_total_two(player, target)`（HOST_EFFECT 173）：把主动技
本次选择的手牌交给其他 `target`，并复用技能实例的出牌阶段计数记录累计给牌数；累计数首次
从不足两张跨到至少两张时，在移动及其嵌套触发完成后令 `player` 回复一点体力。该效果要求
`targets 1 1 other select_hand 1 255`，不增加 continuation，`EffectRequest` 稳定哈希标签
为 214。

ABI 173 增加
`choose_other_hand_targets_gain_one_replace_draw(player)`（HOST_EFFECT 174）：摸牌阶段可选择
一至两名有手牌的其他存活角色，按座次逐个选择并获得一张当前手牌，然后把本次通常摸牌置零。
目标与卡牌选择使用 continuation 115，`EffectRequest` 稳定哈希标签为 215。

ABI 174 增加 `distribute_top_two_cards(player)`（HOST_EFFECT 175）：把牌堆顶至多两张牌移到
公共处理区，随后反复选择非空剩余牌子集和任意存活接收者，直到全部分配。通常与
`owner_is_target_per_damage_point optional` 组合；选择使用 continuation 116，
`EffectRequest` 稳定哈希标签为 216。

ABI 175 增加 `owner_from_equip_per_card`：只匹配从技能拥有者装备区离开的
`cards_moved`，并按该批次失去的装备牌张数逐次触发。它复用普通可选触发与已有效果，不增加
HOST_EFFECT、`EffectRequest` 或 continuation。

ABI 176 增加视为牌过滤器 `owner_outside_turn_red`：仅当技能拥有者不是当前回合角色且来源牌
的有效花色为红色时匹配。STANDARD「急救」以
`view_as tao response owner_outside_turn_red from hand_equip` 复用普通救援管线。

ABI 177 增加
`judge_make_current_sha_target_unrespondable_on_red(player, card, target)`（HOST_EFFECT 176）：
先重检保存中的实体【杀】逐目标上下文，再为 `player` 判定；红色时只关闭 `target` 对该【杀】
的响应窗口。它不安装持久修改器、不增加 continuation，`EffectRequest` 稳定哈希标签为 217。

ABI 178 增加 `owner_alive_other_phase_end_has_discardable_basic_hand` 与
`choose_discard_basic_then_target_discard_equip_or_damage(player, target)`（HOST_EFFECT 177）：
其他角色结束阶段开始时，拥有者可弃一张有效基本手牌；费用结算后，目标可弃一张有效装备牌，
否则受到一点普通伤害。两段选择使用 continuation 117，`EffectRequest` 稳定哈希标签为 218。

ABI 179 增加 `owner_alive_prepare_player_hp_gt_owner` 与
`owner_alive_prepare_lord_hp_gt_owner`：拥有者观察体力更高角色的准备阶段，后者还要求事件角色
具有主公身份。两者可把基础摸牌与主公额外摸牌、临时技能、叠层标记组合为基础与增量分支，
不增加 HOST_EFFECT、`EffectRequest` 或 continuation。

ABI 180 增加 `owner_alive_other_sha_target_can_redirect_to_owner` 与
`choose_current_sha_target_discard_redirect_to_owner(owner, source, card, target)`
（HOST_EFFECT 178）：当前【杀】目标可弃一张手牌/装备；费用触发链完成后，把保存中的该目标槽
替换为技能拥有者并发出新增目标事件。选择使用 continuation 118，`EffectRequest` 稳定哈希
标签为 219。

ABI 181 增加 `cards_given`、`owner_alive_gave_cards_to_other_alive` 与
`choose_target_gain_bottom_card(player)`（HOST_EFFECT 179）。共享赠牌管线在普通移动事件后
追加赠予者、受赠者和实体牌归因；底牌先公开进入处理区，移动触发完成后再选择受赠者。选择使用
continuation 119，`EffectRequest` 稳定哈希标签为 220。

ABI 182 增加 `judge_black_gain_repeat(player)`（HOST_EFFECT 180）：红色判定结束，黑色判定牌
若仍在弃牌堆则获得，并在移动触发完成后询问是否继续。重复选择使用 continuation 120，
`EffectRequest` 稳定哈希标签为 221。

ABI 183 增加 `second_can_use_juedou_on_first` 与
`use_virtual_juedou_second_on_first()`（HOST_EFFECT 181）。前者对精确两个有序目标在费用前
校验第二名能否对第一名使用无来源【决斗】；后者复用普通独立锦囊队列结算该虚拟【决斗】，
但不打开【无懈可击】窗口。未增加 continuation，`EffectRequest` 稳定哈希标签为 222。

ABI 184 增加参数化视为牌来源 `from group_<tag>`，当前用于
`view_as shan response any from group_wei`。拥有者选择已有无来源响应令牌后，引擎从下一座
起依次询问其他匹配势力角色；首名响应者按普通实体/视为牌规则打出【闪】，全部拒绝则返回原
响应失败分支。选择使用 continuation 121，未新增 HOST_EFFECT 或 `EffectRequest`。

ABI 185 增加 `owner_dying_other_tao_responder_group_<tag>` 与
`recover_hp_as(source, player, amount)`（HOST_EFFECT 182）。前者监听其他指定势力角色为濒死
技能拥有者响应【桃】；后者复用普通回复请求但显式保留回复来源。求桃的实体牌和无来源视为牌
都会先发出 `CardResponded`，相关强制触发在基础回复及继续求桃前结算。未新增 continuation
或 `EffectRequest`。

ABI 186 增加 `owner_sha_target_can_discard_redirect_to_other_in_attack_range` 与
`choose_discard_redirect_current_sha(player, source, card)`（HOST_EFFECT 183）。效果先选择费用
牌和替换目标，两项都确认后才弃牌；弃牌嵌套触发完成后复用保存中【杀】的目标槽替换。替换目标
必须位于技能拥有者攻击范围内、不是【杀】使用者、未被该【杀】指定且通过目标禁用校验。选择
使用 continuation 122，`EffectRequest` 稳定哈希标签为 223。

ABI 187 将势力代打来源扩展为 `view_as sha any any from group_<tag>`。该声明同时覆盖主动使用与
南蛮入侵、决斗等【杀】响应窗口；匹配势力角色从技能拥有者下一座起依次提供实体或视为【杀】，
首名成功者支付牌，拥有者继续原逻辑【杀】上下文。复用 continuation 121 和无来源视为牌续体，
未新增 HOST_EFFECT 或 `EffectRequest`。

ABI 188 增加 `judge_set_skill_mark_to_rank(player, skill)`（HOST_EFFECT 184）：复用标准判定
管线，并以来源技能定义 ID 为标记 ID，把最终实体结果点数替换写入拥有者标记。配套
`sha_distance_unlimited_if_rank_lt_skill_mark` 与
`sha_no_use_count_if_rank_gt_skill_mark` 两个查询时 modifier；后者既绕过也不增加通常【杀】
次数。严格等号不匹配；无点数逻辑【杀】按旧 `unsure` 分支同时匹配两项。未增加
continuation，`EffectRequest` 稳定哈希标签为 224。

ABI 189 增加 `owner_is_target_linked` 与
`choose_discard_one_from_each_linked(player, skill)`（HOST_EFFECT 185）。效果冻结按座次排列、
横置且有可弃手牌或装备牌的存活角色，并依次复用目标区域单牌选择续体；技能拥有者负责全部
选择。未增加 continuation，`EffectRequest` 稳定哈希标签为 225。

ABI 190 增加只读查询 `alive_wounded_or_guiming_wu_count(player)`（HOST_QUERY 25），复用
`stdcanshi` 已有的“受伤角色，启用主公技归命时并入其他吴势力角色”集合语义；返回真实计数，
不再强制最小一。`owner_is_target_alive_wounded_or_guiming_wu_exists` 只在该集合非空时开放摸牌
阶段发动窗口，`owner_source_sha_or_ordinary_trick_has_discardable_hand_equip` 则约束使用
【杀】或普通锦囊后的强制自弃。未新增 HOST_EFFECT、continuation 或 `EffectRequest`。

ABI 191 增加参数化过滤器
`owner_source_jiu_boosted_sha_damage_skill_<def>_enabled` 与
`disable_skill_by_def_until_turn_end(player, skill_def)`（HOST_EFFECT 186）。过滤器仅匹配
拥有者以酒加成的逻辑【杀】造成伤害且指定技能仍有效的事件。效果保留原技能实例与状态，只把
匹配定义从触发器/修改器索引临时撤下；下一回合开始清除禁用位并重建索引。未增加 continuation，
`EffectRequest` 稳定哈希标签为 226。

ABI 192 增加主动目标约束 `hand_equip_nonempty`，以及
`target_discard_then_duel_if_non_sha_and_hp_gte_owner(player, target)`（HOST_EFFECT 187）。
目标自己选择并弃置一张手牌或装备牌；续体 123 等待移动触发完成，再按弃牌有效牌名、双方当前
体力和当前【决斗】合法性决定是否进入普通无懈与决斗响应管线。`EffectRequest` 稳定哈希标签
为 227。

ABI 193 为主动技 `uses_per_turn` 增加动态值源 `owner_lost_hp_min_one`。权威发动入口在每次
发动时读取拥有者当前损失体力，最少取一，并继续复用既有每回合技能使用计数；未新增 opcode、
HOST_EFFECT、continuation 或 `EffectRequest`。

ABI 194 增加 `owner_is_target_any_alive_self_discardable_hand` 与
`choose_target_lose_hp_discard_hand_sha_if_basic(player)`（HOST_EFFECT 188）。效果选择任意
能自行弃置手牌的存活角色，先令拥有者失去 1 点体力，再令目标弃置一张手牌；续体 124 等待
移动触发完成，并按弃牌有效类别决定是否使用忽略距离但遵守通常次数的虚拟【杀】。
`EffectRequest` 稳定哈希标签为 228。

ABI 195 增加目标域 `wounded`、`owner_is_target_any_alive_wounded` 与
`choose_optional_targets_draw(player, count, min_count, max_count, scope)`（HOST_EFFECT 189）。
效果可取消；合法目标少于下限时不开放选择，上限按当前合法目标数收窄。复用续体 43，在恢复时
重验目标并按座次依次执行带来源技能的摸牌；`EffectRequest` 稳定哈希标签为 229。

ABI 196 增加
`reveal_target_hand_sha_or_discard_bind(player, target, helper_skill_def)`（HOST_EFFECT 190）。
效果先失去 1 点体力并等待完整濒死窗口；之后按目标当前手牌是否含有效【闪】走强制弃牌或
私密展示、无距离虚拟【杀】分支，且【杀】仍遵守通常次数。杀结算完成后仅绑定仍存活的目标。
新增 `distance_from_skill_bound_target override 1`，把拥有者至技能实例绑定目标的最终距离设为
1；`EffectRequest` 稳定哈希标签为 230，不新增 continuation。

ABI 197 增加 `draw_remembered_hand_cards(player, count)`（HOST_EFFECT 191）和
`discard_remembered_hand_cards(player)`（HOST_EFFECT 192）。前者把本次摸到且持续留在拥有者
手牌区的实体牌记录在 originating skill 实例；实体牌一旦离开该手牌区即被遗忘，之后返回也
不会恢复记录。后者先清空记录，再弃置其中仍在手牌区且当前可弃置的牌。
`EffectRequest` 稳定哈希标签分别为 231、232，不新增 continuation。

ABI 198 增加
`gain_draw_cards_for_lifetime_card_action_divisors(player, divisor, group, ...)`
（HOST_EFFECT 193）。效果读取既有本局用牌/打出累计数，为每个命中的正整数倍数从共享牌堆
顶部优先取得一张对应组实体牌；组为 `sha_or_shan`、`tao_or_jiu` 和
`juedou_or_wuzhong`。同次命中的牌经一次移动进入手牌，不搜索弃牌堆。
`EffectRequest` 稳定哈希标签为 233，不新增 continuation。

ABI 199 增加参数化过滤器
`owner_alive_other_shan_outside_turn_group_<tag>_once_per_turn`。它统一匹配其他存活角色在
自己回合外使用或打出逻辑【闪】、且其势力等于 `<tag>` 的事件；同一技能实例在当前回合首次
确认后才消耗次数。可选触发可声明 `by event_player`，由实际出闪角色确认。未新增 opcode、
HOST_EFFECT、continuation 或 `EffectRequest`。

ABI 200 增加
`lose_hp_gain_discard_card_for_opening_extra_play(player, card_kind, mandatory_skill_def)`
（HOST_EFFECT 194）。效果只在 originating skill 创建的回合开头额外出牌阶段生效；拥有
`mandatory_skill_def` 时直接失去一点体力，否则通过续体 125 提供可取消确认，随后复用既有
弃牌堆按类别取牌效果。稳定技能 ID 保留完整 `u32` 位模式。`EffectRequest` 稳定哈希标签为
234；Rule IR 提升至 86，Engine semantics 提升至 174。

ABI 201 增加过滤器
`owner_is_target_lost_cards_at_most_hp_gainable` 与效果
`choose_gain_own_lost_discard_card_this_turn(player)`（HOST_EFFECT 195）。内容注册表只在
加载该过滤器时启用回合失牌溯源：累计手牌/装备区实际失去次数，同时按首次失去顺序保存唯一
实体牌。过滤器要求累计数处于 `1..=当前体力值` 且至少一张候选仍在弃牌堆；续体 126 的
强制选牌会重新验证技能、回合、体力阈值、失牌归属和弃牌堆位置。`EffectRequest` 稳定哈希
标签为 235；Rule IR 提升至 87，Engine semantics 提升至 175。

ABI 202 增加过滤器 `owner_is_target_source_movable_field_card` 与效果
`choose_move_field_card_to(player, target)`（HOST_EFFECT 196）。共享查询只返回其他存活
角色装备区或判定区中可移入固定目标的牌：装备要求目标对应槽位启用且空置，延时锦囊要求
目标判定区没有同类牌。过滤器、效果打开选择及续体 127 都调用该查询，屏障恢复时还会重验
技能实例和固定目标。`EffectRequest` 稳定哈希标签为 236；Rule IR 提升至 88，
Engine semantics 提升至 176。

ABI 203 增加效果
`draw_then_optionally_distribute_hand_cards(player, draw_count, give_cap)`（HOST_EFFECT 197）。
效果先摸牌并通过尾部屏障等待所有嵌套触发，再允许拥有者从当前手牌反复选择非空子集和其他
存活目标；可在任一选牌轮取消。规划的实体牌与目标以位置配对保存在续体载荷中，提交前仍留在
原手牌，最终一次性重验技能、存活状态、牌归属、去重和总上限后按目标给牌。
`EffectRequest` 稳定哈希标签为 237，续体标签为 128；Rule IR 提升至 89，
Engine semantics 提升至 177。

ABI 204 增加主动技效果
`give_selected_then_continue_distinct_recover_at_two(player, target)`（HOST_EFFECT 198）。
首次批次取自权威主动技选牌和首目标；每批给牌及恢复的嵌套触发全部完成后，续体 129 才允许
拥有者取消，或选择新的非空当前手牌批次，再选择本次尚未使用的其他存活目标。状态机自身保存
已使用目标和本次累计给牌数，累计首次跨过两张时排入一次来源为拥有者的恢复。
`EffectRequest` 稳定哈希标签为 238；Rule IR 提升至 90，Engine semantics 提升至 178。

ABI 205 增加死亡触发效果
`choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(player, mark_def)`（HOST_EFFECT 199）。
效果允许已死亡的技能拥有者从指定正标记数并列最高的其他存活角色中强制选择一名；续体 130
恢复时重验最高标记，再执行实体牌判定。判定牌为【桃】或【桃园结义】时结束，否则排入无来源
`KillPlayer`。`EffectRequest` 稳定哈希标签为 239；Rule IR 提升至 91，
Engine semantics 提升至 179。

ABI 206 增加逐目标随机取牌效果
`choose_zone_priority_gain_random_card_from_each_other_then_turn_over(player)`（HOST_EFFECT 200）。
效果先以续体 131 强制选择手牌/装备牌/判定牌优先级，再按稳定座次逐名处理其他存活角色。
每名目标结算时重新读取区域，只从首个非空优先区域使用权威 `RuleRng` 选择一张实体牌；
该移动及其嵌套触发结束后才继续，全部完成后翻面。匹配过滤器
`owner_is_target_per_damage_point_other_hand_equip_judge_nonempty` 会按伤害点数重复并避免空窗口。
`EffectRequest` 稳定哈希标签为 240；Rule IR 提升至 92，Engine semantics 提升至 180。

ABI 207 增加主动技 `mark_at_least <mark_def> <count>` 门槛、
`add_skill_until_turn_end(player, skill_def)` 与
`resolve_mass_damage_discard_turn_over(player, mark_def, mark_count, damage_amount, hand_count)`，
并增加 `record_active_target(player, target)` 写入声明式回合目标历史。
临时技能按到期回合记录来源，永久同名技能不叠加也不被回合清理；群体效果按稳定座次快照逐项
跨越结算屏障。HOST_EFFECT 标签为 201、202、203，`EffectRequest` 稳定哈希标签为 241、242、243，
续体标签为 132；Rule IR 提升至 93，Engine semantics 提升至 181。

ABI 208 增加主动目标约束 `linked` 与
`max_by_owner_mark <mark_def>`，分别要求目标当前横置，并以技能拥有者的当前标记数约束
目标数上限。新增
`resolve_mark_parity_control(player, mark_def, mass_threshold, damage_amount)` 和
`resolve_selected_equipment_discard_then_choose_damage(player, damage_amount, nature)`；
前者保存奇偶目标分支、弃牌选择、清标记确认和逐座次群体伤害，后者逐目标弃置全部装备后
从仍存活的原目标中选择一名造成有来源伤害。HOST_EFFECT 标签为 204、205，
`EffectRequest` 稳定哈希标签为 244、245，续体标签为 133–136；
Rule IR 提升至 94，Engine semantics 提升至 182。

ABI 209 增加
`replace_draw_with_distinct_suit_selection(player, count)` 和
`inspect_target_hand_choose_heart_discard_or_top(player, target)`。
前者以权威标志清零当前普通摸牌，将牌堆顶实体牌移入处理区并公开，续体强制每种有效花色
恰好选择一张进入手牌，其余进入弃牌堆；后者只向技能拥有者展示目标当前手牌，可选择一张
红桃牌，再强制选择弃置或公开置于牌堆顶。HOST_EFFECT 标签为 206、207，
`EffectRequest` 稳定哈希标签为 246、247，续体标签为 137–139；
Rule IR 提升至 95，Engine semantics 提升至 183。

ABI 210 增加弃牌阶段结束事件 `phase_discard_end`、过滤器
`owner_is_target_discarded_at_least_two_in_discard_phase`，以及
`resolve_yeyan(player)`、`choose_all_lose_hp_or_recover(player)`。
注册表仅在加载该过滤器时记录当前弃牌阶段由本人从手牌/装备区实际弃置的实体牌数。
业炎状态机保存四花色成本、目标快照、失去体力屏障与伤害分配；琴音可取消，或按稳定座次
对当前全部存活角色执行失去/回复体力。HOST_EFFECT 标签为 208、209，
`EffectRequest` 稳定哈希标签为 248、249，续体标签为 140–145；
Rule IR 提升至 96，Engine semantics 提升至 184。

ABI 211 为实体来源 `view_as` 增加 `up_to 2`、`same_suit`、`nature fire` 与
`multi_card_bonus damage_1|recover_1|discard_current_hand_equip`。主动用牌直接提交一至
两张实体来源；响应窗口通过稳定的不透明技能选项进入续体 146，再选择具体来源牌，恢复时
重验技能、次数、牌归属、区域、花色与卡牌禁用规则。统一用牌/响应管线负责消耗全部来源、
火【杀】、伤害/回复增量和响应后的目标弃牌；未新增 HOST_EFFECT 或 `EffectRequest`。
Rule IR 提升至 97，Engine semantics 提升至 185。

ABI 212 增加 `resolve_qixing(player, initialize)` 与
`resolve_qixing_weather(player, qixing_skill_def, helper_skill_def, max_targets)`。
前者复用公开技能扩展牌堆，开局将牌堆顶七张牌移入并提供等量手牌交换；后者按等量“星”和
存活目标结算目标侧辅助技能。辅助技能状态保存规范化多来源集合，每个来源在自己的下回合
实际开始时独立到期。HOST_EFFECT 标签为 210、211，`EffectRequest` 稳定哈希标签为
250、251，续体标签为 147–150；Rule IR 提升至 98，Engine semantics 提升至 186。

ABI 213 增加 `resolve_nzry_longnu(player, fire_skill_def, thunder_skill_def)`、
`choose_link_other(player)`、`view_as` 的 `card_trick`/`nature thunder`，以及
`link_loss_cancel`、`sha_thunder` 和
`queried_player_linked_first_living_skill_owner` modifier 语义。显式转换后的逻辑牌名参与
不限次数判定。HOST_EFFECT 标签为 212、213，`EffectRequest` 稳定哈希标签为 252、253，
续体标签为 151；Rule IR 提升至 99，Engine semantics 提升至 187。

ABI 214–215 增加
`resolve_drlt_poxi(player, target, hand_limit_skill_def)` 与
`resolve_drlt_jieying(player, event_player, mark_def, mode)`。魄袭通过私密观察者投影公开
目标手牌，并在续体恢复时重验双方牌区、四张实体牌和四种有效花色；劫营复用稳定标记、
回合生命周期及给牌事件。`global_draw_count`、`global_sha_use_limit`、`global_hand_limit`
可使用 `queried_player_mark_positive_<mark>`，每名存活技能拥有者独立贡献一次。
HOST_EFFECT 标签为 214、215，`EffectRequest` 稳定哈希标签为 254、255，续体标签为
152、153；Rule IR 提升至 100，Engine semantics 提升至 188。

ABI 216–217 增加 `resolve_drlt_duorui(...)`、`resolve_drlt_zhiti(...)`、技能声明标记
`non_gainable`，以及止啼所需的受伤/攻击范围全局手限值源。技能实例可记录多个失效原因；
移除原因技能时统一恢复派生触发器和 modifier。新 `EffectRequest` 稳定哈希以零字节转义到
16 位标签 256、257，旧标签及旧包哈希不变；续体标签为 154–156。Rule IR 提升至 101，
Engine semantics 提升至 189。

ABI 218 增加 `resolve_shen_simayi(player, event_player, event_amount, mark, mode)`、
`judgment_rewrite_hand_equip_cost_mark_<mark>` 与带标记条件的普通锦囊触发过滤器。连破使用
回合外队列和原回合锚点；当前回合角色死亡也先结算死亡事件和 `TurnEnded`，再推进回合。
完杀的激活标记同时驱动【桃】限制，并在濒死结算期间禁用其他非濒死角色的非锁定技。
HOST_EFFECT 标签为 218，`EffectRequest` 稳定哈希标签为 259，续体标签为 157；
Rule IR 提升至 102，Engine semantics 提升至 190。

ABI 229 增加只读查询 `hand_equip_suit_count(player, suit)` 与过滤器
`owner_source_sha_or_juedou_single_target_first_same_name_this_turn`。查询按指定玩家视角
统计当前手牌、装备牌的有效花色；过滤器让首次单目标逻辑【杀】与首次单目标【决斗】分别
获得触发窗口。`increase_current_card_base_damage` 继续使用既有 HOST_EFFECT，并可修改
暂停中的【决斗】独立目标队列伤害基数。Engine semantics 提升至 201。

ABI 230 增加过滤器
`owner_source_sha_damage_before_endpoint_discardable_hand_equip` 与通用效果
`choose_discard_damage_endpoint_card_increase_if_suit(player, target, suit, amount)`。
效果使用 HOST_EFFECT 标签 226、稳定哈希标签 266 和续体标签 162；弃牌移动及其嵌套触发
先于条件加伤，直接和连环逻辑【杀】伤害均可绑定。Rule IR 提升至 109，Engine semantics
提升至 202。

ABI 231 增加过滤器 `owner_longdan_view_as_first_counterpart_hand_nonempty` 与通用效果
`take_one_event_target_hand(player)`。用牌与响应管线临时记录实体/无实体 `view_as` 的
技能实例和实体牌；过滤器只接受名称含 `longdan`/`fanghun` 的转换【杀】【闪】，并将首个
目标或被响应来源写入事件目标。效果使用 HOST_EFFECT 标签 227、稳定哈希标签 267，复用
既有 `PickTargetCard` 续体并新增仅手牌模式。Rule IR 提升至 110，Engine semantics
提升至 203。

ABI 282 增加过滤器 `owner_source_sha_damage_target_alive_has_discardable_equip` 与效果
`resolve_kuangfu(player, target)`。效果先选择受伤目标的一张可弃装备；兼容装备可在弃置和
移动之间选择，不兼容装备直接弃置。移动分支复用标准装备栏替换规则。HOST_EFFECT 标签为
272，`EffectRequest` 稳定哈希标签为 314，续体标签为 198。

ABI 283 增加过滤器 `owner_source_equip_used_skill_mark_below_seven` 与效果
`resolve_shanjia(player, count)`。过滤器只累计技能拥有者使用的逻辑装备牌；效果将数量封顶
为 7，摸牌后强制弃置等量当前手牌/装备，若选择时含装备区牌，则在移动触发完成后提供一张
无距离限制但计通常次数的虚拟【杀】。HOST_EFFECT 标签为 273，`EffectRequest` 稳定哈希
标签为 315，续体标签为 199。

ABI 284 增加过滤器 `owner_source_outside_turn_card_used_or_responded` 与效果
`resolve_new_yajiao(player, card)`。效果将牌堆顶牌移入公开处理区，强制选择任意存活角色
获得；若其有效基本/锦囊/装备类别与触发牌不同，则在获得牌及嵌套触发完成后强制拥有者弃置
一张手牌/装备。HOST_EFFECT 标签为 274，`EffectRequest` 稳定哈希标签为 316，续体标签
为 200。

ABI 285 增加主动选牌约束 `unused_category_by_owner_history_this_turn` 与效果
`resolve_dingpin(player, target)`。前者统一记录拥有者本回合使用及因弃置失去的逻辑
基本/锦囊/装备类别；后者令目标判定，黑色时摸已损失体力张牌并记录本技能目标，红色时令
拥有者翻面。HOST_EFFECT 标签为 275，`EffectRequest` 稳定哈希标签为 317。

ABI 286 增加 `finish_current_phase_abort_resolution(player)`：清空共享处理区与当前结算续体，
随后结束当前阶段但不产生 `PhaseSkipped`。HOST_EFFECT 标签为 276，`EffectRequest`
稳定哈希标签为 318。

ABI 287 增加过滤器 `owner_alive_shan_used_by_owner_or_during_owner_turn`、
`owner_alive_other_phase_play_skill_pile_nonempty`，以及效果
`gain_top_card_to_skill_pile(player)`、
`choose_discard_skill_pile_card_add_marked_skill(player, target, helper_skill_def)`。
两项效果复用公开技能牌堆、精确卡牌选择和同 ID 标记 modifier；HOST_EFFECT 标签为
277/278，`EffectRequest` 稳定哈希标签为 319/320，续体标签为 201。

ABI 288 增加主动目标约束 `max_by_owner_hp` 与效果
`gain_one_hand_from_selected_targets(player)`：目标数由技能拥有者当前体力值封顶，并按主动技
提交顺序逐个选择、获得目标当前手牌。HOST_EFFECT 标签为 279，`EffectRequest` 稳定哈希标签
为 321，续体标签为 202。

ABI 289 增加视为牌过滤器 `owner_outside_turn`：只在技能拥有者不是当前回合角色时匹配合法
来源牌。SP「牺牲」以 `view_as tao response owner_outside_turn from hand_equip_expansion
uses_per_turn 1 exactly 2` 复用多牌视为、回合计数和普通濒死救援管线。

ABI 290 增加视为牌来源 `hand_expansion`，仅匹配拥有者的手牌区与任意扩展区，不包含装备区。
REFRESH「龙胆」以四条单牌视为规则精确表达旧 JavaScript 的 `position: "hs"`。

ABI 291 增加死亡过滤器 `owner_alive_is_source` 与效果
`choose_repolu_targets_draw(player)`。后者允许死亡拥有者取消或选择一至任意名存活角色，成功
选择后只增加一次同技能 ID 标记，并令目标按座次各摸此前标记数加一张牌。HOST_EFFECT 标签
为 280，`EffectRequest` 稳定哈希标签为 322，续体标签为 203。

ABI 292 增加主动目标约束 `unused_by_skill_this_play_phase`。它复用
`record_active_target(player, target)`，但将目标记录在独立的出牌阶段状态块中；每次进入
出牌阶段（含额外出牌阶段）时清空，不改变既有 `unused_by_skill_this_turn` 的整回合语义。

ABI 293 增加全局出牌阶段结束过滤器
`owner_alive_play_phase_card_category_count_positive`：存活技能拥有者只要在该精确出牌阶段
使用过至少一种基本牌、锦囊牌或装备牌类别即可触发；类别计数复用既有按玩家键控的阶段历史。

ABI 326 增加主动效果
`resolve_mbshanjia(player, lost_equipment_count, distance_skill_def)`。它先摸三张，再强制弃置
`max(0, 3-lost_equipment_count)` 张当前手牌/装备；弃牌中没有基本牌时提供一张不计通常次数、
但遵守当前距离规则的虚拟【杀】，没有锦囊牌时授予配置的出牌阶段全牌无距离技能。两项奖励
独立判断，弃牌移动及嵌套触发全部结束后才继续。装备离区数量复用 ABI 175 的
`owner_from_equip_per_card` 与同 ID 标记，不保存旧 JavaScript `actionHistory`。
HOST_EFFECT 标签为 309，`EffectRequest` 稳定哈希标签为 348，续体标签为 217；
Rule IR 提升至 116，Engine semantics 提升至 222。

ABI 327 增加主动选牌过滤器 `card_nonbasic`、主动目标约束 `has_gainable_equip`，以及效果
`resolve_daoji(player, target)`。目标约束复用统一的装备获得保护查询；效果选择并获得目标
当前可获得的一张装备，等待移动触发结束后在仍合法时强制使用。实体牌为武器时，无论装备
使用是否成功，都在该次尝试及嵌套触发结束后对原目标造成一点普通伤害。HOST_EFFECT 标签为
310，`EffectRequest` 稳定哈希标签为 349，续体标签为 218；Rule IR 提升至 117，
Engine semantics 提升至 223。

ABI 328 增加主动效果 `resolve_jiaohua(player)`。它先选择当前持久三类别循环中尚未记录的
基本、锦囊或装备类别，再选择任一存活角色；只从摸牌堆取得最靠上的对应实体牌。牌堆没有
对应牌时仍记录该类别，三类齐全后立即清空记录开始下一轮；记录不会随出牌阶段或回合重置。
HOST_EFFECT 标签为 311，`EffectRequest` 稳定哈希标签为 350，续体标签为 219；
Rule IR 提升至 118，Engine semantics 提升至 224。

ABI 329 增加主动效果
`resolve_luanqun(player, effect_skill_def, direct_hit_skill_def)`。所有有手牌的存活角色按座次
依次经过仅本人可见的强制选牌屏障，全部完成后才统一公开；身份规则集下拥有者可获得至多四张
与自身展示牌同色的其他展示牌，其他规则集上限为二。异色角色获得两个可合并多来源的临时助手：
首张【杀】只能指定对应乱群来源，使用后或首个出牌阶段结束时移除该助手；其本人回合内对乱群
来源使用的【杀】不可响应，第二助手在该角色回合结束时清理。HOST_EFFECT 标签为 312，
`EffectRequest` 稳定哈希标签为 351，续体标签为 220；Rule IR 提升至 119，
Engine semantics 提升至 225。

ABI 330 增加触发效果
`choose_give_two_different_colors_or_recover(owner, giver, target)`。交牌者可取消，或从当前
手牌、装备区交给濒死目标恰好两张有效颜色不同的实体牌；选择恢复时重新校验技能、三方存活、
目标濒死、牌快照、权属、区域与颜色。没有合法异色牌对或取消时，目标回复 1 点体力。
HOST_EFFECT 标签为 313，`EffectRequest` 稳定哈希标签为 352，续体标签为 221；
Rule IR 提升至 120，Engine semantics 提升至 226。

ABI 331 增加主动效果 `resolve_mbzhuguo(player, target)`。目标手牌少于
`min(max_hp, 5)` 时先摸至该值，并在摸牌及嵌套移动触发结束后检查是否并列全场最多；技能
拥有者可选择另一名角色，被调整者再选择一张当前实体或合法视为【杀】，最终目标集合必须包含
该角色，且本次使用忽略普通距离与次数限制。未摸牌的分支在完成可行的强制手牌弃置后回复
1 点体力。HOST_EFFECT 标签为 314，`EffectRequest` 稳定哈希标签为 353，续体标签为 222；
Rule IR 提升至 121，Engine semantics 提升至 227。

ABI 332 增加手牌弃置过滤器 `owner_first_hand_discard_has_sha_in_discard` 与效果
`resolve_mbanxian(player)`。过滤器按角色每回合的手牌弃置事件计数，只有第一次事件中仍在
共享弃牌堆的实体【杀】可供选择；若第一次没有【杀】，后续弃置不再触发。所选牌进入手牌并
完成嵌套移动触发后，有合法目标时强制走统一实体【杀】流程，忽略通常距离与次数限制。
HOST_EFFECT 标签为 315，`EffectRequest` 稳定哈希标签为 354，续体标签为 223；
Rule IR 提升至 122，Engine semantics 提升至 228。

ABI 333 增加主动效果 `resolve_mbjinzu(player, target, helper_skill_def)` 与助手效果
`apply_mbjinzu_next_sha(player, card)`。主动调用复用声明式单手牌、单目标选择，对方手牌超过
两张时私密选择两张，再统一公开；拥有者牌为最值时，双方展示牌按弃置原因分别移入共享弃牌堆，
嵌套移动触发完成后刷新主动技次数。拥有者牌不为唯一最值时，锁定临时助手按目标累计次数；
下一张【杀】在 `card_used` 暂停窗口一次性消费映射，只令实际命中的对应目标不可响应并分别
增加伤害，随后移除助手。HOST_EFFECT 标签为 316，`EffectRequest` 稳定哈希标签为 355，
续体标签为 224；Rule IR 提升至 123，Engine semantics 提升至 229。

ABI 334 将现有 `take_one_card(from, to, scope)` 扩展到
`hand_equip_judge` 作用域。新作用域复用统一的目标牌可获得性查询，选择前和续体恢复时均
校验角色存活、牌权属、区域与获得保护；没有合法牌时直接完成，使同一触发链中的后续效果仍
可执行。HOST_EFFECT 标签为 317，`EffectRequest::TakeOneScopedCard` 稳定哈希标签为 356；
Rule IR 提升至 124，Engine semantics 提升至 230。

ABI 335 追加卡牌选择器 `sha_diamond`，只匹配有效花色为方片的逻辑【杀】，用于
`card_distance_unlimited(sha_diamond)` 等既有卡牌感知 modifier。该版本不改变 Rule IR
结构、HOST_EFFECT 或 `EffectRequest`，Engine semantics 提升至 231。

ABI 336 增加通用公开展示整手牌效果 `reveal_hand(player)`（HOST_EFFECT 318，
`EffectRequest::RevealHand` 稳定哈希标签 357），以及触发过滤器
`owner_is_target_first_hp_change_turn_unique_trick_majority`。引擎仅在加载该过滤器时记录
每名角色本回合的实际体力变化次数；伤害、失去体力、回复体力及因上限下降造成的体力截断
共用同一计数。Rule IR 提升至 125，Engine semantics 提升至 232。

ABI 337 增加 `damage_prevented` 事件。正数伤害在提交前被取消或修正至零时触发；
`event_player` 为当前回合角色，`event_source` 为原伤害来源，目标保存在事件响应对象中。
回合外没有当前角色时 `event_player` 为 `NONE`。Engine semantics 提升至 233。

ABI 338 为通用玩家目标范围增加 `hand_lte_chooser`：只包含手牌数不大于选择者的其他
存活角色，并在 Choice 恢复时重新校验。`choose_targets_draw` 与
`choose_targets_damage` 共用该范围。Rule IR 提升至 126，Engine semantics 提升至 234。

ABI 339 增加通用触发过滤器 `owner_is_target_event_card_sha`：技能拥有者必须为
`event_player`，且 `event_card` 的有效牌名必须为【杀】；在 `hand_compare_finished` 中，
该牌固定为发起者的拼点牌。Rule IR 结构不变，Engine semantics 提升至 235。

ABI 340 增加伤害前触发过滤器
`owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn`：每回合限一次，
技能拥有者须为伤害来源，目标存活且座次更靠前，同时拥有者手牌数小于目标体力上限。
Rule IR 结构不变，Engine semantics 提升至 236。

ABI 341 增加通用转化过滤器 `owner_single_hand`：技能拥有者须恰有一张手牌；选中的
实体牌仍由 `source_zone` 校验，因此可与任意转化牌名复用。Rule IR 结构不变，
Engine semantics 提升至 237。

ABI 342 增加改判修正器 `judgment_rewrite_red_hand_equip_expansion`：技能拥有者可用
手牌区、装备区或扩展区的一张有效红色牌替换判定牌；与其他改判能力叠加时按每张牌分别
校验免费或标记费用通道。Rule IR 结构不变，Engine semantics 提升至 238。

ABI 343 将既有主动效果 `use_virtual_card_on_selected_targets` 扩展到【铁索连环】：
声明须选择 1–2 名不同的存活角色，可以包含使用者自己；宿主生成无花色虚拟牌后复用普通
目标策略、无懈与横置结算。原有多目标【决斗】继续保持 ABI 95。Rule IR 结构不变，
Engine semantics 提升至 239。

ABI 344 增加触发过滤器
`owner_source_or_targeted_club_linked_exists_once_per_turn` 与效果
`draw_linked_players(count)`：同一技能实例的使用牌/成为目标触发共享每回合次数；效果按
座次令结算时仍存活且横置的角色各摸指定张数。HOST_EFFECT 标签为 319，
`EffectRequest` 稳定哈希标签为 358；Rule IR 提升至 127，Engine semantics 提升至 240。

ABI 345 增加 `owner_source_sha_single_trick_source_card`、
`owner_source_sha_single_equip_source_card` 与
`owner_is_target_hand_above_hp_no_liubei_lord` 三个触发过滤器：前两者读取权威【杀】来源牌，
只匹配恰好一张有效锦囊牌或装备牌；后者保留“手牌多于体力且场上刘备主公不存在”的觉醒
条件。复用既有返还【杀】次数、增加当前牌伤害、减体力上限与获得技能效果。Rule IR 结构
不变，Engine semantics 提升至 241。

ABI 346 增加 `owner_alive_other_dying_first_per_target`：技能实例按角色永久记录已经观察过的
濒死事件，只在每名其他角色首次进入濒死时触发；记录参与确定性状态哈希。Rule IR 结构不变，
Engine semantics 提升至 242。

ABI 347 增加按角色记录的本回合受伤次数，以及首/第二次受伤过滤器；Engine semantics 提升至 243。
ABI 348、349 分别增加 `choose_scskuiji_discard`、`choose_pskuiji_discard` 可恢复选择；
Engine semantics 依次提升至 244、245。ABI 350 增加 `discard_hand_equip`，Engine semantics 提升至 246。

ABI 351 增加 `owner_is_target_current_turn`：拥有者须存活、为事件角色且为当前回合角色；
用于精确表达只在自己回合防止自身伤害的技能。Rule IR 结构不变，Engine semantics 提升至 247。

ABI 352 增加 `reveal_top_gain_if_color(player, red|black)`（HOST_EFFECT tag 323）：公开展示牌堆顶，
按印刷花色判定颜色，匹配时收入手牌，否则置入弃牌堆。`EffectRequest` 稳定哈希标签为 362；同时
将 ABI 350 的 `discard_hand_equip`、ABI 348/349 的两项试探选择效果分别固定为 359、360、361，
避免复用旧标签。Rule IR 提升至 128，Engine semantics 提升至 248。

ABI 353 增加 `owner_source_target_distance_le_one_first_damage_to_target_this_turn` 与
`sha_response_count_if_source_owner_target_distance_le_one`：前者读取按来源和目标分组、回合边界
清零的实际伤害事件计数，后者按每个【杀】目标的最终距离增加所需【闪】数。Rule IR 提升至 129，
Engine semantics 提升至 249。

> **ABI 日志维护约定（2026-08-21 起）**：自 ABI 354 起的逐号变更不再回填本节。
> 权威的编号领取/落地记录见根目录 `ABI-COORDINATION.md`（ABI-MAX 与
> `RULE_VM_ABI_VERSION` 由 `cargo test -p engine-api abi_coordination` 钉住）；
> 逐武将的 ABI 落地流水见 `content/original/ported-character-log.md`。当前版本三轴见上文
> 「编译流水线」一节。

## 能力画像

`skill-capability-matrix.json` 是完整机器可读矩阵；`skill-capability-matrix.md` 是聚合摘要。
当前由 `legacy-analyze matrix` 从 standard 与 shenhua 源码重建，覆盖 243 个技能，且明确
标记 Trigger Point、Choice/Cost、Subskill State、Intent/Effect 和禁止 Host 的待办；它只做
源码清单与交叉校验，不替代逐技能语义 Golden。全仓活清单（计数以该文件为准）见
`content/original/skill-capability-matrix.md`，目录口径以
`content/original/catalog.json` 为准。
