import { useState, useEffect, useRef } from 'react';

/**
 * Custom hook to monitor and adjust performance for WebGL content
 * Provides dynamic quality adjustments based on actual performance
 * This version works outside the Canvas component
 */
export function usePerformanceMonitor(
  initialQuality: number = 1.0,
  targetFPS: number = 60,
  adaptiveMode: boolean = true
) {
  // Current quality level (1.0 = 100%, 0.5 = 50%)
  const [quality, setQuality] = useState(initialQuality);

  // Performance metrics
  const [fps, setFPS] = useState(targetFPS);
  const [isStable, setIsStable] = useState(false);

  // Refs for tracking
  const frameTimeRef = useRef<number[]>([]);
  const lastTimeRef = useRef<number>(0);
  const frameCountRef = useRef<number>(0);
  const frameIdRef = useRef<number | null>(null);
  const startTimeRef = useRef<number>(0);
  const temperatureCheckTimeRef = useRef<number>(0);

  // Temperature estimation (inferred from performance degradation)
  const [temperatureStatus, setTemperatureStatus] = useState<'normal' | 'warm' | 'hot'>('normal');

  // Frame timing function using requestAnimationFrame
  useEffect(() => {
    if (!adaptiveMode) return;

    // Start time tracking
    startTimeRef.current = performance.now();
    lastTimeRef.current = performance.now();

    // Function to call on each animation frame
    const measurePerformance = () => {
      const currentTime = performance.now();
      const deltaTime = (currentTime - lastTimeRef.current) / 1000; // convert to seconds
      lastTimeRef.current = currentTime;

      // Skip anomalous frames
      if (deltaTime > 0.1) {
        frameIdRef.current = requestAnimationFrame(measurePerformance);
        return;
      }

      // Add frame time to rolling buffer (keep last 60 frames)
      frameTimeRef.current.push(deltaTime);
      if (frameTimeRef.current.length > 60) {
        frameTimeRef.current.shift();
      }

      // Count frames for FPS calculation
      frameCountRef.current++;

      // Calculate FPS every second
      if (currentTime - startTimeRef.current >= 1000) {
        const newFPS = frameCountRef.current / ((currentTime - startTimeRef.current) / 1000);
        setFPS(Math.round(newFPS));
        frameCountRef.current = 0;
        startTimeRef.current = currentTime;

        // Adaptive quality based on FPS
        if (frameTimeRef.current.length >= 30) {
          // Calculate average frame time
          const avgFrameTime =
            frameTimeRef.current.reduce((a, b) => a + b, 0) / frameTimeRef.current.length;
          const currentFPS = 1.0 / avgFrameTime;

          // Check if we need to adjust quality
          if (currentFPS < targetFPS * 0.8 && quality > 0.2) {
            // Performance too low, reduce quality
            setQuality((prev) => Math.max(0.2, prev - 0.1));
            frameTimeRef.current = []; // Reset measurements after adjustment
          } else if (
            currentFPS > targetFPS * 0.95 &&
            quality < 1.0 &&
            temperatureStatus === 'normal'
          ) {
            // Performance good, try increasing quality slowly if device isn't hot
            setQuality((prev) => Math.min(1.0, prev + 0.05));
            frameTimeRef.current = []; // Reset measurements after adjustment
          }
        }
      }

      // Check for thermal throttling every 10 seconds
      if (currentTime - temperatureCheckTimeRef.current >= 10000) {
        temperatureCheckTimeRef.current = currentTime;

        // If FPS is consistently dropping despite quality adjustments,
        // the device might be thermal throttling
        if (frameTimeRef.current.length >= 30) {
          const firstHalf = frameTimeRef.current.slice(0, 15);
          const secondHalf = frameTimeRef.current.slice(-15);

          const firstHalfAvg = firstHalf.reduce((a, b) => a + b, 0) / firstHalf.length;
          const secondHalfAvg = secondHalf.reduce((a, b) => a + b, 0) / secondHalf.length;

          // If frame times are increasing (FPS decreasing) steadily
          if (secondHalfAvg > firstHalfAvg * 1.2) {
            // Device might be getting warm
            setTemperatureStatus(temperatureStatus === 'normal' ? 'warm' : 'hot');

            // Reduce quality more aggressively if device seems hot
            if (temperatureStatus === 'warm' || temperatureStatus === 'hot') {
              setQuality((prev) => Math.max(0.2, prev - 0.2));
              frameTimeRef.current = [];
            }
          } else if (secondHalfAvg < firstHalfAvg * 0.9 && temperatureStatus !== 'normal') {
            // Device seems to be cooling down
            setTemperatureStatus(temperatureStatus === 'hot' ? 'warm' : 'normal');
          }
        }

        // Check stability
        if (frameTimeRef.current.length >= 30) {
          const variance = calculateVariance(frameTimeRef.current);
          setIsStable(variance < 0.0001); // Low variance means stable framerate
        }
      }

      // Continue the loop
      frameIdRef.current = requestAnimationFrame(measurePerformance);
    };

    // Start the performance monitoring loop
    frameIdRef.current = requestAnimationFrame(measurePerformance);

    // Cleanup
    return () => {
      if (frameIdRef.current !== null) {
        cancelAnimationFrame(frameIdRef.current);
      }
    };
  }, [adaptiveMode, quality, targetFPS, temperatureStatus]);

  // Calculate variance of frame times
  function calculateVariance(values: number[]): number {
    const avg = values.reduce((a, b) => a + b, 0) / values.length;
    const squareDiffs = values.map((value) => {
      const diff = value - avg;
      return diff * diff;
    });
    return squareDiffs.reduce((a, b) => a + b, 0) / values.length;
  }

  return {
    quality,
    fps,
    isStable,
    temperatureStatus,
    // Allow manual quality adjustment if needed
    setQuality,
  };
}
