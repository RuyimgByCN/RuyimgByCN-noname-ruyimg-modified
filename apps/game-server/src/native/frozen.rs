//! 原局包副本：保留原始内容锁与全部声明资产，恢复不依赖当前启用版本。

use super::{ExtensionPack, NativeContent};
use content_runtime::{content_hash_hex, LockedPackage};
use content_schema::ContentPackage;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

/// 已安装并验证的原局完整内容；VM 模块包含在原始 package JSON 中。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrozenNativePackage {
    pub binding: LockedPackage,
    pub package_json: String,
    pub assets: BTreeMap<String, Vec<u8>>,
}

fn reference_binding(reference: &str) -> Result<LockedPackage, String> {
    let (name, version) = reference.rsplit_once('@').ok_or("无效的原生包引用")?;
    version.parse::<u32>().map_err(|_| "无效的原生包版本")?;
    let identity = name
        .strip_prefix("installed:")
        .ok_or("旧原生包引用缺少安装内容锁")?;
    let binding: LockedPackage =
        serde_json::from_str(identity).map_err(|e| format!("无效的安装内容锁: {e}"))?;
    for value in [&binding.namespace, &binding.package_id, &binding.version] {
        let mut components = Path::new(value).components();
        if value.is_empty()
            || value.contains('\\')
            || !matches!(components.next(), Some(Component::Normal(_)))
            || components.next().is_some()
        {
            return Err("无效的安装包身份".into());
        }
    }
    Ok(binding)
}

impl FrozenNativePackage {
    fn parsed_package(&self) -> Result<ContentPackage, String> {
        if content_hash_hex(self.package_json.as_bytes()) != self.binding.content_hash {
            return Err("原局包 content hash 不匹配".into());
        }
        let package = ContentPackage::from_json_bytes(self.package_json.as_bytes())?;
        package.validate()?;
        content_runtime::check_engine_compat(
            &package.manifest,
            &content_runtime::tooling_host_versions(),
        )
        .map_err(|e| e.to_string())?;
        let manifest = &package.manifest;
        if manifest.namespace != self.binding.namespace
            || manifest.package_id != self.binding.package_id
            || manifest.version != self.binding.version
            || blake3::Hash::from(manifest.logic_hash).to_hex().as_str() != self.binding.logic_hash
        {
            return Err("原局包身份或 logic hash 不匹配".into());
        }
        Ok(package)
    }

    pub(super) fn package(&self) -> Result<ContentPackage, String> {
        let package = self.parsed_package()?;
        let mut declared = BTreeMap::new();
        if let Some(manifest) = &package.asset_manifest {
            for entry in &manifest.entries {
                let relative = package
                    .asset_paths
                    .get(&entry.asset_id)
                    .ok_or("资产路径缺失")?;
                if entry.source_hash == [0u8; 32] {
                    return Err(format!("原局包资产 hash 为零: {relative}"));
                }
                if let Some((hash, kind)) =
                    declared.insert(relative, (entry.source_hash, entry.kind))
                {
                    if hash != entry.source_hash || kind != entry.kind {
                        return Err(format!("原局包资产路径冲突: {relative}"));
                    }
                }
                let bytes = self
                    .assets
                    .get(relative)
                    .ok_or_else(|| format!("原局包资产缺失: {relative}"))?;
                if blake3::hash(bytes).as_bytes() != &entry.source_hash {
                    return Err(format!("原局包资产 hash 不匹配: {relative}"));
                }
            }
        }
        if declared.len() != self.assets.len() {
            return Err("原局包含未声明资产".into());
        }
        Ok(package)
    }

    pub(super) fn text_asset(&self, relative: &str) -> Result<Option<String>, String> {
        self.assets
            .get(relative)
            .map(|bytes| {
                String::from_utf8(bytes.clone()).map_err(|e| format!("原局包资产不是 UTF-8: {e}"))
            })
            .transpose()
    }
}

impl NativeContent {
    pub(super) fn check_not_revoked(&self, binding: &LockedPackage) -> Result<(), String> {
        let revoked = self.store.read_revoke().map_err(|e| e.to_string())?;
        if revoked.is_hash_revoked(&binding.content_hash)
            || revoked.is_slot_revoked(&binding.namespace, &binding.package_id)
        {
            return Err(format!(
                "原局包已撤销: {}/{}@{}",
                binding.namespace, binding.package_id, binding.version
            ));
        }
        Ok(())
    }

    pub(super) fn freeze_binding(
        &self,
        binding: &LockedPackage,
    ) -> Result<FrozenNativePackage, String> {
        self.check_not_revoked(binding)?;
        let packages_root = self
            .store
            .root()
            .join("packages")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let root = packages_root
            .join(format!("{}__{}", binding.namespace, binding.package_id))
            .join(&binding.version)
            .canonicalize()
            .map_err(|e| format!("原局安装包缺失: {e}"))?;
        if !root.starts_with(&packages_root) {
            return Err("安装包路径越界".into());
        }
        let checked_read = |relative: &str| -> Result<Vec<u8>, String> {
            let path = root
                .join(relative)
                .canonicalize()
                .map_err(|e| format!("原局包文件缺失 {relative}: {e}"))?;
            if !path.starts_with(&root) || !path.is_file() {
                return Err(format!("安装包文件越界: {relative}"));
            }
            std::fs::read(path).map_err(|e| e.to_string())
        };
        let mut frozen = FrozenNativePackage {
            binding: binding.clone(),
            package_json: String::from_utf8(checked_read("package.json")?)
                .map_err(|e| e.to_string())?,
            assets: BTreeMap::new(),
        };
        let package = frozen.parsed_package()?;
        for relative in package.asset_paths.values() {
            frozen
                .assets
                .insert(relative.clone(), checked_read(relative)?);
        }
        frozen.package()?;
        Ok(frozen)
    }

    /// 为旧精确引用补齐原始副本；不把当前启用的新版本当作旧版本。
    pub fn freeze_reference(&self, reference: &str) -> Result<FrozenNativePackage, String> {
        let binding = reference_binding(reference)?;
        let frozen = self.freeze_binding(&binding)?;
        self.load_frozen_reference(reference, &frozen)?;
        Ok(frozen)
    }

    /// 只从原局副本桥接；副本损坏时不退回当前包存储。
    pub fn load_frozen_reference(
        &self,
        reference: &str,
        frozen: &FrozenNativePackage,
    ) -> Result<ExtensionPack, String> {
        if reference_binding(reference)? != frozen.binding {
            return Err("房间引用与原局包内容锁不匹配".into());
        }
        let loaded = self.load_frozen(frozen)?;
        if format!("{}@{}", loaded.pack.def.name, loaded.pack.def.version) != reference {
            return Err("房间引用与原局包版本不匹配".into());
        }
        Ok(loaded.pack)
    }

    /// 启动恢复统一入口：新档严格验副本，旧档只迁移精确可验证的安装版本。
    pub fn load_room_packages(
        &self,
        references: &[String],
        frozen: Option<&[FrozenNativePackage]>,
    ) -> Result<(Vec<ExtensionPack>, Vec<FrozenNativePackage>), String> {
        let copies = match frozen {
            Some(copies) => copies.to_vec(),
            None => references
                .iter()
                .map(|r| self.freeze_reference(r))
                .collect::<Result<Vec<_>, _>>()?,
        };
        if copies.len() != references.len() {
            return Err("房间引用与原局包副本数量不一致".into());
        }
        let mut slots = BTreeSet::new();
        let mut packs = Vec::with_capacity(copies.len());
        for (reference, copy) in references.iter().zip(&copies) {
            if !slots.insert((&copy.binding.namespace, &copy.binding.package_id)) {
                return Err("原局包槽位重复".into());
            }
            packs.push(self.load_frozen_reference(reference, copy)?);
        }
        Ok((packs, copies))
    }
}
