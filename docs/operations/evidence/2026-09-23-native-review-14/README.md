# 2026-09-23 原生内核第 14 轮独立核验

ADR 0774/0775 后源码，规则版本 28；[源码摘要](source-hashes.json)、[环境](environment.txt)。
当前状态与带完成条件的待办见 [进度主稿](../../../architecture/native-progress.md)。
本轮只改文档及保存证据，未修改内核、工程测试、示例或 golden。

## 证据

| 文件 | 执行结果与边界 |
|---|---|
| [doc-guard.log](doc-guard.log) | 整理文档后重跑覆盖清单登记及 14 golden 两项通过 |
| [native-tests.log](native-tests.log) | cargo test --offline -p engine-native：**232 passed**，0 failed；native_kernel=155，doc-tests=0（当轮 ADR 0775 后源码）；ADR 0776 后 235 项见 [本次证据](../2026-09-23-extension-status/README.md) |
| [probes.rs](probes.rs) / [probes.log](probes.log) | 真实 API 独立断言：流离内存、六技能接线与中段缺牌原反例关闭；确认尾端/全量缺牌、字节/选择恢复、护驾队列/多闪、AI 信息集/归属/序号/候选及 holder 越界问题 |
| [prompt-audit.txt](prompt-audit.txt) | 源码字面 prompt 对照固定表，发现 7 种缺项；其中决斗/AOE/无懈已实际构造并证实合法自产字节无法解码，其余仅静态发现 |
| [runner.log](runner.log) / [samples.csv](samples.csv) | release counting，20 9000；FFA/身份各 20/20，判定 10 组；11758 行无表头 CSV，四列 load,game,seq,ns |
| [table.log](table.log) | release n10_table 5 7000：5 局、14207 样本、12628 次 ShaLimit；没有帧提交，fallback 不在样本内 |
| [writer.log](writer.log) / [reader.log](reader.log) | 独立进程 write/read，687 字节；额外解析两份日志比较 H25/H35 相等；该路径通过不代表全部选择点可恢复 |
| [kof-source.rs](kof-source.rs) / [kof-probe.rs](kof-probe.rs) / [kof.log](kof.log) | 当前示例临时副本仅加结果计数及解析断言：seed 11 首局 2048 命令含 1998 次 ShaLimit、outcome=None；后两局真实终局。实际解析器空池/损坏/越界通过，重复将与场次矛盾仍被接受 |

探针退出 0 表示其修复/反例断言成立，不表示引擎全部验收通过。
新引擎测试中的 KOF 负例只复制解析片段，本目录副本实际调用当前 KofSession::restore 以独立核验。

## 复跑

在项目根目录执行：

```bash
cargo test --offline -p engine-native
N09_SAMPLES=/tmp/review14-samples.csv cargo run --offline --release -p engine-native --example n09_runner -- 20 9000
cargo run --offline --release -p engine-native --example n10_table -- 5 7000
cargo run --offline --release -p engine-native --example n12_crossproc -- write /tmp/review14-archive.bin
cargo run --offline --release -p engine-native --example n12_crossproc -- read /tmp/review14-archive.bin
```

独立探针使用匹配 debug 库的工具链；本轮为 /opt/homebrew/bin/rustc 1.98.0：

```python
from pathlib import Path
import subprocess
root = Path.cwd()
evidence = root / 'docs/operations/evidence/2026-09-23-native-review-14'
lib = max((root / 'target/debug/deps').glob('libengine_native-*.rlib'), key=lambda p: p.stat().st_mtime)
for name in ('probes', 'kof-probe'):
    output = '/tmp/native-review14-' + name
    subprocess.run(['/opt/homebrew/bin/rustc', '--edition=2021', str(evidence / (name + '.rs')),
                    '--extern', 'engine_native=' + str(lib), '-L', 'dependency=' + str(root / 'target/debug/deps'),
                    '-o', output], check=True)
    subprocess.run([output], check=True)
```

KOF 副本保留原策略、轮换和超时判胜，只有诊断和额外解析断言；没有修改生产 example。
独立断言在相应问题修复后应改为预期正确行为，再加入工程回归；不能长期把缺陷冻结为正确规则。

## 范围

- 14 golden 属一个场景测试，17 pause_roundtrip 不是所有新增技能的字节恢复矩阵。
- 本轮没有复验旧线浏览器、原生 GPU、跨平台/WASM 或动态扩展；counting 单次不用于提速倍数。
- [第 13 轮](../2026-09-22-native-review-13/README.md) 与 [版本 24 记录](../2026-09-22-native-review/README.md) 保留为历史，过期反例由本轮状态替代。
- 扩展 E01–E06 依本任务用户明确要求保持必做；未找到可将其从当前目标剥离的对应指令，原稿的关闭声明不作为验收证据。
