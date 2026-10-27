//! Installed package assets cross the Worker boundary together with their exact manifest.
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::content::{names, pile::extension_pack_from_pile};
use serde_json::Value;
use std::collections::BTreeSet;

fn required<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("manifest.{field} 须为非空字符串"))
}

fn json(value: &Value) -> String {
    value.to_string()
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
        content_native_loader::bridge_vm_skill_decls(&package)
            .map_err(|e| e.to_string())?
            .decls
    } else {
        Vec::new()
    };
    let vm_names: BTreeSet<String> = vm_decls.iter().map(|decl| decl.name.clone()).collect();
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
            content_native_loader::bridge_generals_with_known(&json(characters), &vm_names)
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

pub(super) fn packs_from_json(packs_json: &str) -> Result<Vec<ExtensionPack>, String> {
    let value: Value = serde_json::from_str(packs_json).map_err(|e| format!("packs JSON: {e}"))?;
    let arr = value
        .as_array()
        .ok_or_else(|| "packs 须为 JSON 数组".to_string())?;
    let mut slots = BTreeSet::new();
    for (index, item) in arr.iter().enumerate() {
        if let Some(manifest) = item.get("manifest") {
            let namespace =
                required(manifest, "namespace").map_err(|e| format!("packs[{index}]: {e}"))?;
            let package_id =
                required(manifest, "package_id").map_err(|e| format!("packs[{index}]: {e}"))?;
            required(manifest, "version").map_err(|e| format!("packs[{index}]: {e}"))?;
            if !slots.insert((namespace, package_id)) {
                return Err(format!(
                    "packs[{index}]: 重复包槽位 {namespace}/{package_id}"
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
