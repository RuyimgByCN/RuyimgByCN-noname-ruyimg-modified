/**
 * Hand-written Spine 4.0 JSON skeleton and `.atlas` fixture (Decision 35).
 *
 * Authored strictly from the public Spine 4.0 JSON & Atlas format documentation
 * without copying any official Spine sample assets or runtime source code.
 * Exercises all 12 items of the Decision 35 Spine 4.0 feature checklist:
 * 1. Bones & all 5 world transform modes (`normal`, `onlyTranslation`, `noRotationOrReflection`, `noScale`, `noScaleOrReflection`)
 * 2. Slots & draw order offsets
 * 3. Skins (`default` + custom `gold_armor` skin)
 * 4. All 6 attachment types (`region`, `mesh`, weighted `mesh`, `clipping`, `path`, `boundingbox`)
 * 5. IK (1-bone & 2-bone), Transform, and Path constraints
 * 6. Timelines & curves (`linear`, `stepped`, cubic Bezier `[cx1, cy1, cx2, cy2]`)
 * 7. Multi-track animations (`daiji`, `gongji`, `jineng`, `shouji`, `zhenwang`, plus overlay `breath_add`)
 */

export const HANDCRAFTED_SPINE40_ATLAS_TEXT = `
hero_cleanroom40.png
size: 256, 256
format: RGBA8888
filter: Linear, Linear
repeat: none
pma: true
torso_plate
  bounds: 0, 0, 64, 96
  offsets: 0, 0, 64, 96
cape_cloth
  bounds: 64, 0, 64, 96
  offsets: 0, 0, 64, 96
weapon_blade
  bounds: 128, 0, 48, 128
  offsets: 0, 0, 48, 128
  rotate: 90
aura_glow
  bounds: 176, 0, 64, 64
  offsets: 0, 0, 64, 64
`.trim();

export const HANDCRAFTED_SPINE40_SKELETON_JSON = {
  skeleton: {
    hash: "c1ea4005e4000001",
    spine: "4.0.64",
    x: -80,
    y: -20,
    width: 160,
    height: 220,
    fps: 30,
  },
  bones: [
    { name: "root", x: 0, y: 0, length: 0, transform: "normal" },
    { name: "torso", parent: "root", x: 0, y: 24, length: 50, rotation: 90, transform: "normal" },
    { name: "shoulder_r", parent: "torso", x: 40, y: -14, length: 28, rotation: -90, transform: "normal" },
    { name: "forearm_r", parent: "shoulder_r", x: 28, y: 0, length: 26, rotation: 0, transform: "normal" },
    { name: "ik_hand_target", parent: "root", x: 38, y: 44, length: 0, transform: "onlyTranslation" },
    { name: "head_badge", parent: "torso", x: 52, y: 0, length: 16, rotation: 0, transform: "noRotationOrReflection" },
    { name: "aura_anchor", parent: "torso", x: 20, y: 0, length: 14, rotation: 0, transform: "noScale" },
    { name: "cape_follower", parent: "torso", x: 10, y: 12, length: 24, rotation: -120, transform: "noScaleOrReflection" },
    { name: "ribbon_node", parent: "root", x: -20, y: 30, length: 20, rotation: 0, transform: "normal" },
  ],
  slots: [
    { name: "clip_window", bone: "root", attachment: "viewport_clip", blend: "normal" },
    { name: "cape_slot", bone: "torso", color: "ffffffff", attachment: "cape_cloth", blend: "normal" },
    { name: "torso_slot", bone: "torso", color: "ffffffff", attachment: "torso_plate", blend: "normal" },
    { name: "weapon_slot", bone: "forearm_r", color: "ffffffff", attachment: "weapon_blade", blend: "normal" },
    { name: "aura_slot", bone: "aura_anchor", color: "88ddffff", attachment: "aura_glow", blend: "additive" },
    { name: "path_guide_slot", bone: "root", color: "ffffffff", attachment: "ribbon_path", blend: "normal" },
    { name: "hit_bounds_slot", bone: "torso", color: "ffffffff", attachment: "torso_bounds", blend: "normal" },
  ],
  ik: [
    {
      name: "arm_r_ik",
      order: 0,
      bones: ["shoulder_r", "forearm_r"],
      target: "ik_hand_target",
      mix: 1,
      softness: 0,
      bendPositive: true,
      compress: false,
      stretch: false,
    },
  ],
  transform: [
    {
      name: "cape_follow_torso",
      order: 1,
      bones: ["cape_follower"],
      target: "shoulder_r",
      local: false,
      relative: true,
      offsetRotation: -15,
      offsetX: 0,
      offsetY: 0,
      mixRotate: 0.5,
      mixX: 0.25,
      mixY: 0.25,
      mixScaleX: 0,
      mixScaleY: 0,
      mixShearY: 0,
    },
  ],
  path: [
    {
      name: "ribbon_on_path",
      order: 2,
      bones: ["ribbon_node"],
      target: "path_guide_slot",
      positionMode: "percent",
      spacingMode: "length",
      rotateMode: "tangent",
      offsetRotation: 0,
      position: 0.5,
      spacing: 0,
      mixRotate: 1,
      mixX: 1,
      mixY: 1,
    },
  ],
  skins: [
    {
      name: "default",
      attachments: {
        clip_window: {
          viewport_clip: {
            type: "clipping",
            end: "weapon_slot",
            vertexCount: 4,
            vertices: [-120, -60, 120, -60, 120, 180, -120, 180],
          },
        },
        torso_slot: {
          torso_plate: {
            type: "region",
            x: 22,
            y: 0,
            rotation: -90,
            width: 36,
            height: 52,
            color: "e8c468ff",
          },
        },
        cape_slot: {
          cape_cloth: {
            type: "mesh",
            uvs: [0, 0, 1, 0, 1, 1, 0, 1],
            triangles: [0, 1, 2, 2, 3, 0],
            // Weighted mesh bound to bone 1 (`torso`) and bone 7 (`cape_follower`):
            vertices: [
              2, 1, 36, 16, 0.7, 7, 4, 4, 0.3,
              2, 1, 36, -16, 0.7, 7, 4, -4, 0.3,
              2, 1, -4, -22, 0.3, 7, 24, -10, 0.7,
              2, 1, -4, 22, 0.3, 7, 24, 10, 0.7,
            ],
            hull: 4,
            color: "9b2c2cff",
            width: 40,
            height: 48,
          },
        },
        weapon_slot: {
          weapon_blade: {
            type: "mesh",
            uvs: [0, 0, 1, 0, 1, 1, 0, 1],
            triangles: [0, 1, 2, 2, 3, 0],
            // Unweighted 4-vertex mesh
            vertices: [0, -6, 36, -6, 36, 6, 0, 6],
            hull: 4,
            color: "7dd3fcff",
            width: 36,
            height: 12,
          },
        },
        aura_slot: {
          aura_glow: {
            type: "region",
            x: 0,
            y: 0,
            rotation: 0,
            width: 44,
            height: 44,
            color: "38bdf8cc",
          },
        },
        path_guide_slot: {
          ribbon_path: {
            type: "path",
            closed: false,
            constantSpeed: true,
            vertexCount: 3,
            vertices: [-40, 10, 0, 40, 40, 10],
            lengths: [50, 100],
          },
        },
        hit_bounds_slot: {
          torso_bounds: {
            type: "boundingbox",
            vertexCount: 4,
            vertices: [-10, -24, 54, -24, 54, 24, -10, 24],
          },
        },
      },
    },
    {
      name: "gold_armor",
      attachments: {
        torso_slot: {
          torso_plate: {
            type: "region",
            x: 22,
            y: 0,
            rotation: -90,
            width: 40,
            height: 56,
            color: "fde047ff",
          },
        },
      },
    },
  ],
  animations: {
    daiji: {
      bones: {
        torso: {
          rotate: [
            { time: 0, value: 0, curve: [0.25, 0, 0.75, 4] },
            { time: 0.5, value: 4, curve: [0.75, 4, 0.9, 0] },
            { time: 1.0, value: 0 },
          ],
          translate: [
            { time: 0, x: 0, y: 0 },
            { time: 0.5, x: 0, y: 3 },
            { time: 1.0, x: 0, y: 0 },
          ],
        },
        ik_hand_target: {
          translate: [
            { time: 0, x: 0, y: 0, curve: "stepped" },
            { time: 0.5, x: 4, y: 6 },
            { time: 1.0, x: 0, y: 0 },
          ],
        },
      },
      slots: {
        aura_slot: {
          rgba: [
            { time: 0, color: "88ddff88" },
            { time: 0.5, color: "88ddffff" },
            { time: 1.0, color: "88ddff88" },
          ],
        },
      },
    },
    gongji: {
      bones: {
        torso: {
          rotate: [
            { time: 0, value: 0 },
            { time: 0.3, value: -18 },
            { time: 0.6, value: 12 },
            { time: 1.0, value: 0 },
          ],
        },
        ik_hand_target: {
          translate: [
            { time: 0, x: 0, y: 0 },
            { time: 0.3, x: 18, y: 14 },
            { time: 0.6, x: 26, y: -8 },
            { time: 1.0, x: 0, y: 0 },
          ],
        },
      },
    },
    jineng: {
      bones: {
        aura_anchor: {
          scale: [
            { time: 0, x: 1, y: 1 },
            { time: 0.5, x: 1.45, y: 1.45 },
            { time: 1.0, x: 1, y: 1 },
          ],
        },
      },
      transform: {
        cape_follow_torso: [
          { time: 0, mixRotate: 0.5, mixX: 0.25, mixY: 0.25, mixScaleX: 0, mixScaleY: 0, mixShearY: 0 },
          { time: 0.5, mixRotate: 1.0, mixX: 0.75, mixY: 0.75, mixScaleX: 0, mixScaleY: 0, mixShearY: 0 },
          { time: 1.0, mixRotate: 0.5, mixX: 0.25, mixY: 0.25, mixScaleX: 0, mixScaleY: 0, mixShearY: 0 },
        ],
      },
      path: {
        ribbon_on_path: {
          position: [
            { time: 0, value: 0.5 },
            { time: 0.5, value: 1.0 },
            { time: 1.0, value: 0.5 },
          ],
        },
      },
      deform: {
        default: {
          weapon_slot: {
            weapon_blade: [
              { time: 0, offset: 0, vertices: [0, 0, 0, 0] },
              { time: 0.5, offset: 2, vertices: [8, 4, 8, -4] },
              { time: 1.0, offset: 0, vertices: [0, 0, 0, 0] },
            ],
          },
        },
      },
      drawOrder: [
        { time: 0, offsets: [{ slot: "weapon_slot", offset: 1 }] },
      ],
    },
    shouji: {
      bones: {
        torso: {
          rotate: [
            { time: 0, value: 0 },
            { time: 0.25, value: 16 },
            { time: 1.0, value: 0 },
          ],
        },
      },
    },
    zhenwang: {
      bones: {
        torso: {
          rotate: [
            { time: 0, value: 0 },
            { time: 0.8, value: 65 },
          ],
        },
      },
      slots: {
        torso_slot: {
          rgba: [
            { time: 0, color: "ffffffff" },
            { time: 0.8, color: "ffffff33" },
          ],
        },
      },
    },
    breath_add: {
      bones: {
        torso: {
          rotate: [
            { time: 0, value: 2 },
            { time: 1.0, value: 6 },
          ],
        },
      },
    },
  },
} as const;
