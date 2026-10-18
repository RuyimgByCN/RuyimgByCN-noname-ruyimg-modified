# NC08 第⑦层样本 19：`jizhi`（集智，**标准面** / 黄月英）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个标准面技能样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项验收通过**；其中 **2 项为差异登记（①属规则损失）**

## 1 上游源码（归档快照，逐字引用；见 [upstream-jizhi.js.txt](upstream-jizhi.js.txt)）

`archive/upstream/noname/apps/core/character/sb/skill.js`：

```js
jizhi: {
    trigger: { player: "useCard" },
    filter(event, player) { return get.type(event.card) === "trick"; },   // **任意锦囊**
    forced: true,                                                         // 锁定技
    async content(event, trigger, player) {
        const next = player.draw("nodelay");
        next.gaintag.add("sbjizhi");           // 摸到的牌打标记
        await next;
        player.addTempSkill("sbjizhi_mark");   // 本回合该牌**不可被弃置**
    },
    subSkill: { mark: { mod: { ignoredHandcard(…) / cardDiscardable(…) } } },
},
```

## 2 验收结果（`nc08_upstream_jizhi.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+3 | `forced` + `draw("nodelay")` | 使用【决斗】（**原生已覆盖**的 5 种之一）→ 摸 1 张且**无确认门** | 通过 |
| 2 | `get.type === "trick"`（**任意锦囊**） | **①覆盖范围差异（规则损失）**：原生 filter 仅列 5 种（南蛮/万箭/决斗/铁索/无懈）→ 使用【无中生有】**不触发**（手牌净 +1 而非 +2） | 通过（**差异登记①**） |
| 3 | `gaintag.add("sbjizhi")` + subSkill | **②标记面未实现**：原生无 `sbjizhi`/gaintag（上游摸到的牌本回合不可弃置） | 通过（**差异登记②**） |

**原生实现引用**：`content/standard/skills/ji_zhi.rs`（filter 仅 5 种锦囊；效果 `draw_cards(owner, 1)`）。

## 3 工程发现（写入证据）

- **标准面目录牌名为英文**：`duel`/`nullify`/`savage`/`chain`/`archery`/`wuzhong`/`tao`/`shan`/`sha`…（**不是拼音**
  `juedou`/`wuxie`，也**不是中文**）→ 夹具取牌须按目录名（本样本连续踩 `juedou`/`决斗`/`nanman` 三次）；
- **技能目录名为中文**（`集智` = `names::JI_ZHI`）→ `debug_grant` 用中文名；
- **档案路径**：测试位于 `crates/engine-native` → 归档相对路径为 `../../../archive/...`（三级）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **差异①（规则损失）修复**：`ji_zhi.rs` 的 filter 应改为「任意锦囊」（该文件**未跟踪**，
      属 host/standard 拆分 → 已并入待实施修复清单）
- [ ] **差异②（sbjizhi 标记面）**：是否实现（摸到的牌本回合不可弃）需口径判定
- [ ] 样本 20+：其余来源（当前 **17/37 来源**）
