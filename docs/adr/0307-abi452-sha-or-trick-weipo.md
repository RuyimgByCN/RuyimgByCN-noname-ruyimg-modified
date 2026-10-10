# ADR 0307：ABI 452 目标过滤器 + SP2 威迫

本批落地 C 段第三个 ABI（452）：`owner_is_target_sha_or_trick` 触发器过滤器，
并以此移植 SP2 韩馥的「威迫」（weipo）摸牌部分。

## ABI 452：`owner_is_target_sha_or_trick`

新触发器过滤器：拥有者成为当前事件的【杀】或锦囊牌目标。复用 `card_is_sha` 与
`card_is_trick` 既有判定（含延时锦囊，与旧 JS `get.type(event.card) == "trick"` 一致）。

改动面（全部追加式）：
- engine-core trigger/catalog.rs：`TriggerFilter::OwnerIsEventPlayerEventCardShaOrTrick` 变体
- trigger/dispatch.rs：分发臂 `owner_is_player && (card_is_sha || card_is_trick)`
- content/registry.rs：字符串 → 变体映射
- content-compiler typecheck.rs 过滤器列表 + `required_rule_vm_abi` 452 门控
- `RULE_VM_ABI_VERSION` 451→452（ABI-COORDINATION.md 已领取 ABI-MAX 452）

该过滤器有 33 个消费技能（weipo/tianming/danlao 等），为高复用原语。

## 技能移植

「威迫」（`weipo`，3483198633，SP2）：成为其他角色使用的【杀】或锦囊牌目标后，若
手牌数不足 `min(5, 体力上限)` 张则补足。DSL 以 ABI 452 过滤器 + `min()` 表达式
（ABI 451）+ `adjust_hand_to` 表达。

注：旧 JS 为 forced 锁定技并记录补足前手牌数供 weipo2（用牌后手牌减少则目标弃牌）使用；
weipo2 的跨事件记录未移植，本定义为威迫的摸牌部分（沿用 moon_jushou 先例）。
韩馥（`hanfu`）另一技能「截营」（hfjieying）依赖 bound-target 临时技，暂不发布该武将。

## 验证

- engine-api 4 + engine-core 54 测试全过。
- weipo 打包正确报告 ABI 452；sp2 npack 重建为 48 技能/5 武将/ABI 452；
  README 构建命令与部署 npack 逐字节一致（logic_hash f8a058a2）。
- 26 包 `import-extensions` 全量导入通过。
