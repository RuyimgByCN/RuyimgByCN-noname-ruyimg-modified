# U03 cardpile（noname 仓）：整包验收（2026-10-08）——来源级第二包

U03 cardpile（`upstream-noname/cardpile@1.0.0`，比例表 160→113 张唯一
实体牌+扩展酒/杀规则结算，第 18 轮纵向样本）完成真实宿主全生命周期
验收——**来源级第二包（2/37）**。场景特点：**双仓同装**（noname 与
liuli 两 namespace 同 slot 名 `cardpile`）。

## 生命周期（`cardpile_real_host_full_lifecycle`，g1_cardpile.rs）

1. **双仓独立安装+激活**：两 namespace 各自进入激活集与内容锁。
2. **安装产物→原生建局**：noname 仓 `assets/pile.json` → PileSpec →
   桥接 → 113 张唯一实体牌入目录。
3. **对局中卸载（单撤）**：`revoke_slot("upstream-noname","cardpile",
   deactivate=true)`——仅撤 noname slot；**liuli 仓激活不受影响**
   （namespace 隔离验证）。
4. **在局续玩（lock_pinned）**：113 张锁定副本恢复+提交继续。
5. **重装守卫**：noname npack 再 install → `Revoked`。
6. **重新 import 被拒**（全量导入撞撤销守卫）。

## 验收口径

- **回归已加入**：`cardpile_real_host_full_lifecycle`。
- **全量通过**：engine-native **394 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **上游等价边界**：倍率缩放（160 总量补足）与随机点数映射已登记为
  上游等价剩余（确定性流水号+建局洗牌承接），不阻 G3。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
