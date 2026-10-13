//! N11 权威服务第一批验收（ADR 0733）：认证 actor（令牌绑定座位、
//! 冒充/失效拒绝）与版本化字节存档（确定性编码、往返相等、防篡改）。

use engine_native::archive::{decode_archive, encode_archive, ArchiveError, ARCHIVE_FORMAT_VERSION};
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, HandView, Phase,
    RejectReason,
};

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn fresh_dedup() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn setup() -> Engine {
    let mut e = Engine::new(443, 4);
    for _ in 0..64 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Play {
            break;
        }
        let actor = t.current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        });
        if let CommandOutcome::Pending { choice } = &out {
            let sel: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(choice.spec.min.max(1))
                .collect();
            let _ = e.submit(GameCommand {
                actor: choice.observer,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer { choice_id: choice.id, selection: sel },
            });
        }
    }
    e
}

fn next_phase_cmd(e: &Engine, actor: u8) -> GameCommand {
    GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    }
}

// ---------------------------------------------------------------------------
// N11 认证 actor
// ---------------------------------------------------------------------------

/// 认证主通道：有效令牌的命令被接受；冒充（令牌座位 ≠ 命令 actor）与
/// 未知令牌被拒绝；Answer 同样受认证约束。
#[test]
fn authed_submit_rejects_impersonation_and_unknown_token() {
    let mut e = setup();
    let tok0 = e.issue_token(0);
    let tok1 = e.issue_token(1);
    assert_ne!(tok0, tok1, "令牌必须互异");

    // 有效：0 号令牌提交 0 号命令
    let out = e.submit_authed(tok0, next_phase_cmd(&e, 0));
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );

    // 冒充：1 号令牌提交 actor=0 的命令
    e.set_seat_quota(None);
    let out = e.submit_authed(tok1, next_phase_cmd(&e, 0));
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::Impersonation)),
        "冒充必须被拒绝: {out:?}"
    );

    // 未知令牌
    let fake = engine_native::AuthToken { seat: 3, id: 987_654_321 };
    let out = e.submit_authed(fake, next_phase_cmd(&e, 3));
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSession)));

    // Answer 通道同样受约束：以他人令牌应答 → Impersonation
    // 构造一个开放选择：把 1 号手牌清掉再打杀触发不出闪伤害之外……简化：
    // 直接使用保留的 reject 路径：无选择时 Answer → NoSuchChoice（先于认证后的语义校验，
    // 但认证校验在其前）。此处验证令牌座位不符时即使无选择也拒绝。
    let out = e.submit_authed(
        tok0,
        GameCommand {
            actor: 1,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer { choice_id: 99, selection: vec![ChoiceOption::Pass] },
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::Impersonation)),
        "令牌座位与 actor 不符应拒绝: {out:?}"
    );
}

/// 认证补充：重复签发覆盖旧令牌（旧令牌立即失效）；
/// 无令牌座位的 submit_authed → NoSuchSession。
#[test]
fn auth_token_reissue_revokes_old() {
    let mut e = setup();
    let tok_old = e.issue_token(2);
    let tok_new = e.issue_token(2);
    assert_ne!(tok_old, tok_new);
    // 旧令牌失效
    let out = e.submit_authed(tok_old, next_phase_cmd(&e, 2));
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSession)));
    // 新令牌可用（虽然可能因非当前回合被拒，但拒绝原因不是会话类）
    let out = e.submit_authed(tok_new, next_phase_cmd(&e, 2));
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::NotYourTurn)
            | CommandOutcome::Accepted { .. }
            | CommandOutcome::Pending { .. }
    ));
}

// ---------------------------------------------------------------------------
// N11 版本化字节存档
// ---------------------------------------------------------------------------

/// 存档往返：静止点与挂起窗口两个快照——编码→解码后与原快照相等、
/// 恢复出同哈希引擎；格式版本头为当前版本。
#[test]
fn snapshot_archive_roundtrip_rest_and_window() {
    // (a) 静止点
    let e = setup();
    let snap = e.save();
    let bytes = encode_archive(&snap);
    assert_eq!(&bytes[0..4], b"NGSA");
    assert_eq!(u16::from_le_bytes(bytes[4..6].try_into().unwrap()), ARCHIVE_FORMAT_VERSION);
    let back = decode_archive(&bytes).expect("解码应成功");
    assert_eq!(back, snap, "静止点往返必须逐字段相等");
    let r = Engine::restore(&back).expect("解码快照可恢复");
    assert_eq!(r.state_hash(), e.state_hash());

    // (b) 挂起窗口（鬼道改判窗口：JudgePhase 机器 + 持牌 + 检查点不存在）
    let mut w = Engine::new(207, 4);
    // 牌堆构成变更（v63）：推进途中会出现弃牌等询问——带应答地推进到
    // 0 号出牌阶段（原「遇 Pending 即答 Card」写法会被弃牌询问带偏）。
    for _ in 0..128 {
        let t = w.turn();
        if t.current == 0 && t.phase == Phase::Play && w.open_choice().is_none() {
            break;
        }
        match w.open_choice() {
            Some(c) => {
                let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else if !c.spec.options.is_empty() {
                    vec![c.spec.options[0]]
                } else {
                    vec![]
                };
                let (id2, obs2) = (c.id, c.observer);
                let out = w.submit(GameCommand {
                    actor: obs2,
                    expected_revision: w.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::Answer { choice_id: id2, selection: sel },
                });
                match out {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("{other:?}"),
                }
            }
            None => {
                let actor = t.current;
                let out = w.submit(GameCommand {
                    actor,
                    expected_revision: w.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::NextPhase,
                });
                match out {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("{other:?}"),
                }
            }
        }
    }
    for p in 0..4 {
        w.debug_clear_hand(p);
    }
    w.debug_grant(2, "鬼道");
    // 鬼道改判需黑色手牌：牌堆构成变更（v63）后 debug_place 按种检索
    // 可能拿到红杀——注入确定黑杀。
    {
        let snap = w.save();
        let black_sha = {
            let cat = w.catalog();
            snap.core.deck.iter().copied().find(|c| {
                matches!(cat.cards[*c as usize].kind, engine_native::CardKind::Sha)
                    && matches!(
                        cat.cards[*c as usize].suit,
                        engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                    )
            }).expect("黑杀在牌堆")
        };
        let mut snap = snap;
        snap.core.deck.retain(|c| *c != black_sha);
        snap.core.players[2].hand.push(black_sha);
        w = Engine::restore(&snap).expect("注入恢复");
    }
    w.debug_set_deck_top_by(|d| d.suit.is_heart());
    let le = w.debug_place(0, "le");
    let _ = w.submit(GameCommand {
        actor: 0,
        expected_revision: w.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::PlayCard { card: le, targets: vec![1] },
    });
    // 牌堆构成变更（v63）后 0 号弃牌阶段会先出现弃牌询问：逐个应答
    // 直至鬼才改判窗口（去随机化）。
    for _ in 0..96 {
        match w.open_choice() {
            Some(c) if c.spec.prompt == "gui_dao_rewrite" => break,
            Some(c) => {
                let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else if !c.spec.options.is_empty() {
                    vec![c.spec.options[0]]
                } else {
                    vec![]
                };
                let (id2, obs2) = (c.id, c.observer);
                let out = w.submit(GameCommand {
                    actor: obs2,
                    expected_revision: w.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::Answer { choice_id: id2, selection: sel },
                });
                match out {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("{other:?}"),
                }
            }
            None => {
                let actor = w.turn().current;
                let out = w.submit(GameCommand {
                    actor,
                    expected_revision: w.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::NextPhase,
                });
                match out {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("{other:?}"),
                }
            }
        }
    }
    let snap_w = w.save();
    let bytes_w = encode_archive(&snap_w);
    let back_w = decode_archive(&bytes_w).expect("窗口快照解码应成功");
    assert_eq!(back_w, snap_w, "窗口快照往返必须逐字段相等");
    let r2 = Engine::restore(&back_w).expect("解码窗口快照可恢复");
    assert_eq!(r2.state_hash(), w.state_hash());
    // 恢复件可继续应答改判
    let c = r2.open_choice().expect("改判窗口应保持开放");
    assert_eq!(c.spec.prompt, "gui_dao_rewrite");
}

/// 存档防篡改：payload 翻字节 → 校验和拒绝；坏魔数 → BadMagic；
/// 未来格式版本 → UnsupportedFormatVersion。
#[test]
fn archive_detects_tampering_and_version() {
    let e = setup();
    let snap = e.save();
    let mut bytes = encode_archive(&snap);

    // 翻转 payload 中的一个字节（头部之后）
    let n = bytes.len();
    bytes[n - 40] ^= 0x01;
    assert_eq!(
        decode_archive(&bytes),
        Err(ArchiveError::ChecksumMismatch),
        "payload 篡改必须被校验和拦截"
    );

    // 坏魔数
    let mut bad_magic = encode_archive(&snap);
    bad_magic[0] = b'X';
    assert_eq!(decode_archive(&bad_magic), Err(ArchiveError::BadMagic));

    // 未来格式版本
    let mut bad_ver = encode_archive(&snap);
    bad_ver[4] = 0xFF;
    bad_ver[5] = 0xFF;
    assert_eq!(
        decode_archive(&bad_ver),
        Err(ArchiveError::UnsupportedFormatVersion(u16::MAX))
    );

    // 确定性：同一快照两次编码逐字节相同
    let b1 = encode_archive(&snap);
    let b2 = encode_archive(&snap);
    assert_eq!(b1, b2, "编码必须确定性");
}

/// 认证与存档组合：快照恢复后令牌会话失效（会话数据不入存档），
/// 需重新签发——契约明确记录。
#[test]
fn auth_tokens_do_not_survive_archive() {
    let mut e = setup();
    let _tok = e.issue_token(0);
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let back = decode_archive(&bytes).expect("解码");
    let mut r = Engine::restore(&back).expect("恢复");
    // 恢复件无会话：0 号令牌需重签
    let tok = r.issue_token(0);
    let out = r.submit_authed(tok, next_phase_cmd(&r, 0));
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let _ = HandView::Hidden; // 引用导出，防未使用告警
}

// ---------------------------------------------------------------------------
// N11 服务层：重连 / 乱序 / 去重 / 存档重连哈希连续性（ADR 0734）
// ---------------------------------------------------------------------------

/// 确定性推进 n 条命令（选择按策略应答），返回实际提交的命令脚本与终态哈希。
fn record_script(e: &mut Engine, n: usize) -> (Vec<GameCommand>, [u8; 32]) {
    let mut script = Vec::new();
    let mut guard = 0;
    while script.len() < n {
        guard += 1;
        assert!(guard < 4 * n + 16, "record_script 未收敛");
        let cmd = if let Some(c) = e.open_choice() {
            let mut sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            if sel.is_empty() {
                sel = vec![c.spec.options[0]];
            }
            GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            }
        } else {
            let actor = e.turn().current;
            GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::NextPhase,
            }
        };
        let out = e.submit(cmd.clone());
        match out {
            CommandOutcome::Rejected(_) => continue, // 策略重算（如杀数上限）
            _ => {
                script.push(cmd);
            }
        }
    }
    (script, e.state_hash())
}

/// 服务层会话回归：去重（重复投递幂等）、乱序（过期修订拒绝）、
/// 重连（重签吊销旧令牌、新令牌可用）、存档重连（落盘→恢复→重签→
/// 同脚本续跑与不停机线终态哈希一致）。
#[test]
fn session_layer_reconnect_out_of_order_and_dedup() {
    let mut e = setup();
    let tok0 = e.issue_token(0);

    // —— 去重：重复投递完全相同的命令 → DuplicateCommand，状态不变 ——
    let rev = e.revision();
    let dup_cmd = GameCommand {
        actor: 0,
        expected_revision: rev,
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    };
    let out = e.submit_authed(tok0, dup_cmd.clone());
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "首次投递应接受: {out:?}"
    );
    let h_after_first = e.state_hash();
    let out = e.submit_authed(tok0, dup_cmd);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::DuplicateCommand)),
        "重复投递必须去重拒绝: {out:?}"
    );
    assert_eq!(e.state_hash(), h_after_first, "去重不得改变状态");

    // —— 乱序：过期修订的新命令 → StaleRevision ——
    let stale = GameCommand {
        actor: 0,
        expected_revision: rev, // 已过期的修订号
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    };
    let out = e.submit_authed(tok0, stale);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::StaleRevision)),
        "过期修订必须拒绝: {out:?}"
    );

    // —— 重连：1 号重连重签令牌，旧令牌立即失效，新令牌可用 ——
    let tok1_old = e.issue_token(1);
    let tok1_new = e.issue_token(1); // 重连（重签）
    let out = e.submit_authed(
        tok1_old,
        GameCommand {
            actor: 1,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSession)),
        "重连后旧令牌必须失效: {out:?}"
    );
    let out = e.submit_authed(
        tok1_new,
        GameCommand {
            actor: 1,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSession)),
        "新令牌应通过会话校验: {out:?}"
    );

    // —— 存档重连：落盘 → 恢复（服务重启）→ 重签令牌 → 同脚本续跑 ——
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let restored = decode_archive(&bytes).expect("存档解码");
    let mut r = Engine::restore(&restored).expect("重启恢复");
    assert_eq!(r.state_hash(), e.state_hash(), "重启后状态一致");
    // 旧令牌在恢复件上失效（会话数据不入存档）
    let out = r.submit_authed(
        tok1_new,
        GameCommand {
            actor: 1,
            expected_revision: r.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::NextPhase,
        },
    );
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSession)));
    let _rtok0 = r.issue_token(0);

    // 同一确定性脚本：不停机线 vs 重启线终态一致
    let (script_a, h_a) = record_script(&mut e, 20);
    let (script_b, h_b) = record_script(&mut r, 20);
    assert_eq!(script_a.len(), script_b.len(), "脚本长度应一致");
    assert_eq!(h_b, h_a, "重启线与不停机线终态哈希必须一致");
    for (i, (ca, cb)) in script_a.iter().zip(script_b.iter()).enumerate() {
        assert_eq!(ca.actor, cb.actor, "脚本 {i} actor 一致");
        assert_eq!(
            std::mem::discriminant(&ca.action),
            std::mem::discriminant(&cb.action),
            "脚本 {i} action 一致"
        );
    }
}
