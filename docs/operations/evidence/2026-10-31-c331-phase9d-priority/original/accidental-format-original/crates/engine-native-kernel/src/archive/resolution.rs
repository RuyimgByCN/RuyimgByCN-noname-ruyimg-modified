use super::*;

// ---- 选择 ----

pub(super) fn enc_choice_option(w: &mut W, o: &ChoiceOption) {
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
        0 => ChoiceOption::Confirm,
        1 => ChoiceOption::Pass,
        2 => ChoiceOption::Card(r.u32()?),
        3 => ChoiceOption::Player(r.u8()?),
        4 => ChoiceOption::Tag(r.u8()?),
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

/// N11-b（ADR 0774）：固定提示词注册表——零分配，未知 prompt
/// 直接拒绝。所有引擎生成的 prompt 均为编译期常量。
pub(super) const KNOWN_PROMPTS: &[&str] = &[
    "discard_to_limit",
    "respond_shan",
    "dying_save",
    "gui_dao_rewrite",
    "wudian_fuchou_cancel",
    "wudian_diyjuntun_recast",
    "guanxing_arrange",
    "tuxi_target",
    "rende_cards",
    "rende_target",
    "jieyin_cards",
    "jieyin_target",
    "fanjian_target",
    "fanjian_pick",
    "lijian_discard",
    "lijian_targets",
    "jijiang_helper",
    "jijiang_respond",
    "jijiang_target",
    "liuli_transfer",
    "hujia_respond",
    "qingnang_target",
    "qingnang_card",
    "sancai_target",
    "sancai_card",
    "yishang_target",
    "zhiheng_select",
    "trigger_confirm",
    "respond_sha",
    "aoe_response",
    "nullify_window",
    "duel_sha",
    "jiji_card",
    "jiji_target",
    "tian_du_gain",
    "host_target_discard_equip",
    "wuxing_select_card",
    "wuxing_select_nature",
    "d3_jinggou_target",
    "wd_sanjian_discard",
    "wd_sanjian_target",
    "wd_fudi_target_pick",
    "wd_fudi_player_pick",
    "wd_fudi_compare",
    "wd_touliang_own",
    "wd_touliang_target",
    "yc_gean_t1_pick",
    "yc_gean_t2_pick",
    "wd_caochuan_branch",
    "gj_shujinsan_discard",
    "gj_feibiao_discard",
    "gj_jinlianzhu_confirm",
    "gj_xiajiao_discard",
    "gj_luyugeng_cost",
    "gj_dinvxuanshuang_rider",
    "hh_dunpai_discard",
    "hh_yuansu_discard",
    "gw_poxiao_branch",
    // T8 增量 2/7：rule-vm 技能选择挂起（rvm_bridge）
    "rvm_choice",
    "huanren_pick",
    "d3_moyan_target",
    "d3_moyan_gain",
    // J1（ADR 0786 §A.3，规则 v328）：VM 提交期选择窗挂起的受限 prompt 键
    // （ADR 预期的「rvm_effect_pick 类受限 prompt」；复用既有 ChoiceKind::
    // SelectCards 与 Answer 通道，无新增变体/载荷）。
    // 静态解码白名单；提示按字符串编码，尾追加保留历史登记。
    "rvm_discard_pick",
    "xunxun_arrange",
    "jijie_give_target",
    "gaiming_pick",
    "haotianta_pick",
    "liaoyuan_discard",
    "fanjian_suit",
    "rvm_danshou_option",
    "rvm_danshou_target",
    "rvm_danshou_card",
    "rvm_danshou_discard",
    "rvm_fenghe_card",
    "rvm_fenghe_destination",
    "rvm_pingkou_targets",
    "trigger_order",
    "xuanying_target",
];

pub(super) fn intern_prompt(bytes: &[u8]) -> Result<&'static str, ArchiveError> {
    let s = std::str::from_utf8(bytes).map_err(|_| ArchiveError::BadTag("prompt_utf8", 0))?;
    KNOWN_PROMPTS
        .iter()
        .find(|k| **k == s)
        .copied()
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
    });
}

pub(super) fn dec_choice_kind(r: &mut R<'_>) -> Result<ChoiceKind, ArchiveError> {
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
