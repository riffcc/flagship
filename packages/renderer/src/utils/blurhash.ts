import { decode, encode } from 'blurhash';

function loadImageElement(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.crossOrigin = 'anonymous';
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error(`Failed to load image for BlurHash: ${src}`));
    image.src = src;
  });
}

function createCanvas(width: number, height: number): HTMLCanvasElement {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  return canvas;
}

export function decodeBlurHashToDataUrl(
  blurHash: string,
  width = 32,
  height = 32,
  punch = 1,
): string | null {
  if (!blurHash) {
    return null;
  }

  try {
    const pixels = decode(blurHash, width, height, punch);
    const canvas = createCanvas(width, height);
    const context = canvas.getContext('2d');

    if (!context) {
      return null;
    }

    const imageData = context.createImageData(width, height);
    imageData.data.set(pixels);
    context.putImageData(imageData, 0, 0);
    return canvas.toDataURL();
  } catch (error) {
    console.warn('[BlurHash] Failed to decode blurhash:', error);
    return null;
  }
}

export async function generateBlurHashFromBlob(
  blob: Blob,
  options?: {
    width?: number;
    height?: number;
    componentX?: number;
    componentY?: number;
  },
): Promise<string | null> {
  const objectUrl = URL.createObjectURL(blob);

  try {
    const image = await loadImageElement(objectUrl);
    const targetWidth = options?.width ?? 32;
    const targetHeight = options?.height ?? 32;
    const canvas = createCanvas(targetWidth, targetHeight);
    const context = canvas.getContext('2d', { willReadFrequently: true });

    if (!context) {
      return null;
    }

    context.drawImage(image, 0, 0, targetWidth, targetHeight);
    const { data } = context.getImageData(0, 0, targetWidth, targetHeight);
    return encode(data, targetWidth, targetHeight, options?.componentX ?? 4, options?.componentY ?? 4);
  } catch (error) {
    console.warn('[BlurHash] Failed to generate blurhash:', error);
    return null;
  } finally {
    URL.revokeObjectURL(objectUrl);
  }
}
