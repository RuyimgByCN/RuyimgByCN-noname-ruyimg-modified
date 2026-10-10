# ADR 0401 — shenhua 祝融【巨象】【烈刃】(ABI 530-531)

- 状态:已接受
- 日期:2026-08-02
- 归属:C (omp)
- 关联 ADR:0400(袁术)、0398(王平)、0395(孙策制霸)
- ABI:530(过滤器与既有原语批次)、531(`resolve_lieren`)

## 目标

发布 shenhua 祝融完整武将:

- **巨象(锁定)**:南蛮入侵对你无效;其他角色使用的南蛮入侵结算完毕后,若其仍
  在弃牌堆,你获得之。
- **烈刃**:你使用【杀】对一名角色造成伤害后,可与其拼点;赢则获得其一张手牌,
  否则无效果。

## 决策

### 巨象

- 「南蛮入侵对你无效」以通用修正器 `card_target_cancel(nanman) override 1` 表达,
  与既有万箭/南蛮目标取消修正器同源,无新机制。
- 「结算完毕后若仍在弃牌堆则获得」以 `card_resolved` 触发 +
  `gain_card_if_discard(self, event_card)` 表达。旧 JS 挂 `useCardAfter`;引擎的
  `CardResolved` 即完整结算边界(ABI 503 通用事件),`gain_card_if_discard` 保证
  实体牌仍处弃牌堆时才移动,与旧 JS 的「仍在弃牌堆」条件一致。
- 新过滤器 `owner_alive_other_nanman_cards_discard`:持有者存活、他人使用的
  南蛮结算完成且事件牌仍处弃牌堆。ABI 530 登记。

### 烈刃

- 拼点目标必须是「受伤害角色」,因此不能复用 `choose_hand_compare`(目标自由选择)。
- 新增 ABI 531 原语 `resolve_lieren(player, target, win_skill_def, lose_skill_def)`:
  复用 `ChooseHandComparePlayerCard` 通用拼点选牌续体与「胜者获绑定辅助技能」管线,
  以固定目标启动;胜利后绑定辅助技能 `lieren_win` 在 `hand_compare_finished`
  触发 `take_one_card(event_source, self, hand_equip)`;平局/失败挂
  `lieren_lose`(仅移除自身)。主动技专属的 `choose_compare_selected_hand_card`
  需要选中牌,触发内无法使用,故需专用启动原语。
- 触发过滤器 `owner_source_sha_damage_target_alive_both_hand`(杀伤害且双方均有
  手牌)与 `owner_alive_skill_bound_target_is_event_source`(绑定目标为拼点第二
  参与者)在 ABI 530 登记。

## 接线

`ResolveLieren` 变体 → 操作码 469(表尾追加,避开 Codex 468 `RESOLVE_XINMINGSHI`)
→ vm 弹栈(ConstI32 先压 lose/win,再压 target/player)→ vm_host 注入技能实例 →
primitives `resolve_lieren`(校验持有技能/存活/双方有手牌后打开持有者选牌续体)
→ hash 臂 502 → DSL `resolve_lieren(player, target, win_def, lose_def)` →
IR/codegen/门控链 `ADDITIVE_RULE_VM_ABI_VERSION_531`。

## 测试与发布

- Golden `original_shenhua_zhurong.rs` 4 项:契约(触发/过滤器/ABI)、巨象结算
  后获牌、烈刃胜获一牌、烈刃负无效果。全绿。
- 40 个 shenhua 测试二进制零失败;conformance 8/8 ALL PASS;rule-vm/engine-api/
  content-compiler 492 项零失败。
- npack 重建:ABI 531、技能 105、角色 28(zhurong 完整武将,id 2674218755,
  技能 [巨象 4234952102, 烈刃 3941156292],蜀 4 体力 女)、触发 69、模块 80;
  `extensions/shenhua/{shenhua,package}.npack.json` 与源三副本一致。
- 顺带对齐 `original_card_use_history.rs` 的 refresh 包陈旧断言(292→356,
  该包由其他 agent 于 08-01 重建,非本批次回归)。

## 近似与遗留

- 巨象以 `card_resolved`(结算完成)近似旧 JS 的 `useCardAfter`,语义等价。
- 烈刃胜者的取牌范围为手牌/装备(`hand_equip`),旧 JS 亦如此。
- 拼点流程(选牌、揭示、胜负、绑定辅助)完全复用通用拼点管线,无重复实现。
