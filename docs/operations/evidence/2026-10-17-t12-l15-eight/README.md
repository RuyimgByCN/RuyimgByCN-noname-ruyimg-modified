# L15 手杀全扩八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。未改 U04、L05、L22、L02 的 npack 字节。

本目录证明来源 **L15 手杀全扩**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04、L04、L05、L22、L02 一起，整包计数为 **11/37**。其余 26 个来源未验收。如真似幻没有安装，不计入。U01 末焱切片和 U02 仍未验收。G0 清单不单独构成这次上调。

## 代码

改名、将包筛选、禁将和技能说明句号在 `crates/content-native-loader/src/shousha.rs`。武将、将包和技能说明由调用方给出。不挂标准局，也不新增宿主行为标签。空的武将、卡牌和技能索引不是空扩展：表长为 511、11、167、373。

安装包是 `packages/extensions/shousha-full-upstream-liuli`（`upstream-liuli/shousha-full@1.0.0`）。`logic_hash` 前四字节为 `42,40,35,233`，`asset_hash` 前四字节为 `221,126,130,68`，配置 `source_hash` 前四字节为 `56,25,86,102`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `shousha_apply`。上游提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`。出厂配置没有 `changeGroup`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l15_shousha_eight -- --test-threads=8
cargo test --offline -p engine-native-wasm --lib shousha_pack_json_enters_wasm_helper -- --test-threads=8
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.10s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，11 filtered，0.02s）。两条命令退出码都是 0。未跑 engine-native 全量。

## 八项

`l15_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__shousha-full/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源 `noname-liuli`。默认链把 `[diy,standard,standard,huicui,tw]` 收成 `[standard,huicui]`。`liubei` 保持 `刘备` 且无禁将标记。`手杀刘备` 改成 `界刘备`。`新杀赵云` 改成 `赵云`。`OL刘备` 在 shousha 下不变。`outsider` 与身份模式的 `sunwukong` 获得 `forbidai`。`txhj_wolf` 获得 `forbidai+unseen`。没有 `re_sunben` 武将时译名是 `界孙策`。`摸一张牌` 补 `。`，已有 `。` 或 `）` 不补，`_map` 与纯空白不补，带空格的原文在末尾补 `。`。列表外的 `shiji` 在没有 `refresh` 时被还原筛掉，有 `refresh` 时也不把允许表里多出来的包追加进去 | 斗地主不给 `sunwukong` 打标记。太虚幻境不给 `txhj_` 打 `unseen`。空的 unseen 变成标记 `unseen`，`"0"` 仍是标记 `0`。`all` 把 `OL刘备` 改成 `刘备`。关闭改名时保留 `手杀刘备`，空名可通过。改名开启时空名返回 `BadName`。带上 `re_sunben` 且名为 `手杀孙策` 时，直接写入的 `界孙策` 被改成 `孙策`。还原与引荐同时打开时 `forbidai` 只出现一次。`changeGroup` 写入还原表里的 `刘备`，不改显示名。性别只认 `male` 和 `female`。禁将名单是 `shen_diaochan,ol_xiaoqiao,old_maliang,outsider`；`fanchou`、kept_* 和 id 含 `boss_` 的不进名单。空数组成员算包存在。BT 名单是 `shen_diaochan,kept_refresh,fanchou`，`shen_zhugeliang` 仍允许。打开奶龙后音频和循环仍是 0 |
| 配置 | 商店 `content_hash` 等于本包 npack 字节的哈希，版本 `1.0.0`，依赖为空。武将、卡牌、技能索引都为空。文本含上游路径 `noname/extension/手杀全扩/extension.js` 和提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`。表长 511、11、167、373。`base/liubei` 为 `刘备`，`base/guanyu` 为 `关羽`，`doudizhu/sunwukong` 为 `孙悟空`，`huanyuan/guanyu` 与 `yinjian/guanyu` 没有条目。禁将包 12 个，以 `standard` 开头，含 `huicui`，不含 `tw` | 空 `source` 拒绝。改掉 base 里的 `liubei` 拒绝。缺 `mobile` 为 `MissingPack`。BT 缺 `extra` 为 `MissingPack`。`manifest.enabled=false` 的 `shousha-full-disabled` 被跳过 |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../shousha-full` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍安装 `upstream-liuli/coin@1.0.0`、`upstream-noname/coin@1.0.0`、`upstream-liuli/elo-sanction@1.0.0` 与 `upstream-liuli/sj-settings@1.0.0`，内容哈希仍等于各自未改的 npack。公式不改内容锁，也不改配置文件。不安装如真似幻 | 失败的缺依赖安装不撤掉已安装包。升级和回滚手杀全扩后两个金币包、段位制裁和 SJ Settings 仍是 `1.0.0` |
| 保存恢复 | 重开商店后仍是 `1.0.0`，来源仍是 `noname-liuli`，`base/liubei` 仍是 `刘备`，表长仍是 511。稳定 id 在改名后不变 | 公式运行前后商店里的 `config.json` 字节不变 |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集。资产目录只有 `config.json` |
| 目标平台 | `shousha_pack_json_enters_wasm_helper` 读仓库 `config.json`。默认链与宿主测试相同。斗地主、太虚、all、还原将包、禁将和 BT 都走同一字符串辅助。奶龙打开后仍以 `audio:0 \| loop:0` 结束 | 空来源以 `error:BadConfig` 开头。缺包以 `error:MissingPack` 开头。空名以 `error:BadName` 开头。不调用 `wasm_bindgen` |
| 规则差异 | 改名表、将包筛选和禁将名单与上游 `extension.js` 的读取一致。`boss_` 出现在 id 任意位置都留在可用集。始计只跟着界限突破，外服只跟着 sp。不在原列表里的包不会因为允许表而出现 | 不播放奶龙音频，不循环，不按座位重排玩家，不调用 `clsPrefixName`，不写 HTML 配置名和 CSS。出厂配置没有 `changeGroup`，分支仍可测。缺译名返回 `BadName`，不是抛出的 JS 异常。BT 追加表的数组空洞不记为 id，清洗后 78 条。不记为上游等价 |
