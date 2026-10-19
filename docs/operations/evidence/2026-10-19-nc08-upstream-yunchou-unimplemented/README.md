# NC08 第⑦层样本 79：**来源 U05（运筹）三项未实现技能登记**（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（登记型样本）：上游**确有**三项技能定义；原生**均未实现**

## 1 上游（逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/`：

| 上游键 | 译名 | 上游依据 |
|---|---|---|
| `diyjianxiong` | 奸雄·变体 | `skill.js`：`mode: ["identity","guozhan"]` + `trigger: { global: "dieBefore" }` + filter（当前回合角色为持技者；身份局死者**非主公**）+ content 广播改身份并亮明 |
| `xinfu_qiaosi` | 巧思 | `skill.js:489` + `translate.js`：「出牌阶段限一次，你可以**投掷一枚六面骰子**，亮出牌堆顶的 X 张牌并获得之。然后选择一项：1.交给一名其他角色 X 张牌。2.弃置 X 张牌。（X 为骰子点数）」 |
| `xinfu_jingxie` | 精械 | `character.js:71`：马钧技能列为 `["xinfu_jingxie", "xinfu_qiaosi"]`（定义在 `character/mobile/skill.js`） |

## 2 原生（全库检索）

```
grep -rli "jianxiong|jingxie|qiaosi" crates/engine-native/src/   → 0 命中
```

（原始输出：[native-scan.txt](native-scan.txt)）⇒ **三项均未实现**；
样本 `nc08_upstream_yunchou_unimplemented.rs` 同时断言 `behavior.rs`/`keys.rs` **不含**
`Diyjianxiong`/`Jingxie`/`Qiaosi`（**实现后请翻转该断言**）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **实现三项技能**（并入修复清单第 32 项）
- [ ] **骰子机制**：`xinfu_qiaosi` 依赖「六面骰子」——原生是否已有该机制**待单独核对**
- [ ] `diyjianxiong` 需**身份局/国战**模式上下文（原生身份局已有 `new_identity`）
