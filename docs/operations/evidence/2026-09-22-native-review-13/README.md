# 2026-09-22 原生内核第 13 轮独立核验

对应 ADR 0768–0773 后源码，规则版本 26。[源码摘要](source-hashes.json) 固定受验版本，
[环境](environment.txt) 记录时间、系统与工具链。本轮仅文档/证据改动；源码、工程测试、示例保持不变。
当前结论与后续验收见 [进度主稿](../../../architecture/native-progress.md)。

## 结果与证据

| 记录 | 方法与结果 |
|---|---|
| [doc-guard.log](doc-guard.log) | 文档联动复跑 replay_scenarios 两项通过：覆盖登记与 14 golden |
| [native-tests.log](native-tests.log) | cargo test --offline -p engine-native：234 passed，0 failed；14 golden 计入场景测试 |
| [probes.rs](probes.rs) / [probes.log](probes.log) | 独立链接当前 debug 库：复验旧修复，并确认缺牌、流离续跑、护驾权限、AI 信息集/序号/候选与六将接线缺失；退出 0 表示这些断言成立，不表示无缺陷 |
| [runner.log](runner.log) / [samples.csv](samples.csv) | release counting，20 9000；FFA/身份各 20/20；判定 10 组，CSV 无表头四列，共 11758 行 |
| [table.log](table.log) | release n10_table 5 7000；5 局、14207 样本，其中 12628 次 ShaLimit；输出 metric 误标帧提交，实际仅 CPU 策略提交 |
| [writer.log](writer.log) / [reader.log](reader.log) | n12_crossproc 独立 write/read 进程，687 字节；额外解析日志比较 H25/H35 相等 |
| [kof.log](kof.log) / [kof-source.rs](kof-source.rs) / [kof-probe.rs](kof-probe.rs) | 当前 KOF 示例临时副本，仅增加拒绝/outcome 诊断和合法空池恢复断言；seed 11 首局超时，后两局真实终局；空池 panic 被 catch_unwind 捕获，进程退出 0 |

## 复跑独立探针

在项目根目录先 cargo test --offline -p engine-native，再用匹配构建库的 rustc。
本轮使用 /opt/homebrew/bin/rustc 1.98.0；如果本机有多个工具链，必须与该 rlib 一致。
以下 Python 只编译证据里的两个临时程序，不改生产源码：

```python
from pathlib import Path
import subprocess
root = Path.cwd()
evidence = root / 'docs/operations/evidence/2026-09-22-native-review-13'
lib = max((root / 'target/debug/deps').glob('libengine_native-*.rlib'), key=lambda p: p.stat().st_mtime)
for name in ('probes', 'kof-probe'):
    output = '/tmp/native-review13-' + name
    subprocess.run(['/opt/homebrew/bin/rustc', '--edition=2021', str(evidence / (name + '.rs')),
                    '--extern', 'engine_native=' + str(lib), '-L', 'dependency=' + str(root / 'target/debug/deps'),
                    '-o', output], check=True)
    subprocess.run([output], check=True)
```

KOF 副本保留原运行逻辑；增加的 diagnostic 输出不改变策略、轮换或胜负规则。
空池 panic 行是预期捕获的缺陷证明，不是未捕获的验证失败。后续修复后须反转相应反例断言并加入工程回归。

## 适用范围

- 同种子跨实例与恢复旧令牌、观星答案脱敏、头部规则版本/长度、坏将槽/重复牌原反例已关闭。
- 新增身份测试没有真实的中途恢复后续跑；事件摘要不是完整事件，见主稿。测试总数不作为完整正确率。
- n09 本轮仅 counting 一次，未重跑 plain；旧轮版本 24 结果保留在 [上一轮目录](../2026-09-22-native-review/README.md)。
- 同宿主两进程不是跨平台；旧线浏览器未复验；CPU 时间不是 GPU 帧或真实输入延迟。
- ADR 0773 原声明的对照 README 未找到，本轮 [补充索引](../2026-09-22-n09-n12-load/README.md) 明确缺失情况。
