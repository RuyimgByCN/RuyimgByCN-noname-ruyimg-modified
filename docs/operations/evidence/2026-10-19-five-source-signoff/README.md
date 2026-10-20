# 五个有生命周期证据来源的「八项」签署状态（2026-10-19）

- 工作包：**NC08 / 扩展台账**；规则版本 v322（工作树）/ 存档格式 v105
- 范围：**仅 L04、U01、U03、U05、U07**（有 `*lifecycle*` 证据的五个来源，见
  [口径核对](../2026-10-19-g-lifecycle-vs-counted/README.md)）

## 1 表格（✅ 有证据 / ⚠ 部分 / ❌ 无 / — 不适用）

| 来源 | 安装 | 升级 | 配置 | 组合 | 保存恢复 | 目标平台 | 规则差异 | 性能 |
|---|---|---|---|---|---|---|---|---|
| **L04**（cardpile-liuli） | ✅ [lifecycle](../../2026-10-08-l04-cardpile-liuli-lifecycle/README.md) | ✅ **`nc08_upgrade_five_sources`** | ✅ `nc08_config_acceptance` | ✅ `nc08_composition_pair_matrix` | ✅ **单进程（`pile.json` 管线）** `nc08_save_restore_pile` | ❌ | ⚠ 台账行 | ❌ |
| **U01**（d3） | ✅ [lifecycle](../../2026-10-08-u01-d3-lifecycle/README.md) | ✅ **`nc08_upgrade_five_sources`** | ✅ 同上 | ✅ 同上 | ✅ **单进程**（`nc08_save_restore_four_sources`） | ❌ | ⚠ 台账行 | ❌ |
| **U03**（cardpile） | ✅ [lifecycle](../../2026-10-08-u03-cardpile-lifecycle/README.md) | ✅ **`nc08_upgrade_five_sources`** | ✅ 同上 | ✅ 同上 | ✅ **单进程（`pile.json` 管线）** `nc08_save_restore_pile` | ❌ | ⚠ 台账行 | ❌ |
| **U05**（杀海拾遗，5 子包） | ✅ [lifecycle](../../2026-10-08-u05-gwent-lifecycle/README.md) 等 5 份 | ✅ **`nc08_upgrade_five_sources`** | ✅ 同上 | ✅ 同上 | ✅ `t12-g5-batch2`（六子包，**双进程**）+ `nc08_save_restore_four_sources`（单进程） | ❌ | ✅ 13 族 + 缺口清单 | ❌ |
| **U07**（wudian） | ✅ [lifecycle](../../2026-10-08-u07-wudian-lifecycle/README.md) | ✅ **`nc08_upgrade_five_sources`** | ✅ 同上 | ✅ 同上 | ✅ **单进程**（同上用例） | ❌ | ⚠ 台账行 | ❌ |

## 2 逐项说明（**口径**）

| 项 | 证据形态 | 覆盖范围 |
|---|---|---|
| **安装** | 各来源的 `*lifecycle*` 目录（安装→建局→卸载/重装） | 5/5 有 |
| **升级** | **无**逐来源升级用例（通用用例 `nc08_upgrade_acceptance` 用 swd 包，**不等于**来源级签署） | 0/5 |
| **配置** | `nc08_config_acceptance`（**19 包**元数据/开关/`init_hook`/非法数据） | 5/5 **通用**覆盖 |
| **组合** | `nc08_composition_pair_matrix`（**10 卡牌包 45 对**） | 5/5 **通用**覆盖 |
| **保存恢复** | `t12-g5-batch2`（双进程）+ `nc08_save_restore_four_sources`/`nc08_save_restore_pile`（**单进程往返**） | **5/5**（U05 双进程；U01/U03/U07/L04 单进程，**U03/L04 走 `pile.json` 管线**） |
| **目标平台** | **无**（无设备） | 0/5 |
| **规则差异** | 13 族 + 修复清单 + 缺口清单 | 仅 U05 较完整 |
| **性能** | `nc12_bench`（**引擎级基线**，Apple M3 Max / rustc 1.95 / release）：建局 p50 9.7 µs（none）–78.7 µs（swd）、快照 417 ns、恢复 62.7 µs、**整局 268 步 14.2 ms**（rejected=0） | **1/5 部分**（仅 U05/swd 有数据；**非前后对比、非产品级**）|

## 3 结论（**如实**）

1. **5 个来源中，无一个完成八项**（每来源至少缺 升级/目标平台/性能 三项）；
2. **配置与组合**是**通用用例**覆盖（**不是**逐来源签署）——**严格意义上**仍需在台账按来源勾选；
3. **保存恢复**仅 **U05 的 6 个子包**有真双进程接力证据；
4. ⇒ 与[八项验收清单化](../2026-10-19-eight-item-acceptance/README.md)的「**0 来源完成**」**一致**。

## 4 下一步（**可做**）

- [ ] 为 5 个来源**逐项在台账勾选**（配置/组合用通用用例，注明「通用覆盖」）；
- [ ] 补 **U01/U03/U07/L04** 的**保存恢复**（可复用 `t12-g5-batch2` 的接力骨架）；
- [ ] **升级**：为每个来源构造**旧包→新包** fixture（`nc08_upgrade_acceptance` 已给模板）。
