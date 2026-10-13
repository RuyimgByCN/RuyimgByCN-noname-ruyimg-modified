# T12 G5 接力·第三批 + d3 武将装载面（2026-10-14）

## 任务来源

[第二批](../2026-10-14-t12-g5-batch2/README.md) 登记的剩余面：cardpile×2/shahai-gujian/wudian/zhenfa 的 G5 接力 + d3 武将装载面（bridge_generals）。

## 交付（crates/engine-native/tests/t12_g5_relays.rs 追加 5 测）

- **gujian/wudian/zhenfa**（cards.json 桥）：`g5_relay` 直接入列（第二批助手零改动）。
- **cardpile（upstream-noname）**：pile 桥——`bridge_pile(pile.json)`→`extension_pack_from_pile` 产出 113 张比例包后走同款接力（双 namespace 的 liuli 仓 lifecycle 已有专项，本批以 noname 仓为接力样本）。
- **d3（character.json 桥，武将装载面）**：`bridge_pack`+`bridge_generals` 双产物装配；父进程直测 + 独立 worker 测试（`T12_G5_ARCHIVE/REPLY/STORE` env）跨进程接力。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | t12_g5_relays **13/13**（8 第二批+5 本批）——[日志](relays-targeted.log) |
| engine-native 全量 | **687/0/0**（682→687）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 坑位

- d3 `bridge_pack` 返回 `BridgedPack`（取 `.pack`）；`bridge_generals` 同时调用以覆盖武将装载面（unmapped 登记项 dd dmoyan 照旧）。
- d3 worker 的 store 目录取 `T12_G5_STORE` 的父目录（父进程 tempdir 布局）。

## G5 面覆盖进度

**cardpile(noname)/gujian/wudian/zhenfa/d3 = 5 包达成**；累计 **14 包 G5**（9+5）。余：cardpile(liuli 仓——lifecycle 已有专项，接力面随双 namespace 切片)、E06 其余来源（ow/xianjian 武将包 G1 面已全、G5 已在第二批）。
