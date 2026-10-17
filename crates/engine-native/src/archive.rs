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

use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec, OpenChoice};
use crate::catalog::CardKind;
use crate::event::{
    CardUseData, DamageData, DamageNature, EventData, EventKind, Phase, QueuedTrigger, Stage,
};
use crate::ids::{PlayerId, SkillDefId};
use crate::resolution::Slot;
use crate::snap::{
    FanJianStageSnap, GuanXingStageSnap, JiJiStageSnap, JieYinStageSnap, JijiangStageSnap,
    KuRouStageSnap, LiJianStageSnap, QingNangStageSnap, RendeStageSnap, TuXiStageSnap,
    XiaoGuoStageSnap, YaoWuStageSnap, ShuShenStageSnap, XunXunStageSnap, JiJieStageSnap, ZhangbaStageSnap, TongJiStageSnap, JiedaoStageSnap,
    ZhiHengStageSnap, JudgementProgressSnap, MachineSnapshot, ParkSnapshot,
    ResolutionSnapshot, SanCaiStageSnap, StateSnapshot, YiShangStageSnap,
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
pub const ARCHIVE_FORMAT_VERSION: u16 = 90;

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
    RuleVersionMismatch { header: u64, payload: u64 },
    /// 未知的变体标签（数据损坏或版本不匹配）。
    BadTag(&'static str, u8),
    /// P0 解码加固：未知的牌种编号（保留完整 u32，不截断为 u8）。
    UnknownCardKind(u32),
    /// P0 解码加固：快照读尽后载荷仍有剩余字节（尾部夹带数据）。
    TrailingBytes { extra: usize },
}

// ---- 写入器 ----

struct W {
    buf: Vec<u8>,
}

impl W {
    fn new() -> Self {
        Self { buf: Vec::new() }
    }
    fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.u32(v as u32);
    }
    fn boolean(&mut self, v: bool) {
        self.u8(v as u8);
    }
    fn opt_u16(&mut self, v: Option<u16>) {
        match v {
            Some(x) => {
                self.u8(1);
                self.u16(x);
            }
            None => self.u8(0),
        }
    }
    fn opt_u8(&mut self, v: Option<u8>) {
        match v {
            Some(x) => {
                self.u8(1);
                self.u8(x);
            }
            None => self.u8(0),
        }
    }
    fn len(&mut self, n: usize) {
        self.u64(n as u64);
    }
    fn bytes(&mut self, b: &[u8]) {
        self.len(b.len());
        self.buf.extend_from_slice(b);
    }
    fn str(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }
}

// ---- 读取器 ----

struct R<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> R<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ArchiveError> {
        let end = self.pos + n;
        if end > self.data.len() {
            return Err(ArchiveError::UnexpectedEof);
        }
        let out = &self.data[self.pos..end];
        self.pos = end;
        Ok(out)
    }
    fn u8(&mut self) -> Result<u8, ArchiveError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, ArchiveError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, ArchiveError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, ArchiveError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, ArchiveError> {
        Ok(self.u32()? as i32)
    }
    fn boolean(&mut self) -> Result<bool, ArchiveError> {
        Ok(self.u8()? != 0)
    }
    fn opt_u16(&mut self) -> Result<Option<u16>, ArchiveError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.u16()?)),
            t => Err(ArchiveError::BadTag("opt_u16", t)),
        }
    }
    fn opt_u8(&mut self) -> Result<Option<u8>, ArchiveError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.u8()?)),
            t => Err(ArchiveError::BadTag("opt", t)),
        }
    }
    fn len(&mut self) -> Result<usize, ArchiveError> {
        Ok(self.u64()? as usize)
    }
    fn bytes(&mut self) -> Result<&'a [u8], ArchiveError> {
        let n = self.len()?;
        self.take(n)
    }
    fn vec<T>(&mut self, mut f: impl FnMut(&mut R<'a>) -> Result<T, ArchiveError>) -> Result<Vec<T>, ArchiveError> {
        let n = self.len()?;
        let mut out = Vec::with_capacity(n.min(1024));
        for _ in 0..n {
            out.push(f(self)?);
        }
        Ok(out)
    }
}

// ---- 基础类型编码 ----

fn enc_opt_u32(w: &mut W, v: Option<u32>) {
    match v {
        Some(x) => {
            w.u8(1);
            w.u32(x);
        }
        None => w.u8(0),
    }
}

fn dec_opt_u32(r: &mut R<'_>) -> Result<Option<u32>, ArchiveError> {
    match r.u8()? {
        0 => Ok(None),
        1 => Ok(Some(r.u32()?)),
        t => Err(ArchiveError::BadTag("opt_u32", t)),
    }
}

/// PlayerId = u8。
fn enc_pid(w: &mut W, v: &PlayerId) {
    w.u8(*v);
}

fn dec_pid(r: &mut R<'_>) -> Result<PlayerId, ArchiveError> {
    Ok(r.u8()?)
}

/// Option<PlayerId>。
fn enc_opt_pid(w: &mut W, v: &Option<PlayerId>) {
    match v {
        Some(x) => {
            w.u8(1);
            w.u8(*x);
        }
        None => w.u8(0),
    }
}

fn dec_opt_pid(r: &mut R<'_>) -> Result<Option<PlayerId>, ArchiveError> {
    match r.u8()? {
        0 => Ok(None),
        1 => Ok(Some(r.u8()?)),
        t => Err(ArchiveError::BadTag("opt_pid", t)),
    }
}

/// Vec<PlayerId>。
fn enc_pids(w: &mut W, v: &[PlayerId]) {
    w.len(v.len());
    for p in v {
        w.u8(*p);
    }
}

fn dec_pids(r: &mut R<'_>) -> Result<Vec<PlayerId>, ArchiveError> {
    r.vec(|r| r.u8())
}

/// SkillDefId = u16。
fn enc_skill(w: &mut W, v: &SkillDefId) {
    w.u16(*v);
}

fn dec_skill(r: &mut R<'_>) -> Result<SkillDefId, ArchiveError> {
    Ok(r.u16()?)
}

fn enc_damage_nature(w: &mut W, v: &DamageNature) {
    w.u8(match v {
        DamageNature::Normal => 0,
        DamageNature::Fire => 1,
        DamageNature::Thunder => 2,
    });
}

fn dec_damage_nature(r: &mut R<'_>) -> Result<DamageNature, ArchiveError> {
    Ok(match r.u8()? {
        0 => DamageNature::Normal,
        1 => DamageNature::Fire,
        2 => DamageNature::Thunder,
        t => return Err(ArchiveError::BadTag("damage_nature", t)),
    })
}

fn enc_phase(w: &mut W, v: &Phase) {
    w.u8(match v {
        Phase::Judge => 0,
        Phase::Draw => 1,
        Phase::Play => 2,
        Phase::Discard => 3,
        Phase::End => 4,
    });
}

fn dec_phase(r: &mut R<'_>) -> Result<Phase, ArchiveError> {
    Ok(match r.u8()? {
        0 => Phase::Judge,
        1 => Phase::Draw,
        2 => Phase::Play,
        3 => Phase::Discard,
        4 => Phase::End,
        t => return Err(ArchiveError::BadTag("phase", t)),
    })
}

fn enc_event_kind(w: &mut W, v: &EventKind) {
    use EventKind::*;
    let t = match v {
        DamageBefore => 0u8,
        Damage => 1,
        Recover => 2,
        CardsDrawn => 3,
        CardsMoved => 4,
        CardUsed => 5,
        CardResponded => 6,
        Dying => 7,
        Death => 8,
        PhaseBegin => 9,
        PhaseEnd => 10,
        TurnEnded => 11,
        SkillToggled => 12,
        Equipped => 13,
        Linked => 14,
        CardsDiscarded => 15,
        TurnOverBefore => 16,
        TurnOvered => 17,
        LoseHp => 18,
        DyingSaved => 19,
    };
    w.u8(t);
}

fn dec_event_kind(r: &mut R<'_>) -> Result<EventKind, ArchiveError> {
    use EventKind::*;
    Ok(match r.u8()? {
        0 => DamageBefore,
        1 => Damage,
        2 => Recover,
        3 => CardsDrawn,
        4 => CardsMoved,
        5 => CardUsed,
        6 => CardResponded,
        7 => Dying,
        8 => Death,
        9 => PhaseBegin,
        10 => PhaseEnd,
        11 => TurnEnded,
        12 => SkillToggled,
        13 => Equipped,
        14 => Linked,
        15 => CardsDiscarded,
        16 => TurnOverBefore,
        17 => TurnOvered,
        18 => LoseHp,
        19 => DyingSaved,
        t => return Err(ArchiveError::BadTag("event_kind", t)),
    })
}

fn enc_stage(w: &mut W, v: &Stage) {
    w.u8(match v {
        Stage::BeforeTriggers => 0,
        Stage::Commit => 1,
        Stage::AfterTriggers => 2,
        Stage::Intrinsic => 3,
        Stage::Done => 4,
    });
}

fn dec_stage(r: &mut R<'_>) -> Result<Stage, ArchiveError> {
    Ok(match r.u8()? {
        0 => Stage::BeforeTriggers,
        1 => Stage::Commit,
        2 => Stage::AfterTriggers,
        3 => Stage::Intrinsic,
        4 => Stage::Done,
        t => return Err(ArchiveError::BadTag("stage", t)),
    })
}

fn enc_role(w: &mut W, v: &Role) {
    w.u8(match v {
        Role::Lord => 0,
        Role::Loyalist => 1,
        Role::Rebel => 2,
        Role::Traitor => 3,
    });
}

fn dec_role(r: &mut R<'_>) -> Result<Role, ArchiveError> {
    Ok(match r.u8()? {
        0 => Role::Lord,
        1 => Role::Loyalist,
        2 => Role::Rebel,
        3 => Role::Traitor,
        t => return Err(ArchiveError::BadTag("role", t)),
    })
}

fn enc_faction(w: &mut W, f: &crate::catalog::Faction) {
    w.u8(match f {
        crate::catalog::Faction::Wei => 0,
        crate::catalog::Faction::Shu => 1,
        crate::catalog::Faction::Wu => 2,
        crate::catalog::Faction::Qun => 3,
    });
}

fn dec_faction(r: &mut R<'_>) -> Result<crate::catalog::Faction, ArchiveError> {
    Ok(match r.u8()? {
        0 => crate::catalog::Faction::Wei,
        1 => crate::catalog::Faction::Shu,
        2 => crate::catalog::Faction::Wu,
        3 => crate::catalog::Faction::Qun,
        t => return Err(ArchiveError::BadTag("faction", t)),
    })
}

fn enc_winner(w: &mut W, v: &Winner) {
    match v {
        Winner::Player(p) => {
            w.u8(0);
            w.u8(*p);
        }
        Winner::LordSide => w.u8(1),
        Winner::Rebels => w.u8(2),
        Winner::Traitor => w.u8(3),
        // P0 解码加固：势力编码走显式表（与 dec_faction 同源），避免
        // `*f as u8` 依赖枚举声明顺序——重排/增删势力不再静默错位。
        Winner::Faction(f) => {
            w.u8(4);
            enc_faction(w, f);
        }
        Winner::Versus(t) => {
            w.u8(5);
            w.u8(*t as u8);
        }
        Winner::Doudizhu { lord_won } => {
            w.u8(6);
            w.boolean(*lord_won);
        }
        Winner::Boss { boss_won } => {
            w.u8(7);
            w.boolean(*boss_won);
        }
    }
}

fn dec_winner(r: &mut R<'_>) -> Result<Winner, ArchiveError> {
    Ok(match r.u8()? {
        0 => Winner::Player(r.u8()?),
        1 => Winner::LordSide,
        2 => Winner::Rebels,
        3 => Winner::Traitor,
        4 => Winner::Faction(dec_faction(r)?),
        5 => Winner::Versus(r.u8()? as usize),
        6 => Winner::Doudizhu { lord_won: r.boolean()? },
        7 => Winner::Boss { boss_won: r.boolean()? },
        t => return Err(ArchiveError::BadTag("winner", t)),
    })
}

// ---- 事件数据 ----

fn enc_event_data(w: &mut W, v: &EventData) {
    match v {
        EventData::Damage(d) => {
            w.u8(0);
            enc_opt_pid(w, &d.source);
            enc_pid(w, &d.target);
            w.u32(d.amount);
            enc_damage_nature(w, &d.nature);
            w.boolean(d.prevented);
            w.len(d.redirect_chain.len());
            for p in &d.redirect_chain {
                w.u8(*p);
            }
            enc_opt_u32(w, d.card);
            w.boolean(d.from_chain);
        }
        EventData::Recover {
            player,
            amount,
            by_card,
        } => {
            w.u8(1);
            enc_pid(w, player);
            w.u32(*amount);
            enc_opt_u32(w, *by_card);
        }
        EventData::CardsDrawn { player, cards } => {
            w.u8(2);
            enc_pid(w, player);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        EventData::CardsMoved { cards, from, to } => {
            w.u8(3);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            enc_pid(w, from);
            enc_pid(w, to);
        }
        EventData::CardUsed(u) => {
            w.u8(4);
            w.u32(u.card);
            enc_pid(w, &u.player);
            w.len(u.targets.len());
            for t in &u.targets {
                w.u8(*t);
            }
            enc_damage_nature(w, &u.nature);
            enc_opt_u32(w, u.use_as.map(|k| card_kind_to_num(&k)));
            w.boolean(u.cancelled);
        }
        EventData::CardResponded { card, player } => {
            w.u8(5);
            w.u32(*card);
            enc_pid(w, player);
        }
        EventData::Dying { player, source } => {
            w.u8(6);
            enc_pid(w, player);
            enc_opt_pid(w, source);
        }
        EventData::Death { player, killer } => {
            w.u8(7);
            enc_pid(w, player);
            enc_opt_pid(w, killer);
        }
        EventData::Phase {
            player,
            phase,
            begin,
        } => {
            w.u8(8);
            enc_pid(w, player);
            enc_phase(w, phase);
            w.boolean(*begin);
        }
        EventData::TurnEnded { player } => {
            w.u8(9);
            enc_pid(w, player);
        }
        EventData::TurnOverBefore { player, cancelled } => {
            // E03 翻面原语（tag 14，表尾追加）
            w.u8(14);
            enc_pid(w, player);
            w.boolean(*cancelled);
        }
        EventData::TurnOvered { player, face_down } => {
            // E03 翻面完成（tag 15，表尾追加）
            w.u8(15);
            enc_pid(w, player);
            w.boolean(*face_down);
        }
        EventData::SkillToggled {
            player,
            skill,
            enabled,
            temporary,
        } => {
            w.u8(10);
            enc_pid(w, player);
            enc_skill(w, skill);
            w.boolean(*enabled);
            w.boolean(*temporary);
        }
        EventData::Equipped {
            player,
            card,
            replaced,
        } => {
            w.u8(11);
            enc_pid(w, player);
            w.u32(*card);
            enc_opt_u32(w, *replaced);
        }
        EventData::Linked { player, linked } => {
            w.u8(12);
            enc_pid(w, player);
            w.boolean(*linked);
        }
        EventData::CardsDiscarded { player, cards } => {
            w.u8(13);
            enc_pid(w, player);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        EventData::LoseHp { player, amount } => {
            // 逐技对照批三（tag 16，表尾追加）
            w.u8(16);
            enc_pid(w, player);
            w.u32(*amount);
        }
        EventData::DyingSaved { player, source } => {
            // 濒死救回（tag 17，表尾追加；焚逝挂点规则 v157）
            w.u8(17);
            enc_pid(w, player);
            enc_opt_pid(w, source);
        }
    }
}

fn dec_event_data(r: &mut R<'_>) -> Result<EventData, ArchiveError> {
    Ok(match r.u8()? {
        0 => {
            let source = dec_opt_pid(r)?;
            let target = dec_pid(r)?;
            let amount = r.u32()?;
            let nature = dec_damage_nature(r)?;
            let prevented = r.boolean()?;
            let chain = r.vec(|r| r.u8())?;
            let card = dec_opt_u32(r)?;
            let from_chain = r.boolean()?;
            EventData::Damage(DamageData {
                source,
                target,
                amount,
                nature,
                prevented,
                redirect_chain: chain,
                card,
                from_chain,
            })
        }
        1 => {
            let player = dec_pid(r)?;
            let amount = r.u32()?;
            let by_card = dec_opt_u32(r)?;
            EventData::Recover {
                player,
                amount,
                by_card,
            }
        }
        2 => {
            let player = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            EventData::CardsDrawn { player, cards }
        }
        3 => {
            let cards = r.vec(|r| r.u32())?;
            let from = dec_pid(r)?;
            let to = dec_pid(r)?;
            EventData::CardsMoved { cards, from, to }
        }
        4 => {
            let card = r.u32()?;
            let player = dec_pid(r)?;
            let targets = r.vec(|r| r.u8())?;
            let nature = dec_damage_nature(r)?;
            let use_as = match dec_opt_u32(r)? {
                Some(n) => Some(card_kind_from_num(n)?),
                None => None,
            };
            let cancelled = r.boolean()?;
            EventData::CardUsed(CardUseData {
                card,
                player,
                targets,
                nature,
                use_as,
                cancelled,
            })
        }
        5 => {
            let card = r.u32()?;
            let player = dec_pid(r)?;
            EventData::CardResponded { card, player }
        }
        6 => {
            let player = dec_pid(r)?;
            let source = dec_opt_pid(r)?;
            EventData::Dying { player, source }
        }
        7 => {
            let player = dec_pid(r)?;
            let killer = dec_opt_pid(r)?;
            EventData::Death { player, killer }
        }
        8 => {
            let player = dec_pid(r)?;
            let phase = dec_phase(r)?;
            let begin = r.boolean()?;
            EventData::Phase {
                player,
                phase,
                begin,
            }
        }
        9 => {
            let player = dec_pid(r)?;
            EventData::TurnEnded { player }
        }
        14 => {
            let player = dec_pid(r)?;
            let cancelled = r.boolean()?;
            EventData::TurnOverBefore { player, cancelled }
        }
        15 => {
            let player = dec_pid(r)?;
            let face_down = r.boolean()?;
            EventData::TurnOvered { player, face_down }
        }
        16 => {
            let player = dec_pid(r)?;
            let amount = r.u32()?;
            EventData::LoseHp { player, amount }
        }
        17 => {
            let player = dec_pid(r)?;
            let source = dec_opt_pid(r)?;
            EventData::DyingSaved { player, source }
        }
        10 => {
            let player = dec_pid(r)?;
            let skill = dec_skill(r)?;
            let enabled = r.boolean()?;
            let temporary = r.boolean()?;
            EventData::SkillToggled {
                player,
                skill,
                enabled,
                temporary,
            }
        }
        11 => {
            let player = dec_pid(r)?;
            let card = r.u32()?;
            let replaced = dec_opt_u32(r)?;
            EventData::Equipped {
                player,
                card,
                replaced,
            }
        }
        12 => {
            let player = dec_pid(r)?;
            let linked = r.boolean()?;
            EventData::Linked { player, linked }
        }
        13 => {
            let player = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            EventData::CardsDiscarded { player, cards }
        }
        t => return Err(ArchiveError::BadTag("event_data", t)),
    })
}

// ---- 机器快照 ----

fn enc_judgement_progress(w: &mut W, c: &JudgementProgressSnap) {
    w.u32(c.trick);
    w.u32(c.judge_card);
    enc_pids(w, &c.rewriters);
    w.boolean(c.asked);
    enc_pids(w, &c.tian_du);
    w.boolean(c.gained);
    w.boolean(c.in_rewrite);
    w.boolean(c.finalized);
    w.boolean(c.judge_held);
    // v75（swd 卡牌面窗口机批）：昊天塔插窗两字段（尾部追加）
    enc_opt_pid(w, &c.haotian_holder);
    match c.haotian_other {
        None => w.u8(0),
        Some(x) => {
            w.u8(1);
            w.u32(x);
        }
    }
    // v89（T11 Boss 诸葛亮族批 116）：改命 7 张候选（尾部追加）
    match &c.gaiming_cards {
        None => w.u8(0),
        Some(cards) => {
            w.u8(1);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
    }
}

fn dec_judgement_progress(r: &mut R<'_>) -> Result<JudgementProgressSnap, ArchiveError> {
    let trick = r.u32()?;
    let judge_card = r.u32()?;
    let rewriters = dec_pids(r)?;
    let asked = r.boolean()?;
    let tian_du = dec_pids(r)?;
    let gained = r.boolean()?;
    let in_rewrite = r.boolean()?;
    let finalized = r.boolean()?;
    let judge_held = r.boolean()?;
    // v75（swd 卡牌面窗口机批）：昊天塔插窗两字段（尾部追加）
    let haotian_holder = dec_opt_pid(r)?;
    let haotian_other = match r.u8()? {
        0 => None,
        1 => Some(r.u32()?),
        _ => return Err(ArchiveError::UnknownCardKind(255)),
    };
    // v89：改命候选
    let gaiming_cards = match r.u8()? {
        0 => None,
        1 => {
            let len = r.len()?;
            let mut cards = Vec::with_capacity(len);
            for _ in 0..len {
                cards.push(r.u32()?);
            }
            Some(cards)
        }
        _ => return Err(ArchiveError::UnknownCardKind(255)),
    };
    Ok(JudgementProgressSnap {
        trick,
        judge_card,
        rewriters,
        asked,
        tian_du,
        gained,
        in_rewrite,
        finalized,
        judge_held,
        haotian_holder,
        haotian_other,
        gaiming_cards,
    })
}

fn enc_machine(w: &mut W, m: &MachineSnapshot) {
    match m {
        MachineSnapshot::Sha {
            card,
            source,
            target,
            nature,
            asked,
            armor_judged,
            dodges_used,
            tieji_red,
            liuli_done,
            liuli_stage,
            liuli_card,
            hujia_queue,
            hujia_holder,
            jiu_bonus,
            asking_qilin,
            qilin_horse,
            asking_hanbing,
            asking_fengxueren,
            asking_guanshi,
            guanshi_cards,
            asking_qinglong,
            asking_cixiong,
            cixiong_done,
            extra_targets,
            hanbing_pending,
            asking_xiandan,
            xiandan_done,
            xiandan_bonus,
            asking_sadeng,
            asking_sadeng_shan,
            shoulie_direct,
            asking_lueying,
            lueying_victim,
            lueying_declined,
            asking_fuchou,
            fuchou_card,
        } => {
            w.u8(0);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
            enc_damage_nature(w, nature);
            w.boolean(*asked);
            w.boolean(*armor_judged);
            w.u8(*dodges_used);
            w.u8(match tieji_red {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            });
            // N05（ADR 0774/0776）：流离/护驾状态
            w.boolean(*liuli_done);
            w.u8(*liuli_stage);
            match liuli_card {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            // 护驾逐人询问队列：None=0，Some=1+len+pid 序列
            match hujia_queue {
                None => w.u8(0),
                Some(q) => {
                    w.u8(1);
                    w.len(q.len());
                    for p in q {
                        enc_pid(w, p);
                    }
                }
            }
            enc_opt_pid(w, hujia_holder);
            w.boolean(*jiu_bonus);
            w.boolean(*asking_qilin);
            match qilin_horse {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            w.boolean(*asking_hanbing);
            w.boolean(*asking_fengxueren);
            w.boolean(*asking_guanshi);
            w.len(guanshi_cards.len());
            for c in guanshi_cards {
                w.u32(*c);
            }
            w.boolean(*asking_qinglong);
            w.boolean(*asking_cixiong);
            w.boolean(*cixiong_done);
            w.len(extra_targets.len());
            for c in extra_targets {
                w.u32(*c);
            }
            w.u8(*hanbing_pending);
            w.boolean(*asking_xiandan);
            w.boolean(*xiandan_done);
            w.boolean(*xiandan_bonus);
            w.boolean(*asking_lueying);
            match lueying_victim {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            w.boolean(*lueying_declined);
            w.boolean(*asking_fuchou);
            match fuchou_card {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            // v75（swd 卡牌面窗口机批）：萨登荆环两询问挂起旗（尾部追加）
            w.boolean(*asking_sadeng);
            w.boolean(*asking_sadeng_shan);
            // v78（swd deferred 消解批 95）：狩猎不可闪避旗（尾部追加）
            w.boolean(*shoulie_direct);
        }
        MachineSnapshot::Zhangba {
            owner,
            cards,
            stage,
        } => {
            w.u8(60);
            enc_pid(w, owner);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            w.u8(match stage {
                ZhangbaStageSnap::Cards => 0,
                ZhangbaStageSnap::Target => 1,
                ZhangbaStageSnap::TargetRead => 2,
            });
        }
        MachineSnapshot::BossXiangde { owner } => {
            w.u8(61);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossFutai { owner } => {
            w.u8(62);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossLuanchang { owner } => {
            w.u8(63);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossZhaohuo { owner } => {
            w.u8(65);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossBuchun { owner, stage, targets } => {
            w.u8(66);
            enc_pid(w, owner);
            w.u8(*stage);
            w.u32(targets.len() as u32);
            for t in targets {
                w.u32(*t);
            }
        }
        MachineSnapshot::BossXingxia { owner, stage, actor } => {
            w.u8(67);
            enc_pid(w, owner);
            w.u8(*stage);
            w.u32(*actor);
        }
        MachineSnapshot::BossQizuo { owner } => {
            w.u8(68);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossXiaoxiong { owner } => {
            w.u8(69);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossLingbo { owner } => {
            w.u8(70);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossTashui { owner } => {
            w.u8(71);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossZjLeiji { owner } => {
            w.u8(72);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossHeisheng { owner } => {
            w.u8(73);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossLeizhu { owner, emitted } => {
            w.u8(74);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossRenao { owner, emitted } => {
            w.u8(75);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossWanghun { owner } => {
            w.u8(76);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossZhuxin { owner, emitted } => {
            w.u8(77);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossPingdeng { owner, emitted } => {
            w.u8(78);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossHuoxing { owner, emitted } => {
            w.u8(79);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossBingfeng { owner } => {
            w.u8(80);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossWangxiang { owner } => {
            w.u8(81);
            enc_pid(w, owner);
        }
        MachineSnapshot::Rvm {
            owner,
            module,
            paused,
        } => {
            w.u8(82);
            enc_pid(w, owner);
            w.bytes(module);
            match paused {
                Some(p) => {
                    w.u8(1);
                    w.bytes(&p.frames);
                    w.u32(p.pc);
                    w.u32(p.fuel_remaining);
                    w.u64(p.id);
                    w.bytes(&p.module_hash);
                    w.u32(p.options.len() as u32);
                    for t in &p.options {
                        w.u32(*t);
                    }
                    w.bytes(&p.pending);
                }
                None => w.u8(0),
            }
        }
        MachineSnapshot::Shedu { owner } => {
            w.u8(64);
            enc_pid(w, owner);
        }
        MachineSnapshot::DyingSave {
            dying,
            source,
            responders,
            asked,
            round_used,
            guihan_asked,
        } => {
            w.u8(1);
            enc_pid(w, dying);
            enc_opt_pid(w, source);
            enc_pids(w, responders);
            w.boolean(*asked);
            w.boolean(*round_used);
            w.boolean(*guihan_asked);
        }
        MachineSnapshot::Duel {
            card,
            source,
            target,
            on_target,
            asked,
            second_pending,
        } => {
            w.u8(2);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
            w.boolean(*on_target);
            w.boolean(*asked);
            w.boolean(*second_pending);
        }
        MachineSnapshot::NullifyWindow {
            card,
            source,
            nullified,
            queue,
            asked,
            window_open,
            owns_disposal,
            effect,
        } => {
            w.u8(3);
            w.u32(*card);
            enc_pid(w, source);
            w.boolean(*nullified);
            enc_pids(w, queue);
            w.boolean(*asked);
            w.boolean(*window_open);
            w.boolean(*owns_disposal);
            match effect {
                Some(e) => {
                    w.u8(1);
                    enc_machine(w, e);
                }
                None => w.u8(0),
            }
        }
        MachineSnapshot::JudgePhase { owner, queue, current } => {
            w.u8(4);
            enc_pid(w, owner);
            w.len(queue.len());
            for c in queue {
                w.u32(*c);
            }
            match current {
                Some(c) => {
                    w.u8(1);
                    enc_judgement_progress(w, c);
                }
                None => w.u8(0),
            }
        }
        MachineSnapshot::Noop => w.u8(5),
        MachineSnapshot::PlaceDelayed { card, target, placed } => {
            w.u8(6);
            w.u32(*card);
            enc_pid(w, target);
            w.boolean(*placed);
        }
        MachineSnapshot::Aoe {
            card,
            source,
            need,
            targets,
            idx,
            inner,
            nature,
        } => {
            w.u8(7);
            w.u32(*card);
            enc_pid(w, source);
            // need: CardKind 按数值编码
            w.u32(card_kind_to_num(need));
            w.len(targets.len());
            for t in targets {
                w.u8(*t);
            }
            w.u64(*idx as u64);
            match inner {
                Some(e) => {
                    w.u8(1);
                    enc_machine(w, e);
                }
                None => w.u8(0),
            }
            // AOE 伤害属性（批⑦）
            w.u8(*nature as u8);
        }
        MachineSnapshot::AoeTarget {
            source,
            target,
            need,
            asked,
            card,
            nature,
        } => {
            w.u8(8);
            enc_pid(w, source);
            enc_pid(w, target);
            w.u32(card_kind_to_num(need));
            w.boolean(*asked);
            w.u32(*card);
            w.u8(*nature as u8);
        }
        MachineSnapshot::Chain {
            card,
            source,
            targets,
            idx,
            inner,
            dispatcher,
        } => {
            w.u8(9);
            w.u32(*card);
            enc_pid(w, source);
            w.len(targets.len());
            for t in targets {
                w.u8(*t);
            }
            w.u64(*idx as u64);
            match inner {
                None => w.u8(0),
                Some(e) => {
                    w.u8(1);
                    enc_machine(w, e);
                }
            }
            w.boolean(*dispatcher);
        }
        MachineSnapshot::DiscardToLimit { player, need, asked } => {
            w.u8(10);
            enc_pid(w, player);
            w.u64(*need as u64);
            w.boolean(*asked);
        }
        MachineSnapshot::Death { player, killer } => {
            w.u8(11);
            enc_pid(w, player);
            enc_opt_pid(w, killer);
        }
        MachineSnapshot::OptionalGate {
            owner,
            def,
            event,
            asked,
            inner,
        } => {
            w.u8(12);
            enc_pid(w, owner);
            enc_skill(w, def);
            enc_event_data(w, event);
            w.boolean(*asked);
            match inner {
                Some(e) => {
                    w.u8(1);
                    enc_machine(w, e);
                }
                None => w.u8(0),
            }
        }
        MachineSnapshot::SanCai {
            owner,
            stage,
            target,
        } => {
            w.u8(13);
            enc_pid(w, owner);
            w.u8(match stage {
                SanCaiStageSnap::Draw => 0,
                SanCaiStageSnap::Target => 1,
                SanCaiStageSnap::GiveCard => 2,
                SanCaiStageSnap::Finish => 3,
            });
            enc_opt_pid(w, target);
        }
        MachineSnapshot::YiShang { owner, stage } => {
            w.u8(14);
            enc_pid(w, owner);
            w.u8(match stage {
                YiShangStageSnap::Target => 0,
                YiShangStageSnap::Apply => 1,
            });
        }
        MachineSnapshot::SuiDun => w.u8(15),
        MachineSnapshot::SuQing { target } => {
            w.u8(16);
            enc_pid(w, target);
        }
        MachineSnapshot::ShouYi { owner } => {
            w.u8(17);
            enc_pid(w, owner);
        }
        MachineSnapshot::FanKui {
            owner,
            source,
            asked,
        } => {
            w.u8(18);
            enc_pid(w, owner);
            enc_opt_pid(w, source);
            w.boolean(*asked);
        }
        MachineSnapshot::JianXiong { owner, card, asked } => {
            w.u8(19);
            enc_pid(w, owner);
            w.u32(*card);
            w.boolean(*asked);
        }
        MachineSnapshot::BiYue { owner } => {
            w.u8(21);
            enc_pid(w, owner);
        }
        MachineSnapshot::JiZhi { owner } => {
            w.u8(25);
            enc_pid(w, owner);
        }
        MachineSnapshot::ZhiHeng { owner, stage } => {
            w.u8(27);
            enc_pid(w, owner);
            w.u8(match stage {
                ZhiHengStageSnap::Select => 0,
                ZhiHengStageSnap::Draw => 1,
            });
        }
        MachineSnapshot::KuRou { owner, stage } => {
            w.u8(26);
            enc_pid(w, owner);
            w.u8(match stage {
                KuRouStageSnap::Lose => 0,
                KuRouStageSnap::Draw => 1,
            });
        }
        MachineSnapshot::LuoShen { owner } => {
            w.u8(22);
            enc_pid(w, owner);
        }
        MachineSnapshot::YingZi { owner } => {
            w.u8(28);
            enc_pid(w, owner);
        }
        MachineSnapshot::Dismantle {
            card,
            source,
            target,
            asked,
        } => {
            w.u8(29);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
            w.boolean(*asked);
        }
        MachineSnapshot::MonkeySteal {
            card,
            user,
            holders,
            asked,
        } => {
            // E03 猴子偷桃：使用前取消窗口（tag 44，表尾追加）
            w.u8(44);
            w.u32(*card);
            enc_pid(w, user);
            enc_pids(w, holders);
            w.boolean(*asked);
        }
        MachineSnapshot::XiaoJi { owner } => {
            // tag 43（表尾追加）：tag 29 曾与 Dismantle 冲突（解码分支
            // 不可达，枭姬挂起无法恢复），2026-09-24 经 A0 唯一性检查
            // 定位后修正；旧字节中 XiaoJi 挂起本就无法正确解码。
            w.u8(43);
            enc_pid(w, owner);
        }
        MachineSnapshot::QingNang {
            owner,
            stage,
            healed,
        } => {
            w.u8(30);
            enc_pid(w, owner);
            w.u8(match stage {
                QingNangStageSnap::TargetAsk => 0,
                QingNangStageSnap::TargetRead => 1,
                QingNangStageSnap::CardRead => 2,
            });
            enc_opt_pid(w, healed);
        }
        MachineSnapshot::JiJi {
            owner,
            stage,
            target,
            given,
        } => {
            w.u8(20);
            enc_pid(w, owner);
            w.u8(match stage {
                JiJiStageSnap::Draw => 0,
                JiJiStageSnap::Target => 1,
                JiJiStageSnap::Retarget => 2,
                JiJiStageSnap::GiveCard => 3,
                JiJiStageSnap::GiveChooseCard => 4,
            });
            enc_opt_pid(w, target);
            w.u8(*given);
        }
        MachineSnapshot::GangLie {
            owner,
            source,
            asked,
        } => {
            w.u8(31);
            enc_pid(w, owner);
            enc_pid(w, source);
            w.u8(*asked as u8);
        }
        MachineSnapshot::XiaoGuo {
            owner,
            target,
            stage,
        } => {
            w.u8(45);
            enc_pid(w, owner);
            enc_pid(w, target);
            w.u8(match stage {
                crate::snap::XiaoGuoStageSnap::Ask => 0,
                crate::snap::XiaoGuoStageSnap::Read => 1,
            });
        }
        MachineSnapshot::KuangFu { owner, target } => {
            w.u8(46);
            enc_pid(w, owner);
            enc_pid(w, target);
        }
        MachineSnapshot::YaoWu {
            owner,
            source,
            stage,
        } => {
            w.u8(47);
            enc_pid(w, owner);
            enc_pid(w, source);
            w.u8(match stage {
                crate::snap::YaoWuStageSnap::Ask => 0,
                crate::snap::YaoWuStageSnap::Read => 1,
            });
        }
        MachineSnapshot::ShuShen {
            owner,
            target,
            stage,
        } => {
            w.u8(48);
            enc_pid(w, owner);
            enc_opt_pid(w, target);
            w.u8(match stage {
                ShuShenStageSnap::Ask => 0,
                ShuShenStageSnap::Read => 1,
            });
        }
        MachineSnapshot::ShenZhi { owner } => {
            w.u8(49);
            enc_pid(w, owner);
        }
        MachineSnapshot::XunXun {
            owner,
            stage,
            top,
        } => {
            w.u8(50);
            enc_pid(w, owner);
            w.u8(match stage {
                XunXunStageSnap::Undo => 0,
                XunXunStageSnap::Arrange => 1,
            });
            w.len(top.len());
            for c in top {
                w.u32(*c);
            }
        }
        MachineSnapshot::WangXi { owner, other } => {
            w.u8(51);
            enc_pid(w, owner);
            enc_pid(w, other);
        }
        MachineSnapshot::JiJie {
            owner,
            card,
            stage,
        } => {
            w.u8(52);
            enc_pid(w, owner);
            match card {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            w.u8(match stage {
                JiJieStageSnap::Ask => 0,
                JiJieStageSnap::Read => 1,
            });
        }
        MachineSnapshot::JiYuan { owner } => {
            w.u8(53);
            enc_pid(w, owner);
        }
        MachineSnapshot::WangZun { owner, target } => {
            w.u8(54);
            enc_pid(w, owner);
            enc_pid(w, target);
        }
        MachineSnapshot::TongJi { owner, stage } => {
            w.u8(55);
            enc_pid(w, owner);
            w.u8(match stage {
                TongJiStageSnap::Ask => 0,
                TongJiStageSnap::Read => 1,
            });
        }
        MachineSnapshot::Wuzhong { owner } => {
            w.u8(56);
            enc_pid(w, owner);
        }
        MachineSnapshot::Taoyuan { owner } => {
            w.u8(57);
            enc_pid(w, owner);
        }
        MachineSnapshot::Wugu {
            owner,
            cards,
            pickers,
        } => {
            w.u8(58);
            enc_pid(w, owner);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            w.len(pickers.len());
            for p in pickers {
                enc_pid(w, p);
            }
        }
        MachineSnapshot::Jiedao {
            owner,
            holder,
            victim,
            stage,
        } => {
            w.u8(59);
            enc_pid(w, owner);
            enc_pid(w, holder);
            enc_pid(w, victim);
            w.u8(match stage {
                JiedaoStageSnap::Ask => 0,
                JiedaoStageSnap::Read => 1,
            });
        }
        MachineSnapshot::TuXi { owner, stage } => {
            w.u8(32);
            enc_pid(w, owner);
            w.u8(match stage {
                TuXiStageSnap::Ask => 0,
                TuXiStageSnap::Read => 1,
            });
        }
        MachineSnapshot::LuoYi { owner } => {
            w.u8(33);
            enc_pid(w, owner);
        }
        MachineSnapshot::GuanXing { owner, stage, top } => {
            w.u8(34);
            enc_pid(w, owner);
            w.u8(match stage {
                GuanXingStageSnap::Arrange => 0,
                GuanXingStageSnap::Place => 1,
            });
            w.len(top.len());
            for c in top {
                w.u32(*c);
            }
        }
        MachineSnapshot::LianYing { owner } => {
            w.u8(35);
            enc_pid(w, owner);
        }
        MachineSnapshot::Rende { owner, stage, cards } => {
            w.u8(36);
            enc_pid(w, owner);
            w.u8(match stage {
                RendeStageSnap::SelectCardsAsk => 0,
                RendeStageSnap::TargetAsk => 1,
                RendeStageSnap::TargetRead => 2,
            });
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        MachineSnapshot::Jijiang {
            owner,
            stage,
            queue,
            helper,
            sha,
            inner,
        } => {
            w.u8(37);
            enc_pid(w, owner);
            w.u8(match stage {
                JijiangStageSnap::HelperAsk => 0,
                JijiangStageSnap::TargetAsk => 1,
            });
            // 逐人询问队列：None=0，Some=1+len+pid 序列
            match queue {
                None => w.u8(0),
                Some(q) => {
                    w.u8(1);
                    w.len(q.len());
                    for p in q {
                        enc_pid(w, p);
                    }
                }
            }
            enc_opt_pid(w, helper);
            enc_opt_u32(w, *sha);
            match inner {
                Some(m) => {
                    w.u8(1);
                    enc_machine(w, m);
                }
                None => w.u8(0),
            }
        }
        MachineSnapshot::Snatch {
            card,
            source,
            target,
            asked,
        } => {
            w.u8(38);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
            w.boolean(*asked);
        }
        MachineSnapshot::JieYin { owner, stage, cards } => {
            w.u8(39);
            enc_pid(w, owner);
            w.u8(match stage {
                JieYinStageSnap::CardsAsk => 0,
                JieYinStageSnap::TargetAsk => 1,
                JieYinStageSnap::TargetRead => 2,
            });
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        MachineSnapshot::FanJian { owner, stage, target, chosen_suit } => {
            w.u8(40);
            enc_pid(w, owner);
            w.u8(match stage {
                FanJianStageSnap::TargetAsk => 0,
                FanJianStageSnap::SuitAsk => 1,
                FanJianStageSnap::PickAsk => 2,
                FanJianStageSnap::Consume => 3,
            });
            match *chosen_suit {
                None => w.u8(0),
                Some(t) => {
                    w.u8(1);
                    w.u8(t);
                }
            }
            enc_opt_pid(w, target);
        }
        MachineSnapshot::LiJian { owner, stage, card, inner } => {
            w.u8(41);
            enc_pid(w, owner);
            w.u8(match stage {
                LiJianStageSnap::DiscardAsk => 0,
                LiJianStageSnap::TargetsAsk => 1,
                LiJianStageSnap::Launch => 2,
            });
            enc_opt_u32(w, *card);
            match inner {
                Some(m) => {
                    w.u8(1);
                    enc_machine(w, m);
                }
                None => w.u8(0),
            }
        }
        MachineSnapshot::Host { def_key, behavior, stage, owner, target, card, cards, targets } => {
            w.u8(42);
            // 定义键：有界字符串（解码侧限长 64）
            w.str(def_key);
            w.u16(*behavior);
            w.u8(*stage);
            enc_pid(w, owner);
            enc_opt_pid(w, target);
            enc_opt_u32(w, *card);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            w.len(targets.len());
            for p in targets {
                enc_pid(w, p);
            }
        }
    }
}

fn dec_machine(r: &mut R<'_>) -> Result<MachineSnapshot, ArchiveError> {
    Ok(match r.u8()? {
        0 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let nature = dec_damage_nature(r)?;
            let asked = r.boolean()?;
            let armor_judged = r.boolean()?;
            let dodges_used = r.u8()?;
            let tieji_red = match r.u8()? {
                0 => None,
                1 => Some(false),
                2 => Some(true),
                t => return Err(ArchiveError::BadTag("tieji_red", t)),
            };
            let liuli_done = r.boolean()?;
            let liuli_stage = r.u8()?;
            let liuli_card = match r.u8()? {
                0 => None,
                1 => Some(r.u32()?),
                t => return Err(ArchiveError::BadTag("sha_liuli_card", t)),
            };
            let hujia_queue = match r.u8()? {
                0 => None,
                1 => Some(r.vec(dec_pid)?),
                t => return Err(ArchiveError::BadTag("hujia_queue", t)),
            };
            let hujia_holder = dec_opt_pid(r)?;
            let jiu_bonus = r.boolean()?;
            let asking_qilin = r.boolean()?;
            let qilin_horse = match r.u8()? {
                0 => None,
                1 => Some(r.u32()?),
                t => return Err(ArchiveError::BadTag("sha_qilin_horse", t)),
            };
            let asking_hanbing = r.boolean()?;
            let asking_fengxueren = r.boolean()?;
            let asking_guanshi = r.boolean()?;
            let guanshi_cards = r.vec(|r| r.u32())?;
            let asking_qinglong = r.boolean()?;
            let asking_cixiong = r.boolean()?;
            let cixiong_done = r.boolean()?;
            let extra_targets = r.vec(|r| r.u32())?;
            let hanbing_pending = r.u8()?;
            let asking_xiandan = r.boolean()?;
            let xiandan_done = r.boolean()?;
            let xiandan_bonus = r.boolean()?;
            let asking_lueying = r.boolean()?;
            let lueying_victim = match r.u8()? {
                0 => None,
                1 => Some(r.u32()?),
                _ => return Err(ArchiveError::UnknownCardKind(255)),
            };
            let lueying_declined = r.boolean()?;
            let asking_fuchou = r.boolean()?;
            let fuchou_card = dec_opt_u32(r)?;
            // v75（swd 卡牌面窗口机批）：萨登荆环两询问挂起旗（尾部追加）
            let asking_sadeng = r.boolean()?;
            let asking_sadeng_shan = r.boolean()?;
            // v78（swd deferred 消解批 95）：狩猎不可闪避旗（尾部追加）
            let shoulie_direct = r.boolean()?;
            MachineSnapshot::Sha {
                card,
                source,
                target,
                nature,
                asked,
                armor_judged,
                dodges_used,
                tieji_red,
                liuli_done,
                liuli_stage,
                liuli_card,
                hujia_queue,
                hujia_holder,
                jiu_bonus,
                asking_qilin,
                qilin_horse,
                asking_hanbing,
                asking_fengxueren,
                asking_guanshi,
                guanshi_cards,
                asking_qinglong,
                asking_cixiong,
                cixiong_done,
                extra_targets,
                hanbing_pending,
                asking_xiandan,
                xiandan_done,
                xiandan_bonus,
                asking_lueying,
                lueying_victim,
                lueying_declined,
                asking_fuchou,
                fuchou_card,
                asking_sadeng,
                asking_sadeng_shan,
                shoulie_direct,
            }
        }
        1 => {
            let dying = dec_pid(r)?;
            let source = dec_opt_pid(r)?;
            let responders = dec_pids(r)?;
            let asked = r.boolean()?;
            let round_used = r.boolean()?;
            let guihan_asked = r.boolean()?;
            MachineSnapshot::DyingSave {
                dying,
                source,
                responders,
                asked,
                round_used,
                guihan_asked,
            }
        }
        2 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let on_target = r.boolean()?;
            let asked = r.boolean()?;
            let second_pending = r.boolean()?;
            MachineSnapshot::Duel {
                card,
                source,
                target,
                on_target,
                asked,
                second_pending,
            }
        }
        3 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let nullified = r.boolean()?;
            let queue = dec_pids(r)?;
            let asked = r.boolean()?;
            let window_open = r.boolean()?;
            let owns_disposal = r.boolean()?;
            let effect = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("opt_machine", t)),
            };
            MachineSnapshot::NullifyWindow {
                card,
                source,
                nullified,
                queue,
                asked,
                window_open,
                owns_disposal,
                effect,
            }
        }
        4 => {
            let owner = dec_pid(r)?;
            let queue = r.vec(|r| r.u32())?;
            let current = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_judgement_progress(r)?)),
                t => return Err(ArchiveError::BadTag("opt_judge", t)),
            };
            MachineSnapshot::JudgePhase {
                owner,
                queue,
                current,
            }
        }
        5 => MachineSnapshot::Noop,
        6 => {
            let card = r.u32()?;
            let target = dec_pid(r)?;
            let placed = r.boolean()?;
            MachineSnapshot::PlaceDelayed { card, target, placed }
        }
        7 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let need = card_kind_from_num(r.u32()?)?;
            let targets = r.vec(|r| r.u8())?;
            let idx = r.u64()? as usize;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("opt_machine", t)),
            };
            let nature = nature_from_num(r.u8()?)?;
            MachineSnapshot::Aoe {
                card,
                source,
                need,
                targets,
                idx,
                inner,
                nature,
            }
        }
        8 => {
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let need = card_kind_from_num(r.u32()?)?;
            let asked = r.boolean()?;
            let card = r.u32()?;
            let nature = nature_from_num(r.u8()?)?;
            MachineSnapshot::AoeTarget {
                source,
                target,
                need,
                asked,
                card,
                nature,
            }
        }
        9 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let targets = r.vec(|r| r.u8())?;
            let idx = r.u64()? as usize;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("opt_machine", t)),
            };
            let dispatcher = r.boolean()?;
            MachineSnapshot::Chain {
                card,
                source,
                targets,
                idx,
                inner,
                dispatcher,
            }
        }
        10 => {
            let player = dec_pid(r)?;
            let need = r.u64()? as usize;
            let asked = r.boolean()?;
            MachineSnapshot::DiscardToLimit { player, need, asked }
        }
        11 => {
            let player = dec_pid(r)?;
            let killer = dec_opt_pid(r)?;
            MachineSnapshot::Death { player, killer }
        }
        12 => {
            let owner = dec_pid(r)?;
            let def = dec_skill(r)?;
            let event = dec_event_data(r)?;
            let asked = r.boolean()?;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("opt_machine", t)),
            };
            MachineSnapshot::OptionalGate {
                owner,
                def,
                event,
                asked,
                inner,
            }
        }
        13 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => SanCaiStageSnap::Draw,
                1 => SanCaiStageSnap::Target,
                2 => SanCaiStageSnap::GiveCard,
                3 => SanCaiStageSnap::Finish,
                t => return Err(ArchiveError::BadTag("sancai_stage", t)),
            };
            let target = dec_opt_pid(r)?;
            MachineSnapshot::SanCai {
                owner,
                stage,
                target,
            }
        }
        14 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => YiShangStageSnap::Target,
                1 => YiShangStageSnap::Apply,
                t => return Err(ArchiveError::BadTag("yishang_stage", t)),
            };
            MachineSnapshot::YiShang { owner, stage }
        }
        15 => MachineSnapshot::SuiDun,
        16 => {
            let target = dec_pid(r)?;
            MachineSnapshot::SuQing { target }
        }
        17 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::ShouYi { owner }
        }
        18 => {
            let owner = dec_pid(r)?;
            let source = dec_opt_pid(r)?;
            let asked = r.boolean()?;
            MachineSnapshot::FanKui {
                owner,
                source,
                asked,
            }
        }
        19 => {
            let owner = dec_pid(r)?;
            let card = r.u32()?;
            let asked = r.boolean()?;
            MachineSnapshot::JianXiong { owner, card, asked }
        }
        20 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => JiJiStageSnap::Draw,
                1 => JiJiStageSnap::Target,
                2 => JiJiStageSnap::Retarget,
                3 => JiJiStageSnap::GiveCard,
                4 => JiJiStageSnap::GiveChooseCard,
                t => return Err(ArchiveError::BadTag("jiji_stage", t)),
            };
            let target = dec_opt_pid(r)?;
            let given = r.u8()?;
            MachineSnapshot::JiJi {
                owner,
                stage,
                target,
                given,
            }
        }
        21 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BiYue { owner }
        }
        25 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::JiZhi { owner }
        }
        27 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => ZhiHengStageSnap::Select,
                1 => ZhiHengStageSnap::Draw,
                t => return Err(ArchiveError::BadTag("zhiheng_stage", t)),
            };
            MachineSnapshot::ZhiHeng { owner, stage }
        }
        26 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => KuRouStageSnap::Lose,
                1 => KuRouStageSnap::Draw,
                t => return Err(ArchiveError::BadTag("kurou_stage", t)),
            };
            MachineSnapshot::KuRou { owner, stage }
        }
        22 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::LuoShen { owner }
        }
        28 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::YingZi { owner }
        }
        29 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let asked = r.boolean()?;
            MachineSnapshot::Dismantle {
                card,
                source,
                target,
                asked,
            }
        }
        43 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::XiaoJi { owner }
        }
        44 => {
            let card = r.u32()?;
            let user = dec_pid(r)?;
            let holders = r.vec(|r| dec_pid(r))?;
            let asked = r.boolean()?;
            MachineSnapshot::MonkeySteal {
                card,
                user,
                holders,
                asked,
            }
        }
        30 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => QingNangStageSnap::TargetAsk,
                1 => QingNangStageSnap::TargetRead,
                2 => QingNangStageSnap::CardRead,
                t => return Err(ArchiveError::BadTag("qingnang_stage", t)),
            };
            let healed = dec_opt_pid(r)?;
            MachineSnapshot::QingNang {
                owner,
                stage,
                healed,
            }
        }
        31 => {
            let owner = dec_pid(r)?;
            let source = dec_pid(r)?;
            let asked = r.u8()? != 0;
            MachineSnapshot::GangLie {
                owner,
                source,
                asked,
            }
        }
        45 => {
            let owner = dec_pid(r)?;
            let target = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => XiaoGuoStageSnap::Ask,
                1 => XiaoGuoStageSnap::Read,
                other => return Err(ArchiveError::BadTag("xiao_guo_stage", other)),
            };
            MachineSnapshot::XiaoGuo { owner, target, stage }
        }
        46 => {
            let owner = dec_pid(r)?;
            let target = dec_pid(r)?;
            MachineSnapshot::KuangFu { owner, target }
        }
        47 => {
            let owner = dec_pid(r)?;
            let source = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => YaoWuStageSnap::Ask,
                1 => YaoWuStageSnap::Read,
                other => return Err(ArchiveError::BadTag("yao_wu_stage", other)),
            };
            MachineSnapshot::YaoWu { owner, source, stage }
        }
        48 => {
            let owner = dec_pid(r)?;
            let target = dec_opt_pid(r)?;
            let stage = match r.u8()? {
                0 => ShuShenStageSnap::Ask,
                1 => ShuShenStageSnap::Read,
                other => return Err(ArchiveError::BadTag("shu_shen_stage", other)),
            };
            MachineSnapshot::ShuShen { owner, target, stage }
        }
        49 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::ShenZhi { owner }
        }
        50 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => XunXunStageSnap::Undo,
                1 => XunXunStageSnap::Arrange,
                other => return Err(ArchiveError::BadTag("xun_xun_stage", other)),
            };
            let top = r.vec(|r| r.u32())?;
            MachineSnapshot::XunXun { owner, stage, top }
        }
        51 => {
            let owner = dec_pid(r)?;
            let other = dec_pid(r)?;
            MachineSnapshot::WangXi { owner, other }
        }
        52 => {
            let owner = dec_pid(r)?;
            let card = match r.u8()? {
                0 => None,
                1 => Some(r.u32()?),
                other => return Err(ArchiveError::BadTag("ji_jie_card", other)),
            };
            let stage = match r.u8()? {
                0 => JiJieStageSnap::Ask,
                1 => JiJieStageSnap::Read,
                other => return Err(ArchiveError::BadTag("ji_jie_stage", other)),
            };
            MachineSnapshot::JiJie { owner, card, stage }
        }
        53 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::JiYuan { owner }
        }
        54 => {
            let owner = dec_pid(r)?;
            let target = dec_pid(r)?;
            MachineSnapshot::WangZun { owner, target }
        }
        55 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => TongJiStageSnap::Ask,
                1 => TongJiStageSnap::Read,
                other => return Err(ArchiveError::BadTag("tong_ji_stage", other)),
            };
            MachineSnapshot::TongJi { owner, stage }
        }
        56 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::Wuzhong { owner }
        }
        57 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::Taoyuan { owner }
        }
        58 => {
            let owner = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            let pickers = r.vec(dec_pid)?;
            MachineSnapshot::Wugu { owner, cards, pickers }
        }
        60 => {
            let owner = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            let stage = match r.u8()? {
                0 => ZhangbaStageSnap::Cards,
                1 => ZhangbaStageSnap::Target,
                2 => ZhangbaStageSnap::TargetRead,
                t => return Err(ArchiveError::BadTag("zhangba_stage", t)),
            };
            MachineSnapshot::Zhangba { owner, cards, stage }
        }
        61 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossXiangde { owner }
        }
        62 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossFutai { owner }
        }
        63 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossLuanchang { owner }
        }
        65 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossZhaohuo { owner }
        }
        66 => {
            let owner = dec_pid(r)?;
            let stage = r.u8()?;
            let n = r.u32()?;
            let mut targets = Vec::with_capacity(n as usize);
            for _ in 0..n {
                targets.push(r.u32()?);
            }
            MachineSnapshot::BossBuchun { owner, stage, targets }
        }
        67 => {
            let owner = dec_pid(r)?;
            let stage = r.u8()?;
            let actor = r.u32()?;
            MachineSnapshot::BossXingxia { owner, stage, actor }
        }
        68 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossQizuo { owner }
        }
        69 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossXiaoxiong { owner }
        }
        70 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossLingbo { owner }
        }
        71 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossTashui { owner }
        }
        72 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossZjLeiji { owner }
        }
        73 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossHeisheng { owner }
        }
        74 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossLeizhu { owner, emitted }
        }
        75 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossRenao { owner, emitted }
        }
        76 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossWanghun { owner }
        }
        77 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossZhuxin { owner, emitted }
        }
        78 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossPingdeng { owner, emitted }
        }
        79 => {
            let owner = dec_pid(r)?;
            let emitted = r.u8()?;
            MachineSnapshot::BossHuoxing { owner, emitted }
        }
        80 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossBingfeng { owner }
        }
        81 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::BossWangxiang { owner }
        }
        82 => {
            let owner = dec_pid(r)?;
            let module = r.bytes()?.to_vec();
            let paused = match r.u8()? {
                0 => None,
                1 => {
                    let frames = r.bytes()?.to_vec();
                    let pc = r.u32()?;
                    let fuel_remaining = r.u32()?;
                    let id = r.u64()?;
                    let raw = r.bytes()?;
                    if raw.len() != 32 {
                        return Err(ArchiveError::UnexpectedEof);
                    }
                    let mut module_hash = [0u8; 32];
                    module_hash.copy_from_slice(raw);
                    let n = r.u32()? as usize;
                    let mut options = Vec::with_capacity(n.min(1024));
                    for _ in 0..n {
                        options.push(r.u32()?);
                    }
                    let pending = r.bytes()?.to_vec();
                    Some(crate::snap::RvmPaused {
                        frames,
                        pc,
                        fuel_remaining,
                        id,
                        module_hash,
                        options,
                        pending,
                    })
                }
                _ => return Err(ArchiveError::BadTag("Rvm.paused", 255)),
            };
            MachineSnapshot::Rvm {
                owner,
                module,
                paused,
            }
        }
        64 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::Shedu { owner }
        }
        59 => {
            let owner = dec_pid(r)?;
            let holder = dec_pid(r)?;
            let victim = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => JiedaoStageSnap::Ask,
                1 => JiedaoStageSnap::Read,
                other => return Err(ArchiveError::BadTag("jie_dao_stage", other)),
            };
            MachineSnapshot::Jiedao { owner, holder, victim, stage }
        }
        32 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => TuXiStageSnap::Ask,
                1 => TuXiStageSnap::Read,
                t => return Err(ArchiveError::BadTag("tuxi_stage", t)),
            };
            MachineSnapshot::TuXi { owner, stage }
        }
        33 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::LuoYi { owner }
        }
        34 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => GuanXingStageSnap::Arrange,
                1 => GuanXingStageSnap::Place,
                t => return Err(ArchiveError::BadTag("guanxing_stage", t)),
            };
            let n = r.len()?;
            let mut top = Vec::with_capacity(n);
            for _ in 0..n {
                top.push(r.u32()?);
            }
            MachineSnapshot::GuanXing { owner, stage, top }
        }
        35 => {
            let owner = dec_pid(r)?;
            MachineSnapshot::LianYing { owner }
        }
        36 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => RendeStageSnap::SelectCardsAsk,
                1 => RendeStageSnap::TargetAsk,
                2 => RendeStageSnap::TargetRead,
                t => return Err(ArchiveError::BadTag("rende_stage", t)),
            };
            let n = r.len()?;
            let mut cards = Vec::with_capacity(n);
            for _ in 0..n {
                cards.push(r.u32()?);
            }
            MachineSnapshot::Rende { owner, stage, cards }
        }
        37 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => JijiangStageSnap::HelperAsk,
                1 => JijiangStageSnap::TargetAsk,
                t => return Err(ArchiveError::BadTag("jijiang_stage", t)),
            };
            let queue = match r.u8()? {
                0 => None,
                1 => Some(r.vec(dec_pid)?),
                t => return Err(ArchiveError::BadTag("jijiang_queue", t)),
            };
            let helper = dec_opt_pid(r)?;
            let sha = dec_opt_u32(r)?;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("jijiang_inner", t)),
            };
            MachineSnapshot::Jijiang {
                owner,
                stage,
                queue,
                helper,
                sha,
                inner,
            }
        }
        38 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            let asked = r.boolean()?;
            MachineSnapshot::Snatch {
                card,
                source,
                target,
                asked,
            }
        }
        39 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => JieYinStageSnap::CardsAsk,
                1 => JieYinStageSnap::TargetAsk,
                2 => JieYinStageSnap::TargetRead,
                t => return Err(ArchiveError::BadTag("jieyin_stage", t)),
            };
            let n = r.len()?;
            let mut cards = Vec::with_capacity(n);
            for _ in 0..n {
                cards.push(r.u32()?);
            }
            MachineSnapshot::JieYin { owner, stage, cards }
        }
        40 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => FanJianStageSnap::TargetAsk,
                1 => FanJianStageSnap::SuitAsk,
                2 => FanJianStageSnap::PickAsk,
                3 => FanJianStageSnap::Consume,
                t => return Err(ArchiveError::BadTag("fanjian_stage", t)),
            };
            let chosen_suit = match r.u8()? {
                0 => None,
                1 => Some(r.u8()?),
                t => return Err(ArchiveError::BadTag("fanjian_chosen_suit", t)),
            };
            let target = dec_opt_pid(r)?;
            MachineSnapshot::FanJian { owner, stage, target, chosen_suit }
        }
        41 => {
            let owner = dec_pid(r)?;
            let stage = match r.u8()? {
                0 => LiJianStageSnap::DiscardAsk,
                1 => LiJianStageSnap::TargetsAsk,
                2 => LiJianStageSnap::Launch,
                t => return Err(ArchiveError::BadTag("lijian_stage", t)),
            };
            let card = dec_opt_u32(r)?;
            let inner = match r.u8()? {
                0 => None,
                1 => Some(Box::new(dec_machine(r)?)),
                t => return Err(ArchiveError::BadTag("lijian_inner", t)),
            };
            MachineSnapshot::LiJian { owner, stage, card, inner }
        }
        42 => {
            let key_bytes = r.bytes()?;
            if key_bytes.len() > 64 {
                return Err(ArchiveError::BadTag("host_def_key_len", 0));
            }
            let def_key = std::str::from_utf8(key_bytes)
                .map_err(|_| ArchiveError::BadTag("host_def_key_utf8", 0))?
                .to_string();
            let behavior = r.u16()?;
            let stage = r.u8()?;
            let owner = dec_pid(r)?;
            let target = dec_opt_pid(r)?;
            let card = dec_opt_u32(r)?;
            let cards = r.vec(|r| r.u32())?;
            let targets = r.vec(dec_pid)?;
            MachineSnapshot::Host {
                def_key,
                behavior,
                stage,
                owner,
                target,
                card,
                cards,
                targets,
            }
        }
        t => return Err(ArchiveError::BadTag("machine", t)),
    })
}

// ---- 选择 ----

fn enc_choice_option(w: &mut W, o: &ChoiceOption) {
    match o {
        ChoiceOption::Confirm => w.u8(0),
        ChoiceOption::Pass => w.u8(1),
        ChoiceOption::Card(c) => {
            w.u8(2);
            w.u32(*c);
        }
        ChoiceOption::Player(p) => {
            w.u8(3);
            w.u8(*p);
        }
        ChoiceOption::Tag(t) => {
            w.u8(4);
            w.u8(*t);
        }
        ChoiceOption::Kind(k) => {
            w.u8(5);
            w.u32(u32::from(*k));
        }
    }
}

fn dec_choice_option(r: &mut R<'_>) -> Result<ChoiceOption, ArchiveError> {
    Ok(match r.u8()? {
        0 => ChoiceOption::Confirm,
        1 => ChoiceOption::Pass,
        2 => ChoiceOption::Card(r.u32()?),
        3 => ChoiceOption::Player(r.u8()?),
        4 => ChoiceOption::Tag(r.u8()?),
        5 => {
            let code = r.u32()?;
            // 码位必须能反解为目录牌种（未知码=坏档）
            card_kind_from_num(code)?;
            ChoiceOption::Kind(code as u16)
        }
        t => return Err(ArchiveError::BadTag("choice_option", t)),
    })
}

fn enc_open_choice(w: &mut W, o: &OpenChoice) {
    w.u64(o.id);
    enc_pid(w, &o.observer);
    let spec = &o.spec;
    // v2：kind 显式编码（内核机器的种类是权威，不再由提示词静态推导）。
    enc_choice_kind(w, spec.kind);
    w.str(spec.prompt);
    w.len(spec.options.len());
    for o in &spec.options {
        enc_choice_option(w, o);
    }
    w.u64(spec.min as u64);
    w.u64(spec.max as u64);
    w.boolean(spec.allow_cancel);
}


/// N11-b（ADR 0774）：固定提示词注册表——零分配，未知 prompt
/// 直接拒绝。所有引擎生成的 prompt 均为编译期常量。
const KNOWN_PROMPTS: &[&str] = &[
    "discard_to_limit", "respond_shan", "dying_save", "gui_dao_rewrite",
    "wudian_fuchou_cancel", "wudian_diyjuntun_recast",
    "guanxing_arrange", "tuxi_target", "rende_cards", "rende_target",
    "jieyin_cards", "jieyin_target", "fanjian_target", "fanjian_pick",
    "lijian_discard", "lijian_targets", "jijiang_helper", "jijiang_respond",
    "jijiang_target", "liuli_transfer", "hujia_respond", "qingnang_target",
    "qingnang_card", "sancai_target", "sancai_card", "yishang_target",
    "zhiheng_select", "trigger_confirm", "respond_sha",
    "aoe_response", "nullify_window", "duel_sha", "jiji_card", "jiji_target",
    "tian_du_gain", "host_target_discard_equip",
    "wuxing_select_card", "wuxing_select_nature", "d3_jinggou_target",
    "wd_sanjian_discard", "wd_sanjian_target",
    "wd_fudi_target_pick", "wd_fudi_player_pick", "wd_fudi_compare",
    "wd_touliang_own", "wd_touliang_target",
    "yc_gean_t1_pick", "yc_gean_t2_pick", "wd_caochuan_branch",
    "gj_shujinsan_discard", "gj_feibiao_discard", "gj_jinlianzhu_confirm",
    "gj_xiajiao_discard", "gj_luyugeng_cost", "gj_dinvxuanshuang_rider",
    "hh_dunpai_discard", "hh_yuansu_discard", "gw_poxiao_branch",
    // T8 增量 2/7：rule-vm 技能选择挂起（rvm_bridge）
    "rvm_choice",
    "huanren_pick",
    "d3_moyan_target",
    "d3_moyan_gain",
];

fn intern_prompt(bytes: &[u8]) -> Result<&'static str, ArchiveError> {
    let s = std::str::from_utf8(bytes)
        .map_err(|_| ArchiveError::BadTag("prompt_utf8", 0))?;
    KNOWN_PROMPTS
        .iter()
        .find(|k| **k == s)
        .copied()
        .ok_or(ArchiveError::BadTag("prompt_unknown", 0))
}

fn dec_open_choice(r: &mut R<'_>) -> Result<OpenChoice, ArchiveError> {
    let id = r.u64()?;
    let observer = dec_pid(r)?;
    let kind = dec_choice_kind(r)?;
    let prompt_bytes = r.bytes()?;
    if prompt_bytes.len() > 128 {
        return Err(ArchiveError::BadTag("prompt_too_long", 0));
    }
    let prompt: &'static str = intern_prompt(&prompt_bytes)?;
    let options = r.vec(dec_choice_option)?;
    let min = r.u64()? as usize;
    let max = r.u64()? as usize;
    let allow_cancel = r.boolean()?;
    Ok(OpenChoice {
        id,
        observer,
        spec: ChoiceSpec {
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        },
    })
}

/// v2：种类按枚举声明序显式编码，未知标签拒绝。
fn enc_choice_kind(w: &mut W, k: ChoiceKind) {
    w.u8(match k {
        ChoiceKind::Confirm => 0,
        ChoiceKind::SelectCards => 1,
        ChoiceKind::SelectPlayers => 2,
        ChoiceKind::RespondCard => 3,
        ChoiceKind::PickTag => 4,
    });
}

fn dec_choice_kind(r: &mut R<'_>) -> Result<ChoiceKind, ArchiveError> {
    match r.u8()? {
        0 => Ok(ChoiceKind::Confirm),
        1 => Ok(ChoiceKind::SelectCards),
        2 => Ok(ChoiceKind::SelectPlayers),
        3 => Ok(ChoiceKind::RespondCard),
        4 => Ok(ChoiceKind::PickTag),
        t => Err(ArchiveError::BadTag("choice_kind", t)),
    }
}

// ---- 解析与执行位置 ----

fn enc_resolution(w: &mut W, res: &ResolutionSnapshot) {
    w.u32(res.next_event_id);
    w.u64(res.next_choice_id);
    // 栈帧
    w.len(res.stack.len());
    for f in &res.stack {
        w.u32(f.id);
        match f.parent {
            Some(p) => {
                w.u8(1);
                w.u32(p);
            }
            None => w.u8(0),
        }
        enc_event_kind(w, &f.kind);
        enc_event_data(w, &f.data);
        enc_stage(w, &f.stage);
        w.len(f.queue.len());
        for q in &f.queue {
            enc_skill(w, &q.def);
            enc_pid(w, &q.owner);
            w.u8(q.order.0);
            w.i32(q.order.1);
            w.u8(q.order.2);
            w.u8(q.order.3);
            w.u16(q.order.4);
        }
        w.boolean(f.planned);
        w.boolean(f.intrinsic_done);
    }
    // 挂起机器
    w.len(res.parks.len());
    for p in &res.parks {
        w.u64(p.resume_depth as u64);
        match p.slot {
            Slot::Intrinsic { frame } => {
                w.u8(0);
                w.u32(frame);
            }
            Slot::Trigger { frame } => {
                w.u8(1);
                w.u32(frame);
            }
        }
        enc_machine(w, &p.machine);
    }
    // 开放选择与应答
    match &res.open_choice {
        Some(o) => {
            w.u8(1);
            enc_open_choice(w, o);
        }
        None => w.u8(0),
    }
    match &res.answer {
        Some(sel) => {
            w.u8(1);
            w.len(sel.len());
            for o in sel {
                enc_choice_option(w, o);
            }
        }
        None => w.u8(0),
    }
    w.u32(res.budget);
}

fn dec_resolution(r: &mut R<'_>) -> Result<ResolutionSnapshot, ArchiveError> {
    let next_event_id = r.u32()?;
    let next_choice_id = r.u64()?;
    let stack = r.vec(|r| {
        let id = r.u32()?;
        let parent = match r.u8()? {
            0 => None,
            1 => Some(r.u32()?),
            t => return Err(ArchiveError::BadTag("opt_event_id", t)),
        };
        let kind = dec_event_kind(r)?;
        let data = dec_event_data(r)?;
        let stage = dec_stage(r)?;
        let queue = r.vec(|r| {
            let def = r.u16()?;
            let owner = r.u8()?;
            let o0 = r.u8()?;
            let o1 = r.i32()?;
            let o2 = r.u8()?;
            let o3 = r.u8()?;
            let o4 = r.u16()?;
            Ok(QueuedTrigger {
                def,
                owner,
                order: (o0, o1, o2, o3, o4),
            })
        })?;
        let planned = r.boolean()?;
        let intrinsic_done = r.boolean()?;
        Ok(crate::event::Frame {
            id,
            parent,
            kind,
            data,
            stage,
            queue,
            planned,
            intrinsic_done,
        })
    })?;
    let parks = r.vec(|r| {
        let resume_depth = r.u64()? as usize;
        let slot = match r.u8()? {
            0 => Slot::Intrinsic { frame: r.u32()? },
            1 => Slot::Trigger { frame: r.u32()? },
            t => return Err(ArchiveError::BadTag("slot", t)),
        };
        let machine = dec_machine(r)?;
        Ok(ParkSnapshot {
            resume_depth,
            slot,
            machine,
        })
    })?;
    let open_choice = match r.u8()? {
        0 => None,
        1 => Some(dec_open_choice(r)?),
        t => return Err(ArchiveError::BadTag("opt_open_choice", t)),
    };
    let answer = match r.u8()? {
        0 => None,
        1 => Some(r.vec(dec_choice_option)?),
        t => return Err(ArchiveError::BadTag("opt_answer", t)),
    };
    let budget = r.u32()?;
    Ok(ResolutionSnapshot {
        next_event_id,
        next_choice_id,
        stack,
        parks,
        open_choice,
        answer,
        budget,
    })
}

// ---- Boss 关卡流程态（存档 v79 / ADR 0783 B0）----

fn enc_boss_stage(w: &mut W, stage: &BossStageState) {
    w.u8(stage.kind as u8);
    w.u8(stage.level);
    w.boolean(stage.mengpo_spawned);
    w.boolean(stage.mengpo_died);
    w.len(stage.banned_generals.len());
    for g in &stage.banned_generals {
        w.u16(*g);
    }
    w.u8(stage.wing_armed);
    match &stage.pending {
        None => w.u8(0),
        Some(BossStagePending::SelectGeneral { candidates }) => {
            w.u8(1);
            w.len(candidates.len());
            for g in candidates {
                w.u16(*g);
            }
        }
    }
}

fn dec_boss_stage(r: &mut R<'_>) -> Result<BossStageState, ArchiveError> {
    let kind = match r.u8()? {
        0 => BossStageKind::None,
        1 => BossStageKind::ShidianYanluo,
        2 => BossStageKind::SixiangChain,
        3 => BossStageKind::WingGate,
        t => return Err(ArchiveError::BadTag("boss_stage_kind", t)),
    };
    let level = r.u8()?;
    let mengpo_spawned = r.boolean()?;
    let mengpo_died = r.boolean()?;
    let banned_generals = r.vec(|r| r.u16())?;
    let wing_armed = r.u8()?;
    let pending = match r.u8()? {
        0 => None,
        1 => {
            let candidates = r.vec(|r| r.u16())?;
            Some(BossStagePending::SelectGeneral { candidates })
        }
        t => return Err(ArchiveError::BadTag("boss_stage_pending", t)),
    };
    Ok(BossStageState {
        kind,
        level,
        mengpo_spawned,
        mengpo_died,
        banned_generals,
        wing_armed,
        pending,
    })
}

fn enc_vm_aux(w: &mut W, a: &crate::state::VmAux) {
    w.len(a.marks.len());
    for ((p, id), n) in &a.marks {
        w.u8(*p);
        w.u32(*id);
        w.i32(*n);
    }
    w.len(a.piles.len());
    for ((p, name), cards) in &a.piles {
        w.u8(*p);
        w.str(name);
        w.len(cards.len());
        for c in cards {
            w.u32(*c);
        }
    }
    w.len(a.lost.len());
    for (p, cards) in &a.lost {
        w.u8(*p);
        w.len(cards.len());
        for c in cards {
            w.u32(*c);
        }
    }
    w.len(a.once.len());
    for (p, key, scope) in &a.once {
        w.u8(*p);
        w.str(key);
        w.u8(*scope);
    }
    w.len(a.flags.len());
    for ((p, key), n) in &a.flags {
        w.u8(*p);
        w.str(key);
        w.u32(*n);
    }
    w.len(a.hits.len());
    for ((p, key), n) in &a.hits {
        w.u8(*p);
        w.str(key);
        w.u32(*n);
    }
    w.len(a.drawn.len());
    for (p, n) in &a.drawn {
        w.u8(*p);
        w.u32(*n);
    }
}

fn dec_string(r: &mut R<'_>) -> Result<String, ArchiveError> {
    let b = r.bytes()?;
    String::from_utf8(b.to_vec()).map_err(|_| ArchiveError::BadTag("vm_aux_str", 0))
}

fn dec_vm_aux(r: &mut R<'_>) -> Result<crate::state::VmAux, ArchiveError> {
    let mut marks = std::collections::BTreeMap::new();
    for _ in 0..r.len()? {
        let p = r.u8()?;
        let id = r.u32()?;
        let n = r.i32()?;
        marks.insert((p, id), n);
    }
    let mut piles = std::collections::BTreeMap::new();
    for _ in 0..r.len()? {
        let p = r.u8()?;
        let name = dec_string(r)?;
        let n = r.len()?;
        let mut cards = Vec::with_capacity(n.min(256));
        for _ in 0..n {
            cards.push(r.u32()?);
        }
        piles.insert((p, name), cards);
    }
    let mut lost = std::collections::BTreeMap::new();
    for _ in 0..r.len()? {
        let p = r.u8()?;
        let n = r.len()?;
        let mut cards = Vec::with_capacity(n.min(256));
        for _ in 0..n {
            cards.push(r.u32()?);
        }
        lost.insert(p, cards);
    }
    let mut once = std::collections::BTreeSet::new();
    for _ in 0..r.len()? {
        let p = r.u8()?;
        let key = dec_string(r)?;
        let scope = r.u8()?;
        once.insert((p, key, scope));
    }
    let mut flags = std::collections::BTreeMap::new();
    for _ in 0..r.len()? {
        let p = r.u8()?;
        let key = dec_string(r)?;
        let n = r.u32()?;
        flags.insert((p, key), n);
    }
    let mut hits = std::collections::BTreeMap::new();
    for _ in 0..r.len()? {
        let p = r.u8()?;
        let key = dec_string(r)?;
        let n = r.u32()?;
        hits.insert((p, key), n);
    }
    let mut drawn = std::collections::BTreeMap::new();
    for _ in 0..r.len()? {
        let p = r.u8()?;
        let n = r.u32()?;
        drawn.insert(p, n);
    }
    Ok(crate::state::VmAux {
        marks,
        piles,
        lost,
        once,
        flags,
        hits,
        drawn,
    })
}

// ---- 顶层快照 ----

fn enc_snapshot(w: &mut W, s: &StateSnapshot) {
    // core
    w.u64(s.core.revision);
    w.u64(s.core.ruleset_version);
    match &s.core.mode {
        GameMode::Prototype => w.u8(0),
        GameMode::Identity { roles } => {
            w.u8(1);
            w.len(roles.len());
            for r in roles {
                enc_role(w, r);
            }
        }
        GameMode::Kingdoms => w.u8(2),
        GameMode::Doudizhu { lord } => {
            w.u8(4);
            enc_pid(w, lord);
        }
        GameMode::Boss {
            boss,
            chongzheng,
            fellows,
            stage,
        } => {
            w.u8(5);
            enc_pid(w, boss);
            w.u8(*chongzheng);
            w.len(fellows.len());
            for f in fellows {
                enc_pid(w, f);
            }
            // v79：BossStageState
            enc_boss_stage(w, stage);
        }
        GameMode::Kof { pools, wins, games } => {
            w.u8(6);
            for p in pools {
                w.len(p.len());
                for g in p {
                    w.u16(*g);
                }
            }
            w.u32(wins[0]);
            w.u32(wins[1]);
            w.u32(*games);
        }
        GameMode::Versus { teams, substitutes } => {
            w.u8(3);
            w.len(teams.len());
            for t in teams {
                w.len(t.len());
                for p in t {
                    enc_pid(w, p);
                }
            }
            // 替补武将池（v206；与队伍同队序，可为空）
            w.len(substitutes.len());
            for pool in substitutes {
                w.len(pool.len());
                for g in pool {
                    w.u16(*g);
                }
            }
        }
    }
    // 座位环（T19b/v46）：恒等排列亦显式编码（简单且自校验）
    w.len(s.core.seat_ring.len());
    for pid in &s.core.seat_ring {
        w.u8(*pid);
    }
    w.len(s.core.players.len());
    for p in &s.core.players {
        w.boolean(p.alive);
        w.i32(p.hp);
        w.u32(p.max_hp);
        w.len(p.hand.len());
        for c in &p.hand {
            w.u32(*c);
        }
        w.len(p.equips.len());
        for c in &p.equips {
            w.u32(*c);
        }
        w.len(p.judge.len());
        for c in &p.judge {
            w.u32(*c);
        }
        w.boolean(p.dying);
        w.boolean(p.linked);
        // 国战双将槽 + 明置状态（随玩家编码）
        for slot in 0..2 {
            w.opt_u16(p.general_slots[slot]);
        }
        for slot in 0..2 {
            w.boolean(p.revealed_generals[slot]);
        }
        // 五行属性（G1 wuxing）
        w.opt_u8(p.wunature);
        w.opt_u8(p.ctianfu_source);
        match p.ctianfu_card {
            None => w.u8(0),
            Some(v) => {
                w.u8(1);
                w.u32(v);
            }
        }
        w.u32(p.gouhunluo_rounds);
        w.opt_u8(p.gouhunluo_source);
        // 护甲（G1 扩展）
        w.u8(p.hujia);
        // E03 翻面原语（v4）：武将牌朝向
        w.boolean(p.face_down);
        w.u8(p.boss_chongzheng);
        // 筑城标记（v5，U07 刘馥）：跨回合持久标记牌
        w.len(p.zhucheng.len());
        for c in &p.zhucheng {
            w.u32(*c);
        }
        // 机关怨（v43，U05 swd 第二十一批）：待获得牌实体
        w.len(p.jiguanyuan_cards.len());
        for c in &p.jiguanyuan_cards {
            w.u32(*c);
        }
        // 灵剑状态（v44，U05 swd 机关术批）：冷冻/嘲讽/星体改造/隐秘
        w.boolean(p.jileng_active);
        w.boolean(p.chaofeng_active);
        w.u8(p.xingtigaizao_count);
        w.boolean(p.yinmi_active);
        // 卡牌实例改造表（v45，机关术批/ADR 0780）：len+（装备,附着灵剑）
        w.len(p.card_mods.len());
        for (card, attached) in &p.card_mods {
            w.u32(*card);
            w.u32(*attached);
        }
        // 炮台等级（v6，U05 ow 托比昂）：跨回合持久
        w.u8(p.paotai_lv);
        w.u8(p.baolei_mode);
        // 暴雪/急冻（v8，U05 ow 美）：limited 已发动 + 冰封标记
        w.boolean(p.baoxue_used);
        w.boolean(p.guihan_used);
        // 雪煌 limited（swd 芝茵，批 26）
        w.boolean(p.xuehuang_used);
        // 雪霁冰雹标记（swd 卡牌面补切批②）
        w.u8(p.xuejibingbao);
        // 炼妖壶收纳仓（swd 卡牌面补切批③）
        w.len(p.lianyaohu_store.len());
        for c in &p.lianyaohu_store {
            w.u32(*c);
        }
        // 逆攻标记（swd 龟灵之桃，批⑧）
        w.u8(p.nigong_lv);
        w.boolean(p.jidong_active);
        // 强固（v10，U05 ow 奥丽莎）：生效标记
        w.boolean(p.qianggu_active);
        // 屏障（v12，U05 ow 查莉娅）：两标志+上回合次数
        w.boolean(p.pingzhang_self_used);
        w.boolean(p.pingzhang_other_used);
        w.u8(p.pingzhang_count);
        // 潜行（v13，U05 ow 黑影）：生效标记
        w.boolean(p.yinshen_active);
        // 机甲血（v15，U05 ow D.Va）
        w.u8(p.jijia_hp);
        // 风雅颂计数器（v17，U05 xianjian 雷元戈）
        w.u8(p.feng_count);
        w.u8(p.ya_count);
        w.u8(p.song_count);
        // 蛊舞剩余回合数（v21，U05 xianjian 阿奴）
        w.u8(p.anwugu_count);
        // 龙火已觉醒（v23，U05 xianjian 姜城）
        w.boolean(p.jiangcheng_longhuo);
        // 月行对偶（v25，U05 xianjian 龙幽）
        w.opt_u8(p.yuexing_pair);
        // 明绣三字段（v26，U05 xianjian 明绣：回合伤害/灵火/归烬/跳摸）
        w.u8(p.turn_damage);
        w.u32(p.linghuo_ready_round);
        w.u32(p.guijin_ready_round);
        w.boolean(p.skip_next_draw);
        // 贯日已发动（v27，U05 xianjian 云天河）
        w.boolean(p.guanri_used);
        // 离场区+水云计数+忘忧戳（v28，U05 xianjian 紫萱）
        w.len(p.off_court.len());
        for c in &p.off_court {
            w.u32(*c);
        }
        w.u8(p.shuiyun_count);
        w.u32(p.wangyou3_round);
        // 小蛮三字段（v29，U05 xianjian 小蛮：全局用牌/双觉醒）
        w.u32(p.total_used);
        w.boolean(p.lingquan_awakened);
        w.boolean(p.shenwu_awakened);
        // 诚信免伤+冷却（v30，U05 xianjian 明绣）
        w.boolean(p.chengxin_active);
        w.u32(p.chengxin_ready_round);
        // 虎煞标记（v32，U05 xianjian 王小虎）
        w.u8(p.husha_mark);
        w.u8(p.hyunshen_count);
        w.u8(p.daixing_count);
        w.u8(p.juxi_count);
        w.u8(p.pozhou_count);
        w.len(p.sxianjing_traps.len());
        for c in &p.sxianjing_traps {
            w.u32(*c);
        }
        // 死亡拦截+适盈一次性（v33，U05 xianjian 月琦）
        w.boolean(p.death_cancelled);
        w.boolean(p.shiying_used);
        // 花好一次性（v34，U05 xianjian 苏媚）
        w.boolean(p.huahu_used);
        // 玄砚计数（v35，U05 xianjian 慕容紫英）
        w.u32(p.xuanning_mark);
        // 蛇毒标记（v223）
        w.u8(p.shedu);
        // 巨源记录（v226）
        w.i32(p.juyuan_storage);
        // 封印（v228）
        w.boolean(p.fengyin_active);
        // 灵渠计数（v237）
        w.u8(p.lingqu_count);
        // 星囚交替开关（v239）
        w.boolean(p.xingqiu_toggled);
        // 反噬 terra 旗（v269）
        w.boolean(p.zlfanshi_terra);
        // 往事封禁（v270）
        w.boolean(p.wangshi_ban);
        // 复仇标记（玩点论杀批 14）
        w.len(p.fuchou_marked.len());
        for src in &p.fuchou_marked {
            w.u8(*src);
        }
        // 不臣上次发动轮次（v240）
        w.u32(p.buchun_used_round);
        // 星夏上次发动轮次（v242）
        w.u32(p.xingxia_used_round);
        // 形态轮换交替旗标（v249）
        w.boolean(p.nbianshen_toggle);
        // 混乱标记（v60，Boss 郭嘉鬼谋）
        w.boolean(p.boss_mad);
        // 锦鳞一次性发动（v87）
        w.boolean(p.jinlin_used);
        // hlss 星系统（v88）：星数/计数/已受伤类型位图
        w.u8(p.hlss_stars);
        w.u8(p.hlss_count);
        w.u8(p.hlss_damage);
        // 起风已用牌种表（v89）
        w.len(p.hlss_fengqi_used.len());
        for k in &p.hlss_fengqi_used {
            w.u32(u32::from(*k));
        }
        // 潜龙道心值（v90）
        w.u8(p.mbqianlong_marks);
    }
    let _ = &s.core.turn;
    w.len(s.core.deck.len());
    for c in &s.core.deck {
        w.u32(*c);
    }
    w.len(s.core.discard.len());
    for c in &s.core.discard {
        w.u32(*c);
    }
    w.len(s.core.table.len());
    for c in &s.core.table {
        w.u32(*c);
    }
    // rng
    let (rng_state, rng_calls) = s.core.rng.raw_parts();
    w.u64(rng_state);
    w.u64(rng_calls);
    // turn
    let t = &s.core.turn;
    enc_pid(w, &t.current);
    enc_phase(w, &t.phase);
    w.u32(t.round);
    w.boolean(t.sha_used_play_phase);
    w.boolean(t.mufeng_used_basic);
    w.boolean(t.wangchen_discard_basic);
    // v75（swd 卡牌面窗口机批）：神秘果/鼠槌三旗（尾部追加）
    w.boolean(t.shenmiguo_used);
    w.boolean(t.shuchui_armed);
    w.boolean(t.shuchui_hit);
    // v76（swd 武将面批㉔）：连打每回合旗（尾部追加）
    w.boolean(t.lianda_used);
    w.boolean(t.land_used);
    w.boolean(t.land_sha_used);
    w.u32(t.discard_phase_discarded);
    w.len(t.ciqiu_dying.len());
    for p in &t.ciqiu_dying {
        w.u8(*p);
    }
    match t.kangyin2 {
        None => w.u8(0),
        Some((o, x)) => {
            w.u8(1);
            w.u8(o);
            w.u32(x);
        }
    }
    // 还侠跟踪（swd 屈娴，批 27）
    w.len(t.huanxia_track.len());
    for (c, caused) in &t.huanxia_track {
        w.u32(*c);
        w.boolean(*caused);
    }
    w.u32(t.cards_used_play_phase);
    w.u32(t.bte_used_play_phase);
    w.u32(t.phase_number);
    w.u8(t.rende_given);
    w.boolean(t.jiu_used);
    w.boolean(t.jiu_active);
    w.boolean(t.turn_opened);
    w.boolean(t.skip_draw_phase);
    w.boolean(t.skip_play_phase);
    w.boolean(t.guxing_armed);
    w.u8(t.guxing_count);
    // 猛态双旗标（v248）
    w.boolean(t.mengtai_draw_pending);
    w.boolean(t.mengtai_use_pending);
    w.opt_u8(t.zhufangshenshi_target.map(|p| p as u8));
    // 乐动三角（U07 ow 卢西奥，规则 v126）：目标数/摸回量/回复模式
    w.u8(t.yuedong_num);
    w.u8(t.yuedong_eff);
    w.boolean(t.yuedong_recover);
    // 链钩（U05 ow 路霸，规则 v128）：被钩者/伤害增幅已消耗
    w.opt_u8(t.liangou_hook);
    w.boolean(t.liangou_bonus_used);
    // 跳跃（U05 ow 温斯顿，规则 v137）：目标
    w.opt_u8(t.feitiao_target);
    // 返灵（U05 xianjian 瑕，规则 v145）：本回合已发动
    w.boolean(t.fanling_used);
    // 御风（U05 xianjian 李逍遥，规则 v151）：本回合已发动次数
    w.u8(t.xjyufeng_used);
    // 蝶舞（U05 xianjian 龙葵，规则 v153）：本回合已摸牌
    w.boolean(t.diewu_draw_used);
    // 掠影（U05 xianjian 慕长兰，规则 v154）：本回合已发动
    w.boolean(t.lueying_used);
    // 甜香去（U05 xianjian 唐雨柔，规则 v156）：本回合已发动
    w.boolean(t.txianqu_used);
    // 聚宝已回收（U05 xianjian 景天，规则 v158）：本回合已发动
    w.boolean(t.jtjubao_used);
    // 逐月目标（U05 xianjian 云天河，规则 v161）：本出牌阶段白名单
    w.len(t.zhuyue_targets.len());
    for p in &t.zhuyue_targets {
        w.u8(*p);
    }
    // 摄灵本回合次数（U05 xianjian 南宫煌，规则 v168）
    w.u8(t.sheling_used);
    // 射影本回合已发动（U05 xianjian 苏媚，规则 v175）
    w.boolean(t.sheying_used);
    // 张目本回合已发动（U05 xianjian 南宫煌，规则 v181）
    w.boolean(t.zhangmu_used);
    w.boolean(t.yingfeng_chain);
    w.boolean(t.mujiaren_used);
    match t.luanjian_card {
        None => w.u8(0),
        Some(v) => {
            w.u8(1);
            w.u32(v);
        }
    }
    // 冥顽锁定集（v219）
    w.len(t.minwan_targets.len());
    for p in &t.minwan_targets {
        enc_pid(w, p);
    }
    // 炎狱计数（v229）
    w.u8(t.yanyu_count);
    // 奇佐副本载体槽（v60，Boss 郭嘉）
    match t.qizuo_copy {
        None => w.u8(0),
        Some(v) => {
            w.u8(1);
            w.u32(v);
        }
    }
    // 极点链挡旗标（v61，Boss 张角）
    w.boolean(t.jidian_chain);
    // skills（BTreeMap：按键序确定性遍历）
    w.len(s.core.skills.len());
    for ((owner, def), inst) in &s.core.skills {
        enc_pid(w, owner);
        enc_skill(w, def);
        enc_pid(w, &inst.owner);
        w.boolean(inst.enabled);
        w.boolean(inst.once_used);
        w.boolean(inst.expires_end_of_turn);
        w.u8(inst.rounds);
    }
    // outcome
    // view-as 声明牌种表（BTreeMap 确定性遍历）
    w.len(s.core.declared_kinds.len());
    for (card, kind) in &s.core.declared_kinds {
        w.u32(*card);
        w.u8(card_kind_to_num(kind) as u8);
    }
    // 五行属性表（G1 wuxing，BTreeMap 确定性遍历）
    w.len(s.core.card_natures.len());
    for (card, nature) in &s.core.card_natures {
        w.u32(*card);
        w.u8(*nature);
    }
    // 当前地（mtg land，规则 v100）
    match s.core.current_land {
        None => w.u8(0),
        Some(kind) => {
            w.u8(1);
            w.u8(card_kind_to_num(&kind) as u8);
        }
    }
    match &s.core.outcome {
        Some(o) => {
            w.u8(1);
            enc_winner(w, &o.winner);
        }
        None => w.u8(0),
    }
    enc_vm_aux(w, &s.core.vm_aux);
    // 会话数据
    w.u64(s.last_dedup);
    w.boolean(s.segment_open);
    // N11-a：令牌代次随快照延续
    w.u64(s.auth_generation);
    // resolution
    enc_resolution(w, &s.resolution);
    // 段检查点（递归；由 N04 校验保证不嵌套）
    match &s.segment_checkpoint {
        Some(c) => {
            w.u8(1);
            enc_snapshot(w, c);
        }
        None => w.u8(0),
    }
}

fn dec_snapshot(r: &mut R<'_>) -> Result<StateSnapshot, ArchiveError> {
    let revision = r.u64()?;
    let ruleset_version = r.u64()?;
    let mode = match r.u8()? {
        0 => GameMode::Prototype,
        1 => {
            let roles = r.vec(dec_role)?;
            GameMode::Identity { roles }
        }
        2 => GameMode::Kingdoms,
        3 => {
            let teams = r.vec(|r| r.vec(dec_pid))?;
            let substitutes = r.vec(|r| r.vec(|r| r.u16()))?;
            GameMode::Versus { teams, substitutes }
        }
        4 => GameMode::Doudizhu { lord: dec_pid(r)? },
        5 => {
            let boss = dec_pid(r)?;
            let chongzheng = r.u8()?;
            let fellows = r.vec(dec_pid)?;
            let stage = dec_boss_stage(r)?;
            GameMode::Boss {
                boss,
                chongzheng,
                fellows,
                stage,
            }
        }
        6 => {
            let p0 = r.vec(|r| r.u16())?;
            let p1 = r.vec(|r| r.u16())?;
            let w0 = r.u32()?;
            let w1 = r.u32()?;
            let games = r.u32()?;
            GameMode::Kof { pools: [p0, p1], wins: [w0, w1], games }
        }
        t => return Err(ArchiveError::BadTag("mode", t)),
    };
    let ring_len = r.len()?;
    let mut seat_ring = Vec::with_capacity(ring_len);
    for _ in 0..ring_len {
        seat_ring.push(dec_pid(r)?);
    }
    let players = r.vec(|r| {
        let alive = r.boolean()?;
        let hp = r.i32()?;
        let max_hp = r.u32()?;
        let hand = r.vec(|r| r.u32())?;
        let equips = r.vec(|r| r.u32())?;
        let judge = r.vec(|r| r.u32())?;
        let dying = r.boolean()?;
        let linked = r.boolean()?;
        // 双将槽与明置状态（国战 M1 数据模型，随存档往返）
        let mut general_slots = [None, None];
        for slot in 0..2 {
            general_slots[slot] = r.opt_u16()?;
        }
        let mut revealed_generals = [false, false];
        for slot in 0..2 {
            revealed_generals[slot] = r.boolean()?;
        }
        let wunature = r.opt_u8()?;
        let ctianfu_source = r.opt_u8()?;
        let ctianfu_card = match r.u8()? {
            0 => None,
            1 => Some(r.u32()?),
            t => return Err(ArchiveError::BadTag("player_ctianfu_card", t)),
        };
        let gouhunluo_rounds = r.u32()?;
        let gouhunluo_source = r.opt_u8()?;
        let hujia = r.u8()?;
        let face_down = r.boolean()?;
        let boss_chongzheng = r.u8()?;
        let zc_count = r.len()?;
        let mut zhucheng = Vec::new();
        for _ in 0..zc_count {
            zhucheng.push(r.u32()? as crate::ids::CardId);
        }
        let jy_count = r.len()?;
        let mut jiguanyuan_cards = Vec::new();
        for _ in 0..jy_count {
            jiguanyuan_cards.push(r.u32()? as crate::ids::CardId);
        }
        let jileng_active = r.boolean()?;
        let chaofeng_active = r.boolean()?;
        let xingtigaizao_count = r.u8()?;
        let yinmi_active = r.boolean()?;
        let cm_count = r.len()?;
        let mut card_mods = std::collections::BTreeMap::new();
        for _ in 0..cm_count {
            let card = r.u32()?;
            let attached = r.u32()?;
            card_mods.insert(card, attached);
        }
        let paotai_lv = r.u8()?;
        let baolei_mode = r.u8()?;
        let baoxue_used = r.boolean()?;
        let guihan_used = r.boolean()?;
        let xuehuang_used = r.boolean()?;
        let xuejibingbao = r.u8()?;
        let store_len = r.len()?;
        let mut lianyaohu_store = Vec::with_capacity(store_len);
        for _ in 0..store_len {
            lianyaohu_store.push(r.u32()?);
        }
        let nigong_lv = r.u8()?;
        let jidong_active = r.boolean()?;
        let qianggu_active = r.boolean()?;
        let pingzhang_self_used = r.boolean()?;
        let pingzhang_other_used = r.boolean()?;
        let pingzhang_count = r.u8()?;
        let yinshen_active = r.boolean()?;
        let jijia_hp = r.u8()?;
        let feng_count = r.u8()?;
        let ya_count = r.u8()?;
        let song_count = r.u8()?;
        let anwugu_count = r.u8()?;
        let jiangcheng_longhuo = r.boolean()?;
        let yuexing_pair = r.opt_u8()?;
        let turn_damage = r.u8()?;
        let linghuo_ready_round = r.u32()?;
        let guijin_ready_round = r.u32()?;
        let skip_next_draw = r.boolean()?;
        let guanri_used = r.boolean()?;
        let off_court_len = r.len()?;
        let mut off_court = Vec::with_capacity(off_court_len);
        for _ in 0..off_court_len {
            off_court.push(r.u32()?);
        }
        let shuiyun_count = r.u8()?;
        let wangyou3_round = r.u32()?;
        let total_used = r.u32()?;
        let lingquan_awakened = r.boolean()?;
        let shenwu_awakened = r.boolean()?;
        let chengxin_active = r.boolean()?;
        let chengxin_ready_round = r.u32()?;
        let husha_mark = r.u8()?;
        let hyunshen_count = r.u8()?;
        let daixing_count = r.u8()?;
        let juxi_count = r.u8()?;
        let pozhou_count = r.u8()?;
        let sxianjing_n = r.len()?;
        let mut sxianjing_traps = Vec::with_capacity(sxianjing_n);
        for _ in 0..sxianjing_n {
            sxianjing_traps.push(r.u32()?);
        }
        let death_cancelled = r.boolean()?;
        let shiying_used = r.boolean()?;
        let huahu_used = r.boolean()?;
        let xuanning_mark = r.u32()?;
        let shedu = r.u8()?;
        let juyuan_storage = r.i32()?;
        let fengyin_active = r.boolean()?;
        let lingqu_count = r.u8()?;
        let xingqiu_toggled = r.boolean()?;
        let zlfanshi_terra = r.boolean()?;
        let wangshi_ban = r.boolean()?;
        let fuchou_len = r.len()?;
        let mut fuchou_marked = Vec::with_capacity(fuchou_len);
        for _ in 0..fuchou_len {
            fuchou_marked.push(r.u8()?);
        }
        let buchun_used_round = r.u32()?;
        let xingxia_used_round = r.u32()?;
        let nbianshen_toggle = r.boolean()?;
        let boss_mad = r.boolean()?;
        let jinlin_used = r.boolean()?;
        let hlss_stars = r.u8()?;
        let hlss_count = r.u8()?;
        let hlss_damage = r.u8()?;
        let fengqi_len = r.len()?;
        let mut hlss_fengqi_used = Vec::with_capacity(fengqi_len);
        for _ in 0..fengqi_len {
            hlss_fengqi_used.push(r.u32()? as u16);
        }
        let mbqianlong_marks = r.u8()?;
        Ok(crate::state::PlayerState {
            alive,
            hp,
            max_hp,
            hand,
            equips,
            judge,
            dying,
            hujia,
            linked,
            general_slots,
            revealed_generals,
            wunature,
            ctianfu_source,
            ctianfu_card,
            gouhunluo_rounds,
            gouhunluo_source,
            card_mods,
            jiguanyuan_cards,
            jileng_active,
            chaofeng_active,
            xingtigaizao_count,
            yinmi_active,
            face_down,
            boss_chongzheng,
            zhucheng,
            paotai_lv,
            baolei_mode,
            baoxue_used,
            guihan_used,
            xuehuang_used,
            xuejibingbao,
            lianyaohu_store,
            nigong_lv,
            jidong_active,
            qianggu_active,
            pingzhang_self_used,
            pingzhang_other_used,
            pingzhang_count,
            yinshen_active,
            jijia_hp,
            feng_count,
            ya_count,
            song_count,
            anwugu_count,
            jiangcheng_longhuo,
            yuexing_pair,
            turn_damage,
            linghuo_ready_round,
            guijin_ready_round,
            skip_next_draw,
            guanri_used,
            death_cancelled,
            shiying_used,
            huahu_used,
            xuanning_mark,
            shedu,
            juyuan_storage,
            fengyin_active,
            lingqu_count,
            xingqiu_toggled,
            zlfanshi_terra,
            wangshi_ban,
            fuchou_marked,
            buchun_used_round,
            xingxia_used_round,
            nbianshen_toggle,
            boss_mad,
            jinlin_used,
            hlss_stars,
            hlss_count,
            hlss_damage,
            hlss_fengqi_used,
            mbqianlong_marks,
            off_court,
            shuiyun_count,
            wangyou3_round,
            total_used,
            lingquan_awakened,
            shenwu_awakened,
            chengxin_active,
            chengxin_ready_round,
            husha_mark,
            hyunshen_count,
            daixing_count,
            juxi_count,
            pozhou_count,
            sxianjing_traps,
        })
    })?;
    let deck = r.vec(|r| r.u32())?;
    let discard = r.vec(|r| r.u32())?;
    let table = r.vec(|r| r.u32())?;
    let rng_state = r.u64()?;
    let rng_calls = r.u64()?;
    let current = dec_pid(r)?;
    let phase = dec_phase(r)?;
    let round = r.u32()?;
    let sha_used_play_phase = r.boolean()?;
    let mufeng_used_basic = r.boolean()?;
    let wangchen_discard_basic = r.boolean()?;
    // v75（swd 卡牌面窗口机批）：神秘果/鼠槌三旗（尾部追加）
    let shenmiguo_used = r.boolean()?;
    let shuchui_armed = r.boolean()?;
    let shuchui_hit = r.boolean()?;
    let lianda_used = r.boolean()?;
    let land_used = r.boolean()?;
    let land_sha_used = r.boolean()?;
    let discard_phase_discarded = r.u32()?;
    let cq_count = r.len()?;
    let mut ciqiu_dying = Vec::new();
    for _ in 0..cq_count {
        ciqiu_dying.push(r.u8()? as PlayerId);
    }
    let kangyin2 = match r.u8()? {
        0 => None,
        1 => Some((r.u8()? as PlayerId, r.u32()?)),
        _ => return Err(ArchiveError::UnknownCardKind(255)),
    };
    let huanxia_len = r.len()?;
    let mut huanxia_track = Vec::with_capacity(huanxia_len);
    for _ in 0..huanxia_len {
        huanxia_track.push((r.u32()?, r.boolean()?));
    }
    let cards_used_play_phase = r.u32()?;
    let bte_used_play_phase = r.u32()?;
    let phase_number = r.u32()?;
    let rende_given = r.u8()?;
    let jiu_used = r.boolean()?;
    let jiu_active = r.boolean()?;
    let turn_opened = r.boolean()?;
    let skip_draw_phase = r.boolean()?;
    let skip_play_phase = r.boolean()?;
    let guxing_armed = r.boolean()?;
    let guxing_count = r.u8()?;
    let mengtai_draw_pending = r.boolean()?;
    let mengtai_use_pending = r.boolean()?;
    let zhufangshenshi_target = r.opt_u8()?;
    let yuedong_num = r.u8()?;
    let yuedong_eff = r.u8()?;
    let yuedong_recover = r.boolean()?;
    let liangou_hook = r.opt_u8()?;
    let liangou_bonus_used = r.boolean()?;
    let feitiao_target = r.opt_u8()?;
    let fanling_used = r.boolean()?;
    let xjyufeng_used = r.u8()?;
    let diewu_draw_used = r.boolean()?;
    let lueying_used = r.boolean()?;
    let txianqu_used = r.boolean()?;
    let jtjubao_used = r.boolean()?;
    let zhuyue_len = r.len()?;
    let mut zhuyue_targets = Vec::with_capacity(zhuyue_len);
    for _ in 0..zhuyue_len {
        zhuyue_targets.push(r.u8()?);
    }
    let sheling_used = r.u8()?;
    let sheying_used = r.boolean()?;
    let zhangmu_used = r.boolean()?;
    let yingfeng_chain = r.boolean()?;
    let mujiaren_used = r.boolean()?;
    let luanjian_card = match r.u8()? {
        0 => None,
        1 => Some(r.u32()?),
        t => return Err(ArchiveError::BadTag("turn_luanjian_card", t)),
    };
    let minwan_targets = r.vec(dec_pid)?;
    let yanyu_count = r.u8()?;
    let qizuo_copy = match r.u8()? {
        0 => None,
        1 => Some(r.u32()?),
        t => return Err(ArchiveError::BadTag("turn_qizuo_copy", t)),
    };
    let jidian_chain = r.boolean()?;
    let skill_count = r.len()?;
    let mut skills = std::collections::BTreeMap::new();
    for _ in 0..skill_count {
        let key_owner = dec_pid(r)?;
        let key_def = dec_skill(r)?;
        let owner = dec_pid(r)?;
        let enabled = r.boolean()?;
        let once_used = r.boolean()?;
        let expires_end_of_turn = r.boolean()?;
        let rounds = r.u8()?;
        skills.insert(
            (key_owner, key_def),
            SkillInstance {
                owner,
                enabled,
                once_used,
                expires_end_of_turn,
                rounds,
            },
        );
    }
    let dk_count = r.len()?;
    let mut declared_kinds = std::collections::BTreeMap::new();
    for _ in 0..dk_count {
        let card = r.u32()?;
        let kind = card_kind_from_num(r.u8()? as u32)?;
        declared_kinds.insert(card, kind);
    }
    let cn_count = r.len()?;
    let mut card_natures = std::collections::BTreeMap::new();
    for _ in 0..cn_count {
        let card = r.u32()?;
        let nature = r.u8()?;
        card_natures.insert(card, nature);
    }
    // 当前地（mtg land，规则 v100）
    let current_land = match r.u8()? {
        0 => None,
        1 => Some(card_kind_from_num(r.u8()? as u32)?),
        _ => return Err(ArchiveError::UnknownCardKind(255)),
    };
    let outcome = match r.u8()? {
        0 => None,
        1 => Some(crate::state::Outcome {
            winner: dec_winner(r)?,
        }),
        t => return Err(ArchiveError::BadTag("opt_outcome", t)),
    };
    let vm_aux = dec_vm_aux(r)?;
    let last_dedup = r.u64()?;
    let segment_open = r.boolean()?;
    let auth_generation = r.u64()?;
    let resolution = dec_resolution(r)?;
    let segment_checkpoint = match r.u8()? {
        0 => None,
        1 => Some(Box::new(dec_snapshot(r)?)),
        t => return Err(ArchiveError::BadTag("opt_snapshot", t)),
    };
    Ok(StateSnapshot {
        ruleset_version,
        core: crate::state::GameStateCore {
            seat_ring,
            revision,
            ruleset_version,
            mode,
            players,
            deck,
            discard,
            table,
            rng: crate::rng::RuleRng::from_raw_parts(rng_state, rng_calls),
            turn: crate::state::TurnState {
                current,
                phase,
                round,
                sha_used_play_phase,
                mufeng_used_basic,
                wangchen_discard_basic,
                shenmiguo_used,
                shuchui_armed,
                shuchui_hit,
                lianda_used,
                land_used,
                land_sha_used,
                discard_phase_discarded,
                ciqiu_dying,
                kangyin2,
                huanxia_track,
                cards_used_play_phase,
                bte_used_play_phase,
                phase_number,
                rende_given,
                jiu_used,
                jiu_active,
                turn_opened,
                skip_draw_phase,
                skip_play_phase,
                guxing_armed,
                guxing_count,
                mengtai_draw_pending,
                mengtai_use_pending,
                zhufangshenshi_target: zhufangshenshi_target.map(|p| p as PlayerId),
                yuedong_num,
                yuedong_eff,
                yuedong_recover,
                liangou_hook: liangou_hook.map(|p| p as PlayerId),
                liangou_bonus_used,
                feitiao_target: feitiao_target.map(|p| p as PlayerId),
                fanling_used,
                xjyufeng_used,
                diewu_draw_used,
                lueying_used,
                txianqu_used,
                jtjubao_used,
                zhuyue_targets,
                sheling_used,
                sheying_used,
                zhangmu_used,
                yingfeng_chain,
                mujiaren_used,
                luanjian_card,
                minwan_targets,
                yanyu_count,
                qizuo_copy,
                jidian_chain,
            },
            skills,
            declared_kinds,
            card_natures,
            current_land,
            outcome,
            vm_aux,
        },
        last_dedup,
        auth_generation,
        segment_open,
        segment_checkpoint,
        resolution,
    })
}

// ---- CardKind 数值映射（AOE need 字段用 + 选择项牌种码） ----

/// 牌种码（`ChoiceOption::Kind` 载荷，与存档 AOE need 同表）。
pub fn kind_code(kind: CardKind) -> u16 {
    card_kind_to_num(&kind) as u16
}

/// 牌种码反解。
pub fn kind_from_code(code: u16) -> Result<CardKind, ArchiveError> {
    card_kind_from_num(code as u32)
}

fn card_kind_to_num(k: &CardKind) -> u32 {
    use crate::catalog::CardKind::*;
    match k {
        Sha => 0,
        Shan => 1,
        Tao => 2,
        HorsePlus => 3,
        HorseMinus => 4,
        Weapon => 5,
        Armor => 6,
        FireSha => 7,
        Chain => 8,
        SavageAttack => 9,
        ArcheryAttack => 10,
        Duel => 11,
        Nullify => 12,
        Le => 13,
        Bing => 14,
        ShanDian => 15,
        Dismantle => 16,
        Snatch => 17,
        Jiu => 18,
        LeiSha => 19,
        ShouLiJian => 20,
        WuxingPan => 21,
        WangmeiZhike => 22,
        Sanjian => 23,
        Shuigong => 24,
        Fudichouxin => 25,
        Chenhuodajie => 26,
        Suolianjia => 27,
        HongShui => 28,
        Toulianghuanzhu => 29,
        Linghunzhihuo => 30,
        Zhaomingdan => 31,
        Jihuocard => 32,
        Youdishenru => 33,
        Geanguanhuo => 34,
        Caochuanjiejian => 35,
        Shezhanqunru => 36,
        Shujinsan => 37,
        Mianlijinzhen => 38,
        Feibiao => 39,
        Jinlianzhu => 40,
        Tanhuadong => 41,
        Yougeng => 42,
        Qingtuan => 43,
        Yuanbaorou => 44,
        Gudonggeng => 45,
        Mapodoufu => 46,
        Chunbing => 47,
        Liyutang => 48,
        Molicha => 49,
        Xiajiao => 50,
        Mizhilianou => 51,
        Luyugeng => 52,
        Bingpotong => 53,
        Qiankunbiao => 54,
        Longxugou => 55,
        Shenhuofeiya => 56,
        Ziyangdan => 57,
        Yunvyuanshen => 58,
        Shihuifen => 59,
        Liufengsan => 60,
        Shatang => 61,
        Heilonglinpian => 62,
        Liutouge => 63,
        Mutoumianju => 64,
        Dinvxuanshuang => 65,
        Gjyuheng => 66,
        Dunpaigedang => 67,
        Zhiliaobo => 68,
        Yuansuhuimie => 69,
        Xingjiegoutong => 70,
        Chuansongmen => 71,
        Tanshezhiren => 72,
        Shijieshu => 73,
        Yanziyaoshui => 74,
        Aozuzhilei => 75,
        Wenyi => 76,
        Zhihuanjun => 77,
        Baoxueyaoshui => 78,
        Qinpendayu => 79,
        Birinongwu => 80,
        Ciguhanshuang => 81,
        Kunenfayin => 82,
        Shanbengshu => 83,
        Poxiao => 84,
        Zumoshoukao => 85,
        Monkey => 86,
        Xuelunyang => 87,
        Mianju => 88,
        Jiuwei => 89,
        Wuzhong => 90,
        Taoyuan => 91,
        Wugu => 92,
        Jiedao => 93,
        Renwang => 94,
        Qinggang => 95,
        Qilin => 96,
        Hanbing => 97,
        Zhuge => 98,
        Guanshi => 99,
        Qinglong => 100,
        Zhangba => 101,
        Cixiong => 102,
        Fangtian => 103,
        Hufu => 104,
        Liuxinghuoyu => 105,
        Caoyao => 106,
        Chilongya => 107,
        Tianxianjiu => 108,
        Xixueguizhihuan => 109,
        Guiyoujie => 110,
        Shentoumianju => 111,
        Xingjunyan => 112,
        Qinglonglingzhu => 113,
        Zhufangshenshi => 114,
        Fengxueren => 115,
        Yufulu => 116,
        Yangpijuan => 117,
        Daihuofenglun => 118,
        Xiayuncailing => 119,
        Langeguaiyi => 120,
        Longlushanfeng => 121,
        Haidao => 122,
        Shuimomuxue => 123,
        Feixu => 124,
        Yixialan => 125,
        Shamolvzhou => 126,
        Linzhongjianta => 127,
        Bingheyaosai => 128,
        Youlin => 129,
        Lindixiliu => 130,
        Dizaizhen => 131,
        Jiguan => 132,
        Hslingjian => 133,
        Xuejibingbao => 140,
        Shenmiguo => 141,
        Fengyinzhidan => 142,
        Lianyaohu => 143,
        Qiankundai => 144,
        Shennongding => 145,
        Nvwashi => 146,
        Kunlunjingc => 147,
        Donghuangzhong => 148,
        Kongdongyin => 149,
        Jingleishan => 150,
        Chiyuxi => 151,
        Guisheqi => 152,
        Xianluhui => 153,
        Guilingzhitao => 154,
        Sifeizhenmian => 155,
        Shuchui => 161,
        Sadengjinhuan => 162,
        Haotianta => 163,
        Yuruyi => 164,
        Yuchanqian => 165,
        Yuchankun => 166,
        Yuchanzhen => 167,
        Yuchanxun => 168,
        Yuchankan => 169,
        Yuchanli => 170,
        Yuchangen => 171,
        Yuchandui => 172,
        Shencaojie => 173,
        Qinglongzhigui => 156,
        Zhuquezhizhang => 157,
        Baishouzhihu => 158,
        Xuanwuzhihuang => 159,
        Huanglinzhicong => 160,
        Tianfuzhen => 134,
        Qixingzhen => 135,
        Niaoxiangzhen => 136,
        Changshezhen => 137,
        Pozhenjue => 138,
    }
}

fn nature_from_num(n: u8) -> Result<crate::event::DamageNature, ArchiveError> {
    Ok(match n {
        0 => crate::event::DamageNature::Normal,
        1 => crate::event::DamageNature::Fire,
        2 => crate::event::DamageNature::Thunder,
        t => return Err(ArchiveError::BadTag("nature", t)),
    })
}

fn card_kind_from_num(n: u32) -> Result<CardKind, ArchiveError> {
    use crate::catalog::CardKind::*;
    Ok(match n {
        0 => Sha,
        1 => Shan,
        2 => Tao,
        3 => HorsePlus,
        4 => HorseMinus,
        5 => Weapon,
        6 => Armor,
        7 => FireSha,
        8 => Chain,
        9 => SavageAttack,
        10 => ArcheryAttack,
        11 => Duel,
        12 => Nullify,
        13 => Le,
        14 => Bing,
        15 => ShanDian,
        16 => Dismantle,
        17 => Snatch,
        18 => Jiu,
        19 => LeiSha,
        20 => ShouLiJian,
        21 => WuxingPan,
        22 => WangmeiZhike,
        23 => Sanjian,
        24 => Shuigong,
        25 => Fudichouxin,
        26 => Chenhuodajie,
        27 => Suolianjia,
        28 => HongShui,
        29 => Toulianghuanzhu,
        30 => Linghunzhihuo,
        31 => Zhaomingdan,
        32 => Jihuocard,
        33 => Youdishenru,
        34 => Geanguanhuo,
        35 => Caochuanjiejian,
        36 => Shezhanqunru,
        37 => Shujinsan,
        38 => Mianlijinzhen,
        39 => Feibiao,
        40 => Jinlianzhu,
        41 => Tanhuadong,
        42 => Yougeng,
        43 => Qingtuan,
        44 => Yuanbaorou,
        45 => Gudonggeng,
        46 => Mapodoufu,
        47 => Chunbing,
        48 => Liyutang,
        49 => Molicha,
        50 => Xiajiao,
        51 => Mizhilianou,
        52 => Luyugeng,
        53 => Bingpotong,
        54 => Qiankunbiao,
        55 => Longxugou,
        56 => Shenhuofeiya,
        57 => Ziyangdan,
        58 => Yunvyuanshen,
        59 => Shihuifen,
        60 => Liufengsan,
        61 => Shatang,
        62 => Heilonglinpian,
        63 => Liutouge,
        64 => Mutoumianju,
        65 => Dinvxuanshuang,
        66 => Gjyuheng,
        67 => Dunpaigedang,
        68 => Zhiliaobo,
        69 => Yuansuhuimie,
        70 => Xingjiegoutong,
        71 => Chuansongmen,
        72 => Tanshezhiren,
        73 => Shijieshu,
        74 => Yanziyaoshui,
        75 => Aozuzhilei,
        76 => Wenyi,
        77 => Zhihuanjun,
        78 => Baoxueyaoshui,
        79 => Qinpendayu,
        80 => Birinongwu,
        81 => Ciguhanshuang,
        82 => Kunenfayin,
        83 => Shanbengshu,
        84 => Poxiao,
        85 => Zumoshoukao,
        86 => Monkey,
        87 => Xuelunyang,
        88 => Mianju,
        89 => Jiuwei,
        90 => Wuzhong,
        91 => Taoyuan,
        92 => Wugu,
        93 => Jiedao,
        94 => Renwang,
        95 => Qinggang,
        96 => Qilin,
        97 => Hanbing,
        98 => Zhuge,
        99 => Guanshi,
        100 => Qinglong,
        101 => Zhangba,
        102 => Cixiong,
        103 => Fangtian,
        104 => Hufu,
        105 => Liuxinghuoyu,
        106 => Caoyao,
        107 => Chilongya,
        108 => Tianxianjiu,
        109 => Xixueguizhihuan,
        110 => Guiyoujie,
        111 => Shentoumianju,
        112 => Xingjunyan,
        113 => Qinglonglingzhu,
        114 => Zhufangshenshi,
        115 => Fengxueren,
        116 => Yufulu,
        117 => Yangpijuan,
        118 => Daihuofenglun,
        119 => Xiayuncailing,
        120 => Langeguaiyi,
        121 => Longlushanfeng,
        122 => Haidao,
        123 => Shuimomuxue,
        124 => Feixu,
        125 => Yixialan,
        126 => Shamolvzhou,
        127 => Linzhongjianta,
        128 => Bingheyaosai,
        129 => Youlin,
        130 => Lindixiliu,
        131 => Dizaizhen,
        132 => Jiguan,
        133 => Hslingjian,
        140 => Xuejibingbao,
        141 => Shenmiguo,
        142 => Fengyinzhidan,
        143 => Lianyaohu,
        144 => Qiankundai,
        145 => Shennongding,
        146 => Nvwashi,
        147 => Kunlunjingc,
        148 => Donghuangzhong,
        149 => Kongdongyin,
        150 => Jingleishan,
        151 => Chiyuxi,
        152 => Guisheqi,
        153 => Xianluhui,
        154 => Guilingzhitao,
        155 => Sifeizhenmian,
        161 => Shuchui,
        162 => Sadengjinhuan,
        163 => Haotianta,
        164 => Yuruyi,
        165 => Yuchanqian,
        166 => Yuchankun,
        167 => Yuchanzhen,
        168 => Yuchanxun,
        169 => Yuchankan,
        170 => Yuchanli,
        171 => Yuchangen,
        172 => Yuchandui,
        173 => Shencaojie,
        156 => Qinglongzhigui,
        157 => Zhuquezhizhang,
        158 => Baishouzhihu,
        159 => Xuanwuzhihuang,
        160 => Huanglinzhicong,
        134 => Tianfuzhen,
        135 => Qixingzhen,
        136 => Niaoxiangzhen,
        137 => Changshezhen,
        138 => Pozhenjue,
        // P0 解码加固：未知编号返回结构化错误（保留完整 u32），不再 panic。
        _ => return Err(ArchiveError::UnknownCardKind(n)),
    })
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
    let mut r = R { data: payload, pos: 0 };
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
    use crate::content::host::{card_kind_equip_skills, card_kind_host_skills};

    /// CardKind 编号连续且全量编码往返（u8 存档编码域）。
    #[test]
    fn card_kind_encoding_roundtrip_all_variants() {
        let mut count = 0u32;
        while let Ok(kind) = card_kind_from_num(count) {
            assert_eq!(card_kind_to_num(&kind), count, "编号非连续或往返不等 @{count}");
            count += 1;
        }
        assert!(count >= 133, "牌种数应 ≥133（2026-10-11 实测 133），实得 {count}");
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
