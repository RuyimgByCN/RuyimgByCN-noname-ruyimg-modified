import assert from "node:assert/strict";
import test from "node:test";

import {
  cardCategoryLabel,
  cardCategoryOf,
  cardKindLabel,
  computeDecadeSeatAnchor,
  computeRelativeSeatOrder,
  identityBadgeInfo,
} from "../src/engine/surfaces/decadeTabletopLayout.ts";

function makeSeats(n) {
  return Array.from({ length: n }, (_, i) => ({
    player: i,
    character_def: i + 1,
    seat: i,
    hp: 4,
    max_hp: 4,
    alive: true,
    turned_over: false,
    hand_count: 4,
    identity: i === 0 ? "Lord" : null,
    skills: [],
  }));
}

test("computeRelativeSeatOrder places self at position 0 and cycles counter-clockwise for 2-8 players", () => {
  for (let n = 2; n <= 8; n++) {
    const seats = makeSeats(n);
    for (let selfId = 0; selfId < n; selfId++) {
      const ordered = computeRelativeSeatOrder(seats, selfId);
      assert.equal(ordered.length, n);
      assert.equal(ordered[selfId].position, 0);
      assert.equal(ordered[(selfId + 1) % n].position, 1);
      assert.equal(ordered[(selfId + n - 1) % n].position, n - 1);
      const positions = new Set(ordered.map((slot) => slot.position));
      assert.equal(positions.size, n);
    }
    const spectator = computeRelativeSeatOrder(seats, 99);
    assert.equal(spectator[0].position, 0);
  }
});

test("computeDecadeSeatAnchor assigns non-colliding wrap-around perimeter zones for 2-8 players", () => {
  for (let n = 2; n <= 8; n++) {
    const anchors = Array.from({ length: n }, (_, pos) => computeDecadeSeatAnchor(pos, n));
    assert.equal(anchors[0].zone, "self");
    const topLefts = anchors
      .filter((a) => a.zone === "top")
      .map((a) => Number.parseFloat(a.cssVars["--seat-left"]));
    assert.ok(topLefts.length >= 1, `n=${n} must have at least one top seat`);
    const sortedTop = [...topLefts].sort((a, b) => a - b);
    for (let i = 1; i < sortedTop.length; i++) {
      assert.ok(
        sortedTop[i] - sortedTop[i - 1] >= 17.5,
        `n=${n} top seat spacing too tight for 768px+: ${sortedTop[i - 1]} vs ${sortedTop[i]}`,
      );
    }
    if (n >= 4) {
      assert.equal(anchors[1].zone, "right");
      assert.equal(anchors[n - 1].zone, "left");
    }
  }

  const leftSelf = computeDecadeSeatAnchor(0, 5, { selfOnRight: false });
  const rightSelf = computeDecadeSeatAnchor(0, 5, { selfOnRight: true });
  assert.equal(leftSelf.cssVars["--seat-left"], "16px");
  assert.equal(rightSelf.cssVars["--seat-right"], "16px");
});

test("identityBadgeInfo maps Lord/Loyalist/Rebel/Renegade/unknown to DecadeUI corner glyphs", () => {
  assert.deepEqual(identityBadgeInfo("Lord"), { glyph: "主", label: "主公", tone: "lord" });
  assert.deepEqual(identityBadgeInfo("Loyalist"), { glyph: "忠", label: "忠臣", tone: "loyalist" });
  assert.deepEqual(identityBadgeInfo("Rebel"), { glyph: "反", label: "反贼", tone: "rebel" });
  assert.deepEqual(identityBadgeInfo("Renegade"), { glyph: "内", label: "内奸", tone: "renegade" });
  assert.deepEqual(identityBadgeInfo(null), { glyph: "猜", label: "身份未知", tone: "unknown" });
});

test("cardKindLabel, cardCategoryOf, and cardCategoryLabel map kind and CardKind def (0..=174) accurately", () => {
  assert.equal(cardKindLabel("Sha", 0), "杀");
  assert.equal(cardKindLabel("Shan", 1), "闪");
  assert.equal(cardKindLabel("Tao", 2), "桃");
  assert.equal(cardKindLabel("TrickGuohe", 16), "过河拆桥");

  // Equips & tricks sent as kind="Other" with CardKind u16 def must resolve cleanly
  assert.equal(cardKindLabel("Other", 3), "+1马");
  assert.equal(cardCategoryOf("Other", 3), "equip");
  assert.equal(cardCategoryLabel("Other", 3), "装备");

  assert.equal(cardKindLabel("Other", 14), "兵粮寸断");
  assert.equal(cardCategoryOf("Other", 14), "trick");
  assert.equal(cardCategoryLabel("Other", 14), "锦囊");

  assert.equal(cardKindLabel("Other", 99), "诸葛连弩");
  assert.equal(cardCategoryOf("Other", 99), "equip");
  assert.equal(cardCategoryLabel("Other", 99), "装备");

  // All 175 CardKind discriminants must produce real Chinese names (never #def placeholders)
  for (let def = 0; def <= 174; def++) {
    const label = cardKindLabel("Other", def);
    assert.ok(label && !label.startsWith("#"), `def ${def} failed to resolve: ${label}`);
  }

  // Only truly unknown defs fall back to #def / 功能
  assert.equal(cardKindLabel("Other", 9999), "#9999");
  assert.equal(cardCategoryLabel("Other", 9999), "功能");

  assert.equal(cardCategoryOf("Sha"), "basic");
  assert.equal(cardCategoryOf("TrickWuzhong"), "trick");
  assert.equal(cardCategoryOf("Weapon"), "equip");
  assert.equal(cardCategoryOf("HorseOff"), "equip");
});
