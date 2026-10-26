# NC02 套件面板瀏覽器 E2E（2026-10-25）

- 建置：`pnpm build`（[日誌](native-ui-build.log)）。
- 預覽：`pnpm preview --host 127.0.0.1 --port 4173`；API 以本機 mock 於 7700 提供 `community/browser-probe@0.1.0`，並記錄所有生命週期請求。
- 瀏覽器：Google Chrome headless（CDP Runtime.evaluate 點擊）。
- 驗證：
  - 設定頁顯示「原生擴展包」與 `community/browser-probe@0.1.0`（[DOM](native-ui-dom.html)）；
  - 點「停用」→ 收到 `POST /native/packages/disable`，狀態行顯示 `disable 完成`；
  - 點「激活」→ 收到 `POST /native/packages/activate`，顯示 `activate 完成`；
  - 點「掃描安裝」→ 收到 `POST /native/packages/import`，顯示 `安装扫描完成`；
  - 點「卸載」並接受 confirm → 收到 `POST /native/packages/uninstall`，顯示 `uninstall 完成`。
- 截圖：[native-ui-settings-full.png](native-ui-settings-full.png)（1280×1800，含面板、包、狀態行）。
- 原始請求：[native-ui-actions.log](native-ui-actions.log)。
- 限制：API 是 mock；真實服務端 + 瀏覽器同進程全鏈路尚未跑。服務端真實 HTTP 路由由 `real_http_route_lists_and_disables_package` 覆蓋。
