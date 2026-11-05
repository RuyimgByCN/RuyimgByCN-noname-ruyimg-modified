# UP37-I04／I01／N01／R10 兼容与内容锁矩阵、分包审计及双线 AI 估值／RNG 隔离收口证据（2026-11-05）

## 1. 背景与收口范围

- **基线版本**：`da9d222de`（规则 `333`／存档 `NGSA 107`／content schema `5`／`RULE_VM_ABI_VERSION = 759`／`WS8`），本批仅新增与扩充确定性测试与审计门禁，**不修改生产规则语义或分配任何新版本号**。
- **覆盖台账项**：
  1. **`UP37-I04`（兼容／旧锁：旧 0.1.x／0.2.x 存档拒绝与恢复矩阵）**：
     - `shiji` 三版本包链（`0.1.0` ABI 502 $\to$ `0.2.4` ABI 756 $\to$ `0.2.5` ABI 756）与 `onlyOL` 四版本包链（`0.1.0` ABI 688 $\to$ `0.1.2` ABI 754 $\to$ `0.1.4` ABI 757 $\to$ `0.1.5` ABI 758）：
       - `PackageStore::install_with_assets` 升级、回退与精确 `ContentLock` 冻结副本保持（`load_locked_packages` 不因新版本安装而静默替换旧锁副本）；
       - 含歧义 `effect 656` 的历史未冻结包（`onlyOL-0.1.2`、`onlyOL-0.1.4`）在 `ContentPackage::validate`、`PackageStore::install_with_assets`、`content_native_loader::bridge_vm_skill_decls`、`validate_and_freeze_raw_package` 四个入口全量受控拒绝；
       - 旧线 `Engine::from_content_lock` 对局快照与精确锁恢复（`restore_with_manifest`、`restore_snapshot_with_manifest`）100% 还原状态哈希；全部跨版本锁组合（`shiji` 6 组交叉对、`onlyOL` `0.1.0` $\leftrightarrow$ `0.1.5` 交叉对）均以 `SnapshotError::ManifestMismatch`／`ManifestRejectionReason::SnapshotInconsistent` 拒绝；
       - 原生 `VerifiedLockedContent` 锁内容工厂（`onlyOL` `0.1.0` vs `0.1.5`、`shiji` `0.2.4` vs `0.2.5`）精确锁恢复保持 `state_hash`、`core`、`resolution` 恒等并按 `N11-a` 旋转 `auth_generation + 1`，跨版本锁恢复与无锁标准目录恢复均以 `SnapshotError::RuleVersionMismatch` 拒绝；
       - 原生历史 `.ngsa` 字节档全矩阵：格式版本 `105`（`native-v322..=v326.ngsa`、`c331_phase7_genuine_105.ngsa`、`c331-real330-pending.ngsa`）以 `ArchiveError::UnsupportedFormatVersion(105)` 拒绝；格式版本 `106`（`detached-106.ngsa`、`gaiming-window-331.ngsa`）以 `ArchiveError::UnsupportedFormatVersion(106)` 拒绝；格式版本 `107` 下规则版本头尾不一致以 `ArchiveError::RuleVersionMismatch` 拒绝，旧规则版本（`322..=326, 330..=332`）以 `SnapshotError::RuleVersionMismatch { snapshot, catalog: 333 }` 拒绝。
  2. **`UP37-I01`（内容／目录同步：分包差异审计、旧局内容锁反例）**：
     - 全量 31 个 `content/original/ported/*` 包与 `content/original/packages/{offline,onlyOL}.npack.json` 的 schema／模块／逻辑哈希／武将与技能 ID 唯一性不变量校验；
     - `shiji`（`0.1.0` $\to$ `0.2.4` $\to$ `0.2.5`）与 `onlyOL`（`0.1.0` $\to$ `0.1.2` $\to$ `0.1.4` $\to$ `0.1.5`）分包升级差异审计（武将／技能相对声明顺序保持、`pinghe_damage_before` 滤镜 `owner_is_target` $\to$ `owner_is_target_other_source_alive`、`ol_fenli`／`ol_pingkou`／`ol_jie_danshou` 增量模块与逻辑哈希变化）。
  3. **`UP37-R10` & `UP37-N01`（AI／显示／元数据全量 hunk 与双线 AI 估值／RNG 隔离）**：
     - 补齐 `UP37-R10`（`c2c9ae0b`、`673bc921`、`90b58bb9`、`a44934c5`、`2492902c`、`19d86519`）与 `UP37-N01`（`c8d29532`）全部上游提交 hunk 的同步回归：武将表（`ol_jie_zhuhuan`、`ol_jie_zhuran`、`pot_yuji`、`tw_fanchou`、`ol_peixiu`、`sxrm_xunyu`）、技能文案、卡牌翻译（`pss_paper`／`pss_scissor`／`pss_stone`／`db_atk1..2`／`db_def1..2`）、`manualConfirm`、`prompt2`、`poptip` 注册及有限 AI 价值（禁止 `-Infinity`／`NaN`）；
     - 旧线 `engine-ai`（`HeuristicSearch`、`BeamSearch`、`MctsSearch`）在 `UP37-N01`（`ol_jie_danshou` 四分支主动技、`ol_fenli`、`ol_pingkou`）与 `UP37-R10`（`pot_yuji`、`tw_fanchou`、`ol_peixiu`、`sxrm_xunyu`）局面下的确定性可复现、节点预算有界、`SearchRng` 与权威 `RuleRng` 严格隔离及 `apply_if_fresh` 版本门控；
     - 原生 `engine-native::ai::BoundedPlanner`（`policy_variant = 1` 与 `policy_variant = 2` + `ContentTuning`）在装载真实 `ol_jie_danshou`／`ol_fenli`／`ol_pingkou`／`pinghe` DSL 扩展包下的确定性规划、`ContentTuning` 非有限权重／威胁度（`f64::INFINITY` $\to$ `1.0`）有限值钳制、模拟 `RuleRng` 与权威 `state_hash` 零污染、以及 `ChoiceView::Own` vs `ChoiceView::Other` 的 `NC10` 隐私边界。

## 2. 改动文件清单

- [pa12_tag656_compat.rs](../../../../crates/engine-core/tests/pa12_tag656_compat.rs)
- [up37_r10_metadata.rs](../../../../crates/engine-core/tests/up37_r10_metadata.rs)
- [search_policies.rs](../../../../crates/engine-ai/tests/search_policies.rs)
- [adr0786_j2_package_versions.rs](../../../../crates/engine-native/tests/adr0786_j2_package_versions.rs)
- [pa12_tag656_version.rs](../../../../crates/engine-native/tests/pa12_tag656_version.rs)

## 3. 执行命令与验证结果（首次失败与修复后重跑分开记录）

1. **定向测试首次运行（保留失败记录）**：
   - `engine-native` 首次运行见 [01-engine-native-targeted-first-run-failed.log](01-engine-native-targeted-first-run-failed.log)：
     - `onlyol_010_to_015_package_store_and_native_locked_content_matrix` 因直接断言恢复后 `encode_archive(&restored.save()) == bytes_old` 失败（原因：`Engine::restore_with_extensions` 按 `N11-a` 契约将快照内 `auth_generation` 旋转 `+1`，`core`／`resolution`／`state_hash` 保持完全相等）；
     - `up37_n01_and_r10_native_bounded_planner_tuning_and_rng_isolation` 因推进到 `Phase::Play` 的循环未等待 `open_choice().is_none()`（`ol_fenli` 在进入阶段时打开可选确认窗）而提前退出。
   - 修正上述两处测试断言与阶段推进条件，并在 `pa12_tag656_version.rs` 中同时覆盖归档头尾规则版本不一致（`ArchiveError::RuleVersionMismatch`）与一致旧规则版本（`SnapshotError::RuleVersionMismatch`）。
2. **定向测试修复后重跑（全部通过）**：
   - 命令：`cargo test --offline -p engine-core --test misc pa12_tag656_compat:: -- --nocapture && cargo test --offline -p engine-core --test misc up37_r10_metadata:: -- --nocapture && cargo test --offline -p engine-ai --test search_policies -- --nocapture && cargo test --offline -p engine-native --test misc adr0786_j2_package_versions:: -- --nocapture && cargo test --offline -p engine-native --test misc pa12_tag656_version:: -- --nocapture`
   - 结果：`pa12_tag656_compat` **6/0**、`up37_r10_metadata` **7/0**、`search_policies` **8/0**、`adr0786_j2_package_versions` **3/0**、`pa12_tag656_version` **2/0**（合计 **26 passed; 0 failed**）。
   - 日志：[02-targeted-tests-rerun-passed.log](02-targeted-tests-rerun-passed.log)。
3. **行数与分层门禁**：
   - 命令：`python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py' && python3 -B scripts/check-source-lines.py && python3 -B scripts/check-native-layering.py`
   - 结果：自测 2/2 OK，`9112 files <= 2000 lines; Rust non-test files <= 1000 effective lines`，`0 upward references <= 0`。
   - 日志：[03-line-and-layering-gates.log](03-line-and-layering-gates.log)。
4. **原生三 crate 全量测试**：
   - 命令：`cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native`
   - 结果：**1,987 passed; 0 failed; 75 ignored**。
   - 日志：[04-three-crates-tests.log](04-three-crates-tests.log)。
5. **独立 `CARGO_TARGET_DIR` 下 Gate 1b `-D warnings` 与 `wasm32` 检查（私有 target 已当场清理）**：
   - 命令：`CARGO_TARGET_DIR=/tmp/noname-gate1b-up37-b1 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`（随后 `rm -rf /tmp/noname-gate1b-up37-b1`）与 `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm`
   - 结果：全部 `exit 0`、0 警告。
   - 日志：[05-gate1b-warnings.log](05-gate1b-warnings.log)、[06-wasm32-check.log](06-wasm32-check.log)。
