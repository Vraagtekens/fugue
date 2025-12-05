// (tabs)/3d/index.tsx
// import { View } from 'react-native';
// import { Canvas } from '@react-three/fiber/native';
// import { Stack } from 'expo-router';
// import { useRef } from 'react';
// import { useFrame } from '@react-three/fiber';
// import * as THREE from 'three';
// import { useColorScheme } from 'nativewind';

// function RotatingBox() {
//   const meshRef = useRef<THREE.Mesh>(null!);

//   useFrame(() => {
//     if (!meshRef.current) return;
//     meshRef.current.rotation.x += 0.01;
//     meshRef.current.rotation.y += 0.02;
//   });

//   return (
//     <mesh ref={meshRef} position={[0, 0, 0]}>
//       <boxGeometry args={[1, 1, 1]} />
//       <meshStandardMaterial color="orange" />
//     </mesh>
//   );
// }

// const SCREEN_OPTIONS = {
//   title: '3D Box',
//   headerTransparent: true,
//   headerBackButtonDisplayMode: 'minimal',
// };

// export default function Page() {
//   const { colorScheme } = useColorScheme();
//   const isDark = colorScheme === 'dark';
//   const backgroundColor = isDark ? '#111' : '#f8f8f8';

//   return (
//     <>
//       <Stack.Screen options={SCREEN_OPTIONS} />

//       <View style={{ flex: 1, backgroundColor: backgroundColor }}>
//         <Canvas camera={{ position: [0, 0, 5], fov: 60 }} gl={{ antialias: true }}>
//           <color attach="background" args={[backgroundColor]} />
//           <ambientLight intensity={1.2} />
//           <directionalLight position={[3, 3, 3]} intensity={1} />
//           <RotatingBox />
//         </Canvas>
//       </View>
//     </>
//   );
// }

// (tabs)/3d/index.tsx
import { View } from 'react-native';
import { Canvas, useLoader } from '@react-three/fiber/native';
import { Stack } from 'expo-router';
import { Suspense, useEffect, useRef, useState } from 'react';
import { OrbitControls, useAnimations, useGLTF } from '@react-three/drei/native';
import { useColorScheme } from 'nativewind';
import * as THREE from 'three';
import modelPath from '@/assets/models/package.glb';

function Model({ modelPath }: { modelPath: string }) {
  const gltf = useGLTF(modelPath);
  const ref = useRef<any>(null);

  const group = useRef<any>(null);
  const { animations } = gltf;

  // Animation mixer
  const { actions } = useAnimations(animations, group);

  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    if (!group.current) return;

    // Scale + rotate similar to your Next.js version
    group.current.rotation.x = Math.PI / 2;

    gltf.scene.traverse((child: any) => {
      if (child.isMesh) {
        if (child.material) {
          child.material.flatShading = true;
          child.material.needsUpdate = true;
        }
      }
    });

    // Prepare first animation clip
    const firstAction = actions[Object.keys(actions)[0]];
    if (firstAction) {
      firstAction.setLoop(THREE.LoopOnce);
      firstAction.clampWhenFinished = true;
      firstAction.paused = true;
      firstAction.time = 0; // reset so reverse works
    }
  }, []);

  const toggleAnimation = () => {
    const firstAction = actions[Object.keys(actions)[0]];
    if (!firstAction) return;

    firstAction.paused = false;

    if (!isOpen) {
      // Play forward
      firstAction.timeScale = 1;
      firstAction.play();
      setIsOpen(true);
    } else {
      // Play backward
      firstAction.timeScale = -1;
      firstAction.play();
      setIsOpen(false);
    }
  };

  return (
    <group ref={group} scale={0.5} onClick={toggleAnimation}>
      <primitive ref={ref} object={gltf.scene} scale={1.0} />
    </group>
  );
}

const SCREEN_OPTIONS = {
  title: '3D Model',
  headerTransparent: true,
  headerBackButtonDisplayMode: 'minimal',
};

export default function Page() {
  const { colorScheme } = useColorScheme();
  const backgroundColor = colorScheme === 'dark' ? '#111' : '#f8f8f8';

  return (
    <>
      <Stack.Screen options={SCREEN_OPTIONS} />

      <View style={{ flex: 1, backgroundColor }}>
        <Canvas camera={{ position: [0, 0, 5], fov: 60 }} gl={{ antialias: true }}>
          <color attach="background" args={[backgroundColor]} />
          <ambientLight intensity={1.2} />
          <directionalLight position={[3, 3, 3]} intensity={1} />
          <OrbitControls />

          {/* Suspense lets the loader wait for the model */}
          <Suspense fallback={null}>
            <Model modelPath={modelPath} />
          </Suspense>
        </Canvas>
      </View>
    </>
  );
}
