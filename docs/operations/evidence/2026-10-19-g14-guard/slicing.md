# 切片方式对计数的影响（**52 vs 51**，2026-10-19）

扫描「阶段过滤」时，**怎么切 `match` 分支**会改变结果。实测两种切法：

| 切法 | 规则 | 总数 | 差异 |
|---|---|---:|---|
| **A（段切片，当前脚本）** | 从 `HostBehavior::X =>` 到**下一个** `HostBehavior::` | **52** | 含 `Xiefang` 的 `Phase::Draw, begin=false` |
| **B（表达式切片）** | A 再截断到**第一个 `=>`** | **51** | **丢 `Xiefang`** |

## 谁对

**A 对** ✅ —— `Xiefang` 的上游确实是双触发（`Death` + **`phaseDrawAfter` 类**），
原生写成了**跨行 `||` 的多个 `matches!`**：

```rust
HostBehavior::Xiefang => matches!(event, EventData::Death{..})
    || matches!(event, EventData::Phase { phase: Draw, begin: false, .. })
```

**B 会在第一个 `=>` 后截断，把它整条丢掉** ❌

## 所以

- **「52 处阶段过滤」维持** ✅；
- **但这类计数对切片规则敏感** ⇒ **任何脚本必须写明切片规则**，否则换个人重算就是 51；
- **脚本单测里必须钉住条数**（`test_scans_52_phase_filters`），**条数一变就报错**，让人去看是「新增」还是「切片出了问题」。

## ✅ 已解：`BossBaiyi` 不是误报（第 647 轮）

它在 **`filtering/common.rs`** 里**另有一个真实臂**（`filtering.rs:669` 那条是**同名转发**）：

```rust
HostBehavior::BossBaiyi => {
    let armed = ctx.state.boss_stage().map(|s| s.wing_armed).unwrap_or(0);
    matches!(event, EventData::Phase { player, phase: crate::event::Phase::Judge, .. })
}
```

⇒ **同名的两个臂分处两个文件**（`filtering.rs` 转发 / `common.rs` 实体）——
**脚本按文件逐个扫，所以只认到 `common.rs` 那条** ✅ ⇒ **52 成立，无虚计** ✅
