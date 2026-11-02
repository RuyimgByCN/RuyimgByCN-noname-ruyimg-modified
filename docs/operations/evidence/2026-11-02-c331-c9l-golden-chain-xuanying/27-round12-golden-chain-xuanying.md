# 第 12 輪：golden 重錄、鏈序修正、玄影方案 B（2026-11-02）

## 裁決依據（使用者）

1. 「golden 11 場照提案重錄，chain 照上游先解鏈」
2. 「玄影照方案 B 做」

## 一、golden 11 場（已套用）

`git apply 15-golden-11-records.proposal.patch`：11 個 business-equal 場景期望值換成本輪實測 actual。
（提案檔保留原樣作證據。）

## 二、鏈序：上游「先解鏈、後傳導」

- 改 `crates/engine-native-kernel/src/kernel/damage_flow.rs` 的 `P::Chain`：傳導目標在吃傳導傷害
  之前先 `linked = false` + `LogEvent::Linked{false}`，再發 `Damage` 子事件。
- 事件序實測（`C331_GOLDEN_TRACE_DIR` 傾印 `chain_fire_propagation.events.txt`）：
  `DamageDealt{1} → Linked{1,false} → Linked{2,false} → DamageDealt{2}`，
  與 C8 基線紀錄一致（舊 C331 是 `Linked{1} → DamageDealt{2} → Linked{2}`）。
- `chain_fire_propagation` 期望值重錄為 `f1eb4cda86bc1795a8b9dc510bfec402f3b3af7f3ed4c6ca36c04f9d179f2a16`。
- 兩支 C331 舊序測試同步改為上游序（非刪斷言）：
  - `c9b_..._siblings_unlink_on_their_own_submission`：暫停點 p2 已解鏈、p3 仍橫置；
  - `c9c_..._damage_record_precedes_each_recipients_change_hp_unlink`：
    首目標維持「傷害記錄→解鏈」，傳導目標改「解鏈→傷害記錄」。

## 三、玄影方案 B（parent-group grant）

事實（權威來源）：damage 子定義已登記但不在任何 parent group（只有 sha/use），
原生父技所有權就讓傷害面可達 → 具體可達性缺陷。

實作：

1. 新行為 `HostBehavior::SwdXuanyingDamage`（tag **537**、key `host:xuanying_damage`）：
   `behavior.rs`／`keys.rs`／`tags.rs`／`tag_decode.rs`／`behavior_list.rs`（ALL 532→**533**）／
   `metadata.rs` + `metadata/swd.rs`（Damage、Normal、optional）／
   `filtering.rs` dispatch + `filtering/swd.rs` 真實條件（自己受傷、已橫置、來源存活）／
   `step.rs` dispatch + `step/swd_damage_targets.rs` 反擊臂（解除橫置＋對來源虛擬殺）。
2. 父技 group 由**裝載器**補回（`content-native-loader/src/lib.rs`）：
   `HostBehavior::group_children()`（`keys.rs`）宣告 `SwdXuanying → [("xuanying_damage", …)]`；
   橋接時為武將追加子技宣告並授予（等同上游「授予父技即授予子技」），
   **不動轉換包資料**（cards.json／npack 雜湊不變）。
3. 父技只保留主動面（filter 固定 false、metadata None），傷害面完全由子技承擔。
4. 回歸 `crates/engine-native/tests/g1_shahai_swd/xuanying.rs`：
   `source_parent_group_grants_damage_child_counterattack_and_requires_the_grant`
   - 正面：已授予子技 → 受傷後解除橫置＋虛擬殺反擊來源（hp 各 -1）＋byte 還原一致；
   - 負面：手動移除子技授予（父技仍在）→ 不解除橫置、不反擊、不製造殺。
5. `host_behavior_keys_bijective` 同步 532→533、tag 範圍 445–537。
6. `report_event_arms` 回到 **88**（`test_damage_arms_are_enumerated` 由紅轉綠）。

## 四、全量與產品閘門（本輪實跑）

| 閘門 | 命令 | 結果 |
| --- | --- | --- |
| workspace | `cargo test --offline --workspace --no-fail-fast` | **168 二進位／7138 passed／0 failed／77 ignored** |
| three crates | `cargo test -p kernel -p content -p native` | 1897／0 |
| strict 5 crate | `RUSTFLAGS="-D warnings" cargo check … --all-targets` | exit 0 |
| wasm32 | `cargo check --target wasm32-unknown-unknown -p engine-native-wasm` | exit 0 |
| conformance | `cargo run -p conformance` | ALL PASS |
| WASM 重建 | `pnpm build:native` | exit 0；4,287,871 B；sha256 `9794db5a…` |
| 跨標的 | `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true` |
| Worker smoke | `pnpm smoke:native` | 82／0 |
| 前端 | `pnpm test`／`pnpm typecheck` | 187／0；exit 0 |
| 行數門禁 | `python3 -B scripts/check-source-lines.py` | 9052 檔 OK |
| preflight | `python3 -B scripts/preflight.py` | 4 項失敗，全部為私有樹佈局／歷史文件假象：`../../../AGENTS.md` 缺（核心壞鏈 2）、`archive/upstream` 符號連結（腳本單測 2）、歷史證據 `nc08-upstream-kuwu/README.md` 引拆分前 `damage.rs:1015` |

## 五、注意

- `pnpm build`（dist）本輪未跑：TS 未改，且此樹 `dist` 為已追蹤產物（執行會清掉舊檔），避免再度產生無謂 churn。
- 瀏覽器 7 案例、legacy WASM 重建仍未跑（需另建 331/106 夾具／授權）。
