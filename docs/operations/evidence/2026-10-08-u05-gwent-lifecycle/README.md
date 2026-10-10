# U05 杀海拾遗·gwent 子包：真实宿主全生命周期验收（2026-10-08）

按[欢乐包整包验收](../2026-10-08-e03-package-acceptance/README.md)同
模板，对 U05 杀海拾遗的 **gwent 子包**（`upstream-noname/shahai-gwent
@1.0.0`，昆恩/山崩/破晓/天气群等 21 卡切片）执行真实宿主全生命周期
验收——**U05 子包验收第二例**（欢乐包为来源级首例 1/37；gwent 计入
U05 子包进度，非来源计数）。

## 生命周期六步（`gwent_real_host_full_lifecycle`，g1_shahai_gwent.rs）

1. **独立安装+激活**：import → 激活集含 `upstream-noname/shahai-gwent`。
2. **安装包→原生建局**：安装产物 cards.json → C0 桥接 →
   `Engine::new_with_extensions`（gwent 卡入目录）。
3. **对局中卸载**：`revoke_slot(deactivate=true)` → 撤销表+移出激活集。
4. **在局续玩（lock_pinned）**：恢复+提交继续，gwent 卡仍可用。
5. **重装守卫**：同 npack 再 install → `RuntimeError::Revoked`。
6. **重新 import 被拒**。

## 验收口径

- **回归已加入**：`gwent_real_host_full_lifecycle`（真实包，tempdir
  隔离 store）。
- **全量通过**：engine-native **389 passed / 0 failed / 1 ignored**；
  依赖包 **62/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **边界**：gwent 的规则差异（山崩随机改座位序前二/破晓分支一自选等）
  已在 v78/v79 登记对照；本验收=宿主能力面；U05 整包剩余=其余九子包
  同模板续验 + 角色/技能/模式/配置面。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
