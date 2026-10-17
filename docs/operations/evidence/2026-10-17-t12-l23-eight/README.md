# L23 电脑适配八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。

本目录证明来源 **L23 电脑适配**。写入当时与已计入的 U06、L06、L07 一起为 **4/37**。其后的总账以进度主稿为准。G0 清单不单独构成这次上调。

## 代码

预设、导出和鼠标点击判定在 `crates/content-native-loader/src/desktop_adapt.rs`。它不创建 DOM，不注入 CSS，不调用 `game.use_throw_emoji`，也不移动窗口。导出只返回净化后的文件名和 UTF-8 字节。

安装包是 `packages/extensions/desktop-adapt-upstream-liuli`（`upstream-liuli/desktop-adapt@1.0.0`）。`logic_hash` 前四字节为 `186,129,18,138`，`asset_hash` 前四字节为 `16,109,65,215`，配置 `source_hash` 前四字节为 `220,151,160,114`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l23_desktop_adapt -- --test-threads=1 l23_eight_dimension_from_fresh_install
cargo test --offline -p engine-native-wasm --lib -- --test-threads=1 desktop_adapt_pack_json_enters_wasm_helper
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.06s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，4 filtered，0.00s）。加入该包后重跑 U06、L06、L07 八项，各 **1 passed, 0 failed**。未跑 engine-native 全量。

## 八项

`l23_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__desktop-adapt/1.0.0/`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 桌面宿主得到 `android` 预设；5 像素鼠标位移为 `Tap`；触摸布局为 `NativeTouch`；手杀 UI 开启时表情桥为 `Requested` | 刚好 10 像素和勾股 10 为 `Drag`；自己、阵亡、终局或没有抛表情函数时为 `Ignored` |
| 配置 | 商店副本解析为结算提示关闭、雷霆 UI 补丁关闭、阈值 10、布局 `mouse` | 未知布局、未知设备、把雷霆补丁写成 true 都拒绝；`manifest.enabled=false` 不安装 |
| 依赖 | npack `dependencies` 为空。未启用手杀 UI 时桥为 `Inactive`。商店里没有手杀 UI 包 | 激活 `9.9.9` 为 `NotFound`；包 id `../desktop-adapt` 为 `Package`；缺依赖的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍激活欢乐卡牌与全能搜索。应用预设和切换布局不改变内容锁 | 失败的缺依赖安装不撤掉全能搜索；回滚后欢乐卡牌仍在 |
| 保存恢复 | 布局切到触摸后的快照可原样恢复。重开商店后仍是 `1.0.0`，手机宿主仍得到 `android` 预设 | 快照里的未知布局拒绝 |
| 安装升级 | npack `2.0.0` 激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖激活不改写激活集 |
| 目标平台 | `desktop_adapt_pack_json_enters_wasm_helper` 读仓库配置，得到 `profile:android`、`pointer:tap` 和两条预设 | 导出 `..` 为 `error:unsafe`；抛表情为 `dom_refused`；空配置与未知宿主拒绝 |
| 规则差异 | 导出名 `../a:b` 净化为 `...a.b`，且函数不写磁盘。空文件名用 `noname` | 窗口移动、DOM、CSS 和 `game.use_throw_emoji` 未迁入。`invoke_emoji` 返回 `DomRefused`。导出名 `.` / `..` 拒绝，上游会照样下载。不记为上游等价 |
