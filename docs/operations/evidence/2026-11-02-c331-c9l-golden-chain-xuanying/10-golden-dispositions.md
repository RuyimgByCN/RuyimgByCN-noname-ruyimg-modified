# 12 場 golden 差異的裁決材料（2026-11-02 核對，未重錄）

來源：`evidence/phase9d/golden-business-and-outcome-dispositions.json`（2026-10-31 製作），
本輪以 `cargo test --workspace --no-fail-fast` 重跑後，12 個 actual 雜湊與該紀錄**逐位元組相同**（12/12）。

| 場景 | 成對輸入一致 | 業務面差異擷取 | 結果欄位差異 | 處置 |
| --- | --- | --- | --- | --- |
| sha_dodge_then_damage | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| death_terminal_single | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| sancai_nested_chain | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| yishang_redirect | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| suidun_window_matrix | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| shouyi_dynamic_skill | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| rollback_invariants | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| equip_distance_matrix | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| chain_fire_propagation | True | 4 | 0 | source_supported_individual_chain_unlink_order_change |
| aoe_response_queue | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| duel_alternating_response | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |
| nullify_chain_on_savage | True | 0 | 0 | business_equal_at_all_observed_commands_and_final |

## 判讀

- 11 場：`business_equal_at_all_observed_commands_and_final`——已觀察命令的完整狀態／日誌／RNG／座位視圖／選擇全部相同，
  只有 canonical 雜湊不同（內部收據/瞬態面進了雜湊）。
- 1 場：`chain_fire_propagation`＝`source_supported_individual_chain_unlink_order_change`——
  來源支持的真實順序變更（鐵索先記自身傷害再解鏈），屬**語意變更**，不能只改期望值。
- 結果欄位差異總數：0（Accepted 的 state_hash 差異不計入「結果欄位」）。

## 建議（供你裁決，我未執行）

1. 11 場 business-equal：可以「hash-only 重錄」，但需在規則基準/證據標明原因（收據入雜湊），且保留舊 hash 記錄。
2. `chain_fire_propagation`：先確認上游語意（連環是否先結算自身傷害），再決定改實作或把差異登記為既知偏差；不能直接重錄。
3. 兩者都不影響 registry 262/290 的來源驗收計數。
