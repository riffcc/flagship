import { ref, type Ref } from 'vue';

/**
 * Extracts the dominant color from an image and returns a tinted gradient
 * Uses Canvas API to sample the image and calculate average color
 */
export function useImageColorExtraction() {
  const extractedColors = ref<Map<string, string>>(new Map());

  /**
   * Extract dominant color from image URL
   * Returns a gradient string with the color tinted darker
   */
  async function getColorTintedGradient(imageUrl: string | undefined): Promise<string> {
    // Default gradient if no image
    if (!imageUrl) {
      return 'to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)';
    }

    // Check cache first
    const cached = extractedColors.value.get(imageUrl);
    if (cached) {
      return cached;
    }

    try {
      // Create an image element
      const img = new Image();
      img.crossOrigin = 'anonymous';
      
      // Create a promise to wait for image load
      const colorPromise = new Promise<string>((resolve) => {
        img.onload = () => {
          // Create canvas
          const canvas = document.createElement('canvas');
          const ctx = canvas.getContext('2d');
          
          if (!ctx) {
            resolve('to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)');
            return;
          }

          // Set canvas size (small for performance)
          const sampleSize = 50;
          canvas.width = sampleSize;
          canvas.height = sampleSize;
          
          // Draw scaled image
          ctx.drawImage(img, 0, 0, sampleSize, sampleSize);
          
          // Get image data
          const imageData = ctx.getImageData(0, 0, sampleSize, sampleSize);
          const data = imageData.data;
          
          // Calculate average color
          let r = 0, g = 0, b = 0;
          let pixelCount = 0;
          
          for (let i = 0; i < data.length; i += 4) {
            // Skip very dark or very light pixels
            const brightness = (data[i] + data[i + 1] + data[i + 2]) / 3;
            if (brightness > 20 && brightness < 235) {
              r += data[i];
              g += data[i + 1];
              b += data[i + 2];
              pixelCount++;
            }
          }
          
          if (pixelCount === 0) {
            resolve('to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)');
            return;
          }
          
          // Average the colors
          r = Math.round(r / pixelCount);
          g = Math.round(g / pixelCount);
          b = Math.round(b / pixelCount);
          
          // Create gradient with darkened tint
          // Top: 40% opacity with slight darkening
          // Bottom: 60% opacity with more darkening
          const topColor = `rgba(${Math.round(r * 0.7)},${Math.round(g * 0.7)},${Math.round(b * 0.7)},.4)`;
          const bottomColor = `rgba(${Math.round(r * 0.5)},${Math.round(g * 0.5)},${Math.round(b * 0.5)},.6)`;
          
          const gradient = `to bottom, ${topColor}, ${bottomColor}`;
          resolve(gradient);
        };
        
        img.onerror = () => {
          // Fallback gradient on error
          resolve('to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)');
        };
      });
      
      // Start loading the image
      img.src = imageUrl;
      
      // Wait with timeout
      const gradient = await Promise.race([
        colorPromise,
        new Promise<string>((resolve) => 
          setTimeout(() => resolve('to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)'), 2000)
        )
      ]);
      
      // Cache the result
      extractedColors.value.set(imageUrl, gradient);
      
      return gradient;
    } catch (error) {
      console.error('Error extracting color:', error);
      return 'to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)';
    }
  }

  return {
    getColorTintedGradient
  };
}