# 2026-09-24 P0 N11 字节恢复核验（tag 29 冲突修复 + A0 tag 检查）

本证据描述主稿 P0「N11 字节恢复」的修复与验收：修复 `archive.rs` 的
XiaoJi/Dismantle 编码 tag 冲突（S13）、按候选 A0 第 1/4 点与 A5 近期修复落地
tag 唯一性检查与枭姬挂起字节往返/负例，并给出全量原生测试通过证据。
上一轮证据：[回放与目录核验](../2026-09-24-replay-golden-v60/README.md)。

> 2026-09-24 审查补注（[审查修复证据](../2026-09-24-review-fixes/README.md)）：「格式兼容取舍」
> 中"XiaoJi 挂起旧字节在新表下显式 BadTag/EOF"不准确。旧字节仍按 tag 29（Dismantle）
> 错位解码，之后才在后续字段报错（实测 `UnexpectedEof` 或 `BadTag("slot", 3)`），且解码
> 不校验载荷是否读尽；更正已写入规则基准。`CardKind` 编码表无冲突的结论成立，但同批
> 发现未知牌种编号解码会 panic（待修）。

## 检查与结果

| 检查 | 结果 |
|---|---|
| 源码版本 | STANDARD_RULESET_VERSION=60、ARCHIVE_FORMAT_VERSION=2（均未递增，理由见下） |
| 基线校验 | `shasum -a 256 -c ../2026-09-24-replay-golden-v60/source-hashes.txt` 11/11 一致（开工时源码与该证据同源） |
| tag 冲突确认 | 编码侧 `MachineSnapshot::Dismantle` 与 `MachineSnapshot::XiaoJi` 均写 tag 29；解码侧 `29 =>` 双分支、XiaoJi 分支不可达（unreachable-pattern 警告）。全表静态提取：0–42 中仅 23/24 空闲、仅 29 冲突 |
| A0 检查（先失败） | 新增 `tests/a0_tag_registry.rs`：42 变体样例（match 穷尽守卫，新增变体漏登记即编译失败）+ 机器 tag 唯一性 + 编码→解码往返。在未修复源码上先失败：`XiaoJi 往返解码失败: UnexpectedEof`（29 冲突的实际危害——XiaoJi 只写 1 字节 pid、按 Dismantle 读 7 字节错位） |
| 修复 | XiaoJi 编码/解码 tag 29→43（表尾追加）；A0 检查转绿（42 tag 唯一、42 往返相等） |
| 枭姬往返与负例 | 新增 `xiaoji_machine_byte_roundtrip_and_rejects`（boundary_snapshot.rs）：XiaoJi 挂起字节往返（tag 43）；Dismantle 挂起字节往返（tag 29 语义不变）；机器 tag 改未分配 255（重算校验和）→ `BadTag("machine", 255)`；截尾 1 字节 → `UnexpectedEof` |
| 其余编码表 | `HostBehavior` 74 变体 `tag()` 唯一（A0 测试守护，match 穷尽）；`CardKind` 86 变体 ↔ 0..=85 编码双射经静态核查确认（编解码函数私有，未行为级测试化） |
| 定向测试 | `--test a0_tag_registry` 2/2；`--test boundary_snapshot xiaoji_machine_byte_roundtrip_and_rejects` 1/1 |
| 全量验收 | `cargo test --offline -p engine-native`（共享 target，标准命令）**341 通过 / 0 失败**，完整输出见 [native-tests-full.log](native-tests-full.log)；archive.rs 的 unreachable-pattern 警告已消失（engine.rs 另有 6 处 view_as 分支存量警告，非本批范围） |

## 格式兼容取舍（格式版本保持 v2）

- tag 29 的解码语义保持 Dismantle 不变：含 Dismantle 挂起的旧 v2 字节完全兼容
  （有往返测试锚定），故不递增 `ARCHIVE_FORMAT_VERSION`。
- XiaoJi 挂起的旧字节因字段错位从未可正确解码（旧行为：错读为 Dismantle 或
  UnexpectedEof），属本来损坏的数据；新表下此类字节显式 BadTag/EOF，不在兼容范围。
- 未分配 tag（23/24/255 等）解码拒绝并报 `BadTag("machine", t)`。
- 取舍已写入[规则基准](../../../architecture/native-rules-baseline.md)。

## 结论边界

- 已实现并验收：tag 冲突修复（XiaoJi=43）、A0 tag 唯一性检查（先失败→转绿）、
  枭姬挂起字节往返与错 tag/短输入负例、HostBehavior/CardKind 编码表核查；
  全量 341/0。
- A0 验收口径的「先失败」本轮达成（tag 冲突由检查在缺陷源码上捕获）。
- 不在本批范围：engine.rs 6 处 view_as 分支 unreachable-pattern 存量警告、
  A0 第 3 点 `SkillDefId` 只追加检查、A5 生成器条件方案（本轮仅一处漂移，
  未构成「反复出现」，不触发）、与上游规则等价验收、性能测量（本批无性能结论）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（shasum -a 256，相对 noname-next 根）。
