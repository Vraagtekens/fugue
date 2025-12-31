import { Canvas, extend, useThree } from '@react-three/fiber/native';
import { View } from 'react-native';
import { useRef } from 'react';
import { shaderMaterial } from '@react-three/drei/native';
import { MetaballShaderMaterial } from './components/ShaderMaterial';
// import ExpoTHREE from 'expo-three';
// const THREE = ExpoTHREE.THREE;

// export const MetaballShaderMaterial = shaderMaterial(
//   {
//     uTime: 0,
//     uMetaballCount: 10,
//     uMovementRange: 0.5,
//   },
//   `void main() { gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }`,
//   `void main() { gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0); }`
// );

// extend({ MetaballShaderMaterial });

export const PlaneCanvas = () => {
  const materialRef = useRef(null);

  return (
    <View style={{ flex: 1 }}>
      <Canvas style={{ flex: 1 }} camera={{ position: [0, 0, 1], fov: 50 }}>
        <mesh>
          <planeGeometry args={[1, 1]} />
          {/* <meshBasicMaterial color="hotpink" side={2} /> */}
          <metaballShaderMaterial ref={materialRef} />
        </mesh>
      </Canvas>
    </View>
  );
};
