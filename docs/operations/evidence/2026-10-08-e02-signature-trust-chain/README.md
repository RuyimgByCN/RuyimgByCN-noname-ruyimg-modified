# E02/C3 签名信任链与多级历史（2026-10-08）

P1/E02、C3 生命周期的两项剩余（旧版多级历史、签名/信任链）补齐：
`signature_trust_chain`（content-runtime 测试，合成包）。

## 信任链断言（`signature_trust_chain`）

1. **签名安装**：正确密钥 `sign_package` → `verify_package` 通过 →
   `install(verify_key)` 接受，`meta.signed=true`。
2. **篡改拒绝**：字节翻转后同签名 → install 拒。
3. **错钥拒绝**：攻击者密钥签名 → verify 失败 → install 拒。
4. **未签名拒绝**：`verify_key` 模式（allow_unsigned=false）下无签名
   → install 拒。
5. **多级历史**：v1/v2 同槽并存（`list_installed` 双版本保留）；
   `activate_version("2.0.0")` 切换激活集。
6. **撤销+回退**：`revoke_content_hash(v2, deactivate)` → v2 重装
   `Revoked`；v1 仍可 `activate_version` 回激活（历史回退）。

## 验收口径

- **回归已加入**：`signature_trust_chain`（content-runtime）。
- **全量通过**：engine-native **393 passed / 0 failed / 1 ignored**；
  content-runtime+engine-ai+loader **63 passed / 0 failed**（含新测试），
  DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **登记（产品化策略）**：密钥分发/轮换与吊销名单分发属宿主运营面；
  引擎/宿主只验 keyed-blake3 v1（domain `noname-next/package-sig/v1`）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（3 文件）。
