# Batch 1.1（U06 欢乐卡牌 ＋ L06 wuxing）规则修复与独立复核摘要

- **日期**：2026-11-19
- **基线提交**：`352236724b5e173cc1ff74c0fb706509129ab7a1`（合入 `.wt-u06-l29` 验证提交 `fa9dc5fff0cf8f12bef950e5adc31daf74f39272`，当前处于提交前收口验证阶段）
- **冻结版本门禁**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`WS_PROTOCOL_VERSION = 8`（均零变更；14 项标准 golden 哈希全部保持不变）
- **独立复核 agent**：`602b9dd3-b8e7-4e3c-81fe-b57137a6f312`（未参与实现；对照上游源码、原生代码与上游实际运行 16 个探针案例）

## 1. 清单变化汇总

| 来源 | 修改前（等价／近似／缺失） | 修改后（等价／近似／缺失） | 净变化 | 说明 |
| --- | ---: | ---: | ---: | --- |
| U06 欢乐卡牌 | 5 / 6 / 0 | 11 / 0 / 0 | +6 | 5 项技能 + 6 张卡牌全部通过独立复核，记为 `已复核等价` |
| L06 wuxing | 1 / 2 / 0 | 2 / 1 / 0 | +1 | `_shengke`、`wuxingpan_skill` 升为 `已复核等价`；`wuxingpan` 卡牌依 S2 下调为 `简化近似` |
| **全仓 4,424 项** | **625 / 3,799 / 0** | **632 / 3,792 / 0** | **+7** | 整包八项签署维持 **0/37** |

## 2. 本批修复的核心规则语义

1. **`U06:skills/cards:monkey`**：将出牌阶段使用桃与濒死救援桃统一接入 `EventData::CardUsed` ＋ `HostBehavior::HuanleMonkeyCard`（`crates/engine-native-kernel/src/kernel/dying.rs`、`events.rs`、`resolution.rs` 及 `crates/engine-native-content/src/content/host/step/huanle.rs:10-143`），按座位序触发并校验 `filterInD`；无论持猴者拿取或取消，桃于结算完毕后正确入弃牌堆消费。
2. **`U06:skills/cards:xuelunyang`**：修复第二步选择允许取消并干净结束；修复 `crates/engine-native-kernel/src/machine.rs:921-955` 回合末清理临时技能时遗漏被禁用条目的问题（去除 `i.enabled &&` 过滤，仅在原启用时发失技事件）；补全 `content/host.rs:180-200` 的 `is_charlotte` 状态戳集合，并阻止同名/主公/唯一/隐匿/觉醒技误列及原临时技重叠覆盖。
3. **`U06:skills/cards:{kuwu,mianju,shoulijian,jiuwei}`**：补齐 `kuwu` 转化杀（`use_as`）与 `_notrigger` 排除及去重日志；`mianju` 统一覆盖全部普通与扩展直接翻面路径；`shoulijian` 校验多目标距离 +1 与非装备 `he` 弃牌询问；`jiuwei` 复核手牌上限加成与无懈不可响应条件。
4. **`L06:skills:_shengke`**：补齐联机禁用、装备使用与直装区分、`Wuzhong`/`Jiguan` 原主判断及 `Tianfuzhen`/`Niaoxiangzhen`/`use_as=Wuzhong` 无懈窗口包装（`crates/engine-native-content/src/content/host/filtering/wuxing.rs:1-237`、`step/skills_status.rs:677-713`）；桃园/五谷单批次事件结构按冻结 `NGSA 107` 基线维持不变。
5. **`L06:skills:wuxingpan_skill` & `L06:cards:wuxingpan`**：`wuxingpan_skill`（`step/skills_card_use.rs:6-62`）第一步选牌可取消并返还出牌阶段限 1 次数，第二步 `SelectNature` 按上游强制选择与合理 AI 取向工作（含 NGSA 挂起恢复）；`wuxingpan` 卡牌保留 `简化近似`——原生 `wuxing.json:5`／`construction.rs:479-503` 固定按 `1/3` 概率为牌库注五行属性，未读取上游 `lib.config.wuxing_num_playpackconfig`（默认 `0.3`）。

## 3. 独立复核 14 项明细表（`602b9dd3-b8e7-4e3c-81fe-b57137a6f312`）

| 清单项 ID | 结论 | 核对分支、测试与上游对照依据 | 清单新状态 |
| --- | --- | --- | --- |
| `U06:skills:monkey` | **PASS** | `dying.rs`/`events.rs`/`huanle.rs:10-143`；上游 `monkey_play_accept_and_stats`、`monkey_dying_accept_then_decline`；回归 `u06_monkey_cancel_discards_tao_and_filter_in_d`、`u06_monkey_dying_rescue_and_decline_card_used`、`u06_monkey_and_shengke_seat_order_and_dying_card_used` | `已复核等价` |
| `U06:cards:monkey` | **PASS** | 牌面 `assets/cards.json` 及绑定技能 `monkey` 同上核验通过 | `已复核等价` |
| `U06:skills:mianju` | **PASS** | `huanle.rs:144-164`、`TurnOverBefore` 全路径；上游 `mianju_both_directions`；`u06_mianju_protects_all_normal_turnover_paths`、`u06_mianju_protects_extension_direct_turn_over_paths` | `已复核等价` |
| `U06:cards:mianju` | **PASS** | 牌面与绑定技能 `mianju` 同上核验通过 | `已复核等价` |
| `U06:skills:kuwu` | **PASS** | `use_as` 杀与 `_notrigger` 排除、无重复日志；上游 `kuwu_wusheng_chain_notrigger`；`u06_kuwu_no_dup_log_use_as_and_notrigger` | `已复核等价` |
| `U06:cards:kuwu` | **PASS** | 牌面与绑定技能 `kuwu` 同上核验通过 | `已复核等价` |
| `U06:skills:xuelunyang` | **PASS** | `huanle.rs:179-222`、`machine.rs:921-955`、`host.rs:180-200`；上游 `xuelunyang_overlap_disabled_cancel_and_resteal`；`u06_xuelunyang_cancel_filter_and_temp_expiry`、`u06_xuelunyang_ignores_expired_temp_skills_and_preserves_awakened_skill`、`u06_xuelunyang_disabled_temp_skill_expires_and_state_stamps_excluded` | `已复核等价` |
| `U06:cards:xuelunyang` | **PASS** | 牌面与绑定技能 `xuelunyang` 同上核验通过 | `已复核等价` |
| `U06:skills:jiuwei` | **PASS** | 无懈不可响应与手牌上限修正、`u06_jiuwei_six_dimension_real_match_equivalence` | `已复核等价` |
| `U06:cards:jiuwei` | **PASS** | 牌面与绑定技能 `jiuwei` 核验通过 | `已复核等价` |
| `U06:cards:shoulijian` | **PASS** | 多目标范围与非装备 `he` 弃牌询问、`u06_shoulijian_no_dup_log_and_non_equip_he_prompt` | `已复核等价` |
| `L06:skills:_shengke` | **PASS** | `filtering/wuxing.rs:1-237`、`skills_status.rs:677-713`；上游 9 项 `_shengke` 探针；`l06_shengke_*` 与 `l06_shengke_zhenfa_and_view_as_wuzhong_nullify_order`（注：桃园/五谷单批次受冻结 NGSA 107 约束保持） | `已复核等价` |
| `L06:skills:wuxingpan_skill` | **PASS** | `skills_card_use.rs:6-62`；上游 `wuxingpan_skill_flow`；`l06_wuxingpan_stage1_cancel_refund_and_stage2_mandatory` | `已复核等价` |
| `L06:cards:wuxingpan` | **FAIL（S2 留差）** | 牌面/插入牌堆无误；`wuxing.json:5` / `construction.rs:479-503` 固定 `1/3` 概率注入五行属性，与上游默认 `0.3`（`wuxing_num_playpackconfig`）存在数值与配置差异 | `简化近似`（下调维持） |

## 4. 原始复核证据路径与 SHA-256（存于 `.gitignore` 目录 `docs/operations/evidence-raw/`）

- `docs/operations/evidence-raw/review-batch1-1/probe-review-u06.mjs`：`sha256: 7c8703ba87ca8f7c91374b0d3b68c582fd35a985e0b9cfbd0f54dbd333c2cd3e`
- `docs/operations/evidence-raw/review-batch1-1/upstream-u06-review.json`：`sha256: 3cfe527bcf24127b8e380851690eeb530db584352eb800eaed5352e8429d624`（6 案例，`spineBlockedCount: 0`）
- `docs/operations/evidence-raw/review-batch1-1/probe-review-l06.mjs`：`sha256: eb7ea46cf8942ddcf3c41fc3afa61bfcc0b869e6ce3d87bf06026cff27fc47b4`
- `docs/operations/evidence-raw/review-batch1-1/upstream-l06-review.json`：`sha256: b8d8d3a6886419b11adda0c1b3791507b7ba0fb74df751cc95dfd5696805d147`（10 案例，`spineBlockedCount: 0`）

## 5. `noname-next` 本地第五节未过滤门禁实测记录

1. **脚本与静态检查**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：2 tests OK（exit 0）
   - `python3 -B scripts/check-source-lines.py`：9,371 文件 <= 2,000 行、Rust 非测试有效行 <= 1,000（exit 0）
   - `python3 -B scripts/check-native-layering.py`：0 upward references <= 0（exit 0）
   - `python3 -B scripts/check_doc_line_refs.py --strict`：10,713 处检查、0 越界（exit 0；同步修正 `L28-雷霆万钧.md:90`、`decision34_inventory_builder.py:444`、`2026-11-12-u08-hero-kill-review/README.md:18` 因清理移位行号）
   - `python3 -B scripts/recount_g0.py`：来源数 37、缺失 0（exit 0）
2. **Rust 编译与单元测试**：
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native`：**2462 passed; 0 failed; 6 ignored**（18 个测试集，含 14 标准 golden 全通过）
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-batch1-1-lint RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：零警告通过（exit 0，临时目录已删除）
   - `cargo check --target wasm32-unknown-unknown -p engine-native-wasm`：exit 0
   - `cargo test --offline --workspace --no-fail-fast`：**7858 passed; 0 failed; 8 ignored**（178 个测试集，exit 0）
   - `cargo run -p conformance`：**19/19 ALL PASS**（exit 0）
   - `cargo test -p desktop-client`：**20 passed; 0 failed; 1 ignored**（exit 0）
3. **前端与跨端 WASM**：
   - 注：曾试跑不在 CI/第五节范围内的 `wasm-pack test --headless --chrome crates/engine-native-wasm` 报缺 `wasm-bindgen-test` 失败（`Cargo.toml` 未引入该依赖）；`engine-native-wasm` 按 CI 通过 `--workspace` 与下列正式 WASM 门禁验证。
   - `pnpm --dir apps/web-client test`：**489 passed; 0 failed**；`pnpm --dir apps/web-client run typecheck`：exit 0
   - `pnpm --dir apps/web-client run build:native`：exit 0；`pnpm --dir apps/web-client run smoke:native`：**83 PASS / 0 FAIL**
   - `python3 -B scripts/check-native-wasm-cross-target.py`：标准 7 模式与 `huanle-packs.json` 跨端轨迹均匹配（`native_wasm_match: true`, `final_revision: 20`）
   - `bash scripts/cross-target-hash.sh`：`start_4_42` / `start_2_1` Native 与 WASM 哈希一致，`ALL CROSS-TARGET CHECKS PASSED`
   - `pnpm --dir apps/web-client run build`：exit 0（随后按第四节规范 `git checkout -- apps/web-client/dist/` 还原非本批构建产物改动）
4. **最终提交前检查**：`python3 -B scripts/preflight.py` 全量通过（22/22 项，exit 0）。
