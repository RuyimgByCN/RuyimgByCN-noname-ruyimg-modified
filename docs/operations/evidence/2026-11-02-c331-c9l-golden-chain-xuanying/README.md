# C331 C9L、golden 重錄、鏈序修正與玄影方案 B（2026-11-02）

核驗日期 2026-11-02；私有樹 C331：規則 **331**／NGSA **106**／**WS7**（`NETWORK_PROTOCOL_VERSION = 7`），
基準 `5fc0f774`；Main 仍 330／105／WS6，本批**未合入**、未提交。

## 一、C9J 四項（C9L 修復）

| 項目 | 修法 | 回歸 |
| --- | --- | --- |
| P1 同疾未付費改道／收據剝除 | 裝備分支補發 `CardsLost`、兩分支都記 `loss: Some(id)`；驗證端要求「收據幀精確核對」或「代價牌在棄牌堆且不在付款者牌區」 | `c331_jiding_proofs`（2 項，還原舊驗證器時紅） |
| P1 同疾／移花 origin 未綁定 | 新增 `jiding_card_is_bound`（真實 jiding 牌幀＋`jiding_card==card`），先綁定、先記錄、再付款／改寫；收據層「有既定牌就驗證」 | 同上 |
| P2 `jiding_changes` 缺邊界／驗證 | 來源綁定、張數上界、`before_player/before_target/actor` 邊界、空變更要求原始載荷 | 同上 |
| P2 WS7 文字仍寫 WS6 | `networkClient.ts` 動態 `WS${NETWORK_PROTOCOL_VERSION}`；兩個 game-server 斷言與註解改 7（刻意測舊 Hello 的 5／6 保留） | `nc09_projection_parity` 10/10、`nc09_referee_projection` 4/4 |

附帶：`adr0787_timeout` 的 329 舊檔斷言改為「格式或規則版本拒絕皆可」（格式 105→106 後在格式門先拒），
仍保留「不得改寫房間校驗和」；前端 `unsigned()` 形參改 `number | undefined`（缺省即非法）修掉 typecheck 紅。

## 二、golden 12 場（使用者裁決：11 場重錄、chain 照上游先解鏈）

- 11 場 `business_equal_at_all_observed_commands_and_final` 依提案重錄（僅雜湊位移）。
- `chain_fire_propagation`：C331 原本「先傳導傷害、後解鏈」，改為上游「先解鏈、後傳導」
  （`damage_flow.rs` 的 `P::Chain` 先 `linked=false`＋`Linked{false}` 再發 `Damage`）。
  事件序實測 `DamageDealt{1}→Linked{1}→Linked{2}→DamageDealt{2}`，與 C8 基線紀錄一致（見
  `29-chain-events-after.log`）。舊序兩支測試（`c9b`／`c9c`）改為斷言上游序。
- 結果：`replay_scenarios_match_golden` **12/12 通過**。

## 三、玄影（`SwdXuanying`）方案 B：parent-group grant

審查原文：「damage 子定義已登記但不在任何 parent group（只有 sha/use），原生父技所有權讓傷害面可達」
→ 具體可達性缺陷。修法：

1. 新行為 `HostBehavior::SwdXuanyingDamage`：tag **537**、key `host:xuanying_damage`、
   `ALL` 532→**533**、metadata（Damage／Normal／optional）、filter（自己受傷、已橫置、來源存活）、
   step（解除橫置＋對來源虛擬殺）。
2. 父技 group 由**裝載器**補回：`HostBehavior::group_children()` 宣告
   `SwdXuanying → [("xuanying_damage", SwdXuanyingDamage)]`，橋接時追加子技宣告並授予；
   **不動轉換包資料**（`cards.json`／npack 雜湊不變），也不新增 schema 合同。
3. 父技只保留主動面（未橫置出殺）；傷害面完全由子技承擔，可達性＝是否被授予。
4. 回歸 `g1_shahai_swd/xuanying.rs::source_parent_group_grants_damage_child_counterattack_and_requires_the_grant`：
   已授予 → 反擊來源且 byte 還原一致；未授予（手動移除）→ 不解除橫置、不反擊、不製造殺。
5. `report_event_arms` 回到 **88**（`test_damage_arms_are_enumerated` 由紅轉綠）。

## 四、閘門（全部本日實跑）

| 命令 | 結果 |
| --- | --- |
| `cargo test --offline --workspace --no-fail-fast` | **168 二進位／7138 passed／0 failed／77 ignored**（`28-final-workspace-round12.log`） |
| `cargo test -p engine-native-kernel -p engine-native-content -p engine-native` | **1897／0**（`23-three-crates.log`） |
| `RUSTFLAGS="-D warnings" cargo check -p kernel -p content -p native -p wasm -p loader --all-targets` | exit 0（`24-strict-after-xuanying.log`） |
| `cargo check --target wasm32-unknown-unknown -p engine-native-wasm` | exit 0（`25-wasm32-check.log`） |
| `cargo run -p conformance` | ALL PASS（`26-conformance-final.log`） |
| `pnpm build:native` | exit 0；4,287,871 B、sha256 `9794db5a…`（`30-build-native-wasm-round12.log`） |
| `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true`（`31-cross-target-round12.log`） |
| `pnpm smoke:native` | 82／0（`32-worker-smoke-round12.log`） |
| `pnpm test`／`pnpm typecheck` | 187／0；exit 0（`33-frontend-tests-round12.log`／`34-typecheck-round12.log`） |
| `python3 -B scripts/check-source-lines.py` | 9052 檔 OK（`36-source-lines-round12.log`） |
| `python3 -B scripts/preflight.py` | 4 項失敗，全為私有樹佈局／歷史文件假象（缺 `../../../AGENTS.md`、`archive/upstream` 符號連結、歷史證據引拆分前 `damage.rs:1015`），見 `35-preflight-round12.log` |

## 四之二、Main 的 park 應答路由修復移入本分支

C331 樹基底 `5fc0f774` 早於 Main 的 `a4483b64`，原本**仍會重現** park-1 的
`engine_native_kernel::machine::Ctx::answer` panic。本批把該修復移入並適配：

- `Park::awaiting_answer`（執行期標記，不入快照／雜湊）＋ 快照期用 `active_ask` 唯一匹配重建；
- C331 樹的喚醒入口是 `ready_park_index`（`choice_source` 的唯讀投影共用它），因此**不改**
  `ready_park_index`，而是在 `run` 恢復段內做「有應答 → 只在同深度找 `awaiting_answer` 的那台，
  否則走原 `ready_park_index`」，恢復時清掉標記；`kernel/phase.rs`、`engine.rs`、`apply_command.rs`、
  `play_card.rs`、兩個 trigger park 初始化同步補 `awaiting_answer: false`；
- 新增約束 `answer_without_ask`（`snap/validation.rs`）；
- 回歸 `nc10_answer_routing`（2 項；把選擇還原為舊行為時 `restored_same_depth_answer_reaches_the_asking_machine` 紅）
  與 `nc10_park_answer_routing`（wasm typed＋byte 2 項）。測試中「同深度只有一台」的形狀斷言
  改為 C331 實際形狀（同深度另有 TriggerOrder／DamageFlow 陪跑），語意斷言不變。

### 含 park 修復後的最終重跑（2026-11-02）

| 命令 | 結果 |
| --- | --- |
| `cargo test --offline --workspace --no-fail-fast` | **169 二進位／7142 passed／0 failed／77 ignored** |
| `nc10_answer_routing`（2）／`nc10_park_answer_routing`（2） | 綠；把 `run` 喚醒段還原為舊選擇時前者紅（紅綠已驗） |
| strict 5 crate all-targets／wasm32 | exit 0／exit 0 |
| `pnpm build:native` | exit 0；4,288,854 B、sha256 `10a277b6…` |
| 跨標的／Worker smoke | `native_wasm_match: true`／82／0 |

## 五、邊界（未做）

- 註冊 262/290 仍為 **262／290（剩 28＝16 譯名＋12 機制）**，本批未推進。
- 規則 332（改命）與 R05（333／106／759／WS7）仍排在 331 驗收之後。
- 瀏覽器 7 案例（需 331／106 專屬夾具）與 legacy WASM 重建（舊線凍結）未跑。
- 本批未合入 Main、未提交。

## 六、最終認證（2026-11-02，含 park 修復移入後的最終碼面）

| 閘門 | 命令 | 結果 |
| --- | --- | --- |
| 三 crate | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | **1899 passed／0 failed** |
| 全量 workspace | `cargo test --offline --workspace --no-fail-fast` | **169 二進位／7142 passed／0 failed／77 ignored** |
| strict（5 crate、`-D warnings`、all-targets） | `RUSTFLAGS="-D warnings" cargo check …` | exit 0 |
| wasm32 | `cargo check --target wasm32-unknown-unknown -p engine-native-wasm` | exit 0 |
| conformance | `cargo run -p conformance` | ALL PASS |
| WASM 重建 | `pnpm build:native` | exit 0；4,288,854 B、sha256 `10a277b6…` |
| 跨標的 | `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true` |
| Worker smoke | `pnpm smoke:native` | 82／0 |
| 前端 | `pnpm test`／`pnpm typecheck` | 187／0；exit 0 |
| Tauri IPC | `cargo test -p desktop-client` | 12＋6 passed、0 failed |
| 行數門禁 | `unittest … test_check_source_lines.py`／`check-source-lines.py` | 2/2 OK；9054 檔 OK |
| preflight | `python3 -B scripts/preflight.py` | **17 ✅／4 ❌**：4 項皆為私有樹佈局／歷史文件假象（缺 `../../../AGENTS.md` ×2、`archive/upstream` 符號連結 ×2、歷史證據引拆分前 `damage.rs:1015`） |

未跑：瀏覽器 7 案例（需 331／106 專屬夾具）、`pnpm build`（避免清 tracked dist）、legacy WASM 重建（舊線凍結）。
