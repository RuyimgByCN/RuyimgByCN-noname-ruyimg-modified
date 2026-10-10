//! Golden coverage for REFRESH「资守·制衡」追加技.

use content_schema::ContentPackage;

const DECADEZISHOU_ZHIHENG: u32 = 3_869_142_148;
const SOURCE: &str =
    include_str!("../../../content/original/ported/refresh/decadezishou_zhiheng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

#[test]
fn source_and_npack_publish_end_phase_recast() {
    let compiled = content_compiler::compile_source_named("decadezishou_zhiheng", SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DECADEZISHOU_ZHIHENG)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert!(trigger.optional);
    assert_eq!(compiled.skill_index[0].skill_def_id, DECADEZISHOU_ZHIHENG);

    let pack = ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DECADEZISHOU_ZHIHENG && trigger.event == "phase_end"
    }));
}
