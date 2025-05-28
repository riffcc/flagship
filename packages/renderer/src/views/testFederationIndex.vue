<template>
  <v-container>
    <v-card>
      <v-card-title>Federation Index Test</v-card-title>
      <v-card-text>
        <p>This page directly adds test entries to the federation index for testing purposes.</p>
        
        <v-text-field
          v-model="numberOfEntries"
          label="Number of entries to add"
          type="number"
          min="1"
          max="100"
        />
        
        <v-btn
          color="primary"
          :loading="loading"
          @click="addTestEntries"
        >
          Add Test Entries to Federation Index
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

const { lensService } = useLensService();
const numberOfEntries = ref(5);
const loading = ref(false);
const message = ref('');
const messageType = ref<'success' | 'error'>('success');

// Sample IPFS CIDs from real content
const sampleCids = [
  'QmWfVY9y3xjsixTgbd9AorQxH7VtMpzfx2HaWtsoUYecaX',
  'QmUNLLsPACCz1vFQAnkoPuBfTR7H9cQMcv54YdxKqJbvxE',
  'QmXGpMJMKcdfVxaWsW8FizUb8SqydHwb5TwAH3iC6pMvYr',
  'QmPZ9gcCEpqKTo6aq61g2nXGUhM4iCL3ewB6LDXZCtioEB',
  'QmbFMfYgPHB87cXGqQ9tWp8Yf1HjBkwc8QqJcgvRwhYQxe',
];

const sampleThumbnails = [
  'QmUNLLsPACCz1vFQAnkoPuBfTR7H9cQMcv54YdxKqJbvxE',
  'QmXvdGJK9RgBpwCYgZKGMCwHnmJpchV9VhYiHz8XZy2pNa',
];

const categories = ['music', 'video', 'podcast', 'ebook', 'software'];
const contentTypes = ['video', 'audio', 'text', 'application'];

async function addTestEntries() {
  loading.value = true;
  message.value = '';
  
  try {
    // Get the site program to access federation index directly
    const siteProgram = (lensService as any).siteProgram;
    if (!siteProgram?.federationIndex) {
      throw new Error('Federation index not available. Make sure the site is opened.');
    }
    
    // Get site metadata for source info
    const siteId = await lensService.getSiteId();
    const siteMetadata = await lensService.getSiteMetadata();
    const siteName = siteMetadata.name || 'Test Site';
    
    let successCount = 0;
    for (let i = 0; i < numberOfEntries.value; i++) {
      const contentType = contentTypes[Math.floor(Math.random() * contentTypes.length)];
      const categoryId = categories[Math.floor(Math.random() * categories.length)];
      
      const indexEntry = {
        contentCid: sampleCids[Math.floor(Math.random() * sampleCids.length)],
        title: `Test Entry ${i + 1} - ${new Date().toLocaleDateString()}`,
        sourceSiteId: siteId,
        sourceSiteName: siteName,
        contentType: contentType,
        categoryId: categoryId,
        timestamp: Date.now(),
        description: `This is test entry number ${i + 1} with a valid IPFS CID`,
        thumbnailCid: sampleThumbnails[Math.floor(Math.random() * sampleThumbnails.length)],
        tags: ['test', contentType, 'sample'],
        featured: i < 2, // Make first 2 featured
        metadata: JSON.stringify({
          duration: Math.floor(Math.random() * 3600),
          fileSize: Math.floor(Math.random() * 1000000000),
          format: contentType === 'video' ? 'mp4' : contentType === 'audio' ? 'mp3' : 'pdf',
        }),
      };
      
      console.log(`Adding federation index entry ${i + 1}:`, indexEntry);
      
      try {
        await siteProgram.federationIndex.insertContent(indexEntry);
        successCount++;
        console.log(`Successfully added entry ${i + 1}`);
      } catch (error) {
        console.error(`Failed to add entry ${i + 1}:`, error);
      }
    }
    
    message.value = `Successfully added ${successCount} entries to federation index`;
    messageType.value = successCount > 0 ? 'success' : 'error';
    
    // Refresh the homepage data by navigating there
    if (successCount > 0) {
      setTimeout(() => {
        window.location.href = '#/';
      }, 2000);
    }
  } catch (error) {
    console.error('Failed to add test entries:', error);
    message.value = `Failed to add test entries: ${error.message}`;
    messageType.value = 'error';
  } finally {
    loading.value = false;
  }
}
</script>