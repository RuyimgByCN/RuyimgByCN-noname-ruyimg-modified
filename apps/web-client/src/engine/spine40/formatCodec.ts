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

import { encodeStandardSpine40Skel, parseStandardSpine40Skel } from "./binarySkelCodec";

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
  const pendingJsonLinked: Array<{
    skinIdx: number;
    slotName: string;
    attKey: string;
    parentSkinName: string;
    parentName: string;
    hasWidth: boolean;
    hasHeight: boolean;
  }> = [];
  if (Array.isArray(obj.skins)) {
    for (const sk of obj.skins as Array<Record<string, unknown>>) {
      const skinIdx = skins.length;
      const attachmentsMap: Record<string, Record<string, Attachment>> = {};
      const rawAtts = (sk.attachments ?? {}) as Record<string, Record<string, Record<string, unknown>>>;
      for (const [slotName, slotEntries] of Object.entries(rawAtts)) {
        attachmentsMap[slotName] = {};
        for (const [attKey, attVal] of Object.entries(slotEntries ?? {})) {
          attachmentsMap[slotName][attKey] = parseAttachment(attKey, attVal);
          if (asString(attVal?.type, "region") === "linkedmesh" && !Array.isArray(attVal?.uvs)) {
            pendingJsonLinked.push({
              skinIdx,
              slotName,
              attKey,
              parentSkinName: asString(attVal.skin, "default"),
              parentName: asString(attVal.parent, attKey),
              hasWidth: typeof attVal.width === "number",
              hasHeight: typeof attVal.height === "number",
            });
          }
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
  for (const lm of pendingJsonLinked) {
    const parentSkin = skins.find((s) => s.name === lm.parentSkinName) ?? skins[0];
    const parentAtt = parentSkin?.attachments?.[lm.slotName]?.[lm.parentName];
    const curAtt = skins[lm.skinIdx]?.attachments?.[lm.slotName]?.[lm.attKey];
    if (parentAtt && parentAtt.type === "mesh" && curAtt && curAtt.type === "mesh") {
      curAtt.uvs = [...parentAtt.uvs];
      curAtt.triangles = [...parentAtt.triangles];
      curAtt.vertices = [...parentAtt.vertices];
      curAtt.hull = parentAtt.hull;
      curAtt.edges = [...parentAtt.edges];
      curAtt.weighted = parentAtt.weighted;
      if (!lm.hasWidth) curAtt.width = parentAtt.width;
      if (!lm.hasHeight) curAtt.height = parentAtt.height;
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

/**
 * Clean-room Spine 4.0 binary `.skel` encoder delegating directly to the
 * standard Spine 4.0 binary byte layout (`encodeStandardSpine40Skel`).
 */
export function encodeSpine40Binary(data: SkeletonData): Uint8Array {
  return encodeStandardSpine40Skel(data);
}

/**
 * Clean-room Spine 4.0 binary `.skel` decoder delegating directly to the
 * standard Spine 4.0 binary byte layout (`parseStandardSpine40Skel`).
 */
export function parseSpine40Binary(bytes: Uint8Array): SkeletonData {
  return parseStandardSpine40Skel(bytes);
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
