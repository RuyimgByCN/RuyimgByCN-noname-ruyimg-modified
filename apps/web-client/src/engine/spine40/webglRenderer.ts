/**
 * Clean-room Spine 4.0 WebGL batch builder, premultiplied alpha blend solver,
 * polygon clipping processor, and `dynamicSkin.js` / `saveSkinParams` import adapter (Decision 35).
 */

import { isSpine40Version, parseSpine40Binary, parseSpine40Json } from "./formatCodec";
import {
  computeAttachmentWorldVertices,
  parseHexColorRgba,
  resolveAttachment,
  type SkeletonInstance,
} from "./poseSolver";
import type {
  BlendMode,
  DynamicSkinParams,
  MeshAttachment,
  RegionAttachment,
  SkeletonData,
  TextureAtlas,
} from "./types";

// Standard WebGL blend factor constants
export const GL_ZERO = 0;
export const GL_ONE = 1;
export const GL_SRC_COLOR = 768;
export const GL_ONE_MINUS_SRC_COLOR = 769;
export const GL_SRC_ALPHA = 770;
export const GL_ONE_MINUS_SRC_ALPHA = 771;
export const GL_DST_ALPHA = 772;
export const GL_ONE_MINUS_DST_ALPHA = 773;
export const GL_DST_COLOR = 774;

export interface WebGLBlendFactors {
  srcRgb: number;
  dstRgb: number;
  srcAlpha: number;
  dstAlpha: number;
}

export function resolvePremultipliedBlendFunc(
  blend: BlendMode,
  premultipliedAlpha: boolean,
): WebGLBlendFactors {
  switch (blend) {
    case "normal":
      return {
        srcRgb: premultipliedAlpha ? GL_ONE : GL_SRC_ALPHA,
        dstRgb: GL_ONE_MINUS_SRC_ALPHA,
        srcAlpha: premultipliedAlpha ? GL_ONE : GL_SRC_ALPHA,
        dstAlpha: GL_ONE_MINUS_SRC_ALPHA,
      };
    case "additive":
      return {
        srcRgb: premultipliedAlpha ? GL_ONE : GL_SRC_ALPHA,
        dstRgb: GL_ONE,
        srcAlpha: GL_ZERO,
        dstAlpha: GL_ONE,
      };
    case "multiply":
      return {
        srcRgb: GL_DST_COLOR,
        dstRgb: GL_ONE_MINUS_SRC_ALPHA,
        srcAlpha: GL_ONE_MINUS_SRC_ALPHA,
        dstAlpha: GL_ONE_MINUS_SRC_ALPHA,
      };
    case "screen":
      return {
        srcRgb: GL_ONE,
        dstRgb: GL_ONE_MINUS_SRC_COLOR,
        srcAlpha: GL_ONE,
        dstAlpha: GL_ONE_MINUS_SRC_COLOR,
      };
  }
}

export interface SpineRenderBatch {
  page: string;
  blend: BlendMode;
  premultipliedAlpha: boolean;
  blendFactors: WebGLBlendFactors;
  /** Interleaved `[x, y, u, v, r, g, b, a]` per vertex (8 floats per vertex). */
  vertices: Float32Array;
  indices: Uint16Array;
  slotNames: string[];
  clipped: boolean;
}

function computePolygonBoundingBox(poly: number[]): { minX: number; minY: number; maxX: number; maxY: number } {
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  for (let i = 0; i + 1 < poly.length; i += 2) {
    minX = Math.min(minX, poly[i]);
    minY = Math.min(minY, poly[i + 1]);
    maxX = Math.max(maxX, poly[i]);
    maxY = Math.max(maxY, poly[i + 1]);
  }
  return { minX, minY, maxX, maxY };
}

export function buildBatchGeometry(
  skeleton: SkeletonInstance,
  atlas?: TextureAtlas,
  premultipliedAlpha = true,
  hiddenSlots?: Set<string>,
): SpineRenderBatch[] {
  const batches: SpineRenderBatch[] = [];
  let activeClipBox: { minX: number; minY: number; maxX: number; maxY: number } | null = null;
  let activeClipEndSlot: string | null = null;

  let currentPage = "default";
  let currentBlend: BlendMode = "normal";
  let currentClipped = false;
  let currentVerts: number[] = [];
  let currentIndices: number[] = [];
  let currentSlots: string[] = [];

  const flushBatch = () => {
    if (currentVerts.length === 0 || currentIndices.length === 0) return;
    batches.push({
      page: currentPage,
      blend: currentBlend,
      premultipliedAlpha,
      blendFactors: resolvePremultipliedBlendFunc(currentBlend, premultipliedAlpha),
      vertices: new Float32Array(currentVerts),
      indices: new Uint16Array(currentIndices),
      slotNames: [...currentSlots],
      clipped: currentClipped,
    });
    currentVerts = [];
    currentIndices = [];
    currentSlots = [];
  };

  for (const slotIdx of skeleton.drawOrder) {
    const slot = skeleton.slots[slotIdx];
    if (!slot) continue;
    if (hiddenSlots?.has(slot.data.name)) {
      if (activeClipEndSlot === slot.data.name) {
        activeClipBox = null;
        activeClipEndSlot = null;
      }
      continue;
    }

    const att = resolveAttachment(skeleton, slotIdx);
    if (!att) {
      if (activeClipEndSlot === slot.data.name) {
        activeClipBox = null;
        activeClipEndSlot = null;
      }
      continue;
    }

    if (att.type === "clipping") {
      const clipPoly = computeAttachmentWorldVertices(skeleton, slotIdx, att);
      if (clipPoly.length >= 6) {
        activeClipBox = computePolygonBoundingBox(clipPoly);
        activeClipEndSlot = att.end;
      }
      continue;
    }

    if (att.type !== "region" && att.type !== "mesh") {
      if (activeClipEndSlot === slot.data.name) {
        activeClipBox = null;
        activeClipEndSlot = null;
      }
      continue;
    }

    const regionInfo = atlas?.regions[att.path] ?? atlas?.regions[att.name];
    const pageName = regionInfo?.page ?? "default";
    const isClipped = activeClipBox !== null;

    if (
      currentVerts.length > 0 &&
      (pageName !== currentPage || slot.blend !== currentBlend || isClipped !== currentClipped)
    ) {
      flushBatch();
    }
    currentPage = pageName;
    currentBlend = slot.blend;
    currentClipped = isClipped;
    currentSlots.push(slot.data.name);

    const worldVerts = computeAttachmentWorldVertices(skeleton, slotIdx, att);
    const attColor = parseHexColorRgba(att.color);
    const rawAlpha = slot.color[3] * attColor[3];
    const r = slot.color[0] * attColor[0] * (premultipliedAlpha ? rawAlpha : 1);
    const g = slot.color[1] * attColor[1] * (premultipliedAlpha ? rawAlpha : 1);
    const b = slot.color[2] * attColor[2] * (premultipliedAlpha ? rawAlpha : 1);
    const a = rawAlpha;

    const baseVertexIndex = currentVerts.length >> 3;

    if (att.type === "region") {
      const reg = att as RegionAttachment;
      const u0 = regionInfo?.u ?? 0;
      const v0 = regionInfo?.v ?? 0;
      const u1 = regionInfo?.u2 ?? 1;
      const v1 = regionInfo?.v2 ?? 1;
      const uvs: Array<[number, number]> = [
        [u0, v1],
        [u1, v1],
        [u1, v0],
        [u0, v0],
      ];
      for (let i = 0; i < 4 && i * 2 + 1 < worldVerts.length; i++) {
        let vx = worldVerts[i * 2];
        let vy = worldVerts[i * 2 + 1];
        if (activeClipBox) {
          vx = Math.max(activeClipBox.minX, Math.min(activeClipBox.maxX, vx));
          vy = Math.max(activeClipBox.minY, Math.min(activeClipBox.maxY, vy));
        }
        currentVerts.push(vx, vy, uvs[i][0], uvs[i][1], r, g, b, a);
      }
      if (reg.width > 0 && reg.height > 0) {
        currentIndices.push(
          baseVertexIndex,
          baseVertexIndex + 1,
          baseVertexIndex + 2,
          baseVertexIndex + 2,
          baseVertexIndex + 3,
          baseVertexIndex,
        );
      }
    } else if (att.type === "mesh") {
      const mesh = att as MeshAttachment;
      const u0 = regionInfo?.u ?? 0;
      const v0 = regionInfo?.v ?? 0;
      const du = (regionInfo?.u2 ?? 1) - u0;
      const dv = (regionInfo?.v2 ?? 1) - v0;
      const vCount = worldVerts.length >> 1;
      for (let i = 0; i < vCount; i++) {
        let vx = worldVerts[i * 2];
        let vy = worldVerts[i * 2 + 1];
        if (activeClipBox) {
          vx = Math.max(activeClipBox.minX, Math.min(activeClipBox.maxX, vx));
          vy = Math.max(activeClipBox.minY, Math.min(activeClipBox.maxY, vy));
        }
        const u = u0 + (mesh.uvs[i * 2] ?? 0) * du;
        const v = v0 + (mesh.uvs[i * 2 + 1] ?? 0) * dv;
        currentVerts.push(vx, vy, u, v, r, g, b, a);
      }
      for (const triIdx of mesh.triangles) {
        currentIndices.push(baseVertexIndex + triIdx);
      }
    }

    if (activeClipEndSlot === slot.data.name) {
      activeClipBox = null;
      activeClipEndSlot = null;
    }
  }

  flushBatch();
  return batches;
}

export function renderSpineBatchesToCanvas2D(
  ctx: CanvasRenderingContext2D,
  skeleton: SkeletonInstance,
  batches: SpineRenderBatch[],
): { drawCallCount: number; triangleCount: number } {
  const width = ctx.canvas?.width ?? 220;
  const height = ctx.canvas?.height ?? 140;
  ctx.clearRect(0, 0, width, height);
  ctx.save();
  ctx.translate(width * 0.5, height * 0.78);
  ctx.scale(1, -1);

  let triangleCount = 0;
  for (const batch of batches) {
    const verts = batch.vertices;
    const indices = batch.indices;
    for (let i = 0; i + 2 < indices.length; i += 3) {
      const i0 = indices[i] * 8;
      const i1 = indices[i + 1] * 8;
      const i2 = indices[i + 2] * 8;
      const r = Math.round((verts[i0 + 4] ?? 0.9) * 255);
      const g = Math.round((verts[i0 + 5] ?? 0.75) * 255);
      const b = Math.round((verts[i0 + 6] ?? 0.35) * 255);
      const a = Math.max(0.15, Math.min(1, verts[i0 + 7] ?? 0.85));

      ctx.beginPath();
      ctx.moveTo(verts[i0], verts[i0 + 1]);
      ctx.lineTo(verts[i1], verts[i1 + 1]);
      ctx.lineTo(verts[i2], verts[i2 + 1]);
      ctx.closePath();
      ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${(a * 0.38).toFixed(2)})`;
      ctx.fill();
      ctx.strokeStyle = `rgba(${r}, ${g}, ${b}, ${a.toFixed(2)})`;
      ctx.lineWidth = 1.2;
      ctx.stroke();
      triangleCount++;
    }
  }

  for (const bone of skeleton.bones) {
    const tipX = bone.worldX + bone.a * bone.data.length;
    const tipY = bone.worldY + bone.c * bone.data.length;
    ctx.beginPath();
    ctx.moveTo(bone.worldX, bone.worldY);
    ctx.lineTo(tipX, tipY);
    ctx.strokeStyle = "#f5d08a";
    ctx.lineWidth = 1.8;
    ctx.stroke();
    ctx.beginPath();
    ctx.arc(bone.worldX, bone.worldY, 2.2, 0, Math.PI * 2);
    ctx.fillStyle = "#7dd3fc";
    ctx.fill();
  }

  ctx.restore();
  return { drawCallCount: batches.length, triangleCount };
}

export interface DynamicSkinResolution {
  status: "ready-spine40" | "static-portrait-fallback";
  reason: string;
  normalizedParams: {
    generalId: string;
    skinName: string;
    x: number;
    y: number;
    scale: number;
    angle: number;
    speed: number;
    action: string;
    background: string | null;
    alpha: number;
    flipX: boolean;
    clipSlots: string[];
    hideSlots: string[];
  };
  skeletonData: SkeletonData | null;
}

/**
 * Resolves a player-imported dynamic skin entry using upstream `dynamicSkin.js` / `saveSkinParams`
 * parameter conventions. If no asset is imported or if the asset is not Spine 4.0
 * (e.g., 3.5, 3.7, 3.8, 4.1, 4.2), cleanly falls back to static portrait mode.
 */
export function resolveDynamicSkinImport(
  params: Partial<DynamicSkinParams> | null | undefined,
  rawSkeletonAsset?: unknown,
): DynamicSkinResolution {
  const xVal = Array.isArray(params?.x) ? Number(params?.x[0] ?? 0) : Number(params?.x ?? 0);
  const yVal = Array.isArray(params?.y) ? Number(params?.y[0] ?? 0) : Number(params?.y ?? 0);
  const normalizedParams = {
    generalId: typeof params?.generalId === "string" && params.generalId ? params.generalId : "unknown",
    skinName: typeof params?.skinName === "string" && params.skinName ? params.skinName : "默认动皮",
    x: Number.isFinite(xVal) ? xVal : 0,
    y: Number.isFinite(yVal) ? yVal : 0,
    scale: typeof params?.scale === "number" && Number.isFinite(params.scale) && params.scale > 0 ? params.scale : 1,
    angle: typeof params?.angle === "number" && Number.isFinite(params.angle) ? params.angle : 0,
    speed: typeof params?.speed === "number" && Number.isFinite(params.speed) && params.speed > 0 ? params.speed : 1,
    action: typeof params?.action === "string" && params.action ? params.action : "DaiJi",
    background: typeof params?.background === "string" ? params.background : null,
    alpha: typeof params?.alpha === "number" && Number.isFinite(params.alpha) ? Math.max(0, Math.min(1, params.alpha)) : 1,
    flipX: Boolean(params?.flipX),
    clipSlots: Array.isArray(params?.clipSlots) ? params.clipSlots.map(String) : [],
    hideSlots: Array.isArray(params?.hideSlots) ? params.hideSlots.map(String) : [],
  };

  if (params?.version && !isSpine40Version(params.version)) {
    return {
      status: "static-portrait-fallback",
      reason: `NON_SPINE_40_VERSION:${params.version} (仅支持 Spine 4.0，已按规则基准回退静态立绘)`,
      normalizedParams,
      skeletonData: null,
    };
  }

  if (!rawSkeletonAsset) {
    return {
      status: "static-portrait-fallback",
      reason: "NO_DYNAMIC_SKIN_ASSET_IMPORTED (产品不附带动皮资源，未导入玩家自有 4.0 资源时回退静态立绘)",
      normalizedParams,
      skeletonData: null,
    };
  }

  try {
    const skeletonData =
      rawSkeletonAsset instanceof Uint8Array
        ? parseSpine40Binary(rawSkeletonAsset)
        : parseSpine40Json(rawSkeletonAsset);
    return {
      status: "ready-spine40",
      reason: `SPINE_40_LOADED:${skeletonData.header.spine}`,
      normalizedParams,
      skeletonData,
    };
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    return {
      status: "static-portrait-fallback",
      reason: `SPINE_40_PARSE_FALLBACK:${msg}`,
      normalizedParams,
      skeletonData: null,
    };
  }
}
