use super::*;

/// N11/A5：受 tag 冲突影响的挂起机器字节往返与拒绝负例。
///
/// 历史：XiaoJi（枭姬摸牌机）与 Dismantle 曾共用编码 tag 29，解码的
/// XiaoJi 分支不可达——枭姬挂起无法恢复（编译器 unreachable-pattern）。
/// 2026-09-24 修复为 XiaoJi=43（表尾追加）。语义边界：29 的解码语义保持
/// Dismantle（含 Dismantle 挂起的旧 v2 字节兼容）；XiaoJi 挂起的旧字节
/// 从未可正确解码，新表下未分配 tag 显式拒绝。
#[test]
fn xiaoji_machine_byte_roundtrip_and_rejects() {
    use engine_native::snap::ParkSnapshot;

    fn with_machine(m: MachineSnapshot) -> engine_native::StateSnapshot {
        let mut snap = Engine::new(7, 2).save();
        snap.resolution.parks = vec![ParkSnapshot {
            resume_depth: 0,
            slot: Slot::Intrinsic { frame: 0 },
            machine: m,
        }];
        snap
    }
    fn payload(data: &[u8]) -> &[u8] {
        &data[22..data.len() - 32]
    }
    // 重算校验和：篡改 payload 后通过帧完整性检查，进入解码语义校验。
    fn forge_checksum(mut data: Vec<u8>) -> Vec<u8> {
        let payload_len = data.len() - 22 - 32;
        let checksum = blake3::hash(&data[22..22 + payload_len]);
        data[22 + payload_len..].copy_from_slice(checksum.as_bytes());
        data
    }

    // 往返：XiaoJi（新 tag 43）。
    let snap = with_machine(MachineSnapshot::XiaoJi { owner: 1 });
    let data = encode_archive(&snap);
    assert_eq!(decode_archive(&data).expect("枭姬挂起字节往返"), snap);

    // 往返：Dismantle（tag 29 语义不变——旧 v2 字节兼容）。
    let snap_d = with_machine(MachineSnapshot::Dismantle {
        card: 3,
        source: 0,
        target: 1,
        asked: true,
    });
    let data_d = encode_archive(&snap_d);
    assert_eq!(decode_archive(&data_d).expect("拆牌挂起字节往返"), snap_d);

    // 负例：机器 tag 改为未分配的 255 → BadTag("machine", 255)。
    // 定位：payload 末尾 9 字节 = XiaoJi(tag+pid) + open_choice/answer/
    // budget/checkpoint 各 1 字节（budget u32=4）；失准时下方断言先失败。
    let machine_tag_pos = payload(&data).len() - 9;
    assert_eq!(payload(&data)[machine_tag_pos], 43, "机器 tag 定位失准");
    let mut corrupted = data.clone();
    corrupted[22 + machine_tag_pos] = 255;
    let forged = forge_checksum(corrupted);
    assert!(matches!(
        decode_archive(&forged),
        Err(ArchiveError::BadTag("machine", 255))
    ));

    // 负例：短输入（截去末字节，长度字段与实际不符）→ UnexpectedEof。
    assert!(matches!(
        decode_archive(&data[..data.len() - 1]),
        Err(ArchiveError::UnexpectedEof)
    ));
}
