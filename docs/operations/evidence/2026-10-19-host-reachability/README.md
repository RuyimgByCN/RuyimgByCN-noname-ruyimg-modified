# 宿主行为键**可达性扫描**：520 键 / 370 被引用 / **150 未被引用**（2026-10-19）

- 工作包：**NC08 / 规则差异（可达性）**
- 脚本：`scripts/host_key_reachability.py`（**可复跑**）
- 输出：[summary.txt](summary.txt)、[unreachable.txt](unreachable.txt)（**150 条明细**）

## 1 方法

1. **`keys.rs` 里的全部 `"host:*"` 键**（520 个）；
2. **扫描所有 `packages/extensions/*/assets/*.json`** 里出现的 `"host:*"` 字符串（**任何字段、任何层级**）；
3. **差集 = 未被任何包数据引用的键**。

## 2 结果

```
宿主键 520 个；被包引用 370；不可达 150
```

**按前缀分组（前 10）**：

| 前缀 | 未引用数 |
|---|---:|
| `host:swd_*` | **30** |
| `host:gujian_*` | **23** |
| `host:gwent_*` | **15** |
| `host:mtg_*` | **13** |
| `host:jiguan_*` | **8** |
| `host:hearth_*` | **7** |
| `host:huanle_*` | 3 |
| `host:baiyi*` | 2 |
| `host:huanren*` | 2 |
| `host:lianyaohu*` | 2 |
| 其余（多为单条）| 45 |

## 3 ⚠ **这是上界，不是「150 条全部不可达」**

**方法只检查「包数据里的静态引用」** ❌ —— 以下情况会被**误判为不可达**：

| 情况 | 说明 |
|---|---|
| **运行时授予**（`addTempSkill`/`addSkill` 类）| 包数据里不出现，但内容执行时会授予 |
| **子技能由父技能派生** | 如 `xxx2`/`xxx3`/`xxxX` 后缀由 `content` 添加 |
| **卡牌/装备的运行时授技** | `cards[].skills` 为空但 content 里 `gainSkill` |

⇒ **150 是「静态未引用」的上界** ✅ —— **真实不可达数 ≤ 150，且需要逐条确认** ❗

## 4 目前**已确认**的不可达（8 条，手工核实）

见[规则差异交接页 §4.4](../2026-10-19-rules-diff-handler/README.md)（**U08 五技能 + Gwent 三状态技**）。

⇒ **本扫描把候选从 8 扩到 150** —— **值得逐条核**，**但不能直接当成 150 条缺陷** ❗

## 5 下一步（**未做**）

1. 对 **`swd_*` 的 30 条**（最大组）做**运行期核实**：跑一局 `swd`，看是否出现「授予了但静态未引用」的技能；
2. 检查**后缀派生规律**（`2`/`3`/`X`/`_state`）—— **若大量是派生，则真实不可达数会大幅下降**；
3. 把确认后的结果并入 §4.4。

---

## 6 **更正：漏了第三条挂载路径（`content/host.rs` 的 `CardKind → HostSkillDecl` 表）**

第一版脚本**只扫包数据的 `host:` 引用**，得出 **150 未引用** —— **过高** ❌。

**实际有三条挂载路径**：

| # | 路径 | 例子 | 数量 |
|---|---|---|---:|
| **1** | **包数据 JSON**（`skills[].mapping`、`generals[].skills[].mapping`）| `mapping: "host:yxs_fanji"` | **370** |
| **2** | **`content/host.rs` 的 `CardKind → HostSkillDecl` 表** | **`CardKind::Chunbing => behavior: HostBehavior::GujianFoodChunbing`** | **56** |
| 3 | 运行时授予（`addTempSkill` 等）| 未扫 | ❓ |

**并入第 2 条后**：

```
宿主键 520；包引用 370；host.rs 声明 56；并集 426；仍无引用 94（上界）
```

⇒ **150 → 94** ✅（**`gujian` 从 23 降到 6**，与「`content/host.rs` 大量声明卡牌宿主」一致）

## 7 ⚠ 94 **仍是上界**

- **运行时授予**（第 3 条路径）**未扫**；
- **`swd` 22 / `gwent` 10 / `jiguan` 8 / `hearth` 7** 这些**卡牌子包**是否还有别的挂载方式，**未核**；
- **真实不可达 ≤ 94**，**已手工确认的仍只有 8 条**（U08 5 + Gwent 3）。

## 8 教训

**「不可达」的判定必须枚举全部挂载路径** ❗ ——
**我第一版只查了包数据**，得出 150；**补上 `host.rs` 的表后降到 94**；
**如果还有第三条，还会再降** ⇒ **任何「不可达」结论必须写明「查了哪几条路径」** ✅。

---

## 9 **死键抽查：6 个样本键全部不在目录里**（实测，非推理）

用例 `crates/engine-native/tests/nc08_dead_host_keys.rs`：

```
DBG 死键抽查：装载 18 个包；样本 6 个；出现在目录里的 0 个 []
test result: ok. 1 passed
```

**抽查样本**（来自 94 条里）：
`host:baimang2`、`host:baimang3x`、`host:chiyan2`、`host:qingmu2`、`host:xuanlin2`、`host:zhaomingdan`

⇒ **装载全部包后，这 6 个键都不在 `catalog.skills` 里** ✅

### 为什么「不在目录里」= 死键

**运行时授予必须经目录查名** —— 例（`support.rs:1020`）：

```rust
pub(super) fn grant_shihuifen_ban(…) {
    if let Some(def) = ctx.catalog.skill_by_name(…) { ctx.core.skills.insert(…); }
}
```

⇒ **目录来自「包数据 + `host.rs` 的 `HostSkillDecl` 表」** ⇒
**两者都没有的键，运行时也查不到、授不了** ✅

### 因此

| 结论 | 证据 |
|---|---|
| **94 键既不在包数据、也不在 `host.rs` 表** | 静态扫描（§6）|
| **抽查 6 个都不在目录里** | 本用例 |
| **运行时授予要过目录** | `support.rs:1020` 的 `skill_by_name` |
| ⇒ **这 94 个键无法被授予** | **三者合起来** ✅ |

**边界**：抽查 **6/94**（**不是全部**）；**其余 88 条未逐个验证**。
