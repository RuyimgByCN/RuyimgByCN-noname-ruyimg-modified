use super::*;

/// E06：扩展武将元数据与技能引用在目录中可查询。
#[test]
fn e06_extension_general_metadata() {
    use engine_native::content::extension::example_extension;
    let ext = example_extension();
    // 验证扩展武将的技能引用在基础目录中存在
    let e0 = Engine::new(1, 2);
    for g in &ext.generals {
        for sk in g.skills {
            assert!(
                e0.catalog().skill_by_name(sk).is_some(),
                "扩展武将 {} 的技能 {} 应在目录中",
                g.name,
                sk
            );
        }
        assert!(g.max_hp >= 1 && g.max_hp <= 10);
    }
}
