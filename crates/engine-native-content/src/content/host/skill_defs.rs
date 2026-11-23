//! 宿主技能元数据、发动限制与目录注册定义（从 `host.rs` 按职责拆分）。

use super::*;

/// 按技能名与宿主行为解析触发元数据（含 U01 3D精选分批技能）。
pub fn host_skill_trigger_meta(
    name: &str,
    behavior: HostBehavior,
) -> Option<crate::catalog::TriggerMeta> {
    match behavior {
        HostBehavior::U01DddBatchA => step::u01_batch_a::trigger_meta_u01_batch_a(name),
        HostBehavior::U01DddBatchB => step::u01_batch_b::trigger_meta_u01_batch_b(name),
        HostBehavior::U01DddBatchC => step::u01_batch_c::trigger_meta_u01_batch_c(name),
        HostBehavior::U01DddBatchD => step::u01_batch_d::trigger_meta_u01_batch_d(name),
        HostBehavior::L28ThunderBatchA => step::l28_batch_a::trigger_meta_l28_batch_a(name),
        HostBehavior::L28ThunderBatchB => step::l28_batch_b::trigger_meta_l28_batch_b(name),
        HostBehavior::L27BeautifyBatchA => step::l27_batch_a::trigger_meta_l27_batch_a(name),
        HostBehavior::L27BeautifyBatchB => step::l27_batch_b::trigger_meta_l27_batch_b(name),
        HostBehavior::L19DoomsdayBatchA => step::l19_batch_a::trigger_meta_l19_batch_a(name),
        HostBehavior::L19DoomsdayBatchB => step::l19_batch_b::trigger_meta_l19_batch_b(name),
        HostBehavior::L20MarkBatchA => step::l20_batch_a::trigger_meta_l20_batch_a(name),
        HostBehavior::L20MarkBatchB | HostBehavior::FzLiegong => {
            step::l20_batch_b::trigger_meta_l20_batch_b(name)
        }
        HostBehavior::U08YxsBatchA => step::u08_batch_a::trigger_meta_u08_batch_a(name),
        HostBehavior::U08YxsBatchB => step::u08_batch_b::trigger_meta_u08_batch_b(name),
        HostBehavior::U08YxsBatchC => step::u08_batch_c::trigger_meta_u08_batch_c(name),
        HostBehavior::U08YxsBatchD => step::u08_batch_d::trigger_meta_u08_batch_d(name),
        HostBehavior::L25ZuanBatchA => step::l25_batch_a::trigger_meta_l25_batch_a(name),
        HostBehavior::L25ZuanBatchB => step::l25_batch_b::trigger_meta_l25_batch_b(name),
        HostBehavior::L25ZuanBatchC => step::l25_batch_c::trigger_meta_l25_batch_c(name),
        HostBehavior::L25ZuanBatchD => step::l25_batch_d::trigger_meta_l25_batch_d(name),
        HostBehavior::L25ZuanBatchE => step::l25_batch_e::trigger_meta_l25_batch_e(name),
        HostBehavior::L25ZuanBatchF => step::l25_batch_f::trigger_meta_l25_batch_f(name),
        HostBehavior::L25ZuanBatchG => step::l25_batch_g::trigger_meta_l25_batch_g(name),
        HostBehavior::L10ScsCompanion => step::l10_scs::trigger_meta_l10_scs(name),
        HostBehavior::L11TrickCard => step::l11_cards::trigger_meta_l11_trick(name),
        HostBehavior::L11EquipCards => step::l11_equips::trigger_meta_l11_equips(name),
        HostBehavior::L11Daiyuan
        | HostBehavior::L11ServantSkills
        | HostBehavior::L11SubStates => step::l11_servants::trigger_meta_l11_servants(name),
        HostBehavior::L11GeneralSkillsA => step::l11_skills_a::trigger_meta_l11_skills_a(name),
        HostBehavior::L11GeneralSkillsB => step::l11_skills_b::trigger_meta_l11_skills_b(name),
        HostBehavior::L11GeneralSkillsC => step::l11_skills_c::trigger_meta_l11_skills_c(name),
        HostBehavior::L11ServantHooks => {
            step::l11_servants_b::trigger_meta_l11_servant_hooks(name)
        }
        HostBehavior::L11ServantSkillsB => step::l11_servants_b::trigger_meta_l11_servants_b(name),
        HostBehavior::L11GeneralSkillsD => step::l11_skills_d::trigger_meta_l11_skills_d(name),
        HostBehavior::L11GeneralSkillsE => step::l11_skills_e::trigger_meta_l11_skills_e(name),
        HostBehavior::L11GeneralSkillsF => step::l11_skills_f::trigger_meta_l11_skills_f(name),
        HostBehavior::L11BuffsA => step::l11_buffs_a::trigger_meta_l11_buffs_a(name),
        HostBehavior::L11BuffsB => step::l11_buffs_b::trigger_meta_l11_buffs_b(name),
        HostBehavior::L11BuffHelpersA => {
            step::l11_buff_helpers_a::trigger_meta_l11_buff_helpers_a(name)
        }
        HostBehavior::L11BuffsC => step::l11_buffs_c::trigger_meta_l11_buffs_c(name),
        HostBehavior::L11BuffsD => step::l11_buffs_d::trigger_meta_l11_buffs_d(name),
        HostBehavior::L11BuffHelpersB => {
            step::l11_buff_helpers_b::trigger_meta_l11_buff_helpers_b(name)
        }
        HostBehavior::L11SystemHooks => step::l11_system_hooks::trigger_meta_l11_system_hooks(name),
        HostBehavior::L11Weapons => step::l11_weapons::trigger_meta_l11_weapons(name),
        HostBehavior::L11GeneralBossC | HostBehavior::L11BossLingfeng => {
            step::l11_boss_c::trigger_meta_l11_boss_c(name)
        }
        HostBehavior::U05MainStoneBatch | HostBehavior::U05ZhenfaRecastSkill => {
            step::u05_b1_zhenfa::trigger_meta_u05_zhenfa(name)
        }
        HostBehavior::U05MtgCharBatch | HostBehavior::U05MtgLandExtraBatch => {
            step::u05_b1_mtg::trigger_meta_u05_mtg(name)
        }
        HostBehavior::U05YunchouCharBatch => {
            step::u05_b1_yunchou::trigger_meta_u05_yunchou_char(name)
        }
        HostBehavior::U05GujianCardBatch
        | HostBehavior::U05GujianCharBatchA
        | HostBehavior::U05GujianCharBatchB
        | HostBehavior::U05GujianCharBatchC
        | HostBehavior::U05GujianCharBatchD
        | HostBehavior::U05GujianCharBatchE => step::u05_b2_gujian::trigger_meta_u05_gujian(behavior, name),
        _ => crate::event::NativeDamageEffect::from_host(behavior)
            .map(|e| e.meta())
            .or_else(|| behavior.trigger_meta()),
    }
}

fn runtime_host_skill_trigger_meta(
    name: &str,
    behavior: HostBehavior,
) -> Option<crate::catalog::TriggerMeta> {
    match behavior {
        HostBehavior::U01DddBatchA => {
            step::u01_fixes_1::trigger_meta_u01_fix(name)
        }
        HostBehavior::U01DddBatchB | HostBehavior::U01DddBatchC | HostBehavior::U01DddBatchD => {
            step::u01_fixes_1::trigger_meta_u01_fix(name)
                .or_else(|| step::u01_fixes_2::trigger_meta_u01_fix2(name))
                .or_else(|| step::u01_fixes_3::trigger_meta_u01_fix3(name))
        }
        _ => None,
    }
    .or_else(|| host_skill_trigger_meta(name, behavior))
}

/// 按技能名与宿主行为解析发动限制（含 U01 3D精选分批技能）。
pub fn host_skill_limits(name: &str, behavior: HostBehavior) -> crate::catalog::SkillLimits {
    match behavior {
        HostBehavior::U01DddBatchA => {
            step::u01_fixes_1::limits_u01_fix(name)
                .unwrap_or_else(|| step::u01_batch_a::limits_u01_batch_a(name))
        }
        HostBehavior::U01DddBatchB => {
            step::u01_fixes_1::limits_u01_fix(name)
                .or_else(|| step::u01_fixes_2::limits_u01_fix2(name))
                .unwrap_or_else(|| step::u01_batch_b::limits_u01_batch_b(name))
        }
        HostBehavior::U01DddBatchC => {
            step::u01_fixes_1::limits_u01_fix(name)
                .or_else(|| step::u01_fixes_2::limits_u01_fix2(name))
                .or_else(|| step::u01_fixes_3::limits_u01_fix3(name))
                .unwrap_or_else(|| step::u01_batch_c::limits_u01_batch_c(name))
        }
        HostBehavior::U01DddBatchD => {
            step::u01_fixes_1::limits_u01_fix(name)
                .or_else(|| step::u01_fixes_2::limits_u01_fix2(name))
                .or_else(|| step::u01_fixes_3::limits_u01_fix3(name))
                .unwrap_or_else(|| step::u01_batch_d::limits_u01_batch_d(name))
        }
        HostBehavior::L28ThunderBatchA => step::l28_batch_a::limits_l28_batch_a(name),
        HostBehavior::L28ThunderBatchB => step::l28_batch_b::limits_l28_batch_b(name),
        HostBehavior::L27BeautifyBatchA => step::l27_batch_a::limits_l27_batch_a(name),
        HostBehavior::L27BeautifyBatchB => step::l27_batch_b::limits_l27_batch_b(name),
        HostBehavior::L19DoomsdayBatchA => step::l19_batch_a::limits_l19_batch_a(name),
        HostBehavior::L19DoomsdayBatchB => step::l19_batch_b::limits_l19_batch_b(name),
        HostBehavior::L20MarkBatchA => step::l20_batch_a::limits_l20_batch_a(name),
        HostBehavior::L20MarkBatchB => step::l20_batch_b::limits_l20_batch_b(name),
        HostBehavior::U08YxsBatchA => step::u08_batch_a::limits_u08_batch_a(name),
        HostBehavior::U08YxsBatchB => step::u08_batch_b::limits_u08_batch_b(name),
        HostBehavior::U08YxsBatchC => step::u08_batch_c::limits_u08_batch_c(name),
        HostBehavior::U08YxsBatchD => step::u08_batch_d::limits_u08_batch_d(name),
        HostBehavior::L25ZuanBatchA => step::l25_batch_a::limits_l25_batch_a(name),
        HostBehavior::L25ZuanBatchB => step::l25_batch_b::limits_l25_batch_b(name),
        HostBehavior::L25ZuanBatchC => step::l25_batch_c::limits_l25_batch_c(name),
        HostBehavior::L25ZuanBatchD => step::l25_batch_d::limits_l25_batch_d(name),
        HostBehavior::L25ZuanBatchE => step::l25_batch_e::limits_l25_batch_e(name),
        HostBehavior::L25ZuanBatchF => step::l25_batch_f::limits_l25_batch_f(name),
        HostBehavior::L25ZuanBatchG => step::l25_batch_g::limits_l25_batch_g(name),
        HostBehavior::L10ScsCompanion => step::l10_scs::limits_l10_scs(name),
        HostBehavior::L11TrickCard => step::l11_cards::limits_l11_trick(name),
        HostBehavior::L11EquipCards => step::l11_equips::limits_l11_equips(name),
        HostBehavior::L11Daiyuan
        | HostBehavior::L11ServantSkills
        | HostBehavior::L11SubStates => step::l11_servants::limits_l11_servants(name),
        HostBehavior::L11GeneralSkillsA => step::l11_skills_a::limits_l11_skills_a(name),
        HostBehavior::L11GeneralSkillsB => step::l11_skills_b::limits_l11_skills_b(name),
        HostBehavior::L11GeneralSkillsC => step::l11_skills_c::limits_l11_skills_c(name),
        HostBehavior::L11ServantHooks => crate::catalog::SkillLimits::default(),
        HostBehavior::L11ServantSkillsB => step::l11_servants_b::limits_l11_servants_b(name),
        HostBehavior::L11GeneralSkillsD => step::l11_skills_d::limits_l11_skills_d(name),
        HostBehavior::L11GeneralSkillsE => step::l11_skills_e::limits_l11_skills_e(name),
        HostBehavior::L11GeneralSkillsF => step::l11_skills_f::limits_l11_skills_f(name),
        HostBehavior::L11BuffsA => step::l11_buffs_a::limits_l11_buffs_a(name),
        HostBehavior::L11BuffsB => step::l11_buffs_b::limits_l11_buffs_b(name),
        HostBehavior::L11BuffHelpersA => step::l11_buff_helpers_a::limits_l11_buff_helpers_a(name),
        HostBehavior::L11BuffsC => step::l11_buffs_c::limits_l11_buffs_c(name),
        HostBehavior::L11BuffsD => step::l11_buffs_d::limits_l11_buffs_d(name),
        HostBehavior::L11BuffHelpersB => step::l11_buff_helpers_b::limits_l11_buff_helpers_b(name),
        HostBehavior::L11SystemHooks => step::l11_system_hooks::limits_l11_system_hooks(name),
        HostBehavior::L11Weapons => step::l11_weapons::limits_l11_weapons(name),
        HostBehavior::L11GeneralBossC | HostBehavior::L11BossLingfeng => {
            step::l11_boss_c::limits_l11_boss_c(name)
        }
        HostBehavior::U05MainStoneBatch | HostBehavior::U05ZhenfaRecastSkill => {
            step::u05_b1_zhenfa::limits_u05_zhenfa(name)
        }
        HostBehavior::U05MtgCharBatch | HostBehavior::U05MtgLandExtraBatch => {
            step::u05_b1_mtg::limits_u05_mtg(name)
        }
        HostBehavior::U05YunchouCharBatch => step::u05_b1_yunchou::limits_u05_yunchou_char(name),
        HostBehavior::U05GujianCardBatch
        | HostBehavior::U05GujianCharBatchA
        | HostBehavior::U05GujianCharBatchB
        | HostBehavior::U05GujianCharBatchC
        | HostBehavior::U05GujianCharBatchD
        | HostBehavior::U05GujianCharBatchE => step::u05_b2_gujian::limits_u05_gujian(behavior, name),
        HostBehavior::GujianFoodQingtuan | HostBehavior::GujianFoodLuyugeng => {
            crate::catalog::SkillLimits {
                once_per_turn: true,
                ..Default::default()
            }
        }
        // swd 装备主动技 usable:1（潜偷 v92 补执行/御夫录 v96）：出牌
        // 阶段限一次（上游 usable:1 同口径）
        // 革新印（批 135）：每回合限一次（上游 usable:1）
        HostBehavior::Gxianyin
        | HostBehavior::SwdShentouEquipSkill
        | HostBehavior::SwdYufuluEquipSkill
        | HostBehavior::SwdLangeguaiyiEquipSkill
        | HostBehavior::MtgYixialanRecast
        | HostBehavior::SwdShuchuiUse
        | HostBehavior::SwdDuishi
        | HostBehavior::SwdZhanlu
        // 解纷（第 1007 轮补）：上游 `enable:'phaseUse'` + `usable: 1`
        | HostBehavior::SwdJiefen
        // 魔焰（第 1010 轮补）：上游 `enable:'phaseUse'` + `usable: 1`
        | HostBehavior::Moyan
        // 攻心（第 1011 轮补）：上游 `enable:'phaseUse'` + `usable: 1`
        | HostBehavior::Gongxin
        | HostBehavior::SwdXianghui
        | HostBehavior::SwdSusheng
        | HostBehavior::SwdSenluo
        | HostBehavior::SwdDuanyi
        | HostBehavior::SwdFengming
        | HostBehavior::SwdXiaomoyu
        | HostBehavior::SwdBingjian
        | HostBehavior::SwdShending
        | HostBehavior::SwdGuiyan
        | HostBehavior::SwdXingzhui
        | HostBehavior::SwdHuangyu
        | HostBehavior::SwdTianhuo
        | HostBehavior::SwdYunchou
        | HostBehavior::SwdLuanji
        | HostBehavior::SwdZhanxing
        | HostBehavior::SwdZhenjiu
        | HostBehavior::SwdLmazui
        | HostBehavior::SwdHuodan
        | HostBehavior::SwdYijue
        | HostBehavior::SwdDangping
        | HostBehavior::SwdMingfu
        | HostBehavior::SwdCyxianjiang
        | HostBehavior::SwdJianji
        | HostBehavior::SwdAojian
        | HostBehavior::SwdHuajin
        | HostBehavior::SwdXuanzhou
        | HostBehavior::SwdFuniao
        | HostBehavior::SwdHeihuo
        | HostBehavior::SwdToudan
        | HostBehavior::SwdNlianji
        | HostBehavior::SwdShehun
        | HostBehavior::SwdJuxi
        | HostBehavior::SwdPozhou2
        | HostBehavior::SwdSxianjing
        | HostBehavior::SwdGuxing => crate::catalog::SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        HostBehavior::MtgLinzhongjiantaStrike
        | HostBehavior::MtgYoulinDiscard
        | HostBehavior::ChangeCardNature
        | HostBehavior::WudianKangyin
        | HostBehavior::WudianLiangou
        | HostBehavior::WudianQianggu
        | HostBehavior::WudianYinbo
        | HostBehavior::XianjianGuiyuan
        | HostBehavior::SwdDuanyue
        | HostBehavior::SwdXingdian
        | HostBehavior::SwdLianyaohuShouna
        | HostBehavior::SwdLianyaohuLianhua
        | HostBehavior::SwdShennongdingCard
        | HostBehavior::SwdKunlunjingcCard
        | HostBehavior::SwdSifeizhenmianCard
        | HostBehavior::MbcmFangzhu => crate::catalog::SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => crate::catalog::SkillLimits::default(),
    }
}

/// 目录侧注册：行为 → 触发元数据 + 宿主行为实现（打包为 SkillDef）。
pub fn host_skill_def(name: String, behavior: HostBehavior) -> crate::catalog::SkillDef {
    // 目录技能名为 'static：经符号表 intern，同名名称全局只分配一次
    // （重复开局/启停循环内存恒定，见 content::names）。
    let name: &'static str = crate::content::names::intern(name);
    let limits = host_skill_limits(name, behavior);
    crate::catalog::SkillDef {
        name,
        trigger: runtime_host_skill_trigger_meta(name, behavior),
        limits,
        behavior: Box::new(HostSkillBehavior {
            name: name.to_string(),
            behavior,
        }),
        original_source: None,
        presentation_binding: None,
    }
}

