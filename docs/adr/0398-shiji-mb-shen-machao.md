# ADR 0398：SHIJI 神马超闭环（驭雳 / 霆威，ABI 409 / 410）

本批发布 SHIJI 神马超（`mb_shen_machao`，2744541689）。三个技能「驭雳」（`yuli`）、
「霆威」（`tingwei`）与既有「寂灭」（`jimie`）完成闭环；旧 JavaScript 仅作为语义
来源，不进入运行时。

## 决策

- **ABI 409 `resolve_yuli(player, mode)`**：mode 0（自己造成伤害前）非雷伤害
  prevent + 同源同量同牌雷属性重放（`DealCardDamage`），已是雷则伤害 +1；mode 1
  （自己受伤前）仅雷伤害被取消并摸伤害点数张（对齐旧 JS `damageBegin4` 的雷属性
  filter）。重放复用 ABI 374 伤害重放管线（`prevented_damage_ids` +
  `damage_redirect_in_progress`）；mode 0 在重放期间跳过防递归，mode 1 是重放链的
  合法观察者（不跳过）。
- **ABI 410 `resolve_tingwei(player, mode)`**：杀指定首个目标后（新过滤器
  `owner_source_sha_first_target`）加 4 枚霆标记并打开「选择该杀一个目标」choice；
  所选目标从四选一（非锁定技失效至其回合结束 / 交给拥有者一张装备牌 / 此杀对其
  伤害 +1 / 随机弃一张牌）中选择，取消则横置。两段 choice 用
  `ContinuationKind::Custom` + payload[0] 魔法标记（`RESOLVE_TINGWEI_SELECT` /
  `RESOLVE_TINGWEI_OPTION`）驱动。
- 霆标记定义 ID 4145075227 与既有寂灭共用；神马超技能顺序对齐旧 JS
  （yuli/tingwei/jimie）。
- 顺手补齐并行会话（omp，ABI 523/524）缺失的 `effect_tag` 常量（460/461/462）与
  `resolve_zhiba` 返回类型，并将 `RULE_VM_ABI_VERSION` 522 → 524 以容纳其已打包的
  524 内容包。

## 验证

- 新增 `original_shiji_yuli_tingwei.rs` 7 个行为测试：普通伤害改雷（交叉验证：
  目标同为驭雳拥有者时改雷触发其防御）、雷伤害 +1、普通伤害不触发防御、雷伤害
  取消+摸牌、霆标记+选目标 choice、取消横置、非锁失效选项。
- 全量门禁通过（golden 向量无漂移——identity 对局未触发新原语路径）；
  仅剩 refresh 包 RE 黄月英测试失败（Codex B 段活跃领域，ABI 292→356 断言过时）。
- 打包：46 技能 / 10 武将，`rule_vm_abi_version` 502（≥ 410 满足）。
