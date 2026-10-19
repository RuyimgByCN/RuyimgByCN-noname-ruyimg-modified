# 宿主行为「触发面未接入」清单（`=> false` 块，2026-10-19）

- 工作包：**NC08 / 修复清单第 53 项**；规则版本 v322（工作树）/ 存档格式 v105
- 位置：`crates/engine-native/src/content/host/filtering/common.rs` 的 `filter_common`

## 发现

`filter_common` 的匹配臂中，**67 个**宿主行为被**硬编码为 `=> false`**：

```rust
HostBehavior::YunchouJieyan => false,
HostBehavior::YunchouYiesheng => false,
HostBehavior::WudianDiyjuntun => false,
…
HostBehavior::ChenhuodajieCard => false,      // ← 第 53 项（趁火打劫窗口）
HostBehavior::ChangeCardNature => false,      // ← 五行盘技能（样本 140 只验证了「授予」）
…
```

**对比**：同一匹配臂中**正常接入**的行为（如 `YunchouLuweiyan2 => u_matches_lw2(…)`、
`ShengkeCheck => matches!(…)`）⇒ 说明 **`=> false` 是「触发面未接入」的显式占位**，
而非「条件永不满足」的语义表达。

## 对第 53 项的意义（**根因确定**）

趁火打劫窗口未出现**不是**夹具问题、**也非**「仅本人可响应」的语义差异（第 387 轮已排除），
而是 **`ChenhuodajieCard` 处于 `=> false` 占位块** ⇒ **窗口永不触发**。

**附带**：`ChangeCardNature => false` 说明**五行盘技能（样本 140 只验证了「授予」）的触发面同样未接入**
⇒ 与该样本的「待办」一致（**非新缺陷，但提供了精确落点**）。

## 清单

完整 67 项见同目录 [`names.txt`](names.txt)（按出现顺序）。

## 意义与边界

1. **性质**：这是**系统性登记**（显式 `false`），**不是隐式 bug**——但**从产品视角即「未接入」**；
2. **影响面**：67 个行为覆盖运筹/玩点论杀/古剑/swd 等多来源的**窗口类**效果；
3. **边界**：本清单只统计 **`filter_common` 中的 `=> false`**；
   其他过滤文件（`filtering.rs` 等）与**未挂载**（包数据无 `skills` 声明）的情形**另计**。

## 待办

- [ ] 逐项判定：**应当接入**（修复）还是**登记暂缓**（补注释/文档）；
- [ ] 与**规则基准族表**、**修复清单**建立双向链接；
- [ ] 补一条**回归断言**：清单项数变化时提示复核（类似缺口回归测试）。
