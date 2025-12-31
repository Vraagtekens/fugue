import { getPerformanceTier } from '../hooks/deviceDetection';
import { shaderMaterial } from '@react-three/drei/native';
import { extend } from '@react-three/fiber/native';
import * as THREE from 'three';

const textureLoader = new THREE.TextureLoader();
// const matcapTexture = textureLoader.load("/models/matcap_bubble.jpg");
const matcapTexture = textureLoader.load('../assets/models/matcap-white.jpg');
// const matcapTexture = textureLoader.load("/models/matcap_toon.png");

// Generate initial sphere positions closer to the center by reducing amplitude.
export function generateInitialSpherePositions(numSpheres: number) {
  return new Array(numSpheres).fill(0).map(
    () =>
      new THREE.Vector3(
        (Math.random() - 0.5) * 0.08, // x
        (Math.random() - 0.5) * 0.08, // y
        (Math.random() - 0.5) * 0.01 // z
      )
  );
}

// const initialSphereRadii = new Array(NUM_SPHERES).fill(0.07);

// // Store sphere positions in a ref so we can animate them.
// const spherePositions = useRef(initialSpherePositions);

// ========== IMPROVED CONFIGURATION ==========
const CONFIG = {
  // Improved colors for gradient effect
  BACKGROUND_TOP: new THREE.Vector3(0.1, 0.1, 0.2), // Dark blue
  BACKGROUND_BOTTOM: new THREE.Vector3(0.0, 0.0, 0.05), // Almost black

  // Gradient colors for metaballs (similar to your target shader)
  // GRADIENT_COLOR_1: new THREE.Vector3(0.082, 0.69, 0.972), // #15B0F8 - Light blue
  // GRADIENT_COLOR_2: new THREE.Vector3(0.784, 0.208, 0.972), // #C835F8 - Purple
  // GRADIENT_COLOR_3: new THREE.Vector3(0.31, 0.439, 0.933), // #4F70EE - Blue
  GRADIENT_COLOR_1: new THREE.Color('#a4e56d'), // #15B0F8 - Light blue
  GRADIENT_COLOR_2: new THREE.Color('#63cea7'), // #C835F8 - Purple
  GRADIENT_COLOR_3: new THREE.Color('#22b6e1'), // #4F70EE - Blue

  // Core metaball parameters
  METABALL_RADIUS: 0.07, // Larger radius for more prominent balls
  METABALL_COUNT: 10, // Default count (will be adjusted by performance)
  MOVEMENT_SPEED: 0.01, // Base movement speed
  MOVEMENT_RANGE: 0.5, // How far balls move from center
  BLEND_FACTOR: 0.1, // Lower value for smoother blending (was 0.25)
  THRESHOLD: 1.0, // Visibility threshold
  RIM_INTENSITY: 0.6, // Rim light effect to accentuate edges

  // Matcap
  MATCAP_TEXTURE: matcapTexture,

  // Performance settings by tier
  LOW_TIER: {
    METABALL_COUNT: 5,
    METABALL_QUALITY: 3.0, // Higher = less smooth but better performance
    FPS_TARGET: 30,
    RAY_STEPS: 32, // Fewer ray steps for low-end devices
    METABALLS_POS: generateInitialSpherePositions(10),
    METABALLS_RADIUS: new Array(5).fill(0.07),
  },
  MEDIUM_TIER: {
    METABALL_COUNT: 8,
    METABALL_QUALITY: 2.0,
    FPS_TARGET: 60,
    RAY_STEPS: 64, // Medium ray steps
    METABALLS_POS: generateInitialSpherePositions(10),
    METABALLS_RADIUS: new Array(8).fill(0.07),
  },
  HIGH_TIER: {
    METABALL_COUNT: 8,
    METABALL_QUALITY: 1.0,
    FPS_TARGET: 60,
    RAY_STEPS: 128, // More ray steps for high-end devices
    METABALLS_POS: generateInitialSpherePositions(10),
    METABALLS_RADIUS: new Array(10).fill(0.07),
  },
};

export interface MetaballShaderUniforms {
  uTime: number;
  uResolution: THREE.Vector2;
  uMouse: THREE.Vector2;
  uPerformanceTier: number; // 0: low, 1: medium, 2: high
  uMetaballCount: number; // Dynamic metaball count based on device
  uMetaballQuality: number; // Controls smoothness of the metaballs
  uMatcap: THREE.Texture;
  uRaySteps: number; // Number of ray marching steps
  uFPSTarget: number; // Target FPS to throttle animations if needed
  uBackgroundTop: THREE.Vector3; // Top color of background gradient
  uBackgroundBottom: THREE.Vector3; // Bottom color of background gradient
  uGradientColor1: THREE.Vector3; // First gradient color
  uGradientColor2: THREE.Vector3; // Second gradient color
  uGradientColor3: THREE.Vector3; // Third gradient color
  uEnablePointerInteraction: boolean; // Toggle mouse/touch interaction
  uMetaballRadius: number; // Base size of metaballs
  uMovementRange: number; // How far balls move from center
  uMovementSpeed: number; // Base movement speed
  uBlendFactor: number; // Controls how much balls blend together
  uThreshold: number; // Threshold for field value
  uRimIntensity: number; // Intensity of the rim lighting effect

  uSpherePositions: THREE.Vector3[];
  uSphereRadii: number[];
}

// Type for our custom shader material instance
export type MetaballShaderMaterialType = THREE.ShaderMaterial & MetaballShaderUniforms;

// Vertex shader
const vertexShader = /* glsl */ `
  precision highp float;
  
  uniform float uTime;
  uniform vec2 uResolution;
  
  varying vec2 vUv;
  
  void main() {
    vUv = uv;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

// Fragment shader with 3D raymarching similar to your target
const fragmentShader = /* glsl */ `
  precision highp float;
  // precision mediump float;
  
  #define PI 3.14159265359
  #define MAX_METABALLS 10
  
  uniform float uTime;
  uniform vec2 uResolution;
  uniform vec2 uMouse;
  uniform float uPerformanceTier;
  uniform float uMetaballCount;
  uniform float uMetaballQuality;
  uniform sampler2D uMatcap;
  uniform int uRaySteps;
  uniform float uFPSTarget;
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
  
  // Corrected UV for aspect ratio
  vec2 getCorrectedUV(vec2 uv, vec2 res) {
    vec2 centered = uv - 0.5;
    centered.x *= res.x / res.y;
    return centered + 0.5;
  }
  
  // Smooth min function for blending metaballs (from Inigo Quilez)
  float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
  }
  
  // Signed distance function for a sphere
  float sdSphere(vec3 p, float r) {
    return length(p) - r;
}
  
  // Scene SDF combining multiple spheres
  float sdf(vec3 p) {
    float d = 10000.0;

    for (int i = 0; i < MAX_METABALLS; i++) {
      if (i >= int(uMetaballCount)) break;
  
      float factor = mod(float(i), 2.0) < 1.0 ? 2.0 : 1.5;
      float sphereDist = sdSphere(p - uSpherePositions[i], 0.15 * factor);
      d = smin(d, sphereDist, 0.1);
    }

    if (uEnablePointerInteraction) {
        float mouseSphere = sdSphere(p - vec3(uMouse.x, uMouse.y, 0.0), 0.15);
        d = smin(mouseSphere, d, 0.1);
    }

    return d;
  }

  // Compute normal using gradient approximation
  vec3 calcNormal(vec3 p) {
      float eps = 0.0001;
      return normalize(vec3(
          sdf(p + vec3(eps, 0.0, 0.0)) - sdf(p - vec3(eps, 0.0, 0.0)),
          sdf(p + vec3(0.0, eps, 0.0)) - sdf(p - vec3(0.0, eps, 0.0)),
          sdf(p + vec3(0.0, 0.0, eps)) - sdf(p - vec3(0.0, 0.0, eps))
      ));
  }

  // Generate a pastel gradient background
  vec3 pastelGradient(vec2 uv) {
      vec3 color1 = vec3(1.0, 0.8, 0.9); // Soft pink
      vec3 color2 = vec3(0.8, 0.9, 1.0); // Soft blue
      vec3 color3 = vec3(1.0, 1.0, 0.8); // Soft yellow

      float mix1 = smoothstep(0.0, 1.0, uv.y);
      float mix2 = smoothstep(0.0, 1.0, uv.x);

      return mix(mix(color1, color2, mix1), color3, mix2);
  }

  // Generate a pastel color based on sphere position
  vec3 sphereColor(vec3 pos) {
      // Remap pos.xy from [-1,1] to [0,1]
      vec2 uv = pos.xy * 0.5 + 0.5;

      // Your custom colors:
      vec3 color1 = vec3(0.082, 0.690, 0.972); // #15B0F8
      vec3 color2 = vec3(0.784, 0.208, 0.972); // #C835F8
      vec3 color3 = vec3(0.310, 0.439, 0.933); // #4F70EE

      float mix1 = smoothstep(0.0, 1.0, uv.y);
      float mix2 = smoothstep(0.0, 1.0, uv.x);

      return mix(mix(color1, color2, mix1), color3, mix2);
  }

  vec3 getGradientColor(float t) {
      vec3 a = mix(uGradientColor1, uGradientColor2, t);
      return mix(a, uGradientColor3, t);
  }

  vec3 backgroundGradient(vec2 uv) {
      float t = clamp(uv.x, 0.0, 1.0);
      return getGradientColor(t);
  }

  vec3 sphereGradientColor(vec3 pos) {
      float t = clamp((pos.x + 1.0) * 0.5, 0.0, 1.0);
      return getGradientColor(t);
  }
  
  void main() {
    // Adjust UV for aspect ratio
    vec2 correctedUV = getCorrectedUV(vUv, uResolution);
    vec2 centeredUV = correctedUV - 0.5;
    
    // Setup camera and ray
    vec3 cameraPos = vec3(0.0, 0.0, 1.5);
    vec3 rayDir = normalize(vec3(centeredUV, -1.0));
    
    // Raymarch setup
    float t = 0.0;
    float tMax = 3.0; // Maximum ray distance
    
    for (int i = 0; i < 256; i++) {
      vec3 pos = cameraPos + t * rayDir;
      float h = sdf(pos);
      if (h < 0.00001 || t > tMax) break;
      t += h;
    }
    
    // Background: if no hit, output fully transparent and exit.
    if (t >= tMax) {
      gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
      return;
    }
    
    // Calculate hit position and normal
    vec3 pos = cameraPos + t * rayDir;
    vec3 normal = calcNormal(pos);
    vec2 matcapUV = normal.xy * 0.5 + 0.5;

    vec3 sphereMatcap = texture2D(uMatcap, matcapUV).rgb;
    vec3 spherePastel = sphereGradientColor(pos);

    // Adjust the mix factor if you want the metaballs to “pop” more.
    float mixFactor = 0.5; // Increase or decrease this value to boost contrast.
    vec3 color = mix(sphereMatcap, spherePastel, mixFactor);

    // Optionally, add a rim light effect to accentuate the edges:
    float rim = 1.0 - abs(dot(normal, normalize(rayDir)));
    color += rim * 0.6; // tweak the intensity as needed
    vec3 bg = backgroundGradient(centeredUV);
    vec3 color1 = mix(color, bg, 0.6);

    gl_FragColor = vec4(color1, 1.0);
  }
`;

// Create shader material using drei's shaderMaterial helper
const MetaballShaderMaterial = shaderMaterial(
  {
    uTime: 0,
    uResolution: new THREE.Vector2(1, 1),
    uMouse: new THREE.Vector2(0, 0),
    uMatcap: CONFIG.MATCAP_TEXTURE,
    uPerformanceTier:
      getPerformanceTier() === 'low' ? 0 : getPerformanceTier() === 'medium' ? 1 : 2,
    uMetaballCount:
      getPerformanceTier() === 'low'
        ? CONFIG.LOW_TIER.METABALL_COUNT
        : getPerformanceTier() === 'medium'
          ? CONFIG.MEDIUM_TIER.METABALL_COUNT
          : CONFIG.HIGH_TIER.METABALL_COUNT,
    uMetaballQuality:
      getPerformanceTier() === 'low'
        ? CONFIG.LOW_TIER.METABALL_QUALITY
        : getPerformanceTier() === 'medium'
          ? CONFIG.MEDIUM_TIER.METABALL_QUALITY
          : CONFIG.HIGH_TIER.METABALL_QUALITY,
    uRaySteps:
      getPerformanceTier() === 'low'
        ? CONFIG.LOW_TIER.RAY_STEPS
        : getPerformanceTier() === 'medium'
          ? CONFIG.MEDIUM_TIER.RAY_STEPS
          : CONFIG.HIGH_TIER.RAY_STEPS,
    uFPSTarget:
      getPerformanceTier() === 'low'
        ? CONFIG.LOW_TIER.FPS_TARGET
        : getPerformanceTier() === 'medium'
          ? CONFIG.MEDIUM_TIER.FPS_TARGET
          : CONFIG.HIGH_TIER.FPS_TARGET,

    uSpherePositions:
      getPerformanceTier() === 'low'
        ? CONFIG.LOW_TIER.METABALLS_POS
        : getPerformanceTier() === 'medium'
          ? CONFIG.MEDIUM_TIER.METABALLS_POS
          : CONFIG.HIGH_TIER.METABALLS_POS,
    uSphereRadii: new Float32Array(CONFIG.HIGH_TIER.METABALLS_RADIUS),

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

// Extend Three.js with our custom material
extend({ MetaballShaderMaterial });

// Export the material
export { MetaballShaderMaterial };
