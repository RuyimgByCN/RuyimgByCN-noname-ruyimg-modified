// Evidence-only regression: real existing GaiMing window, no fixture state fabrication.
#[path = "/Users/ruyimg/Documents/Claude/sanguosha/noname-next/crates/engine-native/tests/support/nc09_machine_windows.rs"]
mod windows;

fn main() {
    let mut fixture = windows::fixture(windows::Window::GaiMing);
    let pending = fixture.engine.save();
    let mut before = fixture.engine.view_referee().visible_card_ids();
    before.sort_unstable();
    assert_eq!(before, fixture.physical_ids, "initial pending physical multiset");
    let restored = fixture.restore(&pending).expect("pending state restores");
    assert_eq!(restored.state_hash(), fixture.engine.state_hash());
    windows::answer(&mut fixture.engine, vec![]);
    let mut after = fixture.engine.view_referee().visible_card_ids();
    after.sort_unstable();
    let duplicates: Vec<_> = after.windows(2).filter(|pair| pair[0] == pair[1]).map(|pair| pair[0]).collect();
    println!("cancel: before_count={} after_count={} duplicates={duplicates:?}", before.len(), after.len());
    let snapshot = fixture.engine.save();
    println!("post_cancel_restore={:?}", fixture.restore(&snapshot).map(|_| ()));
    assert_eq!(after, before, "cancel must preserve each physical card exactly once");
    fixture.restore(&snapshot).expect("post-cancel state must restore");
}
