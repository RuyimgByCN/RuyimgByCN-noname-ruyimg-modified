/**
 * Clean-room Spine 4.0 data structures and runtime types (Decision 35).
 *
 * Implemented strictly from the public Spine 4.0 JSON & binary format
 * specifications and upstream `dynamicSkin.js` / `saveSkinParams` parameter
 * conventions. No official Spine runtime source code or upstream-bundled
 * `spine.js` / `spine-lib/*` / `spine-pixi-v8.js` was read or copied.
 */

export type TransformMode =
  | "normal"
  | "onlyTranslation"
  | "noRotationOrReflection"
  | "noScale"
  | "noScaleOrReflection";

export type BlendMode = "normal" | "additive" | "multiply" | "screen";

export type PositionMode = "fixed" | "percent";
export type SpacingMode = "length" | "fixed" | "percent" | "proportional";
export type RotateMode = "tangent" | "chain" | "chainScale";
export type MixBlend = "setup" | "first" | "replace" | "add";

export interface SpineSkeletonHeader {
  hash: string;
  spine: string;
  x: number;
  y: number;
  width: number;
  height: number;
  fps: number;
  images?: string;
  audio?: string;
}

export interface BoneData {
  name: string;
  parent: string | null;
  length: number;
  transform: TransformMode;
  skin: boolean;
  x: number;
  y: number;
  rotation: number;
  scaleX: number;
  scaleY: number;
  shearX: number;
  shearY: number;
}

export interface SlotData {
  name: string;
  bone: string;
  color: string;
  dark: string | null;
  attachment: string | null;
  blend: BlendMode;
}

export interface RegionAttachment {
  type: "region";
  name: string;
  path: string;
  x: number;
  y: number;
  scaleX: number;
  scaleY: number;
  rotation: number;
  width: number;
  height: number;
  color: string;
}

export interface MeshAttachment {
  type: "mesh";
  name: string;
  path: string;
  uvs: number[];
  triangles: number[];
  /**
   * If `weighted` is false, `vertices` has length `uvs.length` (`[x0, y0, x1, y1, ...]`).
   * If `weighted` is true, `vertices` follows the Spine 4.0 weighted encoding:
   * `[boneCount, boneIndex, bindX, bindY, weight, ...]`.
   */
  vertices: number[];
  hull: number;
  edges: number[];
  color: string;
  width: number;
  height: number;
  weighted: boolean;
}

export interface ClippingAttachment {
  type: "clipping";
  name: string;
  end: string | null;
  vertexCount: number;
  vertices: number[];
  color: string;
  weighted: boolean;
}

export interface PathAttachment {
  type: "path";
  name: string;
  closed: boolean;
  constantSpeed: boolean;
  vertexCount: number;
  vertices: number[];
  lengths: number[];
  color: string;
  weighted: boolean;
}

export interface BoundingBoxAttachment {
  type: "boundingbox";
  name: string;
  vertexCount: number;
  vertices: number[];
  color: string;
  weighted: boolean;
}

export interface PointAttachment {
  type: "point";
  name: string;
  x: number;
  y: number;
  rotation: number;
  color: string;
}

export type Attachment =
  | RegionAttachment
  | MeshAttachment
  | ClippingAttachment
  | PathAttachment
  | BoundingBoxAttachment
  | PointAttachment;

export interface SkinData {
  name: string;
  bones: string[];
  ik: string[];
  transform: string[];
  path: string[];
  attachments: Record<string, Record<string, Attachment>>;
}

export interface IkConstraintData {
  name: string;
  order: number;
  skin: boolean;
  bones: string[];
  target: string;
  mix: number;
  softness: number;
  bendPositive: boolean;
  compress: boolean;
  stretch: boolean;
  uniform: boolean;
}

export interface TransformConstraintData {
  name: string;
  order: number;
  skin: boolean;
  bones: string[];
  target: string;
  local: boolean;
  relative: boolean;
  offsetRotation: number;
  offsetX: number;
  offsetY: number;
  offsetScaleX: number;
  offsetScaleY: number;
  offsetShearY: number;
  mixRotate: number;
  mixX: number;
  mixY: number;
  mixScaleX: number;
  mixScaleY: number;
  mixShearY: number;
}

export interface PathConstraintData {
  name: string;
  order: number;
  skin: boolean;
  bones: string[];
  target: string;
  positionMode: PositionMode;
  spacingMode: SpacingMode;
  rotateMode: RotateMode;
  offsetRotation: number;
  position: number;
  spacing: number;
  mixRotate: number;
  mixX: number;
  mixY: number;
}

/**
 * Spine 4.0 curve definition:
 * - `"linear"` (default)
 * - `"stepped"`
 * - `[cx1, cy1, cx2, cy2]` (or 8 numbers for 2D timelines `[cx1, cy1, cx2, cy2, cx3, cy3, cx4, cy4]`)
 *   where `cx1, cx2` are in keyframe time units and `cy1, cy2` are in value units.
 */
export type CurveDef = "linear" | "stepped" | number[];

export interface ScalarKeyframe {
  time: number;
  value: number;
  curve?: CurveDef;
}

export interface PairKeyframe {
  time: number;
  x: number;
  y: number;
  curve?: CurveDef;
}

export interface ColorKeyframe {
  time: number;
  color: string;
  curve?: CurveDef;
}

export interface AttachmentKeyframe {
  time: number;
  name: string | null;
}

export interface IkKeyframe {
  time: number;
  mix: number;
  softness: number;
  bendPositive: boolean;
  compress: boolean;
  stretch: boolean;
  curve?: CurveDef;
}

export interface TransformKeyframe {
  time: number;
  mixRotate: number;
  mixX: number;
  mixY: number;
  mixScaleX: number;
  mixScaleY: number;
  mixShearY: number;
  curve?: CurveDef;
}

export interface PathKeyframe {
  time: number;
  value: number;
  curve?: CurveDef;
}

export interface DeformKeyframe {
  time: number;
  offset: number;
  vertices: number[];
  curve?: CurveDef;
}

export interface DrawOrderKeyframe {
  time: number;
  offsets: Array<{ slot: string; offset: number }>;
}

export interface BoneTimelineGroup {
  rotate?: ScalarKeyframe[];
  translate?: PairKeyframe[];
  scale?: PairKeyframe[];
  shear?: PairKeyframe[];
}

export interface SlotTimelineGroup {
  attachment?: AttachmentKeyframe[];
  rgba?: ColorKeyframe[];
}

export interface PathTimelineGroup {
  position?: PathKeyframe[];
  spacing?: PathKeyframe[];
  mix?: Array<{ time: number; mixRotate: number; mixX: number; mixY: number; curve?: CurveDef }>;
}

export interface AnimationData {
  name: string;
  bones: Record<string, BoneTimelineGroup>;
  slots: Record<string, SlotTimelineGroup>;
  ik: Record<string, IkKeyframe[]>;
  transform: Record<string, TransformKeyframe[]>;
  path: Record<string, PathTimelineGroup>;
  deform: Record<string, Record<string, Record<string, DeformKeyframe[]>>>;
  drawOrder: DrawOrderKeyframe[];
}

export interface SkeletonData {
  header: SpineSkeletonHeader;
  bones: BoneData[];
  slots: SlotData[];
  skins: SkinData[];
  ik: IkConstraintData[];
  transform: TransformConstraintData[];
  path: PathConstraintData[];
  animations: Record<string, AnimationData>;
}

export interface AtlasPage {
  name: string;
  width: number;
  height: number;
  format: string;
  minFilter: string;
  magFilter: string;
  repeat: string;
  pma: boolean;
}

export interface AtlasRegion {
  name: string;
  page: string;
  x: number;
  y: number;
  width: number;
  height: number;
  offsetX: number;
  offsetY: number;
  originalWidth: number;
  originalHeight: number;
  rotate: number;
  index: number;
  u: number;
  v: number;
  u2: number;
  v2: number;
}

export interface TextureAtlas {
  pages: AtlasPage[];
  regions: Record<string, AtlasRegion>;
}

export interface DynamicSkinParams {
  generalId: string;
  skinName: string;
  directory: string;
  skelFile: string;
  atlasFile: string;
  version?: string;
  x?: number | [number, number];
  y?: number | [number, number];
  scale?: number;
  angle?: number;
  speed?: number;
  action?: string;
  background?: string;
  alpha?: number;
  flipX?: boolean;
  clipSlots?: string[];
  hideSlots?: string[];
}
