//! 原局包副本：保留原始内容锁与全部声明资产，恢复不依赖当前启用版本。

use super::{ExtensionPack, NativeContent};
use content_native_loader::{prepare_locked_content_with_bridge, verify_locked_content_with_bridge};
use content_runtime::{ContentLock, FrozenPackage, LockedPackage, VerifiedLockedContent};
use content_schema::ContentPackage;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

/// 已安装并验证的原局完整内容；共享 `content_runtime::FrozenPackage` 形状（R05 v4/v5）。
pub type FrozenNativePackage = FrozenPackage;

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
        let package_json =
            String::from_utf8(checked_read("package.json")?).map_err(|e| e.to_string())?;
        let raw_pkg = ContentPackage::from_json_bytes(package_json.as_bytes())?;
        let mut assets = BTreeMap::new();
        for relative in raw_pkg.asset_paths.values() {
            assets.insert(relative.clone(), checked_read(relative)?);
        }
        let frozen = FrozenNativePackage {
            binding: binding.clone(),
            package_json,
            assets,
        };
        frozen.validate_and_parse()?;
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

    /// 新局唯一采集时刻（R05 v4 §2）：从一次一致的 `current_lock` 读取选定包及其完整依赖闭包，
    /// 冻结原始 `package.json` 字节与声明资产并经共享 `prepare_locked_content_with_bridge` 验证。
    pub fn freeze_room_selection(
        &self,
        namespace: Option<&str>,
        package_ids: &[String],
    ) -> Result<VerifiedLockedContent, String> {
        if package_ids.is_empty() {
            return Ok(VerifiedLockedContent::empty());
        }
        let lock = self.store.current_lock().map_err(|e| e.to_string())?;
        let packages = self
            .store
            .load_locked_packages(&lock)
            .map_err(|e| format!("装载安装副本失败: {e}"))?;
        let mut needed_slots = BTreeSet::new();
        let mut queue = Vec::new();
        for pid in package_ids {
            let matched: Vec<&ContentPackage> = packages
                .iter()
                .filter(|p| {
                    p.manifest.package_id == *pid
                        && namespace.map(|ns| p.manifest.namespace == ns).unwrap_or(true)
                })
                .collect();
            let parsed = match matched.len() {
                0 => return Err(format!("包未安装或未激活: {pid}")),
                1 => matched[0],
                n => {
                    return Err(format!(
                        "包 id 在 {n} 个命名空间下存在，请显式指定命名空间: {pid}"
                    ))
                }
            };
            let slot = (
                parsed.manifest.namespace.clone(),
                parsed.manifest.package_id.clone(),
            );
            if !needed_slots.insert(slot.clone()) {
                return Err(format!("原局请求重复包槽位: {}/{}", slot.0, slot.1));
            }
            queue.push(parsed);
        }
        while let Some(pkg) = queue.pop() {
            for dep in &pkg.manifest.dependencies {
                let dep_slot = (dep.namespace.clone(), dep.package_id.clone());
                if needed_slots.insert(dep_slot.clone()) {
                    let dep_pkg = packages
                        .iter()
                        .find(|p| {
                            p.manifest.namespace == dep.namespace
                                && p.manifest.package_id == dep.package_id
                                && p.manifest.version == dep.version
                        })
                        .ok_or_else(|| {
                            format!(
                                "缺少激活依赖: {}/{}@{}",
                                dep.namespace, dep.package_id, dep.version
                            )
                        })?;
                    queue.push(dep_pkg);
                }
            }
        }
        let mut copies = Vec::with_capacity(needed_slots.len());
        for (ns, pid) in &needed_slots {
            let binding = lock
                .packages
                .iter()
                .find(|p| &p.namespace == ns && &p.package_id == pid)
                .ok_or_else(|| format!("安装包不在内容锁中: {ns}/{pid}"))?;
            copies.push(self.freeze_binding(binding)?);
        }
        prepare_locked_content_with_bridge(copies)
    }

    /// 房间/存档恢复的共享验证入口（R05 v4 §2/§3 & v5）：
    /// - 若存在 `canonical_lock`，则必须同时存在匹配的 `frozen` 副本并通过 `verify_locked_content_with_bridge`；
    /// - 若旧档缺 `canonical_lock`，仅允许按已验证 `frozen` 或精确 `installed:{LockedPackage}@v` 引用重冻并校验；
    /// - 全部副本均须通过撤销表检查（`check_not_revoked`）。
    pub fn verify_room_locked_content(
        &self,
        references: &[String],
        canonical_lock: Option<&ContentLock>,
        frozen: Option<&[FrozenNativePackage]>,
    ) -> Result<VerifiedLockedContent, String> {
        let copies = match (canonical_lock, frozen) {
            (Some(_), None) => {
                return Err("房间状态含 canonical_locked_content 但缺少 frozen_packages".into());
            }
            (_, Some(copies)) => copies.to_vec(),
            (None, None) => references
                .iter()
                .map(|r| self.freeze_reference(r))
                .collect::<Result<Vec<_>, _>>()?,
        };
        if !references.is_empty() && copies.len() != references.len() {
            return Err("房间引用与原局包副本数量不一致".into());
        }
        let mut slots = BTreeSet::new();
        for copy in &copies {
            if !slots.insert((&copy.binding.namespace, &copy.binding.package_id)) {
                return Err("原局包槽位重复".into());
            }
            self.check_not_revoked(&copy.binding)?;
        }
        if !references.is_empty() {
            for (reference, copy) in references.iter().zip(&copies) {
                if reference_binding(reference)? != copy.binding {
                    return Err("房间引用与原局包内容锁不匹配".into());
                }
                let _ = self.load_frozen_reference(reference, copy)?;
            }
        }
        match canonical_lock {
            Some(lock) => verify_locked_content_with_bridge(lock, &copies),
            None => prepare_locked_content_with_bridge(copies),
        }
    }

    /// 启动恢复统一入口：新档严格验副本，旧档只迁移精确可验证的安装版本。
    pub fn load_room_packages(
        &self,
        references: &[String],
        frozen: Option<&[FrozenNativePackage]>,
    ) -> Result<(Vec<ExtensionPack>, Vec<FrozenNativePackage>), String> {
        if let Some(copies) = frozen {
            if copies.len() != references.len() {
                return Err("房间引用与原局包副本数量不一致".into());
            }
        }
        let verified = self.verify_room_locked_content(references, None, frozen)?;
        let mut packs = Vec::with_capacity(verified.copies().len());
        for copy in verified.copies() {
            packs.push(self.load_frozen(copy)?.pack);
        }
        Ok((packs, verified.copies().to_vec()))
    }
}
