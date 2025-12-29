import { MetaballShaderMaterial } from './ShaderMaterial';
import { extend } from '@react-three/fiber/native';

// Extend Three.js with our custom materials
// This ensures the materials are registered before any component tries to use them
extend({
  MetaballShaderMaterial,
});

// This file doesn't export anything - it just needs to be imported
