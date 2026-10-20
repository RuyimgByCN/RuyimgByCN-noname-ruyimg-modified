# 金色一致性套件（`crates/conformance`）**全通过**（2026-10-19）

- 工作包：**NC11/NC12（验收数据）+ 产品级回归**；规则版本 v322（工作树）/ 存档格式 v105
- 入口：`cargo run --offline -p conformance`（CI 入口，`run_all()`）
- 原始输出：[run.txt](run.txt)（含编译告警）；纯结果：[run-clean.txt](run-clean.txt)

## 1 结果

```
engine_build_id=…
passed=19
  OK package_jianxiong_repro
  OK dual_vm_standard_m1
  OK guozhan_gz_jianxiong_repro
  OK guozhan_wei_batch_repro
  OK guozhan_other_batch_repro
  OK guozhan_batch_all_repro
  OK guozhan_zhulian_bihe_abi641
  OK huanle_shoulijian_abi642
  OK shahai_impl_repro
  OK gwent_dudayuanshuai_abi643
  OK shahai_hupeng_abi644
  …（共 19 条）
ALL PASS
```

| 判据 | 结果 |
|---|---|
| 退出码 | **0** |
| 通过场景 | **19 / 19** |
| 失败 | **0** |
| 结论行 | **`ALL PASS`** |

## 2 覆盖面（从场景名可见）

| 家族 | 场景 |
|---|---|
| 标准/国战复现 | `package_jianxiong_repro`、`guozhan_{gz_jianxiong,wei_batch,other_batch,batch_all}_repro`、`guozhan_zhulian_bihe_abi641` |
| VM/双实现 | `dual_vm_standard_m1`、`shahai_impl_repro` |
| 扩展包 ABI 断言 | `huanle_shoulijian_abi642`、`gwent_dudayuanshuai_abi643`、`shahai_hupeng_abi644` |

⇒ 覆盖**标准规则、国战、VM 双实现、三个扩展包的 ABI 断言**，且**逐场景给出 ID**，可追溯。

## 3 意义与边界（**如实**）

1. 这是**金色一致性套件**（golden conformance），**属于产品级回归的一部分** —— 但它**不等于**：
   - 完整产品长程（见[产品 soak](../2026-10-19-product-soak/README.md)、[WS 整局](../2026-10-19-network-path/README.md)）；
   - 37 来源逐项验收（见[五来源签署表](../2026-10-19-five-source-signoff/README.md)）；
2. **未覆盖**：真机/浏览器、TLS/跨机、性能；
3. 运行时有 **`engine-core` 未使用导入告警**（不影响结果，**未改动**——属其它任务工作树）。

## 4 复现

```bash
cd noname-next
cargo run --offline -p conformance
```
