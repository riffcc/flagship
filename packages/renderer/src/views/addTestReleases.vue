<template>
  <v-container>
    <v-card>
      <v-card-title>Add Test Releases with Valid IPFS CIDs</v-card-title>
      <v-card-text>
        <p>This page allows you to add test releases with valid IPFS content CIDs to test the federation index.</p>
        
        <v-text-field
          v-model="numberOfReleases"
          label="Number of releases to add"
          type="number"
          min="1"
          max="100"
        />
        
        <v-btn
          color="primary"
          :loading="loading"
          @click="addTestReleases"
        >
          Add Test Releases
        </v-btn>
        
        <v-alert
          v-if="message"
          :type="messageType"
          class="mt-4"
        >
          {{ message }}
        </v-alert>
      </v-card-text>
    </v-card>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useLensService } from '../plugins/lensService/hooks';
import { v4 as uuid } from 'uuid';

const { lensService } = useLensService();
const numberOfReleases = ref(5);
const loading = ref(false);
const message = ref('');
const messageType = ref<'success' | 'error'>('success');

// Sample IPFS CIDs from real content (these are public test files)
const sampleCids = [
  'QmWfVY9y3xjsixTgbd9AorQxH7VtMpzfx2HaWtsoUYecaX', // Sample video
  'QmUNLLsPACCz1vFQAnkoPuBfTR7H9cQMcv54YdxKqJbvxE', // Sample image
  'QmXGpMJMKcdfVxaWsW8FizUb8SqydHwb5TwAH3iC6pMvYr', // Sample document
  'QmPZ9gcCEpqKTo6aq61g2nXGUhM4iCL3ewB6LDXZCtioEB', // Sample text
  'QmbFMfYgPHB87cXGqQ9tWp8Yf1HjBkwc8QqJcgvRwhYQxe', // Sample data
];

const sampleThumbnails = [
  'QmUNLLsPACCz1vFQAnkoPuBfTR7H9cQMcv54YdxKqJbvxE', // Use images as thumbnails
  'QmXvdGJK9RgBpwCYgZKGMCwHnmJpchV9VhYiHz8XZy2pNa',
];

const categories = ['music', 'video', 'podcast', 'ebook', 'software'];
const contentTypes = ['video', 'audio', 'text', 'application'];

async function addTestReleases() {
  loading.value = true;
  message.value = '';
  
  try {
    // Use some predefined category IDs or a single category
    // Since we don't have a way to get categories, let's use a simple ID
    const categoryIds = ['music', 'video', 'podcast', 'ebook', 'software'];
    
    // Add test releases
    let successCount = 0;
    for (let i = 0; i < numberOfReleases.value; i++) {
      const contentType = contentTypes[Math.floor(Math.random() * contentTypes.length)];
      const categoryId = categoryIds[Math.floor(Math.random() * categoryIds.length)] || categoryIds[0];
      
      const releaseData = {
        name: `Test Release ${i + 1} - ${new Date().toLocaleDateString()}`,
        categoryId: categoryId,
        contentCID: sampleCids[Math.floor(Math.random() * sampleCids.length)],
        thumbnailCID: sampleThumbnails[Math.floor(Math.random() * sampleThumbnails.length)],
        metadata: JSON.stringify({
          contentType: contentType,
          description: `This is test release number ${i + 1} with a valid IPFS CID`,
          tags: ['test', contentType, 'sample'],
          duration: Math.floor(Math.random() * 3600),
          fileSize: Math.floor(Math.random() * 1000000000),
          format: contentType === 'video' ? 'mp4' : contentType === 'audio' ? 'mp3' : 'pdf',
        }),
      };
      
      console.log(`Adding release ${i + 1}:`, releaseData);
      const result = await lensService.addRelease(releaseData);
      if (result.success) {
        successCount++;
      } else {
        console.error(`Failed to add release ${i + 1}:`, result.error);
      }
    }
    
    message.value = `Successfully added ${successCount} test releases with valid IPFS CIDs`;
    messageType.value = 'success';
  } catch (error) {
    console.error('Failed to add test releases:', error);
    message.value = `Failed to add test releases: ${error.message}`;
    messageType.value = 'error';
  } finally {
    loading.value = false;
  }
}
</script>