# L26 笨蛋插件八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。未改 U04、L05、L22、L02、L15、L17、L18 的 npack 字节。

本目录证明来源 **L26 笨蛋插件**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04、L04、L05、L22、L02、L15、L17、L18 一起，整包计数为 **14/37**。其余 23 个来源未验收。如真似幻没有安装，不计入。U01 末焱切片和 U02 仍未验收。G0 清单不单独构成这次上调。

## 代码

点数筛选、杀目标提示、设备判断、配音配对、衍生描述和选将语音在 `crates/content-native-loader/src/fool.rs`。不挂标准局，也不新增宿主行为标签。空的武将、卡牌和技能索引不是空扩展。

安装包是 `packages/extensions/fool-plugin-upstream-liuli`（`upstream-liuli/fool-plugin@1.0.0`）。`logic_hash` 前四字节为 `246,137,217,154`，`asset_hash` 前四字节为 `24,103,185,221`，配置 `source_hash` 前四字节为 `49,57,192,21`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `fool_apply`。上游提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`。出厂表情、计算优化和选将语音打开，配音修复和衍生详细显示关闭，选中卡牌提示是 `shousha`。上游 `package.version` 是 `2.0.2`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l26_fool_eight -- --test-threads=8
cargo test --offline -p engine-native-wasm --lib fool_pack_json_enters_wasm_helper -- --test-threads=8
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.09s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，14 filtered，0.01s）。两条命令退出码都是 0。日志是同一二进制的后续运行，`Finished` 分别是 0.19s 和 0.06s。首次编译分别是 1.51s 和 1.57s。未跑 engine-native 全量。

## 八项

`l26_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__fool-plugin/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源 `noname-liuli`。资援和敏思在已选点数 10 时接受 3、拒绝 4。方统第一张可以是手牌或装备，不能是「方」；选中手牌 20 后再接受「方」16，拒绝 17 和另一张装备。点数和为 36 时选择数等于已选张数，否则再加 2。界烈弓、手杀样式、在范围内写成「强中加伤」；十周年样式写成「可烈弓 不可响应 伤害+1」；其他样式收成「可烈弓」。DIY 烈弓只写「可烈弓」。伏骑看对方到己方的距离，界伏骑看己方到对方。势敌标记为 0 且杀为黑色时写「不可响应」。武神红桃同样。进讨在本阶段杀数为 0 时手杀样式写「加伤」，十周年样式写「 伤害+1」，即使不在距离内。iPhone、Android、OperaMini 判为手机。配音修复把无配音的「杀」记到后出现的数字音源 `c`，并把 `d2` 记到父技能。衍生描述把标签外的「资援」换成「※资援」，标签内的留下。选中武将且允许说话时，`rende` 的第一条是 `skill/rende1.mp3`。带武将名的两条里取第二条 `skill/rende_liubei2.mp3`。互相指向的配音收成技能引用。`noChoosingAudio` 的技能让位给下一条。砸花在禁连发时用 linear、0.7 秒，并记下 `throw_flower2`。酒在禁连发时是 0.9 秒。调用方给出的随机数 1、1 得到横向 50 | 手牌数不够的 DIY 烈弓、不在距离内、距离为 2 的伏骑、奇数势敌标记都不写提示。不是自己出牌、目标是自己、或样式为 close 时不写提示。提示不改变原过滤结果。Windows 和带空格的 Opera Mini 不是手机。公共技能、数字 0 的音源不借出，`zero2` 保持无配音。只出现在 `>资援<` 里的名字不改。父技能和衍生技同名时不改。衍生技没有描述时不改。没选中、或不允许说话时不挑语音。对自己、或正在投掷时不投。关掉表情开关不投。选择卡牌优化不运行 |
| 配置 | 商店 `content_hash` 等于本包 npack 字节的哈希，版本 `1.0.0`，依赖为空。武将、卡牌、技能索引都为空。文本含上游路径 `noname/extension/笨蛋插件/extension.js`、提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade` 和 `upstream_version` `2.0.2`。出厂表情、计算和选将语音为真，配音修复和衍生显示为假，提示样式是 `shousha` | 空 `source` 拒绝。`manifest.enabled=false` 的 `fool-plugin-disabled` 被跳过。包文本不含 `eval(`、`game.reload`、`fetch(`。打开配音修复后默认行含 `peiyin:a,d2`；打开衍生显示后含 `deriv:获得※资援<b>资援</b>结束`，且不含 `javascript:` |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../fool-plugin` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍安装两个金币包、`elo-sanction`、`sj-settings`、`shousha-full`、`douzhuan-xingyi` 与 `update-guard`，版本都是 `1.0.0`，内容哈希仍等于各自未改的 npack。决策不改内容锁。不安装如真似幻 | 失败的缺依赖安装不撤掉已安装包。升级和回滚笨蛋插件后更新保护和 liuli 金币仍是 `1.0.0` |
| 保存恢复 | 重开商店后仍是 `1.0.0`，来源仍是 `noname-liuli`，默认行不变。内容锁哈希与安装后相同 | 决策运行前后内容锁哈希不变。资产目录只有 `config.json` |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `fool_pack_json_enters_wasm_helper` 读仓库 `config.json`。默认行、对自己不投掷、Windows 不是手机、关闭提示和关闭计算都走同一字符串辅助，并以 `sleep:0 \| dom:0 \| audio:0` 结束 | 空来源以 `error:BadConfig` 开头。不调用 `wasm_bindgen` |
| 规则差异 | 资援、敏思和方统的点数门，烈弓类提示的文案分支，手机判断，同名配音与子技能配音，`Skill`/`SkillLog` 的描述改写，选将语音的路径和死循环，表情是否投出，与上游 `extension.js` 里仍会执行的路径一致。`parseSize` 不在本包。进讨的十周年文案保留前导空格 | 不画对话框、落点和样式，不播放音频，不启动每秒扫描，不采样 `Math.random`，不生成随机锚点。`if(false)` 里的选择卡牌优化不执行。注释里的 `_choose.mp3` 不执行。保存提示样式不调用 `game.reload`。衍生技名字按字面替换。不记为上游等价 |
