'use client';

import React, { Suspense, useEffect, useState, useMemo, useRef } from 'react';
import { Canvas, useFrame, useThree } from '@react-three/fiber';
import * as THREE from 'three';

// Import setup file FIRST - this registers the custom material
// import './SetupMaterial';

import { useContainerSize } from '@/hooks/useContainerSize';
import { useMousePosition } from '@/hooks/useMousePosition';
import { useElementVisibility } from '@/hooks/useElementVisibility';
import { usePerformanceMonitor } from '@/hooks/usePerformanceMonitor';
import { getDevicePixelRatio, isMobile, getPerformanceTier } from '@/hooks/deviceDetection';
import { MetaballShaderPlane } from './ShaderPlane';
import { OrbitControls } from '@react-three/drei';

interface ShaderCanvasProps {
  className?: string;
  style?: React.CSSProperties;
  // Add options to control quality/features
  initialQuality?: number;
  adaptiveQuality?: boolean;
  disableOnLowEnd?: boolean;
}

export const MetaballCanvas: React.FC<ShaderCanvasProps> = ({
  className = '',
  style = {},
  initialQuality = 1.0,
  adaptiveQuality = true,
  disableOnLowEnd = false,
}) => {
  // Get container size and mouse position
  const [containerRef, size] = useContainerSize();
  const mousePosition = useMousePosition();

  // Check if the canvas is in viewport
  const isVisible = useElementVisibility(containerRef, '-10%');

  // Monitor performance and get adaptive quality settings
  const { quality, fps, temperatureStatus } = usePerformanceMonitor(
    initialQuality,
    isMobile() ? 30 : 60, // Lower target FPS on mobile
    adaptiveQuality
  );

  // Low-end device detection
  const isLowEndDevice = useMemo(() => getPerformanceTier() === 'low', []);

  // Track visibility state
  const [visibilityState, setVisibilityState] = useState('visible');

  // Should we render the shader at all?
  const shouldRenderShader = !disableOnLowEnd || !isLowEndDevice;

  // Determine if we should show a simplified fallback on very low-end devices
  const [showFallback, setShowFallback] = useState(false);

  // Update visibility state
  useEffect(() => {
    setVisibilityState(isVisible ? 'visible' : 'hidden');
  }, [isVisible]);

  // After 2 seconds, check if fps is too low on a low-end device
  useEffect(() => {
    if (!adaptiveQuality || !isLowEndDevice) return;

    const timer = setTimeout(() => {
      if (fps < 20) {
        setShowFallback(true); // FIXED: Changed from 1(true) to setShowFallback(true)
      }
    }, 2000);

    return () => clearTimeout(timer);
  }, [fps, adaptiveQuality, isLowEndDevice]);

  // Determine performance-related settings
  const mobile = isMobile();

  // Progressive quality enhancement - start with low quality then increase
  const [progressiveQuality, setProgressiveQuality] = useState(mobile ? 0.5 : 0.7);

  // Gradually increase quality after load for better first impression
  useEffect(() => {
    if (!isVisible) return;

    const timer = setTimeout(() => {
      setProgressiveQuality(Math.min(1.0, quality));
    }, 500);

    return () => clearTimeout(timer);
  }, [isVisible, quality]);

  // Control frameloop based on visibility and device heat
  // const isDeviceHot = temperatureStatus === 'hot'
  // const frameloop = !isVisible ? 'never' : isDeviceHot ? 'demand' : mobile ? 'demand' : 'always'

  // Adaptive DPR based on device and state
  const defaultDpr = getDevicePixelRatio();
  const visibleDpr = mobile ? Math.min(defaultDpr, 1.5) : defaultDpr; // Cap at 1.5 on mobile
  const hiddenDpr = Math.min(1.0, defaultDpr * 0.5);
  const effectiveDpr = isVisible ? visibleDpr * progressiveQuality : hiddenDpr;

  // Dynamic settings for the shader
  const shaderSettings = {
    quality: progressiveQuality,
    metaballCount: quality < 0.5 ? 5 : quality < 0.8 ? 10 : 15,
    fps: fps,
    temperatureStatus: temperatureStatus,
  };

  // Render a colored div fallback for extremely low-end devices
  if (showFallback) {
    return (
      <div
        className={`${className} `}
        style={{
          position: 'relative',
          width: '100%',
          height: '100%',
          background: 'linear-gradient(45deg, #0f2027, #203a43, #2c5364)', // Added a nice gradient fallback
          ...style,
        }}
      />
    );
  }

  return (
    <div
      ref={containerRef}
      className={`${className} bg-gradient-green-blue`}
      // className={`${className}`}
      data-visibility={visibilityState}
      data-quality={Math.round(quality * 100)}
      data-fps={fps}
      style={{
        position: 'relative',
        width: '100%',
        height: '100%',
        overflow: 'hidden',
        ...style,
      }}>
      {shouldRenderShader && size.width > 0 && size.height > 0 && (
        // <CanvasRecorder fps={30} bitrate={5_000_000}>
        <Canvas
          className=""
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: '100%',
          }}
          dpr={effectiveDpr}
          // frameloop={frameloop}
          gl={{
            antialias: (!mobile || (mobile && quality > 0.8)) && isVisible,
            powerPreference: 'high-performance',
            precision: mobile || quality < 0.8 || !isVisible ? 'mediump' : 'highp',
            depth: false,
            stencil: false,
            alpha: true, // Disable alpha for better performance
            preserveDrawingBuffer: true,
          }}
          camera={{
            position: [0, 0, 1],
            fov: 50,
            near: 0.1,
            far: 2000,
          }}>
          <Suspense
            fallback={
              <div
                style={{
                  position: 'absolute',
                  top: '50%',
                  left: '50%',
                  transform: 'translate(-50%, -50%)',
                  color: 'white',
                  fontSize: '16px',
                }}>
                Loading...
              </div>
            }>
            <SceneCamera isInView={isVisible} />
            <MetaballShaderPlane
              size={size}
              mousePosition={mousePosition}
              isVisible={isVisible}
              settings={shaderSettings}
            />
          </Suspense>

          <OrbitControls />

          {/* {process.env.NODE_ENV === "development" && <Stats />} */}
        </Canvas>
        // </CanvasRecorder>
      )}
    </div>
  );
};

const SceneCamera: React.FC<{ isInView: boolean }> = ({ isInView }) => {
  const { camera } = useThree();
  const targetPosition = useRef(new THREE.Vector3(0, 0, 0.0));

  useEffect(() => {
    // Target Y is 0.3 when in view, else back to 0
    targetPosition.current.y = isInView ? 0 : 0.2;
    targetPosition.current.z = isInView ? 1.0 : 0.5;
  }, [isInView]);

  useFrame(() => {
    camera.position.lerp(targetPosition.current, 0.1);
  });

  return null;
};
