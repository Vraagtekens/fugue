import React, { useRef, useEffect, useState } from 'react';
import { extend, useFrame } from '@react-three/fiber/native';
import * as THREE from 'three';
import { isMobile, getPerformanceTier } from '../hooks/deviceDetection';
import { generateInitialSpherePositions, MetaballShaderMaterial } from './ShaderMaterial';
import { shaderMaterial } from '@react-three/drei/native';

// export const MetaballShaderMaterial = shaderMaterial(
//   {
//     uTime: 0,
//     uResolution: new THREE.Vector2(),
//     uMouse: new THREE.Vector2(),
//     uSpherePositions: Array.from({ length: 10 }, () => new THREE.Vector3()),
//     uMetaballCount: 10,
//     uMovementRange: 0.5,
//   },
//   `void main() { gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }`,
//   `void main() { gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0); }`
// );

// extend({ MetaballShaderMaterial });

interface MetaballShaderPlaneProps {
  size: {
    width: number;
    height: number;
    aspect: number;
  };
  mousePosition?: {
    normalized: {
      x: number;
      y: number;
    };
  };
  isVisible?: boolean;
  settings?: {
    quality: number;
    metaballCount: number;
    fps: number;
    temperatureStatus: 'normal' | 'warm' | 'hot';
  };
}

const NUM_SPHERES = 3;

export const MetaballShaderPlane: React.FC<MetaballShaderPlaneProps> = ({
  size,
  mousePosition = { normalized: { x: 0, y: 0 } },
  isVisible = true,
  settings = {
    quality: 1,
    metaballCount: 10,
    fps: 60,
    temperatureStatus: 'normal',
  },
}) => {
  const materialRef = useRef<MetaballShaderMaterialType>(null);
  const basePositions = useRef<THREE.Vector3[]>(generateInitialSpherePositions(NUM_SPHERES));

  const smoothedMouseRef = useRef({ x: 0, y: 0 });
  const fpsHistoryRef = useRef<number[]>([]);
  const [initialized, setInitialized] = useState(false);

  /* -------------------- one-time material init -------------------- */
  useEffect(() => {
    if (!materialRef.current || initialized) return;

    const mobile = isMobile();
    const tier = getPerformanceTier();

    const mat = materialRef.current;

    mat.uPerformanceTier = tier === 'low' ? 0 : tier === 'medium' ? 1 : 2;
    mat.uFPSTarget = mobile ? 30 : 60;

    mat.uMetaballCount = 3;
    mat.uRaySteps = tier === 'low' ? 32 : tier === 'medium' ? 64 : 96;
    mat.uMetaballQuality = tier === 'low' ? 3.0 : tier === 'medium' ? 2.0 : 1.0;
    mat.uMetaballRadius = mobile ? 0.15 : 0.2;

    mat.uEnablePointerInteraction = false;
    mat.uMovementSpeed = 2.0;

    setInitialized(true);
  }, [initialized]);

  /* -------------------- FPS smoothing -------------------- */
  useEffect(() => {
    fpsHistoryRef.current.push(settings.fps);
    if (fpsHistoryRef.current.length > 10) fpsHistoryRef.current.shift();
  }, [settings.fps]);

  /* -------------------- thermal throttling -------------------- */
  useEffect(() => {
    if (!materialRef.current || !initialized) return;

    if (settings.temperatureStatus === 'hot') {
      materialRef.current.uMovementSpeed *= 0.5;
      materialRef.current.uEnablePointerInteraction = false;
    }
  }, [settings.temperatureStatus, initialized]);

  /* -------------------- animation loop -------------------- */
  useFrame(({ clock }, delta) => {
    const mat = materialRef.current;
    if (!mat) return;

    /* ---- smooth pointer ---- */
    const targetX = mousePosition.normalized.x;
    const targetY = mousePosition.normalized.y;

    const smoothing = 10;
    const sx = smoothedMouseRef.current.x;
    const sy = smoothedMouseRef.current.y;

    smoothedMouseRef.current.x = sx + (targetX - sx) * Math.min(1, smoothing * delta);
    smoothedMouseRef.current.y = sy + (targetY - sy) * Math.min(1, smoothing * delta);

    mat.uniforms.uMouse.value.set(smoothedMouseRef.current.x, -smoothedMouseRef.current.y);

    /* ---- time ---- */
    mat.uTime = clock.getElapsedTime() * 0.4;

    /* ---- resolution update (rare) ---- */
    if (
      Math.abs(mat.uResolution.x - size.width) > 1 ||
      Math.abs(mat.uResolution.y - size.height) > 1
    ) {
      mat.uResolution.set(size.width, size.height);
    }

    /* ---- metaball motion ---- */
    const positions = mat.uniforms.uSpherePositions.value;
    const count = Math.min(mat.uniforms.uMetaballCount.value, basePositions.current.length);
    const range = mat.uniforms.uMovementRange.value;
    const t = clock.getElapsedTime() * 0.15;

    for (let i = 0; i < count; i++) {
      const base = basePositions.current[i];
      const phase = i * 1.234;
      const dir = i % 2 === 0 ? 1 : -1;

      const x = Math.sin(t + phase) * range * 2.0 * dir;
      const y = Math.cos(t * 1.3 + phase * 1.1) * range * 1.2;
      const z = Math.sin(t * 0.8 + phase * 0.9) * range * 0.2;

      positions[i].set(base.x + x, base.y + y, base.z + z);
    }
  });

  return (
    <mesh>
      <planeGeometry args={[size.aspect, 1, 1, 1]} />
      {/* <metaballShaderMaterial ref={materialRef} /> */}
      {/* <meshBasicMaterial color="hotpink" /> */}
      <metaballShaderMaterial ref={materialRef} />
    </mesh>
  );
};
