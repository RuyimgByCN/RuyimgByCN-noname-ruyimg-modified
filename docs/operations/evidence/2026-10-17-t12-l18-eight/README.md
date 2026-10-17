# L18 更新保护八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。未改 U04、L05、L22、L02、L15、L17 的 npack 字节。

本目录证明来源 **L18 更新保护**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04、L04、L05、L22、L02、L15、L17 一起，整包计数为 **13/37**。其余 24 个来源未验收。如真似幻没有安装，不计入。U01 末焱切片和 U02 仍未验收。G0 清单不单独构成这次上调。

## 代码

目录识别、更新源、版本比较、断点名单和检查按钮在 `crates/content-native-loader/src/update_guard.rs`。不挂标准局，也不新增宿主行为标签。空的武将、卡牌和技能索引不是空扩展。

安装包是 `packages/extensions/update-guard-upstream-liuli`（`upstream-liuli/update-guard@1.0.0`）。`logic_hash` 前四字节为 `24,85,147,164`，`asset_hash` 前四字节为 `19,187,12,103`，配置 `source_hash` 前四字节为 `11,100,114,252`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `update_guard_apply`。上游提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`。出厂 `update_link` 是空字符串，`cancelForbid` 和 `autoAddExts` 都是 true，上游 `package.version` 是 `1.5`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l18_update_guard_eight -- --test-threads=8
cargo test --offline -p engine-native-wasm --lib update_guard_pack_json_enters_wasm_helper -- --test-threads=8
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.07s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，13 filtered，0.00s）。两条命令退出码都是 0。日志是同一二进制的后续运行，`Finished` 都是 0.05s。首次编译分别是 0.83s 和 1.25s。未跑 engine-native 全量。

## 八项

`l18_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__update-guard/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源 `noname-liuli`。空的 `update_link` 写成 URC。已保存的 `github` 不改当前地址。卸载改回 coding。扩展菜单从 URC 切到 github 并清缓存；同一项不清缓存。`coin2`、甲、丙在有 `extension.js` 时加入，消息是 `已识别扩展：coin2、甲、丙`。`1.2` 对 `1.10` 提示更新，`1.10` 对 `1.2` 视为本地更高，相等则相同。`1.2.0` 对 `1.2` 仍提示。更新日志第一条保留，其后含 `://` 的行丢掉。断点 `a,b,a` 确认后留下 `a,b`。检查游戏更新的落点是「当前版本已是最新」，`updating` 为假。检查素材更新后 `UpdatingForAsset` 仍为真。`{"msg":{"user_not_login":"用户未登录"}}` 判为登录错误。有下载能力且缓存已在时直接 ready。失败四次后再成功仍是 ready。`cancelForbid` 打开时把调用方给出的扩展名标成启用 | `coin`、`boss`、`wuxing`、`cardpile`、`SJ Settings` 和已在名单里的目录不加入。没有 `extension.js` 的乙不加入。`autoAddExts` 关掉时不扫描。扩展菜单拒绝 xuanwu。总菜单把 xuanwu 当成不存在的源，不清缓存。拒绝重下时名单清空，且不下载。按钮文案不对时不动状态。`alert(1)`、普通 JSON、`msg` 为 null 或空对象都拒绝执行。没有下载能力时即使缓存已在也返回手动更新。五次失败是最大重试。空尝试是其他错误。`cancelForbid` 关掉或没有翻译表时不强制启用 |
| 配置 | 商店 `content_hash` 等于本包 npack 字节的哈希，版本 `1.0.0`，依赖为空。武将、卡牌、技能索引都为空。文本含上游路径 `noname/extension/更新保护/extension.js`、提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade` 和 `upstream_version` `1.5`。出厂 `cancelForbid`、`autoAddExts` 为真，`update_link` 为空，`brokenFile` 为空 | 空 `source` 拒绝。`manifest.enabled=false` 的 `update-guard-disabled` 被跳过。包文本不含 `eval(`、`game.reload`、`fetch(` |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../update-guard` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍安装两个金币包、`elo-sanction`、`sj-settings`、`shousha-full` 与 `douzhuan-xingyi`，版本都是 `1.0.0`，内容哈希仍等于各自未改的 npack。公式不改内容锁。不安装如真似幻 | 失败的缺依赖安装不撤掉已安装包。升级和回滚更新保护后斗转星移、手杀全扩和 liuli 金币仍是 `1.0.0` |
| 保存恢复 | 重开商店后仍是 `1.0.0`，来源仍是 `noname-liuli`，默认行不变。内容锁哈希与安装后相同 | 公式运行前后内容锁哈希不变。资产目录只有 `config.json` |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `update_guard_pack_json_enters_wasm_helper` 读仓库 `config.json`。默认行、`alert(1)` 拒绝执行、确认断点名单都走同一字符串辅助，并以 `eval:0 \| fetch:0 \| write:0 \| reload:0` 结束 | 空来源以 `error:BadConfig` 开头。不调用 `wasm_bindgen` |
| 规则差异 | 更新源表、xuanwu 的两处菜单、目录跳过名单、`Number` 分段比较、日志过滤、`Set` 去重、错误计数第六次归零、登录 JSON 和 `canUpdate` 恒为真，与上游 `extension.js` 里仍会执行的路径一致。`parseSize` 的 `slice(index + 1, 2)` 不去掉 `.00`。素材按钮把 `game.Updating` 写成假，却留下 `UpdatingForAsset` | 不请求网络，不写盘，不 `eval` 更新脚本，不睡眠两秒，不画菜单、进度条和 Cordova 通知。注释中的下载、覆盖更新和素材筛选不执行。不记为上游等价 |
