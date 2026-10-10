# 2026-09-24 P0 回放与目录核验（v60 golden 重录 + A0 目录构成检查）

本证据描述主稿 P0「回放与目录」的修复与验收：查清 14 个场景 v59→v60 哈希差异的
成因、修复 `standard_catalog_parts` 拆牌循环缺陷、按候选 A0 第 2 点落地目录构成
声明表检查、确认语义后重录 14 golden，并给出全量原生测试通过证据。

> 2026-09-24 审查补注（[审查修复证据](../2026-09-24-review-fixes/README.md)）：「结论边界」中两处
> 现有实证。A0 目录检查在缺陷源码（规则 59、拆牌循环缺陷还原）上 2/2 报红（88 ≠ 80、
> 顺手牵羊 8 ≠ 4），「先失败」口径补齐；同一源码配 v59 golden 期望值时，14 个场景哈希
> 逐一复现第 48 轮 golden，差异确由版本字段与牌堆修正构成。

## 检查与结果

| 检查 | 结果 |
|---|---|
| 源码版本 | STANDARD_RULESET_VERSION=60、ARCHIVE_FORMAT_VERSION=2（未递增：本批无规则/存档语义变化，见下） |
| 基线校验 | `shasum -a 256 -c ../2026-09-24-native-progress-audit/source-hashes.txt` 4/4 一致（开工时源码与该证据同源） |
| 目录缺陷确认 | `standard_catalog_parts` 拆牌 `for _ in 0..2` 循环体误含顺手牵羊 ×4、酒 ×2、雷杀 ×2 的入目录语句，实际产出顺 8/酒 4/雷 4 共 88 张；修复后 80 张（`tests/standard_catalog_inventory.rs` 声明表锁定） |
| A0 目录构成检查 | 新增 `standard_catalog_matches_declared_inventory`、`defective_kinds_hold_declared_counts`：目录牌名×数量×花色点数与显式声明表逐一相等 |
| 14 golden 重录 | 命令脚本与逐场景断言未改，重录依据见下表；`replay_scenarios_match_golden` 2/2 通过 |
| 定向测试 | `cargo test --offline -p engine-native --test replay_scenarios` 2/2；`--test standard_catalog_inventory` 2/2；`--test g1_shahai_gujian` 13/13 |
| 全量验收 | `cargo test --offline -p engine-native`（共享 target，标准命令）**338 通过 / 0 失败**，完整输出见 [native-tests-full.log](native-tests-full.log)；含新增 A0 测试 2 项 |

## 14 个场景哈希差异的核因（v59 golden → v60 新 golden）

差异由两个已定位来源构成，无未解释项：

1. **规则版本字段**：`ruleset_version` 59→60 显式进入 `canonical_hash`
   （`engine.rs` `push_u64(&mut h, c.ruleset_version)`）——仅此一项即改变全部 14 个哈希。
   v60 行为增量（祖墨手铠/封印）只经 `CardKind::Zumoshoukao`（枚举末位追加）的
   gwent 分支（`kernel.rs` 锦囊牌种 match、`engine.rs` 使用与恢复校验分支），
   标准 14 场景目录不含该牌、分支不可达；`snap.rs`/`state.rs`/`event.rs` 自第 48 轮
   未变。行为级证据：v60 下 14 个场景脚本全部跑通、中途断言（收敛、持牌数、
   技能到期等）全部通过，命令与事件/状态轨迹语义与 v59 一致。
2. **目录构成修正**：v59 的 golden 录在 88 张缺陷态牌堆上（顺 8/酒 4/雷 4）。
   修复为 80 张后建局洗牌与初始发牌改变，14 个哈希再次改变。这是缺陷修复而非
   语义取舍，声明表见[规则基准](../../../architecture/native-rules-baseline.md#标准目录构成声明表)。

| 场景 | v59 golden（第 48 轮） | 缺陷态 v60（修复前实跑） | 修复态 v60（新 golden） |
|---|---|---|---|
| sha_dodge_then_damage | a6f61e3ebe8bd5d771b33038b278e2e70b10c9083b5d5f86d47ddd343165da45 | 3dc5bed80213ce4129303867763876ef048ad5381ec83858ff2c867f37d250f6 | 70047618df51430933d7da6a1cc483feeb211a8c87e60a0200dd94caf46d4750 |
| death_terminal_single | 1f3212c829a0c1dc30d57ee3b10ffdfd6a60044e5d861ec28490da6df1cc3247 | f5117f6f68ad2aeeff07ba4ea1d1e7ed263ad887c402d0d2b946f7ba4361d439 | a5261bfa44b48ddc7ff9497c8598630e74761a639ce7017a0f61727b257016d2 |
| sancai_nested_chain | 925bf38ef2616a5a1d17f2adb09674f7265db9fb3420bd64e55d72761517b52b | 2d5d56840d0ba8c22e1565f702213e9c92038eb438ca13e9690611789cbd76f9 | bd797c707b910fd63d5bf96edc00c7874ccad261da27fc3b4fb92acb066cf822 |
| yishang_redirect | 8d7df18900fb774590bc47e1c6e8f77deece212fcdb77b6388a49eccde4b1789 | 4604f7960b7602557cad155ba987b40093e260f57a8efd30eb5894dd6c14ecb8 | f5592998e03f6475dc109a9581ab56f38e5169c3fd2083ef02ba155a2af53eb2 |
| suidun_window_matrix | 5547ac325199c4cacb1354e50bb47b82a0d474719127ce1ed6aafecd18aa91be | 2856ed3082d7373195cff6bef083757ccce08572809bf0768da878dea1c848f9 | d5a70bfa8d475af351abbbb7f3b728221406a60d4359871457ffd2576f931a9c |
| shouyi_dynamic_skill | 77d644e73d94e4be4bcca526fc94c6df64364c9d57ab2c765546c017a9057909 | 58d462d56669a6e5d531a22c1c8c4c1a95571865f54c99e7a4e955bd2ec039d7 | 1a28a100386d0f90a0210591e3acca1cb5f205f4eb3adf1e2efb0d23bebd9001 |
| discard_to_limit | 58d64ad8ee3908dc60d4a5b9ecb3507fcc554d5ae9fd1bfeea21effe355771ab | 70a1f05b8ec6fd9d7583fb01e1559576c49fff85ca7b7af21228dfbf84eb3c0d | 868a934f1aa8002dee41c224ed504777ccbb3286097f9a55a4b0712e12f599e4 |
| rollback_invariants | a0f30e695442fda6b5a4e4bf7cb922ec2b2f790320556ad425071c63eb01f8ae | cde79adb55851d13e5b356f4abc5c3bcc5efc326e4f74adc3287b95a4e447737 | ad4e304a6c088d689ea4c6595c683bbd2bdb5919231ace5d902fb29416081c78 |
| equip_distance_matrix | 4326a1f20bf0177d2aa9d7b7fdc53b61b48ea99edae316f5e817a6a5405a67a3 | 752329181986eea10889ce6d1b21b10f4262d20d6e77576e7be18ecc953cd095 | 452604d431a400a5d3e3d464bd9400154c07caf6511552e3f50a240b1aa6f329 |
| chain_fire_propagation | 633c51fbde630fc395207c61d037f4a09fb5d8554fb02bfd511b4b696f6dd953 | 5273db07f477ec71a9dddfa3ee5a8bfb4194c7457d56a8e31d2c9d87d1ab3747 | 24c6e1f7c6bb3c743cb0e2291e72fecc8367fb245718956d4bd3bbf22d938954 |
| aoe_response_queue | 48e21474ae37149a58e1c03204560ef7b69d5791368fae7c78573f4b468037b0 | 3926b33a144101b65aec1e81d2a1311066f6c36a941dd958977c2cbe9ceb9b79 | fa5035a70705265135e0d67568a29ba07a7b8fdc0579a82c1af2664ef00342ab |
| duel_alternating_response | 365adf8e008526e9b6d23ac0498ad58d1acd08d64bf8df9ac3923cd78b0ae4bb | 427046e407db16d467197ddc78affba6b39af0edaf5a6a72740a7fe24b432642 | fa5b19d4ca09a57c49e573e2a3a8ca02c402971a0c6155be19a8180a645e23ef |
| nullify_chain_on_savage | af8b6f199045f2d11cc019cbccc9077e834f6111ff342bdd4dad493c710e6dbc | b93041d1f5d5a2fc740dc87cbcd94585d4705542f4a546e2c07125c868a96e31 | 7aae20a56442627b8e77000d78d7ed652f05c75d177de7e6adbbd9c4baa44381 |
| judge_phase_delayed_tricks | ee52de3b4ab9d4d376c8a008a99c21fc8d91295051e624cc823bfd19a3669033 | 210af8c215d0bc0877f5b1c1bf40fd5b180da91aeec665865e11150a548458ec | 5fd111380b67407fe819f9363cd44d3f28c7ee79a5a59bfd30f7fd78c1a2ecac |

「缺陷态 v60」列与 [2026-09-24 进度核验](../2026-09-24-native-progress-audit/native-tests.log)所记 14 个 actual 一致（首尾抽查核对）。

## 连带修复（同一根因：目录构成修正改变建局发牌）

发牌构成变化暴露了三处对发牌敏感的既有测试构造（均为测试侧问题，非引擎缺陷）：

1. `g1_shahai_gujian::hujia_primitive_and_food_batch_three`：鲤鱼汤段的
   `while open_choice + 空应答`清理循环在遇到 `gj_jinlianzhu_confirm`（min=1，
   options=[Confirm, Pass]）时空应答非法、询问重开、无限旋转（新发牌使 1 号摸到
   金莲烛成为持烛者）。修复：抽出 `drain_choices`（min>0 的确认类询问选中性
   Pass，步数上限 64），替换同文件 6 处同型循环。
2. `g1_shahai_gujian::dinvxuanshuang_and_gjyuheng_closeout`：帝女玄霜濒死段
   假设「1/2 号无桃、首问即 0 号」；新发牌使 1 号摸到普通桃、首问 observer=1。
   修复：循环 Pass 掉无玄霜者的救援询问，直到持玄霜的 0 号被问，保持原断言语义
   （0 用帝女玄霜按桃救回、体力不变、守恒）。
3. `native_kernel::p2_death_with_machine_held_cards`：闪电濒死后只 Pass 一次；
   新发牌使 1、2 号均持桃，Pass(1) 的返回值携带开给 2 号的新询问，测试未继续
   应答，后续 NextPhase 全被拒。修复：连续应答 dying_save 直至静止（上限 16）。
   经 `N12DBG` 轨迹与临时内核步进日志定位后还原，`kernel.rs` 哈希与修复前一致
   （source-hashes 可查），无内核改动。
4. `native_kernel::identity_pure_command_match_reaches_outcome`：seed→终局阵营
   映射是快照值（该测试注释即此口径）。目录修正后实跑复核：seed 5 → Rebels
   （243 命令）、seed 1 → LordSide（266 命令），双方阵营仍各覆盖一次；更新期望值，
   终局可达（≤2000 命令）、胜负与身份一致性、守恒断言保留。

## 结论边界

- 已实现并验收：目录循环缺陷修复、A0 声明表检查、14 golden 重录、三处测试
  构造修复、身份局映射实跑复核；全量 338/0。
- 规则语义与存档格式无变化，版本号均未递增；14 场景的命令、事件与状态轨迹
  语义与 v59 一致（哈希差异=版本字段+发牌构成，逐场景脚本断言未改）。
- A0 检查的「先失败」验收口径未完全达成：目录缺陷由静态核查发现（非该检查
  在缺陷源码上跑出红灯），已在演进提案 A0 小节如实记录。
- 不在本批范围：`archive.rs` tag 29 冲突（N11，编译警告仍在）、枭姬挂起字节
  往返负例、与上游规则等价验收、性能测量（本批无性能结论）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（shasum -a 256，相对 noname-next 根）。
