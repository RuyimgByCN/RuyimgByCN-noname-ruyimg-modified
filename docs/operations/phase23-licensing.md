# 阶段 23：法律许可审查基础架构（§42）

> **历史阶段记录**（T16④ 补注，2026-10-11）：本文为旧线阶段存档，「当前/已完成/待做」类陈述仅反映撰写时点；现行状态以[进度主稿](../architecture/native-progress.md)、[覆盖清单](../native-coverage-manifest.md)与[规则基准](../architecture/native-rules-baseline.md)为准。

> 对应架构计划 v2.0 §42（法律、许可与内容来源）。没有明确权利的资源不得
> 进入官方发行包。

## 目标

为每个内容包建立完整的许可元数据，涵盖 §42 要求的全部审查维度：

1. 代码许可审查（§42: 代码许可）
2. 美术、音频、字体和文本来源审查（§42: 资源许可 + 来源）
3. 商标和名称使用审查（§42: 商标）
4. 可再分发范围（§42: 可再分发范围）
5. 修改记录（§42: 修改记录）
6. 官方发行门控：`Unspecified` 许可的包不得进入官方发行

## 新增 crate

`crates/licensing-schema` — 许可类型、来源元数据、再分发范围、验证逻辑。

### 类型

```rust
pub enum LicenseType {
    Mit, Apache2, Gpl3, Agpl3,
    Bsd2Clause, Bsd3Clause,
    CcBy4, CcBySa4, CcByNc4, Cc0,
    PublicDomain,
    Custom(String),
    Unspecified,  // §42: 不允许进入官方包
}

pub enum RedistributionScope {
    FreeRedistribution,  // 官方 + 社区
    OfficialOnly,        // 仅官方
    NonCommercial,       // 非商业
    NoRedistribution,    // 不可再分发
}

pub struct AssetSource {
    pub asset_group: String,
    pub author: String,
    pub source_url: Option<String>,
    pub license: LicenseType,
    pub modified: bool,
    pub modification_notes: Option<String>,
}

pub struct LicenseRecord {
    pub code_license: LicenseType,
    pub default_asset_license: LicenseType,
    pub asset_sources: Vec<AssetSource>,
    pub redistribution: RedistributionScope,
    pub trademark_notes: Option<String>,
    pub changelog: Vec<ChangelogEntry>,
}
```

### 验证逻辑

```rust
/// 官方发行验证：Unspecified 许可或 NoRedistribution → 拒绝
pub fn validate_for_official(record: &LicenseRecord) -> LicenseValidationResult;

/// 社区发行验证：仅检查再分发权限
pub fn validate_for_community(record: &LicenseRecord) -> LicenseValidationResult;
```

## PackageManifest 集成

`PackageManifest` 新增 `license: LicenseRecord` 字段（`#[serde(default)]` 向后兼容）。

每个内容包的 manifest 现在携带：
- 代码许可（`code_license`）
- 资源许可（`default_asset_license` + 每资源组 `asset_sources`）
- 作者和来源（`AssetSource.author` + `source_url`）
- 可再分发范围（`redistribution`）
- 修改记录（`changelog`）
- 商标注记（`trademark_notes`）

## §42 审查清单 → 实现

| §42 要求 | 字段/方法 | 状态 |
|---|---|---|
| 项目代码许可审查 | `code_license` | ✅ |
| 美术音频字体来源审查 | `asset_sources` + `AssetSource` | ✅ |
| 资源许可 | `default_asset_license` + per-source `license` | ✅ |
| 作者和来源 | `AssetSource.author` + `source_url` | ✅ |
| 商标和名称审查 | `trademark_notes` | ✅ |
| 可再分发范围 | `redistribution: RedistributionScope` | ✅ |
| 修改记录 | `changelog: Vec<ChangelogEntry>` | ✅ |
| 官方发行门控 | `validate_for_official()` | ✅ |
| 用户生成内容政策 | `validate_for_community()` + scope 枚举 | ✅ |

## 测试

```bash
cargo test -p licensing-schema   # 7 tests
```

| 测试 | 断言 |
|---|---|
| `mit_license_allows_redistribution` | MIT 允许再分发；Unspecified 不允许 |
| `spdx_ids_are_stable` | SPDX 标识符稳定（MIT, Apache-2.0, CC-BY-4.0） |
| `unspecified_license_rejected_for_official` | Unspecified 代码许可 → 官方发行拒绝 |
| `fully_licensed_package_passes_official` | MIT + CC-BY + 来源 + changelog → 通过 |
| `no_redistribution_rejected_for_community` | NoRedistribution → 社区发行拒绝 |
| `license_record_round_trip` | JSON 序列化往返 |
| `asset_source_with_modification` | 修改过的资源记录 | 
