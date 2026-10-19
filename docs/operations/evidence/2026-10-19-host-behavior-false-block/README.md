# 宿主行为「触发面未接入」清单（`=> false` 块，2026-10-19）

- 工作包：**NC08 / 修复清单第 53 项**；规则版本 v322（工作树）/ 存档格式 v105
- 位置：`crates/engine-native/src/content/host/filtering/common.rs` 的 `filter_common`

## 发现

`filter_common` 的匹配臂中，**50 个**宿主行为被**硬编码为 `=> false`**（**注**：整文件 `grep "=> false,"` 得 **67** 处，其中 **17 处属于其它过滤函数**，不在本清单口径内）：

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

完整 **50** 项见同目录 [`names.txt`](names.txt)（按出现顺序，**仅 `filter_common`**）。

## 逐项定性（**同日完成**）

对 50 项逐一核对 `content/host/step/*.rs` 是否有**实现臂**：

| 类别 | 数量 | 说明 |
|---|---:|---|
| **有实现、无暂缓注释** | **49** | 效果代码存在，但 `filter_common` 恒 `false` ⇒ **触发面未接**（**非未实现**） |
| 有实现、带暂缓/登记注释 | 1 | 注释信号仅按「臂前 6 行」启发式提取，**可能漏检** |
| **无实现** | **0** | —— |

⇒ 结论：这批是**「已实现但触发面未接入」**，而不是「未实现」⇒
**修复工作量在于「接触发面」**（过滤器 + 包数据 `skills` 声明），**不是重写效果**。

**边界（**准确性**）**：注释检测采用「臂前 6 行」启发式，**可能漏掉更远处的登记注释** ⇒
「无注释」不等于「未登记」；判定**登记与否**应以**修复清单/规则基准**为准。

## 双向链接与**精确定性**（同日完成，**修正**「50 项触发面未接」的过宽表述）

把 50 项与**修复清单**、**规则基准**逐一对照，并按实现注释是否属「**内联/无独立效果步**」细分：

| 分类 | 数量 | 判读 |
|---|---:|---|
| **修复清单已含** | **22** | 已登记 ✅ |
| **规则基准已含** | **9** | 已登记 ✅ |
| **未见于清单但属「内联/无独立效果步」** | **8** | `filter_common => false` **合理**（效果在**别的机器内联**实现，如 AOE 目标段查询、杀机器内联消费、判定机内联）⇒ **非缺口** ✅ |
| **未见于清单的其余（真缺口候选）** | **11** | **需逐项定性**（见下） |
| 合计 | **50** | —— |

**11 项真缺口候选**（含注释）：`YunchouYiesheng`（野升）、`Shenwei`、`Hxunzhi2`、`Gxianyin`（革新印）、
`YxsMenshen2`（门神·标记持有，注释自述「主体 70 KB 效果**待批次**」）、`Shengong`、`TargetDiscardEquipOrDamage`、
`LinghunzhihuoCard`（灵魂之火，**已有样本 123**）、`ZhaomingdanCard`（照明弹，**已有样本 121**）、
`MoshouImmunity`、`ZhangbaSha`。

其中 **2 项已有行为面样本**（灵魂之火/照明弹）⇒ **清单登记缺失**（应在修复清单补录）；
其余 9 项**待逐项判定**「应当接入」或「登记暂缓」。

**修正说明**：上一轮「**50 项触发面未接入**」的表述**过宽**——其中 **39 项**（22+9+8）**已有登记或属合理内联**；
真正待处理的是 **11 项候选**（含 2 项清单补录）。

## 分派核对（**决定性**）：50/50 **委托 `filter_common`**

逐一核对 `filtering.rs` 的 `filter_behavior` 分派：

```
50 项中：委托到 filter_common 50 项；非 filter_common 分派 0 项
```

⇒ **全部 50 项的触发判断都走 `filter_common`**（如 `YunchouYiesheng => self.filter_common(…)`、
`ChenhuodajieCard => self.filter_common(…)`）⇒ 它们在该函数中恒为 `false` ⇒
**凡「需要触发」者，触发面确实永不生效**（**事实层**）。

**因此最终口径（三层，互不混用）**：

| 层 | 结论 | 数量 |
|---|---|---:|
| **事实** | 50 项**全部**委托 `filter_common` 且该处恒 `false` ⇒ **凡需触发者永不触发** | **50** |
| **性质** | 其中属「**无独立效果步/内联**」⇒ `false` **合理**（效果在别处），**非缺口** | **8** |
| **待办** | 需要触发但**未登记**者 | **11** |
| **已登记** | 修复清单 22 + 规则基准 9 | **31** |

⇒ **需要触发的 42 项（50−8）中，31 已登记、11 未登记** ⇒ **待处理 = 11 项**（口径与上一轮一致，
但 **`false` 恒不生效这一事实**本轮才被**分派核对**确证）。

## 11 项候选的**挂载核对**（决定性，收窄到 3 项）

对 11 项候选，核对**包数据是否声明**对应宿主键（`skills`/`cards` 的 `mapping`）：

| 行为 | host 键 | 包数据声明 |
|---|---|---|
| `Hxunzhi2` | `host:hxunzhi2` | ✅ **swd**（`hxunzhi2`） |
| `YxsMenshen2` | `host:yxs_menshen2` | ✅ **hero-kill**（`yxs_menshen2`；注释自述「主体 70 KB 效果**待批次**」） |
| `MoshouImmunity` | `host:moshou_immunity` | ✅ **wudian**（`moshou`） |
| `YunchouYiesheng` | `host:yiesheng` | ❌ **未声明** |
| `Shenwei` | `host:shenwei` | ❌ **未声明** |
| `Gxianyin` | `host:gxianyin` | ❌ **未声明** |
| `Shengong` | `host:shengong` | ❌ **未声明** |
| `TargetDiscardEquipOrDamage` | `host:target_discard_equip_or_damage` | ❌ **未声明** |
| `LinghunzhihuoCard` | `host:linghunzhihuo_card` | ❌ **未声明** |
| `ZhaomingdanCard` | `host:zhaomingdan_card` | ❌ **未声明** |
| `ZhangbaSha` | `host:zhangba_sha` | ❌ **未声明** |

⇒ **再收窄**：11 项中**仅 3 项被包数据挂载**（`hxunzhi2`/`yxs_menshen2`/`moshou`）⇒ 它们的
`filter_common => false` **真的会挡掉触发**；其余 **8 项压根未挂载** ⇒ **`false` 无关**
（属「**未挂载**」而非「**触发面未接**」，**另一类待办**）。

**旁证**：`LinghunzhihuoCard`/`ZhaomingdanCard` **未挂载卡窗口**，但二者**效果已实现**
（灵魂之火经 `skills_damage.rs:749`、照明弹经其弃牌选择路径；样本 121/123 已验证）
⇒ **与「未挂载」一致**（效果走别的路径，窗口未接）。

## 意义与边界

1. **性质**：这是**系统性登记**（显式 `false`），**不是隐式 bug**——但**从产品视角即「未接入」**；
2. **影响面**：67 个行为覆盖运筹/玩点论杀/古剑/swd 等多来源的**窗口类**效果；
3. **边界（**精确性**）**：本清单只统计 **`filter_common` 中的 `=> false`（50 项）**；整文件计数为 **67**（含其它过滤函数 17 处），**两者不可混用**；
   其他过滤文件（`filtering.rs` 等）与**未挂载**（包数据无 `skills` 声明）的情形**另计**。

## 待办

- [ ] 逐项判定：**应当接入**（修复）还是**登记暂缓**（补注释/文档）；
- [ ] 与**规则基准族表**、**修复清单**建立双向链接；
- [ ] 补一条**回归断言**：清单项数变化时提示复核（类似缺口回归测试）。
