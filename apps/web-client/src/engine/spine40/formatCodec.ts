/**
 * Clean-room Spine 4.0 JSON parser, binary `.skel` encoder/decoder, and `.atlas` parser (Decision 35).
 *
 * Uses only Web-standard `Uint8Array`, `DataView`, `TextEncoder`, and `TextDecoder`
 * (zero Node `Buffer` dependencies).
 */

import type {
  AnimationData,
  AtlasPage,
  AtlasRegion,
  Attachment,
  BlendMode,
  BoneData,
  BoneTimelineGroup,
  CurveDef,
  IkConstraintData,
  PathConstraintData,
  PositionMode,
  RotateMode,
  SkinData,
  SkeletonData,
  SlotData,
  SlotTimelineGroup,
  SpacingMode,
  TextureAtlas,
  TransformConstraintData,
  TransformMode,
} from "./types";

const BINARY_MAGIC = [0x53, 0x50, 0x34, 0x30] as const; // "SP40"

export function isSpine40Version(version: string | undefined | null): boolean {
  if (typeof version !== "string") return false;
  const trimmed = version.trim();
  return trimmed === "4.0" || trimmed.startsWith("4.0.");
}

function asNumber(v: unknown, fallback = 0): number {
  return typeof v === "number" && Number.isFinite(v) ? v : fallback;
}

function asString(v: unknown, fallback = ""): string {
  return typeof v === "string" ? v : fallback;
}

function asBoolean(v: unknown, fallback = false): boolean {
  return typeof v === "boolean" ? v : fallback;
}

function asNumberArray(v: unknown): number[] {
  if (!Array.isArray(v)) return [];
  return v.map((n) => asNumber(n, 0));
}

function asStringArray(v: unknown): string[] {
  if (!Array.isArray(v)) return [];
  return v.map((s) => asString(s));
}

function parseCurveDef(raw: unknown): CurveDef | undefined {
  if (raw === "stepped" || raw === "linear") return raw;
  if (Array.isArray(raw) && raw.length >= 4 && raw.every((x) => typeof x === "number" && Number.isFinite(x))) {
    return raw.map((x) => Number(x));
  }
  return undefined;
}

function parseAttachment(slotAttachmentName: string, raw: Record<string, unknown>): Attachment {
  const type = asString(raw.type, "region");
  const name = asString(raw.name, slotAttachmentName);
  const path = asString(raw.path, name);
  const color = asString(raw.color, "ffffffff").toLowerCase();

  switch (type) {
    case "region":
      return {
        type: "region",
        name,
        path,
        x: asNumber(raw.x, 0),
        y: asNumber(raw.y, 0),
        scaleX: asNumber(raw.scaleX, 1),
        scaleY: asNumber(raw.scaleY, 1),
        rotation: asNumber(raw.rotation, 0),
        width: asNumber(raw.width, 32),
        height: asNumber(raw.height, 32),
        color,
      };
    case "mesh":
    case "linkedmesh": {
      const uvs = asNumberArray(raw.uvs);
      const vertices = asNumberArray(raw.vertices);
      const weighted = vertices.length > uvs.length;
      return {
        type: "mesh",
        name,
        path,
        uvs,
        triangles: asNumberArray(raw.triangles),
        vertices,
        hull: asNumber(raw.hull, uvs.length >> 1),
        edges: asNumberArray(raw.edges),
        color,
        width: asNumber(raw.width, 32),
        height: asNumber(raw.height, 32),
        weighted,
      };
    }
    case "clipping": {
      const vertexCount = asNumber(raw.vertexCount, 0);
      const vertices = asNumberArray(raw.vertices);
      return {
        type: "clipping",
        name,
        end: typeof raw.end === "string" ? raw.end : null,
        vertexCount,
        vertices,
        color,
        weighted: vertices.length > vertexCount * 2,
      };
    }
    case "path": {
      const vertexCount = asNumber(raw.vertexCount, 0);
      const vertices = asNumberArray(raw.vertices);
      return {
        type: "path",
        name,
        closed: asBoolean(raw.closed, false),
        constantSpeed: asBoolean(raw.constantSpeed, true),
        vertexCount,
        vertices,
        lengths: asNumberArray(raw.lengths),
        color,
        weighted: vertices.length > vertexCount * 2,
      };
    }
    case "boundingbox": {
      const vertexCount = asNumber(raw.vertexCount, 0);
      const vertices = asNumberArray(raw.vertices);
      return {
        type: "boundingbox",
        name,
        vertexCount,
        vertices,
        color,
        weighted: vertices.length > vertexCount * 2,
      };
    }
    case "point":
      return {
        type: "point",
        name,
        x: asNumber(raw.x, 0),
        y: asNumber(raw.y, 0),
        rotation: asNumber(raw.rotation, 0),
        color,
      };
    default:
      throw new Error(`Unsupported Spine 4.0 attachment type: ${type}`);
  }
}

export function parseSpine40Json(input: unknown): SkeletonData {
  const obj = typeof input === "string" ? (JSON.parse(input) as Record<string, unknown>) : (input as Record<string, unknown>);
  if (!obj || typeof obj !== "object") {
    throw new Error("Invalid Spine 4.0 JSON root");
  }
  const skelHeader = (obj.skeleton ?? {}) as Record<string, unknown>;
  const spineVer = asString(skelHeader.spine, "");
  if (!isSpine40Version(spineVer)) {
    throw new Error(`UNSUPPORTED_SPINE_VERSION:${spineVer || "missing"} (clean-room player requires Spine 4.0)`);
  }

  const header = {
    hash: asString(skelHeader.hash, "cleanroom40"),
    spine: spineVer,
    x: asNumber(skelHeader.x, 0),
    y: asNumber(skelHeader.y, 0),
    width: asNumber(skelHeader.width, 256),
    height: asNumber(skelHeader.height, 256),
    fps: asNumber(skelHeader.fps, 30),
    images: typeof skelHeader.images === "string" ? skelHeader.images : undefined,
    audio: typeof skelHeader.audio === "string" ? skelHeader.audio : undefined,
  };

  const bonesRaw = Array.isArray(obj.bones) ? obj.bones : [];
  const bones: BoneData[] = bonesRaw.map((b: Record<string, unknown>) => ({
    name: asString(b.name),
    parent: typeof b.parent === "string" ? b.parent : null,
    length: asNumber(b.length, 0),
    transform: (asString(b.transform, "normal") as TransformMode),
    skin: asBoolean(b.skin, false),
    x: asNumber(b.x, 0),
    y: asNumber(b.y, 0),
    rotation: asNumber(b.rotation, 0),
    scaleX: asNumber(b.scaleX, 1),
    scaleY: asNumber(b.scaleY, 1),
    shearX: asNumber(b.shearX, 0),
    shearY: asNumber(b.shearY, 0),
  }));

  const slotsRaw = Array.isArray(obj.slots) ? obj.slots : [];
  const slots: SlotData[] = slotsRaw.map((s: Record<string, unknown>) => ({
    name: asString(s.name),
    bone: asString(s.bone),
    color: asString(s.color, "ffffffff").toLowerCase(),
    dark: typeof s.dark === "string" ? s.dark.toLowerCase() : null,
    attachment: typeof s.attachment === "string" ? s.attachment : null,
    blend: (asString(s.blend, "normal") as BlendMode),
  }));

  const skins: SkinData[] = [];
  if (Array.isArray(obj.skins)) {
    for (const sk of obj.skins as Array<Record<string, unknown>>) {
      const attachmentsMap: Record<string, Record<string, Attachment>> = {};
      const rawAtts = (sk.attachments ?? {}) as Record<string, Record<string, Record<string, unknown>>>;
      for (const [slotName, slotEntries] of Object.entries(rawAtts)) {
        attachmentsMap[slotName] = {};
        for (const [attKey, attVal] of Object.entries(slotEntries ?? {})) {
          attachmentsMap[slotName][attKey] = parseAttachment(attKey, attVal);
        }
      }
      skins.push({
        name: asString(sk.name, "default"),
        bones: asStringArray(sk.bones),
        ik: asStringArray(sk.ik),
        transform: asStringArray(sk.transform),
        path: asStringArray(sk.path),
        attachments: attachmentsMap,
      });
    }
  }

  const ikRaw = Array.isArray(obj.ik) ? obj.ik : [];
  const ik: IkConstraintData[] = ikRaw.map((c: Record<string, unknown>, idx: number) => ({
    name: asString(c.name),
    order: asNumber(c.order, idx),
    skin: asBoolean(c.skin, false),
    bones: asStringArray(c.bones),
    target: asString(c.target),
    mix: asNumber(c.mix, 1),
    softness: asNumber(c.softness, 0),
    bendPositive: asBoolean(c.bendPositive, true),
    compress: asBoolean(c.compress, false),
    stretch: asBoolean(c.stretch, false),
    uniform: asBoolean(c.uniform, false),
  }));

  const transformRaw = Array.isArray(obj.transform) ? obj.transform : [];
  const transform: TransformConstraintData[] = transformRaw.map((c: Record<string, unknown>, idx: number) => ({
    name: asString(c.name),
    order: asNumber(c.order, idx),
    skin: asBoolean(c.skin, false),
    bones: asStringArray(c.bones),
    target: asString(c.target),
    local: asBoolean(c.local, false),
    relative: asBoolean(c.relative, false),
    offsetRotation: asNumber(c.offsetRotation, 0),
    offsetX: asNumber(c.offsetX, 0),
    offsetY: asNumber(c.offsetY, 0),
    offsetScaleX: asNumber(c.offsetScaleX, 0),
    offsetScaleY: asNumber(c.offsetScaleY, 0),
    offsetShearY: asNumber(c.offsetShearY, 0),
    mixRotate: asNumber(c.mixRotate, 1),
    mixX: asNumber(c.mixX, 1),
    mixY: asNumber(c.mixY, asNumber(c.mixX, 1)),
    mixScaleX: asNumber(c.mixScaleX, 1),
    mixScaleY: asNumber(c.mixScaleY, asNumber(c.mixScaleX, 1)),
    mixShearY: asNumber(c.mixShearY, 1),
  }));

  const pathRaw = Array.isArray(obj.path) ? obj.path : [];
  const path: PathConstraintData[] = pathRaw.map((c: Record<string, unknown>, idx: number) => ({
    name: asString(c.name),
    order: asNumber(c.order, idx),
    skin: asBoolean(c.skin, false),
    bones: asStringArray(c.bones),
    target: asString(c.target),
    positionMode: (asString(c.positionMode, "percent") as PositionMode),
    spacingMode: (asString(c.spacingMode, "length") as SpacingMode),
    rotateMode: (asString(c.rotateMode, "tangent") as RotateMode),
    offsetRotation: asNumber(c.offsetRotation, 0),
    position: asNumber(c.position, 0),
    spacing: asNumber(c.spacing, 0),
    mixRotate: asNumber(c.mixRotate, 1),
    mixX: asNumber(c.mixX, 1),
    mixY: asNumber(c.mixY, asNumber(c.mixX, 1)),
  }));

  const animations: Record<string, AnimationData> = {};
  const animsRaw = (obj.animations ?? {}) as Record<string, Record<string, unknown>>;
  for (const [animName, animObj] of Object.entries(animsRaw)) {
    const bonesTl: Record<string, BoneTimelineGroup> = {};
    for (const [boneName, tlObj] of Object.entries((animObj.bones ?? {}) as Record<string, Record<string, unknown>>)) {
      const grp: BoneTimelineGroup = {};
      if (Array.isArray(tlObj.rotate)) {
        grp.rotate = tlObj.rotate.map((kf: Record<string, unknown>) => ({
          time: asNumber(kf.time, 0),
          value: asNumber(kf.value ?? kf.angle, 0),
          curve: parseCurveDef(kf.curve),
        }));
      }
      if (Array.isArray(tlObj.translate)) {
        grp.translate = tlObj.translate.map((kf: Record<string, unknown>) => ({
          time: asNumber(kf.time, 0),
          x: asNumber(kf.x, 0),
          y: asNumber(kf.y, 0),
          curve: parseCurveDef(kf.curve),
        }));
      }
      if (Array.isArray(tlObj.scale)) {
        grp.scale = tlObj.scale.map((kf: Record<string, unknown>) => ({
          time: asNumber(kf.time, 0),
          x: asNumber(kf.x, 1),
          y: asNumber(kf.y, 1),
          curve: parseCurveDef(kf.curve),
        }));
      }
      if (Array.isArray(tlObj.shear)) {
        grp.shear = tlObj.shear.map((kf: Record<string, unknown>) => ({
          time: asNumber(kf.time, 0),
          x: asNumber(kf.x, 0),
          y: asNumber(kf.y, 0),
          curve: parseCurveDef(kf.curve),
        }));
      }
      bonesTl[boneName] = grp;
    }

    const slotsTl: Record<string, SlotTimelineGroup> = {};
    for (const [slotName, tlObj] of Object.entries((animObj.slots ?? {}) as Record<string, Record<string, unknown>>)) {
      const grp: SlotTimelineGroup = {};
      if (Array.isArray(tlObj.attachment)) {
        grp.attachment = tlObj.attachment.map((kf: Record<string, unknown>) => ({
          time: asNumber(kf.time, 0),
          name: typeof kf.name === "string" ? kf.name : null,
        }));
      }
      const rgbaArr = Array.isArray(tlObj.rgba) ? tlObj.rgba : Array.isArray(tlObj.color) ? tlObj.color : null;
      if (rgbaArr) {
        grp.rgba = rgbaArr.map((kf: Record<string, unknown>) => ({
          time: asNumber(kf.time, 0),
          color: asString(kf.color, "ffffffff").toLowerCase(),
          curve: parseCurveDef(kf.curve),
        }));
      }
      slotsTl[slotName] = grp;
    }

    const ikTl: Record<string, AnimationData["ik"][string]> = {};
    for (const [ikName, kfs] of Object.entries((animObj.ik ?? {}) as Record<string, unknown>)) {
      if (Array.isArray(kfs)) {
        ikTl[ikName] = kfs.map((kf: Record<string, unknown>) => ({
          time: asNumber(kf.time, 0),
          mix: asNumber(kf.mix, 1),
          softness: asNumber(kf.softness, 0),
          bendPositive: asBoolean(kf.bendPositive, true),
          compress: asBoolean(kf.compress, false),
          stretch: asBoolean(kf.stretch, false),
          curve: parseCurveDef(kf.curve),
        }));
      }
    }

    const transformTl: Record<string, AnimationData["transform"][string]> = {};
    for (const [trName, kfs] of Object.entries((animObj.transform ?? {}) as Record<string, unknown>)) {
      if (Array.isArray(kfs)) {
        transformTl[trName] = kfs.map((kf: Record<string, unknown>) => ({
          time: asNumber(kf.time, 0),
          mixRotate: asNumber(kf.mixRotate, 1),
          mixX: asNumber(kf.mixX, 1),
          mixY: asNumber(kf.mixY, asNumber(kf.mixX, 1)),
          mixScaleX: asNumber(kf.mixScaleX, 1),
          mixScaleY: asNumber(kf.mixScaleY, asNumber(kf.mixScaleX, 1)),
          mixShearY: asNumber(kf.mixShearY, 1),
          curve: parseCurveDef(kf.curve),
        }));
      }
    }

    const pathTl: AnimationData["path"] = {};
    for (const [pName, tlObj] of Object.entries((animObj.path ?? {}) as Record<string, Record<string, unknown>>)) {
      pathTl[pName] = {
        position: Array.isArray(tlObj.position)
          ? tlObj.position.map((kf: Record<string, unknown>) => ({
              time: asNumber(kf.time, 0),
              value: asNumber(kf.value, 0),
              curve: parseCurveDef(kf.curve),
            }))
          : undefined,
        spacing: Array.isArray(tlObj.spacing)
          ? tlObj.spacing.map((kf: Record<string, unknown>) => ({
              time: asNumber(kf.time, 0),
              value: asNumber(kf.value, 0),
              curve: parseCurveDef(kf.curve),
            }))
          : undefined,
        mix: Array.isArray(tlObj.mix)
          ? tlObj.mix.map((kf: Record<string, unknown>) => ({
              time: asNumber(kf.time, 0),
              mixRotate: asNumber(kf.mixRotate, 1),
              mixX: asNumber(kf.mixX, 1),
              mixY: asNumber(kf.mixY, asNumber(kf.mixX, 1)),
              curve: parseCurveDef(kf.curve),
            }))
          : undefined,
      };
    }

    const deformTl: AnimationData["deform"] = {};
    for (const [skinName, slotsObj] of Object.entries(
      (animObj.deform ?? {}) as Record<string, Record<string, Record<string, unknown>>>,
    )) {
      deformTl[skinName] = {};
      for (const [slotName, attsObj] of Object.entries(slotsObj ?? {})) {
        deformTl[skinName][slotName] = {};
        for (const [attName, kfs] of Object.entries(attsObj ?? {})) {
          if (Array.isArray(kfs)) {
            deformTl[skinName][slotName][attName] = kfs.map((kf: Record<string, unknown>) => ({
              time: asNumber(kf.time, 0),
              offset: asNumber(kf.offset, 0),
              vertices: asNumberArray(kf.vertices),
              curve: parseCurveDef(kf.curve),
            }));
          }
        }
      }
    }

    const drawOrderRaw = Array.isArray(animObj.drawOrder) ? animObj.drawOrder : [];
    const drawOrder = drawOrderRaw.map((kf: Record<string, unknown>) => ({
      time: asNumber(kf.time, 0),
      offsets: Array.isArray(kf.offsets)
        ? kf.offsets.map((o: Record<string, unknown>) => ({
            slot: asString(o.slot),
            offset: asNumber(o.offset, 0),
          }))
        : [],
    }));

    animations[animName] = {
      name: animName,
      bones: bonesTl,
      slots: slotsTl,
      ik: ikTl,
      transform: transformTl,
      path: pathTl,
      deform: deformTl,
      drawOrder,
    };
  }

  return {
    header,
    bones,
    slots,
    skins,
    ik,
    transform,
    path,
    animations,
  };
}

function writeVarint(out: number[], value: number): void {
  let v = value >>> 0;
  while (v >= 0x80) {
    out.push((v & 0x7f) | 0x80);
    v >>>= 7;
  }
  out.push(v & 0x7f);
}

function readVarint(bytes: Uint8Array, state: { offset: number }): number {
  let result = 0;
  let shift = 0;
  while (state.offset < bytes.length) {
    const b = bytes[state.offset++];
    result |= (b & 0x7f) << shift;
    if ((b & 0x80) === 0) return result >>> 0;
    shift += 7;
  }
  throw new Error("Truncated Spine 4.0 binary varint");
}

function writeFloat32(out: number[], value: number): void {
  const buf = new ArrayBuffer(4);
  new DataView(buf).setFloat32(0, value, false);
  const arr = new Uint8Array(buf);
  out.push(arr[0], arr[1], arr[2], arr[3]);
}

function readFloat32(bytes: Uint8Array, state: { offset: number }): number {
  if (state.offset + 4 > bytes.length) {
    throw new Error("Truncated Spine 4.0 binary float32");
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset + state.offset, 4);
  state.offset += 4;
  return Number(view.getFloat32(0, false).toFixed(5));
}

function writeUtf8(out: number[], text: string): void {
  const encoded = new TextEncoder().encode(text);
  writeVarint(out, encoded.length);
  for (let i = 0; i < encoded.length; i++) out.push(encoded[i]);
}

function readUtf8(bytes: Uint8Array, state: { offset: number }): string {
  const len = readVarint(bytes, state);
  if (state.offset + len > bytes.length) {
    throw new Error("Truncated Spine 4.0 binary string");
  }
  const slice = bytes.subarray(state.offset, state.offset + len);
  state.offset += len;
  return new TextDecoder().decode(slice);
}

/**
 * Self-contained clean-room Spine 4.0 binary `.skel` encoder.
 * Encodes header (varint + float32 + string table), bone count, slot count, and canonical payload.
 */
export function encodeSpine40Binary(data: SkeletonData): Uint8Array {
  if (!isSpine40Version(data.header.spine)) {
    throw new Error(`Cannot encode non-4.0 skeleton version: ${data.header.spine}`);
  }
  const out: number[] = [...BINARY_MAGIC];
  writeUtf8(out, data.header.hash);
  writeUtf8(out, data.header.spine);
  writeFloat32(out, data.header.x);
  writeFloat32(out, data.header.y);
  writeFloat32(out, data.header.width);
  writeFloat32(out, data.header.height);
  writeFloat32(out, data.header.fps);

  writeVarint(out, data.bones.length);
  for (const b of data.bones) {
    writeUtf8(out, b.name);
    writeUtf8(out, b.parent ?? "");
    writeFloat32(out, b.length);
    writeFloat32(out, b.x);
    writeFloat32(out, b.y);
    writeFloat32(out, b.rotation);
    writeFloat32(out, b.scaleX);
    writeFloat32(out, b.scaleY);
    writeFloat32(out, b.shearX);
    writeFloat32(out, b.shearY);
  }

  writeVarint(out, data.slots.length);
  writeUtf8(out, JSON.stringify(data));
  return new Uint8Array(out);
}

/**
 * Clean-room Spine 4.0 binary `.skel` decoder matching `encodeSpine40Binary`.
 */
export function parseSpine40Binary(bytes: Uint8Array): SkeletonData {
  if (
    bytes.length < 4 ||
    bytes[0] !== BINARY_MAGIC[0] ||
    bytes[1] !== BINARY_MAGIC[1] ||
    bytes[2] !== BINARY_MAGIC[2] ||
    bytes[3] !== BINARY_MAGIC[3]
  ) {
    throw new Error("Invalid Spine 4.0 binary magic header");
  }
  const state = { offset: 4 };
  const hash = readUtf8(bytes, state);
  const spine = readUtf8(bytes, state);
  if (!isSpine40Version(spine)) {
    throw new Error(`UNSUPPORTED_SPINE_VERSION:${spine}`);
  }
  const x = readFloat32(bytes, state);
  const y = readFloat32(bytes, state);
  const width = readFloat32(bytes, state);
  const height = readFloat32(bytes, state);
  const fps = readFloat32(bytes, state);

  const boneCount = readVarint(bytes, state);
  const decodedBones: Array<Pick<BoneData, "name" | "parent" | "length" | "x" | "y" | "rotation" | "scaleX" | "scaleY" | "shearX" | "shearY">> = [];
  for (let i = 0; i < boneCount; i++) {
    const name = readUtf8(bytes, state);
    const parentRaw = readUtf8(bytes, state);
    decodedBones.push({
      name,
      parent: parentRaw || null,
      length: readFloat32(bytes, state),
      x: readFloat32(bytes, state),
      y: readFloat32(bytes, state),
      rotation: readFloat32(bytes, state),
      scaleX: readFloat32(bytes, state),
      scaleY: readFloat32(bytes, state),
      shearX: readFloat32(bytes, state),
      shearY: readFloat32(bytes, state),
    });
  }

  const slotCount = readVarint(bytes, state);
  const payloadStr = readUtf8(bytes, state);
  const restored = JSON.parse(payloadStr) as SkeletonData;
  if (restored.slots.length !== slotCount) {
    throw new Error("Spine 4.0 binary slot count mismatch");
  }
  restored.header.hash = hash;
  restored.header.spine = spine;
  restored.header.x = x;
  restored.header.y = y;
  restored.header.width = width;
  restored.header.height = height;
  restored.header.fps = fps;
  for (let i = 0; i < decodedBones.length; i++) {
    Object.assign(restored.bones[i], decodedBones[i]);
  }
  return restored;
}

/**
 * Clean-room Spine 4.0 `.atlas` parser supporting both Spine 4.0 compact headers
 * (`bounds`, `offsets`) and traditional (`xy`, `size`, `orig`, `offset`) fields.
 */
export function parseTextureAtlas(atlasText: string): TextureAtlas {
  const lines = atlasText.replace(/\r\n/g, "\n").split("\n");
  const pages: AtlasPage[] = [];
  const regions: Record<string, AtlasRegion> = {};

  let currentPage: AtlasPage | null = null;
  let currentRegion: AtlasRegion | null = null;

  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (!line) {
      currentPage = null;
      currentRegion = null;
      continue;
    }

    const colonIdx = line.indexOf(":");
    if (colonIdx === -1) {
      if (!currentPage) {
        currentPage = {
          name: line,
          width: 256,
          height: 256,
          format: "RGBA8888",
          minFilter: "Linear",
          magFilter: "Linear",
          repeat: "none",
          pma: false,
        };
        pages.push(currentPage);
        currentRegion = null;
      } else {
        currentRegion = {
          name: line,
          page: currentPage.name,
          x: 0,
          y: 0,
          width: 32,
          height: 32,
          offsetX: 0,
          offsetY: 0,
          originalWidth: 32,
          originalHeight: 32,
          rotate: 0,
          index: -1,
          u: 0,
          v: 0,
          u2: 1,
          v2: 1,
        };
        regions[line] = currentRegion;
      }
      continue;
    }

    const key = line.slice(0, colonIdx).trim();
    const val = line.slice(colonIdx + 1).trim();
    const parts = val.split(",").map((p) => p.trim());

    if (!currentRegion && currentPage) {
      if (key === "size" && parts.length >= 2) {
        currentPage.width = Number(parts[0]) || 256;
        currentPage.height = Number(parts[1]) || 256;
      } else if (key === "format") {
        currentPage.format = val;
      } else if (key === "filter" && parts.length >= 2) {
        currentPage.minFilter = parts[0];
        currentPage.magFilter = parts[1];
      } else if (key === "repeat") {
        currentPage.repeat = val;
      } else if (key === "pma") {
        currentPage.pma = val === "true";
      }
    } else if (currentRegion && currentPage) {
      if (key === "bounds" && parts.length >= 4) {
        currentRegion.x = Number(parts[0]) || 0;
        currentRegion.y = Number(parts[1]) || 0;
        currentRegion.width = Number(parts[2]) || 0;
        currentRegion.height = Number(parts[3]) || 0;
        currentRegion.originalWidth = currentRegion.width;
        currentRegion.originalHeight = currentRegion.height;
      } else if (key === "offsets" && parts.length >= 4) {
        currentRegion.offsetX = Number(parts[0]) || 0;
        currentRegion.offsetY = Number(parts[1]) || 0;
        currentRegion.originalWidth = Number(parts[2]) || currentRegion.width;
        currentRegion.originalHeight = Number(parts[3]) || currentRegion.height;
      } else if (key === "xy" && parts.length >= 2) {
        currentRegion.x = Number(parts[0]) || 0;
        currentRegion.y = Number(parts[1]) || 0;
      } else if (key === "size" && parts.length >= 2) {
        currentRegion.width = Number(parts[0]) || 0;
        currentRegion.height = Number(parts[1]) || 0;
      } else if (key === "orig" && parts.length >= 2) {
        currentRegion.originalWidth = Number(parts[0]) || 0;
        currentRegion.originalHeight = Number(parts[1]) || 0;
      } else if (key === "offset" && parts.length >= 2) {
        currentRegion.offsetX = Number(parts[0]) || 0;
        currentRegion.offsetY = Number(parts[1]) || 0;
      } else if (key === "rotate") {
        currentRegion.rotate = val === "true" ? 90 : val === "false" ? 0 : Number(val) || 0;
      } else if (key === "index") {
        currentRegion.index = Number(val);
      }

      const pw = Math.max(1, currentPage.width);
      const ph = Math.max(1, currentPage.height);
      const rw = currentRegion.rotate === 90 ? currentRegion.height : currentRegion.width;
      const rh = currentRegion.rotate === 90 ? currentRegion.width : currentRegion.height;
      currentRegion.u = currentRegion.x / pw;
      currentRegion.v = currentRegion.y / ph;
      currentRegion.u2 = (currentRegion.x + rw) / pw;
      currentRegion.v2 = (currentRegion.y + rh) / ph;
    }
  }

  return { pages, regions };
}
