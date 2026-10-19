# `pile.json` 管线的保存恢复（U03 / L04，2026-10-19）

- 工作包：**NC08 / 八项验收第 5 项**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc08_save_restore_pile.rs`

## 背景

`cardpile-upstream-noname`（U03）与 `cardpile-upstream-liuli`（L04）的资产是 **`pile.json`**
（牌堆补丁包），**不在 card/general 管线**，故此前的保存恢复用例跳过它们。
本用例走**该管线自己的 API**：`content::pile::extension_pack_from_pile(def, prefix, &spec)`（`pile.rs:56`，已跟踪）。

## 结果

```
DBG pile 保存恢复：cardpile-upstream-noname 构包成功，卡 113 张（source=noname entries=12）
DBG pile 保存恢复：cardpile-upstream-noname 往返成功
DBG pile 保存恢复：cardpile-upstream-liuli 构包成功，卡 113 张（source=noname-liuli entries=12）
DBG pile 保存恢复：cardpile-upstream-liuli 往返成功
```

| 包（来源） | 构包 | 往返（save→restore→hash 一致→可继续） |
|---|---|---|
| **`cardpile-upstream-noname`（U03）** | ✅ **113 张** | ✅ |
| **`cardpile-upstream-liuli`（L04）** | ✅ **113 张** | ✅ |

## 实现要点（**供复用**）

1. **`PileSpec` 未实现 `Deserialize`** ⇒ 从 `serde_json::Value` **手工构造**；
2. `pile.json` 的 `counts` 是**对象**，而 `PileEntry::counts` 是 **`Vec<(String,u32)>`** ⇒ 需**固定顺序**
   （`spade/heart/club/diamond`，其余字典序）以保持**确定性**；
3. 生成的卡名为 `{source}_{card}_{suit}_{i}`（`pile.rs` 文档），**跨来源唯一**。

## 边界

- **单进程往返**，弱于 U05 的双进程接力；
- 未覆盖 `scale_plan`/`snapshot_scale`/`restore_scale` 的**缩放到 160 张**面（另有 API，未测）。
