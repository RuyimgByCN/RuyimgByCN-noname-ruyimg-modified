# `cards[].skills` 装载支持（第 53 项**声明侧**，2026-10-19）

- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 改动：`crates/content-native-loader/src/lib.rs`（**已跟踪**）新增 `cards[].skills` 支持
- 用例：`crates/engine-native/tests/nc08_loader_card_skills.rs`（**4/4 通过**）

## 1 改了什么

```rust
// cards[].skills：牌的宿主技能声明（非装备牌用；装备牌仍走 equip_skills）
for s in opt_array(c, "skills", "cards[].skills")? {
    let s = s.as_str()…?;                                   // 元素须为字符串
    let behavior = HostBehavior::from_key(s)
        .ok_or_else(|| invalid("cards[].skills", format!("未知宿主技能键 {s}")))?;
    let name = names::intern(s.to_string());
    if !host_skills.iter().any(|d| d.name == name) {         // 按名去重
        host_skills.push(HostSkillDecl { name, behavior });
    }
}
```

* 只负责**声明**（并入 `pack.host_skills`，由 `content/standard.rs:428` 注册入目录）；
* **授予**仍由内核侧决定，**不在装载器范围**（见第 53 项）。

## 2 验收（4 项）

```
DBG cards[].skills：host_skills = ["host:chenhuodajie_card"]
DBG cards[].skills 去重：同名声明出现 1 次（应为 1）
DBG cards[].skills 负例：is_err=true；错误=Some(InvalidField { field: "cards[].skills", … })
DBG swd：host_skills 214 项、去重后 214
```

| # | 判据 | 结果 |
|---|---|---|
| 1 | 声明 `host:chenhuodajie_card` ⇒ 进入 `pack.host_skills` 且 `behavior == ChenhuodajieCard` | ✅ |
| 2 | 同一键在两张牌上声明 ⇒ **只登记一次** | ✅ |
| 3 | **未知键** ⇒ `InvalidField { field: "cards[].skills" }`（**不静默忽略**） | ✅ |
| 4 | 真实包 `swd` 仍正常桥接，`host_skills` **214 项无重名** | ✅ |

## 3 为什么**还没**改包数据

给 `packages/extensions/shahai-yunchou-upstream-noname/assets/cards.json` 的 3 张 `chenhuodajie`
补 `"skills": ["host:chenhuodajie_card"]` 后，该技能**只会进入目录、不会被授予**
（授予侧 `kernel/equipment.rs`、`construction.rs` **未跟踪**）⇒ **行为面不会有任何变化**，
却会**改动目录内容**（影响计数类断言）。因此**与授予侧一并接入**更稳妥。

## 4 状态

**第 53 项仍未解除阻塞**：声明侧已就绪（本文件），**授予侧待解阻**。
解阻后需要的两步：①包数据补 `skills` 字段；②授予清单加 `ChenhuodajieCard`（或按持有/使用动态授予）。
