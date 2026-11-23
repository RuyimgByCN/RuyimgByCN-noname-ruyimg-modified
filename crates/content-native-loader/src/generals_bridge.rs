//! 角色包（generals）与内嵌武将段装载桥接：`cards.json` 的 `generals` 数组 →
//! [`BridgedGenerals`]（`ExtensionGeneral` + 宿主技能声明）。

use std::collections::BTreeSet;

use engine_native::content::extension::ExtensionGeneral;
use engine_native::content::host::HostSkillDecl;
use engine_native::content::names;
use serde_json::Value;

use crate::{
    ensure_host_api_compatible, ensure_symbol_capacity, faction_of, gender_of, invalid, mapping_of,
    opt_array, opt_int, parse_json, req_array, req_int, req_str, BridgeError, BridgedDefinitionIds,
    BridgedGenerals, MappedSkill,
};

/// 武将装载（角色包管线）：generals 数组 → `ExtensionGeneral` + 命中宿主
/// 行为的技能声明。势力、性别、体力上限缺失或非法均报错。
pub fn bridge_generals(json: &str) -> Result<BridgedGenerals, BridgeError> {
    ensure_symbol_capacity()?;
    let parsed = parse_json(json)?;
    ensure_host_api_compatible(&parsed)?;
    bridge_generals_inner(&parsed, true, &BTreeSet::new(), false)
}

/// 与 [`bridge_generals`] 相同，但把 `known_vm_skills` 中的 VM 逻辑技能名视为
/// 已解析技能：它们不产生 host 行为，只进入武将技能列表。用于完整 npack
/// （模块 + characters 资产）的 Worker／服务端安装路径。含 VM 技能的包
/// 必须拒绝无法解析的武将技能，避免安装成功后静默丢失技能。
pub fn bridge_generals_with_known(
    json: &str,
    known_vm_skills: &BTreeSet<String>,
) -> Result<BridgedGenerals, BridgeError> {
    ensure_symbol_capacity()?;
    let parsed = parse_json(json)?;
    ensure_host_api_compatible(&parsed)?;
    bridge_generals_inner(&parsed, true, known_vm_skills, true)
}

/// `require_generals`：独立入口必需；全量入口（`bridge_pack`）对纯卡牌包
/// 容许缺省。
pub(crate) fn bridge_generals_inner(
    parsed: &Value,
    require_generals: bool,
    known_vm_skills: &BTreeSet<String>,
    reject_unmapped: bool,
) -> Result<BridgedGenerals, BridgeError> {
    let generals_json = if require_generals {
        req_array(parsed, "generals", "generals")?
    } else {
        opt_array(parsed, "generals", "generals")?
    };
    let mut generals = Vec::new();
    let mut mapped_skill_keys = Vec::new();
    let mut host_skills: Vec<HostSkillDecl> = Vec::new();
    let mut unmapped = 0usize;
    let mut definition_ids = BridgedDefinitionIds::default();
    for g in generals_json {
        let name = req_str(g, "name", "generals[].name")?;
        let gen_def_id =
            opt_int(g, "definition_id", "generals[].definition_id", 0..=u64::from(u32::MAX))?
                .map(|n| n as u32);
        let mut skills: Vec<&'static str> = Vec::new();
        for sk in req_array(g, "skills", "generals[].skills")? {
            let key = req_str(sk, "name", "generals[].skills[].name")?;
            let sk_def_id = opt_int(
                sk,
                "definition_id",
                "generals[].skills[].definition_id",
                0..=u64::from(u32::MAX),
            )?
            .map(|n| n as u32);
            match mapping_of(sk, "generals[].skills[].mapping")? {
                MappedSkill::Host(behavior) => {
                    let interned = names::intern(key.to_string());
                    match host_skills.iter().find(|d| d.name == interned) {
                        Some(d) if d.behavior != behavior => {
                            return Err(BridgeError::DuplicateSkill(key.to_string()));
                        }
                        Some(_) => {}
                        None => host_skills.push(HostSkillDecl {
                            name: interned,
                            behavior,
                            presentation_binding: None,
                        }),
                    }
                    match (
                        definition_ids
                            .host_skill_definition_ids
                            .get(interned)
                            .copied()
                            .flatten(),
                        sk_def_id,
                    ) {
                        (Some(existing), Some(incoming)) if existing != incoming => {
                            return Err(invalid(
                                "generals[].skills[].definition_id",
                                format!("宿主技能 {key} 的 definition_id 冲突: {existing} vs {incoming}"),
                            ));
                        }
                        (None, Some(incoming)) => {
                            definition_ids
                                .host_skill_definition_ids
                                .insert(interned, Some(incoming));
                        }
                        (None, None) => {
                            definition_ids
                                .host_skill_definition_ids
                                .entry(interned)
                                .or_insert(None);
                        }
                        (Some(_), _) => {}
                    }
                    // 上游 `diy_xizhenxihong` 仅含 `["fuchou", "jinyan"]`，不含 `jianbi`
                    //（原手写 `wudian-upstream-noname/assets/cards.json` 误将 `jianbi` 放在西真夕红下；
                    // 保留 `host_skills` 声明与 `source_hash` 不变，但不在目录中授予西真夕红 `jianbi`）。
                    if !(matches!(name, "西真夕红" | "wdb_xizhenxihong") && interned == "jianbi")
                    {
                        skills.push(interned);
                    }
                    mapped_skill_keys.push(interned);
                    // 父技 group：上游把伤害子技登记在父技 group 里，授予父技即授予
                    // 子技；转换包曾漏掉这层关系，导致「父技所有权就让伤害面可达」
                    // 或「子技永不可达」两种错误。此处按 behavior 表补回声明与授予。
                    for &(child_key, child_behavior) in behavior.group_children() {
                        let child = names::intern(child_key.to_string());
                        match host_skills.iter().find(|d| d.name == child) {
                            Some(d) if d.behavior != child_behavior => {
                                return Err(BridgeError::DuplicateSkill(child_key.to_string()));
                            }
                            Some(_) => {}
                            None => host_skills.push(HostSkillDecl {
                                name: child,
                                behavior: child_behavior,
                                presentation_binding: None,
                            }),
                        }
                        if !skills.contains(&child) {
                            skills.push(child);
                        }
                        mapped_skill_keys.push(child);
                    }
                }
                // 内置引用：技能 def 已在标准目录（白名单保证实现存在）
                // ——只 intern 进武将技能列表，不产出 host 行为。
                MappedSkill::Builtin(builtin) => {
                    if sk_def_id.is_some() {
                        return Err(invalid(
                            "generals[].skills[].definition_id",
                            format!("builtin 技能引用 {builtin} 不能声明包内 definition_id"),
                        ));
                    }
                    skills.push(names::intern(builtin.to_string()));
                }
                MappedSkill::Deferred if known_vm_skills.contains(key) => {
                    skills.push(names::intern(key.to_string()));
                }
                MappedSkill::Deferred if reject_unmapped => {
                    return Err(invalid(
                        "generals[].skills[].name",
                        format!("VM 包的武将技能 {key} 未映射"),
                    ));
                }
                MappedSkill::Deferred => {
                    if sk_def_id.is_some() {
                        return Err(invalid(
                            "generals[].skills[].definition_id",
                            format!("暂缓技能 {key} 不能声明 definition_id"),
                        ));
                    }
                    unmapped += 1;
                }
            }
        }
        let faction = faction_of(req_str(g, "faction", "generals[].faction")?)?;
        let gender = gender_of(req_str(g, "gender", "generals[].gender")?)?;
        // D5/T11-BOSS-ZHUGE-LIANG 前置：体力 `Infinity`（上游 JS 字面量，
        // 包数据只能写成字符串）映射为无限体力哨兵；其余非法值仍拒绝。
        let max_hp = match g.get("hp") {
            Some(serde_json::Value::String(text)) if text == "Infinity" => {
                engine_native::state::INFINITE_HP
            }
            Some(serde_json::Value::Number(n))
                if n.as_u64() == Some(u64::from(engine_native::state::INFINITE_HP)) =>
            {
                engine_native::state::INFINITE_HP
            }
            _ => req_int(g, "hp", "generals[].hp", 1..=u64::from(u8::MAX))? as u32,
        };
        generals.push(ExtensionGeneral {
            name: names::intern(name.to_string()),
            faction,
            skills: names::intern_list(skills),
            max_hp,
            gender,
            presentation_binding: None,
        });
        definition_ids.general_definition_ids.push(gen_def_id);
    }
    Ok(BridgedGenerals {
        generals,
        host_skills,
        mapped_skill_keys,
        unmapped_skills: unmapped,
        definition_ids,
    })
}
