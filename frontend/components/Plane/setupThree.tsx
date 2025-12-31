// src/setupThree.ts
import * as THREE from 'three';

// attach single instance to global
if (!(global as any).THREE) {
  (global as any).THREE = THREE;
}

export { THREE };
