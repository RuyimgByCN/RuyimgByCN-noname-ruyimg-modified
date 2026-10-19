# NC08 第⑦层样本 69：**来源 U05（杀海拾遗·运筹）** 武将技 `xiongzi`（雄姿）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面）**，**未发现差异**（含**向下取整**边界）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:143`：

```js
xiongzi: {
    trigger: { player: "phaseDrawBegin" },        // 自己摸牌阶段开始
    forced: true,                                  // 锁定
    content() { trigger.num += 1 + Math.floor(player.countCards("e") / 2); },   // +1+⌊装备数/2⌋
},
```

## 2 验收结果（`nc08_upstream_yunchou_xiongzi.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `1 + Math.floor(countCards("e")/2)` | **行为面**：装备 **0/2/3** 件 → 摸牌阶段净增 **3/4/4** 张（3 件仍 4 张 ⇒ **向下取整** ⌊3/2⌋=1 成立） | 通过 |
| 1 | `phaseDrawBegin` + `forced` | 原生 `yunchou.rs:8` 的 `1 + equips.len() / 2`（整数除法）与上游式一致；注册/过滤见 `behavior.rs:619`（tag 243）、`filtering/common.rs:507` | 通过 |

## 3 夹具要点（**重要，避免误判**）

运筹的**武将面在 `character.json`**（`cards.json` **只有卡牌段**，23 张卡）→ 必须**双产物桥接**：

```rust
let json = read("…/shahai-yunchou-upstream-noname/assets/character.json");
let mut pack = content_native_loader::bridge_pack(&json)?.pack;
pack.generals = content_native_loader::bridge_generals(&json)?.generals;
Engine::new_with_extensions(seed, 4, &[&pack])
```

否则 `debug_grant("ycnxiongzi")` 会 **panic `unknown skill`**。
另：宿主技能在**目录中的名字**为 `ycn`+拼音（`ycnxiongzi`），包面向 `mapping` 为 `host:xiongzi`。

> **诚实记录**：本样本初版因**只读了 `cards.json`**，误判为「运筹武将技不可达」并写入修复清单第 29 项；
> 同轮查明 `character.json` 后**已撤回**该判定（见修复清单第 29 项「已撤回」）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[diagnostic.txt](diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 运筹其它武将技（厉焰/雅量/蝉缘变体/悲歌 refresh 变体）的行为面
- [ ] `audio` 表现面
- [ ] 样本 70+：其余来源/卡牌
