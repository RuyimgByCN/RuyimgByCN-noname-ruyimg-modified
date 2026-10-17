# U01 刘冲末焱切片（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden，未抬升 `ARCHIVE_FORMAT_VERSION`。

本目录只证明 **U01 3D精选**里刘冲的末焱。真实包 `packages/extensions/d3-upstream-noname`（`upstream-noname/d3-select@1.0.0`）的角色表仍只有 `ddd刘冲`，技能为精彀 `host:max_range_weapon_damage` 与末焱 `host:d3_moyan`。上游 `character.js` 现为 **31** 将、**58** 条技能引用，`skill.js` 顶层定义为 **64**。其余 **30** 将与 **56** 条技能引用没有实现，也没有写进 `character.json`。整包计数保持 **5/37**（U03、U06、L06、L07、L23）。本目录不把 U01 计成第 6 个整包，也不把精彀或末焱单独当成来源完成。

## 代码

宿主行为 `HostBehavior::D3Moyan`，标签 **438**，键 `host:d3_moyan`。`HostBehavior::ALL` 长度为 435。触发主事件是 `PhaseBegin`，附加 `DamageBefore`。一轮内首次判定阶段开始时，持有者存活且牌库或弃牌堆非空，则从牌库顶（空则洗回弃牌堆）取出至多 3 张，记 `CardsRevealed`，挂在该宿主机器上。持有者选择一名存活角色（含自己，可取消）；该角色从展示牌中至少选一张（不可取消）。选中的牌推入其手牌并记 `CardsGiven`（`from` 为持有者）。取消或未选中的牌立即弃置。获得后把目标与张数记在 `vm_aux.flags`：`(owner, "d3_moyan_round")`、`(owner, "d3_moyan_target")`、`(owner, "d3_moyan_count")`。新一轮开始时先写下本轮标记并清掉旧的目标与张数，再询问。之后持有者受到未防止且点数大于 0 的伤害时，若记录的目标仍存活且其手牌数不大于获得张数，伤害 +1。

挂起期间这三张牌只在标签 438 的宿主机器上。`machine_held_cards` 只把 `D3Moyan` 的 `cards` 算进牌权，避免 `card_missing_from_all_zones`。其他宿主机器上的牌号仍在手牌里，不进这条臂，避免 `card_in_multiple_zones`。提示 `d3_moyan_target` 与 `d3_moyan_gain` 加在 `KNOWN_PROMPTS` 末尾，按字符串查找，不改存档版本。

`logic_hash` 前四字节为 `111,37,122,150`，`asset_hash` 前四字节为 `16,5,163,133`。测试比较商店 `content_hash` 与真实 npack 字节的哈希，不手写期望摘要。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test u01_d3_moyan -- --test-threads=1 u01_moyan_from_real_install
cargo test --offline -p engine-native --test g1_d3 -- --test-threads=1
cargo test --offline -p engine-native --lib host_behavior_keys_bijective -- --test-threads=1
cargo test --offline -p engine-native --test a0_tag_registry -- --test-threads=1 host_behavior_tags_unique
cargo test --offline -p engine-native --test t12_g2g4_channels -- --test-threads=1 g2g4_d3_select
```

结果：`u01_moyan_from_real_install` **1 passed, 0 failed**（[u01-moyan.log](u01-moyan.log)）。`g1_d3` **4 passed, 0 failed**（[g1-d3.log](g1-d3.log)：生命周期、内容锁、装载桥、精彀纵向）。`host_behavior_keys_bijective` **1 passed, 0 failed**（[host-keys.log](host-keys.log)）。`host_behavior_tags_unique` **1 passed, 0 failed**（[tag-registry.log](tag-registry.log)）。`g2g4_d3_select` **1 passed, 0 failed**（[g2g4-d3.log](g2g4-d3.log)，目录仍是基准加 1 将）。未跑 engine-native 全量。未跑七道门禁。

## 本切片对照

`u01_moyan_from_real_install` 从 `import_extensions` 安装真实目录，读商店副本 `packages/upstream-noname__d3-select/1.0.0/`，不是只读仓库里的 JSON。

| 项 | 已测到的正例 | 已测到的反例，以及仍未关闭的部分 |
|---|---|---|
| 功能 | 展示牌库顶 3 张；座位 1 获得其中 2 张，第 3 张进弃牌堆；其手牌数不大于 2 时，持有者受到 1 点伤害后体力少 2。只剩 1 张时持有者可以获得给自己，同样少 2 | 手牌数大于获得张数、获得者已死亡、同一轮下一判定、取消、未授予、牌库与弃牌堆皆空，都不增幅或不再询问。其余 30 将与 56 条技能引用没有行为。功能未完成，不计整包 |
| 配置 | 版本 `1.0.0`。商店 `content_hash` 等于真实 npack 字节。刘冲两技映射为 `host:max_range_weapon_damage` 与 `host:d3_moyan`，`g1_d3` 的未映射列表为空 | 未 `debug_grant` 时座位上没有末焱。包内没有牵招。空映射名单不能代替技能 |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound` |
| 组合 | 同一次导入还有 `huanle-cards`。建局后武将数为标准目录加 1。`g2g4_d3_select` 同样是加 1 | 不把其余 30 将写入角色表来凑目录长度 |
| 保存恢复 | 选人与选牌两段挂起都能 `restore_with_extensions` 后继续。展示牌在标签 438 的机器上，恢复时牌权完整 | 只统计末焱机器上的牌。把牌从牌库删掉而不放到其他区，恢复会拒绝 |
| 安装升级 | `import_extensions` 安装并激活真实包 | 本测试没有安装 `2.0.0` 再回滚。2026-10-08 生命周期不构成这次整包上调 |
| 目标平台 | 上述测试跑在 `engine-native` | wasm 与 Tauri 不跑这局（T13：浏览器调用方传 `packsJson`，Tauri 只做内容运行时）。本切片没有新增 wasm 辅助 |
| 规则差异 | 见下节。差异保持为差异 | 不记为与上游等价。`playpackconfig` 未逐项对照。立绘与语音未映射 |

## 登记差异

- 开局已经在判定阶段，且 `turn.round == 1` 时不发阶段开始。末焱不补发上游 `{ global: "roundStart" }` 的第 1 轮。测试用座位 2 的结束阶段加 `NextPhase`，发出下一轮判定阶段开始，这时才发动。
- 取消和未选中的牌立即弃置。上游留在处理区。
- 展示牌挂在宿主机器上，不进桌面或处理区。
- 获得是手牌推入加 `CardsGiven`。不发 `CardsMoved`、`CardsDrawn` 或 `CardsDiscarded`，以免带上理断、思继。不是上游的 `gain2`。
- 点数与随机流无关。
- 空牌库与空弃牌堆的负例把多余的牌挪到 `core.table`，只为让恢复时每张牌仍有归属。这不是末焱的结算区。
