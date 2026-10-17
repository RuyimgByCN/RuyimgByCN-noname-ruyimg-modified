# L16 拖拽读取八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。未改其它来源的 npack 字节。

本目录证明来源 **L16 拖拽读取**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04、L04、L05、L22、L02、L15、L17、L18、L26 与 L21 一起，整包计数为 **16/37**。其余 21 个来源未验收。如真似幻没有安装，不计入。

## 代码

拖入类型判定、默认扩展名、`extension.js` 存在性与公共前缀校验、原生 play 名单拒绝、一键完整包布局与 `.`/`_` 前缀过滤在 `crates/content-native-loader/src/drag_read.rs`。不挂标准局，也不新增宿主行为标签。空的武将、卡牌和技能索引不是空扩展。

安装包是 `packages/extensions/drag-read-upstream-liuli`（`upstream-liuli/drag-read@1.0.0`，由 `crates/content-runtime/examples/gen_drag_read_package.rs` 生成）。`logic_hash` 前四字节为 `195,210,59,136`，`asset_hash` 前四字节为 `65,123,92,41`，配置 `source_hash` 前四字节为 `87,88,59,133`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `drag_read_apply`。上游提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`，上游 `package.version` 是 `1.81`，作者 `诗笺`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l16_drag_read_eight -- --test-threads=8
cargo test --offline -p engine-native-wasm --lib drag_read_pack_json_enters_wasm_helper -- --test-threads=8
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.27s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，16 filtered，0.07s）。两条命令退出码都是 0。未跑 engine-native 全量（收口门禁另记）。

## 八项

`l16_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__drag-read/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源 `noname-liuli`。`我的扩展.zip`（mime 含 zip）是 zip；`我的扩展.7z` 是 7z；`archive`（mime 含 zip、名字不以 `.zip` 结尾）也走 7z；单个 `extension.js` 走单文件路径。默认扩展名去掉 `.zip`/`.7z`。`我的扩展/extension.js` 的公共前缀是 `我的扩展/`；根级 `extension.js` 前缀为空；嵌套扩展取最短候选的目录、同前缀候选通过。目录项过滤丢掉 `.`/`_` 前缀。一键完整包含 4 文件 + 7 目录 + cache/audio(6)/image(6)/extension(4)/node_modules(3) | `.zip` 名但 mime 空走忽略分支；`readme.txt` 忽略。缺 `extension.js` 拒绝；`boss` 等原生 play 名拒绝；两个不同目录的 `extension.js` 拒绝（非扩展）。`image/skin/` 不导出。空 `source`、非 JSON 配置拒绝 |
| 配置 | 商店 `content_hash` 等于本包 npack 字节的哈希，版本 `1.0.0`，依赖为空。武将、卡牌、技能索引都为空。文本含上游路径 `noname/extension/拖拽读取/extension.js`、提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade` 和 `upstream_version` `1.81`，作者 `诗笺` | 商店副本与仓库源逐字节一致；文本不含 `eval(`、`fetch(`、`fs.` |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../drag-read` 为 `Package` |
| 组合 | 同一次导入仍安装更新保护、liuli 金币、SJ Settings 与手杀全扩，版本都是 `1.0.0`，内容哈希仍等于各自未改的 npack。纯函数不改内容锁。不安装如真似幻 | 未安装版本拒绝；id 逃逸拒绝 |
| 保存恢复 | 重开商店后仍是 `1.0.0`，内容锁哈希与安装后相同。资产目录只有 `config.json` | 纯函数运行前后内容锁哈希不变 |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 未安装版本 `NotFound` |
| 目标平台 | `drag_read_pack_json_enters_wasm_helper` 读仓库 `config.json`。zip 计划、7z 拒绝、原生 play 拒绝、单文件拒绝都走同一字符串辅助，并以 `eval:0 \| fetch:0 \| write:0 \| dom:0` 结束 | 非 JSON 与空 `source` 以 `error:BadConfig` 开头。不调用 `wasm_bindgen` |
| 规则差异 | 拖入类型分支顺序、默认名去后缀、公共前缀校验、目录跳过名单、完整包固定布局、上游 `package.version` `1.81` 与作者，与上游 `extension.js` 里仍会执行的路径一致 | 不 eval 单文件 `extension.js`，不请求网络，不读写文件，不启动 Electron，不弹对话框。7z 扩展包路径在上游即抛错，本模块返回同一文案。不记为上游等价 |

## 登记差异

1. 单文件 `extension.js` 路径上游用 `eval` 取扩展名——原生不 eval，返回 `extension.js代码有错误，无法导入！`（登记差异，须改走带清单的包路径）。
2. 7z 扩展包导入上游直接抛错（`还不支持读取7z格式的扩展`）——本模块返回同一文案，不解压。
3. 解压落盘、对话框、一键完整包的实际 zip 生成与保存、更新检查（`fetch`）未迁入。
4. 原生 play 名单来自配置（`coin`/`boss`/`wuxing`/`cardpile`，与更新保护 `plays` 同口径）。
