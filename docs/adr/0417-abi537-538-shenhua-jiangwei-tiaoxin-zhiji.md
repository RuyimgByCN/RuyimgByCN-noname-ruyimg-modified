# ADR 0417 — shenhua 姜维【挑衅】【志继】(ABI 537-538)

- 状态:已接受
- 日期:2026-08-02
- 归属:C (omp)
- 关联 ADR:0412(邓艾)、0401(祝融)
- ABI:537(挑衅)、538(志继)

## 目标

发布 shenhua 姜维完整武将:

- **挑衅**:出牌阶段限一次,你可以选择一名攻击范围内含有你的角色,令其对你使用一张
  【杀】;若其不如此做,你弃置其一张牌。
- **志继**:觉醒技,准备阶段,若你没有手牌,你摸两张牌、回复 1 点体力、减 1 点体力
  上限,然后获得「观星」。

## 决策

### 挑衅 (ABI 537)

- DSL:`active play_phase uses_per_turn 1` 承载「出牌阶段限一次」契约,效果
  `resolve_tiaoxin(self)`。
- 目标筛选(在复合原语内权威校验):其他存活角色,其攻击范围含持有者
  (`player_in_attack_range(state, target, owner)`,与旧 JS `target.inRange(player)`
  一致),且有可弃置手牌/装备;无合格目标则不启动。
- 流程(自定义续体 `RESOLVE_TIAOXIN_CUSTOM_CHOICE`,0xDCDC_5370):
  1. 持有者选目标;
  2. 目标二选一(使用杀 / 拒绝,可取消=拒绝);
  3. 选杀源(`prompted_sha_source_cards` 复用乱武受迫杀源,可取消=拒绝);
  4. 受迫杀以 `play_prompted_sha_source_unlimited` 对持有者结算(忽略距离与次数,
     与乱武先例一致,文档化近似);
  5. 拒绝时持有者选目标一张手牌/装备弃置(复用既有 MoveCards 管线)。
- 每个 Choice 恢复时重校验目标/牌合法性。

### 志继 (ABI 538)

- 过滤器 `owner_phase_begin_hand_empty`(准备阶段、持有者回合、无手牌),强制触发。
- 复合原语 `resolve_zhiji(player, guanxing_def)`:复用通用 `DrawCards`/`RecoverHp`/
  `ChangeMaxHp(-1)`/`AddSkill`;已持有观星时幂等返回(觉醒一次性)。
- 观星授予标准包既有定义(def 32,standard_m1);旧 JS 派生「界观星」为版本差异,
  以标准观星近似(文档化)。

## 接线

`ResolveTiaoxin`/`ResolveZhiji` 变体 → 操作码 484/485(表尾追加,避开 Codex
481-483)→ vm 弹栈 → vm_host 注入技能实例 → primitives
`resolve_tiaoxin`(阶段 0 开目标选择)/`resolve_zhiji` → 自定义续体标签
`RESOLVE_TIAOXIN_CUSTOM_CHOICE` + resolve_choice.rs 五段解析器 → hash 臂
505/506 → DSL 词 `resolve_tiaoxin(player)`/`resolve_zhiji(player, guanxing_def)`
→ IR/codegen/门控链 `ADDITIVE_RULE_VM_ABI_VERSION_537/538`;过滤器在
catalog/dispatch/registry/typecheck 四处登记。

## 测试与发布

- Golden `original_shenhua_jiangwei.rs` 6 项:契约(ABI 538/active 契约/触发过滤)、
  挑衅拒绝弃牌、挑衅受迫杀命中持有者、目标筛选(无牌目标排除)、志继空手觉醒
  (摸2/回1/上限 5→4/授观星)、有手牌不觉醒。全绿。
- 40 个既有 shenhua 二进制 + 姜维零失败;engine-core 全量 0 失败;conformance
  8/8 ALL PASS。
- npack:ABI 538、技能 110、角色 30(姜维 id 69634190,蜀 4 体力 男)、触发 72、
  模块 84、active 12;`extensions/shenhua/{shenhua,package}.npack.json` 三副本一致。

## 近似与遗留

- 受迫杀忽略距离与次数(乱武先例);挑衅目标的攻击范围条件在选目标时校验,
  出杀阶段不重复校验。
- 观星以标准版近似旧 JS 的界观星派生。
- 拒绝弃牌范围为手牌/装备(旧 JS `he`),不含判定区。
