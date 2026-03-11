<template>
  <v-dialog
    v-model="dialogOpen"
    width="auto"
  >
    <template #activator="{ props: activatorProps }">
      <slot name="activator" :props="activatorProps">
        <v-btn
          v-bind="activatorProps"
          :disabled="!hasCategory"
          rounded="0"
          text="Advanced"
          variant="outlined"
          class="mb-4"
          block
        />
      </slot>
    </template>
    <v-card
      v-if="metadataSchema"
      width="480px"
      max-height="620px"
      class="pa-8 ma-auto overflow-y-auto"
      color="black"
    >
      <p class="text-subtitle mb-6 text-center">
        Additional metadata options
      </p>

      <!-- License Chooser -->
      <v-select
        v-model="localLicenseType"
        :items="licenseOptions"
        label="License"
        item-title="title"
        item-value="value"
        clearable
        hint="Choose a Creative Commons license or enter a custom URL"
        persistent-hint
        class="mb-4"
      >
        <template #item="{ item, props: itemProps }">
          <v-list-item v-bind="itemProps">
            <template #subtitle>
              <span class="text-caption">{{ item.raw.description }}</span>
            </template>
          </v-list-item>
        </template>
      </v-select>

      <!-- License details (shown when a CC license is selected) -->
      <template v-if="localLicenseType && localLicenseType !== 'custom'">
        <v-row>
          <v-col cols="6">
            <v-select
              v-model="localLicenseVersion"
              :items="licenseVersionOptions"
              label="Version"
              item-title="title"
              item-value="value"
            />
          </v-col>
          <v-col cols="6">
            <v-autocomplete
              v-model="localLicenseJurisdiction"
              :items="jurisdictionOptions"
              label="Country"
              item-title="title"
              item-value="value"
              clearable
            />
          </v-col>
        </v-row>
      </template>

      <!-- Custom URL field (shown when 'custom' is selected) -->
      <v-text-field
        v-if="localLicenseType === 'custom'"
        v-model="localCustomLicenseUrl"
        label="License URL"
        hint="Enter the full URL to the license"
        persistent-hint
        placeholder="https://example.com/license"
        class="mb-4"
      />

      <!-- Attribution field (shown when any license is selected) -->
      <v-text-field
        v-if="localLicenseType"
        v-model="localLicenseAttribution"
        label="Attribution (Optional)"
        hint="Credit the original creator, e.g. 'Music by Artist Name'"
        persistent-hint
        class="mb-4"
      />

      <!-- Category-specific metadata fields -->
      <template
        v-for="[fieldName, fieldConfig] in Object.entries(metadataSchema)"
        :key="fieldName"
      >
        <!-- Skip hidden/auto-managed fields and primary fields shown above -->
        <template v-if="!HIDDEN_METADATA_FIELDS.includes(fieldName) && !PRIMARY_METADATA_FIELDS.includes(fieldName)">
          <v-select
            v-if="(fieldConfig as any).options"
            :items="(fieldConfig as any).options"
            :label="formatFieldLabel(fieldName)"
            :model-value="String(metadata[fieldName] || '')"
            @update:model-value="(v) => updateMetadataField(fieldName, v)"
          />
          <v-text-field
            v-else
            :label="formatFieldLabel(fieldName)"
            :model-value="String(metadata[fieldName] || '')"
            :type="(fieldConfig as any).type || 'text'"
            @update:model-value="(v) => updateMetadataField(fieldName, v)"
          >
            <template #append-inner>
              <v-tooltip
                location="top"
                :text="(fieldConfig as any).description || ''"
              >
                <template #activator="{ props: tooltipProps }">
                  <v-icon
                    size="small"
                    v-bind="tooltipProps"
                    color="grey-lighten-1"
                    icon="$help-circle-outline"
                  />
                </template>
              </v-tooltip>
            </template>
          </v-text-field>
        </template>
      </template>

      <v-btn
        rounded="0"
        text="Save"
        color="primary"
        block
        @click="handleSave"
      />
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';

interface Props {
  modelValue: boolean;
  categoryId: string;
  metadataSchema: Record<string, any> | null;
  metadata: Record<string, any>;
  // License props
  licenseType?: string;
  licenseVersion?: string;
  licenseJurisdiction?: string;
  licenseAttribution?: string;
  customLicenseUrl?: string;
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
  (e: 'update:metadata', value: Record<string, any>): void;
  (e: 'update:licenseType', value: string): void;
  (e: 'update:licenseVersion', value: string): void;
  (e: 'update:licenseJurisdiction', value: string): void;
  (e: 'update:licenseAttribution', value: string): void;
  (e: 'update:customLicenseUrl', value: string): void;
}

const props = withDefaults(defineProps<Props>(), {
  licenseType: '',
  licenseVersion: '4.0',
  licenseJurisdiction: '',
  licenseAttribution: '',
  customLicenseUrl: '',
});

const emit = defineEmits<Emits>();

// Dialog state
const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

// Local copies for editing
const localLicenseType = ref(props.licenseType);
const localLicenseVersion = ref(props.licenseVersion);
const localLicenseJurisdiction = ref(props.licenseJurisdiction);
const localLicenseAttribution = ref(props.licenseAttribution);
const localCustomLicenseUrl = ref(props.customLicenseUrl);
const localMetadata = ref<Record<string, any>>({ ...props.metadata });

// Sync from props
watch(() => props.licenseType, (v) => { localLicenseType.value = v; });
watch(() => props.licenseVersion, (v) => { localLicenseVersion.value = v; });
watch(() => props.licenseJurisdiction, (v) => { localLicenseJurisdiction.value = v; });
watch(() => props.licenseAttribution, (v) => { localLicenseAttribution.value = v; });
watch(() => props.customLicenseUrl, (v) => { localCustomLicenseUrl.value = v; });
watch(() => props.metadata, (v) => { localMetadata.value = { ...v }; }, { deep: true });

const hasCategory = computed(() => Boolean(props.categoryId));

// Hidden metadata fields that are auto-managed (not shown anywhere)
const HIDDEN_METADATA_FIELDS = ['trackMetadata', 'type', 'artistId', 'artist', 'license'];

// Primary fields shown in the main form (not in Advanced dialog)
const PRIMARY_METADATA_FIELDS = ['albumTitle'];

// License options
const licenseOptions = [
  { value: 'cc0', title: 'CC0 (Public Domain)', description: 'No rights reserved - free for any use' },
  { value: 'cc-by', title: 'CC BY', description: 'Attribution required' },
  { value: 'cc-by-sa', title: 'CC BY-SA', description: 'Attribution + ShareAlike (copyleft)' },
  { value: 'cc-by-nd', title: 'CC BY-ND', description: 'Attribution + No Derivatives' },
  { value: 'cc-by-nc', title: 'CC BY-NC', description: 'Attribution + NonCommercial' },
  { value: 'cc-by-nc-sa', title: 'CC BY-NC-SA', description: 'Attribution + NonCommercial + ShareAlike' },
  { value: 'cc-by-nc-nd', title: 'CC BY-NC-ND', description: 'Attribution + NonCommercial + No Derivatives' },
  { value: 'custom', title: 'Custom URL', description: 'Enter a custom license URL' },
];

// CC License versions
const licenseVersionOptions = [
  { value: '4.0', title: '4.0 (Current)' },
  { value: '3.0', title: '3.0' },
  { value: '2.5', title: '2.5' },
  { value: '2.0', title: '2.0' },
  { value: '1.0', title: '1.0' },
  { value: 'unknown', title: 'Unknown' },
];

// Common CC jurisdiction ports (country codes)
const jurisdictionOptions = [
  { value: '', title: 'International' },
  { value: 'au', title: 'Australia' },
  { value: 'at', title: 'Austria' },
  { value: 'be', title: 'Belgium' },
  { value: 'br', title: 'Brazil' },
  { value: 'ca', title: 'Canada' },
  { value: 'cl', title: 'Chile' },
  { value: 'cn', title: 'China' },
  { value: 'co', title: 'Colombia' },
  { value: 'hr', title: 'Croatia' },
  { value: 'cz', title: 'Czech Republic' },
  { value: 'dk', title: 'Denmark' },
  { value: 'ec', title: 'Ecuador' },
  { value: 'fi', title: 'Finland' },
  { value: 'fr', title: 'France' },
  { value: 'de', title: 'Germany' },
  { value: 'gr', title: 'Greece' },
  { value: 'hk', title: 'Hong Kong' },
  { value: 'hu', title: 'Hungary' },
  { value: 'in', title: 'India' },
  { value: 'ie', title: 'Ireland' },
  { value: 'il', title: 'Israel' },
  { value: 'it', title: 'Italy' },
  { value: 'jp', title: 'Japan' },
  { value: 'kr', title: 'South Korea' },
  { value: 'my', title: 'Malaysia' },
  { value: 'mx', title: 'Mexico' },
  { value: 'nl', title: 'Netherlands' },
  { value: 'nz', title: 'New Zealand' },
  { value: 'no', title: 'Norway' },
  { value: 'pe', title: 'Peru' },
  { value: 'ph', title: 'Philippines' },
  { value: 'pl', title: 'Poland' },
  { value: 'pt', title: 'Portugal' },
  { value: 'ro', title: 'Romania' },
  { value: 'rs', title: 'Serbia' },
  { value: 'sg', title: 'Singapore' },
  { value: 'za', title: 'South Africa' },
  { value: 'es', title: 'Spain' },
  { value: 'se', title: 'Sweden' },
  { value: 'ch', title: 'Switzerland' },
  { value: 'tw', title: 'Taiwan' },
  { value: 'th', title: 'Thailand' },
  { value: 'uk', title: 'United Kingdom' },
  { value: 'us', title: 'United States' },
  { value: 'vn', title: 'Vietnam' },
];

const fieldLabelMap: Record<string, string> = {
  // Music fields
  description: 'Description',
  totalSongs: 'Total Songs',
  totalDuration: 'Total Duration',
  genres: 'Genres',
  tags: 'Tags',
  musicBrainzID: 'MusicBrainz ID',
  albumTitle: 'Release Name',
  releaseYear: 'Release Year',
  releaseType: 'Release Type',
  fileFormat: 'File Format',
  bitrate: 'Bitrate',
  mediaFormat: 'Media Format',
  // Video fields
  title: 'Title',
  duration: 'Duration',
  resolution: 'Resolution',
  format: 'Format',
  uploader: 'Uploader',
  uploadDate: 'Upload Date',
  sourceUrl: 'Source URL',
  // Movie fields
  TMDBID: 'TMDB ID',
  IMDBID: 'IMDB ID',
  classification: 'Classification',
  // TV Show fields
  seasons: 'Seasons',
  totalEpisodes: 'Total Episodes',
  firstAiredYear: 'First Aired Year',
  status: 'Status',
  network: 'Network',
  averageEpisodeDuration: 'Average Episode Duration',
  // Common field
  cover: 'Cover Image CID',
  author: 'Author',
};

function formatFieldLabel(fieldName: string): string {
  return fieldLabelMap[fieldName] || fieldName;
}

function updateMetadataField(fieldName: string, value: string | null) {
  localMetadata.value = {
    ...localMetadata.value,
    [fieldName]: value || '',
  };
}

function handleSave() {
  // Emit all updates
  emit('update:licenseType', localLicenseType.value);
  emit('update:licenseVersion', localLicenseVersion.value);
  emit('update:licenseJurisdiction', localLicenseJurisdiction.value);
  emit('update:licenseAttribution', localLicenseAttribution.value);
  emit('update:customLicenseUrl', localCustomLicenseUrl.value);
  emit('update:metadata', localMetadata.value);
  dialogOpen.value = false;
}
</script>
