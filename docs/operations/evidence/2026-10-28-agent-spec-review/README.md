# Agent 规范与现有架构审查（2026-10-28）

本目录记录 `NONAME_NEXT_AGENT_SPEC.md` v1.0 的审查依据，以及 v1.1 将七项意见转为实施约束的文档修订。**本批只改文档并保存审查输出，没有修复产品代码、升级规则／格式／ABI、提交或推送。** 当前状态仍以[进度主稿](../../../architecture/native-progress.md)和[剩余台账](../../../architecture/native-remaining-ledger.md)为准。

## 1. 基准与保护范围

- 审查与文档修订核对的 HEAD：`ec672a1d3c413d39c9972ee02118ae92a21223ee`；规则 v323、存档 v105、全局原生宿主 ABI 753，按当日剩余台账核对。
- 开始时已跟踪工作树无改动；已有未跟踪目录 `docs/operations/evidence/2026-10-28-nc07-lianyaohu-hash/`，原样保留。其存在不代表 NC07 修复已进入该 HEAD。
- 规范文件位于子仓库上层：[NONAME_NEXT_AGENT_SPEC.md](../../../../../NONAME_NEXT_AGENT_SPEC.md)。本批保留其位置；它不会出现在 `noname-next` 的 Git diff 中。修订前 SHA-256 为 `6fcbc1de09a283c9f02ee2385940ea55ec2ab01aa1694c53c7fc97045f963d0f`。
- 结构审查使用现有 CodeGraph CLI；同步已有索引后查看当前源文件与调用关系。Rust 探针在工作区外的独立 crate／target 执行，没有向共享测试目录注入用例。

## 2. 七项意见的落点

此表是本次审查与修订的对应关系，不是另一份动态缺陷台账。下列源码链接只定位本次问题的入口，后续状态查原 NC 台账。

| 审查项 | 证据或已有依据 | v1.1 修订位置与要求 |
|---|---|---|
| 1. WASM 插件被误列为必做后端 | [ADR 0782](../../../adr/0782-extension-backend-and-legacy-js-scope.md)规定内置 HostBehavior + 第三方 rule-vm，WASM 暂缓 | §1、§2、§5、§6、§11、§12 统一后端；区分引擎编译为 WASM 和采用 WASM 插件 |
| 2. 交付范围缺少既定目标 | [性能架构](../../../../noname-performance-architecture.md)、[收敛方案](../../../architecture/native-convergence-plan.md)包含移动端、原生牌桌、37 来源及旧线退出 | §1、§2、§7、§9、§10、§14 承接完整范围；Vue／Tauri 明确为现有路径，不能替代原生表现验收 |
| 3. 从空白重建与已有任务体系重复 | 已有 Workspace、模块、ADR 和 NC01–NC12／六阶段 | §0 明确主稿分工，§2.3 映射现有模块，§10 替换 P0–P4，§11 更新启动提示，§12 复用唯一 ADR 编号 |
| 4. VM 恢复改变技能身份与存储 | [restore_paused](../../../../crates/engine-native/src/rvm_bridge.rs)用模块名重建技能名、重置上下文；[B7 原证据](../2026-10-28-nc04-choice-retry/README.md)；本次独立复现见下节 | §4.1／4.4、§9、GT-011 要求身份、上下文、限次、命中计数、存储键与故障一致；沿用 NC04／B7 |
| 5. 语义牌区漏入摘要 | [PlayerState](../../../../crates/engine-native/src/state.rs)、[canonical_hash](../../../../crates/engine-native/src/engine/hash.rs)；[原反例](../2026-10-28-nc06-zhucheng-restore/lianyaohu-hash-probe.rs)；本次独立复现见下节 | §3.4、§7.3、§9、GT-012 要求字段同源、语义 hash 完整和完整状态／轨迹比较；沿用 NC06／NC07 |
| 6. 本地／联机投影不一致 | [nativeObserverToSnapshot](../../../../apps/web-client/src/worker/nativeBackend.ts)将身份填 null、武将填 0；[服务端投影](../../../../apps/game-server/src/native_view.rs)已有身份规则；[BattleView](../../../../apps/web-client/src/views/BattleView.vue)按身份字段显示 | §8.3、§9、GT-013 要求同局面、同权限的字段与产品显示一致，同时验证不泄密；沿用 NC03／NC09 |
| 7. 已有 VM 副本可用不等于本地正式可安装 | [bundledNativePackage](../../../../apps/web-client/src/engine/bundledNativePackages.ts)仍拒绝含 VM 模块的包，由[本地安装入口](../../../../apps/web-client/src/engine/extensionStore.ts)调用 | §6.2、§9、GT-014 验作者产物经正式入口完成全生命周期，保留校验和旧局内容锁；沿用 NC02／NC05／NC09 |

## 3. 本次审查实际验证

原始 Rust 编译、测试和失败输出保存为 [probes.log](probes.log)。以下结果适用于上述 HEAD；文档修订没有把失败登记为修复。

| 验证 | 结果与边界 |
|---|---|
| 隔离 crate 复制现有 `nc04_vm_resume.rs` 并追加既有 storage 反例 | **4 passed / 1 failed / 0 ignored**。原局只有 `nc04_storage_skill#7`，恢复局另有 `nc04-storage-module#7`；精确相等断言失败，整个 cargo 命令 exit 101 |
| 隔离 crate 运行炼妖壶 hash 反例 | **1 passed / 0 failed / 0 ignored**，此“通过”表示缺陷重现：`[8,80]` 与 `[80,8]` hash 相同、选择顺序不同，绝非修复通过 |
| `node --import ./scripts/ts-register.mjs --test scripts/native-adapter.test.mjs scripts/native-vm-snapshot.test.mjs`，目录 `apps/web-client/` | **10 passed / 0 failed / 0 skipped**。仅证明已有适配与预置副本恢复测试，不覆盖 GT-013／014 的完整产品要求 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **1 test / OK** |
| `python3 -B scripts/check-source-lines.py` | **8651 files <= 2000 lines**，exit 0 |
| `git diff --check` | exit 0；只覆盖子仓库，规范另行比较修订前副本 |

本次没有运行全量 workspace、真实浏览器／Worker、服务端 WS、目标设备、全来源矩阵或性能基准。静态投影与安装入口结论不伪装成新的产品 E2E 通过。

### Rust 反例的复现方法

在 `noname-next/` 根目录运行以下命令；复用仓库已有测试和探针，不修改生产代码或共享 target。需要可用 Rust 工具链与本地依赖缓存。审查基准的预期退出码为 **101**；修复后应逐项检查输出，并将“证明缺陷存在”的 hash 反例改成正式修复回归。

```bash
python3 - <<'PY'
from pathlib import Path
import json
import os
import shutil
import subprocess
import tempfile

repo = Path.cwd().resolve()
probe = Path(tempfile.mkdtemp(prefix="noname-architecture-review-"))
(probe / "tests").mkdir()
manifest = '[package]\nname = "noname-architecture-review"\nversion = "0.0.0"\nedition = "2021"\n\n[workspace]\n\n[dependencies]\n'
for name in ["engine-native", "rule-vm", "content-native-loader"]:
    manifest += f'{name} = {{ path = {json.dumps(str(repo / "crates" / name))} }}\n'
manifest += '\n[profile.test]\ndebug = 0\nincremental = false\n'
(probe / "Cargo.toml").write_text(manifest)
shutil.copyfile(repo / "Cargo.lock", probe / "Cargo.lock")
evidence = repo / "docs/operations/evidence"
vm = (repo / "crates/engine-native/tests/nc04_vm_resume.rs").read_text()
vm += "\n" + (evidence / "2026-10-28-nc04-choice-retry/storage-divergence-probe.rs").read_text()
(probe / "tests/vm_restore.rs").write_text(vm)
jar = (evidence / "2026-10-28-nc06-zhucheng-restore/lianyaohu-hash-probe.rs").read_text()
jar = jar.replace('env!("CARGO_MANIFEST_DIR")', json.dumps(str(repo / "crates/engine-native")))
(probe / "tests/state_hash.rs").write_text(jar)
env = dict(os.environ, CARGO_NET_OFFLINE="true", CARGO_INCREMENTAL="0", CARGO_PROFILE_TEST_DEBUG="0", CARGO_TARGET_DIR=str(probe / "target"))
with (probe / "probes.log").open("w") as log:
    result = subprocess.run(["cargo", "test", "--offline", "--manifest-path", str(probe / "Cargo.toml"), "--no-fail-fast", "--", "--nocapture"], env=env, stdout=log, stderr=subprocess.STDOUT)
print(f"log: {probe / 'probes.log'}; exit: {result.returncode}")
raise SystemExit(result.returncode)
PY
```

## 4. 文档修订核验

七项意见已逐项写入规范正文并映射到已有 NC 工作包；GT-011～014 是待实施的验收要求，不新增完成声明。原 §0～14 均保留，删除了新建插件宿主、替代目录布局与独立 P0–P4 执行指令。

- 规范与本页相对链接检查通过；全仓文档检查 **1860 文件、0 坏链**。
- ADR 索引检查通过：未登记冲突、过时登记、标题错号和索引坏链均为 0；既有已登记重复编号没有改动。
- 复现命令的 Python 语法检查通过；本批未因文档修订重跑完整 Rust 或产品验收。
- 子仓库 `git diff --check` 通过；上层规范与修订前副本的 `git diff --no-index --check` 无空白错误（exit 1 表示文件有内容差异）。首次检查发现五处新增 Markdown 双空格换行被 Git 判为尾随空格，已改为显式换行或段落；[首次输出](document-whitespace-initial.log)保留，未改动其他任务文件。

详细命令与结果见[文档核验日志](document-checks.log)。
