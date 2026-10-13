# T12 G5 接力·第四批：cardpile liuli 仓（双 namespace 切片收尾，2026-10-14）

## 任务来源

[第三批](../2026-10-14-t12-g5-batch3/README.md) 登记的「cardpile liuli 仓接力随双 namespace 切片」——本批完成，G5 接力覆盖面在 G1 转换包内收敛。

## 交付（t12_g5_relays.rs 追加 `t12_g5_relay_cardpile_liuli`）

`bridge_pile(pile.json)` → `extension_pack_from_pile`（def 名 `cardpile-upstream-liuli`，160 张比例包）→ 父进程建局/推进 6 步/落盘 → 子进程（env 识别，dedup 高段偏移）同锁恢复（错版[版本+1]/缺包[纯标准 restore] 拒绝断言）→ 推进 8 步 → 回写 → 父进程 `restore_with_extensions` 校验 revision 前进。与 noname 仓切片（第三批）构成双 namespace 完整覆盖。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | t12_g5_relays **14/14**（13 既有+1 本批）——[日志](relays-targeted.log) |
| engine-native 全量 | **688/0/0**（687→688）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 收敛口径

G1 转换包 G5 接力覆盖：**16 包全部达成**（huanle + 第二批 8 + 第三批 5 + 本批 liuli；cardpile 双仓各计 1，两仓合计覆盖 15 转换包目录+双仓 cardpile）。剩余非机械面：各包 G2–G4 产品通道（目录/搜索/资源/界面，随产品面批）与 E06 其余非 G1 转换来源的内容面（逐批）。
