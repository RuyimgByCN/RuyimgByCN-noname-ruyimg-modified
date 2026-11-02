//! N11 版本化字节存档（ADR 0733）。
//!
//! `StateSnapshot` 的确定性序列化：定长小端整数 + 显式变体标签 + 定长 blake3
//! 校验和。格式 = `magic(4) + format_version(u16) + ruleset_version(u64) +
//! payload_len(u64) + payload + checksum(32)`。
//!
//! 契约：
//! - 同一快照编码结果逐字节相同（BTreeMap/Vec 按序遍历，无哈希随机性）；
//! - 解码是编码的精确逆（往返后快照与原值相等，恢复出的引擎同哈希）；
//! - 技能定义号/牌 id 按目录数值编码——跨规则版本不保证可解码，
//!   头部 `ruleset_version` 供宿主快速核验（`Engine::restore` 再做强校验）；
//! - 选择提示词解码经固定注册表（`KNOWN_PROMPTS`）零分配匹配，
//!   未知 prompt 返回 `BadTag` 错误（ADR 0774）；
//!   换取快照类型零改动。

use crate::catalog::CardKind;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec, OpenChoice};
use crate::event::{
    CardUseData, DamageData, DamageNature, EventData, EventKind, Phase, QueuedTrigger, Stage,
};
use crate::ids::{PlayerId, SkillDefId};
use crate::resolution::Slot;
use crate::snap::{
    FanJianStageSnap, GuanXingStageSnap, JiJiStageSnap, JiJieStageSnap, JieYinStageSnap,
    JiedaoStageSnap, JijiangStageSnap, JudgementProgressSnap, KuRouStageSnap, LiJianStageSnap,
    MachineSnapshot, ParkSnapshot, QingNangStageSnap, RendeStageSnap, ResolutionSnapshot,
    SanCaiStageSnap, ShuShenStageSnap, StateSnapshot, TongJiStageSnap, TuXiStageSnap,
    XiaoGuoStageSnap, XunXunStageSnap, YaoWuStageSnap, YiShangStageSnap, ZhangbaStageSnap,
    ZhiHengStageSnap,
};
use crate::state::{
    BossStageKind, BossStagePending, BossStageState, GameMode, Role, SkillInstance, Winner,
};

/// 存档格式版本：编码布局变化时递增。
/// v2：OpenChoice 显式编码 ChoiceKind（v1 由提示词静态推导，曾与机器实际
/// 种类漂移——nullify_window 实为 Confirm、jiji_target 实为 SelectPlayers）。
/// v3（2026-10-07，E03 使用前取消）：`EventData::CardUsed` 载荷尾部追加
/// `cancelled: u8`。v2 字节被版本门禁拒绝（存档仅存在于测试与内部工具，
/// 无生产迁移数据；取舍见规则基准）。
/// v4（2026-10-07，E03 翻面原语）：`PlayerState` 尾部追加
/// `face_down: u8`。v3 字节同样被版本门禁拒绝（无生产迁移数据）。
/// v42（2026-10-11，T11 掠影 v194）：`MachineSnapshot::Sha` 尾部追加
/// `lueying_declined: u8`（放弃标记，阻止同杀重入再问）。v41 字节被版本
/// 门禁拒绝（存档仅存在于测试与内部工具，无生产迁移数据）。
/// v43（2026-10-11，T12 机关怨 v196）：`PlayerState` 尾部追加
/// `jiguanyuan_cards`（len+u32 序列；待获得牌实体）。v42 字节被版本门禁
/// 拒绝（无生产迁移数据）。
/// v44（2026-10-11，T12 机关术批 v198）：`PlayerState` 尾部追加
/// `jileng_active`/`chaofeng_active`/`xingtigaizao_count`/`yinmi_active`
/// 灵剑状态四字段。v43 字节被版本门禁拒绝（无生产迁移数据）。
/// v45（2026-10-11，T18 机关术×2 v199）：`PlayerState` 尾部追加
/// `card_mods`（len+(u32,u32) 序列；卡牌实例改造表，ADR 0780）。v44 字节
/// 被版本门禁拒绝（无生产迁移数据）。
/// v46（2026-10-11，T19b 阵法 v201）：`GameStateCore` 尾部追加
/// `seat_ring`（len+pid 序列；座位环可变，swapSeat）。v45 字节被版本门禁
/// 拒绝（无生产迁移数据）。
/// v60（2026-10-11，T11 郭嘉批 v259）：`PlayerState` 尾部追加
/// `boss_mad`（混乱标记）；`TurnState` 尾部追加 `qizuo_copy`（副本载体
/// 槽）；`MachineSnapshot` 追加 `BossQizuo` 变体。v59 字节被版本门禁拒绝
///（无生产迁移数据）。
/// v61（2026-10-11，T11 周瑜批 v263）：`TurnState` 尾部追加
/// `jidian_chain`（极点链挡旗标）。v60 字节被版本门禁拒绝（无生产迁移
/// 数据）。
/// v79（2026-10-16，D4 Boss 关卡流程 B0）：`GameMode::Boss` 尾部追加
/// `BossStageState`（kind/level/孟婆旗/禁将表/翼武装/pending）。v78 字节
/// 被版本门禁拒绝（无生产迁移数据；存档仅测试与内部工具）。
/// 80：核心末尾追加 `VmAux`（T8 标记/牌堆/丢失/once/旗标）。空表仍写长度 0。
/// v87（2026-10-17，swd 批 114 锦鳞）：`PlayerState` 尾部追加
/// `jinlin_used`（limited 一次性主动技已发动）。v86 字节被版本门禁拒绝
///（无生产迁移数据；存档仅测试与内部工具）。
/// v88（2026-10-17，T11 Boss 诸葛亮族批 115）：`PlayerState` 尾部追加
/// hlss 星系统三字段（`hlss_stars`/`hlss_count`/`hlss_damage`）。v87 字节
/// 被版本门禁拒绝（无生产迁移数据）。
/// v89（2026-10-17，T11 Boss 诸葛亮族批 116）：选择项新增牌种码
/// `ChoiceOption::Kind(u16)`（判别 5），`PlayerState` 追加起风已用牌种表
/// `hlss_fengqi_used`，判定机追加改命候选位。v88 字节被版本门禁拒绝。
/// v90（2026-10-17，D5 龙血玄黄批 117）：`PlayerState` 尾部追加潜龙
/// 道心值 `mbqianlong_marks`。v89 字节被版本门禁拒绝。
/// v91（2026-10-17，T11 Boss 庞统族批 118）：`PlayerState` 追加涅槃限定
/// 技位 `niepan_used`，`BossStageState` 追加浴火关卡循环型 `loop_type`。
/// v90 字节被版本门禁拒绝。
/// v92（2026-10-17，swd 灵武批 120）：`TurnState` 追加本回合已用牌数
/// `cards_used_turn`、额外阶段队列 `extra_phases`、灵武插入位
/// `extra_phase_lingwu`。v91 字节被版本门禁拒绝。
/// v93（2026-10-17，swd 神焰批 121）：`PlayerState` 追加神焰限定位
/// `shenyan_used` 与待蔓延目标 `shenyan_pending`。v92 字节被版本门禁拒绝。
/// v94（2026-10-17，swd 入梦批 122）：`PlayerState` 追加跳过弃牌阶段位
/// `skip_discard_phase`。v93 字节被版本门禁拒绝。
/// v95（2026-10-17，swd 义山批 123）：`PlayerState` 追加最近失去牌表
/// `yishan_lost`。v94 字节被版本门禁拒绝。
/// v96（2026-10-17，swd 守印批 124）：`PlayerState` 追加守印限定位
/// `shouyin_used`。v95 字节被版本门禁拒绝。
/// v97（2026-10-17，swd 颂词批 129）：`PlayerState` 追加颂词已发动目标表
/// `songci_targets`。v96 字节被版本门禁拒绝。
/// v98（2026-10-17，swd 护天批 131）：`PlayerState` 追加护天四字段
/// （`hutian_deposit`/`hutian_just_given`/`hutian_hold`/`hutian_owner`）。
/// v97 字节被版本门禁拒绝。
/// v99（2026-10-17，swd 极略批 136）：`PlayerState` 追加极略本阶段已用
/// 次数 `jilve_used`。v98 字节被版本门禁拒绝。
/// v100（2026-10-17，swd 脉轮批 137）：`PlayerState` 追加脉轮三字段
/// （`mailun_mode`/`mailun_out_used`/`mailun_in_used`）。v99 字节被拒绝。
/// v101（2026-10-17，swd 天书批 142）：`PlayerState` 追加天书两字段
/// （`swdtianshu_from`/`swdtianshu_granted`）。v100 字节被拒绝。
/// v102（2026-10-17，swd 改造批 143）：`GameStateCore` 追加装备栏改写表
/// `declared_slots`。v101 字节被拒绝。
/// v103（2026-10-17，swd 燎原批 145）：`TurnState` 追加燎原计数
/// `liaoyuan_count`。v102 字节被拒绝。
/// v104（2026-10-17，swd 轩辕剑批 146）：`TurnState` 追加轩辕剑加伤标记
/// `xuanyuanjian_marked`。v103 字节被拒绝。
/// v105（2026-10-17，swd 昆仑镜批 149）：`TurnState` 追加昆仑镜快照
/// （`kunlunjing_snapshot`/`kunlunjing_hp`）。v104 字节被拒绝。
/// v106: Decision29 private C331 integration. Strict105 rejection; unpublished while
/// approved later C Core layouts are still pending in this intermediate format.
pub const ARCHIVE_FORMAT_VERSION: u16 = 106;

/// 存档魔数（"NGSA" = noname-game snapshot archive）。
pub const ARCHIVE_MAGIC: &[u8; 4] = b"NGSA";

/// 字节存档错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveError {
    BadMagic,
    UnsupportedFormatVersion(u16),
    ChecksumMismatch,
    UnexpectedEof,
    /// N11-b：头部与载荷的规则版本不一致（头被改写或载荷被替换）。
    RuleVersionMismatch {
        header: u64,
        payload: u64,
    },
    /// 未知的变体标签（数据损坏或版本不匹配）。
    BadTag(&'static str, u8),
    /// P0 解码加固：未知的牌种编号（保留完整 u32，不截断为 u8）。
    UnknownCardKind(u32),
    /// P0 解码加固：快照读尽后载荷仍有剩余字节（尾部夹带数据）。
    TrailingBytes {
        extra: usize,
    },
}

// ---- 对外 API ----

/// 编码快照为版本化字节存档（确定性：同一快照逐字节相同）。
pub fn encode_archive(snap: &StateSnapshot) -> Vec<u8> {
    let mut w = W::new();
    w.u64(snap.ruleset_version);
    enc_snapshot(&mut w, snap);
    let payload = w.buf;
    let mut out = Vec::with_capacity(payload.len() + 64);
    out.extend_from_slice(ARCHIVE_MAGIC);
    out.extend_from_slice(&ARCHIVE_FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&snap.ruleset_version.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(&payload);
    let checksum = blake3::hash(&payload);
    out.extend_from_slice(checksum.as_bytes());
    out
}

/// 解码字节存档；校验魔数/格式版本/校验和，解码结果与原快照相等。
pub fn decode_archive(data: &[u8]) -> Result<StateSnapshot, ArchiveError> {
    if data.len() < 4 + 2 + 8 + 8 + 32 {
        return Err(ArchiveError::UnexpectedEof);
    }
    if &data[0..4] != ARCHIVE_MAGIC {
        return Err(ArchiveError::BadMagic);
    }
    let fmt = u16::from_le_bytes(data[4..6].try_into().unwrap());
    if fmt != ARCHIVE_FORMAT_VERSION {
        return Err(ArchiveError::UnsupportedFormatVersion(fmt));
    }
    // N11-b：长度字段经有界转换与检查加法——u64::MAX 等恶意长度
    // 返回 UnexpectedEof 而非 usize 溢出 panic。
    let payload_len = usize::try_from(u64::from_le_bytes(data[14..22].try_into().unwrap()))
        .map_err(|_| ArchiveError::UnexpectedEof)?;
    let total_len = 22usize
        .checked_add(payload_len)
        .and_then(|v| v.checked_add(32))
        .ok_or(ArchiveError::UnexpectedEof)?;
    if data.len() != total_len {
        return Err(ArchiveError::UnexpectedEof);
    }
    let payload = &data[22..22 + payload_len];
    let expected = blake3::hash(payload);
    if expected.as_bytes() != &data[22 + payload_len..] {
        return Err(ArchiveError::ChecksumMismatch);
    }
    let mut r = R {
        data: payload,
        pos: 0,
    };
    // N11-b：头/载荷规则版本一致校验——头部改写不再被静默接受
    let ruleset_in_payload = r.u64()?;
    let ruleset_in_header = u64::from_le_bytes(data[6..14].try_into().unwrap());
    if ruleset_in_payload != ruleset_in_header {
        return Err(ArchiveError::RuleVersionMismatch {
            header: ruleset_in_header,
            payload: ruleset_in_payload,
        });
    }
    let snap = dec_snapshot(&mut r)?;
    if snap.ruleset_version != ruleset_in_header {
        return Err(ArchiveError::RuleVersionMismatch {
            header: ruleset_in_header,
            payload: snap.ruleset_version,
        });
    }
    // P0 解码加固：快照读尽后载荷仍有剩余字节即拒绝（尾部夹带数据）。
    if r.pos != payload.len() {
        return Err(ArchiveError::TrailingBytes {
            extra: payload.len() - r.pos,
        });
    }
    Ok(snap)
}
// ---------------------------------------------------------------------------
// A0 archive 单元测试（T17，2026-10-11 重建）：10-10 事故恢复中丢失的
// 3 个 lib 单元测试迁此——CardKind 编号往返/键双射/伴随一致依赖本模块
// 私有的 card_kind_to_num/from_num 枚举源（牌种无 ALL 数组）。
// ---------------------------------------------------------------------------
#[cfg(test)]
mod a0_rebuilt_tests {
    use super::*;
    use crate::catalog::CardKind;
    use crate::vocabulary::card_skills::{card_kind_equip_skills, card_kind_host_skills};

    /// CardKind 编号连续且全量编码往返（u8 存档编码域）。
    #[test]
    fn card_kind_encoding_roundtrip_all_variants() {
        let mut count = 0u32;
        while let Ok(kind) = card_kind_from_num(count) {
            assert_eq!(
                card_kind_to_num(&kind),
                count,
                "编号非连续或往返不等 @{count}"
            );
            count += 1;
        }
        assert!(
            count >= 133,
            "牌种数应 ≥133（2026-10-11 实测 133），实得 {count}"
        );
    }

    /// CardKind 数据键双射 + 唯一（键域经编号枚举全量；ADR 0778 键表）。
    #[test]
    fn card_kind_keys_bijective() {
        let mut keys = std::collections::BTreeMap::new();
        let mut count = 0u32;
        while let Ok(kind) = card_kind_from_num(count) {
            let key = kind.key();
            assert!(keys.insert(key, kind).is_none(), "牌种数据键重复: {key}");
            assert_eq!(CardKind::from_key(key), Some(kind), "键双射失败: {key}");
            count += 1;
        }
        assert!(count >= 133, "牌种数应 ≥133，实得 {count}");
    }

    /// 牌种伴随技能名全局唯一，且各牌种装备技均登记于伴随集合。
    #[test]
    fn card_kind_companion_skills_consistent() {
        let mut companion: std::collections::BTreeSet<&str> = Default::default();
        let mut count = 0u32;
        while let Ok(kind) = card_kind_from_num(count) {
            for d in card_kind_host_skills(kind) {
                assert!(companion.insert(d.name), "伴随技能重名: {}", d.name);
            }
            count += 1;
        }
        count = 0;
        while let Ok(kind) = card_kind_from_num(count) {
            for e in card_kind_equip_skills(kind) {
                assert!(
                    companion.contains(e),
                    "装备技 {e} 未登记为牌种伴随技能（kind @{count}）"
                );
            }
            count += 1;
        }
        assert!(count >= 133);
    }
}

mod io;
use io::*;
mod values;
use values::*;
mod events;
mod damage;
mod invocation;
use events::*;
mod machine_encode;
use machine_encode::*;
mod machine_decode;
use machine_decode::*;
mod resolution;
use resolution::*;
mod auxiliary;
use auxiliary::*;
mod state_encode;
use state_encode::*;
mod state_decode;
use state_decode::*;
mod card_codes;
pub use card_codes::*;
