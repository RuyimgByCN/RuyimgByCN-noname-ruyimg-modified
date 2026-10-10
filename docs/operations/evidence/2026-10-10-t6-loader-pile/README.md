# 2026-10-10 T6①：装载器补 bridge_pile 入口，g1_cardpile 改经装载器

任务来源：[进度主稿 T6①](../../../architecture/native-progress.md)；开工 HEAD `9705fa2c`。

## 改动内容

- `content-native-loader/src/lib.rs`：
  - 新增 `pub fn bridge_pile(json: &str) -> Result<PileSpec, BridgeError>`——已安装 cardpile 包 `assets/pile.json` 的唯一生产解析入口。严格错误检查与 bridge_pack 同风格（MissingField/InvalidField 逐字段路径）；`counts` 键经 `BTreeMap` 排序后转 `Vec<(String, u32)>`，保证生成卡牌序（目录编号、发牌序的一部分）与解析器实现无关、仅由数据决定（与测试原 serde `BTreeMap` 语义一致）；零张数在装载器层即拒（`1..=u32::MAX`），未知牌名/花色仍由 `extension_pack_from_pile` 构建期校验。
  - Display 三处硬编码 "cards.json" 改为"包数据"（无测试断言 Display 文案，断言均为变体+字段路径）。
  - crate 头部文档补 pile 管线一句。
- `engine-native/tests/g1_cardpile.rs`：删除测试内 `PileEntryJson`/`PileJson`/`parse_spec`（约 30 行），3 处调用点改 `content_native_loader::bridge_pile`；`pile_bridge_rejects_invalid_spec` 保留直构 `PileSpec`（引擎侧 API 校验测试）。
- `content-native-loader/tests/loader_acceptance.rs`：新增 `bridge_pile_parses_installed_cardpile_asset`（真实已安装资产：source/total=160/12 条目/counts 字典序/sha 30 张）与 `bridge_pile_rejects_bad_data`（坏 JSON/缺 source/缺 total/缺 counts/零张数 5 负例）。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p content-native-loader --test loader_acceptance` | 3/3（含新增 2 例） |
| `cargo test --offline -p engine-native --test g1_cardpile` | 5/5（113 张包数据不变，双仓生命周期经装载器通过） |
| `cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast` | **65/0/0**（10-10 基线 63+2 个新增验收例） |
| `cargo test --workspace --no-fail-fast`（含本批） | **5098/0/2**，exit 0——[日志](../2026-10-10-t5-workspace-verification/cargo-test-workspace.log) |

包数据未动（`packages/extensions/**` 无改动）。
