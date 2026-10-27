use super::*;

// ---- Boss 关卡流程态（存档 v79 / ADR 0783 B0）----

pub(super) fn enc_boss_stage(w: &mut W, stage: &BossStageState) {
    w.u8(stage.kind as u8);
    w.u8(stage.level);
    w.boolean(stage.mengpo_spawned);
    w.boolean(stage.mengpo_died);
    w.len(stage.banned_generals.len());
    for g in &stage.banned_generals {
        w.u16(*g);
    }
    w.u8(stage.wing_armed);
    // v91：浴火关卡循环型（尾部追加）
    w.u8(stage.loop_type);
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

pub(super) fn dec_boss_stage(r: &mut R<'_>) -> Result<BossStageState, ArchiveError> {
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
    let loop_type = r.u8()?;
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
        loop_type,
        pending,
    })
}

pub(super) fn enc_vm_aux(w: &mut W, a: &crate::state::VmAux) {
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
    w.len(a.storage.len());
    for (key, value) in &a.storage {
        w.str(key);
        w.u32(*value);
    }
}

pub(super) fn dec_string(r: &mut R<'_>) -> Result<String, ArchiveError> {
    let b = r.bytes()?;
    String::from_utf8(b.to_vec()).map_err(|_| ArchiveError::BadTag("vm_aux_str", 0))
}

pub(super) fn dec_vm_aux(r: &mut R<'_>) -> Result<crate::state::VmAux, ArchiveError> {
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
    let mut storage = std::collections::BTreeMap::new();
    for _ in 0..r.len()? {
        let key = dec_string(r)?;
        let value = r.u32()?;
        storage.insert(key, value);
    }
    Ok(crate::state::VmAux {
        marks,
        piles,
        lost,
        once,
        flags,
        hits,
        drawn,
        storage,
    })
}
