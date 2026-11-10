/**
 * Clean-room standard Spine 4.0 binary `.skel` stream encoder & decoder (Decision 35).
 *
 * Authored strictly from the public Spine 4.0 binary format specification
 * using only Web-standard `Uint8Array`, `DataView`, `TextEncoder`, and `TextDecoder`.
 * Zero external Spine runtime code, zero Node `Buffer`.
 */

import type {
  AnimationData,
  Attachment,
  BlendMode,
  BoneData,
  BoneTimelineGroup,
  CurveDef,
  IkConstraintData,
  PathConstraintData,
  PositionMode,
  RotateMode,
  SkeletonData,
  SkinData,
  SlotData,
  SlotTimelineGroup,
  SpacingMode,
  TransformConstraintData,
  TransformMode,
} from "./types";

const TRANSFORM_MODES: readonly TransformMode[] = [
  "normal",
  "onlyTranslation",
  "noRotationOrReflection",
  "noScale",
  "noScaleOrReflection",
];
const BLEND_MODES: readonly BlendMode[] = ["normal", "additive", "multiply", "screen"];
const POSITION_MODES: readonly PositionMode[] = ["fixed", "percent"];
const SPACING_MODES: readonly SpacingMode[] = ["length", "fixed", "percent", "proportional"];
const ROTATE_MODES: readonly RotateMode[] = ["tangent", "chain", "chainScale"];

function isSpine40Ver(v: string): boolean {
  const t = v.trim();
  return t === "4.0" || t.startsWith("4.0.");
}

class BinaryWriter {
  readonly bytes: number[] = [];

  writeByte(b: number): void {
    this.bytes.push(b & 0xff);
  }

  writeBool(v: boolean): void {
    this.bytes.push(v ? 1 : 0);
  }

  writeInt32(v: number): void {
    this.bytes.push((v >>> 24) & 0xff, (v >>> 16) & 0xff, (v >>> 8) & 0xff, v & 0xff);
  }

  writeUint16(v: number): void {
    this.bytes.push((v >>> 8) & 0xff, v & 0xff);
  }

  writeVarint(value: number, optimizePositive = true): void {
    let v = optimizePositive ? value >>> 0 : ((value << 1) ^ (value >> 31)) >>> 0;
    while (v >= 0x80) {
      this.bytes.push((v & 0x7f) | 0x80);
      v >>>= 7;
    }
    this.bytes.push(v & 0x7f);
  }

  writeFloat(v: number): void {
    const buf = new ArrayBuffer(4);
    new DataView(buf).setFloat32(0, v, false);
    const arr = new Uint8Array(buf);
    this.bytes.push(arr[0], arr[1], arr[2], arr[3]);
  }

  writeString(s: string | null | undefined): void {
    if (s === null || s === undefined) {
      this.writeVarint(0);
      return;
    }
    if (s.length === 0) {
      this.writeVarint(1);
      return;
    }
    const enc = new TextEncoder().encode(s);
    this.writeVarint(enc.length + 1);
    for (let i = 0; i < enc.length; i++) this.bytes.push(enc[i]);
  }

  writeColorHex(hex: string | undefined): void {
    const clean = (hex ?? "ffffffff").replace(/^#/, "").padEnd(8, "f").slice(0, 8);
    const num = Number.parseInt(clean, 16);
    this.writeInt32(Number.isFinite(num) ? num : 0xffffffff);
  }
}

class BinaryReader {
  offset = 0;
  readonly bytes: Uint8Array;
  strings: string[] = [];

  constructor(bytes: Uint8Array) {
    this.bytes = bytes;
  }

  readByte(): number {
    if (this.offset >= this.bytes.length) throw new Error("Truncated Spine 4.0 binary byte");
    return this.bytes[this.offset++];
  }

  readSignedByte(): number {
    const b = this.readByte();
    return b > 127 ? b - 256 : b;
  }

  readBool(): boolean {
    return this.readByte() !== 0;
  }

  readInt32(): number {
    if (this.offset + 4 > this.bytes.length) throw new Error("Truncated Spine 4.0 binary int32");
    const view = new DataView(this.bytes.buffer, this.bytes.byteOffset + this.offset, 4);
    this.offset += 4;
    return view.getInt32(0, false);
  }

  readUint32(): number {
    return this.readInt32() >>> 0;
  }

  readUint16(): number {
    if (this.offset + 2 > this.bytes.length) throw new Error("Truncated Spine 4.0 binary uint16");
    const hi = this.bytes[this.offset++];
    const lo = this.bytes[this.offset++];
    return (hi << 8) | lo;
  }

  readVarint(optimizePositive = true): number {
    let result = 0;
    let shift = 0;
    while (this.offset < this.bytes.length && shift < 35) {
      const b = this.bytes[this.offset++];
      result |= (b & 0x7f) << shift;
      if ((b & 0x80) === 0) {
        const u = result >>> 0;
        return optimizePositive ? u | 0 : (u >>> 1) ^ -(u & 1);
      }
      shift += 7;
    }
    throw new Error("Truncated Spine 4.0 binary varint");
  }

  readFloat(): number {
    if (this.offset + 4 > this.bytes.length) throw new Error("Truncated Spine 4.0 binary float");
    const view = new DataView(this.bytes.buffer, this.bytes.byteOffset + this.offset, 4);
    this.offset += 4;
    return Number(view.getFloat32(0, false).toFixed(5));
  }

  readString(): string | null {
    const lenPlusOne = this.readVarint();
    if (lenPlusOne === 0) return null;
    if (lenPlusOne === 1) return "";
    const len = lenPlusOne - 1;
    if (len > 1_048_576 || this.offset + len > this.bytes.length) {
      throw new Error("Invalid or truncated Spine 4.0 binary string");
    }
    const slice = this.bytes.subarray(this.offset, this.offset + len);
    this.offset += len;
    return new TextDecoder("utf-8", { fatal: true }).decode(slice);
  }

  readStringRef(): string | null {
    const idx = this.readVarint();
    if (idx === 0) return null;
    if (idx - 1 >= this.strings.length) throw new Error("Spine 4.0 binary stringRef out of bounds");
    return this.strings[idx - 1];
  }

  readColorHex(): string {
    return this.readUint32().toString(16).padStart(8, "0");
  }
}

function writeVertices(w: BinaryWriter, vertices: number[], weighted: boolean, vertexCount: number): void {
  w.writeBool(weighted);
  if (!weighted) {
    for (let i = 0; i < vertexCount * 2; i++) w.writeFloat(vertices[i] ?? 0);
    return;
  }
  let idx = 0;
  for (let v = 0; v < vertexCount; v++) {
    const boneCount = Math.round(vertices[idx++] ?? 0);
    w.writeVarint(boneCount);
    for (let b = 0; b < boneCount; b++) {
      w.writeVarint(Math.round(vertices[idx++] ?? 0));
      w.writeFloat(vertices[idx++] ?? 0);
      w.writeFloat(vertices[idx++] ?? 0);
      w.writeFloat(vertices[idx++] ?? 0);
    }
  }
}

function readVertices(r: BinaryReader, vertexCount: number): { vertices: number[]; weighted: boolean } {
  const weighted = r.readBool();
  const vertices: number[] = [];
  if (!weighted) {
    for (let i = 0; i < vertexCount * 2; i++) vertices.push(r.readFloat());
    return { vertices, weighted: false };
  }
  for (let v = 0; v < vertexCount; v++) {
    const boneCount = r.readVarint();
    vertices.push(boneCount);
    for (let b = 0; b < boneCount; b++) {
      vertices.push(r.readVarint(), r.readFloat(), r.readFloat(), r.readFloat());
    }
  }
  return { vertices, weighted: true };
}

function writeCurve(w: BinaryWriter, curve: CurveDef | undefined): void {
  if (curve === "stepped") {
    w.writeByte(1);
  } else if (Array.isArray(curve) && curve.length >= 4) {
    w.writeByte(2);
    w.writeFloat(curve[0]);
    w.writeFloat(curve[1]);
    w.writeFloat(curve[2]);
    w.writeFloat(curve[3]);
  } else {
    w.writeByte(0);
  }
}

function readCurve(r: BinaryReader): CurveDef | undefined {
  const t = r.readByte();
  if (t === 1) return "stepped";
  if (t === 2) return [r.readFloat(), r.readFloat(), r.readFloat(), r.readFloat()];
  return undefined;
}

export interface Spine40BinaryEncodeOptions {
  nonessential?: boolean;
  emitLinkedMeshForIdenticalTopology?: boolean;
}

/**
 * Encodes a `SkeletonData` into standard Spine 4.0 binary `.skel` byte layout
 * (8-byte big-endian hash + version string + header floats + string table +
 * bones + slots + constraints + skins + events + animations).
 */
export function encodeStandardSpine40Skel(
  data: SkeletonData,
  options?: Spine40BinaryEncodeOptions,
): Uint8Array {
  if (!isSpine40Ver(data.header.spine)) {
    throw new Error(`Cannot encode non-4.0 skeleton version: ${data.header.spine}`);
  }
  const nonessential = options?.nonessential ?? true;
  const emitLinkedMesh = options?.emitLinkedMeshForIdenticalTopology ?? false;

  const w = new BinaryWriter();
  let hHi = 0x811c9dc5;
  let hLo = 0x01000193;
  if (/^[0-9a-fA-F]{16}$/.test(data.header.hash)) {
    hHi = Number.parseInt(data.header.hash.slice(0, 8), 16) >>> 0;
    hLo = Number.parseInt(data.header.hash.slice(8, 16), 16) >>> 0;
  } else {
    for (let i = 0; i < data.header.hash.length; i++) {
      const ch = data.header.hash.charCodeAt(i);
      hHi = Math.imul(hHi ^ ch, 0x01000193) >>> 0;
      hLo = Math.imul(hLo ^ (ch + i), 0x01000193) >>> 0;
    }
  }
  w.writeInt32(hHi);
  w.writeInt32(hLo);
  w.writeString(data.header.spine);
  w.writeFloat(data.header.x);
  w.writeFloat(data.header.y);
  w.writeFloat(data.header.width);
  w.writeFloat(data.header.height);
  w.writeBool(nonessential);
  if (nonessential) {
    w.writeFloat(data.header.fps);
    w.writeString(data.header.images ?? null);
    w.writeString(data.header.audio ?? null);
  }

  // Build deduplicated string table without polluting strings[0] with header.hash
  const strPool: string[] = [];
  const strIdx = new Map<string, number>();
  const intern = (s: string | null | undefined): number => {
    if (s === null || s === undefined) return 0;
    const hit = strIdx.get(s);
    if (hit !== undefined) return hit;
    strPool.push(s);
    const id = strPool.length;
    strIdx.set(s, id);
    return id;
  };
  for (const s of data.slots) intern(s.attachment);
  const defaultSkin = data.skins.find((s) => s.name === "default");
  if (emitLinkedMesh && defaultSkin) intern("default");
  for (const sk of data.skins) {
    for (const slotMapObj of Object.values(sk.attachments)) {
      for (const [attKey, att] of Object.entries(slotMapObj)) {
        intern(attKey);
        intern(att.name);
        if (att.type === "region" || att.type === "mesh") intern(att.path);
      }
    }
  }

  w.writeVarint(strPool.length);
  for (const s of strPool) w.writeString(s);

  const boneMap = new Map<string, number>();
  data.bones.forEach((b, i) => boneMap.set(b.name, i));
  const slotMap = new Map<string, number>();
  data.slots.forEach((s, i) => slotMap.set(s.name, i));

  // Bones
  w.writeVarint(data.bones.length);
  data.bones.forEach((b, idx) => {
    w.writeString(b.name);
    if (idx > 0) w.writeVarint(b.parent ? (boneMap.get(b.parent) ?? 0) : 0);
    w.writeFloat(b.rotation);
    w.writeFloat(b.x);
    w.writeFloat(b.y);
    w.writeFloat(b.scaleX);
    w.writeFloat(b.scaleY);
    w.writeFloat(b.shearX);
    w.writeFloat(b.shearY);
    w.writeFloat(b.length);
    w.writeByte(Math.max(0, TRANSFORM_MODES.indexOf(b.transform)));
    w.writeBool(b.skin);
    if (nonessential) w.writeInt32(0x9b9b9bff);
  });

  // Slots
  w.writeVarint(data.slots.length);
  for (const s of data.slots) {
    w.writeString(s.name);
    w.writeVarint(boneMap.get(s.bone) ?? 0);
    w.writeColorHex(s.color);
    if (s.dark) {
      w.writeInt32(Number.parseInt(s.dark.slice(0, 6), 16) & 0x00ffffff);
    } else {
      w.writeInt32(-1);
    }
    w.writeVarint(intern(s.attachment));
    w.writeVarint(Math.max(0, BLEND_MODES.indexOf(s.blend)));
  }

  // IK
  w.writeVarint(data.ik.length);
  for (const ik of data.ik) {
    w.writeString(ik.name);
    w.writeVarint(ik.order);
    w.writeBool(ik.skin);
    w.writeVarint(ik.bones.length);
    for (const bn of ik.bones) w.writeVarint(boneMap.get(bn) ?? 0);
    w.writeVarint(boneMap.get(ik.target) ?? 0);
    w.writeFloat(ik.mix);
    w.writeFloat(ik.softness);
    w.writeByte(ik.bendPositive ? 1 : 0xff);
    w.writeBool(ik.compress);
    w.writeBool(ik.stretch);
    w.writeBool(ik.uniform);
  }

  // Transform
  w.writeVarint(data.transform.length);
  for (const tr of data.transform) {
    w.writeString(tr.name);
    w.writeVarint(tr.order);
    w.writeBool(tr.skin);
    w.writeVarint(tr.bones.length);
    for (const bn of tr.bones) w.writeVarint(boneMap.get(bn) ?? 0);
    w.writeVarint(boneMap.get(tr.target) ?? 0);
    w.writeBool(tr.local);
    w.writeBool(tr.relative);
    w.writeFloat(tr.offsetRotation);
    w.writeFloat(tr.offsetX);
    w.writeFloat(tr.offsetY);
    w.writeFloat(tr.offsetScaleX);
    w.writeFloat(tr.offsetScaleY);
    w.writeFloat(tr.offsetShearY);
    w.writeFloat(tr.mixRotate);
    w.writeFloat(tr.mixX);
    w.writeFloat(tr.mixY);
    w.writeFloat(tr.mixScaleX);
    w.writeFloat(tr.mixScaleY);
    w.writeFloat(tr.mixShearY);
  }

  // Path
  w.writeVarint(data.path.length);
  for (const p of data.path) {
    w.writeString(p.name);
    w.writeVarint(p.order);
    w.writeBool(p.skin);
    w.writeVarint(p.bones.length);
    for (const bn of p.bones) w.writeVarint(boneMap.get(bn) ?? 0);
    w.writeVarint(slotMap.get(p.target) ?? 0);
    w.writeVarint(Math.max(0, POSITION_MODES.indexOf(p.positionMode)));
    w.writeVarint(Math.max(0, SPACING_MODES.indexOf(p.spacingMode)));
    w.writeVarint(Math.max(0, ROTATE_MODES.indexOf(p.rotateMode)));
    w.writeFloat(p.offsetRotation);
    w.writeFloat(p.position);
    w.writeFloat(p.spacing);
    w.writeFloat(p.mixRotate);
    w.writeFloat(p.mixX);
    w.writeFloat(p.mixY);
  }

  const writeSkinAttachments = (sk: SkinData) => {
    const slotEntries = Object.entries(sk.attachments);
    w.writeVarint(slotEntries.length);
    for (const [slotName, attRecord] of slotEntries) {
      w.writeVarint(slotMap.get(slotName) ?? 0);
      const attList = Object.entries(attRecord);
      w.writeVarint(attList.length);
      for (const [attKey, att] of attList) {
        w.writeVarint(intern(attKey));
        w.writeVarint(intern(att.name));
        if (att.type === "region") {
          w.writeByte(0);
          w.writeVarint(intern(att.path));
          w.writeFloat(att.rotation);
          w.writeFloat(att.x);
          w.writeFloat(att.y);
          w.writeFloat(att.scaleX);
          w.writeFloat(att.scaleY);
          w.writeFloat(att.width);
          w.writeFloat(att.height);
          w.writeColorHex(att.color);
        } else if (att.type === "boundingbox") {
          w.writeByte(1);
          w.writeVarint(att.vertexCount);
          writeVertices(w, att.vertices, att.weighted, att.vertexCount);
          if (nonessential) w.writeColorHex(att.color);
        } else if (att.type === "mesh") {
          const parentCand =
            emitLinkedMesh && sk.name !== "default" ? defaultSkin?.attachments?.[slotName]?.[attKey] : undefined;
          if (
            parentCand &&
            parentCand.type === "mesh" &&
            parentCand.uvs.length === att.uvs.length &&
            parentCand.triangles.length === att.triangles.length
          ) {
            w.writeByte(3); // LinkedMesh
            w.writeVarint(intern(att.path));
            w.writeColorHex(att.color);
            w.writeVarint(intern("default"));
            w.writeVarint(intern(attKey));
            w.writeBool(true); // inheritDeform
            if (nonessential) {
              w.writeFloat(att.width);
              w.writeFloat(att.height);
            }
          } else {
            w.writeByte(2);
            w.writeVarint(intern(att.path));
            w.writeColorHex(att.color);
            const vCount = att.uvs.length >> 1;
            w.writeVarint(vCount);
            for (const uv of att.uvs) w.writeFloat(uv);
            w.writeVarint(att.triangles.length);
            for (const tri of att.triangles) w.writeUint16(tri);
            writeVertices(w, att.vertices, att.weighted, vCount);
            w.writeVarint(att.hull);
            if (nonessential) {
              w.writeVarint(att.edges.length);
              for (const e of att.edges) w.writeUint16(e);
              w.writeFloat(att.width);
              w.writeFloat(att.height);
            }
          }
        } else if (att.type === "path") {
          w.writeByte(4);
          w.writeBool(att.closed);
          w.writeBool(att.constantSpeed);
          w.writeVarint(att.vertexCount);
          writeVertices(w, att.vertices, att.weighted, att.vertexCount);
          const lenCount = Math.floor(att.vertexCount / 3);
          for (let li = 0; li < lenCount; li++) w.writeFloat(att.lengths[li] ?? 0);
          if (nonessential) w.writeColorHex(att.color);
        } else if (att.type === "point") {
          w.writeByte(5);
          w.writeFloat(att.rotation);
          w.writeFloat(att.x);
          w.writeFloat(att.y);
          if (nonessential) w.writeColorHex(att.color);
        } else if (att.type === "clipping") {
          w.writeByte(6);
          w.writeVarint(att.end ? (slotMap.get(att.end) ?? 0) : 0xffff);
          w.writeVarint(att.vertexCount);
          writeVertices(w, att.vertices, att.weighted, att.vertexCount);
          if (nonessential) w.writeColorHex(att.color);
        }
      }
    }
  };

  if (defaultSkin) {
    writeSkinAttachments(defaultSkin);
  } else {
    w.writeVarint(0);
  }

  const namedSkins = data.skins.filter((s) => s.name !== "default");
  w.writeVarint(namedSkins.length);
  for (const sk of namedSkins) {
    w.writeString(sk.name);
    w.writeVarint(sk.bones.length);
    for (const bn of sk.bones) w.writeVarint(boneMap.get(bn) ?? 0);
    w.writeVarint(sk.ik.length);
    for (const iname of sk.ik) w.writeVarint(Math.max(0, data.ik.findIndex((x) => x.name === iname)));
    w.writeVarint(sk.transform.length);
    for (const tname of sk.transform) w.writeVarint(Math.max(0, data.transform.findIndex((x) => x.name === tname)));
    w.writeVarint(sk.path.length);
    for (const pname of sk.path) w.writeVarint(Math.max(0, data.path.findIndex((x) => x.name === pname)));
    writeSkinAttachments(sk);
  }

  // Events (0)
  w.writeVarint(0);

  // Animations
  const animEntries = Object.entries(data.animations);
  w.writeVarint(animEntries.length);
  for (const [animName, anim] of animEntries) {
    w.writeString(animName);
    const slotTlEntries = Object.entries(anim.slots);
    w.writeVarint(slotTlEntries.length);
    for (const [slotName, grp] of slotTlEntries) {
      w.writeVarint(slotMap.get(slotName) ?? 0);
      const count = (grp.attachment ? 1 : 0) + (grp.rgba ? 1 : 0);
      w.writeVarint(count);
      if (grp.attachment) {
        w.writeByte(0);
        w.writeVarint(grp.attachment.length);
        for (const kf of grp.attachment) {
          w.writeFloat(kf.time);
          w.writeString(kf.name);
        }
      }
      if (grp.rgba) {
        w.writeByte(1);
        w.writeVarint(grp.rgba.length);
        for (const kf of grp.rgba) {
          w.writeFloat(kf.time);
          w.writeColorHex(kf.color);
          writeCurve(w, kf.curve);
        }
      }
    }

    const boneTlEntries = Object.entries(anim.bones);
    w.writeVarint(boneTlEntries.length);
    for (const [boneName, grp] of boneTlEntries) {
      w.writeVarint(boneMap.get(boneName) ?? 0);
      const count = (grp.rotate ? 1 : 0) + (grp.translate ? 1 : 0) + (grp.scale ? 1 : 0) + (grp.shear ? 1 : 0);
      w.writeVarint(count);
      if (grp.rotate) {
        w.writeByte(0);
        w.writeVarint(grp.rotate.length);
        for (const kf of grp.rotate) {
          w.writeFloat(kf.time);
          w.writeFloat(kf.value);
          writeCurve(w, kf.curve);
        }
      }
      if (grp.translate) {
        w.writeByte(1);
        w.writeVarint(grp.translate.length);
        for (const kf of grp.translate) {
          w.writeFloat(kf.time);
          w.writeFloat(kf.x);
          w.writeFloat(kf.y);
          writeCurve(w, kf.curve);
        }
      }
      if (grp.scale) {
        w.writeByte(2);
        w.writeVarint(grp.scale.length);
        for (const kf of grp.scale) {
          w.writeFloat(kf.time);
          w.writeFloat(kf.x);
          w.writeFloat(kf.y);
          writeCurve(w, kf.curve);
        }
      }
      if (grp.shear) {
        w.writeByte(3);
        w.writeVarint(grp.shear.length);
        for (const kf of grp.shear) {
          w.writeFloat(kf.time);
          w.writeFloat(kf.x);
          w.writeFloat(kf.y);
          writeCurve(w, kf.curve);
        }
      }
    }

    // IK timelines
    const ikEntries = Object.entries(anim.ik);
    w.writeVarint(ikEntries.length);
    for (const [ikName, kfs] of ikEntries) {
      const idx = Math.max(0, data.ik.findIndex((x) => x.name === ikName));
      w.writeVarint(idx);
      w.writeVarint(kfs.length);
      for (const kf of kfs) {
        w.writeFloat(kf.time);
        w.writeFloat(kf.mix);
        w.writeFloat(kf.softness);
        w.writeByte(kf.bendPositive ? 1 : 0xff);
        w.writeBool(kf.compress);
        w.writeBool(kf.stretch);
        writeCurve(w, kf.curve);
      }
    }

    // Transform timelines
    const trEntries = Object.entries(anim.transform);
    w.writeVarint(trEntries.length);
    for (const [trName, kfs] of trEntries) {
      const idx = Math.max(0, data.transform.findIndex((x) => x.name === trName));
      w.writeVarint(idx);
      w.writeVarint(kfs.length);
      for (const kf of kfs) {
        w.writeFloat(kf.time);
        w.writeFloat(kf.mixRotate);
        w.writeFloat(kf.mixX);
        w.writeFloat(kf.mixY);
        w.writeFloat(kf.mixScaleX);
        w.writeFloat(kf.mixScaleY);
        w.writeFloat(kf.mixShearY);
        writeCurve(w, kf.curve);
      }
    }

    // Path timelines
    const pEntries = Object.entries(anim.path);
    w.writeVarint(pEntries.length);
    for (const [pName, grp] of pEntries) {
      const idx = Math.max(0, data.path.findIndex((x) => x.name === pName));
      w.writeVarint(idx);
      const count = (grp.position ? 1 : 0) + (grp.spacing ? 1 : 0) + (grp.mix ? 1 : 0);
      w.writeVarint(count);
      if (grp.position) {
        w.writeByte(0);
        w.writeVarint(grp.position.length);
        for (const kf of grp.position) {
          w.writeFloat(kf.time);
          w.writeFloat(kf.value);
          writeCurve(w, kf.curve);
        }
      }
      if (grp.spacing) {
        w.writeByte(1);
        w.writeVarint(grp.spacing.length);
        for (const kf of grp.spacing) {
          w.writeFloat(kf.time);
          w.writeFloat(kf.value);
          writeCurve(w, kf.curve);
        }
      }
      if (grp.mix) {
        w.writeByte(2);
        w.writeVarint(grp.mix.length);
        for (const kf of grp.mix) {
          w.writeFloat(kf.time);
          w.writeFloat(kf.mixRotate);
          w.writeFloat(kf.mixX);
          w.writeFloat(kf.mixY);
          writeCurve(w, kf.curve);
        }
      }
    }

    // Deform timelines
    const deformSkins = Object.entries(anim.deform);
    let deformCount = 0;
    for (const [, slMap] of deformSkins) {
      for (const [, atMap] of Object.entries(slMap)) deformCount += Object.keys(atMap).length;
    }
    w.writeVarint(deformCount);
    for (const [skinName, slMap] of deformSkins) {
      const skinIdx = Math.max(0, data.skins.findIndex((x) => x.name === skinName));
      for (const [slName, atMap] of Object.entries(slMap)) {
        const slIdx = slotMap.get(slName) ?? 0;
        for (const [atName, kfs] of Object.entries(atMap)) {
          w.writeVarint(skinIdx);
          w.writeVarint(slIdx);
          w.writeVarint(intern(atName));
          w.writeVarint(kfs.length);
          for (const kf of kfs) {
            w.writeFloat(kf.time);
            w.writeVarint(kf.offset);
            w.writeVarint(kf.vertices.length);
            for (const v of kf.vertices) w.writeFloat(v);
            writeCurve(w, kf.curve);
          }
        }
      }
    }

    // DrawOrder timelines
    w.writeVarint(anim.drawOrder.length);
    for (const kf of anim.drawOrder) {
      w.writeFloat(kf.time);
      w.writeVarint(kf.offsets.length);
      for (const off of kf.offsets) {
        w.writeVarint(slotMap.get(off.slot) ?? 0);
        w.writeVarint(off.offset, false);
      }
    }

    // Event timeline (0)
    w.writeVarint(0);
  }

  return new Uint8Array(w.bytes);
}

interface PendingBinaryLinkedMesh {
  targetSkinIdx: number;
  slotName: string;
  key: string;
  name: string;
  path: string;
  color: string;
  parentSkinName: string;
  parentName: string;
  width: number;
  height: number;
  hasExplicitSize: boolean;
}

/**
 * Decodes a standard Spine 4.0 binary `.skel` byte stream into `SkeletonData`.
 */
export function parseStandardSpine40Skel(bytes: Uint8Array): SkeletonData {
  if (bytes.length < 12) {
    throw new Error("Invalid Spine 4.0 binary header: stream too short");
  }
  const r = new BinaryReader(bytes);
  const hashHi = r.readUint32();
  const hashLo = r.readUint32();
  const rawHash = `${hashHi.toString(16).padStart(8, "0")}${hashLo.toString(16).padStart(8, "0")}`;

  const spine = r.readString() ?? "";
  if (!/^\d+\.\d+/.test(spine)) {
    throw new Error("Invalid Spine 4.0 binary magic header");
  }
  if (!isSpine40Ver(spine)) {
    throw new Error(`UNSUPPORTED_SPINE_VERSION:${spine} (clean-room player requires Spine 4.0)`);
  }

  const x = r.readFloat();
  const y = r.readFloat();
  const width = r.readFloat();
  const height = r.readFloat();
  const nonessential = r.readBool();
  const fps = nonessential ? r.readFloat() : 30;
  const images = nonessential ? (r.readString() ?? undefined) : undefined;
  const audio = nonessential ? (r.readString() ?? undefined) : undefined;

  const stringCount = r.readVarint();
  for (let i = 0; i < stringCount; i++) {
    r.strings.push(r.readString() ?? "");
  }

  // Bones
  const boneCount = r.readVarint();
  const bones: BoneData[] = [];
  for (let i = 0; i < boneCount; i++) {
    const name = r.readString() ?? `bone_${i}`;
    const parentIdx = i === 0 ? -1 : r.readVarint();
    const rotation = r.readFloat();
    const bx = r.readFloat();
    const by = r.readFloat();
    const scaleX = r.readFloat();
    const scaleY = r.readFloat();
    const shearX = r.readFloat();
    const shearY = r.readFloat();
    const length = r.readFloat();
    const transform = TRANSFORM_MODES[r.readByte()] ?? "normal";
    const skin = r.readBool();
    if (nonessential) r.readInt32();
    bones.push({
      name,
      parent: parentIdx >= 0 && bones[parentIdx] ? bones[parentIdx].name : null,
      length,
      transform,
      skin,
      x: bx,
      y: by,
      rotation,
      scaleX,
      scaleY,
      shearX,
      shearY,
    });
  }

  // Slots
  const slotCount = r.readVarint();
  const slots: SlotData[] = [];
  for (let i = 0; i < slotCount; i++) {
    const name = r.readString() ?? `slot_${i}`;
    const boneIdx = r.readVarint();
    const color = r.readColorHex();
    const darkInt = r.readInt32();
    const dark = darkInt === -1 ? null : (darkInt & 0x00ffffff).toString(16).padStart(6, "0");
    const attachment = r.readStringRef();
    const blend = BLEND_MODES[r.readVarint()] ?? "normal";
    slots.push({
      name,
      bone: bones[boneIdx]?.name ?? bones[0]?.name ?? "root",
      color,
      dark,
      attachment,
      blend,
    });
  }

  // IK
  const ikCount = r.readVarint();
  const ik: IkConstraintData[] = [];
  for (let i = 0; i < ikCount; i++) {
    const name = r.readString() ?? `ik_${i}`;
    const order = r.readVarint();
    const skin = r.readBool();
    const bc = r.readVarint();
    const ikBones: string[] = [];
    for (let j = 0; j < bc; j++) ikBones.push(bones[r.readVarint()]?.name ?? "root");
    const target = bones[r.readVarint()]?.name ?? "root";
    const mix = r.readFloat();
    const softness = r.readFloat();
    const bendPositive = r.readSignedByte() >= 0;
    const compress = r.readBool();
    const stretch = r.readBool();
    const uniform = r.readBool();
    ik.push({ name, order, skin, bones: ikBones, target, mix, softness, bendPositive, compress, stretch, uniform });
  }

  // Transform
  const trCount = r.readVarint();
  const transform: TransformConstraintData[] = [];
  for (let i = 0; i < trCount; i++) {
    const name = r.readString() ?? `tr_${i}`;
    const order = r.readVarint();
    const skin = r.readBool();
    const bc = r.readVarint();
    const trBones: string[] = [];
    for (let j = 0; j < bc; j++) trBones.push(bones[r.readVarint()]?.name ?? "root");
    const target = bones[r.readVarint()]?.name ?? "root";
    const local = r.readBool();
    const relative = r.readBool();
    transform.push({
      name,
      order,
      skin,
      bones: trBones,
      target,
      local,
      relative,
      offsetRotation: r.readFloat(),
      offsetX: r.readFloat(),
      offsetY: r.readFloat(),
      offsetScaleX: r.readFloat(),
      offsetScaleY: r.readFloat(),
      offsetShearY: r.readFloat(),
      mixRotate: r.readFloat(),
      mixX: r.readFloat(),
      mixY: r.readFloat(),
      mixScaleX: r.readFloat(),
      mixScaleY: r.readFloat(),
      mixShearY: r.readFloat(),
    });
  }

  // Path
  const pathCount = r.readVarint();
  const path: PathConstraintData[] = [];
  for (let i = 0; i < pathCount; i++) {
    const name = r.readString() ?? `path_${i}`;
    const order = r.readVarint();
    const skin = r.readBool();
    const bc = r.readVarint();
    const pBones: string[] = [];
    for (let j = 0; j < bc; j++) pBones.push(bones[r.readVarint()]?.name ?? "root");
    const target = slots[r.readVarint()]?.name ?? "";
    const positionMode = POSITION_MODES[r.readVarint()] ?? "percent";
    const spacingMode = SPACING_MODES[r.readVarint()] ?? "length";
    const rotateMode = ROTATE_MODES[r.readVarint()] ?? "tangent";
    path.push({
      name,
      order,
      skin,
      bones: pBones,
      target,
      positionMode,
      spacingMode,
      rotateMode,
      offsetRotation: r.readFloat(),
      position: r.readFloat(),
      spacing: r.readFloat(),
      mixRotate: r.readFloat(),
      mixX: r.readFloat(),
      mixY: r.readFloat(),
    });
  }

  const pendingLinkedMeshes: PendingBinaryLinkedMesh[] = [];

  const readAttachmentEntry = (
    targetSkinIdx: number,
    slotName: string,
    placeholderName: string,
  ): Attachment => {
    const name = r.readStringRef() ?? placeholderName;
    const type = r.readByte();
    if (type === 0) {
      const attPath = r.readStringRef() ?? name;
      const rotation = r.readFloat();
      const ax = r.readFloat();
      const ay = r.readFloat();
      const scaleX = r.readFloat();
      const scaleY = r.readFloat();
      const aw = r.readFloat();
      const ah = r.readFloat();
      const color = r.readColorHex();
      return { type: "region", name, path: attPath, x: ax, y: ay, scaleX, scaleY, rotation, width: aw, height: ah, color };
    }
    if (type === 1) {
      const vertexCount = r.readVarint();
      const { vertices, weighted } = readVertices(r, vertexCount);
      const color = nonessential ? r.readColorHex() : "ffffffff";
      return { type: "boundingbox", name, vertexCount, vertices, color, weighted };
    }
    if (type === 2) {
      const attPath = r.readStringRef() ?? name;
      const color = r.readColorHex();
      const vertexCount = r.readVarint();
      const uvs: number[] = [];
      for (let i = 0; i < vertexCount * 2; i++) uvs.push(r.readFloat());
      const triCount = r.readVarint();
      const triangles: number[] = [];
      for (let i = 0; i < triCount; i++) triangles.push(r.readUint16());
      const { vertices, weighted } = readVertices(r, vertexCount);
      const hull = r.readVarint();
      const edges: number[] = [];
      let mw = 32;
      let mh = 32;
      if (nonessential) {
        const edgeCount = r.readVarint();
        for (let i = 0; i < edgeCount; i++) edges.push(r.readUint16());
        mw = r.readFloat();
        mh = r.readFloat();
      }
      return { type: "mesh", name, path: attPath, uvs, triangles, vertices, hull, edges, color, width: mw, height: mh, weighted };
    }
    if (type === 3) {
      // LinkedMesh: inherits uvs, triangles, vertices, hull, edges from parent mesh
      const attPath = r.readStringRef() ?? name;
      const color = r.readColorHex();
      const parentSkinName = r.readStringRef() ?? "default";
      const parentName = r.readStringRef() ?? name;
      r.readBool(); // inheritDeform
      const mw = nonessential ? r.readFloat() : 32;
      const mh = nonessential ? r.readFloat() : 32;
      pendingLinkedMeshes.push({
        targetSkinIdx,
        slotName,
        key: placeholderName,
        name,
        path: attPath,
        color,
        parentSkinName,
        parentName,
        width: mw,
        height: mh,
        hasExplicitSize: nonessential,
      });
      return {
        type: "mesh",
        name,
        path: attPath,
        uvs: [],
        triangles: [],
        vertices: [],
        hull: 0,
        edges: [],
        color,
        width: mw,
        height: mh,
        weighted: false,
      };
    }
    if (type === 4) {
      const closed = r.readBool();
      const constantSpeed = r.readBool();
      const vertexCount = r.readVarint();
      const { vertices, weighted } = readVertices(r, vertexCount);
      const lengths: number[] = [];
      const lenCount = Math.floor(vertexCount / 3);
      for (let i = 0; i < lenCount; i++) lengths.push(r.readFloat());
      const color = nonessential ? r.readColorHex() : "ffffffff";
      return { type: "path", name, closed, constantSpeed, vertexCount, vertices, lengths, color, weighted };
    }
    if (type === 5) {
      const rotation = r.readFloat();
      const px = r.readFloat();
      const py = r.readFloat();
      const color = nonessential ? r.readColorHex() : "ffffffff";
      return { type: "point", name, x: px, y: py, rotation, color };
    }
    if (type === 6) {
      const endIdx = r.readVarint();
      const vertexCount = r.readVarint();
      const { vertices, weighted } = readVertices(r, vertexCount);
      const color = nonessential ? r.readColorHex() : "ffffffff";
      return {
        type: "clipping",
        name,
        end: endIdx < slots.length ? slots[endIdx].name : null,
        vertexCount,
        vertices,
        color,
        weighted,
      };
    }
    throw new Error(`Unsupported Spine 4.0 binary attachment type: ${type}`);
  };

  const readSkinSlotAttachments = (
    targetSkinIdx: number,
    slotEntryCount: number,
  ): Record<string, Record<string, Attachment>> => {
    const out: Record<string, Record<string, Attachment>> = {};
    for (let i = 0; i < slotEntryCount; i++) {
      const slIdx = r.readVarint();
      const slName = slots[slIdx]?.name ?? `slot_${slIdx}`;
      out[slName] = out[slName] ?? {};
      const attCount = r.readVarint();
      for (let j = 0; j < attCount; j++) {
        const key = r.readStringRef() ?? `att_${j}`;
        out[slName][key] = readAttachmentEntry(targetSkinIdx, slName, key);
      }
    }
    return out;
  };

  const skins: SkinData[] = [];
  const defaultSlotEntries = r.readVarint();
  if (defaultSlotEntries > 0) {
    skins.push({
      name: "default",
      bones: [],
      ik: [],
      transform: [],
      path: [],
      attachments: readSkinSlotAttachments(0, defaultSlotEntries),
    });
  }

  const namedSkinCount = r.readVarint();
  for (let i = 0; i < namedSkinCount; i++) {
    const name = r.readString() ?? `skin_${i}`;
    const sbCount = r.readVarint();
    const sb: string[] = [];
    for (let j = 0; j < sbCount; j++) sb.push(bones[r.readVarint()]?.name ?? "root");
    const sikCount = r.readVarint();
    const sik: string[] = [];
    for (let j = 0; j < sikCount; j++) sik.push(ik[r.readVarint()]?.name ?? "");
    const strCount = r.readVarint();
    const str: string[] = [];
    for (let j = 0; j < strCount; j++) str.push(transform[r.readVarint()]?.name ?? "");
    const spCount = r.readVarint();
    const sp: string[] = [];
    for (let j = 0; j < spCount; j++) sp.push(path[r.readVarint()]?.name ?? "");
    const sec = r.readVarint();
    const targetIdx = skins.length;
    skins.push({
      name,
      bones: sb,
      ik: sik,
      transform: str,
      path: sp,
      attachments: readSkinSlotAttachments(targetIdx, sec),
    });
  }

  for (const lm of pendingLinkedMeshes) {
    const parentSkin = skins.find((s) => s.name === lm.parentSkinName) ?? skins[0];
    const parentAtt = parentSkin?.attachments?.[lm.slotName]?.[lm.parentName];
    const dstSkin = skins[lm.targetSkinIdx];
    if (parentAtt && parentAtt.type === "mesh" && dstSkin?.attachments?.[lm.slotName]) {
      dstSkin.attachments[lm.slotName][lm.key] = {
        type: "mesh",
        name: lm.name,
        path: lm.path,
        uvs: [...parentAtt.uvs],
        triangles: [...parentAtt.triangles],
        vertices: [...parentAtt.vertices],
        hull: parentAtt.hull,
        edges: [...parentAtt.edges],
        color: lm.color,
        width: lm.hasExplicitSize ? lm.width : parentAtt.width,
        height: lm.hasExplicitSize ? lm.height : parentAtt.height,
        weighted: parentAtt.weighted,
      };
    }
  }

  // Events
  const eventCount = r.readVarint();
  const eventHasAudio: boolean[] = [];
  for (let i = 0; i < eventCount; i++) {
    r.readStringRef();
    r.readVarint(false);
    r.readFloat();
    r.readString();
    const audioPath = r.readString();
    const hasAudio = Boolean(audioPath);
    eventHasAudio.push(hasAudio);
    if (hasAudio) {
      r.readFloat();
      r.readFloat();
    }
  }

  // Animations
  const animations: Record<string, AnimationData> = {};
  const animCount = r.readVarint();
  for (let i = 0; i < animCount; i++) {
    const animName = r.readString() ?? `anim_${i}`;
    const slotsTl: Record<string, SlotTimelineGroup> = {};
    const slotTlCount = r.readVarint();
    for (let s = 0; s < slotTlCount; s++) {
      const slName = slots[r.readVarint()]?.name ?? `slot_${s}`;
      const tlCount = r.readVarint();
      const grp: SlotTimelineGroup = {};
      for (let t = 0; t < tlCount; t++) {
        const tlType = r.readByte();
        const kfCount = r.readVarint();
        if (tlType === 0) {
          grp.attachment = [];
          for (let k = 0; k < kfCount; k++) {
            grp.attachment.push({ time: r.readFloat(), name: r.readString() });
          }
        } else if (tlType === 1) {
          grp.rgba = [];
          for (let k = 0; k < kfCount; k++) {
            grp.rgba.push({ time: r.readFloat(), color: r.readColorHex(), curve: readCurve(r) });
          }
        }
      }
      slotsTl[slName] = grp;
    }

    const bonesTl: Record<string, BoneTimelineGroup> = {};
    const boneTlCount = r.readVarint();
    for (let b = 0; b < boneTlCount; b++) {
      const bnName = bones[r.readVarint()]?.name ?? `bone_${b}`;
      const tlCount = r.readVarint();
      const grp: BoneTimelineGroup = {};
      for (let t = 0; t < tlCount; t++) {
        const tlType = r.readByte();
        const kfCount = r.readVarint();
        if (tlType === 0) {
          grp.rotate = [];
          for (let k = 0; k < kfCount; k++) {
            grp.rotate.push({ time: r.readFloat(), value: r.readFloat(), curve: readCurve(r) });
          }
        } else if (tlType === 1) {
          grp.translate = [];
          for (let k = 0; k < kfCount; k++) {
            grp.translate.push({ time: r.readFloat(), x: r.readFloat(), y: r.readFloat(), curve: readCurve(r) });
          }
        } else if (tlType === 2) {
          grp.scale = [];
          for (let k = 0; k < kfCount; k++) {
            grp.scale.push({ time: r.readFloat(), x: r.readFloat(), y: r.readFloat(), curve: readCurve(r) });
          }
        } else if (tlType === 3) {
          grp.shear = [];
          for (let k = 0; k < kfCount; k++) {
            grp.shear.push({ time: r.readFloat(), x: r.readFloat(), y: r.readFloat(), curve: readCurve(r) });
          }
        }
      }
      bonesTl[bnName] = grp;
    }

    const ikTl: AnimationData["ik"] = {};
    const ikTlCount = r.readVarint();
    for (let j = 0; j < ikTlCount; j++) {
      const ikName = ik[r.readVarint()]?.name ?? `ik_${j}`;
      const kfCount = r.readVarint();
      ikTl[ikName] = [];
      for (let k = 0; k < kfCount; k++) {
        ikTl[ikName].push({
          time: r.readFloat(),
          mix: r.readFloat(),
          softness: r.readFloat(),
          bendPositive: r.readSignedByte() >= 0,
          compress: r.readBool(),
          stretch: r.readBool(),
          curve: readCurve(r),
        });
      }
    }

    const transformTl: AnimationData["transform"] = {};
    const trTlCount = r.readVarint();
    for (let j = 0; j < trTlCount; j++) {
      const trName = transform[r.readVarint()]?.name ?? `tr_${j}`;
      const kfCount = r.readVarint();
      transformTl[trName] = [];
      for (let k = 0; k < kfCount; k++) {
        transformTl[trName].push({
          time: r.readFloat(),
          mixRotate: r.readFloat(),
          mixX: r.readFloat(),
          mixY: r.readFloat(),
          mixScaleX: r.readFloat(),
          mixScaleY: r.readFloat(),
          mixShearY: r.readFloat(),
          curve: readCurve(r),
        });
      }
    }

    const pathTl: AnimationData["path"] = {};
    const pTlCount = r.readVarint();
    for (let j = 0; j < pTlCount; j++) {
      const pName = path[r.readVarint()]?.name ?? `path_${j}`;
      const tlCount = r.readVarint();
      const grp: AnimationData["path"][string] = {};
      for (let t = 0; t < tlCount; t++) {
        const tlType = r.readByte();
        const kfCount = r.readVarint();
        if (tlType === 0) {
          grp.position = [];
          for (let k = 0; k < kfCount; k++) {
            grp.position.push({ time: r.readFloat(), value: r.readFloat(), curve: readCurve(r) });
          }
        } else if (tlType === 1) {
          grp.spacing = [];
          for (let k = 0; k < kfCount; k++) {
            grp.spacing.push({ time: r.readFloat(), value: r.readFloat(), curve: readCurve(r) });
          }
        } else if (tlType === 2) {
          grp.mix = [];
          for (let k = 0; k < kfCount; k++) {
            grp.mix.push({
              time: r.readFloat(),
              mixRotate: r.readFloat(),
              mixX: r.readFloat(),
              mixY: r.readFloat(),
              curve: readCurve(r),
            });
          }
        }
      }
      pathTl[pName] = grp;
    }

    const deformTl: AnimationData["deform"] = {};
    const defCount = r.readVarint();
    for (let j = 0; j < defCount; j++) {
      const skName = skins[r.readVarint()]?.name ?? "default";
      const slName = slots[r.readVarint()]?.name ?? "slot_0";
      const atName = r.readStringRef() ?? "att_0";
      const kfCount = r.readVarint();
      deformTl[skName] = deformTl[skName] ?? {};
      deformTl[skName][slName] = deformTl[skName][slName] ?? {};
      deformTl[skName][slName][atName] = [];
      for (let k = 0; k < kfCount; k++) {
        const time = r.readFloat();
        const offset = r.readVarint();
        const vc = r.readVarint();
        const vertices: number[] = [];
        for (let v = 0; v < vc; v++) vertices.push(r.readFloat());
        deformTl[skName][slName][atName].push({ time, offset, vertices, curve: readCurve(r) });
      }
    }

    const doCount = r.readVarint();
    const drawOrder: AnimationData["drawOrder"] = [];
    for (let j = 0; j < doCount; j++) {
      const time = r.readFloat();
      const offCount = r.readVarint();
      const offsets: Array<{ slot: string; offset: number }> = [];
      for (let k = 0; k < offCount; k++) {
        offsets.push({ slot: slots[r.readVarint()]?.name ?? "", offset: r.readVarint(false) });
      }
      drawOrder.push({ time, offsets });
    }

    const evTlCount = r.readVarint();
    for (let j = 0; j < evTlCount; j++) {
      r.readFloat();
      const evIdx = r.readVarint();
      r.readVarint(false);
      r.readFloat();
      if (r.readBool()) r.readString();
      if (eventHasAudio[evIdx]) {
        r.readFloat();
        r.readFloat();
      }
    }

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
    header: {
      hash: rawHash,
      spine,
      x,
      y,
      width,
      height,
      fps,
      images,
      audio,
    },
    bones,
    slots,
    skins,
    ik,
    transform,
    path,
    animations,
  };
}

export const encodeSpine40StandardBinary = encodeStandardSpine40Skel;
export const decodeSpine40StandardBinary = parseStandardSpine40Skel;

