# Batch 1.5（L29 龙血玄黄）规则修复与独立复核摘要

- **基线 HEAD**：`af6ef4899052ad1258f38a242960f8f0ea0fd066`（Batch 1.4 `U07 玩点论杀` 已发布，CI Run `44785367168` Attempt 1 `10/10 success`，已记入 `docs/operations/evidence/ci-runs.md`）
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁（`9_097_112_637_251_845_257` / `11_413_175_519_070_831_972`）保持不变
- **独立复核员**：`3488fd64-4fea-4218-be3f-f5fb6d38add2`（未参与 Batch 1.5 代码实现；Round 1 Chrome CDP 实跑比对 ＋ 源码复审发现 5 项差异打回 `REJECT`，Round 2 全量回归与逐项核验后给出 **`APPROVE`**）
- **清单变更**：全仓 `652 / 3,772 / 0` -> **`656 已复核等价 / 3,768 简化近似 / 0 缺失 / 0 上游未定义`**（`L29 龙血玄黄` 由 `1 / 8 / 0` 升为 **`5 / 4 / 0`**：`mbqianlong`、`mbcmqingzheng`、`mbcmjiushi`、`mbcmfangzhu`、`mbjuejin` 5 项规则技能升级为 `已复核等价`；`L29:configs:shengjian` 依复核员建议由历史 `已复核等价` 下调为 `简化近似` 并归入队列第 5 项界面阶段，`_CMeffects`、`main1`、`main1.css` 维持 `简化近似`）；整包八项签署维持 **`0/37`**（队列 1 仍余 `L10`/`U01` 与自动选择条目及队列 2–11）

---

## 1. Round 1–2 修复与对齐要点

1. **`L29:skills:mbqianlong` ＋ `mbweitong`**：
   - `crates/engine-native-content/src/content/host.rs`：6 项 L29 技能覆写 `is_forever() -> true`（持恒技豁免 `mbcmfangzhu` `baiban` 禁用，同时 `host/step/skills_targets.rs` 与 `kernel/machine.rs` 统一以 `!is_forever_skill && !is_charlotte_skill` 判定）。
   - `content/host.rs` `kind_trigger_meta`：`HostBehavior::MbQianlong` 将 `EventKind::CardsDrawn` 与 `EventKind::CardsMoved` 从默认 `PhaseBegin` 回落修复为 `Timing::Normal, priority: 0`（对齐 `archive/upstream/noname-liuli/noname/character/mobile.js` 与 Chrome CDP `mbqianlong_1`）。
   - `content/host/support.rs` `mbweitong_boost_active` & `support.rs`：`GameMode::Identity { roles }` 严格校验 `Role::Lord`、`Prototype` 沿用仓库原型惯例、`Versus`/`Doudizhu`/`Boss`/`Kof`/`Kingdoms` 显式返回 `false`；道心满 `99` 保留 `mbweitong`。
2. **`L29:skills:mbcmqingzheng`**：
   - `content/host/asks_mtg.rs`、`content/host/step/skills_damage.rs`、`content/host/support.rs`：`allow_cancel: true` 直出 `mbcmqingzheng_suit`（无多余确认），owner 先弃牌并发出 `CardsLost` 后 `Flow::Yield` 再询问 `mbcmqingzheng_target_suit`，空手牌目标保护与伤害 `source: Some(self.owner)` 对齐；`mbcm_qingzheng_suits`、`mbcmqingzheng_suit` AI 与弃牌匹配改用 `catalog.effective_card_suit`（支持 `xinhongyan` 等花色转化）。
3. **`L29:skills:mbcmjiushi`**：
   - `kernel/dying.rs`、`snap/asks/shapes.rs`、`kernel/intrinsic.rs`、`snap/validation.rs`：虚构 `Jiu` 濒死自救经 `DyingSaveMachine` 选择 `ChoiceOption::Skill(mbcmjiushi)` 后 LIFO 先入翻面 `TurnOverBefore` 再结算 `CardUsed(Jiu)`（无虚假 `CardId`、`0` 手牌及本回合用酒后仍可自救）；非 `face_down` 且允许发动时按帧追踪 `DamageBegin3` (`Timing::First`) -> `DamageEnd`（支持连锁、增伤及中间态翻面）；`get.cardPile2` 仅从牌堆 `deck` 逆向检索 `CardKind::is_trick_kind()` 真锦囊。
4. **`L29:skills:mbcmfangzhu`**：
   - `kernel/combat_rules.rs`、`kernel.rs`、`machine.rs`、`kernel/equipment.rs`、`kernel/damage.rs`、`rvm_bridge/modifiers/d34_flags.rs`、`kernel/sha.rs`、`kernel/sha/armor.rs`、`kernel/sha/step.rs`、`engine/play_sha.rs`、`standard/skills/{zhangba,jijiang}.rs`、`host/activation/common.rs`：
     - `baiban` 按 `!is_forever_skill && !is_charlotte_skill` 禁用既有技能（含马术/咆哮等锁定技），并拦截 `baiban` 生效后新装或续期的非持恒非夏洛特技能（`insert_skill`/`add_skill_until_turn_end`/`grant_equip_skills`/`BOSS_ZHONGDU`）及标准装备技（`Zhuge`/`Fangtian`/`Qinggang`/`Bagua`/`Renwang`/`Cixiong`/`Qinglong`/`Guanshi`/`Hanbing`/`Qilin`），至 `phaseEnd` 清理恢复；
     - `jin` 统一通过 `!kind.is_trick_kind()` 封锁出牌与濒死求桃（含 `Caoyao`/`Hufu`/`Ziyangdan` 等扩展基本牌），并阻止 `Zhangba`/`Jijiang`/`DddXiaxing` 转化与 `Qinglong` 追杀，同时保持 `chooseToRespond`（决斗/南蛮/万箭）可用；
     - `catalog/card_kind.rs` 新增 `CardKind::is_trick_kind(self) -> bool` 精确对齐上游 `get.type2(card) == 'trick'` 的 49 种普通/延时锦囊（排除扩展基本牌、`food`、`jiguan`、`spell`、`land`、`hslingjian`、`jiqi`、`zhenfa`）。
5. **`L29:skills:mbjuejin` ＋ `mbcmxscw`**：
   - `host/step/skills_targets.rs` & `kernel/machine.rs`：`mbjuejin` 护甲公式支持 `hp < 1` 角色（按 `1 - old_hp` 补足并加发动者 `plus = 2`，如 `hp == -1 -> +2` 护甲），仅移出手牌区【闪】【桃】【酒】（不越界移出牌堆或既有弃牌堆），`mbcmxscw` 在后续弃牌入弃牌堆时将既有与新入弃牌堆的【闪】【桃】【酒】一并移入 `detached_cards`。

---

## 2. 独立复核 9 项逐案表（Round 2 `APPROVE`）

| # | 清单 ID (`L29`) | 类别 | 原生落点 | 本批变更与复核要点 | 复核结论 |
|---:|---|---|---|---|---|
| 1 | `mbqianlong` (+ `mbweitong`) | `skills` | `host.rs`、`host/support.rs`、`host/step/skills_status.rs` | `forever: true`、`CardsDrawn`/`CardsMoved` `Normal pri 0`、`mbweitong_boost_active` 模式门控、`99` 保留 | `PASS` -> **已复核等价** |
| 2 | `mbcmqingzheng` | `skills` | `host/asks_mtg.rs`、`host/step/skills_damage.rs`、`host/support.rs` | `direct: true` 可取消、先弃发 `CardsLost` 再选目标花色、`source: Some(owner)`、`effective_card_suit` | `PASS` -> **已复核等价** |
| 3 | `mbcmjiushi` | `skills` | `dying.rs`、`intrinsic.rs`、`shapes.rs`、`resolution.rs`、`card_kind.rs`、`skills_targets.rs` | LIFO `TurnOverBefore` 先于 `Jiu`、`DamageBegin3`->`DamageEnd` 帧追踪、49 种真锦囊 `cardPile2`、存档恢复 | `PASS` -> **已复核等价** |
| 4 | `mbcmfangzhu` | `skills` | `combat_rules.rs`、`machine.rs`、`equipment.rs`、`d34_flags.rs`、`sha*.rs`、`play_sha.rs`、`zhangba.rs`、`jijiang.rs`、`activation/common.rs` | `!forever && !charlotte` 全路径禁技（含后装/标准装备）、`jin` 禁 `!is_trick_kind`＋转化/追杀且放行响应、`phaseEnd` 恢复 | `PASS` -> **已复核等价** |
| 5 | `mbjuejin` (+ `mbcmxscw`) | `skills` | `host/step/skills_targets.rs`、`machine.rs` | `hp < 1` 补甲（`1 - hp`）、仅移手牌闪/桃/酒、`mbcmxscw` 后续弃牌全清扫 | `PASS` -> **已复核等价** |
| 6 | `shengjian` | `configs` | `packages/extensions/longxue-upstream-liuli/assets/config.json`、`longxue.rs` | 配置解析与默认值完整；因运行时 `_CMeffects` DOM/CSS 渲染待队列 5 界面阶段，从历史 `已复核等价` 下调 | **下调为 `简化近似`** |
| 7 | `_CMeffects` | `ui_hooks` | 前端展示桩 | 纯前端表现层，待队列第 5 项界面阶段 | 保持 `简化近似` |
| 8 | `main1` | `surfaces` | 前端展示桩 | 专属技能区 JS，待队列第 5 项界面阶段 | 保持 `简化近似` |
| 9 | `main1.css` | `surfaces` | 前端展示桩 | 专属技能区 CSS，待队列第 5 项界面阶段 | 保持 `简化近似` |

---

## 3. Chrome CDP 上游原始证据索引（存 `evidence-raw/`）

- `19847304144dbf50966e573d21f492e199b47d3f679c74ca2dc30b1c1c2824be` — `docs/operations/evidence-raw/review-batch1-5/probe-l29.mjs`
- `6d2108698882d4010d2c0d37344680634b22ccda0bf8bf62b9a641ae02546f09` — `docs/operations/evidence-raw/review-batch1-5/upstream-l29-cdp-results.json`（`exceptions: 0`）

---

## 4. `noname-next` 本地全量未过滤门禁实测记录（集成重跑）

1. **脚本与静态检查**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：2 tests OK（exit 0）
   - `python3 -B scripts/check-source-lines.py`：9,385 文件 `<= 2,000` 行、Rust 非测试有效行 `<= 1,000`（exit 0）
   - `python3 -B scripts/check-native-layering.py`：0 upward references `<= 0`（exit 0）
   - `python3 -B scripts/recount_g0.py`：来源数=37、缺失=0（exit 0）
2. **Rust 编译与单元测试**：
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native`：**2,518 passed; 0 failed; 6 ignored**（18 个测试结果集，含 15 项 `l29_` 回归与 14 标准 golden 全通过）
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-b15-lint CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：零警告（exit 0，临时目录已清理）
   - `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm`：exit 0
   - `cargo test --offline --workspace --no-fail-fast`：**7,914 passed; 0 failed; 8 ignored**（178 个测试结果集 / 146 个非零测试集，exit 0）
   - `cargo run --offline -p conformance`：**19/19 (`passed=19`, `ALL PASS`)**（exit 0）
   - `cargo test --offline -p desktop-client`：**20 passed; 0 failed; 1 ignored**（exit 0）
3. **前端与跨端 WASM**：
   - `pnpm --dir apps/web-client run typecheck`：exit 0；`pnpm --dir apps/web-client test`：**489 passed; 0 failed**
   - `pnpm --dir apps/web-client run build:native`：exit 0；`pnpm --dir apps/web-client run smoke:native`：**83 total / 0 failed**（含 `worker_all_37_sources_46_packages_locked_matrix` `sources=37/37 packages=46/46`）
   - `python3 -B scripts/check-native-wasm-cross-target.py`：`native_wasm_match: true`（标准 `final_revision: 40`、扩展 `final_revision: 20`）
   - `bash scripts/cross-target-hash.sh`：`ALL CROSS-TARGET CHECKS PASSED`（`start_4_42` `8fe8316a...` / `start_2_1` `6fb9d95a...` Native 与 WASM 哈希一致）
   - `pnpm --dir apps/web-client run build`：exit 0（随后 `git checkout -- apps/web-client/dist/` 还原）
4. **提交前综合门禁**：
   - `python3 -B scripts/preflight.py`：全量 22 项检查全部通过（exit 0，含 `check_local_links.py`、`check_doc_line_refs.py --strict`、`check_evidence_budget.py` 等）。
