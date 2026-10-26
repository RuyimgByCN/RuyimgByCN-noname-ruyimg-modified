# B2 批③：UP37 规则/内容批（N01、R01、R03、R10、I01、EffectRequest 拆分）+ B4 shahai 拆分 + B5 mark-supplement 包数据 核对入库（2026-10-26）

- 执行时间：2026-10-26 **21:13 → 21:42 +08:00**（`HEAD = a8cd662d`，领取时实值；暂存树 `git write-tree` = `192fd6a1`，即提交后干净内容，验证在其导出上进行）；执行者 ZCode A 道
- 工作包：[台帐 B2](../../../architecture/native-remaining-ledger.md#当前阻断2026-10-26-扫描按优先级) 第三批，并关闭 **B4**（已提交测试依赖未跟踪 shahai 内容文件）与 **B5**（已提交测试依赖未提交包数据）。

**一句话结论**：731 个文件入库——UP37 旧线规则（N01 界朱然/界朱桓、R01 涤仇、R03 冯河/英霸含交牌/重铸/授标/手牌上限/maxHp>1/平定目标限制、R10 元数据）、I01 内容同步（legacy-source 50 文件、catalog 2582 将/6209 技、asset-manifest）、`EffectRequest` 功能拆分（rule-api 10,778 行 → 703 行 + 22 模块）、shahai 元数据按来源包拆成 9 文件（B4）、mark-supplement 包补 `fz_liegong/fz_xinkuanggu`（B5），及 8 个 10-24 证据目录。暂存树导出上：`cargo check --offline --workspace --all-targets` 通过，**engine-core 3836/0/1、engine-native 1559/0/71（B5 的 7 个失败全部消失）、game-server 358/0/0（含 N01 服务端测试）、content-compiler 541/0/0、rule-vm 15/0/0、rule-api 2/0/0**；前端 80/80、typecheck 通过；行数门禁 8612 文件（`effect.rs` 超限问题随拆分消失）、链接 0 坏链、ADR 索引通过。发现并登记一个脚本缺陷：`sync-original-content.mjs --verify-only` 的文件排序随区域设置变化（en-US 下报 `asset manifest entries mismatch`，zh_CN 下 exit 0），内容本身逐文件核对一致。

## 1 批内各项与核对（工作树内容 = 登记所述内容）

本批的登记来源：台帐 UP37 R01/R03/R10/N01/I01 行、规则基准 `native-rules-baseline.md` 的 2026-10-24/25 条目（随本批入库）、8 个 10-24 证据目录，以及 B4/B5 行。10-24 证据之后的 10-25 00:24–21:56 改动（R03 平定目标限制/maxHp>1/授标、N01 胆守 `ol_jie_danshou` 等）已在台帐 UP37 行以「已实现/已补/完成（24 测）」登记，并有对应测试文件；今天无新改动（19:23 的四个文件 mtime 是 B1 打补丁触碰）。

| 项 | 登记所述 | 工作树核对 | 入库文件 |
|---|---|---|---|
| EffectRequest 拆分（[effect-split](../2026-10-24-effect-split/README.md)、[integration-validation](../2026-10-24-effect-integration-validation/README.md)） | `effect.rs` 10778→700 行，22 个功能模块，815 变体/460 段逐字同；`plan_effect_request_split.py --check` 14 测；`preflight.py` 去掉行数豁免 4 测 | `effect.rs` **703** 行（10-25 新增变体）、`src/effect/*.rs` **22** 个；脚本单测 OK/OK；`check-source-lines.py` 8612 文件通过 | rule-api 34 项（含 7 个旧片段删除、`tests/`、Cargo.toml serde_json dev-dep 与 Cargo.lock 对应行）、scripts 5 项 |
| N01（[zhuhuan](../2026-10-24-up37-n01-zhuhuan/README.md)） | `ol_zhuhuan`+`ol_fenli` 三窗口、`ol_pingkou`；新增 `ol_jie_zhuran`/`ol_jie_danshou` 四选项，共 24 测；onlyOL 0.1.4/ABI 757 | `original_onlyol_n01.rs`、`original_onlyol_n01_danshou.rs`、`effect/primitives/ol_jie_danshou.rs`、`ported/onlyOL/{ol_fenli,ol_pingkou,ol_jie_danshou}.skill` + 重建 npack/characters.json；服务端 `native_onlyol_general.rs`（批②时因缺此 npack 失败，本批树中通过） | engine-core/content/onlyOL/game-server test |
| R01（[dichou](../2026-10-24-up37-r01-dichou/README.md)） | 旧线已修；覆盖提前归零/未选者清标记/至少一目标/空选拒绝/序列化恢复 | 对应 engine-core 文件在批内（`turn.removed_marks` 等见规则基准） | engine-core |
| R03（[pinghe-yingba](../2026-10-24-up37-r03-pinghe-yingba/README.md)、[recast](../2026-10-24-up37-r03-recast/README.md)） | 交牌/重铸 `ChooseGiveOrRecastOneHand`（ABI 756）、授标分支、手牌上限、`maxHp>1`、平定目标限制 | `up37_r03_recast_card_identity.rs`、`up37_r03_mark_modifier_validation.rs`、`original_shiji_pinghe_yingba.rs`；content-schema `max_hp_gt`；content-compiler `max_hp_gt_target_constraint`/`yingba_mark_target_restriction_modifier` 测试；`ported/shiji/{pinghe,yingba}.skill` + npack | engine-core/content-compiler/content-schema/rule-vm/content/shiji |
| R10（[metadata](../2026-10-24-up37-r10-metadata/README.md)） | 元数据 5 测 | `up37_r10_metadata.rs` | engine-core test |
| I01（[P1 §5](../2026-10-24-p1-trust-boundaries-sync/README.md#5-up37-i01内容同步到-1a548c95)） | legacy-source 49 文件变化/0 新增；画像 +4；目录 2576→2582 将、6190→6209 技；`--verify-only` exit 0 | legacy-source 变化 **50** 文件（全部 mtime 10-24 18h 同一次同步；比证据表多 1）；catalog **2582 / 6209** 与证据一致；画像不入 git（硬链自上游）；`--verify-only` 见 §3 | content/original（asset-manifest、catalog、skill-capability-matrix、legacy-source） |
| [upstream-37-impact](../2026-10-24-upstream-37-impact/README.md) | 静态审查快照 | 仅证据 | 证据目录 |
| **B4** shahai 拆分 | 已提交测试 `include_str!` 9 个 `shahai_meta_*.skill` | 旧 `shahai_meta.skill` **9125 行 / 1051 个 skill** 与 9 个拆分文件合计 **9125 行 / 1051 个 skill，技能名集合逐一相同**；README 注记按来源包拆分 | `ported/shahai/`（9 新 + impl/README 改 + 旧文件删除）、`extensions/shahai/README.md` |
| **B5** mark-supplement 包数据 | 已提交 l20/nc08 测试需要 `fz_liegong`/`fz_xinkuanggu` | `cards.json` 恰新增两条 `host:fz_*` 映射与描述「批 1-3」；npack `skill_index` +2（def 4/5）、`logic_hash`/`asset_hash`/`asset_manifest` 重算（装载器在测试中校验通过）；与 [L20 批 3 证据](../2026-10-17-l20-marks-batch3/README.md)（tag 535/536）一致 | 两文件 |
| 规则基准与 37 来源台账 | UP37 差异只写规则基准；N01/I01 补记 | `native-rules-baseline.md` +56/−3（2026-10-24 源码复核 + 10-25 条目）、`upstream-extension-reconstruction.md` +14/−6（锚点 + 10-24 补记） | 两文档 |

## 2 验证

### 2.1 真实工作树（共享 target）

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-core --no-fail-fast`（21:17:40–21:19:49） | **3836 passed / 0 failed / 1 ignored**，exit 0（[摘要](worktree-engine-core-summary.txt)） |
| `cargo test --offline -p rule-api -p rule-vm -p content-compiler -p content-schema --no-fail-fast` | 合计 **558 / 0 / 0**（content-compiler 540+1、rule-api 2、rule-vm 10+5），exit 0（[摘要](worktree-small-crates-summary.txt)） |
| `python3 -B -m unittest discover -s scripts -p 'test_plan_effect_request_split.py'` / `test_preflight.py` / `test_check_source_lines.py` | OK / OK / OK |
| `python3 -B scripts/check-source-lines.py` | 8612 文件 ≤ 2000 行（拆分前 `effect.rs` 10,778 行是既有唯一超限） |
| `pnpm --dir apps/web-client test` / `typecheck` | 80/0/0；vue-tsc 0 错 |

### 2.2 暂存树导出 `/tmp/b3-head`（= 提交后干净内容；独立 `CARGO_TARGET_DIR`；`/tmp/archive` 软链上游存档供 engine-native 读上游源码）

| 命令 | 结果 | 原始 |
|---|---|---|
| `cargo check --offline --workspace --all-targets --keep-going` | **exit 0，Finished** | [汇总](staged-tree-cargo-summary.txt) |
| `cargo test --offline -p rule-api` / `-p rule-vm` / `-p content-schema` / `-p content-compiler` | **2/0/0、15/0/0、0（无测试）、541/0/0** | [rule-api](staged-tree-rule-api.txt)、[rule-vm](staged-tree-rule-vm.txt)、[content-compiler](staged-tree-content-compiler.txt) |
| `cargo test --offline -p engine-native --no-fail-fast` | **1559 / 0 / 71**——B5 的 7 个失败（l20_liegong_batch3 2、nc08_upstream_liegong_xinkuanggu 4、nc08_skill_mapping_inventory 1）随包数据入库全部通过；与 17:10 扫描口径 1559/0/71 一致 | [逐二进制](staged-tree-engine-native.txt) |
| `cargo test --offline -p game-server --no-fail-fast` | **358 / 0 / 0**（含批②移出的 `native_onlyol_general` 1 项） | [逐二进制](staged-tree-game-server.txt) |
| `cargo test --offline -p engine-core --no-fail-fast` | **3836 / 0 / 1**——B4 两个 shahai 测试二进制随 9 个拆分文件入库可编译并通过 | [逐二进制](staged-tree-engine-core.txt) |

### 3 `sync-original-content.mjs --verify-only` 的区域设置依赖（登记为脚本缺陷）

在本机默认 `en-US`（Node 26.4.0 与 /usr/local/bin/node 20.17.0 均然）下 `--verify-only` 以 `asset manifest entries mismatch` 退出 1（[日志](sync-verify-en-US-fail.log)）。逐文件比对 2707 条画像条目的 path/size/sha256/source_path **全部一致**，差别只在数组顺序：`listFiles` 用 `localeCompare` 排序，`guanning.jpg` 与 `guānning.jpg` 在 en-US 下 `-1`、zh-CN 下 `1`；工作树 manifest 是 10-24 在 zh_CN 下生成，`LANG=zh_CN.UTF-8 LC_ALL=zh_CN.UTF-8 node scripts/sync-original-content.mjs --verify-only` **exit 0**（legacy-source 366 文件 14.9 MiB、portraits 2707 文件 387.6 MiB，与 I01 证据数字一致，[日志](sync-verify-zh-CN-pass.log)）。结论：内容正确；脚本应把排序改为区域无关（码点或 `Intl.Collator("und")`），否则验证结果取决于执行者的区域设置——已写入台帐 I01 行。

## 4 边界

- 本批入库的是已登记的旧线实现与内容；UP37 各项的「原生同名/AI 估值/旧 0.1.2 局恢复」等未验收部分状态不变（见台帐 UP37 段）。
- 不改规则/存档版本（v322 / v105），golden 未重录（UP37 工作各自的测试在上述全量中）。
- 剩余未提交：批④ 10-24 文档批（README/overview/evolution-plan/progress-history/self-check/ADR INDEX/10-07、10-18 证据注记 + architecture-doc-audit/native-remaining-ledger/native-arch-perf-review 证据）与 `data/`（native-content/server-content 运行数据）。
