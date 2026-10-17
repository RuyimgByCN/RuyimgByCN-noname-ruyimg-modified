# L17 斗转星移八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。未改 U04、L05、L22、L02、L15 的 npack 字节。

本目录证明来源 **L17 斗转星移**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04、L04、L05、L22、L02、L15 一起，整包计数为 **12/37**。其余 25 个来源未验收。如真似幻没有安装，不计入。U01 末焱切片和 U02 仍未验收。G0 清单不单独构成这次上调。

## 代码

禁将方案、导入导出、描述改写和斗地主排序在 `crates/content-native-loader/src/douzhuan.rs`。武将标签、稀有度、星级和抽样由调用方给出。不挂标准局，也不新增宿主行为标签。空的武将、卡牌和技能索引不是空扩展。

安装包是 `packages/extensions/douzhuan-xingyi-upstream-liuli`（`upstream-liuli/douzhuan-xingyi@1.0.0`）。`logic_hash` 前四字节为 `25,68,250,67`，`asset_hash` 前四字节为 `155,110,200,221`，配置 `source_hash` 前四字节为 `46,106,182,186`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `douzhuan_apply`。上游提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`。出厂 `jinJiangRuZhen` 是 `online`，`forbidReload` 是 true，`miaoshuXiuGai` 是 false。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l17_douzhuan_eight -- --test-threads=8
cargo test --offline -p engine-native-wasm --lib douzhuan_pack_json_enters_wasm_helper -- --test-threads=8
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.07s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，12 filtered，0.00s）。两条命令退出码都是 0。日志是同一二进制的后续运行，`Finished` 分别为 0.06s 和 0.05s。未跑 engine-native 全量。

## 八项

`l17_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__douzhuan-xingyi/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源 `noname-liuli`。出厂不套用方案：禁将路径把缺省网络看成 offline，而 `online` 不含这串。网络改成 `online` 且身份使用方案 1 时，导入 `['liubei','liubei','caocao']` 后 `liubei` 得到 `forbidai`，重复 id 留在方案里，没有标签数组的 `caocao` 不写标记。已有 `unseen` 的武将留在 lock 和 now，新禁的不进入 now。方案 0 是空名单，门仍打开。故事背景盖过乱斗，太虚幻境读方案 1。`hide` 收成 `unseen` 并追加 `imforbid`。`line` 能命中 `online`。斗地主、抽样 0.9、两名 legend 选第二名；抽样 0.79 选第一名。农民分差 2 可以换，第二名 0 分不换。双将用调用方给出的余下下标。`伤害不大于2且不小于1，阴阳` 改成 `伤害≤2且≥1，阳阴`。`阳阴` 对调成 `阴阳`。方案名 `新禁将方案` 收成四字 `新禁将方`。全部禁用再开启时，keep 里的 id 留下。将包计数 0 会移出方案。上游 `标准包` 以 `old_caocao` 开头、以 `mateng` 结尾，`风林火山` 以 `zhulu_jiaxu` 结尾，两份名单不同 | 网络 `offline` 不套方案。网络 `onlineX` 既不套方案，也不做斗地主排序。没有该模式的方案、或方案号没有对应数组时不套。身份模式不排序。空改名不改方案名。`alert(1)`、带调用的文本和非字符串元素拒绝。`forbidReload` 关掉且上次模式不同时仍不调用 `game.reload`。描述改写后的函数文本仍以 `eval:0` 结束 |
| 配置 | 商店 `content_hash` 等于本包 npack 字节的哈希，版本 `1.0.0`，依赖为空。武将、卡牌、技能索引都为空。文本含上游路径 `noname/extension/斗转星移/extension.js` 和提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`。出厂方案名是方案一、方案二、方案三，四个方案都是空数组，模式方案为空 | 空 `source` 拒绝。`manifest.enabled=false` 的 `douzhuan-xingyi-disabled` 被跳过。包文本不含 `playAudio`、`game.reload`、`eval(` |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../douzhuan-xingyi` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍安装 `upstream-liuli/coin@1.0.0`、`upstream-noname/coin@1.0.0`、`upstream-liuli/elo-sanction@1.0.0`、`upstream-liuli/sj-settings@1.0.0` 与 `upstream-liuli/shousha-full@1.0.0`，内容哈希仍等于各自未改的 npack。公式不改内容锁。不安装如真似幻 | 失败的缺依赖安装不撤掉已安装包。升级和回滚斗转星移后手杀全扩、两个金币包、段位制裁和 SJ Settings 仍是 `1.0.0` |
| 保存恢复 | 重开商店后仍是 `1.0.0`，来源仍是 `noname-liuli`，默认行不变。把导入后的方案数组交回去得到同一行。稳定 id 不变 | 公式运行前后内容锁哈希不变。资产目录只有 `config.json` |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `douzhuan_pack_json_enters_wasm_helper` 读仓库 `config.json`。默认行、导入禁将、offline 不套用、描述改写和斗地主第二名都走同一字符串辅助，并以 `reload:0 \| audio:0 \| eval:0` 结束 | 空来源以 `error:BadConfig` 开头。`alert(1)` 以 `error:BadPlan` 开头。不调用 `wasm_bindgen` |
| 规则差异 | 禁将门、indexOf、标记追加、描述替换顺序、导出拼法和斗地主阈值与上游 `extension.js` 一致。星级下标按全能、主公、忠臣、反贼、内奸、地主、农民。菜单不打开时，boss、太虚和乱斗仍可按模式方案写标记 | 不打开菜单，不播放音频，不画启动页、势力颜色、背景和 CSS。不调用 `game.reload`，不 `eval` 方案或动态描述。缺省网络时禁将看 offline、选将强化看 online。空数组导出是 `['']`，再读入是一条空字符串，不是空数组。稀有度和抽样由调用方给出。不记为上游等价 |
