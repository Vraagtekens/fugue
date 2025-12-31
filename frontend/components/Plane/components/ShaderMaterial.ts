import { shaderMaterial } from '@react-three/drei/native';
import { extend } from '@react-three/fiber/native';
import { Asset } from 'expo-asset';
import { getPerformanceTier } from '../hooks/deviceDetection';

// Matcap texture
const matcapTexture = Asset.fromModule(require('@/assets/models/matcap_toon.png'));

// Generate initial sphere positions (array of [x,y,z])
export function generateInitialSpherePositions(numSpheres: number) {
  return new Array(numSpheres).fill(0).map(() => [
    (Math.random() - 0.5) * 0.08, // x
    (Math.random() - 0.5) * 0.08, // y
    (Math.random() - 0.5) * 0.01, // z
  ]);
}

// Improved CONFIG without THREE
const CONFIG = {
  BACKGROUND_TOP: [0.1, 0.1, 0.2],
  BACKGROUND_BOTTOM: [0.0, 0.0, 0.05],

  GRADIENT_COLOR_1: [164 / 255, 229 / 255, 109 / 255], // #a4e56d
  GRADIENT_COLOR_2: [99 / 255, 206 / 255, 167 / 255], // #63cea7
  GRADIENT_COLOR_3: [34 / 255, 182 / 255, 225 / 255], // #22b6e1

  METABALL_RADIUS: 0.07,
  METABALL_COUNT: 10,
  MOVEMENT_SPEED: 0.01,
  MOVEMENT_RANGE: 0.25,
  BLEND_FACTOR: 0.1,
  THRESHOLD: 1.0,
  RIM_INTENSITY: 0.6,

  MATCAP_TEXTURE: matcapTexture,

  LOW_TIER: {
    METABALL_COUNT: 5,
    METABALL_QUALITY: 3.0,
    FPS_TARGET: 30,
    RAY_STEPS: 32,
    METABALLS_POS: generateInitialSpherePositions(5),
    METABALLS_RADIUS: new Array(5).fill(0.07),
  },
  MEDIUM_TIER: {
    METABALL_COUNT: 8,
    METABALL_QUALITY: 2.0,
    FPS_TARGET: 60,
    RAY_STEPS: 64,
    METABALLS_POS: generateInitialSpherePositions(8),
    METABALLS_RADIUS: new Array(8).fill(0.07),
  },
  HIGH_TIER: {
    METABALL_COUNT: 10,
    METABALL_QUALITY: 1.0,
    FPS_TARGET: 60,
    RAY_STEPS: 128,
    METABALLS_POS: generateInitialSpherePositions(10),
    METABALLS_RADIUS: new Array(10).fill(0.07),
  },
};

// Vertex shader
const vertexShader = /*glsl*/ `
  precision highp float;
  uniform float uTime;
  uniform vec2 uResolution;
  varying vec2 vUv;
  void main() {
    vUv = uv;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

// Fragment shader
const fragmentShader = /*glsl*/ `
  precision highp float;
  #define PI 3.14159265359
  #define MAX_METABALLS 10

  uniform float uTime;
  uniform vec2 uResolution;
  uniform vec2 uMouse;
  uniform float uMetaballCount;
  uniform float uMetaballQuality;
  uniform sampler2D uMatcap;
  uniform int uRaySteps;
  uniform vec3 uBackgroundTop;
  uniform vec3 uBackgroundBottom;
  uniform vec3 uGradientColor1;
  uniform vec3 uGradientColor2;
  uniform vec3 uGradientColor3;
  uniform bool uEnablePointerInteraction;
  uniform float uMetaballRadius;
  uniform float uMovementRange;
  uniform float uMovementSpeed;
  uniform float uBlendFactor;
  uniform float uThreshold;
  uniform float uRimIntensity;
  uniform vec3 uSpherePositions[MAX_METABALLS];
  uniform float uSphereRadii[MAX_METABALLS];

  varying vec2 vUv;

  vec2 getCorrectedUV(vec2 uv, vec2 res) {
    vec2 centered = uv - 0.5;
    centered.x *= res.x / res.y;
    return centered + 0.5;
  }

  float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5*(b-a)/k, 0.0, 1.0);
    return mix(b,a,h) - k*h*(1.0-h);
  }

  float sdSphere(vec3 p, float r) {
    return length(p) - r;
  }

  float sdf(vec3 p) {
    float d = 10000.0;
    for (int i=0; i<MAX_METABALLS; i++) {
      if(i >= int(uMetaballCount)) break;
      float sphereDist = sdSphere(p - uSpherePositions[i], uSphereRadii[i]);
      d = smin(d, sphereDist, 0.1);
    }
    return d;
  }

  vec3 calcNormal(vec3 p){
    float eps = 0.0001;
    return normalize(vec3(
      sdf(p + vec3(eps,0,0)) - sdf(p - vec3(eps,0,0)),
      sdf(p + vec3(0,eps,0)) - sdf(p - vec3(0,eps,0)),
      sdf(p + vec3(0,0,eps)) - sdf(p - vec3(0,0,eps))
    ));
  }

  vec3 getGradientColor(float t) {
    vec3 a = mix(uGradientColor1, uGradientColor2, t);
    return mix(a, uGradientColor3, t);
  }

  vec3 backgroundGradient(vec2 uv) {
    float t = clamp(uv.x, 0.0, 1.0);
    return getGradientColor(t);
  }

  void main() {
    vec2 correctedUV = getCorrectedUV(vUv, uResolution);
    vec2 centeredUV = correctedUV - 0.5;

    vec3 cameraPos = vec3(0.0, 0.0, 1.5);
    vec3 rayDir = normalize(vec3(centeredUV, -1.0));

    float t = 0.0;
    float tMax = 3.0;
    for(int i=0;i<256;i++){
      vec3 pos = cameraPos + t*rayDir;
      float h = sdf(pos);
      if(h < 0.00001 || t>tMax) break;
      t += h;
    }

    if(t>=tMax){
      gl_FragColor = vec4(0.0,0.0,0.0,0.0);
      return;
    }

    vec3 pos = cameraPos + t*rayDir;
    vec3 normal = calcNormal(pos);
    vec2 matcapUV = normal.xy*0.5+0.5;
    vec3 sphereMatcap = texture2D(uMatcap, matcapUV).rgb;
    vec3 bg = backgroundGradient(centeredUV);
    vec3 color = mix(sphereMatcap, bg, 0.6);

    gl_FragColor = vec4(color,1.0);
  }
`;

// Create shader material without THREE
const MetaballShaderMaterial = shaderMaterial(
  {
    uTime: 0,
    uResolution: [1, 1],
    uMouse: [0, 0],
    uMatcap: CONFIG.MATCAP_TEXTURE,
    uMetaballCount: CONFIG.METABALL_COUNT,
    uMetaballQuality: CONFIG.HIGH_TIER.METABALL_QUALITY,
    uRaySteps: CONFIG.HIGH_TIER.RAY_STEPS,
    uSpherePositions: CONFIG.HIGH_TIER.METABALLS_POS,
    uSphereRadii: CONFIG.HIGH_TIER.METABALLS_RADIUS,
    uBackgroundTop: CONFIG.BACKGROUND_TOP,
    uBackgroundBottom: CONFIG.BACKGROUND_BOTTOM,
    uGradientColor1: CONFIG.GRADIENT_COLOR_1,
    uGradientColor2: CONFIG.GRADIENT_COLOR_2,
    uGradientColor3: CONFIG.GRADIENT_COLOR_3,
    uEnablePointerInteraction: false,
    uMetaballRadius: CONFIG.METABALL_RADIUS,
    uMovementRange: CONFIG.MOVEMENT_RANGE,
    uMovementSpeed: CONFIG.MOVEMENT_SPEED,
    uBlendFactor: CONFIG.BLEND_FACTOR,
    uThreshold: CONFIG.THRESHOLD,
    uRimIntensity: CONFIG.RIM_INTENSITY,
  },
  vertexShader,
  fragmentShader
);

// Register material with R3F
extend({ MetaballShaderMaterial });

export { MetaballShaderMaterial };
