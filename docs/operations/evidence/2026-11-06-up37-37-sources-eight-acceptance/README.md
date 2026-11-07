# Batch 19：13 个剩余上游扩展来源八维整包验收证据（2026-11-06 · 37/37 收口）

- **核验日期**：2026-11-06
- **基线提交**：`600d32faa`（Batch 18）
- **版本常量（零变更）**：
  - `STANDARD_RULESET_VERSION = 333`（`crates/engine-native/src/content/standard.rs`）
  - `ARCHIVE_FORMAT_VERSION = 107`（`crates/engine-native-kernel/src/archive.rs`）
  - `RULE_VM_ABI_VERSION = 759`（`crates/rule-vm/src/lib.rs`）
  - `NETWORK_PROTOCOL_VERSION = 8`（`crates/protocol/src/lib.rs`）
  - `CONTENT_SCHEMA_VERSION = 5`（`crates/content-schema/src/lib.rs`）
- **14 Golden Hashes**：零改动
- **平台与客户端线索**：覆盖 `engine-native-wasm`（`wasm32-unknown-unknown` 编译门禁与 wasm 桥接）、`game-server` 安装矩阵（`nc08_install_matrix` 48 目录全量发现与导入）及真实客户端 `PackageStore` 激活/回滚闭环。

## 1. 本批收口的 13 个来源（24/37 → 37/37）

| 来源 ID | 名称 | 转换包目录（`packages/extensions/*`） | 装载器模块（`content-native-loader`） | 八维验收测试（`crates/engine-native/tests/`） | 配置验收（`nc08_config_acceptance.rs`） |
|---|---|---|---|---|---|
| `U01` | 3D精选 | `packages/extensions/d3-upstream-noname` | `content_native_loader::bridge` | `u01_d3_select_eight.rs` | 既有资产桥接 |
| `U05` | 杀海拾遗（9 子包） | `packages/extensions/shahai-yunchou-upstream-noname` 等 9 个目录 | `content_native_loader::bridge` | `u05_shahai_eight.rs` | 既有资产桥接 |
| `U07` | 玩点论杀 | `packages/extensions/wudian-upstream-noname` | `content_native_loader::bridge` | `u07_wudian_eight.rs` | 既有资产桥接 |
| `U08` | 英雄杀 | `packages/extensions/hero-kill-upstream-noname` | `content_native_loader::bridge` | `u08_hero_kill_eight.rs` | `hero_kill_config_matches_upstream` |
| `L08` | 十周年UI | `packages/extensions/decade-ui-upstream-liuli`（本批新增） | `content_native_loader::decade_ui`（本批新增） | `l08_decade_ui_eight.rs` | `config_acceptance_l08_decade_ui_package` |
| `L11` | 太虚幻境 | `packages/extensions/taixu-upstream-liuli`（本批新增） | `content_native_loader::taixu`（本批新增） | `l11_taixu_eight.rs` | `config_acceptance_l11_taixu_package` |
| `L12` | 如真似幻 | `packages/extensions/rzsh-upstream-liuli`（本批新增） | `content_native_loader::rzsh`（本批扩充） | `l12_rzsh_eight.rs` | `config_acceptance_l12_rzsh_package` |
| `L14` | 手杀ui | `packages/extensions/shousha-ui-upstream-liuli`（本批新增） | `content_native_loader::shousha_ui`（本批新增） | `l14_shousha_ui_eight.rs` | `config_acceptance_l14_shousha_ui_package` |
| `L19` | 末日乱斗 | `packages/extensions/doomsday-brawl-upstream-liuli` | `content_native_loader::doomsday` + `bridge` | `l19_doomsday_brawl_eight.rs` | `doomsday_brawl_config_matches_upstream` |
| `L20` | 标记补充 | `packages/extensions/mark-supplement-upstream-liuli` | `content_native_loader::bridge` | `l20_mark_supplement_eight.rs` | `mark_supplement_config_matches_upstream` |
| `L25` | 祖安武将 | `packages/extensions/zuan-upstream-liuli`（本批新增） | `content_native_loader::zuan`（本批新增） | `l25_zuan_eight.rs` | `config_acceptance_l25_zuan_package` |
| `L27` | 选将美化 | `packages/extensions/select-beautify-upstream-liuli` | `content_native_loader::bridge` | `l27_select_beautify_eight.rs` | `select_beautify_config_matches_upstream` |
| `L28` | 雷霆万钧 | `packages/extensions/thunder-upstream-liuli` | `content_native_loader::thunder` | `l28_thunder_eight.rs` | `thunder_config_matches_upstream` |

## 2. 八维验收覆盖摘要

1. **① 生命周期**：`discover_extensions` + `import_extensions` 真实商店安装、内容锁计算、`deactivate_slot` 停用与 `rollback` 回滚恢复。
2. **② 配置与包数据**：5 个新配置包（`L08, L11, L12, L14, L25`）携带与 `recount_g0.py` 一致的 G0 统计与默认配置；内容型扩展通过 `bridge_pack` / `bridge_generals` 校验卡牌、武将、宿主技能与 `deferred` 清单。
3. **③ 组合共存**：每个扩展均与同类或同仓其它扩展同店激活或同局 `Engine::new_with_extensions` 建局，互不覆盖定义 ID 或污染内容锁。
4. **④ 保存恢复**：内容包经 `Engine::save` / `Engine::restore_with_extensions` 验证状态哈希一致且篡改扩展集拒恢复；配置/会话包经 `snapshot_*` / `restore_*` 单行快照往返一致。
5. **⑤ 功能执行**：
   - `U01`：国战明置 `ddd刘冲`、精彀远射加伤与末焱判定阶段展示触发；
   - `U05`：9 子包全量桥接（209 张牌、149 名武将），`swd` 护符、`mtg` 龙颅山峰、`yunchou` 曹操戏才、`ow` 士兵76、`xianjian` 赵灵儿同局执行；
   - `U07`：14 张牌 + 9 名武将 + 19 项宿主技能，【望梅止渴】打出结算；
   - `U08`：反击族（`yxs_fanji`/`yxs_fanji2`）与门神标记族（`yxs_menshen`/`yxs_menshen2`/`yxs_menshen3`）触发结算；
   - `L08`：6 档 `DecadeStyle` 切换、通道映射与 14 个上游子模块白名单校验；
   - `L11`：`TaixuRunState` 战役起局、祝福购买、侍灵绑定与 5 节点通关推进；
   - `L12`：`roman(1..=6)`、11 条三国文案与 4 条匹配提示确定性轮询；
   - `L14`：5 个顶层视觉/台词特效技能与 87 个 UI 钩子技能分类判定（`rule_effect=0`）；
   - `L19`：`identity_chain`、`WujinMoshiConfig` 与 `zaibian` 出牌阶段摸牌加成；
   - `L20`：`fz_liegong` 烈弓直伤与 `fz_xinkuanggu` 狂骨距离 1 内伤害回血；
   - `L25`：联机房间准入校验（`check_room_gate online` 拒绝启用编辑器或非标恶搞变体）；
   - `L27`：`xiefang`、`new_zhixi`、`new_meibu`、`sbzhaxiang`、`jishe` 授予与 `jishe` 主动发动；
   - `L28`：12 对 `SAME_HERO`/`CHG_HERO` 同名替换计划与孙寒华妙剑/莲华新旧标记文本补丁。
6. **⑥ 平台与客户端边界**：通过 `content-runtime` 与 `engine-native-wasm` 校验跨平台纯数据/纯函数边界，无 `eval`、无越权本地文件写或网络请求。
7. **⑦ 异常拒装**：损坏 JSON、未知行为键、未知卡牌种、未知装备技、越界配置参数与非离线启用编辑器均返回结构化错误。
8. **⑧ 边界登记**：未迁移的 DOM/CSS/WebGL/Spine 渲染面、单机巨型战役脚本及已登记 `deferred` 条目均如实保留在包资产 `deferred` 列表与台账差异登记中。

## 3. 验证命令与实测结果

```bash
cargo test --offline -p content-native-loader
cargo test --offline -p engine-native --test misc _eight
cargo test --offline -p engine-native --test l_rounds _eight
cargo test --offline -p engine-native --test nc08 config_acceptance_
cargo test --offline -p game-server --test nc08_install_matrix
python3 -B scripts/map_package_sources.py --tsv docs/operations/evidence/2026-10-19-package-source-map/map.tsv --check
python3 -B scripts/report_source_coverage.py --check
python3 -B docs/operations/evidence/2026-10-17-t9-source-samples/gen_registry.py --fail-on-missing
python3 -B scripts/preflight.py
```

- `content-native-loader`：11 个测试目标共 44 项测试全部通过（0 失败）。
- `engine-native --test misc _eight`：8 passed; 0 failed。
- `engine-native --test l_rounds _eight`：27 passed; 0 failed。
- `engine-native --test nc08`：90 passed; 0 failed; 49 ignored（含新增 5 个配置包安装/启停/持久化用例、5 个上游配置对齐用例与 46 个扩展包同店共存用例）。
- `game-server --test nc08_install_matrix`：2 passed; 0 failed（48 个扩展与角色目录全量发现与导入通过）。
- 原生三 crate + `content-native-loader` 全量：`2,170 passed; 0 failed; 74 ignored`（三 crate `2,126/0/74`、loader `44/0/0`）。
- 独立 target Gate 1b `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` 与 `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm`：0 警告通过。
- `map_package_sources.py --check`：包 46；映射到来源 37；未映射 0；无包来源 0。
- `report_source_coverage.py --check`：合计 37 个来源：`{'converted': 12, 'installed-empty': 25}`（`no-package: 0`）。
- `gen_registry.py --date 2026-11-06 --fail-on-missing`：来源 37 | 标签 八项整包 37 | 登记表 resolved 37/open 0 | 引用缺失 0 | 证据缺失 0。
- `check-source-lines.py`（9,183 文件）、`check-native-layering.py`（0 越界）与 `preflight.py` 全量 21/21 门禁全部通过。
