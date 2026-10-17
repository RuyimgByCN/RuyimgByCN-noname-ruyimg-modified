# T13 产品面：扩展入局、恢复、桌面启动与联机拒绝（2026-10-17）

核验日期：2026-10-17。规则 `STANDARD_RULESET_VERSION = 295`，存档 `ARCHIVE_FORMAT_VERSION = 80`。本批不改规则语义，不重录 14 条 golden。

## 已测到的产品路径

1. **game-server 真进程 HTTP**（`target/debug/game-server`，`GAME_SERVER_ADDR=127.0.0.1:7773` 与 `7774`）。启动时安装 `packages/extensions` 的 15 个包。
   - `POST /native/matches` 空包：目录 **124** 张。
   - `POST /native/matches` `huanle-cards`、seed 17017、4 座：目录 **131** 张（+7），返回 NGSA 十六进制。
   - 同进程 `POST /native/matches/restore` 同一包：`catalog_cards=131`。
   - 停掉进程后再起第二个进程、同一原生包存储、同一存档：恢复仍是 131。这是跨进程接力。
   - 缺包（`packages:[]`）：**400** `恢复失败: RuleVersionMismatch`（snapshot 锁 vs 标准目录 295）。
   - 另一已安装包 `wuxing`：`RuleVersionMismatch`（catalog 指纹不同）。
   - 未安装 id `not-installed` 与目录名 `cardpile-upstream-noname`：**400** `包未安装`。包 id 是 `cardpile`，不是目录名。
   - 错版：`load` 读的是安装记录里的 `extension_root`（drop-in 源），不是存储目录里的 cards.json 副本。改存储副本不会改变内容锁（第一次探针因此仍 200，不记为拒绝成功）。把 drop-in 副本的 `pack.version` 从 1 改成 2 后再恢复：**400** `RuleVersionMismatch`。改回 1 后恢复再次 **200**、131 张。源码树 `packages/extensions` 未改。
2. **wasm 窄壳**。`node apps/web-client/scripts/build-native.mjs` 重新生成 `src/worker/native-pkg`。`NativeEngine.new_with_packs_json` / `restore_hex_with_packs_json` / `cards_with_prefix` 进入胶水。Node 冒烟 13/0：欢乐卡牌 `hnl_` **7** 张进入新对局，标准目录恢复扩展存档被拒，同包恢复仍为 7，错版错误文案含 `恢复失败`。
3. **桌面入口**。`cargo check --offline -p desktop-client --features tauri-shell --bin noname-next-desktop` 退出 0。`cargo run` 同一目标打出 `Running target/debug/noname-next-desktop`，进程保持存活（当时 pid 57570），随后由本会话停止。该壳是 content-runtime 宿主命令，本身不跑 engine-native 扩展对局；扩展对局的保存、恢复和拒包证据在 game-server 与 wasm 胶水上。

## 定向测试

```
cargo test --offline -p engine-native-wasm --lib pack_roundtrip
  pack_roundtrip::installed_pack_enters_new_match_and_restore ... ok
  1 passed; 0 failed
cargo test --offline -p game-server --test native_product_path t13_installed_extension
  t13_installed_extension_enters_match_and_restores ... ok
  1 passed; 0 failed; 4 filtered out
node apps/web-client/scripts/native-worker-smoke.mjs
  {"total":13,"failed":0}
```

engine-native 全量、Gate 1b、workspace、conformance、cross-target、sync-verify、web-client build 本批未跑，不记为当前全绿。

## 仍然不是整包、也不是浏览器点击路径

- 扩展管理页只保存元数据。对局开始要由调用方把 cards.json 数组放进 `WorkerRequest.packsJson`（`client.start` / `restoreHex` 已能转发）。本批没有在浏览器里点开一局。
- 桌面窗口启动不等于桌面壳内完成了错版拒绝。
- 登记差异保持为差异。本批不把 15 个转换包或这次 HTTP 200 写成 37/37 整包验收。

日志：[http-product.log](http-product.log)、[wasm-smoke.log](wasm-smoke.log)、[targeted-tests.log](targeted-tests.log)、[tauri-launch.log](tauri-launch.log)。
