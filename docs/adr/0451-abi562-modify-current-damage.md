# 0451 ABI 562 通用当前伤害修改 + yijiang/newjiang 批次（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

yijiang/onlyOL/newjiang 的多个「伤害±N」技能（lianhuo/dckanghui/renjia/oljiaojin 等）
在 damage_before 时点无法用 `increase_current_card_base_damage`（该效果仅限 card_used
挂起结算窗口,damage_before 调用会 ContentFault）。本批实现 E 段首个原语 ABI 562。

## 变更

### 引擎（ABI 562,零 conformance 重钉）

`modify_current_damage(player, delta)` 效果：damage_before 窗口内对当前伤害量增/减
delta（负值逐点钳制到 0,零值空操作）,复用既有 bound-damage 管线
（increase_bound_damage / reduce_bound_damage_by_one）。

| 文件 | 变更 |
|---|---|
| `rule-api/src/effect.rs` | 追加 `EffectRequest::ModifyCurrentDamage { player, delta: i32, skill }`（枚举末尾,序列化序号稳定） |
| `content-schema/src/ir.rs` | 追加 `EffectIr::EffectModifyCurrentDamage` |
| `content-compiler/typecheck.rs` | 注册效果名 + 参数校验 + damage_before 窗口限制 |
| `content-compiler/codegen.rs` + `lib.rs` | codegen + ADDITIVE_RULE_VM_ABI_VERSION_562 |
| `rule-vm/bytecode.rs` + `vm.rs` | effect_tag 528 + 解码 |
| `engine-core/primitives.rs` + `hash.rs` | modify_current_damage 实现 + hash 分支 |

### 技能（2 个,复用 ABI 562）

| 技能 | 包 | DSL | 近似 |
|---|---|---|---|
| `lianhuo`（连火） | yijiang | `damage_before owner_is_target_linked` → `modify_current_damage(self, 1)` | 「非传导/仅火焰」未建模（无事件属性查询）,横置时全伤害+1 略强 |
| `dckanghui`（亢惠,袁术） | newjiang | `damage_before owner_is_target_other_source_alive` → `modify_current_damage(self, 1)` | 「双爻标记均未翻转」未建模（需妄爻联动） |

## 落地

- yijiang 122 技能 / 27 武将 / ABI 562；newjiang 17 技能 / ABI 562。
- `extensions/{yijiang,newjiang}/` 同步（package.npack.json 同源,README 更新）。
- `cargo run -p conformance`：8/8 ALL PASS,无向量重钉。

## 验收

- content-cli pack 全量编译通过（含 damage_before 窗口校验）。
- 引擎 5 crate cargo check 通过；conformance 全绿。
