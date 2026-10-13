
## 本批合并说明（2026-10-14 二轮收尾）

`crates/engine-native/tests/t12_g2g4_channels.rs` 为 **14 个非 U06 G1 转换包**的五断言批量验收（U06 本体已在 g1_huanle 验收）：

- **cards 桥 10 测**：hearth(17)/gwent(21)/yunchou(23)/mtg(10)/gujian(40)/zhenfa(8) 定数卡数 + swd/wuxing 不定数卡数 + ow/xianjian（cards.json 内含武将段，general_count+24/+36 与武将搜索）
- **character 桥 2 测**：d3（1 将）与 wudian（cards 桥——其 assets 仅 cards.json）
- **pile 桥 2 测**：cardpile 双仓（noname/liuli 各 113 张）

断言面与 U06 相同：资源（npack/pile/character 指纹 content_hash 与安装记录一致）+ 按名检索命中 + AI 隔离（Own/Hidden）+ 配置（store 激活集持久）。namespace 按 npack manifest 取 upstream-noname / upstream-liuli（测试初稿误用 official——由 `installed_root` find 失败暴露后修正）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | **14/14** 绿——[日志](channels-targeted.log) |
| engine-native 全量 | **703/0/0**（689→703，新增 14 测）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 结论

**15 个 G1 转换包（含双仓 cardpile）G1+G5+G2–G4 通道全部达成**。E06「其余来源」指两仓非转换来源（未转换的内容面，逐批立项）；E04/G2–G4 契约的通用通道面（模式/AI/配置持久/资源释放）已在 15 包上以真实扩展示例复核。
