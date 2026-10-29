from rebuild_common import *
for p in ['crates/engine-native/src/machine.rs','crates/engine-native/src/content/standard/skills/rende.rs','crates/engine-native/src/event.rs','crates/engine-native/tests/nc04_cards_given_event.rs']:
 copy(p)
p='crates/engine-native/src/content/host/step/xianjian_cards.rs'
replace(p,'for c in &sel {\n                            ctx.give_hand_card(self.owner, t, *c);\n                        }','ctx.give_hand_cards(self.owner, t, &sel);')
evidence(2);version(326)
