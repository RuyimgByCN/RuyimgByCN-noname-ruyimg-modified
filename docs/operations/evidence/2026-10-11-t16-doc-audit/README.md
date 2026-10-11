# 2026-10-11 T16 子项①②：演进提案过时陈述与规则基准非历史段更正

任务来源：[进度主稿 T16](../../../architecture/native-progress.md)（领取范围：**子项①②**——`architecture-evolution-plan.md` 与规则基准非历史段落；③④⑤⑥ 仍列待做）。开工 HEAD `45ecee28`（领取锁）。

## ① architecture-evolution-plan.md（8 处更正）

| 位置 | 原陈述（过时） | 更正 |
|---|---|---|
| S6 落地面 | 原生标准目录 25 非样例将+3 样例 | **34 非样例+3 样例**（原生新增曹仁+10-07 五批八将；实测 `STANDARD_GENERALS` 37 条目） |
| S13 落地面 | 目录 80 张；"当前未修缺陷"两项（`num_to_card_kind` panic、载荷不读尽） | 目录 **124 张**（v74 声明表核定）；两项已于 **10-07 加固**（回归 `archive_hardening_rejects_unknown_kind_and_trailing_bytes`，`boundary_snapshot.rs:2314`） |
| 总体判断段 | "其余测试桥接和正式宿主接线未完成" | 八份测试桥接已全部迁移（10-07）；正式宿主接线仍开放（T6③） |
| 现状段 | "两份 G1 测试桥接已迁移，其余适配未完成" | 同上更正 |
| A0 落地段 | "存档牌种编码表仍私有，且解码遇未知编号 panic（P0 待做）" | 解码遇未知编号已改为拒绝（10-07） |
| A0 落地段尾 | "牌种编码表往返与 enc_winner 不对称随 P0 处理" | 已随 P0 于 10-07 处理 |
| A5 段 | "当前先处理存档未知牌种与载荷读尽" | 已于 10-07 加固 |
| C0 落地段 | "其余 6 份 bridge 迁移仍待做，炉石需先改规范键" | 已于 10-07 全部迁移 |

历史表行（2026-09-24/10-07 核对行）保持原样（带日期语境不改写）。

## ② 规则基准非历史段（"本地上游对齐的新增核查"表 2 处更正）

- "`standard_catalog()` 现为 80 张自造测试牌" → **124 张**（原 80 为 v60 时点计数；v74 起声明表核定+守护）。
- "五谷、桃园、无中、借刀等牌种仍缺，且声明表未逐张对照上游牌表" → **四牌种已于 10-07 缺牌种批①补齐**（v62/v63，落位同上游），逐张对照见同页[上游标准牌表对照]段（实测 `standard.rs` 批①-2 落位代码）。

页首/v61 起版本条目（历史记录）未改。

## 验证

| 项 | 结果 |
|---|---|
| 事实核对 | `STANDARD_GENERALS` 37 条目实测（34+3）；124 张由 `standard_catalog_inventory` 守护；加固回归 `boundary_snapshot.rs:2314` 实存；四牌种落位 `standard.rs:4284-4300` 实测 |
| 本地锚点 | 更正所引 `#本地上游对齐的新增核查`/`#标准目录构成声明表`/`#上游标准牌表对照` 均在基准页实存（grep 6 命中） |
| `cargo test --offline -p engine-native --test replay_scenarios` | **2/2**（覆盖清单联动守护，基准改动后通过） |

## 边界

- 子项③（覆盖清单其余行）、④（18 份旧线阶段文档横幅）、⑤（rule-semantics/protocol 等状态性陈述）、⑥（legacy-reference 补整理节）仍列待做。
- 本子项零代码改动；演进提案历史表行未改写。

---

# 同日 T16③：覆盖清单其余行逐行审计（除 kof_session 归 T14）

## 方法

- 69 个机制行全部过一遍：被引测试名机械化存在性核查（`rg "fn <name>"` 于测试与源码树）、状态/边界注记与近期批次（v193–v197）交叉核对、`replay:` 场景引用由 `scenarios_are_registered_in_coverage_manifest` 守护（2/2 通过）。

## 发现（1 项重大 + 3 处计数过时，均已更正/登记）

1. **重大——5 个 lib 单元测试在 10-10 事故恢复中丢失**：`card_kind_keys_bijective`、`host_behavior_keys_bijective`、`card_kind_companion_skills_consistent`、`card_kind_encoding_roundtrip_all_variants`、`standard_skill_ids_append_only`。10-07 维护批日志显示它们随 `unittests src/lib.rs` 运行（5 passed）；现 `src/lib.rs` 仅 37 行无测试模块（基线 72583b2b 即如此）——恢复重建时 `host_behavior_tags_unique` 重建进了 a0_tag_registry，其余 5 个未重建。后果：数据键双射/伴随一致/编码往返/技能序只追加自 v60 后的新键（CardKind 86→133、HostBehavior 74→226）从未被这些断言验证。已登记 **T17（P1）** 重建。
2. **tag_registry 行三处计数过时**：HostBehavior 74→**226** tag、CardKind 86→**133**、MachineSnapshot 42→**59 变体**（57 结构+2 单元）——均已更正。
3. **tag_registry 行状态**："已验证"→"已验证（部分：5 个 lib 单元测试待重建，见主稿 T17）"，测试清单逐个标注在套件/丢失。

## 其余 68 行结论

- 被引测试名全部实存（105/105）且在 535/0/1 全量中通过；"部分/所测路径"限定词与行内边界描述一致。
- `catalog_inventory` 行（124 张）与 v74 后事实一致；`byte_archive` 行已含 10-07 加固记录；`kof_session` 行为 T14 更正后状态。
- 新机关牌（机关桶/凤/怨/妖术）经既有机制键（aoe_queue/card_use/nullify_window/aoe_full_resolution）覆盖，未新增独立机制键——无缺行。

## 验证

- `cargo test --offline -p engine-native --test replay_scenarios`：**2/2**（清单联动守护，更正后通过）。
- 零代码改动（纯文档 + 主稿/清单登记）。

---

# 同日 T16④：旧线阶段文档横幅与死链

## 横幅补注（19 份）

`docs/operations/phase{10–23,26,27,29,30}*.md`（含双 19；24/25/28 基线前已删除）全部在标题后补统一横幅：**历史阶段记录（T16④ 补注，2026-10-11）**，链接指向[进度主稿](../../architecture/native-progress.md)、[覆盖清单](../../native-coverage-manifest.md)、[规则基准](../../architecture/native-rules-baseline.md)。逐份链接核查通过（脚本断言三链接在每份前 800 字符内）。

## 附带发现：legacy-reference 两处死链

`phase25-game-content.md`、`phase28-mobile-shell.md` 在任何提交中都不存在（基线前已删除），legacy-reference 表仍按存在引用。已标注"该文件已删除（10-10 文档清理）"；phase28 的 ADR 0702 指引改直链实存的 [0702-m7-mobile-blocking](../../../adr/0702-m7-mobile-blocking.md)（该 ADR 未入 INDEX——登记为待办碎片，随 T16⑤ 一并核对 ADR 索引完整性）。

## 验证

- 零代码改动；纯文档。

---

# 同日 T16⑤：rule-semantics 等状态性陈述核对（结论：全部不动）

| 目标 | 状态性陈述扫描 | 结论 |
|---|---|---|
| `docs/rule-semantics/`（phase2/3/5、vocabulary） | 窄/宽两轮措辞扫描 0+1 命中 | vocabulary.md:27「已实现摸牌、移动…等核心路径」为仍为真的历史语境（"等"涵盖后续增长），不动 |
| `docs/protocol/phase7-multiplayer.md` | 0 命中 | 不动 |
| `docs/extensions/author-guide.md` | 0 命中 | 不动 |
| `docs/architecture/phase6-web-client.md` | 已有 2026-09-16 历史横幅（T16 范围外先前已标注） | 不动 |
| `apps/desktop-client/README.md` | 0 命中 | 不动 |
| `fuzz/README.md` | 0 命中（targets 清单与 fuzz_targets/ 实存一致） | 不动 |
| 根 `README.md` | 3 处"当前"均为指向进度主稿的正确导航 | 不动 |
| cross-target | 无独立文档（`scripts/cross-target-hash.sh` 自述） | 不适用 |

零改动。附：ADR 索引缺 0702 条目（文件在）——登记为索引完整性碎片，随 ADR 索引下次维护补（T16④ 段已注明）。
