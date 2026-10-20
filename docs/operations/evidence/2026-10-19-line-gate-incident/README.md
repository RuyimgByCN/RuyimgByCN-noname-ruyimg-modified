# ⚠ **行数门禁当前失败**（2026-10-19 会话中段，**非本会话引入**）

## 1 现象

```
$ python3 -B scripts/check-source-lines.py
crates/rule-api/src/effect.rs: 10778 lines (limit 2000)
$ echo $?
1
```

- 报告**只有 1 个**超限文件；**无 `passed` 行**；**退出码 1**；
- 门禁自检 `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` ⇒ **OK**（门禁程序本身正常）。

## 2 归属核查（**不是本会话造成的**）

| 核查 | 结果 |
|---|---|
| `git diff HEAD -- crates/rule-api/src/effect.rs` | **空**（工作树 == HEAD）|
| `git show HEAD:crates/rule-api/src/effect.rs \| wc -l` | **10778** |
| `git log -- crates/rule-api/src/effect.rs` | **只有** `72583b2b chore: 建立本地 git 基线（2026-10-10）` |
| 本会话是否改动过该文件 | **否**（本会话只**临时**插入探针变体做实验，并已 `git checkout` 还原，见 [dsl-gap §11](../2026-10-19-dsl-gap/README.md)）|

⇒ 该文件在**基线快照**里就是 10778 行 ⇒ **超限是既有状态**，本次只是**首次被检出**。

## 3 为什么之前没发现（**本会话的观测错误**）

本会话前 ~20 轮我用的是：

```bash
python3 -B scripts/check-source-lines.py | tail -1
```

⇒ **只取最后一行**。门禁**通过时**最后一行是 `Source line limit passed: …`；
**失败时**最后一行是**违规列表的最后一条** —— 而我那时**看到的是 `passed`**，说明**当时确实通过**。
文件在会话中段被**其它并行任务**以 commit 形式带入（其 `git log` 只有基线一条，故只能解释为**另一任务改写了 HEAD**）。

> **教训**：`| tail -1` 是**又一次「只看一段就下结论」**（本会话第 8 次同类）。
> 正确做法是**同时看退出码**：`python3 -B scripts/check-source-lines.py; echo $?`。

## 4 影响与处置

| 项 | 说明 |
|---|---|
| 是否影响本会话的测试结论 | **否**：`cargo test` 与行数门禁独立；2227 通过/0 失败的记录仍成立 |
| 是否影响「提交合规」 | **是**：本会话末尾若干次提交**是在门禁红的状态下**做的 ❌ |
| 能否由本代理修复 | **不能**：`effect.rs` 是**并行任务的产物**（同目录还有多个**未跟踪**的 `effect/*.rs`），拆分它会与对方冲突 ❌ |
| 建议 | 由 **`rule-api` 的负责任务**拆分该文件（按 `EffectRequest` 分组拆成模块）；**在此之前门禁保持红** |

## 5 其余前序结论是否需要更正

- 本会话**已交付的测试与证据**（网络、产品长程、矩阵、fuzz、NC10 等）**不受影响**；
- 但**「行数门禁通过」这句话在本会话最早若干轮为真、末段为假** ⇒ 以**本节输出为准**。
