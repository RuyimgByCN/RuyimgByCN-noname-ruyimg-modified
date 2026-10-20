# 2026-10-18 架構稽核原始證據

本目錄歸檔 2026-10-18 架構與文檔稽核已執行的測試、反例及工作區快照。此次歸檔沒有重跑 Rust 測試，沒有修復業務源碼，也不代表稽核方案已完成。後續執行狀態以[原生進度主稿](../../../architecture/native-progress.md)為準。

## 基準與證據適用範圍

- 稽核 HEAD：`98e269a625accd823c062ba523eab475807ae3ce`，**加上當時 dirty 工作樹**；不能用乾淨 HEAD 單獨重建本次受測源碼。
- 規則版本：`321`；原生存檔格式：`105`。
- 測試開始、第一輪結束和全 workspace 重跑結束的完整路徑／SHA-256 清單分別保存在 [snapshot-start.json](snapshot-start.json)、[snapshot-end.json](snapshot-end.json)、[snapshot-final.json](snapshot-final.json)。三份快照的 `hashes` 均為 **19,969 檔**，內容完全相同。這項比較證明快照涵蓋的維護檔案在測試期間未變，不表示 Git 工作樹乾淨，也不表示快照包含了所有檔案內容。
- 快照時間（UTC）為 `07:13:00`、`07:20:11`、`07:31:33`；開始已有 1,810 項 Git 狀態，最後為 1,823 項。文檔及其他證據由並行工作更新，故 `status_unchanged` 為 false；不要將他人的變更歸因於本次稽核。
- 歸檔時 HEAD 已前進到 `985119c478a25b7992ddf9d0baa74455465f71e4`，該區間提交均為 `docs/` 下的文檔／證據變更。此後續提交不改寫上面的測試基準；完整路徑清單見 [archive-provenance.json](archive-provenance.json)。
- 除原始機器路徑、包路徑與測試輸出外，所選輸出不含環境變數轉儲或憑證。完整相關日誌按原字節複製；未複製 `target/`、編譯產物或 8.7 MB 的座位距離 probe 執行檔。

## 已執行的檢查

命令工作目錄為 `noname-next/`。以下是歷史執行結果，後續源碼變化需要重新執行相應門禁。

| 檢查與命令 | 結果 | 原始證據 |
| --- | --- | --- |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | 1 項通過 | [source-lines-unittest.log](source-lines-unittest.log) |
| `python3 -B scripts/check-source-lines.py` | 8,215 個檔案均不超過 2,000 行 | [source-lines.log](source-lines.log) |
| `cargo test --offline -p engine-native -p content-native-loader -p content-runtime -p engine-native-wasm` | exit 0；各包結果見下表 | [cargo-tests.log](cargo-tests.log)、[cargo-result.json](cargo-result.json)、[cargo-counts.json](cargo-counts.json) |
| `cargo test --offline --workspace --no-fail-fast`，原 Homebrew 工具鏈 | exit 101；在編譯階段因本機動態庫問題中止，尚未執行測試斷言 | [workspace-tests.log](workspace-tests.log)、[workspace-result.json](workspace-result.json) |
| 本機既有 rustup stable 1.95.0，`cargo test --offline --workspace --no-fail-fast` | exit 0；5,727 通過、0 失敗、2 忽略 | [rustup-workspace-tests.log](rustup-workspace-tests.log)、[rustup-workspace-result.json](rustup-workspace-result.json)、[rustup-workspace-counts.json](rustup-workspace-counts.json) |
| `pnpm --dir apps/web-client typecheck` | exit 2；`nativeBackend.ts(198,56)` 的 `{ kind: number }` 選項不符合 `optionValue` 參數型別 | [typecheck.log](typecheck.log) |
| `pnpm --dir apps/web-client smoke:native` | exit 1；13 項中 12 通過，`extension_catalog_15` 實際為 41 而失敗 | [native-smoke.log](native-smoke.log) |

| 定向 Rust 包 | 通過 | 失敗 | 忽略 |
| --- | ---: | ---: | ---: |
| `engine-native` | 1,095 | 0 | 1 |
| `content-native-loader` | 32 | 0 | 0 |
| `content-runtime` | 33 | 0 | 0 |
| `engine-native-wasm` | 20 次測試執行 | 0 | 0 |

原生 WASM 包的 20 次執行包含重複 `#[test]` 導致的同一案例重跑，不能稱為 20 個獨立案例。原生內核忽略項是手動尺度探針 `xly_state_growth_probe`；全 workspace 的第二項忽略來自舊 `engine-core`，詳見完整日誌。workspace 合計包含新舊執行線，不能全部計入新原生內核覆蓋。

Homebrew 首輪故障是 `libLLVM.23.1.dylib` 找不到 `/opt/homebrew/opt/z3/lib/libz3.5.1.dylib`，不是 Rust 測試斷言失敗。重跑只選用已有 rustup 1.95.0 工具鏈，未安裝套件或修改系統庫；兩輪原始結果同時保留，不以重跑成功掩蓋首輪失敗。

WASM 冒煙使用工作區**既有編譯產物**，本次沒有重新構建該產物，沒有啟動真實瀏覽器；因此它不能證明當前 Rust 源碼已完成 WASM 發布構建或玩家介面驗收。

## 三類反例探針

### 1. 原生前端轉接層

[adapter-probe.log](adapter-probe.log) 記錄對當時 TypeScript 源碼轉譯、替換引擎邊界後的三個反例：

1. 候選卡牌值為 42、99 時，介面產生索引 token 0、1；提交 token 0 卻按卡牌值查找，無法找到候選。
2. 引擎返回 `rejected` 結果，轉接層仍返回 `envelope.result = null`。
3. `ChoiceOption Kind(105)` 被標成 `0`。

這是源碼層探針，沒有執行瀏覽器或 WASM。**原始一次性 JS／TS 探針程式未持久化，本目錄只有其輸出日誌**；後續修復應在生產轉接入口補上正式回歸測試，不應宣稱已有完整可重跑的前端端到端工件。

### 2. 安裝包載入與混合包

[loader-probe.log](loader-probe.log) 與 [loader 原始碼](probes/loader/src/main.rs) 使用 `#[path]` 直接載入當時 `apps/game-server/src/native.rs`，並透過正式 `NativeContent` 匯入／載入流程確認：

1. VM 包匯入成功後，移除原始來源目錄；已安裝紀錄與 store 仍存在，載入卻因回讀來源路徑失敗。
2. 同一已安裝 VM 包不重新安裝，只在原始來源新增有效 `assets/cards.json`；載入結果由 1 個 VM 技能變為 0 個，並載入 7 張卡牌。

歸檔包含 [Cargo.toml](probes/loader/Cargo.toml) 與 [Cargo.lock](probes/loader/Cargo.lock)，不包含編譯快取。原始 manifest 及 `#[path]` 保留當時絕對路徑；需要重跑時，將該 harness 複製至倉庫外的臨時目錄，按實際 checkout 調整路徑，再執行 `cargo run --offline --manifest-path <臨時目錄>/Cargo.toml`。這是對實際載入實作的反例，仍不是完整玩家安裝介面的驗收。

### 3. 座位距離與排序語義

[seat-distance-probe.log](seat-distance-probe.log) 與 [seat-distance.rs](probes/seat-distance.rs) 直接呼叫真實 `Engine` 的 `state_core().seat_distance`，遍歷有序座位對：

- 5 人局 25 對中 14 對不符合環狀最短距離，例如 `d(0,1)=0`，預期 1。
- 6 人局 36 對中 12 對不符合，例如 `d(0,3)=1`，預期 3。
- 8 人局 64 對均吻合，顯示只測 8 人局不足以發現這類算術錯誤。
- 4 人局按最短距離穩定排序得到 `[0,1,3,2]`，與順座次序 `[0,1,2,3]` 不同。這證明兩個排序概念不能共用同一原語；觸發規則的預期順序仍須對照採用的上游語義判定。

日誌同時保留第一次以 Homebrew rustc 1.99.1 連結 rustup 1.95.0 產物失敗的 `E0514` 訊息，以及改用相符編譯器後的成功反例。重跑應使用與 `engine-native` 產物一致的工具鏈，不應為此執行 `cargo clean` 或破壞他人的編譯產物。

## 文檔與索引檢查

- [doc-links-audit.json](doc-links-audit.json)：13 份主文檔、1,607 處本地連結、750 個唯一目的地；確認一處壞鏈，位於 `docs/architecture/legacy-reference.md:59`。未驗證 Markdown 錨點或外部 URL。
- 同一 JSON 記錄 731 份編號 ADR、702 個唯一編號及 27 組重複編號，另有兩份標題編號與檔名不一致。這是需要修正索引與識別的證據，不是批量刪除歷史 ADR 的授權。
- [codegraph.log](codegraph.log)：同步 4,310 個變更檔案成功，仍有 8,028 個未解析引用。索引可輔助定位；不能把「查無 caller」解讀成沒有執行路徑。

文檔在稽核期間有並行更新，以上機械清單適用於其掃描時刻。其結果不證明文字描述與生產能力一致；整包驗收、規則等價和平台完成度仍需依具體場景核驗。

## 工件完整性與限制

[archive-provenance.json](archive-provenance.json) 記錄每份原始檔的 `/tmp` 來源、歸檔相對路徑、大小及 SHA-256。[SHA256SUMS](SHA256SUMS) 涵蓋本目錄全部其他工件（不含清單自身）。在本目錄執行 `shasum -a 256 -c SHA256SUMS` 可核驗歸檔完整性；雜湊只證明檔案未變，不證明測試範圍足夠。

本次未執行真實瀏覽器操作、各目標設備驗收、同設備前後性能基準，也未逐條重驗全部上游規則。已通過的 Rust 測試與確證反例可以同時成立；不能據此把 37 來源、跨平台或產品全量驗收標為完成。
