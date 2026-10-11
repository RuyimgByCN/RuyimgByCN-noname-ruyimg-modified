# T8 子项：后端对照原型 + ADR 草稿（2026-10-11）

## 交付物

- 原型：`crates/content-compiler/examples/t8_backend_prototype.rs`（可复跑：`cargo run --offline -p content-compiler --example t8_backend_prototype`；成本档复跑加 `--release`）
- ADR 草稿（非定案）：[docs/adr/notes/draft-t8-backend-comparison.md](../../../adr/notes/draft-t8-backend-comparison.md)
- 样本：huanle-upstream-noname「酒尾」`huanle_jiuwei`（现状 `mapping: host:phase_end_recover_or_draw`，kernel.rs `PhaseEndRecoverOrDraw` 编译臂为候选 A 语义基准）

## 实测输出（本机，Apple Silicon，2026-10-11）

```
① 安装即数据：3 技能 DSL 编译为 npack 形态包（logic_hash 复现一致，rule_vm_abi=3）
② 模块 to_bytes/from_bytes 往返一致（npack 载体可执行）
③ 语义一致：受伤时回 1 / 满血摸 1，两分支均与 kernel 臂等价
④ 挂起/恢复：Paused→continuation JSON 往返→resume 落账所选分支
⑤ 内容锁：continuation 换模块（logic_hash 漂移）续跑被拒
⑥ 超限/回滚：fuel 截断死循环；trap 时排队效果整批丢弃（不留下半次效果）
⑦ 成本（本机初步测量，release 构建）：酒尾触发一次（含 verify）≈ 85 ns 均值（N=20000；解释器量级参考值，非验收口径）
```

debug 档同场景 ≈ 703 ns 均值（同 N）。全部断言通过（任一失败示例即 panic）。

## 候选 C（WASM）与候选 A 成本

- 候选 C：wasmtime 不在离线依赖图，**未实测**，仅桌评（见草稿对照表）。
- 候选 A：执行路径在 engine-native 编译臂内，本原型不依赖 engine-native，**未单独测量**——按工作区规则不写未测量对比数字。

## 与 C1 验收的对应

原型证明 rule-vm 后端（候选 B）的内核能力满足 C1 必须项且可用真实扩展技能对齐语义；C1 全量验收（无需重编译安装含新技能/牌的包的端到端）还差 engine-native 调度与效果提交管线接线、跨进程恢复、对抗性样本矩阵——已列入草稿「边界」节，属定案后的集成任务，不在本子项范围。
