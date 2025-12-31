import React, { useRef, useEffect, useState } from 'react';
import { useFrame, useThree } from '@react-three/fiber/native';
import { isMobile, getPerformanceTier } from '../hooks/deviceDetection';
import { generateInitialSpherePositions, MetaballShaderMaterialType } from './ShaderMaterial';
import { THREE } from '@/components/Plane/setupThree';

interface MetaballShaderPlaneProps {
  size: {
    width: number;
    height: number;
    aspect: number;
  };
  mousePosition: {
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

const NUM_SPHERES = 10;

export const MetaballShaderPlane: React.FC<MetaballShaderPlaneProps> = ({
  size,
  mousePosition,
  isVisible = true,
  settings = {
    quality: 1.0,
    metaballCount: 10,
    fps: 60,
    temperatureStatus: 'normal',
  },
}) => {
  // Initialize ref for material
  const materialRef = useRef<MetaballShaderMaterialType>(null);
  const [pos, setPos] = useState(new THREE.Vector3(0, 0, 0));

  const basePositions = useRef<THREE.Vector3[]>(generateInitialSpherePositions(NUM_SPHERES));

  // Track initialization state
  const [initialized, setInitialized] = useState(false);

  // Smoothed FPS values for stability
  const [smoothedFps, setSmoothedFps] = useState(settings.fps);
  const fpsHistoryRef = useRef<number[]>([]);

  // Smoothed mouse position for stability
  const smoothedMouseRef = useRef({ x: 0, y: 0 });

  // Initialize material with fixed settings once
  useEffect(() => {
    if (!materialRef.current || initialized) return;

    // Get device capabilities once
    const isMobileDevice = isMobile();
    const performanceTier = getPerformanceTier();

    // Set initial parameters once and avoid changing them frequently
    if (materialRef.current) {
      materialRef.current.uPerformanceTier =
        performanceTier === 'low' ? 0 : performanceTier === 'medium' ? 1 : 2;

      // Set FPS target once
      materialRef.current.uFPSTarget = isMobileDevice ? 30 : 60;

      // Set metaball count once based on tier - don't change it
      materialRef.current.uMetaballCount =
        performanceTier === 'low' ? 5 : performanceTier === 'medium' ? 8 : 10;

      // Set raymarching steps once
      materialRef.current.uRaySteps =
        performanceTier === 'low' ? 32 : performanceTier === 'medium' ? 64 : 96;

      // Set metaball quality once
      materialRef.current.uMetaballQuality =
        performanceTier === 'low' ? 3.0 : performanceTier === 'medium' ? 2.0 : 1.0;

      // Set metaball radius once
      materialRef.current.uMetaballRadius = isMobileDevice ? 0.15 : 0.2;

      // Pointer interaction (touch/mouse)
      // materialRef.current.uEnablePointerInteraction =
      //   !isMobileDevice || performanceTier === "high";
      materialRef.current.uEnablePointerInteraction = false;

      // Movement speed based on device
      // materialRef.current.uMovementSpeed =
      //   performanceTier === 'low' ? 0.2 : performanceTier === 'medium' ? 0.3 : 0.6
      materialRef.current.uMovementSpeed = 2.0;
    }

    setInitialized(true);
  }, [initialized]);

  // Update smooth FPS value
  useEffect(() => {
    // Keep a history of the last 10 FPS readings
    fpsHistoryRef.current.push(settings.fps);
    if (fpsHistoryRef.current.length > 10) {
      fpsHistoryRef.current.shift();
    }

    // Calculate average (without extremes for stability)
    if (fpsHistoryRef.current.length >= 3) {
      const sorted = [...fpsHistoryRef.current].sort((a, b) => a - b);
      // Remove highest and lowest values to get more stable average
      const trimmed = sorted.slice(1, sorted.length - 1);
      const avg = trimmed.reduce((sum, val) => sum + val, 0) / trimmed.length;
      setSmoothedFps(avg);
    }
  }, [settings.fps]);

  // Only handle visibility change (minimize expensive operations)
  useEffect(() => {
    if (!materialRef.current) return;

    // // Only update very limited properties when visibility changes
    // if (!isVisible) {
    //   // When not visible, pause animations by setting time to 0
    //   // No need to update other parameters as they won't be seen
    //   // setPos(new THREE.Vector3(0, 0.5, 0));
    //   materialRef.current.uMetaballCount = 1;
    // } else {
    //   // setPos(new THREE.Vector3(0, 0.0, 0));
    //   materialRef.current.uMetaballCount = 10;
    // }
  }, [isVisible]);

  // Handle extreme device temperature
  useEffect(() => {
    if (!materialRef.current || !initialized) return;

    // Only update if device is overheating - otherwise maintain stable settings
    if (settings.temperatureStatus === 'hot') {
      // Reduce movement speed to reduce calculations
      materialRef.current.uMovementSpeed *= 0.5;
      // Disable interaction to reduce calculations
      materialRef.current.uEnablePointerInteraction = false;
    }
  }, [settings.temperatureStatus, initialized]);

  const { size: canvasSize } = useThree();
  // Animation with fixed frame times to prevent jitter
  useFrame(({ clock }, delta) => {
    if (!materialRef.current) return;

    // 1. Compute your raw mouse target
    const targetX = mousePosition.normalized.x;
    const targetY = mousePosition.normalized.y;

    const normalized = {
      x: (targetX / size.width) * 2 - 1,
      y: -((targetY / size.height) * 2 - 1),
    };

    // 2. Do a delta-scaled lerp
    const smoothing = 10; // higher = snappier
    const sx = smoothedMouseRef.current.x;
    const sy = smoothedMouseRef.current.y;
    const nx = sx + (normalized.x - sx) * Math.min(1, smoothing * delta);
    const ny = sy + (normalized.y - sy) * Math.min(1, smoothing * delta);

    // 3. Snap if really close
    smoothedMouseRef.current.x = Math.abs(targetX - nx) < 0.001 ? targetX : nx;
    smoothedMouseRef.current.y = Math.abs(targetY - ny) < 0.001 ? targetY : ny;

    // 4. Write into your uniform
    materialRef.current.uniforms.uMouse.value.set(
      smoothedMouseRef.current.x,
      -smoothedMouseRef.current.y
    );

    // Apply stable time value - avoid using speedScale which can cause jumps
    // Use a fixed time factor based on initial settings rather than changing it
    const baseSpeed = 0.4;
    materialRef.current.uTime = clock.getElapsedTime() * baseSpeed;

    // // real elapsed seconds
    // const elapsed = clock.getElapsedTime();
    // // wrap to [0, loopDuration)
    // const loopDuration = 5;
    // const tWrapped = elapsed % loopDuration;
    // // normalize to [0,1)
    // const tNorm = tWrapped / loopDuration;
    // // convert to a full revolution [0,2π)
    // const angle = tNorm * 2.0 * Math.PI;
    // // feed your shader this looping angle
    // materialRef.current.uTime = angle;

    // Update resolution when size changes (rare operation)
    const currentWidth = materialRef.current.uResolution.x;
    const currentHeight = materialRef.current.uResolution.y;

    if (Math.abs(currentWidth - size.width) > 1 || Math.abs(currentHeight - size.height) > 1) {
      materialRef.current.uResolution.set(size.width, size.height);
    }

    // Update mouse with smoothed values to prevent jitter
    materialRef.current.uMouse.set(smoothedMouseRef.current.x, -smoothedMouseRef.current.y);

    const mat = materialRef.current;
    if (!mat) return;

    const uni = mat.uniforms.uSpherePositions.value;
    const count = mat.uniforms.uMetaballCount.value;
    const range = mat.uniforms.uMovementRange.value;
    const t = clock.getElapsedTime() * 0.15;
    // const t = angle;

    for (let i = 0; i < count; i++) {
      const base = basePositions.current[i];
      const phase = i * 1.234; // per-sphere phase offset
      const dir = i % 2 == 0 ? 1 : -1; // either +1 or -1

      // Lissajous offsets (with half the range on x/y for a “tighter” CCW)
      const xOff = Math.sin(t + phase) * range * 2.0 * dir;
      const yOff = Math.cos(t * 1.3 + phase * 1.1) * range * 1.2;
      const zOff = Math.sin(t * 0.8 + phase * 0.9) * range * 0.2;

      uni[i].set(base.x + xOff, base.y + yOff, base.z + zOff);
      // uni[i].set(base.x, base.y, base.z);
    }
  });

  return (
    <mesh position={pos}>
      <planeGeometry args={[size.aspect, 1, 1, 1]} />
      <metaballShaderMaterial ref={materialRef} />
    </mesh>
  );
};
