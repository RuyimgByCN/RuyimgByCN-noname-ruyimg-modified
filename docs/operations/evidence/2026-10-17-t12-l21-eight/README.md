# L21 武皇争霸八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。未改 U04、L05、L22、L02、L15、L17、L18、L26 的 npack 字节。

本目录证明来源 **L21 武皇争霸**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04、L04、L05、L22、L02、L15、L17、L18、L26 一起，整包计数为 **15/37**。其余 22 个来源未验收。如真似幻没有安装，不计入。U01 末焱切片和 U02 仍未验收。G0 清单不单独构成这次上调。

## 代码

选池、座位、摸牌、战绩、阵亡替换和强敌过滤在 `crates/content-native-loader/src/martial.rs`。不挂标准局，也不新增宿主行为标签。`martial_qiangdi` 只作为标记参与胜负判断。空的武将、卡牌和技能索引不是空扩展。

安装包是 `packages/extensions/martial-upstream-liuli`（`upstream-liuli/martial@1.0.0`）。`logic_hash` 前四字节为 `155,117,1,49`，`asset_hash` 前四字节为 `101,213,160,144`，配置 `source_hash` 前四字节为 `130,223,231,5`。哈希由 `rehash_cards_asset` 与 `rehash_npack` 按文件重算，不是手改。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `martial_apply`。上游提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`。上游 `package.version` 是 `1.3`。出厂显示 5、上阵 5、隐藏 2、双将关闭、体力规则 `hejiansan`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l21_martial_eight -- --test-threads=1
cargo test --offline -p engine-native-wasm --lib pack_roundtrip::martial_pack_json_enters_wasm_helper -- --test-threads=1
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.09s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，15 filtered，0.00s）。两条命令退出码都是 0。宿主日志是第一次通过的运行，`Finished` 0.52s，二进制 `l21_martial_eight-9aecca7ef6f21a05`；这次只重新编译了 `engine-native`。wasm 日志是首次编译，`Finished` 1.73s，二进制 `engine_native_wasm-15ede17e3b73a404`。未跑 engine-native 全量。

## 八项

`l21_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__martial/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源 `noname-liuli`。出厂 5/5/2 得到宽 5、上阵 5、隐藏 4、高 4、牌位 20、比例 1.2。显示 2、上阵 6、隐藏 8 得到 6/6/12/24。显示 8、上阵 3 得到宽 8、上阵 3、隐藏 4。上阵数字 0 得到上阵 0、隐藏 4。先手 0 时主位 1、反位 0、侧从 3 开始，主拿敌方。先手 1 时主位 0、反位 1、侧从 1 开始，主拿友方。隐藏将吃可见将平均分后，默认序是 j,i,h,a,b,c,d,g,f,e。先手 0 的默认选将是友方 a,b,c,d,e、敌方 j,i,h,g,f。先手 1 是友方 a,b,c,e,f、敌方 j,i,h,d,g。等权 w,x,y,z 在先手 1 时友方 w,z、敌方 x,y。跳过选将时敌方 j,i,h,a,b、友方 c,d,g,f,e。敌方加权袋 `[5,3]` 再按 `[1,0]` 换位得到 q,r。后手摸 5，先手摸 4。同座态度 10，否则 -10。主公技始终为假。录像名是 `刘备/关羽` 与 `新1v1 - 先手`。先手胜一次再负一次、后手胜一次、非布尔不记账，写成 `新1v1：<br>先手：1胜 1负<br>后手：1胜 0负<br>`。强敌且存活且有金币为胜 +150；强敌但已死为胜 +0；无强敌且已死为负。己方阵亡且无替补时结算并尝试强敌，不替换。敌方阵亡、己方有副将、无强敌、无替补时只尝试强敌。替补还在时只替换。挫锐替补 a,b,c 换上 a、再摸 4；只剩 a 时摸 2；不是挫锐摸 4。双将且接受时第二将换成替补第一人，体力不变。已知武将里强敌索引 0 是 jin_simashi / re_huanggai，接受后摸 4 并在出牌阶段跳过。索引 7 是 jlsgsoul_guanyu，拒绝后结算且不获得技能。改选把剩余、友方、敌方接在一起。自由选将在选择中换成新名字 | 先手 2 拒绝。上阵宽度大于敌方名单拒绝。空可见序保持原顺序。双方都隐藏时不重排。空已知表不提供强敌，直接结算。无替补时不提供加入。已有副将或不双将时不提供加入。拒绝加入不写副将。改选或重选关闭时不可用。自由选将关闭时不可用；开启但还没在选时不替换。`shen_guojia` 两项没有 name2，过滤时丢掉。同目录 `mightyFoe.js` 以 `window.mightyFoe` 开头，但不执行 |
| 配置 | 商店 `content_hash` 等于本包 npack 字节的哈希，版本 `1.0.0`，依赖为空。武将、卡牌、技能索引都为空。文本含上游路径 `noname/extension/武皇争霸/extension.js`、提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade` 和 `upstream_version` `1.3`。显示、上阵、隐藏为 5/5/2，双将、自由选将和改选为假，体力规则是 `hejiansan`。`show_number` 数字 0 读成 5，字符串 `"0"` 读成 0，`battle_number` 数字 0 读成 0。默认行以 `registered:1` 开头，`connect:0`，`mode:normal`，`scriptRan:0` | 空 `source` 拒绝。`manifest.enabled=false` 的 `martial-disabled` 被跳过。包文本不含 `eval(`、`game.reload`、`fetch(`。双将关闭时 `{"join":true}` 仍是 `join:-`。布局不是宽 5、隐藏 4 的行拒绝 |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../martial` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍安装两个金币包、`elo-sanction`、`sj-settings`、`shousha-full`、`douzhuan-xingyi`、`update-guard` 与 `fool-plugin`，版本都是 `1.0.0`，内容哈希仍等于各自未改的 npack。决策不改内容锁。不安装如真似幻 | 失败的缺依赖安装不撤掉已安装包。升级和回滚武皇争霸后笨蛋插件与 liuli 金币仍是 `1.0.0` |
| 保存恢复 | 重开商店后仍是 `1.0.0`，来源仍是 `noname-liuli`，默认行不变。内容锁哈希与安装后相同 | 决策运行前后内容锁哈希不变。资产目录只有 `config.json` |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `martial_pack_json_enters_wasm_helper` 读仓库 `config.json`。默认行、先手 1 的座位和行列、双将加入后的 `openZhu:j/i` 与 `hpUi:1` 都走同一字符串辅助，并以 `sampled:0 \| dom:0 \| sleep:0` 约束 | 关闭启用时以 `registered:0` 开头且含 `scriptRan:0`。空来源以 `error:BadConfig` 开头。不调用 `wasm_bindgen` |
| 规则差异 | 布局数字、座位、隐藏将平均分（可见与“不可见”两段都按未隐藏过滤）、选将循环、跳过选将、敌方加权、摸牌、战绩、胜负金币、阵亡替换摸牌、加入第二将的门、强敌过滤，与上游 `extension.js` 里仍会执行的路径一致。模式被写成 `normal`。`connect` 在注册后写成 false。加载地址是 `extension/武皇/mightyFoe.js` | 不画界面，不采样 `Math.random`，不播放录像，不走联机，不 `eval`，不睡眠，不写盘。双将体力上限的五种算法在注释中，不执行。旁路 `mightyFoe.js` 不自动执行。`chooseCharacterOL` 不在仍会执行的路径上。不记为上游等价 |
