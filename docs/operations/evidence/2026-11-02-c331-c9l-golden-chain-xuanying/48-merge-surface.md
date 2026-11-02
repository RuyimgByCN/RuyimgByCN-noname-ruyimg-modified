# C331 → Main 合併面（2026-11-02 實測，未執行任何合併）

## 事實

- C331 私有樹基底：`5fc0f774`（2026-10-31）。工作樹 **319** 項未提交（含本輪 park 修復移入）。
- Main：`a4483b64`（park 修復），工作樹另有 **68** 項未提交 WIP（I02 定義語音／NC10 public-other-plan，尚未驗收）。
- `5fc0f774..a4483b64` 之間有 **4 個提交**：
  `d07b35f9`（Own 選擇規劃只用公開投影）、`34e83f05`（I02 呈現欄位 ingress 驗證）、
  `9f4b7423`（文件）、`a4483b64`（park 修復）。

## 直接重疊（會衝突的檔案）

### A. 與主樹 4 個提交／WIP 重疊的 13 檔（主樹側±／C331 側±）

| 檔案 | Main | C331 | 性質 |
| --- | --- | --- | --- |
| `crates/content-schema/src/package.rs` | +101−14 | +66−2 | 兩邊都改 schema（I02 呈現欄位 vs C331 invocation）→ **語意合併** |
| `crates/content-schema/src/package/hashing.rs` | +18 | +19−1 | 兩邊都改雜湊 → **語意合併** |
| `crates/engine-native/src/ai.rs` | +29−12 | +1−1 | Main 的 `d07b35f9` 改了 API（`projected_choice_action` 變 pub/Option）；C331 只有 1 行 |
| `crates/engine-native/tests/groups/nc_rounds.rs` | +9 | +32 | 兩邊都加測試註冊 → 併集即可 |
| `apps/game-server/tests/native_frozen_packages.rs` | +119 | +1−1 | Main 加了一大段測試；C331 1 行 |
| `crates/content-compiler/src/lib.rs` | +1 | +21 | 低衝突 |
| `crates/content-runtime/examples/gen_*_package.rs`（5 檔） | +2..+7 | +16..+56 | 兩邊都改產生器的版本／欄位 → 逐檔小合併 |
| `docs/architecture/native-progress.md` | +23−4 | +2 | 文件併集（本輪已加 C331 段落） |

### B. 與 park 修復（`a4483b64`）重疊的 12 檔

C331 已把修復移入（`run` 喚醒段、`Park::awaiting_answer`、`answer_without_ask`、5 個欄位初始化、2 個 trigger park），
主樹則是原文版本。合併時這 12 檔會衝突，**建議整檔取 C331 版**（它已包含等價修復＋C331 其他改動），
再逐項核對主樹修復的每個 hunk 都在（清單見證據 `27-round12-…md` 第四之二節）。

## 建議順序

1. **先處置主樹 68 項 WIP**（依 AGENTS.md 它尚未驗收）：提交或另存，不要與 C331 混在同一批。
2. 在私有樹把 319 項提交成一個（或數個）C331 分支提交；`docs/operations/evidence/2026-11-02-c331-c9l-golden-chain-xuanying/` 一併入庫。
3. 以 Main 為基底 merge C331，按上表解衝突：B 組取 C331、A 組語意合併（schema/hashing 需先跑 `content-schema` 單測）。
4. 合併後重跑：三 crate、workspace、strict＋wasm32、WASM 重建＋跨標的、smoke、前端、conformance、行數門禁、preflight（在**正常 checkout** 下跑，私有樹的佈局假象會消失）。
5. 規則版本：C331 已是 331／106；合併後 Main 才切換。R05（333／106→759／WS7）與改命 332 依裁決順序排在 331 驗收後。

## 未做

本文件只做盤點；未建立分支、未提交、未合併、未改任何主樹檔案。

## `content-schema/src/package.rs` 的實際差異（抽樣核對）

- Main（I02）：新增 `voice_assets: BTreeMap<String, String>`（presentation/audio，三處重複欄位定義）＋
  「module 使用 effect 772 低於 ABI 758」的檢查。
- C331：新增 `host_key` / `force_die`（schema5 執行面）、`trigger_order` / `mechanisms` 模組與
  `TriggerTiming` / `TriggerRepeat` 等型別，並改用 `crate::PACKAGE_FORMAT_VERSION`。

兩側是**不同欄位的加性變更**，衝突風險主要在 serde 屬性與 `deny_unknown_fields` 的組合，
以及 `package/hashing.rs` 兩邊都改雜湊輸入；合併時需先跑 `content-schema` 的單測與
`native_frozen_packages`／`c331_invocation_controls` 類測試。
