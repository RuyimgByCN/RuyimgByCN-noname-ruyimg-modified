# NC02 真实服务端 + 浏览器同链 E2E（2026-10-25）

## 环境

- `cargo build -p game-server` 构建真实二进制。
- 临时目录：`native-packages` 放 huanle-cards，`native-store` 放安装存储，`native-room-store` 禁用，原始内容指向仓库 `content/original`。
- `GAME_SERVER_ADDR=127.0.0.1:7700` 启动；日志见 [native-real-server.log](native-real-server.log)。
- 生产前端 `pnpm build` 后 `pnpm preview --port 4173`；Chrome headless + CDP 点击。
- 页面默认服务端 `http://127.0.0.1:7700`。

## 结果

- 设置页列出真实服务端包 `upstream-noname/huanle-cards@1.0.0`。
- 点「停用」→ 真实 `POST /native/packages/disable`，显示 `disable 完成`。
- 点「激活」→ 真实 `POST /native/packages/activate`，显示 `activate 完成`。
- 点「扫描安装」→ 真实 `POST /native/packages/import`，显示 `安装扫描完成`。
- 点「卸载」并接受 confirm → 真实 `POST /native/packages/uninstall`，显示 `uninstall 完成`；随后 `GET /native/packages` 返回 `[]`。
- 杀进程后以同一临时目录重启：启动日志 `native content ready packages=1`，`GET /native/packages` 重新列出 huanle-cards（见 [native-real-server-restart.log](native-real-server-restart.log)）。

截图：[native-real-settings.png](native-real-settings.png)（真实包列表 + 安装扫描完成）。
CDP 输出：[native-real-e2e.out](native-real-e2e.out)。

## 限制

- 浏览器 E2E 使用的是卡牌子包（huanle-cards），不是带 VM 武将/技能的完整包；完整 VM 包的选将/开局仍由 `vm_package_with_characters_installs_loads_and_runs` 进程内覆盖，尚未在真实服务端 + 浏览器链路复跑。
