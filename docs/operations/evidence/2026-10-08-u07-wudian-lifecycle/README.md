# U07 玩点论杀（wudian）：整包验收（2026-10-08）——来源级第五包

U07 wudian（`upstream-noname/wudian@1.0.0`，14 卡 + 6 武将元数据 +
莫守接线）完成真实宿主全生命周期验收——**来源级第五包（5/37）**。

## 生命周期（`wudian_real_host_full_lifecycle`，g1_wudian.rs）

1. **独立安装+激活**：激活集含 `upstream-noname/wudian`。
2. **安装包→原生建局**：14 卡入目录 + 费诗莫守接线生效（目录元数据
   skills=`["moshou"]`）。
3. **对局中卸载**：`revoke_slot(deactivate=true)`。
4. **在局续玩（lock_pinned）**：恢复+提交继续，14 卡目录仍可用。
5. **重装守卫**：同 npack 再 install → `Revoked`。
6. **重新 import 被拒**。

## 验收口径

- **回归已加入**：`wudian_real_host_full_lifecycle`（真实包，tempdir
  隔离 store；含费诗莫守接线断言，扩展武将下标基线=
  STANDARD_GENERALS.len() 而非卡牌数——首跑修正）。
- **全量通过**：engine-native **398 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **边界**：技能内容面（其余 9 条声明转换：调兵遣将×2 完整语义、诱敌
  深入完整化、8 将技能）为 G3 前置、不阻生命周期验收。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
