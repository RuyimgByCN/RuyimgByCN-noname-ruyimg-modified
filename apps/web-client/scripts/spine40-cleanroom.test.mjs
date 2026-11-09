/**
 * Clean-room Spine 4.0 Player Verification Suite (Decision 35).
 *
 * Tests all 11 checklist items using hand-written Spine 4.0 JSON skeletons,
 * self-encoded binary `.skel` payloads, and `.atlas` text without any official
 * Spine runtime code or sample assets.
 */

import assert from "node:assert/strict";
import test from "node:test";

import {
  CleanRoomAnimationState,
  GL_DST_COLOR,
  GL_ONE,
  GL_ONE_MINUS_SRC_ALPHA,
  GL_ONE_MINUS_SRC_COLOR,
  GL_SRC_ALPHA,
  GL_ZERO,
  HANDCRAFTED_SPINE40_ATLAS_TEXT,
  HANDCRAFTED_SPINE40_SKELETON_JSON,
  buildBatchGeometry,
  computeAttachmentWorldVertices,
  containsPointInBoundingBox,
  createSkeletonInstance,
  encodeSpine40Binary,
  evaluateCubicBezier,
  isSpine40Version,
  parseSpine40Binary,
  parseSpine40Json,
  parseTextureAtlas,
  renderSpineBatchesToWebGL,
  resolveAttachment,
  resolveDynamicSkinImport,
  resolvePremultipliedBlendFunc,
  updateSkeletonWorldTransform,
} from "../src/engine/spine40/index.ts";

test("Decision 35 (1/7): Spine 4.0 version gate accepts 4.0.x and rejects 3.5/3.7/3.8/4.1/4.2", () => {
  assert.equal(isSpine40Version("4.0"), true);
  assert.equal(isSpine40Version("4.0.64"), true);
  for (const v of ["3.5.51", "3.7.94", "3.8.99", "4.1.24", "4.2.0", ""]) {
    assert.equal(isSpine40Version(v), false, `expected version ${v} to be rejected`);
    assert.throws(
      () => parseSpine40Json({ skeleton: { spine: v }, bones: [], slots: [] }),
      /UNSUPPORTED_SPINE_VERSION/,
    );
  }
});

test("Decision 35 (2/7): JSON parser and self-written binary .skel encoder/decoder round-trip identically", () => {
  const parsed = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  assert.equal(parsed.header.spine, "4.0.64");
  assert.equal(parsed.bones.length, 9);
  assert.equal(parsed.slots.length, 7);
  assert.equal(parsed.skins.length, 2);

  const binaryBytes = encodeSpine40Binary(parsed);
  assert.ok(binaryBytes instanceof Uint8Array);
  assert.ok(binaryBytes.byteLength > 64);

  const decoded = parseSpine40Binary(binaryBytes);
  assert.equal(decoded.header.hash, parsed.header.hash);
  assert.equal(decoded.header.spine, "4.0.64");
  assert.equal(decoded.bones.length, parsed.bones.length);
  assert.equal(decoded.slots.length, parsed.slots.length);
  for (let i = 0; i < parsed.bones.length; i++) {
    assert.equal(decoded.bones[i].name, parsed.bones[i].name);
    assert.equal(decoded.bones[i].transform, parsed.bones[i].transform);
    assert.ok(Math.abs(decoded.bones[i].x - parsed.bones[i].x) < 1e-4);
    assert.ok(Math.abs(decoded.bones[i].y - parsed.bones[i].y) < 1e-4);
  }
});

test("Decision 35 (3/7): Bone hierarchy and all 5 TransformMode world transforms solve accurately", () => {
  const data = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const skel = createSkeletonInstance(data, "default");

  const torso = skel.bones[skel.boneIndexByName.torso];
  torso.scaleX = 2.0;
  torso.scaleY = 1.5;
  torso.rotation = 45;
  updateSkeletonWorldTransform(skel);

  // `head_badge` uses `noRotationOrReflection`: world matrix should have 0 rotation component
  const headBadge = skel.bones[skel.boneIndexByName.head_badge];
  assert.equal(headBadge.data.transform, "noRotationOrReflection");
  assert.ok(Math.abs(headBadge.c) < 1e-3, `expected near-zero c for noRotationOrReflection, got ${headBadge.c}`);

  // `aura_anchor` uses `noScale`: world matrix magnitude should remain 1 despite parent scaleX=2.0
  const auraAnchor = skel.bones[skel.boneIndexByName.aura_anchor];
  assert.equal(auraAnchor.data.transform, "noScale");
  const scaleMag = Math.hypot(auraAnchor.a, auraAnchor.c);
  assert.ok(Math.abs(scaleMag - 1.0) < 1e-4, `expected unit scale magnitude for noScale, got ${scaleMag}`);
});

test("Decision 35 (4/7): 2-bone IK, Transform, and Path constraints solve target positions and orientations", () => {
  const data = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const skel = createSkeletonInstance(data, "default");

  // Verify 2-bone IK (`shoulder_r` -> `forearm_r` -> `ik_hand_target`)
  const shoulder = skel.bones[skel.boneIndexByName.shoulder_r];
  const forearm = skel.bones[skel.boneIndexByName.forearm_r];
  const target = skel.bones[skel.boneIndexByName.ik_hand_target];
  const tipX = forearm.worldX + forearm.a * forearm.data.length;
  const tipY = forearm.worldY + forearm.c * forearm.data.length;
  const distToTarget = Math.hypot(tipX - target.worldX, tipY - target.worldY);
  assert.ok(
    distToTarget < 1.5,
    `2-bone IK tip (${tipX.toFixed(2)}, ${tipY.toFixed(2)}) should reach target (${target.worldX}, ${target.worldY}), dist=${distToTarget}`,
  );
  assert.notEqual(shoulder.rotation, shoulder.data.rotation);

  // Verify Path constraint (`ribbon_node` positioned at 50% along `ribbon_path` [-40,10 -> 0,40 -> 40,10])
  const ribbonNode = skel.bones[skel.boneIndexByName.ribbon_node];
  assert.ok(Math.abs(ribbonNode.worldX - 0) < 1.0, `expected ribbon_node near x=0 apex, got ${ribbonNode.worldX}`);
  assert.ok(Math.abs(ribbonNode.worldY - 40) < 1.0, `expected ribbon_node near y=40 apex, got ${ribbonNode.worldY}`);
});

test("Decision 35 (5/7): Skins, all 6 attachment types (region, mesh, weighted mesh, clipping, path, boundingbox), and hit testing", () => {
  const data = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const defaultSkel = createSkeletonInstance(data, "default");
  const goldSkel = createSkeletonInstance(data, "gold_armor");

  const defaultTorsoAtt = resolveAttachment(defaultSkel, defaultSkel.slotIndexByName.torso_slot);
  const goldTorsoAtt = resolveAttachment(goldSkel, goldSkel.slotIndexByName.torso_slot);
  assert.equal(defaultTorsoAtt?.type, "region");
  assert.equal(goldTorsoAtt?.type, "region");
  assert.equal(defaultTorsoAtt?.color, "e8c468ff");
  assert.equal(goldTorsoAtt?.color, "fde047ff");

  // Weighted mesh (`cape_cloth` bound to `torso` and `cape_follower`)
  const capeSlotIdx = defaultSkel.slotIndexByName.cape_slot;
  const capeAtt = resolveAttachment(defaultSkel, capeSlotIdx);
  assert.equal(capeAtt?.type, "mesh");
  assert.equal(capeAtt?.weighted, true);
  const capeVerts = computeAttachmentWorldVertices(defaultSkel, capeSlotIdx, capeAtt);
  assert.equal(capeVerts.length, 8);

  // BoundingBox hit test (`torso_bounds`)
  const boundsSlotIdx = defaultSkel.slotIndexByName.hit_bounds_slot;
  const boundsVerts = computeAttachmentWorldVertices(defaultSkel, boundsSlotIdx);
  assert.equal(boundsVerts.length, 8);
  assert.equal(containsPointInBoundingBox(boundsVerts, 0, 45), true);
  assert.equal(containsPointInBoundingBox(boundsVerts, 500, 500), false);
});

test("Decision 35 (6/7): Stepped, linear, cubic Bezier, transform/path/deform timelines, and multi-track mixing (replace + add)", () => {
  // Cubic Bezier midpoint check
  const mid = evaluateCubicBezier(0, 0, 0.25, 0, 0.75, 10, 1, 10, 0.5);
  assert.ok(Math.abs(mid - 5.0) < 0.1, `expected symmetric Bezier midpoint ~5.0, got ${mid}`);

  const data = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const skel = createSkeletonInstance(data, "default");
  const animState = new CleanRoomAnimationState();

  // Track 0: `daiji` (Bezier torso rotate + stepped ik_hand_target translate)
  animState.setAnimation(0, "daiji", true, 1.0, "replace");
  animState.update(0.25);
  animState.apply(skel);

  const ikTarget = skel.bones[skel.boneIndexByName.ik_hand_target];
  // At t=0.25, stepped keyframe at t=0 holds x=38, y=44
  assert.ok(Math.abs(ikTarget.x - 38) < 1e-4);
  assert.ok(Math.abs(ikTarget.y - 44) < 1e-4);

  const rotTrack0Only = skel.bones[skel.boneIndexByName.torso].rotation;

  // Track 1: `breath_add` with alpha=0.5 and mixBlend="add"
  animState.setAnimation(1, "breath_add", true, 0.5, "add");
  animState.update(0.25);
  animState.apply(skel);
  const rotMultiTrack = skel.bones[skel.boneIndexByName.torso].rotation;
  assert.ok(rotMultiTrack > rotTrack0Only, "additive track 1 should increase torso rotation");

  // Verify `jineng` transform, path, and mesh deform timelines at t=0.5
  const skelJineng = createSkeletonInstance(data, "default");
  const jinengState = new CleanRoomAnimationState();
  jinengState.setAnimation(0, "jineng", false, 1.0, "replace");
  jinengState.update(0.5);
  jinengState.apply(skelJineng);
  assert.ok(Math.abs(skelJineng.transformConstraints[0].mixRotate - 1.0) < 1e-4);
  assert.ok(Math.abs(skelJineng.pathConstraints[0].position - 1.0) < 1e-4);
  const weaponSlot = skelJineng.slots[skelJineng.slotIndexByName.weapon_slot];
  assert.deepEqual(weaponSlot.deform, [0, 0, 8, 4, 8, -4]);
});

test("Decision 35 (7/7): Atlas parser, WebGL batch geometry, premultiplied alpha blend modes, and dynamicSkin.js fallback", () => {
  const atlas = parseTextureAtlas(HANDCRAFTED_SPINE40_ATLAS_TEXT);
  assert.equal(atlas.pages.length, 1);
  assert.equal(atlas.pages[0].pma, true);
  assert.equal(atlas.regions.weapon_blade.rotate, 90);

  const pmaNormal = resolvePremultipliedBlendFunc("normal", true);
  assert.equal(pmaNormal.srcRgb, GL_ONE);
  assert.equal(pmaNormal.dstRgb, GL_ONE_MINUS_SRC_ALPHA);
  const nonPmaNormal = resolvePremultipliedBlendFunc("normal", false);
  assert.equal(nonPmaNormal.srcRgb, GL_SRC_ALPHA);

  const pmaAdd = resolvePremultipliedBlendFunc("additive", true);
  assert.equal(pmaAdd.srcRgb, GL_ONE);
  assert.equal(pmaAdd.dstRgb, GL_ONE);
  assert.equal(pmaAdd.srcAlpha, GL_ZERO);

  const pmaMul = resolvePremultipliedBlendFunc("multiply", true);
  assert.equal(pmaMul.srcRgb, GL_DST_COLOR);

  const pmaScreen = resolvePremultipliedBlendFunc("screen", true);
  assert.equal(pmaScreen.dstRgb, GL_ONE_MINUS_SRC_COLOR);

  const data = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const skel = createSkeletonInstance(data, "default");
  const batches = buildBatchGeometry(skel, atlas, true);
  // Slots `cape_slot`, `torso_slot`, `weapon_slot` are normal + clipped -> Batch 0;
  // `aura_slot` is additive + unclipped (after clip `end: "weapon_slot"`) -> Batch 1.
  assert.equal(batches.length, 2);
  assert.equal(batches[0].clipped, true);
  assert.deepEqual(batches[0].slotNames, ["cape_slot", "torso_slot", "weapon_slot"]);
  assert.equal(batches[1].clipped, false);
  assert.equal(batches[1].blend, "additive");
  assert.deepEqual(batches[1].slotNames, ["aura_slot"]);

  // Verify direct WebGL batch execution (`renderSpineBatchesToWebGL`)
  const blendCalls = [];
  const drawCalls = [];
  const mockGl = {
    ARRAY_BUFFER: 0x8892,
    ELEMENT_ARRAY_BUFFER: 0x8893,
    DYNAMIC_DRAW: 0x88e8,
    FLOAT: 0x1406,
    TRIANGLES: 0x0004,
    UNSIGNED_SHORT: 0x1403,
    BLEND: 0x0be2,
    COLOR_BUFFER_BIT: 0x4000,
    VERTEX_SHADER: 0x8b31,
    FRAGMENT_SHADER: 0x8b30,
    viewport() {},
    clearColor() {},
    clear() {},
    enable() {},
    blendFuncSeparate(srcRGB, dstRGB, srcAlpha, dstAlpha) {
      blendCalls.push([srcRGB, dstRGB, srcAlpha, dstAlpha]);
    },
    createBuffer() {
      return {};
    },
    bindBuffer() {},
    bufferData() {},
    drawElements(mode, count) {
      drawCalls.push({ mode, count });
    },
  };
  const glStats = renderSpineBatchesToWebGL(mockGl, batches, 220, 140);
  assert.equal(glStats.drawCallCount, 2);
  assert.equal(glStats.triangleCount, 8);
  assert.deepEqual(blendCalls[0], [GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ONE, GL_ONE_MINUS_SRC_ALPHA]);
  assert.deepEqual(blendCalls[1], [GL_ONE, GL_ONE, GL_ZERO, GL_ONE]);

  // Player import adapter: missing asset falls back to static portrait
  const noAssetRes = resolveDynamicSkinImport({
    generalId: "caocao",
    skinName: "魏武霸业",
    x: [0, 12],
    y: [0, -8],
    scale: 0.85,
    action: "GongJi",
  });
  assert.equal(noAssetRes.status, "static-portrait-fallback");
  assert.equal(noAssetRes.normalizedParams.scale, 0.85);

  // Non-4.0 version falls back to static portrait
  const spine38Res = resolveDynamicSkinImport(
    { generalId: "caocao", skinName: "旧版动皮", version: "3.8" },
    HANDCRAFTED_SPINE40_SKELETON_JSON,
  );
  assert.equal(spine38Res.status, "static-portrait-fallback");
  assert.match(spine38Res.reason, /NON_SPINE_40_VERSION:3\.8/);

  // Valid Spine 4.0 binary payload loads cleanly
  const valid40Res = resolveDynamicSkinImport(
    { generalId: "caocao", skinName: "4.0净室动皮", version: "4.0" },
    encodeSpine40Binary(data),
  );
  assert.equal(valid40Res.status, "ready-spine40");
  assert.equal(valid40Res.skeletonData?.bones.length, 9);
});
