from pathlib import Path
import json,hashlib
base=Path(__file__).parent.resolve();changes=json.loads((base/'changed-files.json').read_text());paths=[c['path'] for c in changes];taskbase=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq');evidence=taskbase/'evidence';out=Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs/358affc0-bc9f-4ef1-92f3-9dc5c617cf1a/implementation/f7-stage3-layering.md')
commands=[
 {'command':'python3 -B -m unittest discover -s scripts -p test_check_native_layering.py (before migration)','result':'failed','summary':'Expected red: current-source zero-reference regression observed 240 upward references; layering-red.log.'},
 {'command':'python3 -B -m unittest discover -s scripts -p test_check_native_layering.py','result':'passed','summary':'Final 3 tests: real source zero, recursive vocabulary coverage, grouped use / crate alias / relative import detection.'},
 {'command':'python3 -B scripts/check-native-layering.py','result':'passed','summary':'0 upward references <= BASELINE 0; vocabulary included in KERNEL_MODULES.'},
 {'command':f'CARGO_TARGET_DIR={taskbase}/target cargo check --offline -p engine-native --all-targets','result':'passed','summary':'check-final.log; all targets finished successfully.'},
 {'command':f'CARGO_TARGET_DIR={taskbase}/target cargo test --offline -p engine-native','result':'passed','summary':'native-tests-final.log: 1644 passed, 0 failed, 73 ignored; unchanged golden replay passed.'},
 {'command':f'CARGO_TARGET_DIR={taskbase}/target cargo test --offline -p engine-native-wasm -p content-native-loader -p game-server','result':'passed','summary':'dependent-tests-final.log: wasm 71/0/0, loader 32/0/0, game-server 362/0/0.'},
 {'command':f'CARGO_TARGET_DIR={taskbase}/gate-target RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --all-targets','result':'passed','summary':'Independent Gate 1b target, final source: gate1b-final.log.'},
 {'command':'cargo test --offline -p engine-native -- --list (before and after, isolated target)','result':'passed','summary':'1717 full names before and after, identical multisets; no additions/removals/renames.'},
 {'command':'Native nc08_cross_target_real + read-only previous generated WASM trajectory + compare.py','result':'passed','summary':'40 standard steps, save/restore at 20; hashes, revisions, events identical. Initial relative output path failed; absolute output path rerun passed.'},
 {'command':'Native WASM cross_target_package + read-only previous generated WASM package trajectory + compare.py','result':'passed','summary':'20 installed huanle-cards steps, save/restore at 10; hashes, revisions, events identical.'},
 {'command':'python3 -B -m unittest discover -s scripts','result':'failed','summary':'184 tests at that point; 183 passed, only existing doc file:line guard failed after source relocation. Final additional grouping scanner regression separately passed (3 layering tests).'},
 {'command':'python3 -B -m unittest discover -s scripts -p test_report_event_arms.py; corresponding test_report_g14.py, test_report_phase_filters.py, test_report_filter_meta_mismatch.py','result':'passed','summary':'Final focused suites 15 + 7 + 9 + 13 tests passed; metadata/key/host declaration extraction also identical against frozen source.'},
 {'command':'python3 -B -m unittest discover -s scripts -p test_check_source_lines.py; corresponding test_rust_effective_lines.py; python3 -B scripts/check-source-lines.py','result':'passed','summary':'2 + 1 unit tests passed; 8910 maintained files <=2000 physical lines; Rust non-test files <=1000 effective lines.'},
 {'command':'python3 -B scripts/preflight.py --quick','result':'failed','summary':'18 passed, 1 failed (public documentation source file:line references), 2 skipped; skipped ledger-consistency and script tests were exercised by the full script suite.'},
 {'command':'python3 -B scripts/check_local_links.py --all','result':'passed','summary':'1880 documents, 0 broken links after linking missing read-only AGENTS/spec dependencies in task base.'},
 {'command':f'CARGO_TARGET_DIR={taskbase}/gate-target cargo check --offline --manifest-path {evidence}/kernel-closure/Cargo.toml','result':'passed','summary':'Standalone library includes only complete KERNEL_MODULES and LogEvent facade re-export, no content/engine module; passed with one unused private constructor warning. Initial warnings-denied harness failed only on MonkeyStealMachine::new dead_code, recorded separately.'},
 {'command':'git apply --check on frozen baseline; apply to evidence/patch-probe and compare every changed path','result':'passed','summary':'All 88 paths reconstructed exactly, including 16 new and 11 deleted files; no index used.'},
 {'command':'frozen 757-path manifest audit; moved-source identity checks; git diff --cached --quiet; git rev-parse HEAD','result':'passed','summary':'No unexpected drift outside stage3 patch; 10 whole moved source files byte-identical; nothing staged; HEAD unchanged at 95f51867a3648321da1cc92149b60e2bbfaae748.'},
]
report={
 'criteriaSatisfied':[
  {'id':'criterion-1','status':'satisfied','evidence':'88 paths confined to engine-native source/necessary regression scanners plus authorized script consumers. Complete kernel closure, including vocabulary, has 0 upward references and BASELINE 0. Legacy public paths/default metadata behavior preserved; no rules/archive/ABI/Cargo/protocol/docs changes.'},
  {'id':'criterion-2','status':'satisfied','evidence':'Frozen-state patch with SHA256, full changed-file manifest, actual patch reconstruction, red/green logs, final full Rust/dependent suites, independent Gate 1b, identical 1717 test names, unchanged golden definitions and matching old-WASM state trajectories, line gates, explicit remaining doc-review work.'}
 ],
 'changedFiles':paths,
 'testsAddedOrUpdated':[p for p in paths if '/tests/' in p or p=='scripts/test_check_native_layering.py'],
 'commandsRun':commands,
 'validationOutput':['layering red: 240 != 0; final: 0 <= BASELINE 0','engine-native: 1644 passed / 0 failed / 73 ignored','engine-native-wasm: 71 passed; content-native-loader: 32 passed; game-server: 362 passed','test names: 1717 == 1717, identical multisets','standard trajectory final hash: a3c8dfab6e9722906de982e6a44644549bb979ce39ac5e56f9588e5349350b01','installed package trajectory final hash: afd094672ab990cb8c457d60481bb5f69995d95767becfa44cc6e223ce1e0005','ruleset 328 / archive 105 / native VM ABI 753 unchanged; golden table/source unchanged; no golden re-recording'],
 'residualRisks':['Public historical source citations need parent integration using source-reference-map.json; doc guard and therefore full preflight are not green in frozen isolated docs. No public documents or parent new doc validator were edited.','Actual three-crate split, cold/incremental performance, and new WASM build were not performed. Cross-target comparison used the unchanged read-only previously generated WASM.','Stage4 must expose required cross-crate pub(crate) APIs, including shared assembly and MonkeyStealMachine::new; keep all HostBehavior inherent impls with its definition, and content wrapper SkillBehavior impl with its local wrapper.','Independent reviewer gate remains required before parent integration; 73 pre-existing ignored Rust tests remain ignored.'],
 'noStagedFiles':True,
 'diffSummary':'16 additions, 61 modifications, 11 deletions; pure vocabulary/metadata moves and pub use compatibility, shared assembly in state, equipment-response content callbacks, recursive complete-root layering scan and path-only consumer synchronization.',
 'reviewFindings':['no blockers in implementation self-review; independent parent reviewer required','integration: historical document source paths/line ranges must be remapped; no doc-validator changes included'],
 'manualNotes':f'All artifacts under {evidence}; authoritative patch f7-stage3.patch SHA256 41cbc09212a09e691563bd128854743e390879b67abc3ffc880967ba5d4aa56e. No commits, staging, reference changes, main-tree writes, or main-target builds. The optional kernel closure harness initially selected newer cached dependencies; its final run reused the frozen Cargo.lock with blake3 pinned to 1.8.5. The actual product Gate1b used the unchanged workspace lock throughout.'
}
text=f'''Implemented F7 第 3 步：完整目标内核对 content/engine 的源码依赖 **240 → 0**，`BASELINE = 0`。隔离树 HEAD 保持 `95f51867a3648321da1cc92149b60e2bbfaae748`，无暂存、无提交、无引用修改；对原冻结九批最终状态 757 路径的核验没有发现本批清单以外的漂移。未混入后续 B9、desktop 或文档修复。

可审查补丁为 [f7-stage3.patch]({evidence}/f7-stage3.patch)，SHA256 `41cbc09212a09e691563bd128854743e390879b67abc3ffc880967ba5d4aa56e`。补丁从本轮开始前的冻结源码副本生成，包含新增与删除文件，**不包含原九批 overlay 本身**。完整清单及每个文件的动作、前后 SHA256 见 [changed-files.json]({evidence}/changed-files.json)：88 路径，其中源码 67、必要 Rust 测试 8、Python 脚本 13；新增 16、修改 61、删除 11。已在冻结副本执行 `git apply --check`，并在独立 probe 真正应用后逐文件对照最终内容，88 路径均完全一致；整个过程未使用 index。

迁移设计保持现有对外路径和默认行为：

- `vocabulary::names` 承接标准技能名；`vocabulary::host` 承接公开键、HostBehavior 枚举、ALL、key/tag 双向映射和 TriggerMeta 纯描述。**全部 HostBehavior inherent impl 与枚举同处该共享模块**。10 个完整迁移源文件逐字节一致；没有迁入内容机器、filter、activation、factory、step 或技能结算实现。
- `vocabulary::generals` 承接 Gender、标准/Boss 不可变武将描述、纯查询和 ExtensionGeneral。父线程明确批准此选择，以保留 `Catalog::build` 原来的默认将表及无 Catalog 参数的 `effective_gender` 接口。标准／扩展／Boss 下标和排列仍按原实现，扩展段在标准段之后、Boss 动态尾段在全部扩展之后；既有标准性别查询的范围及默认 Male 行为保持不变。
- `vocabulary::card_skills` 承接 HostSkillDecl 伴随技能数据表、装备技能描述和纯牌种分类 `yangpijuan_class`。`content::host`、`content::standard`、`content::extension` 通过 `pub use` 保持既有名称、调用路径及类型身份的使用方式；内容私有调参常量留在原内容模块。
- 神弓借牌候选和归属查询仍由内容函数实现，新增 `Catalog::hooks.borrow_options` / `borrowed_owner` 回调承接内核调用。标准目录及扩展合并目录都继续装入同一个 CONTENT_HOOKS；原始 `Catalog::build` 继续使用 ContentHooks::NONE，返回空候选/无归属。三个实际 build 调用方全部核对；原始 build 的默认将表，以及插入扩展后的 Boss 尾段顺序，加入既有 PA01 测试中验证。
- `apply_general_assembly` 的唯一实现迁到 `state/general_assembly.rs`；内核、engine 和 content 调用方复用该函数，保留体力调整、技能授予、初始化及禁用状态逻辑。engine 私有导入供其已有子模块沿用。没有改变 return／? 控制流、错误处理、规则、存档、ABI 或协议。

分层回归先在当前源码跑红，明确断言 `240 != 0`，见 [layering-red.log]({evidence}/layering-red.log)。迁移同一次修改中把 BASELINE 降到 0，随后保持为 0。新 vocabulary 已列入 KERNEL_MODULES，state/general_assembly 随 state 递归扫描；所有新下沉模块都在扫描闭包内。最终 3 个扫描单测覆盖当前源码、新 vocabulary 子模块回归、分组 use、crate 别名和相对导入，见 [layering-green.log]({evidence}/layering-green.log)。此外，用仅声明这套内核模块且完全不声明 content/engine 的临时库编译成功，进一步证明当前真实依赖闭包成立，见 [kernel-closure-final.log]({evidence}/kernel-closure-final.log)。

为保留源码扫描消费者，13 个 Python 文件各自的必要改动如下；没有修改主线程新文档校验器，也没有删除扫描、放宽豁免或抬高阈值。

| 文件 | 原因 |
|---|---|
| `scripts/check-native-layering.py` | BASELINE 降为 0，加入 vocabulary 根；展开分组 use 和 crate 别名、相对导入以避免路径隐藏。 |
| `scripts/test_check_native_layering.py` | 当前源码零引用红绿回归及新目录／导入形式回归。 |
| `scripts/audit_upstream_skill_flags.py` | 读取实际迁移后的 metadata/keys 定义。 |
| `scripts/check_guard_references.py` | 保留守卫命名校验，读取迁移后的键表。 |
| `scripts/check_ledger_consistency.py` | 台账行为名继续对照真实键表。 |
| `scripts/host_key_reachability.py` | 键表和牌种 HostSkillDecl 声明分别迁入 vocabulary/host 与 card_skills。 |
| `scripts/report_event_arms.py` | 触发元数据读取 vocabulary；执行/过滤臂继续读取 content。 |
| `scripts/report_filter_meta_mismatch.py` | 元数据根改为 vocabulary，过滤臂根保持 content。 |
| `scripts/report_g14.py` | 行为到 host:key 的读取路径迁移。 |
| `scripts/report_phase_arms.py` | 同上；过滤臂与上游扫描逻辑保持原样。 |
| `scripts/report_phase_filters.py` | 规则登记表继续读取真实键表。 |
| `scripts/report_registered_deviations.py` | 遍历内容行为与下沉元数据两处，保留原扫描集合。 |
| `scripts/report_unimplemented_hosts.py` | 同时计入原内容树及迁移后的纯描述，维持原出现次数判据。 |

旧／新源码上的 metadata_events 275 项、metadata_classes 517 项、native_metas 275 项、host_keys 520 项、HostSkillDecl 56 项及未实现行为分析结果完全相同，见 [scanner-source-conservation.json]({evidence}/scanner-source-conservation.json)。8 个 Rust 测试仅包含必要源码读取路径同步和既有 PA01 测试的默认目录／扩展尾段断言；测试注册和测试函数名称没有变化。

验证结果如下；所有 cargo 构建只使用指定 `{taskbase}/target`，Gate 1b 与额外闭包检查使用 `{taskbase}/gate-target`，没有使用或清理主 target。

| 验证 | 结果／证据 |
|---|---|
| `cargo check --offline -p engine-native --all-targets` | 通过，[check-final.log]({evidence}/check-final.log)。 |
| engine-native 全量 | **1644 passed / 0 failed / 73 ignored**，[native-tests-final.log]({evidence}/native-tests-final.log)。 |
| 受影响依赖全量 | WASM **71/0/0**、loader **32/0/0**、game-server **362/0/0**；在最终源码稳定后重跑完成，[dependent-tests-final.log]({evidence}/dependent-tests-final.log)。 |
| 独立 Gate 1b | `RUSTFLAGS=-D warnings` all-targets 通过，[gate1b-final.log]({evidence}/gate1b-final.log)。 |
| 测试全名守恒 | **1717 → 1717**，多重集合完全一致，无新增、删除或更名；[test-name-conservation.json]({evidence}/test-name-conservation.json)。 |
| 行数门禁 | 8910 文件物理行 ≤2000；Rust 非测试有效行 ≤1000；脚本单测 2+1 通过，[line-gates-final.log]({evidence}/line-gates-final.log)。 |
| 原 golden | `replay_scenarios.rs` 与冻结副本字节一致；`replay_scenarios_match_golden` 全量通过，无重录，[golden-preservation.json]({evidence}/golden-preservation.json)。 |
| 规范状态摘要／跨目标 | 标准 40 步及中点 20 恢复，正式 huanle 包 20 步及中点 10 恢复，逐步 hash/revision/event-code 与只读既有 WASM 全等；[cross-compare.log]({evidence}/cross-compare.log)、[cross-package-compare.log]({evidence}/cross-package-compare.log)。 |
| 脚本全量与定向 | 主迁移后全量 184 测试仅文档行号守卫 1 失败；最终分层 3、event-arms 15、G14 7、phase-filters 9、filter-meta 13 单测均通过。 |
| 文件与补丁保护 | 冻结 757 路径无非本批漂移；10 个整文件迁移字节一致；补丁应用／重建 88 路径一致；无暂存、HEAD 不变。 |

跨目标最终摘要分别为 `a3c8dfab6e9722906de982e6a44644549bb979ce39ac5e56f9588e5349350b01` 和 `afd094672ab990cb8c457d60481bb5f69995d95767becfa44cc6e223ce1e0005`。规则 v328、存档 v105、原生 VM ABI 753 保持冻结值。只读既有 WASM 通过本 base 的 native-pkg 链接读取；没有重建或写入主树 WASM。首次轨迹输出用了相对路径，因测试 cwd 不同而失败；改成指定 evidence 绝对路径后完整重跑通过，原失败日志保留。

Open risks/questions：**公共历史文档的源码引用需要父线程整合**。隔离树最终本地链接 1880 文件／0 坏链；缺失的只读 AGENTS/spec 文件只在 base 建链接。quick preflight 为 18 通过、1 失败、2 跳过，唯一失败是迁移后的公共文档 file:line 引用；跳过的台账与脚本测试已有完整脚本套件覆盖。具体缺失路径／越界 11 处见 [doc-line-refs.log]({evidence}/doc-line-refs.log)，旧→新路径及连续等同行段映射见 [source-reference-map.json]({evidence}/source-reference-map.json)。本批未写任何公共文档或 `check_doc_line_refs.py` / `test_check_doc_line_refs.py`，没有把父线程主树 preflight 21/21 当作本隔离树的通过结果。

第 4 步须继续留意：HostBehavior 的定义及全部 inherent impl 必须一起留在 kernel crate；SkillBehavior 实现在内容层的本地 HostSkillBehavior wrapper 上，因此不会形成“给外部 enum 实现外部 trait”的孤儿实现。HostMachine／标准具体机器仍在 content，能合法实现 kernel 的 SkillMachine；它们自己的 inherent impl 仍随本地类型。共享装配等现有 pub(crate) 项要在实际拆 crate 时开放所需可见性。额外纯内核库在普通 `cargo check` 下通过，但沿用 `-D warnings` 的首次探针因未被 engine 使用的 `MonkeyStealMachine::new` 触发 dead_code；此项在实际拆分时同属构造函数可见性处理，**产品独立 Gate 1b 已全绿**。临时库须在根重导出 `LogEvent`，维持现有 `crate::LogEvent` 消费路径。

Recommended next step：父线程按所附 patch 进行独立 reviewer 审查，再在保留 B9／desktop／新 doc 校验器修复的 main 上整合；根据映射只更新必要文档引用后补跑主树 preflight。本轮没有进行实际三个 crate 拆分、构建性能测量或新 WASM 构建；73 个既有 ignored 测试仍保持忽略。

```acceptance-report
{json.dumps(report,ensure_ascii=False,indent=2)}
```
'''
out.parent.mkdir(parents=True,exist_ok=True);out.write_text(text);(base/'acceptance-report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(out);print('bytes',len(text.encode()))
