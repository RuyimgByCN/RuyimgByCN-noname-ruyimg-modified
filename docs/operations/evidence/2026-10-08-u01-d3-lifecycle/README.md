# U01 3D精选：整包验收（2026-10-08）——来源级第四包

U01 3D精选（`upstream-noname/d3-select@1.0.0`，角色包：扩展武将
ddd刘冲 入目录 + 精彀宿主行为，末焱登记暂缓）完成真实宿主全生命周期
验收——**来源级第四包（4/37）**。

## 生命周期（`d3_real_host_full_lifecycle`，g1_d3.rs）

1. **独立安装+激活**：激活集含 `upstream-noname/d3-select`。
2. **安装包→原生建局**：安装产物 `character.json` → C0 桥接 → 扩展
   武将接在基础表后（`general_meta` 可查，末焱 unmapped 登记不变）。
3. **对局中卸载**：`revoke_slot(deactivate=true)`。
4. **在局续玩（lock_pinned）**：恢复+提交继续，扩展武将元数据仍可查。
5. **重装守卫**：同 npack 再 install → `Revoked`。
6. **重新 import 被拒**。

## 验收口径

- **回归已加入**：`d3_real_host_full_lifecycle`（真实包，tempdir 隔离
  store）。
- **全量通过**：engine-native **396 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **边界**：技能内容面（32 将重写/末焱复合原语/立绘语音资源映射）为
  G3 前置、不阻生命周期验收；纵向行为（刘冲/精彀）第 21 轮已验收。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
