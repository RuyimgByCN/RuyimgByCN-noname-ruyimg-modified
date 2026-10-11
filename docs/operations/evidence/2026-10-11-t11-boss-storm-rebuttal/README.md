# 2026-10-11 T11 「Suspended 风暴」勘误：测试脚手架假象，引擎无缺口

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `e8416213`；v249 批登记的引擎待查项根因调查）。**结论：撤销该待查登记**——风暴为测试脚手架假象，引擎分片续行无缺口。

## 根因调查

- **逐 resume 探针**（60 步逐 outcome 打印）：形态轮换后驱动 0/1/2/3 全回合——每 resume 均有进展（`Accepted`/`Pending` 交替：判定→摸牌[年瑞+2]→出牌→弃牌询问[按 hp 应答]→结束→下家），**全程零 `Suspended`**。
- Yielded 的唯一产生点=`resolution.rs:338` 的 `steps >= slice`——`time_slice=None`（测试未设片）时**不可达**；`budget` 为 DEFAULT_BUDGET 大额上限，`BudgetExhausted` 走 `Rejected` 而非 Suspended。
- 早前「风暴」的真因：v249 首版回归草稿的循环推进存在三处脚手架缺陷——①`match next_phase` 缺 `Suspended`/`Rejected` 分支（编译期暴露）；②`open_choice` 未前置（挂起未先消费 → `next_phase` 撞 `Rejected(ChoicePending)`）；③迭代计数把「应答迭代」与「阶段推进迭代」混计（12 次上限被应答消耗，未到下一判定锚即退出，误读为「卡死」）。
- 轮换双向交替（round-2→忍性、round-3→合体、round-4→忍性…）为正确行为——首版断言锚定 round-3 期望忍性态，属断言锚定错误。

## 多回合回归补测（v249 遗留项关闭）

`boss_nianshou_nbianshen_rotation_renxing` 追加 ⑤ 段：轮换后驱动至 round-3 判定锚（60 步上限、救援 Pass/其余按牌应答、Suspended/Rejected 显式分支）——断言互斥轮换回年兽合体（slot 62）、hp 恒 12、存档往返+守恒。**通过**。

## 验证

| 项 | 结果 |
|---|---|
| 引擎改动 | 无（勘误+回归补强批） |
| 多回合回归 | 轮换后 round-2/3 全回合推进正常、round-3 互斥回切断言通过 |
| 全量 | **595 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本/存档/golden | 规则 249/存档 v59/14 golden 均不动（无语义变化） |

## 方法论登记

「引擎风暴」结论必须以**逐 resume 探针**（逐 outcome 打印）为证，不能以「测试循环未收敛」推断——测试循环的臂缺失/迭代混计/断言锚定错误都会伪装成引擎不收敛（本会话第三次此类误报：第一次=贪欲跳弃牌靠 drain 掩盖、第二次=急叮随条目装配假象、第三次=本批）。
