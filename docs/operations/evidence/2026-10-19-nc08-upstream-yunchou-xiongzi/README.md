# NC08 第⑦层样本 69：**来源 U05（杀海拾遗·运筹）** 武将技 `xiongzi`（雄姿）——**不可达，如实登记**（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（源码事实）+ 1 项未验收（不可达登记）**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:143`：

```js
xiongzi: {
    trigger: { player: "phaseDrawBegin" },        // 自己摸牌阶段开始
    forced: true,                                  // 锁定
    content() { trigger.num += 1 + Math.floor(player.countCards("e") / 2); },   // +1+⌊装备数/2⌋
},
```

## 2 结果

| # | 判据 | 结果 |
|---|---|---|
| 1 | 原生 `content/host/step/yunchou.rs:8` 的 `1 + equips.len() / 2`（整数除法=向下取整）与上游式一致；注册/过滤见 `behavior.rs:619`（tag 243）、`filtering/common.rs:507` | 通过（**源码事实**） |
| 2 | **行为面**：`debug_grant(0, "雄姿")` → **panic `unknown skill`**（见 [diagnostic.txt](diagnostic.txt)）→ 用例 `#[ignore]` | **未验收（登记）** |

## 3 根因：**运筹武将技在产品路径上不可达**

- `packages/extensions/shahai-yunchou-upstream-noname/assets/cards.json`：**只有 23 张卡、无 `generals` 段**
  （卡牌条目仅 `{name, kind, suit, rank}`）；
- 全库扫描：**没有任何包**的 cards/generals 携带 `host:` 键（宿主行为经**卡名**映射，
  如 `name.contains("shijianhuisu")`）；
- 宿主侧 `HostBehavior::YunchouXiongzi` 等（tag 241–252+）**已实现行为**，但**无包数据交付**
  → 与「激将不可达」同类，**成因不同**：**包数据缺武将面**（不是缺少调用点）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed / 1 ignored**（行为面**未构成验收**） | [targeted.log](targeted.log)、[diagnostic.txt](diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **补齐运筹（及同类来源）的武将面**：包数据需交付 `host:yunchou` 系技能（或宿主按 tag 注册技能名）
- [ ] 运筹其它武将技（厉焰/雅量/蝉缘变体/悲歌 refresh 变体）的行为面
- [ ] 该「**宿主已实现但包未交付**」模式是否波及其它来源（**建议全来源普查**）
