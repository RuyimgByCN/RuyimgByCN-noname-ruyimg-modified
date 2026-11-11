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

type Vertex8 = [number, number, number, number, number, number, number, number];

function ensureCcwPolygonPoints(pts: Array<[number, number]>): Array<[number, number]> {
  const out = pts.map(([x, y]): [number, number] => [x, y]);
  if (out.length < 3) return out;
  let signedArea2 = 0;
  for (let i = 0; i < out.length; i++) {
    const [x0, y0] = out[i];
    const [x1, y1] = out[(i + 1) % out.length];
    signedArea2 += x0 * y1 - x1 * y0;
  }
  if (signedArea2 < 0) out.reverse();
  return out;
}

function normalizeCcwPolygon(poly: number[]): Array<[number, number]> {
  const pts: Array<[number, number]> = [];
  for (let i = 0; i + 1 < poly.length; i += 2) {
    pts.push([poly[i], poly[i + 1]]);
  }
  return ensureCcwPolygonPoints(pts);
}

function cross2d(a: [number, number], b: [number, number], c: [number, number]): number {
  return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
}

function isConvexCcwPolygon(pts: Array<[number, number]>): boolean {
  if (pts.length <= 3) return true;
  for (let i = 0; i < pts.length; i++) {
    const a = pts[i];
    const b = pts[(i + 1) % pts.length];
    const c = pts[(i + 2) % pts.length];
    if (cross2d(a, b, c) < -1e-6) return false;
  }
  return true;
}

function pointInTriangleCcw(
  p: [number, number],
  a: [number, number],
  b: [number, number],
  c: [number, number],
): boolean {
  const c1 = cross2d(a, b, p);
  const c2 = cross2d(b, c, p);
  const c3 = cross2d(c, a, p);
  return c1 >= -1e-7 && c2 >= -1e-7 && c3 >= -1e-7;
}

/**
 * Clean-room ear-clipping triangulation for arbitrary simple CCW polygons
 * (both convex and concave).
 */
export function triangulateSimplePolygon(
  rawPolygon: Array<[number, number]>,
): Array<[[number, number], [number, number], [number, number]]> {
  const pts = ensureCcwPolygonPoints(rawPolygon);
  if (pts.length < 3) return [];
  if (pts.length === 3) return [[pts[0], pts[1], pts[2]]];

  const indices: number[] = pts.map((_, idx) => idx);
  const ears: Array<[[number, number], [number, number], [number, number]]> = [];
  let guard = indices.length * indices.length + 8;

  while (indices.length > 3 && guard-- > 0) {
    let earFound = false;
    const len = indices.length;
    for (let i = 0; i < len; i++) {
      const iPrev = indices[(i + len - 1) % len];
      const iCurr = indices[i];
      const iNext = indices[(i + 1) % len];
      const a = pts[iPrev];
      const b = pts[iCurr];
      const c = pts[iNext];
      if (cross2d(a, b, c) <= 1e-7) continue;

      let containsReflex = false;
      for (let j = 0; j < len; j++) {
        const idx = indices[j];
        if (idx === iPrev || idx === iCurr || idx === iNext) continue;
        const p = pts[idx];
        if (
          (Math.abs(p[0] - a[0]) < 1e-7 && Math.abs(p[1] - a[1]) < 1e-7) ||
          (Math.abs(p[0] - b[0]) < 1e-7 && Math.abs(p[1] - b[1]) < 1e-7) ||
          (Math.abs(p[0] - c[0]) < 1e-7 && Math.abs(p[1] - c[1]) < 1e-7)
        ) {
          continue;
        }
        if (pointInTriangleCcw(p, a, b, c)) {
          containsReflex = true;
          break;
        }
      }
      if (!containsReflex) {
        ears.push([a, b, c]);
        indices.splice(i, 1);
        earFound = true;
        break;
      }
    }
    if (!earFound) {
      // Degenerate collinear/self-touching fallback: clip sharpest positive corner
      const i0 = indices[0];
      const i1 = indices[1];
      const i2 = indices[2];
      ears.push([pts[i0], pts[i1], pts[i2]]);
      indices.splice(1, 1);
    }
  }

  if (indices.length === 3) {
    ears.push([pts[indices[0]], pts[indices[1]], pts[indices[2]]]);
  }
  return ears;
}

/**
 * Clips a single `[x, y, u, v, r, g, b, a]` triangle against a CONVEX CCW clipping polygon
 * using Sutherland-Hodgman half-plane clipping with exact linear interpolation of UVs and RGBA.
 */
export function clipTriangleAgainstPolygon(
  v0: Vertex8,
  v1: Vertex8,
  v2: Vertex8,
  clipPolygon: Array<[number, number]>,
): Vertex8[] {
  if (clipPolygon.length < 3) return [v0, v1, v2];
  let subject: Vertex8[] = [v0, v1, v2];
  const eps = -1e-6;

  const lerpVert = (a: Vertex8, b: Vertex8, t: number): Vertex8 => [
    a[0] + (b[0] - a[0]) * t,
    a[1] + (b[1] - a[1]) * t,
    a[2] + (b[2] - a[2]) * t,
    a[3] + (b[3] - a[3]) * t,
    a[4] + (b[4] - a[4]) * t,
    a[5] + (b[5] - a[5]) * t,
    a[6] + (b[6] - a[6]) * t,
    a[7] + (b[7] - a[7]) * t,
  ];

  for (let i = 0; i < clipPolygon.length; i++) {
    const [cx0, cy0] = clipPolygon[i];
    const [cx1, cy1] = clipPolygon[(i + 1) % clipPolygon.length];
    const ex = cx1 - cx0;
    const ey = cy1 - cy0;
    const input = subject;
    subject = [];
    if (input.length === 0) break;

    let prev = input[input.length - 1];
    let prevCross = ex * (prev[1] - cy0) - ey * (prev[0] - cx0);
    let prevInside = prevCross >= eps;

    for (const curr of input) {
      const currCross = ex * (curr[1] - cy0) - ey * (curr[0] - cx0);
      const currInside = currCross >= eps;
      if (currInside) {
        if (!prevInside) {
          const denom = prevCross - currCross;
          const t = Math.abs(denom) > 1e-9 ? Math.max(0, Math.min(1, prevCross / denom)) : 0.5;
          subject.push(lerpVert(prev, curr, t));
        }
        subject.push(curr);
      } else if (prevInside) {
        const denom = prevCross - currCross;
        const t = Math.abs(denom) > 1e-9 ? Math.max(0, Math.min(1, prevCross / denom)) : 0.5;
        subject.push(lerpVert(prev, curr, t));
      }
      prev = curr;
      prevCross = currCross;
      prevInside = currInside;
    }
  }

  return subject.length >= 3 ? subject : [];
}

/**
 * Clips indexed `[x, y, u, v, r, g, b, a]` batch triangles against a convex OR concave
 * `ClippingAttachment` polygon by decomposing concave polygons via ear-clipping first.
 */
export function clipBatchTrianglesToPolygon(
  verts: Vertex8[],
  tris: number[],
  rawClipPolygon: Array<[number, number]>,
): Array<[Vertex8, Vertex8, Vertex8]> {
  const ccwClip = ensureCcwPolygonPoints(rawClipPolygon);
  if (ccwClip.length < 3) return [];
  const convexRegions: Array<Array<[number, number]>> = isConvexCcwPolygon(ccwClip)
    ? [ccwClip]
    : triangulateSimplePolygon(ccwClip);

  const out: Array<[Vertex8, Vertex8, Vertex8]> = [];
  for (let t = 0; t + 2 < tris.length; t += 3) {
    const p0 = verts[tris[t]];
    const p1 = verts[tris[t + 1]];
    const p2 = verts[tris[t + 2]];
    if (!p0 || !p1 || !p2) continue;
    for (const region of convexRegions) {
      const poly = clipTriangleAgainstPolygon(p0, p1, p2, region);
      if (poly.length < 3) continue;
      for (let k = 1; k + 1 < poly.length; k++) {
        out.push([poly[0], poly[k], poly[k + 1]]);
      }
    }
  }
  return out;
}

export function buildBatchGeometry(
  skeleton: SkeletonInstance,
  atlas?: TextureAtlas,
  premultipliedAlpha = true,
  hiddenSlots?: Set<string>,
): SpineRenderBatch[] {
  const batches: SpineRenderBatch[] = [];
  let activeClipPoly: Array<[number, number]> | null = null;
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

  const appendGeometry = (verts: Vertex8[], tris: number[]) => {
    if (!activeClipPoly) {
      const baseIdx = currentVerts.length >> 3;
      for (const v of verts) currentVerts.push(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]);
      for (const ti of tris) currentIndices.push(baseIdx + ti);
      return;
    }
    const clippedTris = clipBatchTrianglesToPolygon(verts, tris, activeClipPoly);
    for (const [a, b, c] of clippedTris) {
      const baseIdx = currentVerts.length >> 3;
      currentVerts.push(
        a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7],
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
      );
      currentIndices.push(baseIdx, baseIdx + 1, baseIdx + 2);
    }
  };

  for (const slotIdx of skeleton.drawOrder) {
    const slot = skeleton.slots[slotIdx];
    if (!slot) continue;
    if (hiddenSlots?.has(slot.data.name)) {
      if (activeClipEndSlot === slot.data.name) {
        activeClipPoly = null;
        activeClipEndSlot = null;
      }
      continue;
    }

    const att = resolveAttachment(skeleton, slotIdx);
    if (!att) {
      if (activeClipEndSlot === slot.data.name) {
        activeClipPoly = null;
        activeClipEndSlot = null;
      }
      continue;
    }

    if (att.type === "clipping") {
      const clipPoly = computeAttachmentWorldVertices(skeleton, slotIdx, att);
      if (clipPoly.length >= 6) {
        activeClipPoly = normalizeCcwPolygon(clipPoly);
        activeClipEndSlot = att.end;
      }
      continue;
    }

    if (att.type !== "region" && att.type !== "mesh") {
      if (activeClipEndSlot === slot.data.name) {
        activeClipPoly = null;
        activeClipEndSlot = null;
      }
      continue;
    }

    const regionInfo = atlas?.regions[att.path] ?? atlas?.regions[att.name];
    const pageName = regionInfo?.page ?? "default";
    const isClipped = activeClipPoly !== null;

    if (
      currentVerts.length > 0 &&
      (pageName !== currentPage || slot.blend !== currentBlend || isClipped !== currentClipped)
    ) {
      flushBatch();
    }
    currentPage = pageName;
    currentBlend = slot.blend;
    currentClipped = isClipped;

    const worldVerts = computeAttachmentWorldVertices(skeleton, slotIdx, att);
    const attColor = parseHexColorRgba(att.color);
    const rawAlpha = slot.color[3] * attColor[3];
    const r = slot.color[0] * attColor[0] * (premultipliedAlpha ? rawAlpha : 1);
    const g = slot.color[1] * attColor[1] * (premultipliedAlpha ? rawAlpha : 1);
    const b = slot.color[2] * attColor[2] * (premultipliedAlpha ? rawAlpha : 1);
    const a = rawAlpha;

    const beforeVertLen = currentVerts.length;

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
      const slotVerts: Vertex8[] = [];
      for (let i = 0; i < 4 && i * 2 + 1 < worldVerts.length; i++) {
        slotVerts.push([worldVerts[i * 2], worldVerts[i * 2 + 1], uvs[i][0], uvs[i][1], r, g, b, a]);
      }
      if (reg.width > 0 && reg.height > 0 && slotVerts.length === 4) {
        appendGeometry(slotVerts, [0, 1, 2, 2, 3, 0]);
      }
    } else if (att.type === "mesh") {
      const mesh = att as MeshAttachment;
      const u0 = regionInfo?.u ?? 0;
      const v0 = regionInfo?.v ?? 0;
      const du = (regionInfo?.u2 ?? 1) - u0;
      const dv = (regionInfo?.v2 ?? 1) - v0;
      const vCount = worldVerts.length >> 1;
      const slotVerts: Vertex8[] = [];
      for (let i = 0; i < vCount; i++) {
        const u = u0 + (mesh.uvs[i * 2] ?? 0) * du;
        const v = v0 + (mesh.uvs[i * 2 + 1] ?? 0) * dv;
        slotVerts.push([worldVerts[i * 2], worldVerts[i * 2 + 1], u, v, r, g, b, a]);
      }
      appendGeometry(slotVerts, mesh.triangles);
    }

    if (currentVerts.length > beforeVertLen) {
      currentSlots.push(slot.data.name);
    }

    if (activeClipEndSlot === slot.data.name) {
      activeClipPoly = null;
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

/**
 * Minimal WebGL context interface (compatible with `WebGLRenderingContext`,
 * `WebGL2RenderingContext`, and headless test harnesses) for executing Spine 4.0 batches.
 */
export interface MinimalWebGLContext {
  readonly ARRAY_BUFFER: number;
  readonly ELEMENT_ARRAY_BUFFER: number;
  readonly DYNAMIC_DRAW: number;
  readonly FLOAT: number;
  readonly TRIANGLES: number;
  readonly UNSIGNED_SHORT: number;
  readonly BLEND: number;
  readonly COLOR_BUFFER_BIT: number;
  readonly VERTEX_SHADER: number;
  readonly FRAGMENT_SHADER: number;
  viewport(x: number, y: number, width: number, height: number): void;
  clearColor(r: number, g: number, b: number, a: number): void;
  clear(mask: number): void;
  enable(cap: number): void;
  blendFuncSeparate(srcRGB: number, dstRGB: number, srcAlpha: number, dstAlpha: number): void;
  createBuffer(): unknown;
  bindBuffer(target: number, buffer: unknown): void;
  bufferData(target: number, data: ArrayBufferView, usage: number): void;
  drawElements(mode: number, count: number, type: number, offset: number): void;
}

interface FullWebGLContext extends MinimalWebGLContext {
  readonly COMPILE_STATUS: number;
  readonly LINK_STATUS: number;
  readonly TEXTURE_2D: number;
  readonly TEXTURE0: number;
  readonly RGBA: number;
  readonly UNSIGNED_BYTE: number;
  readonly TEXTURE_MIN_FILTER: number;
  readonly TEXTURE_MAG_FILTER: number;
  readonly TEXTURE_WRAP_S: number;
  readonly TEXTURE_WRAP_T: number;
  readonly LINEAR: number;
  readonly CLAMP_TO_EDGE: number;
  createShader(type: number): unknown;
  shaderSource(shader: unknown, source: string): void;
  compileShader(shader: unknown): void;
  getShaderParameter(shader: unknown, pname: number): unknown;
  createProgram(): unknown;
  attachShader(program: unknown, shader: unknown): void;
  linkProgram(program: unknown): void;
  getProgramParameter(program: unknown, pname: number): unknown;
  useProgram(program: unknown): void;
  getAttribLocation(program: unknown, name: string): number;
  getUniformLocation(program: unknown, name: string): unknown;
  enableVertexAttribArray(index: number): void;
  vertexAttribPointer(index: number, size: number, type: number, normalized: boolean, stride: number, offset: number): void;
  uniform2f(location: unknown, x: number, y: number): void;
  uniform1i(location: unknown, x: number): void;
  createTexture(): unknown;
  activeTexture(texture: number): void;
  bindTexture(target: number, texture: unknown): void;
  texImage2D(
    target: number,
    level: number,
    internalformat: number,
    width: number,
    height: number,
    border: number,
    format: number,
    type: number,
    pixels: ArrayBufferView | null,
  ): void;
  texParameteri(target: number, pname: number, param: number): void;
}

interface CachedSpineWebGLPipeline {
  program: unknown;
  vbo: unknown;
  ibo: unknown;
  whiteTex: unknown;
  locPos: number;
  locUv: number;
  locColor: number;
  uScale: unknown;
  uOrigin: unknown;
  uSampler: unknown;
}

const SPINE_VS_SRC = `
attribute vec2 a_position;
attribute vec2 a_texCoord;
attribute vec4 a_color;
uniform vec2 u_origin;
uniform vec2 u_scale;
varying vec2 v_texCoord;
varying vec4 v_color;
void main() {
  vec2 clip = (a_position + u_origin) * u_scale;
  gl_Position = vec4(clip.x, clip.y, 0.0, 1.0);
  v_texCoord = a_texCoord;
  v_color = a_color;
}
`;

const SPINE_FS_SRC = `
precision mediump float;
uniform sampler2D u_sampler;
varying vec2 v_texCoord;
varying vec4 v_color;
void main() {
  gl_FragColor = texture2D(u_sampler, v_texCoord) * v_color;
}
`;

const cachedPipelines = new WeakMap<object, CachedSpineWebGLPipeline>();

function isFullWebGL(gl: MinimalWebGLContext): gl is FullWebGLContext {
  const cand = gl as Partial<FullWebGLContext>;
  return (
    typeof cand.createShader === "function" &&
    typeof cand.createProgram === "function" &&
    typeof cand.vertexAttribPointer === "function" &&
    typeof cand.createTexture === "function"
  );
}

function getOrCreateWebGLPipeline(gl: FullWebGLContext): CachedSpineWebGLPipeline | null {
  const existing = cachedPipelines.get(gl as unknown as object);
  if (existing) return existing;

  const vs = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vs, SPINE_VS_SRC);
  gl.compileShader(vs);
  if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) return null;

  const fs = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fs, SPINE_FS_SRC);
  gl.compileShader(fs);
  if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) return null;

  const program = gl.createProgram();
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) return null;

  const vbo = gl.createBuffer();
  const ibo = gl.createBuffer();
  const whiteTex = gl.createTexture();
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, whiteTex);
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA,
    1,
    1,
    0,
    gl.RGBA,
    gl.UNSIGNED_BYTE,
    new Uint8Array([255, 255, 255, 255]),
  );
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

  const pipeline: CachedSpineWebGLPipeline = {
    program,
    vbo,
    ibo,
    whiteTex,
    locPos: gl.getAttribLocation(program, "a_position"),
    locUv: gl.getAttribLocation(program, "a_texCoord"),
    locColor: gl.getAttribLocation(program, "a_color"),
    uScale: gl.getUniformLocation(program, "u_scale"),
    uOrigin: gl.getUniformLocation(program, "u_origin"),
    uSampler: gl.getUniformLocation(program, "u_sampler"),
  };
  cachedPipelines.set(gl as unknown as object, pipeline);
  return pipeline;
}

/**
 * Uploads RGBA8888 image pixels into a WebGL 2D texture for a Spine `.atlas` page.
 */
export function uploadRgbaTextureToWebGL(
  gl: MinimalWebGLContext,
  width: number,
  height: number,
  pixels: Uint8Array | Uint8ClampedArray,
): unknown | null {
  if (!isFullWebGL(gl) || width <= 0 || height <= 0) return null;
  const tex = gl.createTexture();
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, tex);
  const view =
    pixels instanceof Uint8Array
      ? pixels
      : new Uint8Array(pixels.buffer, pixels.byteOffset, pixels.byteLength);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, view);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  return tex;
}

/**
 * Executes Spine 4.0 batches on a WebGL context, compiling & binding the clean-room
 * GLSL shader program and vertex attributes when full WebGL APIs are available,
 * setting each batch's premultiplied or straight-alpha blend factors via
 * `gl.blendFuncSeparate`, uploading interleaved `[x, y, u, v, r, g, b, a]` vertices
 * and `Uint16Array` indices, and issuing `gl.drawElements(gl.TRIANGLES, ...)`.
 */
export function renderSpineBatchesToWebGL(
  gl: MinimalWebGLContext,
  batches: SpineRenderBatch[],
  viewportWidth = 220,
  viewportHeight = 140,
  pageTextures?: Record<string, unknown>,
): { drawCallCount: number; triangleCount: number; shaderProgramActive: boolean } {
  gl.viewport(0, 0, viewportWidth, viewportHeight);
  gl.clearColor(0.07, 0.1, 0.15, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT);
  gl.enable(gl.BLEND);

  const fullGl = isFullWebGL(gl) ? gl : null;
  const pipeline = fullGl ? getOrCreateWebGLPipeline(fullGl) : null;

  const vbo = pipeline ? pipeline.vbo : gl.createBuffer();
  const ibo = pipeline ? pipeline.ibo : gl.createBuffer();

  if (fullGl && pipeline) {
    fullGl.useProgram(pipeline.program);
    fullGl.uniform2f(pipeline.uOrigin, 0, -45);
    fullGl.uniform2f(
      pipeline.uScale,
      2.0 / Math.max(160, viewportWidth),
      2.0 / Math.max(140, viewportHeight),
    );
    fullGl.uniform1i(pipeline.uSampler, 0);
  }

  let drawCallCount = 0;
  let triangleCount = 0;
  const strideBytes = 8 * 4;

  for (const batch of batches) {
    if (batch.indices.length === 0 || batch.vertices.length === 0) continue;
    gl.blendFuncSeparate(
      batch.blendFactors.srcRgb,
      batch.blendFactors.dstRgb,
      batch.blendFactors.srcAlpha,
      batch.blendFactors.dstAlpha,
    );
    if (fullGl && pipeline) {
      fullGl.activeTexture(fullGl.TEXTURE0);
      const tex = pageTextures?.[batch.page] ?? pipeline.whiteTex;
      fullGl.bindTexture(fullGl.TEXTURE_2D, tex);
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.bufferData(gl.ARRAY_BUFFER, batch.vertices, gl.DYNAMIC_DRAW);
    if (fullGl && pipeline) {
      if (pipeline.locPos >= 0) {
        fullGl.enableVertexAttribArray(pipeline.locPos);
        fullGl.vertexAttribPointer(pipeline.locPos, 2, fullGl.FLOAT, false, strideBytes, 0);
      }
      if (pipeline.locUv >= 0) {
        fullGl.enableVertexAttribArray(pipeline.locUv);
        fullGl.vertexAttribPointer(pipeline.locUv, 2, fullGl.FLOAT, false, strideBytes, 8);
      }
      if (pipeline.locColor >= 0) {
        fullGl.enableVertexAttribArray(pipeline.locColor);
        fullGl.vertexAttribPointer(pipeline.locColor, 4, fullGl.FLOAT, false, strideBytes, 16);
      }
    }
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, ibo);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, batch.indices, gl.DYNAMIC_DRAW);
    gl.drawElements(gl.TRIANGLES, batch.indices.length, gl.UNSIGNED_SHORT, 0);
    drawCallCount += 1;
    triangleCount += Math.floor(batch.indices.length / 3);
  }

  return { drawCallCount, triangleCount, shaderProgramActive: Boolean(pipeline) };
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
    const isBinary = rawSkeletonAsset instanceof Uint8Array;
    const skeletonData = isBinary
      ? parseSpine40Binary(rawSkeletonAsset)
      : parseSpine40Json(rawSkeletonAsset);
    return {
      status: "ready-spine40",
      reason: `SPINE_40_LOADED:${skeletonData.header.spine}${isBinary ? " (二进制解码：未经真实资源验证)" : ""}`,
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
