use super::*;

// ---- 基础类型编码 ----

pub(super) fn enc_opt_u32(w: &mut W, v: Option<u32>) {
    match v {
        Some(x) => {
            w.u8(1);
            w.u32(x);
        }
        None => w.u8(0),
    }
}

pub(super) fn dec_opt_u32(r: &mut R<'_>) -> Result<Option<u32>, ArchiveError> {
    match r.u8()? {
        0 => Ok(None),
        1 => Ok(Some(r.u32()?)),
        t => Err(ArchiveError::BadTag("opt_u32", t)),
    }
}

/// PlayerId = u8。
pub(super) fn enc_pid(w: &mut W, v: &PlayerId) {
    w.u8(*v);
}

pub(super) fn dec_pid(r: &mut R<'_>) -> Result<PlayerId, ArchiveError> {
    Ok(r.u8()?)
}

/// Option<PlayerId>。
pub(super) fn enc_opt_pid(w: &mut W, v: &Option<PlayerId>) {
    match v {
        Some(x) => {
            w.u8(1);
            w.u8(*x);
        }
        None => w.u8(0),
    }
}

pub(super) fn dec_opt_pid(r: &mut R<'_>) -> Result<Option<PlayerId>, ArchiveError> {
    match r.u8()? {
        0 => Ok(None),
        1 => Ok(Some(r.u8()?)),
        t => Err(ArchiveError::BadTag("opt_pid", t)),
    }
}

/// Vec<PlayerId>。
pub(super) fn enc_pids(w: &mut W, v: &[PlayerId]) {
    w.len(v.len());
    for p in v {
        w.u8(*p);
    }
}

pub(super) fn dec_pids(r: &mut R<'_>) -> Result<Vec<PlayerId>, ArchiveError> {
    r.vec(|r| r.u8())
}

/// SkillDefId = u16。
pub(super) fn enc_skill(w: &mut W, v: &SkillDefId) {
    w.u16(*v);
}

pub(super) fn dec_skill(r: &mut R<'_>) -> Result<SkillDefId, ArchiveError> {
    Ok(r.u16()?)
}

pub(super) fn enc_damage_nature(w: &mut W, v: &DamageNature) {
    w.u8(match v {
        DamageNature::Normal => 0,
        DamageNature::Fire => 1,
        DamageNature::Thunder => 2,
    });
}

pub(super) fn dec_damage_nature(r: &mut R<'_>) -> Result<DamageNature, ArchiveError> {
    Ok(match r.u8()? {
        0 => DamageNature::Normal,
        1 => DamageNature::Fire,
        2 => DamageNature::Thunder,
        t => return Err(ArchiveError::BadTag("damage_nature", t)),
    })
}

pub(super) fn enc_phase(w: &mut W, v: &Phase) {
    w.u8(match v {
        Phase::Judge => 0,
        Phase::Draw => 1,
        Phase::Play => 2,
        Phase::Discard => 3,
        Phase::End => 4,
        // R05（规则 333）：准备阶段追加编码位，既有 0–4 不移位。
        Phase::Prepare => 5,
    });
}

pub(super) fn dec_phase(r: &mut R<'_>) -> Result<Phase, ArchiveError> {
    Ok(match r.u8()? {
        0 => Phase::Judge,
        1 => Phase::Draw,
        2 => Phase::Play,
        3 => Phase::Discard,
        4 => Phase::End,
        5 => Phase::Prepare,
        t => return Err(ArchiveError::BadTag("phase", t)),
    })
}

pub(super) fn enc_event_kind(w: &mut W, v: &EventKind) {
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
        CardsLost => 20,
        ShaDodged => 21,
        CardsGiven => 22,
        DamageSource => 23,
        DamageEnd => 24,
        DamageAfter => 25,
        DamageZero => 26,
        DamageCancelled => 27,
        DamageBegin => 28,
        DamageBegin1 => 29,
        DamageBegin2 => 30,
        DamageBegin3 => 31,
        DamageBegin4 => 32,
        SkillBefore => 33,
        SkillBegin => 34,
        SkillInvocation => 35,
        SkillEnd => 36,
        SkillAfter => 37,
        SkillOmitted => 38,
        SkillLogBegin => 39,
        SkillLogAfter => 40,
        OrdinaryJudge => 41,
        // R05（规则 333）：回合开始窗，表尾追加。
        TurnBegin => 42,
        // R05 抽牌事件族：D/R/G 与窗口，表尾追加。
        DrawPhase => 43,
        DrawPhaseBefore => 44,
        DrawPhaseBegin => 45,
        DrawPhaseBegin1 => 46,
        DrawPhaseBegin2 => 47,
        DrawPhaseEnd => 48,
        DrawPhaseAfter => 49,
        Draw => 50,
        DrawGain => 51,
        // R05（K21）：整回合结束窗两段，表尾追加。
        TurnPhaseEnd => 52,
        TurnPhaseAfter => 53,
    };
    w.u8(t);
}

pub(super) fn dec_event_kind(r: &mut R<'_>) -> Result<EventKind, ArchiveError> {
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
        20 => CardsLost,
        21 => ShaDodged,
        22 => CardsGiven,
        23 => DamageSource,
        24 => DamageEnd,
        25 => DamageAfter,
        26 => DamageZero,
        27 => DamageCancelled,
        28 => DamageBegin,
        29 => DamageBegin1,
        30 => DamageBegin2,
        31 => DamageBegin3,
        32 => DamageBegin4,
        33 => SkillBefore,
        34 => SkillBegin,
        35 => SkillInvocation,
        36 => SkillEnd,
        37 => SkillAfter,
        38 => SkillOmitted,
        39 => SkillLogBegin,
        40 => SkillLogAfter,
        41 => OrdinaryJudge,
        42 => TurnBegin,
        43 => DrawPhase,
        44 => DrawPhaseBefore,
        45 => DrawPhaseBegin,
        46 => DrawPhaseBegin1,
        47 => DrawPhaseBegin2,
        48 => DrawPhaseEnd,
        49 => DrawPhaseAfter,
        50 => Draw,
        51 => DrawGain,
        52 => TurnPhaseEnd,
        53 => TurnPhaseAfter,
        t => return Err(ArchiveError::BadTag("event_kind", t)),
    })
}

pub(super) fn enc_stage(w: &mut W, v: &Stage) {
    w.u8(match v {
        Stage::BeforeTriggers => 0,
        Stage::Commit => 1,
        Stage::AfterTriggers => 2,
        Stage::Intrinsic => 3,
        Stage::Done => 4,
    });
}

pub(super) fn dec_stage(r: &mut R<'_>) -> Result<Stage, ArchiveError> {
    Ok(match r.u8()? {
        0 => Stage::BeforeTriggers,
        1 => Stage::Commit,
        2 => Stage::AfterTriggers,
        3 => Stage::Intrinsic,
        4 => Stage::Done,
        t => return Err(ArchiveError::BadTag("stage", t)),
    })
}

pub(super) fn enc_role(w: &mut W, v: &Role) {
    w.u8(match v {
        Role::Lord => 0,
        Role::Loyalist => 1,
        Role::Rebel => 2,
        Role::Traitor => 3,
    });
}

pub(super) fn dec_role(r: &mut R<'_>) -> Result<Role, ArchiveError> {
    Ok(match r.u8()? {
        0 => Role::Lord,
        1 => Role::Loyalist,
        2 => Role::Rebel,
        3 => Role::Traitor,
        t => return Err(ArchiveError::BadTag("role", t)),
    })
}

pub(super) fn enc_faction(w: &mut W, f: &crate::catalog::Faction) {
    w.u8(match f {
        crate::catalog::Faction::Wei => 0,
        crate::catalog::Faction::Shu => 1,
        crate::catalog::Faction::Wu => 2,
        crate::catalog::Faction::Qun => 3,
    });
}

pub(super) fn dec_faction(r: &mut R<'_>) -> Result<crate::catalog::Faction, ArchiveError> {
    Ok(match r.u8()? {
        0 => crate::catalog::Faction::Wei,
        1 => crate::catalog::Faction::Shu,
        2 => crate::catalog::Faction::Wu,
        3 => crate::catalog::Faction::Qun,
        t => return Err(ArchiveError::BadTag("faction", t)),
    })
}

pub(super) fn enc_winner(w: &mut W, v: &Winner) {
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

pub(super) fn dec_winner(r: &mut R<'_>) -> Result<Winner, ArchiveError> {
    Ok(match r.u8()? {
        0 => Winner::Player(r.u8()?),
        1 => Winner::LordSide,
        2 => Winner::Rebels,
        3 => Winner::Traitor,
        4 => Winner::Faction(dec_faction(r)?),
        5 => Winner::Versus(r.u8()? as usize),
        6 => Winner::Doudizhu {
            lord_won: r.boolean()?,
        },
        7 => Winner::Boss {
            boss_won: r.boolean()?,
        },
        t => return Err(ArchiveError::BadTag("winner", t)),
    })
}
