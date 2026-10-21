use super::*;

/// 自愈（U07 ow 批三，规则 v120）：禅雅塔——每 4 个阶段（phaseNumber
/// %4==0），你可以摸一张牌或回复 1 点体力（可放弃；满血时无回复选项）。
#[test]
fn chanyata_ziyu_every_four_phases() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9301, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "ziyu");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 2); // 受伤：摸/回二选

    // 逐步推进阶段并计数——每当 End 阶段开始且 phaseNumber%4==0 时应挂
    // 起二选（Confirm 门 + 二选）；验证至少触发一次并选择回 1（2→3）
    let mut triggered = false;
    for _ in 0..160 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == engine_native::content::host::WUDIAN_ZIYU_CHOICE {
                triggered = true;
                let hp_before_trigger = e.debug_hp(0);
                // 选 Tag(1)=回 1（若存在）否则摸 1
                let pick = if c.spec.options.contains(&ChoiceOption::Tag(1)) {
                    ChoiceOption::Tag(1)
                } else {
                    ChoiceOption::Tag(0)
                };
                let (cid, obs) = (c.id, c.observer);
                submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![pick],
                    },
                );
                if pick == ChoiceOption::Tag(1) {
                    assert_eq!(e.debug_hp(0), hp_before_trigger + 1, "回 1");
                }
                continue;
            }
            // trigger_confirm 门（OptionalGate）
            let (cid, obs) = (c.id, c.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        if e.turn().round > 12 {
            break;
        }
    }
    assert!(triggered, "12 回合内 %4 门控至少触发一次");
}
