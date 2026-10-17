# L13 手杀MVP 八项整包（2026-10-17）

核验日期：2026-10-17。来源：`noname-liuli` 仓 `noname/extension/手杀MVP/extension.js`
（上游提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`，包版本 `1.1.7`）。
**规则 v301 / 存档 v86 不变**；未改引擎语义、未重录 golden（本批为装载器 + 包数据面）。
本目录证明来源 **L13 手杀MVP** 八项整包；与已计入的 U03、U06、L06、L07、L23、U04、
L04、L05、L22、L02、L15、L17、L18、L26、L21、L16、U02、L03 一起，整包计数为
**19/37**。其余 18 个来源未验收。

## 交付

| 面 | 内容 |
|---|---|
| 装载器模块 | `crates/content-native-loader/src/shousha_mvp.rs`——MVP 计分（攻击/治疗/辅助/惩罚 + 基础分）、战绩统计（伤害/受伤/摸牌/出牌/杀敌）、结算分组（胜/负/平，`jiesuanfix` 两条口径）、排序与 MVP、状态导出/恢复、单行入口 `apply_mvp_line` |
| 安装包 | `packages/extensions/shousha-mvp-upstream-liuli`（`upstream-liuli/shousha-mvp@1.0.0`；资产仅 `assets/config.json`）。`logic_hash` 前四字节 `101,132,111,230`，`asset_hash` 前四字节 `187,162,192,101`，配置 `source_hash` 前四字节 `219,168,173,87` |
| 八项测试 | `crates/engine-native/tests/l13_shousha_mvp_eight.rs::l13_eight_dimension_from_fresh_install` |
| 目标平台 | `crates/engine-native-wasm/src/lib.rs::shousha_mvp_apply` + `pack_roundtrip::shousha_mvp_pack_json_enters_wasm_helper` |

包配置（`assets/config.json`）带上游默认值：`shoushajiesuan=true`（主开关）、
`jiesuanfix=true`（新版结算）、`yangshi=on`、`oneMore=true`、`qingyao_shoushapeiyin=true`、
`mvplutou=shousha`、作者/版本与结算面板列名表。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l13_shousha_mvp_eight
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native-wasm --lib
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
node apps/web-client/scripts/extension-store-smoke.mjs
```

结果：八项测试 **1 passed / 0 failed**（[targeted.log](targeted.log)）；engine-native 全量
**898 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)；1 ignored＝既有诊断探针）；
装载三包 **66/0/0**（[deps-tests.log](deps-tests.log)）；wasm **20/0**（[wasm-tests.log](wasm-tests.log)）；
服务端 **49/0/0**（[game-server-tests.log](game-server-tests.log)）；wasm32 目标 `cargo check
-p engine-native-wasm --target wasm32-unknown-unknown` **通过**；Gate 1b **exit 0**
（[gate1b-all-targets.log](gate1b-all-targets.log)）；客户端扩展存储冒烟 **8/8**
（[client-store-smoke.log](client-store-smoke.log)）。全部 exit 0。

## 八项

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 计分：伤害牌 +2 攻击、无懈 +2 辅助、非自目标多目标锦囊 +1 辅助；伤害来源按 `attitude<0` 或内奸 `num>5 ? +15 : +3*num`；获得/失去牌按张数记辅助；回复 `num>5 ? +10 : +2*num`；击杀敌方 +3 攻击、击杀友方 +5 惩罚（内奸杀非主公再 −5/+3）。样例事实（14 条）得 0 号 `14/0/7/0`、1 号 `0/4/0/0`、2 号 `0/0/1/0`、3 号（内奸）`18/10/0/5`。战绩：0 号 `伤害3/摸牌2/出牌2/杀敌1`，1 号 `受伤6/摸牌1/出牌1` | 重复 `seq` 不入账（分数不变，`dup` 计数 +1）；未知玩家事实 `Err(UnknownPlayer)`；来源阵亡的 `gain/recover/die` 不记（上游 `isIn()` 门）；`toself` 的多目标锦囊不计辅助 |
| 配置 | 真实 `config.json` 解析出上游默认：主开关开、`jiesuanfix` 开、`yangshi=on`、`oneMore`、配音开、`手杀MVP`/`1.1.7`/作者、列名表（攻击/治疗/辅助/惩罚/杀敌…）；主开关关闭时整块不登记（分数全 0、分组空） | 空 `source`、缺 `package_version`、缺 `settings`、`shoushajiesuan` 非布尔、`yangshi` 非字符串、列名表值非字符串一律拒绝（`BadField`/`Missing`/`Json`） |
| 依赖 | `manifest.dependencies` 为空；组合导入时同目录其余包照常安装 | 激活未安装版本 `9.9.9` → `NotFound`；包 id `../shousha-mvp` 逃逸拒绝；声明 `missing-base` 的 `1.0.1` 安装 → `Invalid(missing dependency)`；失败安装不撤掉已装包 |
| 组合 | 同一次 `import_extensions` 仍安装 `upstream-liuli/coin@1.0.0`、`upstream-liuli/elo-sanction@1.0.0`，两者 `content_hash` 等于各自未改 npack；不安装如真似幻 | 失败安装后激活集仍是手杀MVP 1.0.0 / 金币 1.0.0 / 段位制裁 1.0.0 |
| 保存恢复 | `export_state()` → `restore_state()` 分数与战绩逐项相同；恢复后重复投递不入账、新 `seq` 照常入账；`apply_mvp_line` 带 `state` 字段的续跑报告含 `dup:1` 且分数不翻倍 | `{}`（缺 `lastSeq`）与坏字段状态 → `BadState`；报错事实不消费序号（其后合法 `seq` 仍可入账） |
| 安装升级 | npack `2.0.0`（重算 logic/asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0`；重开商店仍是 `1.0.0` 且配置默认值不变 | `manifest.enabled=false` 的 `shousha-mvp-disabled` 被跳过；升级不改写金币/段位制裁版本 |
| 目标平台 | `shousha_mvp_apply` 读仓库 `config.json`，同一事实串得 handler 一致报告（`rank:1,3,0,2 | mvp:1 | win:3 fail:1,0,2`）；主开关关闭 → `enable:0` 且分组全空 | 未知事实 `kind` → `error:UnknownFact`；坏配置 JSON → `error:Json`；未知观察者 → `UnknownPlayer` |
| 规则差异 | 订阅权威事实形成统计、结果页数据可选；胜负由内核给出（本模块只分组展示）；事实按 `seq` 去重，恢复/录像不重复入账；`jiesuanfix` 两条口径都实现（失败分支：新版存活反贼胜、旧版存活内奸胜，均可复现） | 不替换内核终局；不画界面（资产只有 `config.json`，无图片/音频/`innerHTML`/`commandnode`）；不播音；不检测作弊输入（`effect5` 的 DOM 钩子未迁移）；不生成随机名单（`randomName` 未迁移）；不读 `get.attitude`（关系由调用方给出）；国战/平局分支按上游只在分组面成立 |

上游怪癖按原样保留并在测试中固化：`cunhuoPlayerGroups['nei']` 的计数写入键为折叠
`nei`、读取键为 `nei<id>` ⇒ 内奸存活计数恒 1（`nei_count > 1` 不可达）；
排序为「冒泡稳定升序 + 整体反转」，并列时按逆输入序（存活者在前、阵亡者在后）。

## 顺带修复（同批发现）

`apps/web-client/scripts/extension-store-smoke.mjs` 的 `catalog_files_15` 断言在
2026-10-14 之后已过期（转换包目录增至 29/30，脚本此前为 7/8 通过）。改为断言
**原始 15 个 G1 包全部在场**（`catalog_files_g1`），不再锁死精确数量；改后 8/8。
本批同时把 L23 测试中「商店里没有 `shousha-mvp` 包」的过期依赖反例收窄为只查
`shousha-ui`（L13 已成为独立转换包，预设应用仍不依赖它）。

## 影响

- 整包计数 **18/37 → 19/37**；`T12-SRC-L13` 关闭。台账、覆盖清单、进度主稿、
  登记表同步。
- 无规则/存档版本变化，无 golden 变化；扩展组合面新增一个数据型包（不进目录、
  不注册宿主行为，不影响标准局）。
