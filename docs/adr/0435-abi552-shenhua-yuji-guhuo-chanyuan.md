# ADR 0435 — shenhua 于吉【新蛊惑】【禅缘】(ABI 552)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0431(张角)、0425(毌丘俭)、0421(孟获)
- ABI:552(禅缘)

## 目标

发布 shenhua 于吉完整武将:

- **蛊惑**(新):你可以将一张手牌扣置于武将牌上,声明为任意基本牌或普通锦囊牌使用
  或打出;其他角色依次质疑,无人质疑按声明生效,有人质疑验真并结算。
- **禅缘**(锁定):你不能于蛊惑的结算流程中进行质疑;当你的体力值为 1 时,你的
  其他技能失效。

## 决策

### 蛊惑(纯 DSL,零新引擎)

复用引擎既有 `view_as ... cost guhuo` 质疑流程(与 `old_guhuo` 同机制):
`view_as basic any any from none cost guhuo` + `view_as ordinary_trick any any from
none cost guhuo`;声明范围(任意基本/普通锦囊)由引擎蛊惑选择器覆盖。

### 禅缘 (ABI 552)

- 新修正器 `skills_disabled_when_hp_one`:锁定技,持有者体力值为 1 时其非锁定技能
  失效。`skill_effectively_enabled` 增加钩子:存在该修正器(owner 匹配)且持有者
  hp == 1 且被评估技能非锁定时返回禁用。
- 「蛊惑结算中不能质疑」未建模(引擎质疑参与者选择不含该排除,文档化近似)。

## 接线

修正器目标 `SkillsDisabledWhenHpOne` 在 modifier.rs 枚举尾追加 → registry 双向词
`skills_disabled_when_hp_one` → typecheck MOD_TARGETS → skill_effectively_enabled
钩子(只读查询,无状态变更)→ 门控链 `ADDITIVE_RULE_VM_ABI_VERSION_552`(按修正器
词表判定)→ version 552。无新操作码/效果变体(纯修正器批次)。

## 测试与发布

- Golden `original_shenhua_yuji.rs` 5 项:契约(ABI 552/view_as guhuo 成本/修正器/
  锁定)、蛊惑声明杀无质疑命中、真实声明被质疑验真结算、禅缘 hp1 禁用蛊惑且禅缘
  自身仍启用、hp1 时蛊惑使用被拒。全绿。
- 46 个 shenhua 二进制零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 552、技能 121、角色 34(于吉 id 3952134131,群 3 体力 男)、修正器 45、
  view_as 13;`extensions/shenhua/{shenhua,package}.npack.json` 三副本一致。

## 近似与遗留

- 禅缘「蛊惑结算中不能质疑」未建模。
- 角色池增长改变身份随机分配的座次角色:jushou 测试(旧用例)对下一家准备阶段
  的可选触发(甄姬洛神)增加拒绝分支,保证与角色池无关。
