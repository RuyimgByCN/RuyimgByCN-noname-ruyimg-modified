# T12 G5 接力·第二批：8 个 G1 转换包（2026-10-14）

## 任务来源

[U06 欢乐卡牌 G5 切片](../2026-10-14-t12-huanle-g5/README.md) 模式泛化：为其余已有 G1 生命周期的转换包逐包补 G5「同内容锁真双进程存档接力 + 错版/缺包拒绝」。

## 交付（crates/engine-native/tests/t12_g5_relays.rs 新增 8 测）

共享 `g5_relay(package_id, pack_name, test_name)` 助手：父进程 `new_with_extensions` 建局 → `advance_once` 推进（挂起询问妥善应答/无询问 NextPhase）6 步 → NGSA 落盘 → 自启子进程（`T12_G5_ARCHIVE/REPLY/STORE` env 识别；`DEDUP` 高段偏移 +5 亿避键冲突）→ 同锁恢复（错版[版本+1]/缺包[纯标准 restore] 拒绝断言）→ `advance_once`×8 → 回写 → 父进程 `restore_with_extensions` 校验 revision 前进。

**覆盖 8 包**：shaai-ow、shaai-xianjian（武将包）、shahai-hearth、shahai-gwent、shahai-yunchou、shahai-swd、shahai-mtg、wuxing。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | **8/8** 绿（每包=父建局/落盘+子恢复/错版缺包拒绝/续推 8 步/回写校验） |
| engine-native 全量 | **682/0/0**（674→682，新增 8 测） |
| Gate 1b | 独立 target `--all-targets` **exit 0**（首跑抓出 1 条 unused import，已清）——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 坑位实录

- 子进程 dedup：父/子同测试二进制、同静态计数器初值——子进程必须高段偏移（+5 亿），否则应答/推进全按重复命令拒绝。
- 驱动循环必须经 `advance_once`（挂起询问优先妥善应答）：裸 NextPhase 在 ChoicePending 下被拒。
- pack.def.name 来自 cards.json pack 段=**目录名**（非 package_id；d3 的 package_id=d3-select）；d3 的 assets 无 cards.json（武将装载面 bridge_generals）——与 ow/xianjian 实际走 cards.json 不同，d3 移出本批登记为武将装载面切片。

## G5 面覆盖进度

U06 欢乐+8 包=**9 包 G5 完成**（15 G1 转换包中）；余：cardpile×2、shahai-gujian、wudian、zhenfa、d3（武将装载面）。
