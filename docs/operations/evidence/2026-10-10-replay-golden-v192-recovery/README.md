# 2026-10-10 replay golden 重录（v191/v192 事故恢复补齐）

## 背景

golden 原值最后重录于 v188（ctianfu 批，2026-10-10-xianjian-ctianfu-batch）。
其后两批只 bump 未重录：

- v191（勾魂锣/蓄谋族批）：`gouhunluo_rounds`/`gouhunluo_source` 入 canonical_hash。
- v192（机关桶批）：版本 191→192（`STANDARD_RULESET_VERSION`）。

v192 批开发中 host.rs 被误清零，重录未及进行；事故恢复全程（405→524 passing）
期间 replay_scenarios 1 failed 保持挂账。本轮其余全部测试转绿后补齐重录。

## 原因核验（重录前）

1. **版本回退探针**：`STANDARD_RULESET_VERSION` 临时 192→188 复跑，14 场景仍全
   mismatch → 位移不止版本号，含 v191 哈希字段扩展（与上述构成一致）。
2. **轨迹语义探针**（sha_dodge_then_damage 手工复跑）：杀 28 置桌面 → 闪 32 响应 →
   弃牌堆序 [..., 32, 28]、双方 hp 不变 —— 与场景语义一致。
3. **行为面核验**：重录前全量 `cargo test --offline -p engine-native --no-fail-fast`
   = **524 passed / 1 failed（仅 replay_scenarios）/ 1 ignored**，其中 g1_wudian
   112/112、g1_shahai_mtg 11/11 当轮修复后全绿。

## 重录

- 命令：`cargo test --offline -p engine-native --test replay_scenarios`（2/2 通过）。
- 逐场景命令与断言不变，仅替换 SCENARIOS 表 14 个 golden hex 并在测试文件头
  追加重录依据注释（v192 事故恢复收尾）。

| 场景 | 旧（v188 era） | 新（v192） |
|---|---|---|
| sha_dodge_then_damage | 4b2881d5ecf4c58a73d84628c346ac22f06ee6322a8e29a7719640f4ef6c04dc | de821172ccd077335e0c6d78ab1f1043e516b759eadc2962b2044d9fc9079fe6 |
| death_terminal_single | 49178726a487085fdf1b480ff5649ffb6d2148edf0703f8d34acccfd37b944a3 | 480a0db9bf9d0232cd426340a55569de94ca19246c029bc93e35ebb0d5cb6ca1 |
| sancai_nested_chain | 7241c447fc2cb60afe24d8f996054635381c8288832705055d5245fb4cb503ce | 3811485ded73d6e39eca5c19de7c9e1e80933a153887fcfaf3fc3a0bd852ac02 |
| yishang_redirect | 8b7360375f1804ca3a2bd33a0a1c891d79b785f0311e4c1eef91515f2c930114 | 68ec52583a95e7f3a055450cf155cbb5d651306ba81243a2d51e68c991cbf960 |
| suidun_window_matrix | f9b5ad403cca92e5b1141e994668d4e0ee4f05ea5d20b3e5023c0a2c042cb85c | 76af0c1edcea386283011364e641628f15b59a7b66333e44e22e97c46eae3336 |
| shouyi_dynamic_skill | 12194fc74e6efd82a9f5c8f749446ab82af0e875de67b0bcb4376cd33944e352 | 41030c95c97a65eb1083c76ffcc041c24ca9142fe4fc925079968bcded043563 |
| discard_to_limit | de454f886af4ce3c2bb98286688768426a3b8aaf5a2fe5d17bc4cd339c59d810 | 27a62e35caa108cc53110d9272916d9ff28c8a215aff29c5f50352cc486f892d |
| rollback_invariants | 75c3103916abfc5a50dd6b4a336431a125a9dd6c8a68db75e812ccc93a83f311 | 6cf66d8957a30eb882feb0011e4b333bf4712b628f343147a6de6baf8bf284d4 |
| equip_distance_matrix | cd9137024585b16651c35a91d178c421c631247364a0cb4f48ed8dcfb4c5f702 | 187ccc31bd2b7c54f2d635652e1dc37a4f0f141b4c41d9c0536c7576215290bf |
| chain_fire_propagation | 0d732ec47e35e83b0da1694f10910e1f876344c68a7f6aeb1066652534b90d21 | 79b3f01c2c7652bcf08a53a8d738cbdfec1ee7d67f81419eb2b85b86c09ed521 |
| aoe_response_queue | 3ccdb6e3cb211fd88a589434ec304849d6e01560bc9a8eb1330a8f9064eb347a | 893ccc8947b0df249f39871b7cc14158aa4ca5eb5c4d0970b99e8915d9eedf55 |
| duel_alternating_response | b2938b0df3b913deb261625bc123a316fe9534263c1413570a32c26fd03df55f | feee481b4111aa7ada6398bf320b71e9e921d3b76947f5009b63f5a8d44d53ae |
| nullify_chain_on_savage | c43681bb34ba98858774ee3382eecd20eea77ef4bddd142d264b677b7123063c | a62ab86f800ee1fc52318ff96ec9c0e283c595b19079cb711448ccb282fe00d2 |
| judge_phase_delayed_tricks | 2c7cf18cc6ac6d73a53d799606eb7bc1fa3c1b26ad5c3501cde8a56414e56994 | 871a4eb15bed01720492755bebad4d5eaf77d92a74e28e6b93e07e4427a846ba |

## 同轮修复项（重录前置，wudian 8 + mtg 3）

- ZhuchengGuard 代抽：`hand.remove(idx)` 后再 `discard_from_hand` 双重移除 panic
  → 改读下标交 discard_from_hand。
- Fenglue：机内 take + emit CardUsed → commit 二次 take panic；恢复原语义
  （emit 交 commit 取牌，逐张循环去掉手牌守卫）。
- Liangou：TargetChoice 误读玩家应答（应读牌应答存 self.cards，Act 读目标）
  → 恢复 pre-incident 三段。
- Feitiao：TargetChoice/Act 塌缩单臂 → 恢复独立 Act 臂（快照恢复 stage=Act
  原先落入 `_ =>` 空臂）。
- Qianfang/Liguang：TODO stub → 按 ops 原臂恢复（Liguang 日志 CardMoved→Equipped，
  LogEvent 无 CardMoved 变体）。
- Xuanmo：activate 臂缺失（InvalidChoice）→ 按 batch-32 op 恢复。
- mtg：早四地（龙颅/海岛/水没/废墟）补 `sync_land_skill` 首行；sync_land_skill
  恢复三映射（依夏兰→重铸/林中尖塔→塔击/幽林→发现）；幽林地/塔击/林中尖塔地
  emit 前实体牌入手（commit 从手牌取牌适配）。
- 清理 4 处 DBG eprintln（Lingyu/Diewu/Ctianfu/Jiguantong filter/step）。
