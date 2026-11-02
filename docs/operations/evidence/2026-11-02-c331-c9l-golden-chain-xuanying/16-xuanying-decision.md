# 玄影（SwdXuanying）停用：裁決材料（2026-11-02；已依權威來源更正建議）

> 更正（2026-11-02 本輪）：`docs/operations/evidence/2026-10-31-c331-source-followups/` 的原文寫明：
> 「Xuanying damage is registered as a child definition but **absent from both parent groups (sha/use only)**;
> selected package grants the parent, not damage. **Native parent ownership incorrectly makes damage
> reaction reachable.**」——所以正解是 **parent-group grant（方案 B）**，不是恢復單一雙面行為；
> 先前的「方案 A（建議）」與登記載體矛盾，已作廢。

## 事實（兩棵樹對照 + 實測）

| 觀察 | 5fc0f774（主樹） | C331 私有樹 |
| --- | --- | --- |
| `filtering/swd.rs` 的 `SwdXuanying` filter | 真條件：自己受傷（真實）、已橫置、來源存活 | `=> false`（附註「旋影父技不授予 damage 子技；保留獨立的主动入口」） |
| 主動面 `activation/swd_skills.rs`（未橫置時可出殺） | 有 | **有**（未受影響） |
| 受傷反擊面 `step/swd_damage_targets.rs` | 可觸發 | 死碼（filter 永假） |
| `report_event_arms` 條目數 | **88** | **87**（差額即 `SwdXuanying`） |

private tree 的 `test_report_event_arms.py::test_damage_arms_are_enumerated` 斷言 88 → 在 C331 樹失敗；
這是**真實功能缺口**的登記載體訊號，不是文件噪音。

## 為什麼 C331 停用它（依註解推測，未經作者確認）

C331 把「父技 + 子技」的授予關係顯式化（invocation/觸發順序重構）。玄影在上游/原始設計是
「父技授予受傷反擊子技」；C331 的 host 模型目前沒有「父技授予子技」的通道，作者選擇先停用
反擊面，避免「未授予卻自動觸發」。

## 兩個可選方案（我未執行）

**方案 A（已作廢）：恢復為單一雙面技能**
- 把 `filtering/swd.rs` 的 `SwdXuanying => false` 還原成主樹條件。
- 依據：本專案的 swd 內容資料只宣告一個技能（tag 387 → `HostBehavior::SwdXuanying`），
  package 資料裡沒有 group/sub-skill 宣告；「父技/子技」是實作敘事，不是資料事實。
- 代價：arms 恢復 88（測試轉綠），玄影反擊面恢復；需在規則基準/證據登記「雙面合併為一行為」
  （與主樹一致，不是新差異）。

**方案 B（正解，依權威來源）：parent-group grant——把 damage 子定義納入父技 group，由父技授予後才可達；native 端不得因父技所有權就讓 damage 反應可達**
- 在 host 模型加入「父技授予子技」通道 + package 資料的 group 宣告，再由父技在條件成立時授予反擊面。
- 代價：需新資料欄位/ABI 討論；arms 可能變 89 或維持 87（登記需重寫）；不是 C9L 範圍。

## 採方案 B 需同步（取代先前的方案 A 清單）

1. 內容資料：把 damage 子定義加入父技 group（兩個 parent group 皆需，sha/use 之外）。
2. `filtering/swd.rs`：把 `SwdXuanying => false` 還原成真實條件（已橫置、自己受傷、來源存活）。
3. native grant 可達性：只認「父技已授予」的實例，不得因父技所有權就讓子技可達（C9L 的 `=> false` 在此步移除）。
4. `report_event_arms` 應回到 88（子定義重新可達）；若內容資料不改而只還原 filter，就等於恢復 C331 想修的缺陷，**不可接受**。

## 附：若最終決定「暫不實作」的方案 A 清單（僅供對照）

1. `crates/engine-native-content/src/content/host/filtering/swd.rs`：還原 filter 條件。
2. `report_event_arms` 的 88 斷言自動恢復（無需改）。
3. 需要一場真實回歸（已橫置受傷 → 解除橫置 + 對來源虛擬殺）；現有 `native_kernel/swd*` 測試可為基礎。
4. 若你認為「停用」才是正解，則必須把 arms 登記改成 87 並在規則基準登記「玄影反擊面未實作」，
   不能兩邊都不動（目前就是測試紅的狀態）。
