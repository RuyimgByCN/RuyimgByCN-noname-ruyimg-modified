# stash@{0}（I02／NC10 WIP）在 C331 之上的恢复、适配与验收（2026-11-02）

按[裁决 32①](../2026-10-28-user-decisions/README.md)：`stash@{0}`＝`a4886399`（基于 `a4483b64`，消息 "I02/NC10 WIP before C331 merge"；备份 `/private/tmp/noname-main-wip-backup-20261102T205333Z/`）在 main 最新提交上经隔离工作树冲突分析后恢复、定向与全量门禁后入库。**stash 本体未 pop／drop，备份未删**；入库并核验后再按裁决处理。

## 内容与恢复方式

- 跟踪部分 35 文件（I02 裁决 30 的 definition 级 `voice_assets`（PackageCharacterEntry／PackageSkillEntry／PackageCardEntry 可缺省字段＋校验＋asset-hash 覆盖层）；NC10 公开 Other 规划修复（`engine-native/src/ai.rs` 的 `ChoiceView::Other` 臂去 `vm_choice_leaf` 守卫，非所有者建议一律 idle→NextPhase）；进度主稿 5 条 2026-10-31 历史条目）。
- 未跟踪部分 8,871 文件全部恢复（与 `stash@{0}^3` 逐路径双向核对零缺失零多余）；与 main HEAD 已跟踪路径重叠为 0。
- 冲突：`git stash apply` 在 6 个文件（`content-compiler/src/lib.rs`＋5 个 `gen_*_package.rs`）出现 24 处内容冲突——C331 在同一批 `PackageSkillEntry` 字面量头部插入 schema5 七字段、stash 同位置插 `voice_assets`；在隔离工作树 `/private/tmp/noname-stash-restore-20261102/worktree`（分支 `stash-restore-i02-nc10`）按「两侧都保留」解决（C331 七字段在前、`voice_assets` 在后），然后把解决的 6 个文件与全部恢复内容核对着移入 main 工作区（[02](02-resolve.log)：其余 34 路径与隔离树逐字节相同，唯一差异为 `native-progress.md`——main 在 stash 基线后新增了 6 条顶部条目，自动合并已处理）。
- **「已被 C331 等价取代」集合为空**：main 中不存在 definition 级 `voice_assets`、`validate_definition_voice_assets`、legacy 摘要拆分或 `nc10_public_other_plan` 的任何等价物（main 仅有既存的 skin 级 `PackageSkinEntry.voice_assets`，不同层）；无舍弃片段。
- 中性核验（裁决 30）：`voice_assets` 缺省不序列化、`compute_logic_hash` 不含 voices、asset hash 仅在存在非空声明时以定长域 `noname-next/npack-definition-voice-assets/v1` 混入——与 C331 的 schema 精确门禁（`required_content_schema` 只看 mechanisms/非默认序）正交。

## 对 C331 的两处测试适配（其余零改动）

`crates/content-runtime/tests/definition_voice_assets.rs` 原样来自 stash 时有 2 处与 C331 新增的「逻辑技能身份」规则（`package/trigger_order.rs`：skill_index 的 `skill_def_id` 与 `name` 必须各自唯一）相撞：

1. `voice_hash_uses_category_and_stable_id_and_normalized_map_order`：构造「同 id 第二条目＋倒序」样例时，skill_index 的重复 `name` 先触发身份冲突——适配为 skill_index 分支给第二条目改名再比较哈希序无关性（哈希仍只受 voice 域影响，见变异 2）。
2. `voice_definition_ids_must_be_nonzero_and_unambiguous`：重复 `skill_def_id` 的拒绝错误现在是 `conflicting logical skill identity`（C331 身份检查先于声明语音检查），character/card 分支仍是 `ambiguous … voice definition`——按分支断言各自的实际错误文本。

两处均为测试样例构造适配，未改任何产品代码；`nc10_public_other_plan.rs`、`native_frozen_packages.rs` 追加测试零改动直接通过。首轮红（2 失败）保留在 [03](03-targeted-new-tests-first.log)，适配后 [04](04-targeted-after-adapt.log) 7/0。子代理逐文件冲突分析报告（含 API 兼容静态核验、`.pi-lens.json`＝本机工具配置不入库、版本号待改清单为空）在 `/private/tmp/noname-stash-restore-20261102/REPORT.md`。

## 变异红→绿（修复被测试真实覆盖）

| 变异 | 结果 | 日志 |
| --- | --- | --- |
| `ai.rs` Other 臂回退到 HEAD（恢复 `vm_choice_leaf` 守卫） | `nc10_public_other_plan` **3 红**（普通 Other／死亡 Other／真实缓存 VM 效果三例），恢复后 5/5 | [05](05-mutation-ai-other-arm.log) |
| 去掉 `validate_definition_voice_assets` 调用 | **3 红**（非零 id／歧义 id／安全键与本地 Audio 成员），恢复后 7/0 | [06](06-mutation-voice.log) |
| `compute_asset_hash` 直接返回 legacy 摘要（覆盖层失效） | **3 红**（往返只变 asset hash／空 voices 保持 legacy 零摘要／归一化键序），恢复后 7/0 | [06](06-mutation-voice.log) |

## 定向与全量门禁（2026-11-02，main 工作区＝恢复后状态；331／NGSA 106／ABI 758／WS7 不变）

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| `cargo test -p content-runtime --test definition_voice_assets` | **7/0** | [04](04-targeted-after-adapt.log) |
| `cargo test -p engine-native --test nc_rounds nc10_public_other_plan::` | **5/0** | [04](04-targeted-after-adapt.log) |
| `cargo test -p game-server --test native_frozen_packages` | **8/0**（含冻结链路新测试；首轮即为绿） | [03](03-targeted-new-tests-first.log) |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | OK | [10](10-source-lines-unit.log) |
| `python3 -B scripts/check-source-lines.py` | 9,077 文件通过 | [11](11-source-lines.log) |
| `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast` | **1908／0／75**（17 段） | [12](12-three-crates.log) |
| `cargo test --offline --workspace --no-fail-fast` | **7167／0／77**（173 段；较合并树基线 7154 增 13＝voice 7＋frozen +1＋public_other 5，测试二进制 146→147） | [13](13-workspace.log) |
| `pnpm build:native` | exit 0；native WASM 4,289,839 B，sha256 `668e0eb4…`（含 ai.rs Other 臂改动，与合并树产物不同） | [20](20-build-native-wasm.log) |
| `pnpm smoke:native`（真实 Worker） | **82／0** | [21](21-worker-smoke.log) |
| `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true` | [22](22-cross-target.log) |
| `cargo test -p desktop-client --no-fail-fast` | 12＋6／0 | [23](23-desktop-client.log) |
| `pnpm test`（前端） | **449／0** | [24](24-frontend-test.log) |
| `pnpm typecheck` | exit 0 | [25](25-typecheck.log) |
| 独立 `CARGO_TARGET_DIR` 下 `RUSTFLAGS="-D warnings" cargo check … --all-targets`；`cargo check --target wasm32-unknown-unknown -p engine-native-wasm` | 两项 exit 0（私有 target 用后删除） | [26](26-strict-and-wasm32.log) |
| `python3 -B scripts/preflight.py` | 21 项全部通过 | [27](27-preflight.log) |

## 边界

- `voice_assets` 字段只是声明层：生产 presentation constructor 仍为 `None`，I02 其余（真实公开投影接线、资源 binding、一次消费、自动语音、皮肤、18 类资源、2,487 缺 backing）未完成；NC10 的 19 项 NoCertifiedRoot、root／Confirm 前缀／Cancel 尾链／VM 续行门禁未放宽。
- 规则／存档／ABI／WS 版本不变；golden 未重录（本批无行为面变化，`start-hash` 等基准不受影响——同机对照见 CI 发布证据）。
- `.pi-lens.json` 保持未跟踪（本机工具配置，历史证据靠已存档摘要复现）。
