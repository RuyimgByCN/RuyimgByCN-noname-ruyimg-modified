use super::*;

// ---- 选择 ----

pub(super) fn enc_choice_option(w: &mut W, o: &ChoiceOption) {
    match o {
        ChoiceOption::HiddenCard(slot) => {
            w.u8(8);
            w.u32(*slot);
        }
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
        ChoiceOption::Control(option) => {
            w.u8(7);
            w.u8(option.tag());
        }
        ChoiceOption::Skill(s) => {
            w.u8(6);
            w.u16(*s);
        }
        ChoiceOption::Kind(k) => {
            w.u8(5);
            w.u32(u32::from(*k));
        }
    }
}

pub(super) fn dec_choice_option(r: &mut R<'_>) -> Result<ChoiceOption, ArchiveError> {
    Ok(match r.u8()? {
        8 => ChoiceOption::HiddenCard(r.u32()?),
        0 => ChoiceOption::Confirm,
        1 => ChoiceOption::Pass,
        2 => ChoiceOption::Card(r.u32()?),
        3 => ChoiceOption::Player(r.u8()?),
        4 => ChoiceOption::Tag(r.u8()?),
        7 => ChoiceOption::Control(
            crate::event::YuanceOption::from_tag(r.u8()?)
                .ok_or(ArchiveError::BadTag("yuance_control", 0))?,
        ),
        6 => ChoiceOption::Skill(r.u16()?),
        5 => {
            let code = r.u32()?;
            // 码位必须能反解为目录牌种（未知码=坏档）
            card_kind_from_num(code)?;
            ChoiceOption::Kind(code as u16)
        }
        t => return Err(ArchiveError::BadTag("choice_option", t)),
    })
}

pub(super) fn enc_open_choice(w: &mut W, o: &OpenChoice) {
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

#[path = "known_prompts.rs"]
mod known_prompts;
#[path = "known_prompts_l25_c.rs"]
mod known_prompts_l25_c;
#[path = "known_prompts_l25_d.rs"]
mod known_prompts_l25_d;
#[path = "known_prompts_l25_e.rs"]
mod known_prompts_l25_e;
#[path = "known_prompts_l25_f.rs"]
mod known_prompts_l25_f;
#[path = "known_prompts_l25_g.rs"]
mod known_prompts_l25_g;

pub(super) fn intern_prompt(bytes: &[u8]) -> Result<&'static str, ArchiveError> {
    let s = std::str::from_utf8(bytes).map_err(|_| ArchiveError::BadTag("prompt_utf8", 0))?;
    known_prompts::KNOWN_PROMPTS
        .iter()
        .find(|k| **k == s)
        .copied()
        .or_else(|| {
            known_prompts_l25_c::KNOWN_PROMPTS_L25_C
                .iter()
                .find(|k| **k == s)
                .copied()
        })
        .or_else(|| {
            known_prompts_l25_d::KNOWN_PROMPTS_L25_D
                .iter()
                .find(|k| **k == s)
                .copied()
        })
        .or_else(|| {
            known_prompts_l25_e::KNOWN_PROMPTS_L25_E
                .iter()
                .find(|k| **k == s)
                .copied()
        })
        .or_else(|| {
            known_prompts_l25_f::KNOWN_PROMPTS_L25_F
                .iter()
                .find(|k| **k == s)
                .copied()
        })
        .or_else(|| {
            known_prompts_l25_g::KNOWN_PROMPTS_L25_G
                .iter()
                .find(|k| **k == s)
                .copied()
        })
        .ok_or(ArchiveError::BadTag("prompt_unknown", 0))
}

pub(super) fn dec_open_choice(r: &mut R<'_>) -> Result<OpenChoice, ArchiveError> {
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
pub(super) fn enc_choice_kind(w: &mut W, k: ChoiceKind) {
    w.u8(match k {
        ChoiceKind::Confirm => 0,
        ChoiceKind::SelectCards => 1,
        ChoiceKind::SelectPlayers => 2,
        ChoiceKind::RespondCard => 3,
        ChoiceKind::PickTag => 4,
        ChoiceKind::Control => 5,
    });
}

pub(super) fn dec_choice_kind(r: &mut R<'_>) -> Result<ChoiceKind, ArchiveError> {
    match r.u8()? {
        0 => Ok(ChoiceKind::Confirm),
        1 => Ok(ChoiceKind::SelectCards),
        2 => Ok(ChoiceKind::SelectPlayers),
        3 => Ok(ChoiceKind::RespondCard),
        4 => Ok(ChoiceKind::PickTag),
        5 => Ok(ChoiceKind::Control),
        t => Err(ArchiveError::BadTag("choice_kind", t)),
    }
}

// ---- 解析与执行位置 ----

pub(super) fn enc_resolution(w: &mut W, res: &ResolutionSnapshot) {
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

pub(super) fn dec_resolution(r: &mut R<'_>) -> Result<ResolutionSnapshot, ArchiveError> {
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
