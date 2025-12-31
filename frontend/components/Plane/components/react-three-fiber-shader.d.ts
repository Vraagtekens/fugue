import type { ReactThreeFiber } from '@react-three/fiber/native';
import type { MetaballShaderMaterialType, MetaballShaderMaterial } from './ShaderMaterial';

// 2) Augment the @react-three/fiber module
declare module '@react-three/fiber' {
  interface ThreeElements {
    /**
     * <metaballShaderMaterial />
     *   - uTime, uResolution, uMouse, etc. all flow through as props
     */
    metaballShaderMaterial: ReactThreeFiber.Object3DNode<
      MetaballShaderMaterialType,
      typeof MetaballShaderMaterial
    >;
  }
}
