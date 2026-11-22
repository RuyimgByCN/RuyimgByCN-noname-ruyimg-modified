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
  applyIkConstraint,
  applyPathConstraint,
  applyTransformConstraint,
  buildBatchGeometry,
  computeAttachmentWorldVertices,
  containsPointInBoundingBox,
  createSkeletonInstance,
  encodeSpine40Binary,
  evaluateCubicBezier,
  getAnimationDuration,
  isSpine40Version,
  parseSpine40Binary,
  parseSpine40Json,
  parseTextureAtlas,
  renderSpineBatchesToCanvas2D,
  renderSpineBatchesToWebGL,
  resetSkeletonToSetupPose,
  resolveAttachment,
  resolveDynamicSkinImport,
  resolvePremultipliedBlendFunc,
  triangulateSimplePolygon,
  updateSkeletonWorldTransform,
  uploadRgbaTextureToWebGL,
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

test("Decision 35 (8/9): Standard Spine 4.0 binary .skel stream codec round-trips and rejects non-4.0 binary headers", async () => {
  const { encodeStandardSpine40Skel, clipTriangleAgainstPolygon } = await import("../src/engine/spine40/index.ts");
  const data = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const stdSkelBytes = encodeStandardSpine40Skel(data);
  // Standard .skel does NOT start with SP40 magic
  assert.notEqual(String.fromCharCode(...stdSkelBytes.slice(0, 4)), "SP40");

  const decoded = parseSpine40Binary(stdSkelBytes);
  assert.equal(decoded.header.spine, "4.0.64");
  assert.equal(decoded.bones.length, data.bones.length);
  assert.equal(decoded.slots.length, data.slots.length);
  assert.equal(decoded.skins.length, data.skins.length);
  assert.ok(decoded.animations.daiji);
  assert.ok(decoded.animations.jineng);

  // Forge a non-4.0 ("3.8.99") standard binary header (8-byte hash + len+1=7 + "3.8.99")
  const non40Bytes = new Uint8Array([
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    7, 0x33, 0x2e, 0x38, 0x2e, 0x39, 0x39,
    0, 0, 0, 0,
  ]);
  const fallbackRes = resolveDynamicSkinImport({ generalId: "lvbu", skinName: "3.8旧动皮" }, non40Bytes);
  assert.equal(fallbackRes.status, "static-portrait-fallback");
  assert.match(fallbackRes.reason, /UNSUPPORTED_SPINE_VERSION:3\.8\.99/);

  // Verify Sutherland-Hodgman polygon clipping interpolates UVs along diagonal clip edge `x + y = 10`
  const clipTri = [
    [0, 0],
    [10, 0],
    [0, 10],
  ];
  const clippedPoly = clipTriangleAgainstPolygon(
    [0, 0, 0, 0, 1, 1, 1, 1],
    [20, 0, 1, 0, 1, 1, 1, 1],
    [0, 20, 0, 1, 1, 1, 1, 1],
    clipTri,
  );
  assert.equal(clippedPoly.length, 3);
  // Vertex [20, 0, u=1] clipped at [10, 0] must interpolate u=0.5!
  const rightIntercept = clippedPoly.find((v) => Math.abs(v[0] - 10) < 1e-4 && Math.abs(v[1]) < 1e-4);
  assert.ok(rightIntercept, "expected clipped vertex at (10, 0)");
  assert.ok(Math.abs(rightIntercept[2] - 0.5) < 1e-4, `expected interpolated u=0.5, got ${rightIntercept[2]}`);

  // Triangle completely outside clip polygon produces 0 vertices (discarded, not clamped to edge)
  const outsideClipped = clipTriangleAgainstPolygon(
    [30, 30, 0, 0, 1, 1, 1, 1],
    [40, 30, 1, 0, 1, 1, 1, 1],
    [30, 40, 0, 1, 1, 1, 1, 1],
    clipTri,
  );
  assert.equal(outsideClipped.length, 0);
});

test("Decision 35 (9/9): Full WebGL shader program, attribute pointers, and RGBA texture pipeline execute cleanly", () => {
  const data = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const atlas = parseTextureAtlas(HANDCRAFTED_SPINE40_ATLAS_TEXT);
  const skel = createSkeletonInstance(data, "default");
  const batches = buildBatchGeometry(skel, atlas, true);

  const attribPointers = [];
  let compiledShaders = 0;
  let linkedPrograms = 0;
  const fullMockGl = {
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
    COMPILE_STATUS: 0x8b81,
    LINK_STATUS: 0x8b82,
    TEXTURE_2D: 0x0de1,
    TEXTURE0: 0x84c0,
    RGBA: 0x1908,
    UNSIGNED_BYTE: 0x1401,
    TEXTURE_MIN_FILTER: 0x2801,
    TEXTURE_MAG_FILTER: 0x2800,
    TEXTURE_WRAP_S: 0x2802,
    TEXTURE_WRAP_T: 0x2803,
    LINEAR: 0x2601,
    CLAMP_TO_EDGE: 0x812f,
    viewport() {},
    clearColor() {},
    clear() {},
    enable() {},
    blendFuncSeparate() {},
    createBuffer: () => ({}),
    bindBuffer() {},
    bufferData() {},
    drawElements() {},
    createShader: (type) => ({ type }),
    shaderSource() {},
    compileShader() {
      compiledShaders += 1;
    },
    getShaderParameter: () => true,
    createProgram: () => ({}),
    attachShader() {},
    linkProgram() {
      linkedPrograms += 1;
    },
    getProgramParameter: () => true,
    useProgram() {},
    getAttribLocation: (_p, name) => (name === "a_position" ? 0 : name === "a_texCoord" ? 1 : 2),
    getUniformLocation: (_p, name) => ({ name }),
    enableVertexAttribArray() {},
    vertexAttribPointer(index, size, _type, _norm, stride, offset) {
      attribPointers.push({ index, size, stride, offset });
    },
    uniform2f() {},
    uniform1i() {},
    createTexture: () => ({}),
    activeTexture() {},
    bindTexture() {},
    texImage2D() {},
    texParameteri() {},
  };

  const res1 = renderSpineBatchesToWebGL(fullMockGl, batches, 220, 140);
  assert.equal(res1.shaderProgramActive, true);
  assert.equal(res1.drawCallCount, 2);
  assert.equal(compiledShaders, 2);
  assert.equal(linkedPrograms, 1);
  assert.equal(attribPointers.length, 6); // 3 attributes * 2 batches

  // Second frame reuses cached GLSL program without recompiling
  const res2 = renderSpineBatchesToWebGL(fullMockGl, batches, 220, 140);
  assert.equal(res2.shaderProgramActive, true);
  assert.equal(compiledShaders, 2);
  assert.equal(linkedPrograms, 1);
});

test("Decision 35 (10/11): Binary .skel zig-zag signed varints (negative drawOrder), header.hash fidelity, nonessential=false, and LinkedMesh (type 3)", async () => {
  const {
    encodeSpine40StandardBinary,
    decodeSpine40StandardBinary,
  } = await import("../src/engine/spine40/index.ts");
  assert.equal(typeof encodeSpine40StandardBinary, "function");
  assert.equal(typeof decodeSpine40StandardBinary, "function");

  // 1) encodeSpine40Binary must emit real Spine 4.0 binary bytes (never a "SP40" JSON wrapper)
  const baseData = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const binFromPrimary = encodeSpine40Binary(baseData);
  assert.notEqual(String.fromCharCode(...binFromPrimary.slice(0, 4)), "SP40");

  // 2) Negative drawOrder offsets (-1, -2) must round-trip via zig-zag optimizePositive=false varints
  const negDrawJson = structuredClone(HANDCRAFTED_SPINE40_SKELETON_JSON);
  negDrawJson.animations.daiji.drawOrder = [
    {
      time: 0.2,
      offsets: [
        { slot: "weapon_slot", offset: -2 },
        { slot: "cape_slot", offset: 1 },
      ],
    },
  ];
  const negDrawData = parseSpine40Json(negDrawJson);
  const negDrawSkel = encodeSpine40StandardBinary(negDrawData, { nonessential: false });
  const decodedNegDraw = decodeSpine40StandardBinary(negDrawSkel);
  assert.deepEqual(decodedNegDraw.animations.daiji.drawOrder, [
    {
      time: 0.2,
      offsets: [
        { slot: "weapon_slot", offset: -2 },
        { slot: "cape_slot", offset: 1 },
      ],
    },
  ]);

  // 3) Third-party .skel header.hash must decode from the 8-byte header, NOT strings[0]
  assert.notEqual(decodedNegDraw.header.hash, decodedNegDraw.skins[0]?.attachments?.clip_window?.viewport_clip?.name);
  assert.match(decodedNegDraw.header.hash, /^[0-9a-f]{16}$/);

  // 4) JSON & Binary LinkedMesh (attachment type === 3) inherits parent mesh topology
  const linkedJson = structuredClone(HANDCRAFTED_SPINE40_SKELETON_JSON);
  linkedJson.skins[1].attachments.cape_slot = {
    cape_cloth: {
      type: "linkedmesh",
      skin: "default",
      parent: "cape_cloth",
      deform: true,
      color: "93c5fdff",
      width: 64,
      height: 96,
    },
  };
  const parsedLinked = parseSpine40Json(linkedJson);
  const goldCape = parsedLinked.skins[1].attachments.cape_slot.cape_cloth;
  assert.equal(goldCape.type, "mesh");
  assert.equal(goldCape.triangles.length, 6);
  assert.equal(goldCape.weighted, true);

  // Hand-craft minimal Spine 4.0 binary stream with default mesh + named skin LinkedMesh (type = 3)
  const linkedBytes = encodeSpine40StandardBinary(parsedLinked, { nonessential: false, emitLinkedMeshForIdenticalTopology: true });
  const decodedLinked = decodeSpine40StandardBinary(linkedBytes);
  const decodedGoldCape = decodedLinked.skins[1].attachments.cape_slot.cape_cloth;
  assert.equal(decodedGoldCape.type, "mesh");
  assert.equal(decodedGoldCape.triangles.length, 6);
  assert.equal(decodedGoldCape.uvs.length, 8);
  assert.equal(decodedGoldCape.color, "93c5fdff");
});

test("Decision 35 (11/11): Concave ClippingAttachment polygon triangulates via ear-clipping without cutting off valid wings", async () => {
  const { clipBatchTrianglesToPolygon } = await import("../src/engine/spine40/index.ts");
  assert.equal(typeof clipBatchTrianglesToPolygon, "function");

  // Concave V-notched polygon: [0,0] -> [10,0] -> [10,10] -> [5,4] (reflex notch) -> [0,10]
  // Area = 10*10 - (1/2 * 10 * 6) = 100 - 30 = 70.
  // Full 10x10 quad split into 2 triangles covering [0,10]x[0,10]: clipped area MUST equal 70!
  const concavePoly = [
    [0, 0],
    [10, 0],
    [10, 10],
    [5, 4],
    [0, 10],
  ];
  const quadVerts = [
    [0, 0, 0, 0, 1, 1, 1, 1],
    [10, 0, 1, 0, 1, 1, 1, 1],
    [10, 10, 1, 1, 1, 1, 1, 1],
    [0, 10, 0, 1, 1, 1, 1, 1],
  ];
  const clippedTris = clipBatchTrianglesToPolygon(quadVerts, [0, 1, 2, 0, 2, 3], concavePoly);
  let totalArea = 0;
  for (const [a, b, c] of clippedTris) {
    totalArea += Math.abs((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])) * 0.5;
  }
  assert.ok(
    Math.abs(totalArea - 70) < 1e-3,
    `expected concave-clipped quad area 70.0 (preserving both upper wings), got ${totalArea}`,
  );
});

test("Decision 35 (12): All 5 bone TransformModes (normal, onlyTranslation, noRotationOrReflection, noScale, noScaleOrReflection) incl. parent rotation & negative reflection", () => {
  const modeSpec = {
    skeleton: { spine: "4.0.64" },
    bones: [
      { name: "root", x: 0, y: 0, length: 0, transform: "normal" },
      { name: "parent", parent: "root", x: 10, y: 20, rotation: 90, scaleX: 2, scaleY: -1.5, transform: "normal" },
      { name: "c_normal", parent: "parent", x: 5, y: 0, length: 10, transform: "normal" },
      { name: "c_only_trans", parent: "parent", x: 5, y: 0, length: 10, transform: "onlyTranslation" },
      { name: "c_no_rot_refl", parent: "parent", x: 5, y: 0, length: 10, transform: "noRotationOrReflection" },
      { name: "c_no_scale", parent: "parent", x: 5, y: 0, length: 10, transform: "noScale" },
      { name: "c_no_scale_refl", parent: "parent", x: 5, y: 0, length: 10, transform: "noScaleOrReflection" },
    ],
    slots: [],
    skins: [{ name: "default", attachments: {} }],
  };
  const skel = createSkeletonInstance(parseSpine40Json(modeSpec), "default");
  const get = (name) => skel.bones[skel.boneIndexByName[name]];
  const detOf = (b) => b.a * b.d - b.b * b.c;

  // Parent rotates 90 deg, so local x=5 maps to world (10, 30)
  for (const childName of ["c_normal", "c_only_trans", "c_no_rot_refl", "c_no_scale", "c_no_scale_refl"]) {
    const c = get(childName);
    assert.ok(Math.abs(c.worldX - 10) < 1e-4, `${childName}.worldX expected 10, got ${c.worldX}`);
    assert.ok(Math.abs(c.worldY - 30) < 1e-4, `${childName}.worldY expected 30, got ${c.worldY}`);
  }

  // 1) normal: inherits full 90 deg rotation, scale (2, -1.5), and negative reflection (det = -3)
  const bNormal = get("c_normal");
  assert.ok(Math.abs(bNormal.c - 2) < 1e-4 && Math.abs(bNormal.b - 1.5) < 1e-4);
  assert.ok(Math.abs(detOf(bNormal) - -3) < 1e-4);

  // 2) onlyTranslation: ignores parent rotation/scale/reflection -> identity basis
  const bOnlyTrans = get("c_only_trans");
  assert.ok(Math.abs(bOnlyTrans.a - 1) < 1e-4 && Math.abs(bOnlyTrans.b) < 1e-4);
  assert.ok(Math.abs(bOnlyTrans.c) < 1e-4 && Math.abs(bOnlyTrans.d - 1) < 1e-4);

  // 3) noRotationOrReflection: unrotated (b=0, c=0), preserves parent axis scales (2, +1.5), positive det = +3
  const bNoRotRefl = get("c_no_rot_refl");
  assert.ok(Math.abs(bNoRotRefl.c) < 1e-4 && Math.abs(bNoRotRefl.b) < 1e-4, `noRotationOrReflection must have 0 off-diagonal terms, got b=${bNoRotRefl.b}, c=${bNoRotRefl.c}`);
  assert.ok(Math.abs(bNoRotRefl.a - 2) < 1e-4 && Math.abs(bNoRotRefl.d - 1.5) < 1e-4, `expected a=2, d=1.5, got (${bNoRotRefl.a}, ${bNoRotRefl.d})`);
  assert.ok(detOf(bNoRotRefl) > 0);

  // 4) noScale: strips parent scale magnitude (unit axis lengths 1,1) while retaining 90 deg rotation AND reflection (det = -1)
  const bNoScale = get("c_no_scale");
  assert.ok(Math.abs(Math.hypot(bNoScale.a, bNoScale.c) - 1) < 1e-4);
  assert.ok(Math.abs(Math.hypot(bNoScale.b, bNoScale.d) - 1) < 1e-4);
  assert.ok(detOf(bNoScale) < 0, `noScale must preserve negative reflection det<0, got ${detOf(bNoScale)}`);

  // 5) noScaleOrReflection: strips scale magnitude (1,1) AND strips reflection (det = +1) while preserving 90 deg rotation
  const bNoScaleRefl = get("c_no_scale_refl");
  assert.ok(Math.abs(Math.hypot(bNoScaleRefl.a, bNoScaleRefl.c) - 1) < 1e-4);
  assert.ok(Math.abs(Math.hypot(bNoScaleRefl.b, bNoScaleRefl.d) - 1) < 1e-4);
  assert.ok(detOf(bNoScaleRefl) > 0, `noScaleOrReflection must strip reflection det>0, got ${detOf(bNoScaleRefl)}`);
});

test("Decision 35 (13): Slots, unoverridden skin fallback, runtime drawOrder timeline (+ and - offsets), and batch slot ordering", () => {
  const data = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  // Add an animation testing negative drawOrder offsets cleanly
  data.animations.neg_order = {
    name: "neg_order",
    bones: {},
    slots: {},
    ik: {},
    transform: {},
    path: {},
    deform: {},
    drawOrder: [{ time: 0, offsets: [{ slot: "aura_slot", offset: -3 }] }],
    events: [],
  };
  const skel = createSkeletonInstance(data, "gold_armor");
  // Named skin `gold_armor` overrides `torso_slot`, falls back to `default` skin on `weapon_slot` and `cape_slot`
  assert.equal(resolveAttachment(skel, skel.slotIndexByName.torso_slot)?.color, "fde047ff");
  assert.equal(resolveAttachment(skel, skel.slotIndexByName.weapon_slot)?.name, "weapon_blade");
  assert.equal(resolveAttachment(skel, skel.slotIndexByName.cape_slot)?.name, "cape_cloth");

  // Setup drawOrder places weapon_slot (idx 3) before aura_slot (idx 4)
  assert.deepEqual(skel.drawOrder, [0, 1, 2, 3, 4, 5, 6]);

  // Runtime `jineng` shifts `weapon_slot` (+1 -> idx 4), after `aura_slot`!
  const state = new CleanRoomAnimationState();
  state.setAnimation(0, "jineng", false, 1, "replace");
  state.apply(skel);
  const batchesPos = buildBatchGeometry(skel, undefined, true);
  const flatSlotsPos = batchesPos.flatMap((b) => b.slotNames);
  assert.ok(flatSlotsPos.indexOf("aura_slot") < flatSlotsPos.indexOf("weapon_slot"), `positive drawOrder shift failed: ${flatSlotsPos.join(",")}`);

  // Runtime `neg_order` shifts `aura_slot` (-3 -> idx 1), placing aura before torso/cape/weapon
  resetSkeletonToSetupPose(skel);
  state.setAnimation(0, "neg_order", false, 1, "replace");
  state.apply(skel);
  const batchesNeg = buildBatchGeometry(skel, undefined, true);
  const flatSlotsNeg = batchesNeg.flatMap((b) => b.slotNames);
  assert.equal(flatSlotsNeg[0], "aura_slot", `negative drawOrder -3 expected aura_slot first, got ${flatSlotsNeg.join(",")}`);
});

test("Decision 35 (14): All 6 attachment types (region, unweighted mesh, weighted mesh, path, clipping convex+concave, boundingbox) and hiddenSlots", () => {
  const atlas = parseTextureAtlas(HANDCRAFTED_SPINE40_ATLAS_TEXT);
  const data = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const skel = createSkeletonInstance(data, "default");

  // 1) Region + unclipped/hidden clip removal check
  const torsoVerts = computeAttachmentWorldVertices(skel, skel.slotIndexByName.torso_slot);
  assert.equal(torsoVerts.length, 8);

  // 2) Unweighted mesh (`weapon_blade`) + weighted mesh (`cape_cloth`) + point check
  const weaponVerts = computeAttachmentWorldVertices(skel, skel.slotIndexByName.weapon_slot);
  assert.equal(weaponVerts.length, 8);
  const capeVertsBefore = computeAttachmentWorldVertices(skel, skel.slotIndexByName.cape_slot);
  skel.bones[skel.boneIndexByName.cape_follower].x += 20;
  updateSkeletonWorldTransform(skel);
  const capeVertsAfter = computeAttachmentWorldVertices(skel, skel.slotIndexByName.cape_slot);
  assert.notDeepEqual(capeVertsBefore, capeVertsAfter);

  // 3) Path attachment (`ribbon_path` — 3-knot Spine 4.0 Bezier control points)
  const pathVerts = computeAttachmentWorldVertices(skel, skel.slotIndexByName.path_guide_slot);
  const expectedPath = [-40, 10, -40, 10, -40, 10, 0, 40, 0, 40, 0, 40, 40, 10, 40, 10, 40, 10];
  assert.equal(pathVerts.length, expectedPath.length);
  for (let i = 0; i < expectedPath.length; i++) {
    assert.ok(Math.abs(pathVerts[i] - expectedPath[i]) < 1e-10, `pathVerts[${i}] expected ${expectedPath[i]}, got ${pathVerts[i]}`);
  }

  // 4) BoundingBox hit testing shifts dynamically when torso moves
  const boxBefore = computeAttachmentWorldVertices(skel, skel.slotIndexByName.hit_bounds_slot);
  assert.equal(containsPointInBoundingBox(boxBefore, 0, 50), true);
  assert.equal(containsPointInBoundingBox(boxBefore, -300, -300), false);
  skel.bones[skel.boneIndexByName.torso].x = 200;
  updateSkeletonWorldTransform(skel);
  const boxMoved = computeAttachmentWorldVertices(skel, skel.slotIndexByName.hit_bounds_slot);
  assert.equal(containsPointInBoundingBox(boxMoved, 0, 50), false);
  assert.equal(containsPointInBoundingBox(boxMoved, 200, 50), true);

  // 5) Convex & Concave ClippingAttachment inside buildBatchGeometry + hiddenSlots
  const skelClip = createSkeletonInstance(data, "default");
  // Hide torso_slot and shrink clip_window to a concave chevron clipping `cape_slot`
  const clipAtt = skelClip.data.skins[0].attachments.clip_window.viewport_clip;
  assert.equal(clipAtt.type, "clipping");
  clipAtt.vertexCount = 5;
  clipAtt.vertices = [-120, -60, 120, -60, 120, 180, 0, 20, -120, 180];
  const batchesConcave = buildBatchGeometry(skelClip, atlas, true, new Set(["torso_slot"]));
  assert.equal(batchesConcave[0].clipped, true);
  assert.deepEqual(batchesConcave[0].slotNames, ["cape_slot", "weapon_slot"]);
  assert.ok(batchesConcave[0].indices.length >= 6);
});

test("Decision 35 (15): 1-bone IK (incl. stretch & partial mix) and 2-bone IK (bendPositive true vs false + clamped unreachable)", () => {
  // 1-bone IK with stretch=true and partial mix
  const ik1Json = {
    skeleton: { spine: "4.0.64" },
    bones: [
      { name: "root", transform: "normal" },
      { name: "aim", parent: "root", x: 0, y: 0, length: 20, rotation: 0, transform: "normal" },
      { name: "target", parent: "root", x: 40, y: 40, length: 0, transform: "normal" },
    ],
    ik: [{ name: "aim_ik", bones: ["aim"], target: "target", mix: 1, stretch: true, bendPositive: true }],
    slots: [],
    skins: [{ name: "default", attachments: {} }],
  };
  const skel1 = createSkeletonInstance(parseSpine40Json(ik1Json), "default");
  const aim = skel1.bones[skel1.boneIndexByName.aim];
  const tipX = aim.worldX + aim.a * aim.data.length;
  const tipY = aim.worldY + aim.c * aim.data.length;
  assert.ok(Math.hypot(tipX - 40, tipY - 40) < 1e-3, `1-bone stretch IK tip expected (40,40), got (${tipX}, ${tipY})`);

  // Partial mix = 0 stops IK override
  skel1.ikConstraints[0].mix = 0;
  resetSkeletonToSetupPose(skel1);
  skel1.ikConstraints[0].mix = 0;
  updateSkeletonWorldTransform(skel1);
  assert.ok(Math.abs(aim.rotation) < 1e-5);

  // 2-bone IK bendPositive=true vs bendPositive=false produces opposite elbow Y coordinates
  const make2Bone = (bendPositive, targetX = 30, targetY = 0) => {
    const s = createSkeletonInstance(
      parseSpine40Json({
        skeleton: { spine: "4.0.64" },
        bones: [
          { name: "root", transform: "normal" },
          { name: "upper", parent: "root", x: 0, y: 0, length: 30, rotation: 0 },
          { name: "lower", parent: "upper", x: 30, y: 0, length: 30, rotation: 0 },
          { name: "target", parent: "root", x: targetX, y: targetY },
        ],
        ik: [{ name: "arm_ik", bones: ["upper", "lower"], target: "target", mix: 1, bendPositive }],
        slots: [],
        skins: [{ name: "default", attachments: {} }],
      }),
      "default",
    );
    const lower = s.bones[s.boneIndexByName.lower];
    return { elbowY: lower.worldY, tipX: lower.worldX + lower.a * 30, tipY: lower.worldY + lower.c * 30 };
  };
  const posBend = make2Bone(true);
  const negBend = make2Bone(false);
  assert.ok(Math.hypot(posBend.tipX - 30, posBend.tipY) < 1e-3);
  assert.ok(Math.hypot(negBend.tipX - 30, negBend.tipY) < 1e-3);
  assert.ok(posBend.elbowY < 0 && negBend.elbowY > 0, `expected opposite elbow signs, pos=${posBend.elbowY}, neg=${negBend.elbowY}`);

  // Unreachable target (100, 100) clamps smoothly to straight line at full reach l1+l2=60 along 45 deg
  const unreachable = make2Bone(true, 100, 100);
  const expectedReach = 60 * Math.SQRT1_2;
  assert.ok(
    Math.hypot(unreachable.tipX - expectedReach, unreachable.tipY - expectedReach) < 0.05,
    `expected unreachable clamp near (${expectedReach.toFixed(2)}, ${expectedReach.toFixed(2)}), got (${unreachable.tipX}, ${unreachable.tipY})`,
  );
});

test("Decision 35 (16): TransformConstraint (rotate, translate, scale, shear mixes, local vs world, relative vs absolute, child propagation)", () => {
  const trJson = {
    skeleton: { spine: "4.0.64" },
    bones: [
      { name: "root" },
      { name: "src", parent: "root", x: 50, y: 80, rotation: 40, scaleX: 1.8, scaleY: 0.6, shearY: 12 },
      { name: "dst_abs_local", parent: "root", x: 0, y: 0, rotation: 0, scaleX: 1, scaleY: 1, shearY: 0 },
      { name: "dst_child", parent: "dst_abs_local", x: 10, y: 0, length: 10 },
      { name: "dst_rel_world", parent: "root", x: 10, y: -20, rotation: 10, scaleX: 1, scaleY: 1 },
    ],
    transform: [
      {
        name: "tr_local",
        order: 0,
        bones: ["dst_abs_local"],
        target: "src",
        local: true,
        relative: false,
        offsetRotation: 5,
        offsetX: 10,
        offsetY: -10,
        offsetScaleX: 0.2,
        offsetScaleY: 0.4,
        offsetShearY: 3,
        mixRotate: 1,
        mixX: 0.5,
        mixY: 1,
        mixScaleX: 1,
        mixScaleY: 1,
        mixShearY: 1,
      },
      {
        name: "tr_world_rel",
        order: 1,
        bones: ["dst_rel_world"],
        target: "src",
        local: false,
        relative: true,
        offsetRotation: -5,
        offsetX: 5,
        offsetY: 5,
        mixRotate: 0.5,
        mixX: 1,
        mixY: 1,
        mixScaleX: 0,
        mixScaleY: 0,
        mixShearY: 0,
      },
    ],
    slots: [],
    skins: [{ name: "default", attachments: {} }],
  };
  const skel = createSkeletonInstance(parseSpine40Json(trJson), "default");
  const dst = skel.bones[skel.boneIndexByName.dst_abs_local];
  assert.ok(Math.abs(dst.rotation - 45) < 1e-4);
  assert.ok(Math.abs(dst.x - 30) < 1e-4); // 0 -> (50 + 10) * 0.5 = 30
  assert.ok(Math.abs(dst.y - 70) < 1e-4); // 80 - 10
  assert.ok(Math.abs(dst.scaleX - 2.0) < 1e-4);
  assert.ok(Math.abs(dst.scaleY - 1.0) < 1e-4);
  assert.ok(Math.abs(dst.shearY - 15) < 1e-4);

  // Descendant `dst_child` receives updated parent transform automatically
  const child = skel.bones[skel.boneIndexByName.dst_child];
  assert.ok(Math.hypot(child.worldX - (dst.worldX + dst.a * 10), child.worldY - (dst.worldY + dst.c * 10)) < 1e-4);

  // Relative world constraint shifts dst_rel_world rotation & position smoothly
  const rel = skel.bones[skel.boneIndexByName.dst_rel_world];
  assert.ok(Math.abs(rel.worldX - (10 + 50 + 5)) < 1e-4);
  assert.ok(Math.abs(rel.worldY - (-20 + 80 + 5)) < 1e-4);
  assert.ok(Math.abs(rel.rotation - 27.5) < 1e-4); // 10 + 0.5 * ((10 + 40 - 5) - 10)
});

test("Decision 35 (17): PathConstraint (positionMode percent/fixed, spacingMode length/fixed/percent, partial mix preservation)", () => {
  const makePathSkel = (positionMode, spacingMode, position, spacing, mixX = 1) =>
    createSkeletonInstance(
      parseSpine40Json({
        skeleton: { spine: "4.0.64" },
        bones: [
          { name: "root" },
          { name: "b0", parent: "root", x: 0, y: 0, length: 20 },
          { name: "b1", parent: "root", x: 0, y: 0, length: 20 },
        ],
        slots: [{ name: "ps", bone: "root", attachment: "p" }],
        path: [
          {
            name: "pc",
            bones: ["b0", "b1"],
            target: "ps",
            positionMode,
            spacingMode,
            rotateMode: "tangent",
            offsetRotation: 0,
            position,
            spacing,
            mixRotate: 1,
            mixX,
            mixY: 1,
          },
        ],
        skins: [
          {
            name: "default",
            attachments: {
              ps: {
                p: {
                  type: "path",
                  closed: false,
                  constantSpeed: true,
                  vertexCount: 6,
                  vertices: [0, 0, 0, 0, 0, 0, 100, 0, 100, 0, 100, 0],
                  lengths: [100, 100],
                },
              },
            },
          },
        ],
      }),
      "default",
    );

  // fixed position 25 / 100 = x=25, spacing percent 0.5 -> b1 at x=75
  const sFixed = makePathSkel("fixed", "percent", 25, 0.5, 1);
  assert.ok(Math.abs(sFixed.bones[sFixed.boneIndexByName.b0].worldX - 25) < 1e-3);
  assert.ok(Math.abs(sFixed.bones[sFixed.boneIndexByName.b1].worldX - 75) < 1e-3);

  // percent position 0.2 -> x=20, fixed spacing 30 -> b1 at x=50
  const sFixedSp = makePathSkel("percent", "fixed", 0.2, 30, 1);
  assert.ok(Math.abs(sFixedSp.bones[sFixedSp.boneIndexByName.b0].worldX - 20) < 1e-3);
  assert.ok(Math.abs(sFixedSp.bones[sFixedSp.boneIndexByName.b1].worldX - 50) < 1e-3);

  // Partial mixX = 0.5 must NOT be overwritten to sample.x when mixRotate > 0!
  const sPartial = makePathSkel("percent", "fixed", 1.0, 0, 0.5);
  assert.ok(Math.abs(sPartial.bones[sPartial.boneIndexByName.b0].worldX - 50) < 1e-3);
});

test("Decision 35 (18): All 12 timeline channels (rotate/translate/scale/shear/rgba/attachment/deform/ik/transform/path/drawOrder/event) and curves", () => {
  // Asymmetric cubic Bezier curve test (`Bx(s) = time` vs fast-ease `By(s)`)
  const bezVal = evaluateCubicBezier(0, 0, 0.1, 8, 0.2, 10, 1, 10, 0.5);
  assert.ok(bezVal > 8.5, `fast ease-out cubic Bezier at t=0.5 should exceed linear 5, got ${bezVal}`);

  const fullTlJson = {
    skeleton: { spine: "4.0.64" },
    bones: [
      { name: "root" },
      { name: "b", parent: "root", x: 0, y: 0, length: 20, rotation: 0, scaleX: 1, scaleY: 1, shearX: 0, shearY: 0 },
      { name: "b_ik", parent: "root", x: 0, y: 0, length: 20, rotation: 0 },
      { name: "ik_tgt", parent: "root", x: 0, y: 20 },
      { name: "tr_dst", parent: "root", x: 0, y: 0 },
      { name: "p_node", parent: "root", x: 0, y: 0 },
    ],
    slots: [
      { name: "s0", bone: "b", color: "ffffffff", attachment: "m0" },
      { name: "s1", bone: "root", color: "ffffffff", attachment: "guide" },
    ],
    ik: [{ name: "ik1", bones: ["b_ik"], target: "ik_tgt", mix: 0 }],
    transform: [{ name: "tr1", bones: ["tr_dst"], target: "b", local: true, relative: false, mixRotate: 0, mixX: 0, mixY: 0, mixScaleX: 0, mixScaleY: 0, mixShearY: 0 }],
    path: [{ name: "p1", bones: ["p_node"], target: "s1", position: 0, spacing: 10, mixRotate: 0, mixX: 0, mixY: 0 }],
    events: { footstep: { int: 1, float: 0.25, string: "left", audio: "step.ogg", volume: 0.8, balance: -0.2 } },
    skins: [
      {
        name: "default",
        attachments: {
          s0: {
            m0: { type: "mesh", uvs: [0, 0, 1, 0, 0, 1], triangles: [0, 1, 2], vertices: [0, 0, 10, 0, 0, 10], width: 10, height: 10 },
            m1: { type: "region", x: 0, y: 0, width: 12, height: 12 },
          },
          s1: {
            guide: { type: "path", closed: false, constantSpeed: true, vertexCount: 6, vertices: [0, 0, 0, 0, 0, 0, 40, 0, 40, 0, 40, 0], lengths: [40, 40] },
          },
        },
      },
    ],
    animations: {
      all_channels: {
        bones: {
          b: {
            rotate: [{ time: 0, value: 0 }, { time: 1, value: 90 }],
            translate: [{ time: 0, x: 0, y: 0 }, { time: 1, x: 40, y: 20 }],
            scale: [{ time: 0, x: 1, y: 1 }, { time: 1, x: 2, y: 3 }],
            shear: [{ time: 0, x: 0, y: 0 }, { time: 1, x: 10, y: 20 }],
          },
        },
        slots: {
          s0: {
            rgba: [{ time: 0, color: "00000000" }, { time: 1, color: "ffffffff" }],
            attachment: [{ time: 0, name: "m0" }, { time: 0.5, name: "m1" }, { time: 0.9, name: null }],
          },
        },
        ik: { ik1: [{ time: 0, mix: 0 }, { time: 1, mix: 1 }] },
        transform: { tr1: [{ time: 0, mixRotate: 0, mixX: 0, mixY: 0, mixScaleX: 0, mixScaleY: 0, mixShearY: 0 }, { time: 1, mixRotate: 1, mixX: 1, mixY: 1, mixScaleX: 0, mixScaleY: 0, mixShearY: 0 }] },
        path: { p1: { position: [{ time: 0, value: 0 }, { time: 1, value: 1 }], spacing: [{ time: 0, value: 10 }, { time: 1, value: 20 }], mix: [{ time: 0, mixRotate: 0, mixX: 0, mixY: 0 }, { time: 1, mixRotate: 1, mixX: 1, mixY: 1 }] } },
        deform: { default: { s0: { m0: [{ time: 0, offset: 0, vertices: [0, 0] }, { time: 1, offset: 0, vertices: [6, 8] }] } } },
        drawOrder: [{ time: 0.5, offsets: [{ slot: "s1", offset: -1 }] }],
        events: [
          { time: 0.25, name: "footstep", int: 2, string: "right" },
          { time: 0.75, name: "footstep" },
        ],
      },
    },
  };
  const data = parseSpine40Json(fullTlJson);
  assert.equal(getAnimationDuration(data, "all_channels"), 1.0);
  const skel = createSkeletonInstance(data, "default");
  const st = new CleanRoomAnimationState();
  const cbEvents = [];
  st.onEvent = (ev) => cbEvents.push(ev);
  st.setAnimation(0, "all_channels", true, 1, "replace");

  // Step 1: advance to 0.25 (`m0` active -> check deform, rotate, untouched drawOrder, and event #1!)
  st.update(0.25);
  st.apply(skel);
  const b = skel.bones[skel.boneIndexByName.b];
  const s0 = skel.slots[skel.slotIndexByName.s0];
  assert.ok(Math.abs(b.rotation - 22.5) < 1e-4, `expected rotate=22.5 at t=0.25, got ${b.rotation}`);
  assert.equal(s0.attachmentName, "m0");
  assert.ok(Math.abs(s0.deform[0] - 1.5) < 1e-4 && Math.abs(s0.deform[1] - 2.0) < 1e-4, `expected deform [1.5,2] at t=0.25, got ${JSON.stringify(s0.deform)}`);
  assert.deepEqual(skel.drawOrder, [0, 1]); // drawOrder keyframe is at t=0.5 -> untouched at t=0.25!
  assert.equal(st.firedEvents.length, 1);
  assert.equal(st.firedEvents[0].int, 2);
  assert.equal(st.firedEvents[0].string, "right");
  assert.equal(st.firedEvents[0].audio, "step.ogg");

  // Step 2: reset to setup pose then advance to 0.5 (linear midpoints + ik1 50% + attachment m1 + drawOrder shift)
  resetSkeletonToSetupPose(skel);
  st.update(0.25);
  st.apply(skel);
  assert.ok(Math.abs(b.rotation - 45) < 1e-4);
  assert.ok(Math.abs(b.x - 20) < 1e-4 && Math.abs(b.y - 10) < 1e-4);
  assert.ok(Math.abs(b.scaleX - 1.5) < 1e-4 && Math.abs(b.scaleY - 2.0) < 1e-4);
  assert.ok(Math.abs(b.shearX - 5) < 1e-4 && Math.abs(b.shearY - 10) < 1e-4);
  assert.ok(Math.abs(s0.color[0] - 0.5) < 0.01 && Math.abs(s0.color[3] - 0.5) < 0.01);
  assert.equal(s0.attachmentName, "m1");
  assert.ok(Math.abs(skel.ikConstraints[0].mix - 0.5) < 1e-4);
  assert.ok(Math.abs(skel.bones[skel.boneIndexByName.b_ik].rotation - 45) < 1e-4, `expected b_ik rotation=45, got ${skel.bones[skel.boneIndexByName.b_ik].rotation}`);
  assert.ok(Math.abs(skel.transformConstraints[0].mixX - 0.5) < 1e-4);
  assert.ok(Math.abs(skel.pathConstraints[0].position - 0.5) < 1e-4);
  assert.ok(Math.abs(skel.pathConstraints[0].spacing - 15) < 1e-4);
  assert.ok(Math.abs(skel.pathConstraints[0].mixX - 0.5) < 1e-4);
  assert.deepEqual(skel.drawOrder, [1, 0]);
  assert.equal(st.firedEvents.length, 0);

  // Re-applying same frame without update must NOT double-fire events
  st.apply(skel);
  assert.equal(st.firedEvents.length, 0);

  // Advance across t=0.75 (2nd event with default int=1, string="left") -> 0.95 (null attachment; avoid 0.5+0.4=0.8999999999999999 float underflow)
  st.update(0.45);
  st.apply(skel);
  assert.equal(s0.attachmentName, null);
  assert.equal(st.firedEvents.length, 1);
  assert.equal(st.firedEvents[0].int, 1);
  assert.equal(st.firedEvents[0].string, "left");

  // Wrap around loop past duration=1.0 to t=1.35 -> fires 0.25 event on loop #2!
  st.update(0.4);
  st.apply(skel);
  assert.equal(st.firedEvents.length, 1);
  assert.equal(st.firedEvents[0].time, 0.25);
  assert.equal(cbEvents.length, 3);
});

test("Decision 35 (19): Multi-track mixing (replace + add + track alpha across translate, scale, shear, deform)", () => {
  const mixJson = {
    skeleton: { spine: "4.0.64" },
    bones: [{ name: "root" }, { name: "b", parent: "root", x: 0, y: 0, scaleX: 1, scaleY: 1, shearX: 0, shearY: 0 }],
    slots: [{ name: "s", bone: "b", attachment: "m" }],
    skins: [{ name: "default", attachments: { s: { m: { type: "mesh", uvs: [0, 0, 1, 0, 0, 1], triangles: [0, 1, 2], vertices: [0, 0, 10, 0, 0, 10] } } } }],
    animations: {
      base: {
        bones: { b: { translate: [{ time: 0, x: 10, y: 0 }], scale: [{ time: 0, x: 2, y: 2 }], shear: [{ time: 0, x: 4, y: 0 }] } },
        deform: { default: { s: { m: [{ time: 0, offset: 0, vertices: [2, 0] }] } } },
      },
      overlay: {
        bones: { b: { translate: [{ time: 0, x: 20, y: 0 }], scale: [{ time: 0, x: 1.5, y: 1 }], shear: [{ time: 0, x: 8, y: 0 }] } },
        deform: { default: { s: { m: [{ time: 0, offset: 0, vertices: [4, 0] }] } } },
      },
    },
  };
  const skel = createSkeletonInstance(parseSpine40Json(mixJson), "default");
  const st = new CleanRoomAnimationState();
  st.setAnimation(0, "base", false, 1, "replace");
  st.setAnimation(1, "overlay", false, 0.5, "add");
  st.apply(skel);
  const b = skel.bones[skel.boneIndexByName.b];
  // translate: base 10 + 0.5 * 20 = 20
  assert.ok(Math.abs(b.x - 20) < 1e-4);
  // scale: base 2 -> target (2 * 1.5 = 3), mixed by alpha 0.5 => 2.5
  assert.ok(Math.abs(b.scaleX - 2.5) < 1e-4);
  // shear: base 4 + 0.5 * 8 = 8
  assert.ok(Math.abs(b.shearX - 8) < 1e-4);
  // deform: base 2 + 0.5 * 4 = 4
  assert.ok(Math.abs(skel.slots[0].deform[0] - 4) < 1e-4);
});

test("Decision 35 (20): Atlas parser (Spine 4.0 compact & legacy xy/size/orig/offset/rotate:true) + rotate:90 UV remapping", () => {
  const legacyAtlasText = `
page_legacy.png
size: 512, 256
format: RGBA8888
filter: Linear, Linear
repeat: none
pma: false
banner
  rotate: true
  xy: 64, 32
  size: 100, 40
  orig: 120, 48
  offset: 10, 4
  index: 2
page_extra.png
size: 128, 128
pma: true
gem
  bounds: 0, 0, 32, 32
  offsets: 0, 0, 32, 32
`.trim();
  const atlas = parseTextureAtlas(legacyAtlasText);
  assert.equal(atlas.pages.length, 2);
  assert.equal(atlas.pages[0].pma, false);
  assert.equal(atlas.pages[1].pma, true);
  const banner = atlas.regions.banner;
  assert.equal(banner.rotate, 90);
  assert.equal(banner.index, 2);
  assert.equal(banner.originalWidth, 120);
  assert.equal(banner.offsetX, 10);
  // With rotate:90 in 512x256, packed width = height(40) / 512, packed height = width(100) / 256
  assert.ok(Math.abs(banner.u - 64 / 512) < 1e-5);
  assert.ok(Math.abs(banner.u2 - (64 + 40) / 512) < 1e-5);
  assert.ok(Math.abs(banner.v - 32 / 256) < 1e-5);
  assert.ok(Math.abs(banner.v2 - (32 + 100) / 256) < 1e-5);

  // Verify rotate:90 remaps region corners and mesh UVs accurately in buildBatchGeometry
  const rotSkel = createSkeletonInstance(
    parseSpine40Json({
      skeleton: { spine: "4.0.64" },
      bones: [{ name: "root" }],
      slots: [
        { name: "rs", bone: "root", attachment: "banner" },
        { name: "ms", bone: "root", attachment: "banner_mesh" },
      ],
      skins: [
        {
          name: "default",
          attachments: {
            rs: { banner: { type: "region", width: 100, height: 40 } },
            ms: {
              banner_mesh: {
                type: "mesh",
                path: "banner",
                uvs: [0, 0, 1, 0, 1, 1, 0, 1],
                triangles: [0, 1, 2, 2, 3, 0],
                vertices: [0, 0, 10, 0, 10, 10, 0, 10],
              },
            },
          },
        },
      ],
    }),
    "default",
  );
  const batches = buildBatchGeometry(rotSkel, atlas, false);
  // First 4 vertices are `banner` (region rotated 90 deg CCW -> corner 0 [BL] maps to (u2, v2), corner 2 [TR] maps to (u, v))
  const v = batches[0].vertices;
  assert.ok(Math.abs(v[2] - banner.u2) < 1e-5 && Math.abs(v[3] - banner.v2) < 1e-5);
  assert.ok(Math.abs(v[2 * 8 + 2] - banner.u) < 1e-5 && Math.abs(v[2 * 8 + 3] - banner.v) < 1e-5);
  // Mesh vertex 0 at uv (0,0) maps to (u, v2); mesh vertex 1 at uv (1,0) maps to (u, v)
  assert.ok(Math.abs(v[4 * 8 + 2] - banner.u) < 1e-5 && Math.abs(v[4 * 8 + 3] - banner.v2) < 1e-5);
  assert.ok(Math.abs(v[5 * 8 + 2] - banner.u) < 1e-5 && Math.abs(v[5 * 8 + 3] - banner.v) < 1e-5);
});

test("Decision 35 (21): WebGL RGBA texture upload, pageTextures binding, non-PMA vs PMA RGB+blend modes across all 4 blend modes, and Canvas2D renderer", () => {
  const data = parseSpine40Json({
    skeleton: { spine: "4.0.64" },
    bones: [{ name: "root", length: 20 }],
    slots: [
      { name: "s_norm", bone: "root", color: "80402080", attachment: "r", blend: "normal" },
      { name: "s_add", bone: "root", color: "ffffffff", attachment: "r", blend: "additive" },
      { name: "s_mul", bone: "root", color: "ffffffff", attachment: "r", blend: "multiply" },
      { name: "s_scr", bone: "root", color: "ffffffff", attachment: "r", blend: "screen" },
    ],
    skins: [{ name: "default", attachments: { s_norm: { r: { type: "region", width: 10, height: 10 } }, s_add: { r: { type: "region", width: 10, height: 10 } }, s_mul: { r: { type: "region", width: 10, height: 10 } }, s_scr: { r: { type: "region", width: 10, height: 10 } } } }],
  });
  const atlas = parseTextureAtlas("page_a.png\nsize: 64, 64\nr\n  bounds: 0, 0, 32, 32\n  offsets: 0, 0, 32, 32");
  const skel = createSkeletonInstance(data, "default");

  // PMA=true vs PMA=false color multiplication & blend factors
  const pmaBatches = buildBatchGeometry(skel, atlas, true);
  const nonPmaBatches = buildBatchGeometry(skel, atlas, false);
  assert.equal(pmaBatches.length, 4);
  const aAlpha = nonPmaBatches[0].vertices[7];
  assert.ok(Math.abs(pmaBatches[0].vertices[4] - nonPmaBatches[0].vertices[4] * aAlpha) < 1e-3);
  assert.equal(pmaBatches[0].blendFactors.srcRgb, GL_ONE);
  assert.equal(nonPmaBatches[0].blendFactors.srcRgb, GL_SRC_ALPHA);
  assert.equal(nonPmaBatches[1].blendFactors.srcRgb, GL_SRC_ALPHA);
  assert.equal(pmaBatches[1].blendFactors.srcRgb, GL_ONE);
  assert.equal(pmaBatches[2].blendFactors.srcRgb, GL_DST_COLOR);
  assert.equal(pmaBatches[2].blendFactors.srcAlpha, GL_ONE);
  assert.equal(pmaBatches[2].blendFactors.dstAlpha, GL_ONE_MINUS_SRC_ALPHA);
  assert.equal(pmaBatches[3].blendFactors.dstRgb, GL_ONE_MINUS_SRC_COLOR);

  // uploadRgbaTextureToWebGL + pageTextures binding
  const boundTextures = [];
  const uploadedPixels = [];
  const glMock = {
    ARRAY_BUFFER: 0x8892, ELEMENT_ARRAY_BUFFER: 0x8893, DYNAMIC_DRAW: 0x88e8, FLOAT: 0x1406,
    TRIANGLES: 0x0004, UNSIGNED_SHORT: 0x1403, BLEND: 0x0be2, COLOR_BUFFER_BIT: 0x4000,
    VERTEX_SHADER: 0x8b31, FRAGMENT_SHADER: 0x8b30, COMPILE_STATUS: 0x8b81, LINK_STATUS: 0x8b82,
    TEXTURE_2D: 0x0de1, TEXTURE0: 0x84c0, RGBA: 0x1908, UNSIGNED_BYTE: 0x1401,
    TEXTURE_MIN_FILTER: 0x2801, TEXTURE_MAG_FILTER: 0x2800, TEXTURE_WRAP_S: 0x2802, TEXTURE_WRAP_T: 0x2803,
    LINEAR: 0x2601, CLAMP_TO_EDGE: 0x812f,
    viewport() {}, clearColor() {}, clear() {}, enable() {}, blendFuncSeparate() {},
    createBuffer: () => ({}), bindBuffer() {}, bufferData() {}, drawElements() {},
    createShader: () => ({}), shaderSource() {}, compileShader() {}, getShaderParameter: () => true,
    createProgram: () => ({}), attachShader() {}, linkProgram() {}, getProgramParameter: () => true,
    useProgram() {}, getAttribLocation: () => 0, getUniformLocation: () => ({}),
    enableVertexAttribArray() {}, vertexAttribPointer() {}, uniform2f() {}, uniform1i() {},
    createTexture: () => ({ id: "page_tex" }),
    activeTexture() {},
    bindTexture(_t, tex) { boundTextures.push(tex); },
    texImage2D(_t, _l, _if, w, h, _b, _f, _ty, pixels) { uploadedPixels.push({ w, h, len: pixels?.byteLength ?? 0 }); },
    texParameteri() {},
  };
  const texHandle = uploadRgbaTextureToWebGL(glMock, 2, 2, new Uint8ClampedArray(16).fill(255));
  assert.ok(texHandle);
  assert.equal(uploadedPixels[uploadedPixels.length - 1].len, 16);
  const webglRes = renderSpineBatchesToWebGL(glMock, pmaBatches, 256, 256, { "page_a.png": texHandle });
  assert.equal(webglRes.drawCallCount, 4);
  assert.ok(boundTextures.includes(texHandle));

  // Canvas2D software fallback renders triangles cleanly
  let moves = 0, fills = 0;
  const ctx2d = {
    canvas: { width: 220, height: 140 },
    clearRect() {}, save() {}, translate() {}, scale() {},
    beginPath() {}, moveTo() { moves += 1; }, lineTo() {}, closePath() {},
    fill() { fills += 1; }, stroke() {}, arc() {}, restore() {},
    fillStyle: "", strokeStyle: "", lineWidth: 1,
  };
  const c2dRes = renderSpineBatchesToCanvas2D(ctx2d, skel, pmaBatches);
  assert.equal(c2dRes.drawCallCount, 4);
  assert.equal(c2dRes.triangleCount, 8);
  assert.equal(moves, 8 + skel.bones.length); // 8 triangle paths + debug bone segments
  assert.equal(fills, 8 + skel.bones.length); // 8 triangle fills + debug bone joint dots
});

test("Decision 35 (22): Standard binary .skel lossless round-trip for skeleton events, per-animation events, and dynamicSkin.js options", () => {
  const evJson = structuredClone(HANDCRAFTED_SPINE40_SKELETON_JSON);
  evJson.events = {
    hit_sfx: { int: 7, float: 1.5, string: "clang", audio: "hit.mp3", volume: 0.9, balance: 0.25 },
    step: { int: 0, float: 0, string: "walk" },
  };
  evJson.animations.gongji.events = [
    { time: 0.3, name: "hit_sfx", int: 9, float: 2.25, string: "crit", volume: 1.0, balance: -0.5 },
    { time: 0.6, name: "step" },
  ];
  const parsed = parseSpine40Json(evJson);
  const skelBytes = encodeSpine40Binary(parsed);
  const decoded = parseSpine40Binary(skelBytes);

  assert.equal(decoded.events?.length, 2);
  assert.equal(decoded.events?.[0].name, "hit_sfx");
  assert.equal(decoded.events?.[0].audio, "hit.mp3");
  assert.ok(Math.abs((decoded.events?.[0].volume ?? 0) - 0.9) < 1e-4);
  assert.equal(decoded.animations.gongji.events?.length, 2);
  assert.equal(decoded.animations.gongji.events?.[0].name, "hit_sfx");
  assert.equal(decoded.animations.gongji.events?.[0].int, 9);
  assert.ok(Math.abs((decoded.animations.gongji.events?.[0].float ?? 0) - 2.25) < 1e-4);
  assert.equal(decoded.animations.gongji.events?.[0].string, "crit");
  assert.ok(Math.abs((decoded.animations.gongji.events?.[0].balance ?? 0) - -0.5) < 1e-4);

  // saveSkinParams / dynamicSkin.js normalized metadata options (`flipX`, `clipSlots`, `hideSlots`, `version`)
  const imp = resolveDynamicSkinImport(
    { generalId: "guanyu", skinName: "武圣显灵", version: "4.0.64", x: [12, 4], y: [8, 0], scale: 1.2, flipX: true, clipSlots: ["clip_window"], hideSlots: ["aura_slot"] },
    skelBytes,
  );
  assert.equal(imp.status, "ready-spine40");
  assert.equal(imp.normalizedParams.flipX, true);
  assert.deepEqual(imp.normalizedParams.clipSlots, ["clip_window"]);
  assert.deepEqual(imp.normalizedParams.hideSlots, ["aura_slot"]);
  assert.equal(imp.skeletonData?.animations.gongji.events?.length, 2);

  // Corrupted binary header or empty bytes falls back cleanly without throwing
  const badBin = resolveDynamicSkinImport({ generalId: "guanyu", skinName: "损坏资源" }, new Uint8Array([1, 2, 3]));
  assert.equal(badBin.status, "static-portrait-fallback");
  assert.match(badBin.reason, /SPINE_40_PARSE_FALLBACK/);
});

test("Decision 35 (23): triangulateSimplePolygon decomposes arbitrary non-convex polygons into area-preserving CCW triangles", () => {
  const lShape = [
    [0, 0],
    [10, 0],
    [10, 4],
    [4, 4],
    [4, 10],
    [0, 10],
  ];
  const tris = triangulateSimplePolygon(lShape);
  assert.equal(tris.length, 4); // n - 2 triangles for 6-gon
  const sumArea = tris.reduce(
    (acc, [a, b, c]) => acc + Math.abs((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])) * 0.5,
    0,
  );
  // L-shape area: 10*4 + 4*6 = 64
  assert.ok(Math.abs(sumArea - 64) < 1e-5, `expected area 64, got ${sumArea}`);
});

test("Decision 35 (24): Regression checks for noScale reflected rotation, multi-offset drawOrder, PathConstraint parent rot, TransformConstraint shear wrap, multiply srcAlpha, and binary PathAttachment.lengths", () => {
  // 1) Reflected parent (rot=90, scaleX=2, scaleY=-1.5) + noScale child with rotation=30 -> reflected world angle 60 deg (a=+0.5, c=+0.866)
  const reflSkel = createSkeletonInstance(
    parseSpine40Json({
      skeleton: { spine: "4.0.64" },
      bones: [
        { name: "root" },
        { name: "p", parent: "root", rotation: 90, scaleX: 2, scaleY: -1.5 },
        { name: "c_ns", parent: "p", rotation: 30, transform: "noScale" },
        { name: "c_nsor", parent: "p", rotation: 30, transform: "noScaleOrReflection" },
      ],
      slots: [],
      skins: [{ name: "default", attachments: {} }],
    }),
    "default",
  );
  const cNs = reflSkel.bones[reflSkel.boneIndexByName.c_ns];
  assert.ok(Math.abs(cNs.a - 0.5) < 1e-4 && Math.abs(cNs.c - Math.sqrt(3) / 2) < 1e-4, `expected a=0.5, c=0.866, got (${cNs.a}, ${cNs.c})`);
  const cNsor = reflSkel.bones[reflSkel.boneIndexByName.c_nsor];
  assert.ok(Math.abs(cNsor.a - -0.5) < 1e-4 && Math.abs(cNsor.c - Math.sqrt(3) / 2) < 1e-4, `expected a=-0.5, c=0.866, got (${cNsor.a}, ${cNsor.c})`);

  // 2) Multi-offset drawOrder without splice shifting & untouched before first keyframe
  const drawSkel = createSkeletonInstance(
    parseSpine40Json({
      skeleton: { spine: "4.0.64" },
      bones: [{ name: "root" }],
      slots: [
        { name: "s0", bone: "root" },
        { name: "s1", bone: "root" },
        { name: "s2", bone: "root" },
        { name: "s3", bone: "root" },
        { name: "s4", bone: "root" },
      ],
      skins: [{ name: "default", attachments: {} }],
      animations: {
        multi: {
          drawOrder: [{ time: 0.5, offsets: [{ slot: "s0", offset: 2 }, { slot: "s3", offset: -2 }] }],
        },
      },
    }),
    "default",
  );
  const drawSt = new CleanRoomAnimationState();
  drawSt.setAnimation(0, "multi", false, 1, "replace");
  drawSt.update(0.25);
  drawSt.apply(drawSkel);
  assert.deepEqual(drawSkel.drawOrder, [0, 1, 2, 3, 4]);
  drawSt.update(0.25);
  drawSt.apply(drawSkel);
  assert.deepEqual(drawSkel.drawOrder, [1, 3, 0, 2, 4]);

  // 3) PathConstraint under 90-deg parent aligns bone world angle to 0 deg along horizontal path
  const pRotSkel = createSkeletonInstance(
    parseSpine40Json({
      skeleton: { spine: "4.0.64" },
      bones: [
        { name: "root" },
        { name: "p90", parent: "root", rotation: 90 },
        { name: "follower", parent: "p90", rotation: 0, length: 20 },
      ],
      slots: [{ name: "ps", bone: "root", attachment: "p" }],
      path: [{ name: "pc", bones: ["follower"], target: "ps", positionMode: "percent", spacingMode: "length", rotateMode: "tangent", position: 0.5, mixRotate: 1, mixX: 1, mixY: 1 }],
      skins: [{ name: "default", attachments: { ps: { p: { type: "path", vertexCount: 6, vertices: [0, 0, 0, 0, 0, 0, 100, 0, 100, 0, 100, 0], lengths: [100, 100] } } } }],
    }),
    "default",
  );
  const follower = pRotSkel.bones[pRotSkel.boneIndexByName.follower];
  assert.ok(Math.abs(follower.c) < 1e-4 && follower.a > 0.99, `follower under 90-deg parent must have world angle 0 deg, got a=${follower.a}, c=${follower.c}`);

  // 4) TransformConstraint world targetShearY wrapping at target.rotation = 120, shearY = 0
  const shSkel = createSkeletonInstance(
    parseSpine40Json({
      skeleton: { spine: "4.0.64" },
      bones: [
        { name: "root" },
        { name: "t120", parent: "root", rotation: 120, shearY: 0 },
        { name: "dst", parent: "root", shearY: 0 },
      ],
      transform: [{ name: "tr", bones: ["dst"], target: "t120", local: false, relative: false, mixRotate: 0, mixX: 0, mixY: 0, mixScaleX: 0, mixScaleY: 0, mixShearY: 0.5 }],
      slots: [],
      skins: [{ name: "default", attachments: {} }],
    }),
    "default",
  );
  assert.ok(Math.abs(shSkel.bones[shSkel.boneIndexByName.dst].shearY) < 1e-4);

  // 5) multiply blend mode preserves opaque destination alpha via srcAlpha = GL_ONE
  const mulBlend = resolvePremultipliedBlendFunc("multiply", true);
  assert.equal(mulBlend.srcAlpha, GL_ONE);
  assert.equal(mulBlend.dstAlpha, GL_ONE_MINUS_SRC_ALPHA);

  // 6) Binary .skel roundtrip preserves HANDCRAFTED_SPINE40_SKELETON_JSON ribbon_path.lengths losslessly
  const orig = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
  const rt = parseSpine40Binary(encodeSpine40Binary(orig));
  const origPath = orig.skins[0].attachments.path_guide_slot.ribbon_path;
  const rtPath = rt.skins[0].attachments.path_guide_slot.ribbon_path;
  assert.equal(origPath.type, "path");
  assert.equal(rtPath.type, "path");
  assert.equal(rtPath.vertexCount, 9);
  assert.deepEqual(rtPath.lengths, origPath.lengths);
});


