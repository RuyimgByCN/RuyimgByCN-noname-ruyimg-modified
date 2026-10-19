# 目录名双前缀**回归防线**（2026-10-19）

- 工作包：**NC08 / 数据规范**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc08_card_name_prefix_regression.rs`

## 结果

```
DBG 双前缀回归：检查 19 个有 card_prefix 的包；登记白名单 3 个；违规 0 项
```

## 口径（**登记式**，不因现状失败）

| 规则 | 行为 |
|---|---|
| **白名单内**（`shahai-mtg` 10 / `shahai-swd` 89 / `shahai-zhenfa` 3） | 断言**不超出上限**（防止扩大） |
| **白名单外** | 断言**双前缀 = 0**（防止新增） |

⇒ 当前 **0 违规** ✅；**若将来其它包出现双前缀** ⇒ **测试失败并列出包名与数量** ⇒ 提示修正**桥接
（`content-native-loader`）或包数据**，再更新白名单。

## ✅ 修复已落地（同日）

`crates/content-native-loader/src/lib.rs`（**已跟踪**）的 `bridge_pack`：

```rust
// 目录名 = `<card_prefix><name>_<i>`；若包内名已含该前缀则不重复叠加
let def_name = if name.starts_with(meta.card_prefix.as_str()) {
    names::intern(format!("{}_{}", name, i))
} else {
    names::intern(format!("{}{}_{}", meta.card_prefix, name, i))
};
```

**效果**：`swd_swd_huanglinzhicong_73` → **`swd_huanglinzhicong_73`**（探针复核 ✅）；
**白名单已清空**，本用例现断言「**目录名中不得出现 `<prefix><prefix>`**」，实测 **19 包 0 违规** ✅。

**安全性依据**：`engine/hash.rs` 只使用 `cards.len()` 与 `card_kind`，**不使用卡名**
⇒ 改名**不影响状态摘要/存档/golden**；全量 **1504 passed / 0 failed / 22 ignored** ✅。

## 为何当初用「登记式」口径

桥接层的**前缀去重**尚未实现（属 `content-native-loader` 面），当前 **3 包**确有双前缀；
**直接断言全 0 会立即失败**，反而掩盖「是否新增」这一关键回归信号 ⇒ 采用**登记式**口径。

## 待办

- [ ] **桥接层去重**：`card_prefix` 已包含于卡名时不再叠加（或在包数据置空 `card_prefix`）
- [ ] 去重实现后**收紧白名单至 0**
- [ ] 若目录名用于界面呈现，确认双前缀是否外露
