//! Installed package assets cross the Worker boundary together with their exact manifest.
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::content::{names, pile::extension_pack_from_pile};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn valid_identity(value: &str) -> bool {
    !value.trim().is_empty()
        && value != "."
        && value != ".."
        && !value.contains('/')
        && !value.contains('\\')
}

fn required<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| valid_identity(value))
        .ok_or_else(|| format!("manifest.{field} 须为有效的身份字段"))
}

fn json(value: &Value) -> String {
    value.to_string()
}

// Validate original declaration text before Value can normalize duplicate keys.
fn parse_pack_json(pack_json: &str) -> Result<Value, String> {
    let item: Value = serde_json::from_str(pack_json).map_err(|e| format!("pack JSON: {e}"))?;
    let declarations = [
        "skill_index",
        "trigger_index",
        "active_index",
        "modifier_index",
        "view_as_index",
    ]
    .iter()
    .any(|key| item.get(*key).is_some());
    if item.get("modules").is_some() {
        let package = content_schema::ContentPackage::from_json_bytes(pack_json.as_bytes())
            .map_err(|e| format!("npack: {e}"))?;
        package.validate().map_err(|e| format!("npack: {e}"))?;
    } else if declarations {
        return Err("declaration-bearing partial npack requires modules".into());
    } else if let Some(schema) = item
        .get("manifest")
        .and_then(|m| m.get("content_schema_version"))
    {
        if !matches!(schema.as_u64(), Some(3 | 4)) {
            return Err("unsupported asset-only content_schema; schema5 declaration package requires modules".into());
        }
    }
    Ok(item)
}

fn installed_pack(item: &Value) -> Result<ExtensionPack, String> {
    let manifest = &item["manifest"];
    let namespace = required(manifest, "namespace")?;
    let package_id = required(manifest, "package_id")?;
    let version = required(manifest, "version")?;
    // Bind both the semantic version and canonical installed assets, not cards.json's integer version.
    let digest = blake3::hash(item.to_string().as_bytes()).to_hex();
    let identity = names::intern(format!(
        "installed:{}:{digest}",
        serde_json::json!([namespace, package_id, version])
    ));
    // 完整 npack（含 modules/trigger_index）也是合法的 Worker 安装载荷：
    // 服务端已按 `bridge_vm_skill_decls` 执行，客户端此前只认 cards/characters/pile，
    // 导致 VM-only 包在 Worker 被“缺少载荷”拒绝。这里直接解析整包并桥接技能声明。
    let vm_decls = if item.get("modules").is_some() {
        let package: content_schema::ContentPackage =
            serde_json::from_value(item.clone()).map_err(|e| format!("npack: {e}"))?;
        package.validate().map_err(|e| format!("npack: {e}"))?;
        content_native_loader::bridge_vm_skill_decls(&package)
            .map_err(|e| e.to_string())?
            .decls
    } else {
        Vec::new()
    };
    let mut pack = if let Some(cards) = item.get("cards") {
        content_native_loader::bridge_pack(&json(cards))
            .map_err(|e| e.to_string())?
            .pack
    } else {
        ExtensionPack {
            def: ExtensionDef {
                name: identity,
                version: 1,
                description: "",
            },
            cards: Vec::new(),
            generals: Vec::new(),
            host_skills: Vec::new(),
            vm_skills: Vec::new(),
            init_hook: InitHook::None,
            enabled: true,
        }
    };
    // characters 资产独立于 cards 存在：VM-only 包也需要武将。
    if let Some(characters) = item.get("characters") {
        let vm_modules = item
            .get("modules")
            .and_then(Value::as_array)
            .is_some_and(|v| !v.is_empty());
        let generals = if vm_modules {
            content_native_loader::bridge_generals_with_vm_package(
                &json(characters),
                &serde_json::from_value(item.clone()).map_err(|e| format!("npack: {e}"))?,
            )
        } else {
            content_native_loader::bridge_generals(&json(characters))
        }
        .map_err(|e| e.to_string())?;
        pack.generals.extend(generals.generals);
        for decl in generals.host_skills {
            match pack.host_skills.iter().find(|d| d.name == decl.name) {
                Some(d) if d.behavior != decl.behavior => {
                    return Err(format!("重复技能 {}", decl.name))
                }
                Some(_) => {}
                None => pack.host_skills.push(decl),
            }
        }
    }
    if let Some(pile) = item.get("pile") {
        let spec = content_native_loader::bridge_pile(&json(pile)).map_err(|e| e.to_string())?;
        let addition = extension_pack_from_pile(pack.def.clone(), identity, &spec)
            .map_err(|e| format!("pile: {e:?}"))?;
        pack.cards.extend(addition.cards);
    }
    pack.vm_skills.extend(vm_decls);
    if !["cards", "characters", "pile"]
        .iter()
        .any(|key| item.get(*key).is_some())
        && item.get("modules").is_none()
    {
        return Err(format!(
            "{namespace}:{package_id} 缺少 cards/characters/pile 载荷"
        ));
    }
    pack.def.name = identity;
    Ok(pack)
}

/// 导入期深层校验（GT-014，裁决第 12 项）：对**单个**安装载荷执行与开局装载
/// 完全相同的逐包校验——manifest 身份字段、`ContentPackage::validate()`
/// （模块结构 + 包/模块 logic_hash）、`bridge_vm_skill_decls`（rule-vm 装载期
/// verify + 宿主 ABI 窗口）、cards/characters/pile 桥接。只校验不建局；跨包
/// 依赖解析仍在开局（`packs_from_json`）执行。与 `packs_from_json` 的单项
/// 分派保持一致（含无 manifest 的旧 cards.json 元素）。
pub(super) fn validate_pack_json(pack_json: &str) -> Result<(), String> {
    let item = parse_pack_json(pack_json)?;
    if item.get("manifest").is_some() {
        installed_pack(&item).map(|_| ())
    } else {
        content_native_loader::bridge_pack(&json(&item))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

pub(super) fn packs_from_json(packs_json: &str) -> Result<Vec<ExtensionPack>, String> {
    let raw: Vec<&serde_json::value::RawValue> =
        serde_json::from_str(packs_json).map_err(|e| format!("packs 须为 JSON 数组: {e}"))?;
    let items: Vec<Value> = raw
        .iter()
        .map(|item| parse_pack_json(item.get()))
        .collect::<Result<_, _>>()?;
    let arr = &items;
    let mut slots = BTreeMap::new();
    for (index, item) in arr.iter().enumerate() {
        if let Some(manifest) = item.get("manifest") {
            let namespace =
                required(manifest, "namespace").map_err(|e| format!("packs[{index}]: {e}"))?;
            let package_id =
                required(manifest, "package_id").map_err(|e| format!("packs[{index}]: {e}"))?;
            let version =
                required(manifest, "version").map_err(|e| format!("packs[{index}]: {e}"))?;
            if slots
                .insert((namespace, package_id), (version, manifest))
                .is_some()
            {
                return Err(format!(
                    "packs[{index}]: 重复包槽位 {namespace}/{package_id}"
                ));
            }
        }
    }
    for (index, item) in arr.iter().enumerate() {
        let Some(manifest) = item.get("manifest") else {
            continue;
        };
        let Some(raw) = manifest.get("dependencies") else {
            continue;
        };
        let dependencies: Vec<content_schema::PackageDependency> =
            serde_json::from_value(raw.clone())
                .map_err(|error| format!("packs[{index}]: 依赖格式无效: {error}"))?;
        let own = (
            required(manifest, "namespace")?,
            required(manifest, "package_id")?,
        );
        let mut seen = BTreeSet::new();
        for dependency in dependencies {
            if ![
                &dependency.namespace,
                &dependency.package_id,
                &dependency.version,
            ]
            .iter()
            .all(|field| valid_identity(field))
            {
                return Err(format!("packs[{index}]: 依赖身份字段无效"));
            }
            let key = (
                dependency.namespace.as_str(),
                dependency.package_id.as_str(),
            );
            if key == own
                || !seen.insert((dependency.namespace.clone(), dependency.package_id.clone()))
            {
                return Err(format!(
                    "packs[{index}]: 依赖重复或指向自身 {}/{}",
                    key.0, key.1
                ));
            }
            let Some((version, target)) = slots.get(&key) else {
                return Err(format!(
                    "packs[{index}]: 缺少依赖 {}/{}@{}",
                    key.0, key.1, dependency.version
                ));
            };
            let hash = target
                .get("logic_hash")
                .and_then(|value| serde_json::from_value::<[u8; 32]>(value.clone()).ok());
            if *version != dependency.version
                || (dependency.logic_hash != [0; 32] && hash != Some(dependency.logic_hash))
            {
                return Err(format!(
                    "packs[{index}]: 依赖版本或指纹不匹配 {}/{}@{}",
                    key.0, key.1, dependency.version
                ));
            }
        }
    }
    let mut installed = BTreeSet::new();
    arr.iter()
        .enumerate()
        .map(|(index, item)| {
            let result = if item.get("manifest").is_some() {
                installed_pack(item)
            } else {
                content_native_loader::bridge_pack(&json(item))
                    .map(|out| out.pack)
                    .map_err(|e| e.to_string())
            };
            let pack = result.map_err(|e| format!("packs[{index}]: {e}"))?;
            if !installed.insert(pack.def.name) {
                return Err(format!("packs[{index}]: 重复包 {}", pack.def.name));
            }
            Ok(pack)
        })
        .collect()
}
