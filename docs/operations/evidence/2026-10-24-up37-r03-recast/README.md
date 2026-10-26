# R03 冯河交牌/重铸二选一（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；规则 **v322**、存档 **v105**。本批只完成 **冯河「交牌或重铸」分支**，不把 R03 标完成。

## 实现

- 新增权威效果 `ChooseGiveOrRecastOneHand`（ABI **756**）：
  - 第一步从技能拥有者手牌选 1 张；
  - 第二步 `Custom` 选择：`0` = 重铸（弃 1 摸 1），其他 token = 交给对应存活其他角色；
  - 重铸/交牌由既有 `RecastCards` / `GiveCards` 原语提交，非法目标/牌已离开手牌会拒绝且不变更状态。
- 新效果贯通 rule-api、content-schema、content-compiler、rule-vm、engine-core 的 tag/ABI/hash/continuation；`pinghe.skill` 改用该效果。
- shiji npack **0.1.1 → 0.2.4**（中间试作版本未作为交付），`rule_vm_abi_version=756`；metadata 补 0.1.6 记录。
- `original_shiji_spxizhan_huaman` 的整包 ABI 断言由 502 改为 **756**（因新增效果/奖励参数，语义性变更）。
- 冯河测试选择顺序改为「先选牌，再选目标/重铸」；新增重铸回放：不交给目标、手牌数弃一摸一不变、上限仍 -1。
- 冯河补 `modifier hand_limit_base_max_hp override 1`，手牌上限基数使用最大体力；新增 hp2/maxHp4 → 手牌上限 4 的回放。
- 恢复平定标记的目标限制：新 modifier target `card_target_cancel_if_other_user_card_use_count_lt_owner_mark`；记录每玩家本回合（全阶段）卡牌使用数，使用数 < 平定标记数时不能指定拥有英霸的目标；新增“第 1 张被拒、使用 1 张后允许”的回放。仅当内容存在该 modifier 时记录计数，标准局 golden 不变。
- 移除平定标记上错误的手牌上限加成（上游 `yingba_mark` 只限制“每回合前 X 张牌不能指定英霸角色”，没有手牌上限）；新增断言：目标获得 1 枚平定后手牌上限仍为 3，不会变成 4。
- 新增通用 active target 约束 `max_hp_gt`（schema/parser/typecheck/hash/校验），英霸目标改为 `targets 1 1 other max_hp_gt 1`；新增 maxHp1 目标被拒绝的回放。

## 结果

| 检查 | 结果 | 证据 |
|---|---|---|
| `original_merged_shiji` | **115 / 0 / 0** | `original_merged_shiji` 日志随本目录重跑 |
| `engine-core` 完整 | **3804 / 0 / 1**（173 target）（172 target） | [日志](engine-core.log)、[计数](engine-core-counts.json)、[退出码](engine-core.exit) |
| `content-compiler` 完整 | **541 / 0 / 0**（含 `max_hp_gt` 3 项、平定 target 2 项）（含 `max_hp_gt` 2 项） | [日志](content-compiler.log)、[退出码](content-compiler.exit) |
| `rule-api` | **2 / 0 / 0** | [日志](rule-api.log)、[退出码](rule-api.exit) |
| `rule-vm` | **15 / 0 / 0** | [日志](rule-vm.log)、[退出码](rule-vm.exit) |
| `content-native-loader` | **32 / 0 / 0** | [日志](content-native-loader.log)、[退出码](content-native-loader.exit) |
| `engine-native` | **1548 / 0 / 71**（316 target） | [日志](engine-native.log)、[计数](engine-native-counts.json)、[退出码](engine-native.exit) |
| `game-server` 完整 | **330 / 0 / 0**（23 target） | [日志](game-server.log)、[计数](game-server-counts.json)、[退出码](game-server.exit) |
| 前端 `pnpm test` / `tsc --noEmit` | **exit 0 / exit 0** | [测试日志](web-test.log)、[类型日志](web-tsc.log) |
| Worker WASM 重建 `pnpm build:native` | **exit 0** | [日志](build-native.log) |
| Worker smoke `pnpm smoke:native` | **47 / 47，0 失败** | [日志](smoke-native.log) |
| `game-server --test native_product_path` | **10 / 0 / 0** | [日志](product-path.log)、[退出码](product-path.exit) |
| native/WASM 轨迹对比 | **match=true**，final hash `12ac6844…d1cd` | [日志](cross-target.log)、[退出码](cross-target.exit) |
| `sync-original-content --verify-only` | **exit 0**（最终） | [成功日志](sync-verify-03.log)、[退出码](sync-verify-03.exit) |

## 已知差异与未完成

- 重铸不授平定：授标已收进效果的「实际交牌给存活目标」分支；重铸仅弃一摸一，测试断言无标记。
- 重铸已按可重铸牌过滤（当前实现：标准 `tiesuo` / 铁索连环；其他可重铸牌尚未登记）；非重铸牌不出现选项 0。新增稳定 def 身份测试：`tiesuo` 可重铸、`sha` 不可。
- 平定用牌计数在**获得标记前**也会累计（内容定义该 modifier 时即记录，等价上游 `yingba_mark` init 继承已有 useCard 历史）；标准局无该 modifier，golden 不变。新增标记前已用 1 张→获得 1 枚后即可指定的回放。
- 目标选择仍无「体力上限 > 1」过滤；冯河手牌上限基数=已损失体力未建模；平定跨角色计数、原生同名技能仍缺。
- 奖励依赖 `require_skill_def` 参数检查拥有（含暂时失效）；目标死亡不授。
- 目标选择已按 maxHp>1 过滤；冯河本身不要求目标上限过滤（已核对）。
- **冯河防伤时序缺陷仍在**：经真实【杀】路径到达选择时伤害已提交（`prevented_damage_ids` 仍含 id）；本批未修。
- 未做 0.1.0/0.1.1/0.1.6 旧档恢复反例；按内容锁应拒绝。
- 初次 `sync-original-content --verify-only` 因 N01 新增文件仍标 `source_synced` 失败（[失败日志](sync-verify.log)）；把 `ol_fenli`、`ol_pingkou`、`ol_zhuhuan` 标为 `ported` 并同步 `counts.migrated_*` 后通过，未改来源或放宽检查。
