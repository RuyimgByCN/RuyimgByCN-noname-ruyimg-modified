use super::*;

/// 使用时无懈窗口：Confirm（v1 曾还原为 RespondCard → OpenChoiceMismatch）。
#[test]
fn archive_choice_kind_nullify_window_roundtrip() {
    assert_choice_byte_roundtrip(build_judge_window, "nullify_window", ChoiceKind::Confirm);
}

/// 遗计给牌目标：SelectPlayers（v1 曾还原为 RespondCard）。
#[test]
fn archive_choice_kind_jiji_target_roundtrip() {
    fn build() -> Engine {
        let mut e = Engine::new(468, 4);
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(1, "遗计");
        let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
        assert!(matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        let c = e.open_choice().expect("遗计触发确认");
        assert_eq!(c.spec.prompt, "trigger_confirm");
        answer_sel(&mut e, vec![ChoiceOption::Confirm]);
        e
    }
    assert_choice_byte_roundtrip(build, "jiji_target", ChoiceKind::SelectPlayers);
}

/// 天妒获得判定牌：Confirm；提示词 tian_du_gain 必须在注册表内（v1 未注册）。
#[test]
fn archive_choice_kind_tian_du_gain_roundtrip() {
    fn build() -> Engine {
        let mut e = Engine::new(213, 4);
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(1, "天妒");
        let le = e.debug_place(0, "le");
        play(&mut e, le, vec![1]);
        loop {
            if e.open_choice().is_some() {
                break;
            }
            match next_phase(&mut e) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("{other:?}"),
            }
        }
        e
    }
    assert_choice_byte_roundtrip(build, "tian_du_gain", ChoiceKind::Confirm);
}

/// 决斗应杀：RespondCard 种类同样逐字节还原。
#[test]
fn archive_choice_kind_duel_sha_roundtrip() {
    assert_choice_byte_roundtrip(build_duel_respond, "duel_sha", ChoiceKind::RespondCard);
}

/// 弃牌上限：SelectCards 种类同样逐字节还原。
#[test]
fn archive_choice_kind_discard_to_limit_roundtrip() {
    assert_choice_byte_roundtrip(
        build_discard_to_limit,
        "discard_to_limit",
        ChoiceKind::SelectCards,
    );
}

/// 注册表守卫：未注册提示词在解码期拒绝（BadTag），版本门禁同时拦截旧 v1 字节。
#[test]
fn archive_rejects_unknown_prompt_and_v1_bytes() {
    let e = build_judge_window();
    let mut snap = e.save();
    let open = snap
        .resolution
        .open_choice
        .as_mut()
        .expect("挂起快照含开放选择");
    open.spec.prompt = "rogue_prompt_from_extension";
    assert_eq!(
        decode_archive(&encode_archive(&snap)),
        Err(ArchiveError::BadTag("prompt_unknown", 0)),
        "未知提示词必须被注册表拒绝"
    );

    // v1 字节（格式版本字段=1）在 v2 解码器下明确拒绝，不静默按旧布局解读
    //（校验和仅覆盖 payload，头部版本字段可直接改写）。
    let mut bytes = encode_archive(&e.save());
    bytes[4] = 1;
    bytes[5] = 0;
    assert_eq!(
        decode_archive(&bytes),
        Err(ArchiveError::UnsupportedFormatVersion(1)),
        "旧 v1 布局不得被 v2 解码器静默解读"
    );
}

/// P0 解码加固（2026-10-06 探针复现缺陷的关闭验证）：
/// ① declared_kinds 中的未知牌种编号返回结构化 `UnknownCardKind`（完整
///    u32），不再 panic；
/// ② 快照读尽后载荷仍有剩余字节（重算长度与校验和的夹带数据）返回
///    `TrailingBytes`，不再被静默接受；
/// ③ `Winner::Faction` 的势力字节走显式 `enc_faction` 表——写入非法
///    势力值被 `dec_faction` 的 BadTag 拒绝，编码/解码同源。
#[test]
fn archive_hardening_rejects_unknown_kind_and_trailing_bytes() {
    use engine_native::catalog::Faction;
    use engine_native::state::Outcome;

    // 重算校验和：篡改/追加 payload 后通过帧完整性检查，进入语义校验。
    fn seal(bytes: &mut [u8]) {
        let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
        let hash = blake3::hash(&bytes[22..22 + len]);
        bytes[22 + len..].copy_from_slice(hash.as_bytes());
    }

    // ① 未知牌种编号（双快照差分定位 declared_kinds 的 kind 字节）。
    let mut snapshot = Engine::new(7, 4).save();
    snapshot.core.declared_kinds.insert(0, CardKind::Sha);
    let mut other = snapshot.clone();
    other.core.declared_kinds.insert(0, CardKind::Tao);
    let mut bytes = encode_archive(&snapshot);
    let other_bytes = encode_archive(&other);
    let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
    let offsets: Vec<_> = (22..22 + len)
        .filter(|&i| bytes[i] != other_bytes[i])
        .collect();
    assert_eq!(offsets.len(), 1, "declared_kinds kind 字节应唯一可定位");
    bytes[offsets[0]] = 255;
    seal(&mut bytes);
    assert_eq!(
        decode_archive(&bytes),
        Err(ArchiveError::UnknownCardKind(255)),
        "未知牌种编号必须返回结构化错误且保留完整编号"
    );

    // ② 载荷尾部夹带 4 字节（重算长度与校验和）→ TrailingBytes。
    let snapshot = Engine::new(7, 4).save();
    let mut bytes = encode_archive(&snapshot);
    let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
    bytes.splice(22 + len..22 + len, [0xde, 0xad, 0xbe, 0xef]);
    bytes[14..22].copy_from_slice(&((len + 4) as u64).to_le_bytes());
    seal(&mut bytes);
    assert_eq!(
        decode_archive(&bytes),
        Err(ArchiveError::TrailingBytes { extra: 4 }),
        "快照读尽后的剩余载荷必须被拒绝"
    );

    // ③ Winner::Faction 往返（enc_faction 显式表路径）+ 非法势力值拒绝。
    let mut snapshot = Engine::new(7, 4).save();
    snapshot.core.outcome = Some(Outcome {
        winner: Winner::Faction(Faction::Wu),
    });
    let data = encode_archive(&snapshot);
    assert_eq!(decode_archive(&data).expect("势力胜负往返"), snapshot);
    let mut bytes = data.clone();
    // outcome 段在 payload 中段：用仅势力不同的双快照差分定位 faction 字节。
    let mut other = snapshot.clone();
    other.core.outcome = Some(Outcome {
        winner: Winner::Faction(Faction::Qun),
    });
    let other_bytes = encode_archive(&other);
    let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
    let offsets: Vec<_> = (22..22 + len)
        .filter(|&i| bytes[i] != other_bytes[i])
        .collect();
    assert_eq!(offsets.len(), 1, "faction 字节应唯一可定位");
    let faction_pos = offsets[0];
    assert_eq!(bytes[faction_pos], 2, "Wu 应编码为 2（显式表）");
    bytes[faction_pos] = 5;
    seal(&mut bytes);
    assert_eq!(
        decode_archive(&bytes),
        Err(ArchiveError::BadTag("faction", 5)),
        "非法势力值必须被显式表拒绝"
    );
}
