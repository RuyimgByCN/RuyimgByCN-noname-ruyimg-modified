# 整包验收首包：欢乐卡牌真实宿主全生命周期（2026-10-08）

U06 欢乐卡牌升 G3 的最后一项（独立安装/卸载全生命周期在真实宿主）
验收完成——**整包验收 1/37**。验收对照：

| 完成条件 | 证据 |
|---|---|
| 功能 | E03 六卡宿主行为全链路（2026-10-07 载荷对照+酒尾补齐+猴子切片）+ U06 跨牌组合矩阵 |
| 配置 | 门控通道（config.shengjian 等宿主 config 读取） |
| 依赖 | C3 依赖冲突/级联激活原子性（content-runtime `c3_dependency_conflict`） |
| 组合 | 窃取×据守×面具/苦无×伤害来源负例矩阵 |
| 保存恢复 | 手里剑嵌套选择字节恢复 + 本批锁定副本续玩 |
| 规则差异 | 苦无弃牌选择/酒尾可选等已登记 |
| 独立安装/卸载 | **本批**：`huanle_real_host_full_lifecycle` |

## 生命周期六步（真实宿主 PackageStore）

1. **独立安装+激活**：`import_extensions`（真实发现→安装→激活集含
   `community/huanle-cards`）。
2. **安装包→原生建局**：安装产物 `assets/cards.json` → C0 桥接 →
   `Engine::new_with_extensions`（目录 7 卡）。
3. **对局中卸载**：`revoke_slot(ns, pkg, reason, deactivate=true)` —
   撤销表登记 + 移出激活集。
4. **在局续玩（lock_pinned）**：对局存档 `restore_with_extensions`
   （引擎持有已装载副本）→ 提交继续、扩展目录 7 卡仍可用——宿主撤销
   不影响在局对局。
5. **重装守卫**：撤销后同 npack 再 `install` → `RuntimeError::Revoked`。
6. **重新 import 被拒**：`import_extensions` 内部 install 撞撤销守卫。

## 验收口径（分开陈述）

- **回归已加入**：`huanle_real_host_full_lifecycle`（g1_huanle.rs，
  真实包 `packages/extensions/huanle-upstream-noname`，tempdir 隔离
  store）。
- **全量通过**：engine-native **388 passed / 0 failed / 1 ignored**；
  依赖包 **62/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **边界**：本验收=引擎/宿主能力面；G3（目录/搜索/资源/界面部件）
  按 G2–G4 通道另行分配；「整包 1/37」的计数口径=按台账 37 来源逐包
  完成条件（本批首包）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
