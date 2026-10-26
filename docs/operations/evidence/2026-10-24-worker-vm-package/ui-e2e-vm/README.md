# NC02 完整 VM 武将包：真实服务端 + 浏览器同链 E2E（2026-10-25）

## 夹具

`emit_vm_fixture.rs` 用正式 content-compiler 在测试时生成：
- 顶层技能 `vm_fixture_phase_end`（phase_end 摸 1）；
- 武将 `vm_fixture`（qun，4 HP，技能 `vm_fixture_phase_end`）；
- 完整 npack + `assets/character.json`。

## 结果

- 真实 `game-server` 导入后 `/native/packages`：
  `official/vm-fixture@0.1.0`（[native-vm-import.json](native-vm-import.json)、[服务端日志](native-vm-server.log)）。
- `POST /native/games` 选将 `vm_fixture`、4 座、40 步：游戏跑完，`catalog_generals=94`（base 93，+1），`steps=217`，见 [native-vm-games.json](native-vm-games.json)。
- 浏览器（生产构建 + headless Chrome/CDP）面板列出 `official/vm-fixture@0.1.0`，依次：
  - 停用 → `disable 完成`；
  - 激活 → `activate 完成`；
  - 扫描安装 → `安装扫描完成`；
  - 卸载并确认 → `uninstall 完成`；随后 `/native/packages` 返回 `[]`。
- 原始输出：[native-vm-e2e.out](native-vm-e2e.out)；截图：[native-real-settings.png](native-real-settings.png)。

## 结论

完整 VM 武将包的正式安装、选将、开局、停用、激活、重新扫描、卸载、客户端面板与真实服务端同链均已跑通；客户端未重新构建。
