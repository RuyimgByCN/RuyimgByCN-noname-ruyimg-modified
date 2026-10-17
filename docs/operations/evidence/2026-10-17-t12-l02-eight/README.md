# L02 SJ Settings 八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。未改 U04、L05、L22 的 npack 字节。

本目录证明来源 **L02 SJ Settings**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04、L04、L05、L22 一起，整包计数为 **10/37**。其余 27 个来源未验收。如真似幻没有安装，不计入。U01 末焱切片和 U02 仍未验收。G0 清单不单独构成这次上调。

## 代码

启动名单、`asset_version`、配置导入、版本比较、目录标记、下载筛选、文件故障和错误文本在 `crates/content-native-loader/src/sj_settings.rs`。确认和提示由调用方给出布尔或返回字符串。不挂标准局，也不新增宿主行为标签。

安装包是 `packages/extensions/sj-settings-upstream-liuli`（`upstream-liuli/sj-settings@1.0.0`）。`logic_hash` 前四字节为 `212,186,250,154`，`asset_hash` 前四字节为 `145,149,148,59`，配置 `source_hash` 前四字节为 `126,119,162,126`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `sj_apply`。上游提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`，上游版本 `1.4204`，`openTools` 为假。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l02_sj_eight -- --test-threads=1
cargo test --offline -p engine-native-wasm --lib sj_pack_json_enters_wasm_helper -- --test-threads=1
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.06s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，10 filtered，0.00s）。两条命令退出码都是 0。未跑 engine-native 全量。

## 八项

`l02_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__sj-settings/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源 `noname-liuli`。安卓键为空时名单是 SJ Settings，并只在当前值为假时启用。缺省 `asset_version` 写成 `无`。`{"a":1}` 编码为 `eyJhIjoxfQ==`。数据库导入、保留旧名单时，旧名单 `[甲, SJ Settings]` 加上 `[B]` 得到 `B,甲,SJ Settings`；不保留时得到 `B,SJ Settings`。没有扩展键时，不保留会在剩余名单后补上 SJ Settings。`1.2` 对 `1.10` 为旧，`1.10` 对 `1.2` 为新，`1` 对 `1.0` 相等，`无` 对 `1.2` 两个方向都是新。同名版本字符串相等。目录标记：版本字符串不等为不兼容，较新为可更新，相等为相同，较旧或无序为本地更高。默认命令链得到 `boot:SJ Settings en:SJ Settings=1 key:absent asset:无 version:false version:true version:equal version:true version:true version:undefined version:true version:true asset:refused console:0 eruda:0 fault:skip fault:retry` | 空载荷不导入。非法 base64、`null`、非对象的数据库载荷拒绝，且不清存储。非 BMP 字符拒绝编码。`openTools` 不是布尔、空的上游版本拒绝。安卓键已有名单时不自动补 SJ Settings。不兼容项两次确认仍不选。`Unable to resolve host` 或 http 404 跳过；500、缺码、小写 host 文案重试。扩展导入才改名单；其他类型不重载 |
| 配置 | 商店 `content_hash` 等于本包 npack 字节的哈希，版本 `1.0.0`，依赖为空。文本含上游路径 `noname/extension/SJ Settings/extension.js`、提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`、版本 `1.4204`，`openTools` 为假 | 空 `source`、空 `upstream_version`、`openTools` 不是布尔都拒绝。`manifest.enabled=false` 的 `sj-settings-disabled` 被跳过 |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../sj-settings` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍安装 `upstream-liuli/coin@1.0.0`、`upstream-noname/coin@1.0.0` 与 `upstream-liuli/elo-sanction@1.0.0`，内容哈希仍等于各自未改的 npack。公式不改内容锁。不安装如真似幻 | 失败的缺依赖安装不撤掉已安装包。升级和回滚 SJ Settings 后两个金币包与段位制裁仍是 `1.0.0` |
| 保存恢复 | 非数据库导入成功后保留非空的 `noname_inited` 与调用方前缀键，空字符串不保留。数据库成功写入配置键和备注，不整段清空。重开商店后仍是 `1.0.0`，来源仍是 `noname-liuli`，版本仍是 `1.4204` | 格式错误和解析错误不清存储。外部包导入写入的 `noname_inited` 在随后的其他类型命令里仍可读，该命令本身不改名单、不请求重载 |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `sj_pack_json_enters_wasm_helper` 读仓库 `config.json`。默认链与宿主测试相同。`boot:["甲"]` 得到 `boot:甲 en:甲=1 key:removed`。保留导入得到 `exts:甲,SJ Settings`。扩展、安装包、其他类型依次为 `external:extension`、`external:package`（`inited:blob`）和 `external:other`（仍打印 `inited:blob`，`reload:0`） | `eval:update.js`、在线目录和 PDF 分别是 `eval:refused`、`catalog:offline`、`catalog:refused`、`asset:refused`。`openTools` 为真时第一次 `console:1 eruda:0`，第二次因已经打开而是 `console:0`。空来源、`boot:null`、坏的 onerror 都以 `error:` 开头 |
| 规则差异 | 启用名单、导入并集、版本比较、目录标记、下载筛选、故障分类和错误文本与上游 `extension.js` 的读取一致。空文件列表不编造下载地址 | 不请求网络，不 eval `update.js`，不打开 PDF，不启动 eruda，不调用 `game.reload` 或 `game.loop`。`reload_requested` 可以为真，`restarted`、`continued`、`looped` 保持为假。Cordova 拷贝、进度条和系统通知没有迁入。数据库导入要求 `config` 与 `data` 都是对象；上游把非空数组当对象，这一处更严，已标明。不记为上游等价 |
