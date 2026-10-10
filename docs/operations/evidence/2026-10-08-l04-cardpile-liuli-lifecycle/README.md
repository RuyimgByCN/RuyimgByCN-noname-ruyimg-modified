# L04 cardpile（liuli 仓）：整包验收（2026-10-08）——来源级第三包

L04 cardpile（`upstream-liuli/cardpile@1.0.0`，与 U03 共用同一桥接
构建器与酒/雷杀规则）完成真实宿主全生命周期验收——**来源级第三包
（3/37）**。镜像 U03 的双仓同装场景，方向相反：单撤 liuli slot。

## 生命周期（`cardpile_liuli_real_host_full_lifecycle`，g1_cardpile.rs）

1. **双仓独立安装+激活**。
2. **liuli 仓安装产物→桥接→建局**（113 张唯一实体牌）。
3. **对局中卸载（单撤 liuli）**：`revoke_slot("upstream-liuli",
   "cardpile", deactivate=true)`——**noname 仓激活不受影响**。
4. **在局续玩（lock_pinned）**：liuli 仓 113 张锁定副本恢复+提交继续。
5. **重装守卫**：liuli npack 再 install → `Revoked`。
6. **重新 import 被拒**。

## 验收口径

- **回归已加入**：`cardpile_liuli_real_host_full_lifecycle`。
- **全量通过**：engine-native **395 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **上游等价边界**：倍率缩放/随机点数映射登记同 U03，不阻 G3。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
