# 逐包「安装」矩阵：**41/41 包目录通过**（2026-10-19）

- 工作包：**NC08 / 八项第 1 项（安装）**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`apps/game-server/tests/nc08_install_matrix.rs`
- 原始输出：[run.txt](run.txt)（41 行 `DBG`）

## 1 判据（每包，**独立临时 root + 独立 store**）

1. 把**单个包目录**放进临时 root ⇒ `NativeContent::import()` **成功**；
2. `installed()` 中出现该包、**恰好 1 条**记录；
3. **再次 `import()`** ⇒ 安装记录数**仍为 1**（**幂等**）。

## 2 结果

```
DBG 安装矩阵：ai-tuning-upstream-liuli ✅ package_id=ai-tuning version=1.0.0 幂等 ✅
DBG 安装矩阵：boss-play-upstream-liuli ✅ package_id=boss-play version=1.0.0 幂等 ✅
…（共 41 行）
DBG 安装矩阵：通过 41/41 个包目录
```

| 判据 | 结果 |
|---|---|
| 单包 root 可安装 | ✅ **41/41** |
| 安装记录恰 1 条 | ✅ |
| **重复 import 幂等** | ✅ 全部 |
| 耗时 | **0.25 s** |

## 3 过程中发现的两点（**重要**）

### 3.1 `import` **不跟随 symlink**

首版用 `std::os::unix::fs::symlink` 造单包 root ⇒ **全部包都"安装 0 个包"**（`import()` 仍返回 `Ok`）。
改成**真复制**后 41/41 全部成功。

⇒ **部署注意**：若有人把包**软链**到包根目录，安装会**静默成功但装 0 个包**（`Ok` 空列表 ⇒ 无报错）。
**建议**（未做）：`import` 对「扫描到 0 个包但目录非空」给出**警告或错误**。

### 3.2 `package_id` 重名（复述）

`cardpile` 出现两次（`upstream-noname` / `upstream-liuli`，**namespace 不同**）、`boss-play`、`coin` 同理
⇒ **单看 `package_id` 无法区分**；本矩阵因**每包独立 store**，不受影响。

## 4 边界（**如实**）

1. 只覆盖 **安装 + 幂等**；
2. **未覆盖「卸载 / 回滚」**——`native.rs` **无 uninstall API**（已登记待办）；
3. **未覆盖「版本升级」**（由[升级矩阵](../2026-10-19-upgrade-matrix/README.md) 13/13 覆盖）；
4. 未覆盖 **并发安装** 与 **安装中断恢复**。
