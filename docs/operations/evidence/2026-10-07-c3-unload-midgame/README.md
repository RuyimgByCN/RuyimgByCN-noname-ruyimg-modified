# C3 卸载进行中对局核验（2026-10-07）

主稿 P1/E02、C3 行剩余场景「卸载进行中对局」闭环。前置批次：
[升级/回滚/依赖图](../2026-10-07-unfinished-batch/README.md)。

## 语义（引擎层固化，测试 `e03_c3_uninstall_during_active_game_lock_pinned`）

卸载只影响后续开局；进行中对局在内容锁下自洽：

1. **活局不受宿主卸载影响**：引擎持有 Arc 目录（`new_with_extensions`
   建局时合并并共享），宿主注册表（`ExtensionRegistry`）卸载无回写通道——
   扩展牌仍可查（`lf_` 前缀计数=1）、命令照常提交、`ruleset_version`
   与 `content_lock_id` 指纹不变。
2. **挂起对局恢复的锁绑定**：卸载后（标准目录）`Engine::restore` 对扩展
   对局存档返回 `RuleVersionMismatch`（拒绝）；带原包集合
   `restore_with_extensions` 恢复成功（存档字节自包含，不依赖注册表状态）。
   **宿主策略登记**：宿主须保留包字节直到无对局引用（引擎不感知宿主
   引用计数；产品化引用计数/延迟删除另批）。
3. **新开局不含已卸载扩展**：按卸载后集合建局（标准目录）无 `lf_` 牌。

## 测试结果（分开陈述）

- **代码已实现**：无新引擎代码——本切片为行为固化（现有 Arc 目录与锁
  绑定恢复语义的验收测试），引擎零改动。
- **回归已加入**：`e03_c3_uninstall_during_active_game_lock_pinned`
  （`e02_c3_lifecycle.rs`）。
- **定向通过**：e02_c3_lifecycle 2/2（含既有升级/回滚场景不回归）。
- **全量通过**：engine-native **358/0/1**（唯一 ignore=写轮眼规模诊断
  探针）、golden（replay_scenarios）全绿、content-runtime 32/0、
  lib `RUSTFLAGS=-D warnings` 通过。
- **规则/格式版本不变**（v61/v4）：无状态/编码/语义变化。

## 未做（照旧登记）

旧版多级历史（多级回退链）、签名/信任链产品化策略；宿主侧包字节
保留/引用计数策略的产品化另批。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（3 文件，相对 noname-next 根）。
