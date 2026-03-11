/**
 * WebGPU Shim for Firefox and other browsers without full WebGPU support
 *
 * Three.js v0.182+ includes WebGPU code that gets evaluated at module load time,
 * even when only using WebGL. This shim defines the missing GPUShaderStage and
 * other WebGPU constants to prevent errors during module initialization.
 *
 * This allows the 3d-force-graph library (which uses Three.js) to load and fall
 * back to WebGL rendering in browsers without WebGPU support.
 */

// Only add shims if WebGPU is not available
if (typeof window !== 'undefined' && typeof (window as any).GPUShaderStage === 'undefined') {
  // Define GPUShaderStage constants
  // These match the WebGPU spec values: https://www.w3.org/TR/webgpu/#typedefdef-gpushaderstageflags
  (window as any).GPUShaderStage = {
    VERTEX: 0x1,
    FRAGMENT: 0x2,
    COMPUTE: 0x4,
  };
}

if (typeof window !== 'undefined' && typeof (window as any).GPUBufferUsage === 'undefined') {
  // Define GPUBufferUsage constants
  (window as any).GPUBufferUsage = {
    MAP_READ: 0x0001,
    MAP_WRITE: 0x0002,
    COPY_SRC: 0x0004,
    COPY_DST: 0x0008,
    INDEX: 0x0010,
    VERTEX: 0x0020,
    UNIFORM: 0x0040,
    STORAGE: 0x0080,
    INDIRECT: 0x0100,
    QUERY_RESOLVE: 0x0200,
  };
}

if (typeof window !== 'undefined' && typeof (window as any).GPUTextureUsage === 'undefined') {
  // Define GPUTextureUsage constants
  (window as any).GPUTextureUsage = {
    COPY_SRC: 0x01,
    COPY_DST: 0x02,
    TEXTURE_BINDING: 0x04,
    STORAGE_BINDING: 0x08,
    RENDER_ATTACHMENT: 0x10,
  };
}

if (typeof window !== 'undefined' && typeof (window as any).GPUColorWrite === 'undefined') {
  // Define GPUColorWrite constants
  (window as any).GPUColorWrite = {
    RED: 0x1,
    GREEN: 0x2,
    BLUE: 0x4,
    ALPHA: 0x8,
    ALL: 0xF,
  };
}

if (typeof window !== 'undefined' && typeof (window as any).GPUMapMode === 'undefined') {
  // Define GPUMapMode constants
  (window as any).GPUMapMode = {
    READ: 0x0001,
    WRITE: 0x0002,
  };
}

// Export a flag to indicate if WebGPU is natively supported
export const isWebGPUSupported = typeof navigator !== 'undefined' && 'gpu' in navigator;

// Export a function to check WebGL support as fallback
export const isWebGLSupported = (): boolean => {
  if (typeof window === 'undefined') return false;

  try {
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl2') || canvas.getContext('webgl');
    return gl !== null;
  } catch {
    return false;
  }
};

// Console log for debugging (only in development)
if (import.meta.env.DEV && typeof window !== 'undefined') {
  if (!isWebGPUSupported) {
    console.log('[WebGPU Shim] WebGPU not natively supported, shim installed. Falling back to WebGL.');
  } else {
    console.log('[WebGPU Shim] WebGPU is natively supported.');
  }
}
