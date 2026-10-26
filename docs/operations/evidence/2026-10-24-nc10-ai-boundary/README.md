# NC10 / P2：AI 合法座位資訊與有界排程核驗（2026-10-24）

核驗基準 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作樹；規則 **v322**。本批**沒有新增產品代碼**，而是對既有 NC10 AI 邊界與排程實作做可重現核對，避免重做已存在的能力；未把本批當成全量 AI 驗收。

## 核對內容

- `nc10_ai_information_boundary`（3 測）：把對手手牌與牌庫做確定性置換，保持座位可見資訊不變，`plan_action` 與 `legal_actions` 必須逐欄相同；反向對照置換自己的手牌必須能改變規劃結果（證明探針有分辨力）。涵蓋非靜止態窗口，直接對應「AI 只讀合法座位資訊」。
- `nc10_ai_bounds`（5 測）：`BoundedPlanner` 節點預算、deadline、取消旗標、預算降級與確定性。
- `nc10_ai_boundary_and_cancel`（6 測）：邊界與取消的組合面（含過期/取消後不套用）。

## 結果

命令與完整輸出：

```bash
cargo test --offline -p engine-native \
  --test nc10_ai_information_boundary \
  --test nc10_ai_bounds \
  --test nc10_ai_boundary_and_cancel
```

- **14 passed / 0 failed / 0 ignored**，exit 0；见[日志](tests.log)、[退出码](tests.exit)。
- 三个 target：3 + 5 + 6。

## 后续补齐与边界

- 已补 **宿主自持规划池**（复用/降级/取消/过期）与**尾延迟基线**，见[池测试日志](planner-pool-latest.log)、[退出码](planner-pool-latest.exit)。
- 已补**主动技候选**（变体 0/1 不进、变体 2 进且实际选择）与**技能威胁度加权**。
- 仍不据此关闭 NC10：未覆盖非静止态**手牌身份**的机器自省（隐藏牌面身份而非顺序）、质量基线与长局多房间负载。
- engine-native 全量最近为 **1555/0/71**（含新增 AI 测试），见[R10 日志](../2026-10-24-up37-r10-metadata/workspace-variant1.log)；本目录原始 14 测日志保留历史语境。
- 未改规则/存档版本，未改 golden。
