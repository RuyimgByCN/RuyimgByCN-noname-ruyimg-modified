# 2026-10-06 原生开发进度复核

核验位置：`noname-next` 当前工作区，规则 v60、NGSA 格式 v2。当前状态与后续计划见[进度主稿](../../../architecture/native-progress.md)，逐来源扩展见[台账](../../../migration/upstream-extension-reconstruction.md)。本轮更新文档与证据，未修改产品源码/包数据。

## 方法与源码边界

- 当前应用任务列表未定位到对应的 noname-next 实施任务；本轮以落盘源码、现有证据和实跑核验交付进度，未读取另一个 agent 的内部工作日志。
- [上次审查](../2026-09-24-review-fixes/source-hashes.txt)登记的源码、包数据和 Cargo 文件共 29 项，本轮逐一计算 SHA-256，全部一致；未发现其后的原生实现增量。[audit-facts.json](audit-facts.json)保存对照结果与包数据数量。
- 对原生内核、装载器、content-runtime、转换包及 Cargo 文件共 106 项生成[source-hashes.txt](source-hashes.txt)，三组验收前后摘要未变化；[test-summary.json](test-summary.json)记录退出码、数量、命令耗时与变动检查。
- 工具链/宿主见[environment.txt](environment.txt)。命令耗时是本轮执行记录，不是优化前后对照或性能收益证据。

## 实跑结果

在 `noname-next` 根目录执行：

| 命令 | 结果 | 完整输出 |
|---|---|---|
| `cargo test --offline -p engine-native` | 345 passed / 0 failed；含一个核对 14 个 golden 场景的测试 | [engine-native-tests.log](engine-native-tests.log) |
| `cargo test --offline -p content-native-loader` | 1 passed / 0 failed | [content-native-loader-tests.log](content-native-loader-tests.log) |
| `cargo test --offline -p content-runtime` | 31 passed / 0 failed | [content-runtime-tests.log](content-runtime-tests.log) |
| `cargo test --offline -p engine-native --test replay_scenarios scenarios_are_registered_in_coverage_manifest` | 文档修改后补跑，1 passed / 0 failed | [coverage-registration.log](coverage-registration.log) |

本轮未重跑全工作区、前端、原生联机或目标设备验收；不把旧线成绩和子包转换数量计为原生完整规则/整包验收。

## 未修缺陷诊断

[archive-probes.rs](archive-probes.rs)在证据目录独立编译；二进制放临时目录，运行后删除，未添加到产品测试目录。[输出](archive-probes.log)的 3 个“通过”分别表示合法存档对照正常与两项缺陷成功复现，**不是解码器通过验收**。

| 探针 | 本轮观察 | 后续验收应改为 |
|---|---|---|
| 合法存档对照 | 编码→解码与原快照相等 | 继续保持 |
| 未知牌种编号 | 在 `declared_kinds` 把唯一牌种字节改为 255，重算校验和，仍 panic：`archive: unknown card kind 255` | 返回结构化错误，不 panic；其余 `use_as` 和两处 `need` 解码路径也覆盖 |
| 多余载荷字节 | 在载荷尾部插入 4 字节，更新长度与校验和后仍被接受，得到原快照 | 读完快照后存在剩余字节即拒绝 |

重现探针（先运行原生全量命令生成当前依赖产物）：

```python
from pathlib import Path
import subprocess, tempfile

base = Path("docs/operations/evidence/2026-10-06-native-progress-audit")
deps = Path("target/debug/deps")
def latest(pattern):
    return max(deps.glob(pattern), key=lambda p: p.stat().st_mtime)

with tempfile.TemporaryDirectory(prefix="noname-audit-") as temp:
    binary = Path(temp) / "archive-probes"
    subprocess.run([
        "rustc", "--edition=2021", "--test", str(base / "archive-probes.rs"),
        "--extern", f"engine_native={latest('libengine_native-*.rlib')}",
        "--extern", f"blake3={latest('libblake3-*.rlib')}",
        "-L", f"dependency={deps}", "-o", str(binary),
    ], check=True)
    subprocess.run([str(binary), "--nocapture", "--test-threads=1"], check=True)
```

## 上游与扩展复核

重跑[来源清点脚本](../../../migration/evidence/2026-09-23-extension-scope/inventory.py)：

```bash
python3 docs/migration/evidence/2026-09-23-extension-scope/inventory.py --workspace .. > /tmp/noname-extension-scope-current.json
diff -u docs/migration/evidence/2026-09-23-extension-scope/inventory.json /tmp/noname-extension-scope-current.json
```

本次[输出](extension-inventory.json)与基线逐字节一致：noname HEAD `5c02a745eb33b61b2e61e1167e1b71ca163911f6`、工作区干净；Liuli HEAD `8485dbb29ff16f9b258a766b1a17ef75f883aade`、50 个路径有本地修改，按工作区字节核验。来源仍为 8+29，共 37 项。

转换目录仍为 10 个；具体数量由 audit-facts.json 记录，台账统一维护。玩点论杀目前 14 张牌、6 名武将；11 条武将技能声明仅思继映射，10 条 `mapping:null`，与 `g1_wudian_generals_and_siji_discard_draw` 的断言一致。莫守宿主行为存在，但费诗的声明仍未映射，`moshou_blocks_delayed_trick_target` 使用 `debug_grant`；不能把宿主原语样本记为武将已装配。

装载器已收敛古剑、玩点论杀两份测试桥接；其余六条路径、五行初始化适配和正式产品调用未完成。`names::intern/intern_list` 复用同包重复装载，首次出现的唯一项仍永久驻留全局表；已有循环测试不证明卸载回收或任意新包总内存有界。

## 文档整理

本轮校正进度主稿、规则基准、覆盖清单、扩展契约、演进候选和台账；将重复的跨领域待办收敛到主稿，已完成的 v60 回放/目录/tag 单列，不再当作阻断。README/索引补已有装载器入口；清理记录见[旧实现参考](../../../architecture/legacy-reference.md)。本地链接/锚点与重复章节检查见[doc-checks.json](doc-checks.json)。
