# L07 全能搜索八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。

本目录证明来源 **L07 全能搜索**。写入当时与已计入的 U06、L06 一起为 **3/37**。其后的总账以进度主稿为准。G0 清单和 G2 通道行不单独构成这次上调。

## 代码

公开索引在 `crates/content-native-loader/src/omnisearch.rs`。它只接收标准目录和 cards.json 的牌名、译名、技能映射，不接收 `Engine`。同一 `name` 的多张实体牌合并为一条，标识不用入局后的 `前缀_序号`。查询等于调用方给出的暗牌实例号时返回 `HiddenCard`。`debug_execute` 对任何脚本返回 `EvalRefused`。

安装包是 `packages/extensions/omnisearch-upstream-liuli`（`upstream-liuli/omnisearch@1.0.0`）。配置只保存上游允许的样式名和背景名，不加载皮肤、图片或语音。`logic_hash` 前四字节为 `43,139,242,212`，`asset_hash` 前四字节为 `245,83,142,131`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l07_omnisearch -- --test-threads=1 l07_eight_dimension_from_fresh_install
cargo test --offline -p engine-native-wasm --lib -- --test-threads=1 omnisearch_pack_json_enters_wasm_helper
cargo test --offline -p engine-native --test g1_huanle -- --test-threads=1 u06_eight_dimension_from_fresh_install
cargo test --offline -p engine-native --test g1_wuxing -- --test-threads=1 l06_eight_dimension_from_fresh_install
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)）。加入该包后重跑 U06、L06 八项，各 **1 passed, 0 failed**。未跑 engine-native 全量。

## 八项

`l07_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。索引读取的是商店副本 `packages/{namespace}__{id}/{version}/`，不是只读仓库路径。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 检索「曹操」「奸雄」「杀」「苦无」「五行盘」；苦无详情含 `huanle_kuwu=host:sha_damage_force_discard`；手里剑 `copies=2` | 空查询为 `EmptyQuery`；不存在的名字为空；`debug_execute` 为 `EvalRefused` |
| 配置 | 商店副本 `config.json` 解析为 `vs2015.min.css` / `none`，激活 `1.0.0` | 未知样式、未知背景拒绝；`manifest.enabled=false` 不安装 |
| 依赖 | npack `dependencies` 为空，`content_hash` 等于真实 npack | 激活 `9.9.9` 为 `NotFound`；包 id `../omnisearch` 为 `Package`；缺依赖的 `1.0.1` 为 `Invalid` |
| 组合 | 欢乐「苦无」与另装的 `kuwu-alias` 同名两条，按包 id 区分；五行盘来自 `wuxing` | `drop_package("huanle-cards")` 后苦无只剩别名包；丢掉 `wuxing` 后五行盘离开索引 |
| 保存恢复 | 重开同一商店后「曹操」查询与配置枚举不变 | 1 号座位视图里 0 号手牌为 `Hidden`；用这些实例号查询得到 `HiddenCard`，公开结果不含这些 id |
| 安装升级 | npack `2.0.0` 激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖激活不改写激活集 |
| 目标平台 | 宿主见上；`omnisearch_pack_json_enters_wasm_helper` 读仓库欢乐包，命中 `hit:huanle-cards:card:kuwu:苦无`，且不含 `hnl_` | 查询 `0` 且暗牌列表含 `0` 时为 `error:hidden`；空查询为 `error:empty`；脚本为 `eval_refused` |
| 规则差异 | 匹配是译名子串或标识后缀，未译的纯 ASCII 名不进索引 | 对话框、分页、高亮皮肤、背景图和语音播放未迁入。调试入口不执行脚本，也不返回技能源码。不记为上游等价 |

索引条目的 id 不含 `hnl_` 或 `wxl_`。标准牌种只索引有固定公开译名的基础牌，未列入译名表的专名武器不进索引。
