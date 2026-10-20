# L08 / L25 的「疑似规则」分组判定（2026-10-19）

## 1 分类脚本口径改进（第 502 轮）

首版权重只认 `ui.`/`document.`/`playertip`/`style.` 等，**漏了动画与聊天**：
`_HpRecover`（2526 B，`animation_new.js`）在首版被判成**规则**，实际是 **回血骨骼动画特效**。
已补 token：`animation` / `skeleton` / `scale:` / `frame` / `as_showImage` / `player.say` / `addChatWord` / `setTimeout`。

### 1.1 重跑结果

| 来源 | 总定义 | 去重 | **规则** | UI | 标记 | 其它 |
|---|---:|---:|---:|---:|---:|---:|
| L08 十周年UI | 332 | 329 | **51**（原 66）| 101 | 169 | 8 |
| **L14 手杀ui** | 92 | 87 | **0** ✅ | **87** | 0 | 0 |
| L25 祖安武将 | 244 | 182 | **86**（原 117）| 92 | 2 | 2 |

⇒ **一个重要交叉验证**：脚本口径改进后，**L14 自动变成「规则 0」**，与我[上一轮逐条人工判定的结论](l14-8-blocks.md)**完全一致** ✅
（8 个疑似中 6 贴图 + 2 聊天）。这说明**特征表补全后，脚本结论可信度显著提高**。

## 2 L08 的 66 个疑似按特征分组

| 组 | 数量 | 样例 |
|---|---:|---|
| **无 UI 特征**（最可能是规则） | **33** | `_HpRecover`(2526B) —— 实为**动画特效** |
| **标记** | 28 | `wangong2`(582B) |
| 日志 | 3 | `_recasting`(4088B) |
| 日志 + 标记 | 2 | `zhenyi_spade`(2423B) |

## 3 **确证：L08 存在真规则** ✅

`wangong2`（`markskill.js`，582 B）：

```js
lib.skill.wangong2 = {
  trigger:{player:'useCard1'}, forced:true, popup:false, firstDo:true, charlotte:true,
  content:function(){ player.removeSkill('wangong2');
    if(trigger.card.name=='sha') trigger.baseDamage++; },
  mod:{ cardUsable:function(card){ if(card.name=='sha') return Infinity; },
        targetInRange:function(card){ if(card.name=='sha') return true; } },
  mark:true,
}
```

⇒ **改动牌局**：使用 **杀** 时 **基础伤害 +1**、**无限次使用**、**无视距离** ⇒ **这是真规则** ✅
（虽然带 `mark:true`，但 `mod` 与 `content` 改的是牌局语义，不是 UI）。

## 4 结论（**如实**）

| 来源 | 结论 |
|---|---|
| **L14 手杀ui** | **规则技能 0** ⇒ **不迁移** ✅（脚本与人工一致）|
| **L08 十周年UI** | **有真规则**（已确证 `wangong2`）⇒ **是迁移来源**；**51 个疑似待逐条判定** |
| **L25 祖安武将** | **86 个疑似待逐条判定**（未抽读到确证的真规则，**也未排除**）|
| **L11 / L12** | 0（台账 G0，**未本轮复核**）|

⇒ **待逐条判定范围：L08 51 + L25 86 = 137**（从 183 缩小）；
⇒ **仍不能给出「待迁移技能总数」** —— 需要逐条读完 137 个块。

## 5 口径声明

1. 「规则」仍是**文本特征分类**（补全 token 后），**不等于人工判定**；
2. 本轮**只抽读了 2 个块**（`_HpRecover` 动画、`wangong2` 真规则）+ 上一轮 8 个（L14）；
3. **未读** L08 其余 49 个、L25 全部 86 个。
