# Batch 1.3（U08 英雄杀）规则修复与独立复核摘要

- **日期**：2026-11-20
- **基线提交**：`748e2974e6f63067e7770278f1f90b5138a60009`（含 `ci-runs.md` 登记，合入 `swarm/author-u08-l28` `00c6c7412cc7e54484f11176649dc57dd62409f5`）
- **冻结版本门禁**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`WS_PROTOCOL_VERSION = 8`（均零变更；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁 `9_097_112_637_251_845_257` 全部保持不变）
- **独立复核 agent**：`080e0e55-1095-4fc2-961b-cf5bb2bca2e5`（未参与实现；三轮复审直至 **23 PASS / 0 FAIL / `APPROVE`**，结合无头 Chrome 上游 `noname` 实跑探针与源码逐行对照）

## 1. 清单变化汇总

| 来源 | 修改前（等价／近似／缺失） | 修改后（等价／近似／缺失） | 净变化 | 说明 |
| --- | ---: | ---: | ---: | --- |
| U08 英雄杀 | 143 / 7 / 0 | 150 / 0 / 0 | +7 | `yxsdili`、`heqin2`、`xumou`、`bolehuiyan`、`bolehuiyan2`、`bolehuiyan3`、`nvquan3` 全部修通并复核为 `已复核等价`；另完成 `cike` 与固定种子抽样 15 项等重审通过 |
| **全仓 4,424 项** | **636 / 3,788 / 0** | **643 / 3,781 / 0** | **+7** | 整包八项签署维持 **0/37**（待 UI/Spine/真机签名等条件完备） |

## 2. 本批修复的核心规则语义

1. **`yxsdili` 与 `heqin2`**：`u08_batch_a.rs:874-881` 删除 `heqin2_active` 标志兜底分支，严格校验目标持有真实 `"heqin2"` 技能；`yxsdili` 保留 `lostHp > 0 && !drawPhase.isSkipped` 与 `nichang` 取消协同（`u08_hero_kill_match.rs:997-1006`、`u08_r2_heqin2_requires_actual_skill`）。
2. **`xumou`**：结束阶段翻面经 `EventData::TurnOverBefore` 派发，受 `U06` `mianju` 等护盾正确取消且不重覆（`u08_batch_c_xumou_turnover_and_mianju_6d_real_match`）。
3. **`bolehuiyan`、`bolehuiyan2`、`bolehuiyan3` 与 `cike`（`Sha` 内核结算与钩子拆分）**：
   - 将相关钩子从 `u08_batch_d_ext.rs` 拆出至 `u08_batch_d_hooks.rs`（1–208 行），保持 Rust 有效行数门禁余量。
   - `kernel/sha.rs`、`sha/step.rs`、`sha/resume.rs`、`sha/armor.rs` 新增 `defer_sha_end`、`finish_or_defer_sha_end`、`take_sha_pending_end` 与 `has_sha_trigger_park`（按精确帧深与 `trigger_order().is_none()` 唤醒），将 `shaEnd` 延后至 `Damage`、风雪刃 `TurnOverBefore` + `start_draw(1)` 及子帧结算完毕后再触发 `bolehuiyan2`/`bolehuiyan3`，并移除 `bolehuiyan_card_frame` 门控以支持嵌套杀（如诱敌深入第二杀，CDP `1h`）。
   - 核实寒冰剑 `damageBegin2`（`card/standard.js:149-151, 3409-3416`，仍进 `shaDamage` → `bolehuiyan3`，CDP `1g`）与风雪刃 `shaHit` `unhurt = true`（`杀海拾遗/card/swd/skill.js:2284-2311` + `card/standard.js:246, 282-286`，跳过 `shaDamage` → `shaEnd` → `bolehuiyan2`）语义差异并覆盖于 `u08_r2_bolehuiyan_hanbing_fengxueren_damage_order_youdishenru_nested`。
   - `cike` 将红判 `directHit`（`shoulie_hit`）调至仁王盾之后、武器窗与 `respond_shan` 之前；在 `OrdinaryJudge` 前后清零 `last_judge_suit` 消除脏标志误判；青龙追杀时清理 `sha_cike_done:{fid}`/`sha_end_settled:{fid}`/`sha_pending_end:{fid}` 并复位 `shoulie_direct = false`（`u08_r2_cike_direct_hit_weapons_qinglong_and_stale_judge`）。
4. **`nvquan3`**：`u08_batch_d.rs:718-723` 改用 `effective_gender_in` 动态性别，正确区分伤害来源/目标男女与距离手牌装备分支（`u08_batch_d_nvquan3_effective_gender_6d_real_match`）。
5. **抽样与运行发现差异全修（10 项、`u08_hero_kill_review_fixes.rs`）**：
   - **静态声明锁保留 + 运行期窗覆盖**（`content/host.rs:304-329`）：在 `HostSkillBehavior::kind_trigger_meta` 无条件覆写 `nichang`（`DrawPhaseBefore`，并修复 `kernel/draw.rs` `DP::Begin` 取消旁路）、`wange`（`DrawPhaseBegin`，改配 `EventData::DrawPhase`）、`nichang2`/`wange2`（`TurnPhaseEnd` / `TurnEndWindow { after: false }`，严格在 `Phase::End` 之后触发）及 `jieyong`（`TurnBegin | TurnEnded` 清理），保持 `decision34_pack9` 静态 `content_lock_id` 不变（`u01_dddanzhi_regression` 20/20 通过）。
   - **`wange2`**：`u08_batch_c_asks.rs:374-417` 与 `u08_batch_c_ext.rs:38-95` 新增逐目标 `ASK_WANGE2_CARD = 25`（`"wange:choose_card"`）交互选牌及 `ai/extension_choices.rs:521` AI 分支。
   - **`ducai3`**：`d34_flags.rs:602-620` 抽出 `is_ducai3_forbidden` 并改用 `effective_card_suit`，接入 `apply_card_use_canceled`、`apply_dying_peach_target_canceled`、`combat_rules.rs`、`duel.rs`、`sha/step.rs`、`snap/asks/{sha,shapes}.rs`。
   - **`tongyu`/`lguiyin`**：`u08_batch_c.rs:276-298` 经 `is_luobinhan_seat` 校验 `general_slots` 含 `"yxs_luobinhan"`。
   - **`qiandu`**：`u08_batch_d.rs` 与 `u08_batch_d_asks.rs` 统一排除 `core.next_alive(owner)`。
   - **`xiadao`**：`u08_batch_c_asks.rs:418-442` 移除自选牌窗，改为 `ctx.core.rng.gen_range` 从目标手牌随机抽取。
   - **`jieyong` 与 `feigong`**：`catalog.rs` + `resolution.rs:860` 新增 `ContentHooks::on_card_use_after`，由 `u08_batch_d_hooks.rs:165-208` 拦截 `!u.cancelled` 且仍在弃牌堆（含延时锦囊/全牌种、让步 `jianxiong`），直接由 `discard` 移回手牌消除悬空 `shouye_pending_gain_card`；`u08_batch_d_ext.rs:301-309` 修正 `feigong` 在 `discard_cards` 压入 `CardsDiscarded` 前置 `u.cancelled = true`；独立核实上游 `jieyong2`（`skill.js:2826-2829`）仅作回合临时标记无 `enable`/`viewAs`/`trigger`、`jieyong3`（`2830-2841`）无引用。

## 3. 独立复核 23 项明细表（`080e0e55-1095-4fc2-961b-cf5bb2bca2e5`）

| 清单项 ID | 结论 | 核对分支、测试与上游对照依据 | 清单新状态 |
| --- | --- | --- | --- |
| `U08:skills:yxsdili` | **PASS** | `u08_batch_a.rs:1-882`、`u08_batch_a_asks.rs:1-664`；校验 `lostHp>0 && !isSkipped` 及 `nichang` 协同取消摸牌；`u08_batch_a_b_yxsdili_*` | `已复核等价` |
| `U08:skills:heqin2` | **PASS** | `u08_batch_a.rs:874-881` 已删兜底分支，校验真实持有 `heqin2`；`u08_r2_heqin2_requires_actual_skill` | `已复核等价` |
| `U08:skills:xumou` | **PASS** | `u08_batch_c.rs`、`u08_batch_c_ext.rs`；经 `TurnOverBefore` 受 `mianju` 取消且先于 `nichang2`；`u08_batch_c_xumou_*` | `已复核等价` |
| `U08:skills:bolehuiyan` | **PASS** | `u08_batch_d_hooks.rs:18-163`、`u08_batch_d_asks.rs:765-905`；`shaBegin` p11 优先、控牌过滤、取消耗次与嵌套杀（CDP `1h`） | `已复核等价` |
| `U08:skills:bolehuiyan2` | **PASS** | `u08_batch_d_hooks.rs:112-163`、`kernel/sha.rs`、`sha/resume.rs`；无门控、延后至风雪刃/诱敌深入子帧后 `shaEnd` | `已复核等价` |
| `U08:skills:bolehuiyan3` | **PASS** | `kernel/sha.rs`、`sha/step.rs`、`sha/resume.rs`；核实寒冰剑 `damageBegin2 -> shaDamage` (CDP `1g`) vs 风雪刃 `shaHit unhurt -> shaEnd` | `已复核等价` |
| `U08:skills:nvquan3` | **PASS** | `u08_batch_d.rs:718-723`、`u08_batch_d_asks.rs`；`effective_gender_in` 动态性别及男女分支；`u08_batch_d_nvquan3_*` | `已复核等价` |
| `U08:skills:cike` | **PASS** | `sha/step.rs:748-840`、`sha/resume.rs:355-367`、`u08_batch_d_ext.rs:463-489`；仁王优先、红判武器窗、清脏标志及青龙复位；`u08_r2_cike_*` | `已复核等价` |
| `U08:skills:nichang` | **PASS** | `content/host.rs:304-329`、`kernel/draw.rs:64-120`、`u08_batch_c.rs:602-614`；`DrawPhaseBefore` 与 `DP::Begin` 取消；`u08_r2_nichang_*` | `已复核等价` |
| `U08:skills:nichang2` | **PASS** | `content/host.rs:304-329`、`u08_batch_c.rs:609-615`、`u08_batch_c_ext.rs:18-36`；`TurnPhaseEnd` 整回合末触发（晚于 `Phase::End`） | `已复核等价` |
| `U08:skills:wange2` | **PASS** | `content/host.rs:304-329`、`u08_batch_c_asks.rs:374-417`、`u08_batch_c_ext.rs:38-95`；`TurnPhaseEnd` + `ASK_WANGE2_CARD` 选牌 + AI；`u08_r2_wange2_*` | `已复核等价` |
| `U08:skills:ducai3` | **PASS** | `d34_flags.rs:602-620`、`combat_rules.rs`、`duel.rs`、`sha/step.rs`、`snap/asks/{sha,shapes}.rs`；`effective_card_suit` 全路径封锁；`u08_r2_ducai3_*` | `已复核等价` |
| `U08:skills:ducai2` | **PASS** | `u08_batch_c.rs`、`d34_flags.rs`；独裁后对所选花色无距离与次数限制 | `已复核等价` |
| `U08:skills:tongyu` | **PASS** | `u08_batch_c.rs:276-298`、`u08_batch_c_ext.rs`；`is_luobinhan_seat` 校验主副将槽，取消退次；`u08_r2_tongyu_*` | `已复核等价` |
| `U08:skills:qiandu` | **PASS** | `u08_batch_d.rs:612-618,808-812`、`u08_batch_d_asks.rs:721-726`；过滤与目标池排除 `core.next_alive(owner)`；`u08_r2_qiandu_*` | `已复核等价` |
| `U08:skills:xiadao` | **PASS** | `u08_batch_c_asks.rs:418-442`；RNG 随机抽目标手牌 1 张无选牌窗；`u08_r2_xiadao_*` | `已复核等价` |
| `U08:skills:jieyong` | **PASS** | `resolution.rs:860`、`u08_batch_d_hooks.rs:165-208`、`u08_batch_d_ext.rs:331-366`；跳过 `u.cancelled`、直接回手无 `shouye_pending_gain_card` 悬空、`jieyong2/3` 核实；`u08_r2_jieyong_*` + `u08_r3_jieyong_*` | `已复核等价` |
| `U08:skills:lzhangyi` | **PASS** | `u08_batch_c.rs`、`u08_batch_c_asks.rs`；弃牌阶段外失牌重洗弃牌/发动与摸牌/伤人选择（`u08_batch_c_modifiers_*`） | `已复核等价` |
| `U08:skills:heqin` | **PASS** | `u08_batch_b.rs`、`u08_batch_b_asks.rs`；限定技出牌交牌、授 `heqin2` 与各摸 2 牌（`u08_batch_b_all_*`） | `已复核等价` |
| `U08:generals:yxs_diaochan` | **PASS** | `decision34_pack9_data.rs:152-196`、`decision34_pack9.rs:479-515`；群雌 3 血 `[fengyi, wange]`，上游 `character.js:43` | `已复核等价` |
| `U08:generals:yxs_zhaoyong` | **PASS** | `decision34_pack9_data.rs:152-196`、`decision34_pack9.rs:479-515`；魏男 3 血 `[tashan, hushe]`，上游 `character.js:235` | `已复核等价` |
| `U08:generals:yxs_yingzheng` | **PASS** | `decision34_pack9_data.rs:152-196`、`decision34_pack9.rs:479-515`；群男 4 血 `[jianxiong, batu]`，上游 `character.js:195` | `已复核等价` |
| `U08:generals:yxs_luobinhan` | **PASS** | `decision34_pack9_data.rs:152-196`、`decision34_pack9.rs:479-515`；吴男 4 血 `[xiadao, sheshu, lguiyin]`，上游 `character.js:129` | `已复核等价` |

（另同轮联合复核通过 `wange`、`lguiyin` 与 `feigong` 取消写回时序修复。）

## 4. 原始复核证据路径与 SHA-256（存于 `.gitignore` 目录 `docs/operations/evidence-raw/`）

- Round 1 上游 CDP 探针脚本：`docs/operations/evidence-raw/review-batch1-3/probe_u08_upstream.mjs`（`sha256: 7b4620577d578e2474c8a2027f348a930f39169047da4726e6093fd189f46159`）
- Round 1 上游 CDP 实测结果：`docs/operations/evidence-raw/review-batch1-3/u08_upstream_cdp_results.json`（`sha256: 03ca2832a07b75b5656a32e21f729c3b1209d6643fbe605a0855de7e56e923a`）
- Round 2 上游 CDP 探针脚本：`docs/operations/evidence-raw/review-batch1-3/probe_u08_upstream_r2.mjs`（`sha256: 41c72c617a599337c691b09a0ac20ecb67b1707a57113c2bc070b39119237c87`）
- Round 2 上游 CDP 实测结果：`docs/operations/evidence-raw/review-batch1-3/u08_upstream_cdp_r2_results.json`（`sha256: 03879990467ad80e6c4a48a2cc4fff2213d0079038a8067e302307b63de156c8`）
  - 边界如实记录：`jieyong_lifecycle` 自动响应器于弃黑牌前点击了取消，故仅覆盖触发条件矩阵（赤兔入装备区 `0`、奸雄夺杀 `0`、未无懈【乐】入判定 `0`、被无懈【乐】入弃 `1`、`useCardBefore` 取消入弃 `0`、`jieyong2` 元数据 `{ enable: null, viewAs: null, trigger: null }`）；`bolehuiyan_fengxueren_shaHit_unhurt` 真实对局中 `p2` AI 自动出【闪】未入 `shaHit`，风雪刃 `shaHit unhurt -> shaEnd` 与寒冰剑 `damageBegin2 -> shaDamage` 差异已由上游 `card/standard.js` 与 `杀海拾遗/card/swd/skill.js` 源码逐行对照及 Round-1 CDP `1g` 完全核实。

## 5. `noname-next` 本地第五节未过滤门禁实测记录

1. **脚本与静态检查**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：2 tests OK（exit 0）
   - `python3 -B scripts/check-source-lines.py`：9,381 文件 <= 2,000 行、Rust 非测试有效行 <= 1,000（exit 0）
   - `python3 -B scripts/check-native-layering.py`：0 upward references <= 0（exit 0）
   - `python3 -B scripts/check_doc_line_refs.py --strict`：10,846 处检查、0 越界（exit 0；同步修正 `2026-11-12-u08-hero-kill-review/README.md` 因 `u08_batch_{a,b,c,d}*` 调整移位的 6 处行号）
   - `python3 -B scripts/recount_g0.py`：来源数=37、缺失=0（exit 0）
2. **Rust 编译与单元测试**：
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native`：**2493 passed; 0 failed; 6 ignored**（18 个测试集，含 14 标准 golden 全通过；其中 `tests/g1` `u08_` 24 passed、`u01_dddanzhi_regression` 20 passed）
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-batch1-3-lint RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：零警告通过（exit 0，临时目录已删除）
   - `cargo check --target wasm32-unknown-unknown -p engine-native-wasm`：exit 0
   - `cargo test --offline --workspace --no-fail-fast`：**7889 passed; 0 failed; 8 ignored**（178 个测试集，exit 0）
   - `cargo run -p conformance`：**19/19 (`passed=19`)**（exit 0）
   - `cargo test -p desktop-client`：**20 passed; 0 failed; 1 ignored**（exit 0）
3. **前端与跨端 WASM**：
   - `pnpm --dir apps/web-client test`：**489 passed; 0 failed**；`pnpm --dir apps/web-client run typecheck`：exit 0
   - `pnpm --dir apps/web-client run build:native`：exit 0；`pnpm --dir apps/web-client run smoke:native`：**83 total / 0 failed**
   - `python3 -B scripts/check-native-wasm-cross-target.py`：标准 7 模式（`final_revision: 40`）与扩展（`final_revision: 20`）跨端轨迹均匹配（`native_wasm_match: true`）
   - `bash scripts/cross-target-hash.sh`：`start_4_42` (`8fe8316...`) / `start_2_1` (`6fb9d95...`) Native 与 WASM 哈希一致，`ALL CROSS-TARGET CHECKS PASSED`
   - `pnpm --dir apps/web-client run build`：exit 0（随后 `git checkout -- apps/web-client/dist/` 还原）
4. **最终提交前检查**：`python3 -B scripts/preflight.py` 全量 22 项通过（exit 0）。
