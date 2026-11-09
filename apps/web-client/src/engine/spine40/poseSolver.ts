/**
 * Clean-room Spine 4.0 pose solver, timeline evaluator, multi-track animation mixer,
 * IK/transform/path constraint solver, and attachment world vertex calculator (Decision 35).
 */

import type {
  Attachment,
  BlendMode,
  BoneData,
  CurveDef,
  IkConstraintData,
  MixBlend,
  PathAttachment,
  PathConstraintData,
  SkeletonData,
  SlotData,
  TransformConstraintData,
} from "./types";

const DEG_TO_RAD = Math.PI / 180;
const RAD_TO_DEG = 180 / Math.PI;

export interface BoneState {
  data: BoneData;
  parentIndex: number;
  x: number;
  y: number;
  rotation: number;
  scaleX: number;
  scaleY: number;
  shearX: number;
  shearY: number;
  // 2x3 world affine matrix: [a, b, worldX; c, d, worldY]
  a: number;
  b: number;
  c: number;
  d: number;
  worldX: number;
  worldY: number;
}

export interface SlotState {
  data: SlotData;
  boneIndex: number;
  color: [number, number, number, number]; // r, g, b, a in [0, 1]
  attachmentName: string | null;
  blend: BlendMode;
  deform: number[];
}

export interface IkConstraintState {
  data: IkConstraintData;
  mix: number;
  softness: number;
  bendPositive: boolean;
  compress: boolean;
  stretch: boolean;
}

export interface TransformConstraintState {
  data: TransformConstraintData;
  mixRotate: number;
  mixX: number;
  mixY: number;
  mixScaleX: number;
  mixScaleY: number;
  mixShearY: number;
}

export interface PathConstraintState {
  data: PathConstraintData;
  position: number;
  spacing: number;
  mixRotate: number;
  mixX: number;
  mixY: number;
}

export interface SkeletonInstance {
  data: SkeletonData;
  skinName: string;
  bones: BoneState[];
  boneIndexByName: Record<string, number>;
  slots: SlotState[];
  slotIndexByName: Record<string, number>;
  drawOrder: number[];
  ikConstraints: IkConstraintState[];
  transformConstraints: TransformConstraintState[];
  pathConstraints: PathConstraintState[];
  x: number;
  y: number;
  scaleX: number;
  scaleY: number;
}

export interface AnimationTrack {
  trackIndex: number;
  animationName: string;
  time: number;
  loop: boolean;
  alpha: number;
  mixBlend: MixBlend;
  timeScale: number;
}

export function parseHexColorRgba(hex: string): [number, number, number, number] {
  const clean = hex.replace(/^#/, "").trim();
  if (clean.length !== 6 && clean.length !== 8) {
    return [1, 1, 1, 1];
  }
  const r = parseInt(clean.slice(0, 2), 16) / 255;
  const g = parseInt(clean.slice(2, 4), 16) / 255;
  const b = parseInt(clean.slice(4, 6), 16) / 255;
  const a = clean.length === 8 ? parseInt(clean.slice(6, 8), 16) / 255 : 1;
  return [
    Number.isFinite(r) ? r : 1,
    Number.isFinite(g) ? g : 1,
    Number.isFinite(b) ? b : 1,
    Number.isFinite(a) ? a : 1,
  ];
}

export function createSkeletonInstance(data: SkeletonData, skinName = "default"): SkeletonInstance {
  const boneIndexByName: Record<string, number> = {};
  data.bones.forEach((b, idx) => {
    boneIndexByName[b.name] = idx;
  });

  const bones: BoneState[] = data.bones.map((b) => ({
    data: b,
    parentIndex: b.parent && b.parent in boneIndexByName ? boneIndexByName[b.parent] : -1,
    x: b.x,
    y: b.y,
    rotation: b.rotation,
    scaleX: b.scaleX,
    scaleY: b.scaleY,
    shearX: b.shearX,
    shearY: b.shearY,
    a: 1,
    b: 0,
    c: 0,
    d: 1,
    worldX: b.x,
    worldY: b.y,
  }));

  const slotIndexByName: Record<string, number> = {};
  const slots: SlotState[] = data.slots.map((s, idx) => {
    slotIndexByName[s.name] = idx;
    return {
      data: s,
      boneIndex: boneIndexByName[s.bone] ?? 0,
      color: parseHexColorRgba(s.color),
      attachmentName: s.attachment,
      blend: s.blend,
      deform: [],
    };
  });

  const drawOrder = slots.map((_, idx) => idx);

  const ikConstraints: IkConstraintState[] = data.ik.map((c) => ({
    data: c,
    mix: c.mix,
    softness: c.softness,
    bendPositive: c.bendPositive,
    compress: c.compress,
    stretch: c.stretch,
  }));

  const transformConstraints: TransformConstraintState[] = data.transform.map((c) => ({
    data: c,
    mixRotate: c.mixRotate,
    mixX: c.mixX,
    mixY: c.mixY,
    mixScaleX: c.mixScaleX,
    mixScaleY: c.mixScaleY,
    mixShearY: c.mixShearY,
  }));

  const pathConstraints: PathConstraintState[] = data.path.map((c) => ({
    data: c,
    position: c.position,
    spacing: c.spacing,
    mixRotate: c.mixRotate,
    mixX: c.mixX,
    mixY: c.mixY,
  }));

  const instance: SkeletonInstance = {
    data,
    skinName,
    bones,
    boneIndexByName,
    slots,
    slotIndexByName,
    drawOrder,
    ikConstraints,
    transformConstraints,
    pathConstraints,
    x: 0,
    y: 0,
    scaleX: 1,
    scaleY: 1,
  };

  updateSkeletonWorldTransform(instance);
  return instance;
}

export function resetSkeletonToSetupPose(skeleton: SkeletonInstance): void {
  for (const b of skeleton.bones) {
    b.x = b.data.x;
    b.y = b.data.y;
    b.rotation = b.data.rotation;
    b.scaleX = b.data.scaleX;
    b.scaleY = b.data.scaleY;
    b.shearX = b.data.shearX;
    b.shearY = b.data.shearY;
  }
  skeleton.slots.forEach((s, idx) => {
    s.color = parseHexColorRgba(s.data.color);
    s.attachmentName = s.data.attachment;
    s.blend = s.data.blend;
    s.deform = [];
    skeleton.drawOrder[idx] = idx;
  });
  for (const ik of skeleton.ikConstraints) {
    ik.mix = ik.data.mix;
    ik.softness = ik.data.softness;
    ik.bendPositive = ik.data.bendPositive;
    ik.compress = ik.data.compress;
    ik.stretch = ik.data.stretch;
  }
  for (const tr of skeleton.transformConstraints) {
    tr.mixRotate = tr.data.mixRotate;
    tr.mixX = tr.data.mixX;
    tr.mixY = tr.data.mixY;
    tr.mixScaleX = tr.data.mixScaleX;
    tr.mixScaleY = tr.data.mixScaleY;
    tr.mixShearY = tr.data.mixShearY;
  }
  for (const pc of skeleton.pathConstraints) {
    pc.position = pc.data.position;
    pc.spacing = pc.data.spacing;
    pc.mixRotate = pc.data.mixRotate;
    pc.mixX = pc.data.mixX;
    pc.mixY = pc.data.mixY;
  }
}

export function resolveAttachment(
  skeleton: SkeletonInstance,
  slotIndex: number,
  attachmentName?: string | null,
): Attachment | null {
  const slot = skeleton.slots[slotIndex];
  if (!slot) return null;
  const targetName = attachmentName === undefined ? slot.attachmentName : attachmentName;
  if (!targetName) return null;

  const slotName = slot.data.name;
  if (skeleton.skinName !== "default") {
    const activeSkin = skeleton.data.skins.find((sk) => sk.name === skeleton.skinName);
    const found = activeSkin?.attachments[slotName]?.[targetName];
    if (found) return found;
  }
  const defaultSkin = skeleton.data.skins.find((sk) => sk.name === "default") ?? skeleton.data.skins[0];
  return defaultSkin?.attachments[slotName]?.[targetName] ?? null;
}

export function computeBoneWorldTransform(bone: BoneState, parent: BoneState | null, skel: SkeletonInstance): void {
  const sx = skel.scaleX;
  const sy = skel.scaleY;
  const radX = (bone.rotation + bone.shearX) * DEG_TO_RAD;
  const radY = (bone.rotation + 90 + bone.shearY) * DEG_TO_RAD;
  const la = Math.cos(radX) * bone.scaleX;
  const lb = Math.cos(radY) * bone.scaleY;
  const lc = Math.sin(radX) * bone.scaleX;
  const ld = Math.sin(radY) * bone.scaleY;

  if (!parent) {
    bone.a = la * sx;
    bone.b = lb * sx;
    bone.c = lc * sy;
    bone.d = ld * sy;
    bone.worldX = bone.x * sx + skel.x;
    bone.worldY = bone.y * sy + skel.y;
    return;
  }

  bone.worldX = parent.a * bone.x + parent.b * bone.y + parent.worldX;
  bone.worldY = parent.c * bone.x + parent.d * bone.y + parent.worldY;

  switch (bone.data.transform) {
    case "normal": {
      bone.a = parent.a * la + parent.b * lc;
      bone.b = parent.a * lb + parent.b * ld;
      bone.c = parent.c * la + parent.d * lc;
      bone.d = parent.c * lb + parent.d * ld;
      return;
    }
    case "onlyTranslation": {
      bone.a = la * sx;
      bone.b = lb * sx;
      bone.c = lc * sy;
      bone.d = ld * sy;
      return;
    }
    case "noRotationOrReflection": {
      let s = parent.a * parent.a + parent.c * parent.c;
      let prx: number;
      if (s > 0.0001) {
        s = Math.abs(parent.a * parent.d - parent.b * parent.c) / s;
        const pa = parent.a / sx;
        const pc = parent.c / sy;
        const pb = pc * s;
        const pd = pa * s;
        prx = Math.atan2(pc, pa) * RAD_TO_DEG;
        bone.a = pa * la + pb * lc;
        bone.b = pa * lb + pb * ld;
        bone.c = pc * la + pd * lc;
        bone.d = pc * lb + pd * ld;
      } else {
        prx = 90 - Math.atan2(parent.d, parent.b) * RAD_TO_DEG;
        bone.a = la;
        bone.b = lb;
        bone.c = lc;
        bone.d = ld;
      }
      const rot = -prx * DEG_TO_RAD;
      const cos = Math.cos(rot);
      const sin = Math.sin(rot);
      const na = bone.a * cos - bone.c * sin;
      const nb = bone.b * cos - bone.d * sin;
      const nc = bone.a * sin + bone.c * cos;
      const nd = bone.b * sin + bone.d * cos;
      bone.a = na;
      bone.b = nb;
      bone.c = nc;
      bone.d = nd;
      return;
    }
    case "noScale":
    case "noScaleOrReflection": {
      const parentAngle = Math.atan2(parent.c, parent.a);
      const cos = Math.cos(parentAngle);
      const sin = Math.sin(parentAngle);
      const det = parent.a * parent.d - parent.b * parent.c;
      const reflect = bone.data.transform === "noScale" && det < 0 ? -1 : 1;
      bone.a = (cos * la - sin * lc) * sx;
      bone.b = (cos * lb - sin * ld) * sx * reflect;
      bone.c = (sin * la + cos * lc) * sy;
      bone.d = (sin * lb + cos * ld) * sy * reflect;
      return;
    }
  }
}

function normalizeDeg(deg: number): number {
  let d = deg % 360;
  if (d > 180) d -= 360;
  if (d < -180) d += 360;
  return d;
}

export function applyIkConstraint(skel: SkeletonInstance, ik: IkConstraintState): void {
  if (ik.mix <= 0) return;
  const targetIdx = skel.boneIndexByName[ik.data.target];
  if (targetIdx === undefined) return;
  const target = skel.bones[targetIdx];

  if (ik.data.bones.length === 1) {
    const bIdx = skel.boneIndexByName[ik.data.bones[0]];
    if (bIdx === undefined) return;
    const bone = skel.bones[bIdx];
    const parent = bone.parentIndex >= 0 ? skel.bones[bone.parentIndex] : null;
    computeBoneWorldTransform(bone, parent, skel);
    const dx = target.worldX - bone.worldX;
    const dy = target.worldY - bone.worldY;
    const parentRot = parent ? Math.atan2(parent.c, parent.a) * RAD_TO_DEG : 0;
    const desiredWorldDeg = Math.atan2(dy, dx) * RAD_TO_DEG;
    const delta = normalizeDeg(desiredWorldDeg - parentRot - bone.rotation);
    bone.rotation += delta * ik.mix;
    if (ik.stretch && bone.data.length > 0.0001) {
      const dist = Math.hypot(dx, dy);
      const targetScale = dist / bone.data.length;
      bone.scaleX += (targetScale - bone.scaleX) * ik.mix;
    }
    computeBoneWorldTransform(bone, parent, skel);
    return;
  }

  if (ik.data.bones.length >= 2) {
    const pIdx = skel.boneIndexByName[ik.data.bones[0]];
    const cIdx = skel.boneIndexByName[ik.data.bones[1]];
    if (pIdx === undefined || cIdx === undefined) return;
    const parentBone = skel.bones[pIdx];
    const childBone = skel.bones[cIdx];
    const grandParent = parentBone.parentIndex >= 0 ? skel.bones[parentBone.parentIndex] : null;
    computeBoneWorldTransform(parentBone, grandParent, skel);
    computeBoneWorldTransform(childBone, parentBone, skel);

    const l1 = Math.max(0.001, parentBone.data.length * Math.abs(parentBone.scaleX));
    const l2 = Math.max(0.001, childBone.data.length * Math.abs(childBone.scaleX));
    const dx = target.worldX - parentBone.worldX;
    const dy = target.worldY - parentBone.worldY;
    const dist = Math.hypot(dx, dy);
    const clampedDist = Math.min(l1 + l2 - 0.0001, Math.max(Math.abs(l1 - l2) + 0.0001, dist));

    const cosChild = Math.max(-1, Math.min(1, (clampedDist * clampedDist - l1 * l1 - l2 * l2) / (2 * l1 * l2)));
    const bendSign = ik.bendPositive ? 1 : -1;
    const childAngleRad = bendSign * Math.acos(cosChild);
    const targetAngleRad = Math.atan2(dy, dx);
    const parentOffsetRad = Math.atan2(l2 * Math.sin(childAngleRad), l1 + l2 * Math.cos(childAngleRad));
    const grandParentRotRad = grandParent ? Math.atan2(grandParent.c, grandParent.a) : 0;

    const solvedParentDeg = (targetAngleRad - parentOffsetRad - grandParentRotRad) * RAD_TO_DEG;
    const solvedChildDeg = childAngleRad * RAD_TO_DEG;

    parentBone.rotation += normalizeDeg(solvedParentDeg - parentBone.rotation) * ik.mix;
    childBone.rotation += normalizeDeg(solvedChildDeg - childBone.rotation) * ik.mix;
    computeBoneWorldTransform(parentBone, grandParent, skel);
    computeBoneWorldTransform(childBone, parentBone, skel);
  }
}

export function applyTransformConstraint(skel: SkeletonInstance, tr: TransformConstraintState): void {
  const targetIdx = skel.boneIndexByName[tr.data.target];
  if (targetIdx === undefined) return;
  const target = skel.bones[targetIdx];

  for (const bName of tr.data.bones) {
    const bIdx = skel.boneIndexByName[bName];
    if (bIdx === undefined) continue;
    const bone = skel.bones[bIdx];
    const parent = bone.parentIndex >= 0 ? skel.bones[bone.parentIndex] : null;

    if (tr.mixRotate > 0) {
      const targetRot = target.rotation + tr.data.offsetRotation;
      bone.rotation += normalizeDeg(targetRot - bone.rotation) * tr.mixRotate;
    }
    if (tr.mixX > 0) {
      const tx = (tr.data.relative ? bone.data.x + target.x : target.x) + tr.data.offsetX;
      bone.x += (tx - bone.x) * tr.mixX;
    }
    if (tr.mixY > 0) {
      const ty = (tr.data.relative ? bone.data.y + target.y : target.y) + tr.data.offsetY;
      bone.y += (ty - bone.y) * tr.mixY;
    }
    if (tr.mixScaleX > 0) {
      const sx = (tr.data.relative ? bone.data.scaleX * target.scaleX : target.scaleX) + tr.data.offsetScaleX;
      bone.scaleX += (sx - bone.scaleX) * tr.mixScaleX;
    }
    if (tr.mixScaleY > 0) {
      const sy = (tr.data.relative ? bone.data.scaleY * target.scaleY : target.scaleY) + tr.data.offsetScaleY;
      bone.scaleY += (sy - bone.scaleY) * tr.mixScaleY;
    }
    if (tr.mixShearY > 0) {
      const shy = (tr.data.relative ? bone.data.shearY + target.shearY : target.shearY) + tr.data.offsetShearY;
      bone.shearY += (shy - bone.shearY) * tr.mixShearY;
    }
    computeBoneWorldTransform(bone, parent, skel);
  }
}

function samplePathPolyline(verts: number[], closed: boolean, ratio: number): { x: number; y: number; tangentDeg: number } {
  const pts: Array<[number, number]> = [];
  for (let i = 0; i + 1 < verts.length; i += 2) {
    pts.push([verts[i], verts[i + 1]]);
  }
  if (pts.length === 0) return { x: 0, y: 0, tangentDeg: 0 };
  if (pts.length === 1) return { x: pts[0][0], y: pts[0][1], tangentDeg: 0 };
  if (closed) pts.push(pts[0]);

  const segLengths: number[] = [];
  let totalLen = 0;
  for (let i = 0; i < pts.length - 1; i++) {
    const d = Math.hypot(pts[i + 1][0] - pts[i][0], pts[i + 1][1] - pts[i][1]);
    segLengths.push(d);
    totalLen += d;
  }
  if (totalLen <= 0.0001) {
    return { x: pts[0][0], y: pts[0][1], tangentDeg: 0 };
  }

  const clampedRatio = closed ? ((ratio % 1) + 1) % 1 : Math.max(0, Math.min(1, ratio));
  let targetDist = clampedRatio * totalLen;
  for (let i = 0; i < segLengths.length; i++) {
    const segLen = segLengths[i];
    if (targetDist <= segLen || i === segLengths.length - 1) {
      const t = segLen > 0.0001 ? Math.max(0, Math.min(1, targetDist / segLen)) : 0;
      const x = pts[i][0] + (pts[i + 1][0] - pts[i][0]) * t;
      const y = pts[i][1] + (pts[i + 1][1] - pts[i][1]) * t;
      const tangentDeg = Math.atan2(pts[i + 1][1] - pts[i][1], pts[i + 1][0] - pts[i][0]) * RAD_TO_DEG;
      return { x, y, tangentDeg };
    }
    targetDist -= segLen;
  }
  const last = pts[pts.length - 1];
  return { x: last[0], y: last[1], tangentDeg: 0 };
}

export function applyPathConstraint(skel: SkeletonInstance, pc: PathConstraintState): void {
  const slotIdx = skel.slotIndexByName[pc.data.target];
  if (slotIdx === undefined) return;
  const att = resolveAttachment(skel, slotIdx);
  if (!att || att.type !== "path") return;
  const pathAtt = att as PathAttachment;
  const pathVerts = computeAttachmentWorldVertices(skel, slotIdx, pathAtt);
  if (pathVerts.length < 4) return;

  const totalLen = pathAtt.lengths[pathAtt.lengths.length - 1] || 100;
  let posRatio = pc.data.positionMode === "fixed" ? pc.position / Math.max(1, totalLen) : pc.position;

  for (const bName of pc.data.bones) {
    const bIdx = skel.boneIndexByName[bName];
    if (bIdx === undefined) continue;
    const bone = skel.bones[bIdx];
    const parent = bone.parentIndex >= 0 ? skel.bones[bone.parentIndex] : null;
    const sample = samplePathPolyline(pathVerts, pathAtt.closed, posRatio);

    if (pc.mixX > 0) {
      bone.worldX += (sample.x - bone.worldX) * pc.mixX;
    }
    if (pc.mixY > 0) {
      bone.worldY += (sample.y - bone.worldY) * pc.mixY;
    }
    if (pc.mixRotate > 0) {
      const desiredDeg = sample.tangentDeg + pc.data.offsetRotation;
      bone.rotation += normalizeDeg(desiredDeg - bone.rotation) * pc.mixRotate;
      computeBoneWorldTransform(bone, parent, skel);
      bone.worldX = sample.x;
      bone.worldY = sample.y;
    }

    const step =
      pc.data.spacingMode === "percent"
        ? pc.spacing
        : pc.data.spacingMode === "fixed"
          ? pc.spacing / Math.max(1, totalLen)
          : Math.max(1, bone.data.length + pc.spacing) / Math.max(1, totalLen);
    posRatio += step;
  }
}

export function updateSkeletonWorldTransform(skel: SkeletonInstance): void {
  for (const bone of skel.bones) {
    const parent = bone.parentIndex >= 0 ? skel.bones[bone.parentIndex] : null;
    computeBoneWorldTransform(bone, parent, skel);
  }

  const ordered: Array<{ order: number; run: () => void }> = [];
  for (const ik of skel.ikConstraints) {
    ordered.push({ order: ik.data.order, run: () => applyIkConstraint(skel, ik) });
  }
  for (const tr of skel.transformConstraints) {
    ordered.push({ order: tr.data.order, run: () => applyTransformConstraint(skel, tr) });
  }
  for (const pc of skel.pathConstraints) {
    ordered.push({ order: pc.data.order, run: () => applyPathConstraint(skel, pc) });
  }
  ordered.sort((a, b) => a.order - b.order);
  for (const item of ordered) {
    item.run();
  }
}

/**
 * Evaluates a Spine 4.0 cubic Bezier curve segment `(t0, v0) -> (cx1, cy1) -> (cx2, cy2) -> (t1, v1)`
 * at `time in [t0, t1]` by solving `B_x(s) = time` via bisection + Newton-Raphson.
 */
export function evaluateCubicBezier(
  t0: number,
  v0: number,
  cx1: number,
  cy1: number,
  cx2: number,
  cy2: number,
  t1: number,
  v1: number,
  time: number,
): number {
  if (time <= t0) return v0;
  if (time >= t1) return v1;

  const bx = (s: number): number => {
    const u = 1 - s;
    return u * u * u * t0 + 3 * u * u * s * cx1 + 3 * u * s * s * cx2 + s * s * s * t1;
  };
  const by = (s: number): number => {
    const u = 1 - s;
    return u * u * u * v0 + 3 * u * u * s * cy1 + 3 * u * s * s * cy2 + s * s * s * v1;
  };

  let lo = 0;
  let hi = 1;
  let s = (time - t0) / Math.max(1e-6, t1 - t0);
  for (let i = 0; i < 16; i++) {
    const x = bx(s);
    const err = x - time;
    if (Math.abs(err) < 1e-6) break;
    if (err < 0) lo = s;
    else hi = s;
    s = 0.5 * (lo + hi);
  }
  return by(s);
}

function interpolateValue(
  t0: number,
  v0: number,
  t1: number,
  v1: number,
  time: number,
  curve: CurveDef | undefined,
  channelOffset = 0,
): number {
  if (time <= t0 || t1 <= t0) return v0;
  if (time >= t1) return v1;
  if (curve === "stepped") return v0;
  if (Array.isArray(curve) && curve.length >= channelOffset + 4) {
    return evaluateCubicBezier(
      t0,
      v0,
      curve[channelOffset],
      curve[channelOffset + 1],
      curve[channelOffset + 2],
      curve[channelOffset + 3],
      t1,
      v1,
      time,
    );
  }
  const ratio = (time - t0) / (t1 - t0);
  return v0 + (v1 - v0) * ratio;
}

function sampleScalarTimeline(kfs: Array<{ time: number; value: number; curve?: CurveDef }>, time: number): number {
  if (kfs.length === 0) return 0;
  if (time <= kfs[0].time) return kfs[0].value;
  if (time >= kfs[kfs.length - 1].time) return kfs[kfs.length - 1].value;
  for (let i = 0; i < kfs.length - 1; i++) {
    if (time >= kfs[i].time && time <= kfs[i + 1].time) {
      return interpolateValue(kfs[i].time, kfs[i].value, kfs[i + 1].time, kfs[i + 1].value, time, kfs[i].curve, 0);
    }
  }
  return kfs[kfs.length - 1].value;
}

function samplePairTimeline(
  kfs: Array<{ time: number; x: number; y: number; curve?: CurveDef }>,
  time: number,
): [number, number] {
  if (kfs.length === 0) return [0, 0];
  if (time <= kfs[0].time) return [kfs[0].x, kfs[0].y];
  if (time >= kfs[kfs.length - 1].time) return [kfs[kfs.length - 1].x, kfs[kfs.length - 1].y];
  for (let i = 0; i < kfs.length - 1; i++) {
    if (time >= kfs[i].time && time <= kfs[i + 1].time) {
      const c = kfs[i].curve;
      const yOffset = Array.isArray(c) && c.length >= 8 ? 4 : 0;
      const x = interpolateValue(kfs[i].time, kfs[i].x, kfs[i + 1].time, kfs[i + 1].x, time, c, 0);
      const y = interpolateValue(kfs[i].time, kfs[i].y, kfs[i + 1].time, kfs[i + 1].y, time, c, yOffset);
      return [x, y];
    }
  }
  return [kfs[kfs.length - 1].x, kfs[kfs.length - 1].y];
}

export function getAnimationDuration(skelData: SkeletonData, animationName: string): number {
  const anim = skelData.animations[animationName];
  if (!anim) return 0;
  let maxTime = 0;
  const checkArr = (arr?: Array<{ time: number }>) => {
    if (arr && arr.length > 0) {
      maxTime = Math.max(maxTime, arr[arr.length - 1].time);
    }
  };
  for (const grp of Object.values(anim.bones)) {
    checkArr(grp.rotate);
    checkArr(grp.translate);
    checkArr(grp.scale);
    checkArr(grp.shear);
  }
  for (const grp of Object.values(anim.slots)) {
    checkArr(grp.attachment);
    checkArr(grp.rgba);
  }
  for (const kfs of Object.values(anim.ik)) checkArr(kfs);
  for (const kfs of Object.values(anim.transform)) checkArr(kfs);
  for (const grp of Object.values(anim.path)) {
    checkArr(grp.position);
    checkArr(grp.spacing);
    checkArr(grp.mix);
  }
  checkArr(anim.drawOrder);
  return maxTime;
}

export class CleanRoomAnimationState {
  public readonly tracks: Map<number, AnimationTrack> = new Map();

  public setAnimation(
    trackIndex: number,
    animationName: string,
    loop = true,
    alpha = 1,
    mixBlend: MixBlend = "replace",
  ): AnimationTrack {
    const track: AnimationTrack = {
      trackIndex,
      animationName,
      time: 0,
      loop,
      alpha: Math.max(0, Math.min(1, alpha)),
      mixBlend,
      timeScale: 1,
    };
    this.tracks.set(trackIndex, track);
    return track;
  }

  public update(deltaSeconds: number): void {
    if (!Number.isFinite(deltaSeconds) || deltaSeconds <= 0) return;
    for (const track of this.tracks.values()) {
      track.time += deltaSeconds * track.timeScale;
    }
  }

  public apply(skeleton: SkeletonInstance): void {
    const sortedTracks = [...this.tracks.values()].sort((a, b) => a.trackIndex - b.trackIndex);
    for (const track of sortedTracks) {
      const anim = skeleton.data.animations[track.animationName];
      if (!anim) continue;
      const duration = getAnimationDuration(skeleton.data, track.animationName);
      const sampleTime =
        track.loop && duration > 0 ? ((track.time % duration) + duration) % duration : Math.min(track.time, duration);
      const alpha = track.alpha;

      for (const [boneName, grp] of Object.entries(anim.bones)) {
        const bIdx = skeleton.boneIndexByName[boneName];
        if (bIdx === undefined) continue;
        const bone = skeleton.bones[bIdx];

        if (grp.rotate && grp.rotate.length > 0) {
          const val = sampleScalarTimeline(grp.rotate, sampleTime);
          const target = track.mixBlend === "add" ? bone.rotation + val : bone.data.rotation + val;
          bone.rotation += (target - bone.rotation) * alpha;
        }
        if (grp.translate && grp.translate.length > 0) {
          const [dx, dy] = samplePairTimeline(grp.translate, sampleTime);
          const tx = track.mixBlend === "add" ? bone.x + dx : bone.data.x + dx;
          const ty = track.mixBlend === "add" ? bone.y + dy : bone.data.y + dy;
          bone.x += (tx - bone.x) * alpha;
          bone.y += (ty - bone.y) * alpha;
        }
        if (grp.scale && grp.scale.length > 0) {
          const [sx, sy] = samplePairTimeline(grp.scale, sampleTime);
          const tsx = track.mixBlend === "add" ? bone.scaleX * sx : bone.data.scaleX * sx;
          const tsy = track.mixBlend === "add" ? bone.scaleY * sy : bone.data.scaleY * sy;
          bone.scaleX += (tsx - bone.scaleX) * alpha;
          bone.scaleY += (tsy - bone.scaleY) * alpha;
        }
        if (grp.shear && grp.shear.length > 0) {
          const [shx, shy] = samplePairTimeline(grp.shear, sampleTime);
          const tshx = track.mixBlend === "add" ? bone.shearX + shx : bone.data.shearX + shx;
          const tshy = track.mixBlend === "add" ? bone.shearY + shy : bone.data.shearY + shy;
          bone.shearX += (tshx - bone.shearX) * alpha;
          bone.shearY += (tshy - bone.shearY) * alpha;
        }
      }

      for (const [slotName, grp] of Object.entries(anim.slots)) {
        const sIdx = skeleton.slotIndexByName[slotName];
        if (sIdx === undefined) continue;
        const slot = skeleton.slots[sIdx];
        if (grp.attachment && grp.attachment.length > 0) {
          let chosen = slot.data.attachment;
          for (const kf of grp.attachment) {
            if (sampleTime >= kf.time) chosen = kf.name;
          }
          slot.attachmentName = chosen;
        }
        if (grp.rgba && grp.rgba.length > 0) {
          let c0 = parseHexColorRgba(grp.rgba[0].color);
          for (let i = 0; i < grp.rgba.length; i++) {
            if (sampleTime >= grp.rgba[i].time) {
              c0 = parseHexColorRgba(grp.rgba[i].color);
              if (i + 1 < grp.rgba.length && sampleTime < grp.rgba[i + 1].time && grp.rgba[i].curve !== "stepped") {
                const c1 = parseHexColorRgba(grp.rgba[i + 1].color);
                const tRatio = (sampleTime - grp.rgba[i].time) / Math.max(1e-6, grp.rgba[i + 1].time - grp.rgba[i].time);
                c0 = [
                  c0[0] + (c1[0] - c0[0]) * tRatio,
                  c0[1] + (c1[1] - c0[1]) * tRatio,
                  c0[2] + (c1[2] - c0[2]) * tRatio,
                  c0[3] + (c1[3] - c0[3]) * tRatio,
                ];
              }
            }
          }
          slot.color = [
            slot.color[0] + (c0[0] - slot.color[0]) * alpha,
            slot.color[1] + (c0[1] - slot.color[1]) * alpha,
            slot.color[2] + (c0[2] - slot.color[2]) * alpha,
            slot.color[3] + (c0[3] - slot.color[3]) * alpha,
          ];
        }
      }

      for (const [ikName, kfs] of Object.entries(anim.ik)) {
        const ikState = skeleton.ikConstraints.find((c) => c.data.name === ikName);
        if (!ikState || kfs.length === 0) continue;
        const mixVal = sampleScalarTimeline(
          kfs.map((k) => ({ time: k.time, value: k.mix, curve: k.curve })),
          sampleTime,
        );
        ikState.mix += (mixVal - ikState.mix) * alpha;
      }

      if (anim.drawOrder.length > 0) {
        let activeOrder = anim.drawOrder[0];
        for (const kf of anim.drawOrder) {
          if (sampleTime >= kf.time) activeOrder = kf;
        }
        const baseOrder = skeleton.slots.map((_, idx) => idx);
        for (const off of activeOrder.offsets) {
          const origIdx = skeleton.slotIndexByName[off.slot];
          if (origIdx !== undefined) {
            const targetIdx = Math.max(0, Math.min(baseOrder.length - 1, origIdx + off.offset));
            const removed = baseOrder.splice(baseOrder.indexOf(origIdx), 1)[0];
            baseOrder.splice(targetIdx, 0, removed);
          }
        }
        skeleton.drawOrder = baseOrder;
      }
    }

    updateSkeletonWorldTransform(skeleton);
  }
}

export function computeAttachmentWorldVertices(
  skel: SkeletonInstance,
  slotIndex: number,
  attachment?: Attachment | null,
): number[] {
  const slot = skel.slots[slotIndex];
  if (!slot) return [];
  const att = attachment ?? resolveAttachment(skel, slotIndex);
  if (!att) return [];
  const bone = skel.bones[slot.boneIndex];

  if (att.type === "region") {
    const hw = (att.width * att.scaleX) / 2;
    const hh = (att.height * att.scaleY) / 2;
    const rad = att.rotation * DEG_TO_RAD;
    const cos = Math.cos(rad);
    const sin = Math.sin(rad);
    const corners: Array<[number, number]> = [
      [-hw, -hh],
      [hw, -hh],
      [hw, hh],
      [-hw, hh],
    ];
    const out: number[] = [];
    for (const [lx, ly] of corners) {
      const rx = lx * cos - ly * sin + att.x;
      const ry = lx * sin + ly * cos + att.y;
      out.push(bone.a * rx + bone.b * ry + bone.worldX, bone.c * rx + bone.d * ry + bone.worldY);
    }
    return out;
  }

  if (att.type === "point") {
    return [bone.a * att.x + bone.b * att.y + bone.worldX, bone.c * att.x + bone.d * att.y + bone.worldY];
  }

  const deform = slot.deform;
  if (!att.weighted) {
    const out: number[] = [];
    for (let i = 0; i + 1 < att.vertices.length; i += 2) {
      const vx = att.vertices[i] + (deform[i] ?? 0);
      const vy = att.vertices[i + 1] + (deform[i + 1] ?? 0);
      out.push(bone.a * vx + bone.b * vy + bone.worldX, bone.c * vx + bone.d * vy + bone.worldY);
    }
    return out;
  }

  // Weighted vertices encoding: [boneCount, boneIdx, bindX, bindY, weight, ...]
  const out: number[] = [];
  const verts = att.vertices;
  let vIdx = 0;
  let deformIdx = 0;
  while (vIdx < verts.length) {
    const boneCount = verts[vIdx++];
    let wx = 0;
    let wy = 0;
    for (let b = 0; b < boneCount && vIdx + 3 < verts.length; b++) {
      const boneIdx = verts[vIdx++];
      const bindX = verts[vIdx++] + (deform[deformIdx++] ?? 0);
      const bindY = verts[vIdx++] + (deform[deformIdx++] ?? 0);
      const weight = verts[vIdx++];
      const wBone = skel.bones[boneIdx] ?? bone;
      wx += (wBone.a * bindX + wBone.b * bindY + wBone.worldX) * weight;
      wy += (wBone.c * bindX + wBone.d * bindY + wBone.worldY) * weight;
    }
    out.push(wx, wy);
  }
  return out;
}

export function containsPointInBoundingBox(worldPolygonVerts: number[], x: number, y: number): boolean {
  const n = worldPolygonVerts.length >> 1;
  if (n < 3) return false;
  let inside = false;
  for (let i = 0, j = n - 1; i < n; j = i++) {
    const xi = worldPolygonVerts[i * 2];
    const yi = worldPolygonVerts[i * 2 + 1];
    const xj = worldPolygonVerts[j * 2];
    const yj = worldPolygonVerts[j * 2 + 1];
    const intersect = yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi + 1e-12) + xi;
    if (intersect) inside = !inside;
  }
  return inside;
}
