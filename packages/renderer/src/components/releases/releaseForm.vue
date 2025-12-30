<template>
  <v-form
    ref="formRef"
    :disabled="addReleaseMutation.isPending.value"
    class="d-flex flex-column ga-2"
    @submit.prevent="handleOnSubmit"
  >
    <v-text-field
      v-model="releaseItem.name"
      :label="isTVCategory ? 'Episode Name' : 'Title'"
      :rules="[rules.required]"
    />
    <v-text-field
      v-model="releaseItem.contentCID"
      label="Content CID"
      :rules="[rules.required, rules.isValidCid]"
    />
    <v-select
      v-model="releaseItem.categoryId"
      :items="contentCategoriesItems"
      :rules="[rules.required]"
      label="Category"
      item-title="title"
      item-value="value"
    />

    <!-- Music Fields (Artist and Release Name visible by default) -->
    <template v-if="isMusicCategory">
      <v-autocomplete
        v-model="selectedArtistId"
        :items="artistItems"
        :loading="artistsQuery.isLoading.value"
        label="Artist"
        placeholder="Type to search or create new artist..."
        clearable
        @update:model-value="(v) => handleChangeMetadataField('artistId', v)"
        @update:search="artistSearchText = $event"
      >
        <template v-if="shouldShowCreateArtistOption" #append-item>
          <v-list-item
            @click="createNewArtist"
            class="text-primary"
          >
            <v-list-item-title>
              <v-icon start>$plus</v-icon>
              Create "{{ artistSearchText }}"
            </v-list-item-title>
          </v-list-item>
        </template>
      </v-autocomplete>

      <v-text-field
        label="Release Name"
        :model-value="String((releaseItem.metadata && releaseItem.metadata['albumTitle']) || '')"
        hint="Album or EP name"
        persistent-hint
        @update:model-value="(v) => handleChangeMetadataField('albumTitle', v)"
      />
    </template>

    <!-- TV Show Fields -->
    <template v-if="isTVCategory">
      <v-autocomplete
        v-model="selectedSeriesId"
        :items="seriesItems"
        :loading="structuresQuery.isLoading.value"
        label="TV Series"
        placeholder="Type to search or create new series..."
        :rules="[rules.required]"
        clearable
        @update:search="seriesSearchText = $event"
      >
        <template v-if="shouldShowCreateSeriesOption" #append-item>
          <v-list-item
            @click="createNewSeries"
            class="text-primary"
          >
            <v-list-item-title>
              <v-icon start>$plus</v-icon>
              Create "{{ seriesSearchText }}"
            </v-list-item-title>
          </v-list-item>
        </template>
      </v-autocomplete>

      <v-row v-if="selectedSeriesId">
        <v-col cols="6">
          <v-text-field
            v-model.number="seasonNumber"
            label="Season Number"
            type="number"
            :rules="[rules.required, rules.positiveNumber]"
            @update:model-value="handleSeasonChange"
          />
        </v-col>
        <v-col cols="6">
          <v-text-field
            v-model.number="episodeNumber"
            label="Episode Number"
            type="number"
            :rules="[rules.required, rules.positiveNumber]"
          />
        </v-col>
      </v-row>
    </template>

    <v-text-field
      v-model="releaseItem.thumbnailCID"
      label="Thumbnail CID (Optional)"
      :rules="[rules.isValidCid]"
    />

    <!-- License Chooser -->
    <v-select
      v-model="selectedLicenseType"
      :items="licenseOptions"
      label="License"
      item-title="title"
      item-value="value"
      clearable
      hint="Choose a Creative Commons license or enter a custom URL"
      persistent-hint
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
    <template v-if="selectedLicenseType && selectedLicenseType !== 'custom'">
      <v-row>
        <v-col cols="6">
          <v-select
            v-model="licenseVersion"
            :items="licenseVersionOptions"
            label="Version"
            item-title="title"
            item-value="value"
          />
        </v-col>
        <v-col cols="6">
          <v-autocomplete
            v-model="licenseJurisdiction"
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
      v-if="selectedLicenseType === 'custom'"
      v-model="customLicenseUrl"
      label="License URL"
      hint="Enter the full URL to the license"
      persistent-hint
      placeholder="https://example.com/license"
    />

    <!-- Attribution field (shown when any license is selected) -->
    <v-text-field
      v-if="selectedLicenseType"
      v-model="licenseAttribution"
      label="Attribution (Optional)"
      hint="Credit the original creator, e.g. 'Music by Artist Name'"
      persistent-hint
    />

    <!-- Advanced: year, format, license, etc. -->
    <v-dialog
      v-model="openAdvanced"
      width="auto"
    >
      <template #activator="{props: activatorProps}">
        <v-btn
          v-bind="activatorProps"
          :disabled="!Boolean(selectedContentCategory)"
          rounded="0"
          text="Advanced"
          variant="outlined"
          class="mb-4"
          block
        ></v-btn>
      </template>
      <v-card
        v-if="selectedContentCategory && releaseItem.metadata"
        width="480px"
        max-height="620px"
        class="pa-8 ma-auto overflow-y-auto"
        color="black"
      >
        <p class="text-subtitle mb-6 text-center">
          Additional metadata options
        </p>

        <template
          v-for="[fieldName, fieldConfig] in Object.entries(selectedContentCategory)"
          :key="fieldName"
        >
          <!-- Skip hidden/auto-managed fields and primary fields shown above -->
          <template v-if="!HIDDEN_METADATA_FIELDS.includes(fieldName) && !PRIMARY_METADATA_FIELDS.includes(fieldName)">
            <v-select
              v-if="(fieldConfig as any).options"
              :items="(fieldConfig as any).options"
              :label="formatFieldLabel(fieldName)"
              :model-value="String((releaseItem.metadata && releaseItem.metadata[fieldName]) || '')"
              @update:model-value="(v) => handleChangeMetadataField(fieldName, v)"
            />
            <v-text-field
              v-else
              :label="formatFieldLabel(fieldName)"
              :model-value="String((releaseItem.metadata && releaseItem.metadata[fieldName]) || '')"
              :type="(fieldConfig as any).type || 'text'"
              @update:model-value="(v) => handleChangeMetadataField(fieldName, v)"
            >
              <template #append-inner>
                <v-tooltip
                  location="top"
                  :text="(fieldConfig as any).description || ''"
                >
                  <template #activator="{props: tooltipProps}">
                    <v-icon
                      size="small"
                      v-bind="tooltipProps"
                      color="grey-lighten-1"
                      icon="$help-circle-outline"
                    ></v-icon>
                  </template>
                </v-tooltip>
              </template>
            </v-text-field>
          </template>
        </template>

        <!-- Quality Tiers Section (for music) -->
        <template v-if="isMusicCategory">
          <v-divider class="my-4" />
          <p class="text-subtitle-2 mb-3">Quality Tiers</p>
          <p class="text-caption text-grey mb-3">
            Manage different quality versions (FLAC, MP3, etc.)
          </p>

          <template v-for="(tierCid, tierName) in qualityTiers" :key="tierName">
            <v-row dense class="align-center mb-2">
              <v-col cols="3">
                <v-chip
                  size="small"
                  :color="getTierColor(tierName)"
                  variant="tonal"
                >
                  {{ formatTierName(tierName) }}
                </v-chip>
              </v-col>
              <v-col cols="7">
                <v-text-field
                  :model-value="tierCid"
                  :label="`${formatTierName(tierName)} CID`"
                  density="compact"
                  hide-details
                  :rules="[rules.isValidCid]"
                  @update:model-value="(v) => updateTierCid(tierName, v)"
                />
              </v-col>
              <v-col cols="2" class="d-flex justify-end">
                <v-btn
                  icon
                  size="small"
                  variant="text"
                  color="error"
                  @click="removeTier(tierName)"
                >
                  <v-icon size="small">$close</v-icon>
                </v-btn>
              </v-col>
            </v-row>
          </template>

          <!-- Add new tier -->
          <v-row dense class="mt-2">
            <v-col cols="4">
              <v-select
                v-model="newTierName"
                :items="availableTiers"
                label="Add tier"
                density="compact"
                hide-details
                item-title="title"
                item-value="value"
              />
            </v-col>
            <v-col cols="5">
              <v-text-field
                v-model="newTierCid"
                label="CID"
                density="compact"
                hide-details
                :disabled="!newTierName"
                :rules="[rules.isValidCid]"
              />
            </v-col>
            <v-col cols="3" class="d-flex justify-end">
              <v-btn
                size="small"
                variant="tonal"
                color="success"
                :disabled="!newTierName || !newTierCid"
                @click="addTier"
              >
                <v-icon start size="small">$plus</v-icon>
                Add
              </v-btn>
            </v-col>
          </v-row>
        </template>

        <v-btn
          rounded="0"
          text="Save"
          color="primary"
          block
          class="mt-4"
          @click="openAdvanced = false"
        />
      </v-card>
    </v-dialog>
    <v-btn
      rounded="0"
      color="primary"
      type="submit"
      block
      text="Submit"
      :disabled="!readyToSave || addReleaseMutation.isPending.value"
      :loading="addReleaseMutation.isPending.value"
    />
  </v-form>
</template>

<script setup lang="ts">
import {cid} from 'is-ipfs';
import {computed, onMounted, ref, watch} from 'vue';
import type { ReleaseItem } from '/@/types';
import type { ReleaseData } from '@riffcc/citadel-sdk';
import {
  useAddReleaseMutation,
  useEditReleaseMutation,
  useContentCategoriesQuery,
  useGetStructuresQuery,
  useAddStructureMutation,
  useEditStructureMutation,
  useGetReleasesQuery
} from '/@/plugins/lensService/hooks';
// import { StringMatch, StringMatchMethod } from '@peerbit/document';

const props = defineProps<{
  initialData?: ReleaseItem;
  mode?: 'create' | 'edit';
}>();

const emit = defineEmits<{
  (e: 'submit', data: ReleaseData): void;
  (e: 'update:success', message: string): void;
  (e: 'update:error', message: string): void;
}>();

const {
  data: contentCategories,
} = useContentCategoriesQuery();


const formRef = ref();
const openAdvanced = ref<boolean>();

const releaseItem = ref<Partial<ReleaseItem>>({});

// TV-specific state
const selectedSeriesId = ref<string>('');
const seriesSearchText = ref<string>('');
const seasonNumber = ref<number>(1);
const episodeNumber = ref<number>(1);
const selectedSeasonId = ref<string>('');

// Artist association state
const selectedArtistId = ref<string>('');
const artistSearchText = ref<string>('');

// License state
const selectedLicenseType = ref<string>('');
const licenseVersion = ref<string>('4.0');
const licenseJurisdiction = ref<string>('');
const licenseAttribution = ref<string>('');
const customLicenseUrl = ref<string>('');

// Quality tier state
const qualityTiers = ref<Record<string, string>>({});
const newTierName = ref<string>('');
const newTierCid = ref<string>('');

// All available quality tier options
const allTierOptions = [
  { value: 'lossless', title: 'Lossless (FLAC/WAV)' },
  { value: 'opus', title: 'Opus' },
  { value: 'mp3_320', title: 'MP3 320kbps' },
  { value: 'mp3_v0', title: 'MP3 V0 (~245kbps)' },
  { value: 'mp3_256', title: 'MP3 256kbps' },
  { value: 'mp3_vbr', title: 'MP3 VBR' },
  { value: 'mp3_192', title: 'MP3 192kbps' },
  { value: 'ogg', title: 'Ogg Vorbis' },
  { value: 'aac', title: 'AAC/M4A' },
];

// Tiers that haven't been added yet
const availableTiers = computed(() =>
  allTierOptions.filter(t => !qualityTiers.value[t.value])
);

// Check if release has quality tiers (from metadata or imported)
const hasQualityTiers = computed(() => {
  // Show if we have existing tiers OR if we're in music category (to allow adding)
  return Object.keys(qualityTiers.value).length > 0 || props.mode === 'edit';
});

// Format tier name for display
function formatTierName(tier: string): string {
  const option = allTierOptions.find(t => t.value === tier);
  return option?.title || tier.toUpperCase();
}

// Get color for tier chip
function getTierColor(tier: string): string {
  switch (tier) {
    case 'lossless': return 'purple';
    case 'opus': return 'blue';
    case 'mp3_320':
    case 'mp3_v0': return 'green';
    case 'mp3_256':
    case 'mp3_vbr': return 'teal';
    case 'mp3_192': return 'orange';
    case 'ogg': return 'cyan';
    case 'aac': return 'amber';
    default: return 'grey';
  }
}

// Update a tier CID
function updateTierCid(tier: string, cid: string) {
  qualityTiers.value = { ...qualityTiers.value, [tier]: cid };
}

// Remove a tier
function removeTier(tier: string) {
  const { [tier]: _, ...rest } = qualityTiers.value;
  qualityTiers.value = rest;
}

// Add a new tier
function addTier() {
  if (newTierName.value && newTierCid.value) {
    qualityTiers.value = { ...qualityTiers.value, [newTierName.value]: newTierCid.value };
    newTierName.value = '';
    newTierCid.value = '';
  }
}

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

// Hidden metadata fields that are auto-managed (not shown anywhere)
const HIDDEN_METADATA_FIELDS = ['trackMetadata', 'type', 'artistId', 'artist', 'license'];

// Primary fields shown in the main form (not in Advanced dialog)
const PRIMARY_METADATA_FIELDS = ['albumTitle'];

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
  isValidCid: (v: string) => !v || cid(v) || 'Please enter a valid CID.',
  positiveNumber: (v: number) => v > 0 || 'Must be a positive number.',
};

// Check if TV category is selected (moved before structuresQuery to avoid circular dependency)
const isTVCategory = computed(() => {
  const catId = releaseItem.value.categoryId;
  // Match by hash ID or by slug (categoryId field on the category object)
  const category = contentCategories.value?.find(c => c.id === catId || c.categoryId === catId);
  const isTV = category?.categoryId === 'tv-shows' || category?.displayName === 'TV Shows';
  return isTV;
});

// Check if Music category is selected
const isMusicCategory = computed(() => {
  const catId = releaseItem.value.categoryId;
  // Match by hash ID or by slug (categoryId field on the category object)
  const category = contentCategories.value?.find(c => c.id === catId || c.categoryId === catId);
  return category?.categoryId === 'music' || category?.displayName === 'Music';
});

const addReleaseMutation = useAddReleaseMutation({
  onSuccess: () => {
    emit('update:success', 'Release added successfully!');
    clearForm();
  },
  onError: (e) => {
    console.error('Error on adding release:', e);
    emit('update:error', `Error on adding release: ${e.message.slice(0, 200)}`);
  },
});

const editReleaseMutation = useEditReleaseMutation({
  onSuccess: () => {
    emit('update:success', 'Release edited successfully!');
    clearForm();
  },
  onError: (e) => {
    console.error('Error in editing release:', e);
    emit('update:error', `Error on editing release: ${e.message.slice(0, 200)}`);
  },
});


// Fetch structures for TV shows (both series and seasons)
const structuresQuery = useGetStructuresQuery({
  searchOptions: {
    // Fetch all structures, we'll filter them in computed properties
    fetch: 1000,
  },
  enabled: computed(() => {
    const enabled = isTVCategory.value;
    console.log('Structures query enabled:', enabled, 'isTVCategory:', isTVCategory.value);
    return enabled;
  }),
});

// Fetch all releases to find artists (for music category)
const artistsQuery = useGetReleasesQuery({
  enabled: computed(() => isMusicCategory.value),
});

// Get list of artists for the dropdown
const artistItems = computed(() => {
  if (!artistsQuery.data.value) return [];

  return artistsQuery.data.value
    .filter((r: any) => r.metadata?.type === 'artist')
    .map((r: any) => ({
      value: r.id,
      title: r.name,
    }));
});

// Watch for structure query errors
watch(() => structuresQuery.error.value, (error) => {
  if (error) {
    console.error('Failed to fetch structures:', error);
  }
});

const addStructureMutation = useAddStructureMutation({
  onSuccess: (response) => {
    console.log('Structure created successfully:', response);
  },
  onError: (e) => {
    console.error('Error creating structure:', e);
  },
});

const editStructureMutation = useEditStructureMutation({
  onSuccess: (response) => {
    console.log('Structure updated successfully:', response);
  },
  onError: (e) => {
    console.error('Error updating structure:', e);
  },
});

const contentCategoriesItems = computed(() => {
  if (!contentCategories.value) {
    console.log('[ReleaseForm] No content categories available');
    return [];
  }

  console.log('[ReleaseForm] Processing categories:', {
    total: contentCategories.value.length,
    categories: contentCategories.value
  });

  // Map all categories - removed siteAddress filter for single-node deployments
  const items = contentCategories.value.map(item => ({
    id: item.id,
    value: item.id,
    title: item.displayName || item.name,
  }));

  console.log('[ReleaseForm] Mapped category items:', items);
  return items;
});

// Get list of series for autocomplete
const seriesItems = computed(() => {
  console.log('Computing series items, structures query state:', {
    isLoading: structuresQuery.isLoading.value,
    isError: structuresQuery.isError.value,
    error: structuresQuery.error.value,
    dataLength: structuresQuery.data.value?.length
  });

  if (!structuresQuery.data.value) {
    console.log('No structures data available');
    return [];
  }

  console.log('All structures:', structuresQuery.data.value);

  const series = structuresQuery.data.value
    .filter((s: any) => s.type === 'series')
    .map((s: any) => ({
      value: s.id,
      title: s.name,
    }));
  console.log('Filtered series for autocomplete:', series);
  return series;
});

// Check if we should show the "Create new series" option
const shouldShowCreateSeriesOption = computed(() => {
  if (!seriesSearchText.value || seriesSearchText.value.length < 2) return false;
  // Check if series already exists
  const exists = structuresQuery.data.value?.some(
    (s: any) => s.name.toLowerCase() === seriesSearchText.value.toLowerCase()
  );
  return !exists;
});

// Check if we should show the "Create new artist" option
const shouldShowCreateArtistOption = computed(() => {
  if (!artistSearchText.value || artistSearchText.value.length < 2) return false;
  // Check if artist already exists
  const exists = artistItems.value.some(
    (a: any) => a.title.toLowerCase() === artistSearchText.value.toLowerCase()
  );
  return !exists;
});

const selectedContentCategory = computed(() => {
  if (!contentCategories.value || !releaseItem.value.categoryId) {
    console.log('Advanced button disabled: no categories or categoryId', {
      hasCategories: !!contentCategories.value,
      categoryId: releaseItem.value.categoryId
    });
    return null;
  }

  const targetItem = contentCategories.value.find(item => item.id === releaseItem.value.categoryId);
  if (!targetItem || !targetItem.metadataSchema) {
    console.log('Advanced button disabled: no matching category or metadataSchema', {
      targetItem,
      categoryId: releaseItem.value.categoryId
    });
    return null;
  }

  // metadataSchema should already be parsed by the query hook
  // If it's still a string, parse it
  if (typeof targetItem.metadataSchema === 'string') {
    try {
      const parsedSchema = JSON.parse(targetItem.metadataSchema);
      return parsedSchema;
    } catch (e) {
      console.error('Failed to parse metadata schema:', e, targetItem.metadataSchema);
      return null;
    }
  }

  return targetItem.metadataSchema;
});

const handleChangeMetadataField = (fieldName: string, value: string | null) => {
  if (!releaseItem.value.metadata) {
    releaseItem.value.metadata = {};
  }

  // Only update fields that are defined in the schema
  if (selectedContentCategory.value && fieldName in selectedContentCategory.value) {
    releaseItem.value.metadata = {
      ...releaseItem.value.metadata,
      [fieldName]: value || '',
    };
  }
};

onMounted(() => {
  if(props.initialData) {
    releaseItem.value = {
      ...releaseItem.value,
      ...props.initialData,
      metadata: props.initialData.metadata || {},
    };

    // If editing a TV episode, initialize the TV-specific fields
    if (props.initialData.metadata?.seriesId) {
      selectedSeriesId.value = props.initialData.metadata.seriesId as string;
      seasonNumber.value = (props.initialData.metadata.seasonNumber as number) || 1;
      episodeNumber.value = (props.initialData.metadata.episodeNumber as number) || 1;
      selectedSeasonId.value = (props.initialData.metadata.seasonId as string) || '';

      console.log('Initialized TV episode fields:', {
        seriesId: selectedSeriesId.value,
        seasonNumber: seasonNumber.value,
        episodeNumber: episodeNumber.value,
        seasonId: selectedSeasonId.value
      });
    }

    // If editing a music release, initialize the artist field
    if (props.initialData.metadata?.artistId) {
      selectedArtistId.value = props.initialData.metadata.artistId as string;
      console.log('Initialized artist ID:', selectedArtistId.value);
    }

    // If editing, initialize the license fields
    if (props.initialData.metadata?.license) {
      const license = typeof props.initialData.metadata.license === 'string'
        ? JSON.parse(props.initialData.metadata.license)
        : props.initialData.metadata.license;

      // Check if it's a custom URL (type is 'custom' or has a url but type doesn't match known CC types)
      if (license.type === 'custom' || (license.url && !licenseOptions.some(o => o.value === license.type))) {
        selectedLicenseType.value = 'custom';
        customLicenseUrl.value = license.url || '';
      } else {
        selectedLicenseType.value = license.type || '';
        licenseVersion.value = license.version || '4.0';
        licenseJurisdiction.value = license.jurisdiction || '';
      }
      licenseAttribution.value = license.attribution || '';
      console.log('Initialized license:', license);
    }

    // If editing, initialize quality tiers
    if (props.initialData.metadata?.qualityTiers) {
      const tiers = typeof props.initialData.metadata.qualityTiers === 'string'
        ? JSON.parse(props.initialData.metadata.qualityTiers)
        : props.initialData.metadata.qualityTiers;
      qualityTiers.value = tiers;
      console.log('Initialized quality tiers:', tiers);
    }
  }
});

// Ensure metadata is preserved when switching categories
watch(() => releaseItem.value.categoryId, () => {
  if (!releaseItem.value.metadata) {
    releaseItem.value.metadata = {};
  }
});

// Watch for series selection changes
watch(selectedSeriesId, (newId) => {
  console.log('Selected series ID changed:', newId);
  if (newId) {
    // When a series is selected, trigger season handling
    handleSeasonChange();
  }
});

// Watch for artist selection changes
watch(selectedArtistId, (newId) => {
  console.log('Selected artist ID changed:', newId);
  if (!releaseItem.value.metadata) {
    releaseItem.value.metadata = {};
  }
  if (newId) {
    releaseItem.value.metadata.artistId = newId;
  } else {
    delete releaseItem.value.metadata.artistId;
  }
});

const readyToSave = computed(() => {
  if (
    releaseItem.value.name &&
    releaseItem.value.contentCID &&
    releaseItem.value.categoryId &&
    formRef.value.isValid
  ) {
    return releaseItem.value;
  }
  return undefined;
});

// Create a new TV series
const createNewSeries = async () => {
  if (!seriesSearchText.value) return;

  console.log('Creating new series:', seriesSearchText.value);

  try {
    const response = await addStructureMutation.mutateAsync({
      name: seriesSearchText.value,
      type: 'series',
      description: '',
      itemIds: [],
    });

    console.log('Series creation response:', response);

    if (response.success) {
      // The response might have the ID in different fields
      const newSeriesId = response.id || response.hash;
      if (newSeriesId) {
        selectedSeriesId.value = newSeriesId;
        console.log('Series created with ID:', newSeriesId);
        // Refetch structures to include the new series
        await structuresQuery.refetch();

        // Clear the search text
        seriesSearchText.value = '';
      } else {
        console.error('No ID returned from series creation');
      }
    } else {
      console.error('Series creation failed:', response);
    }
  } catch (error) {
    console.error('Failed to create series:', error);
    emit('update:error', `Failed to create series: ${error.message}`);
  }
};

// Create a new artist (as a release with type: 'artist')
// TODO: Artists should be Structures, not releases - this is a temporary workaround
const createNewArtist = async () => {
  if (!artistSearchText.value) return;

  try {
    // Find the music category ID
    const musicCategory = contentCategories.value?.find(
      c => c.categoryId === 'music' || c.displayName === 'Music'
    );
    if (!musicCategory) {
      emit('update:error', 'Music category not found');
      return;
    }

    const response = await addReleaseMutation.mutateAsync({
      name: artistSearchText.value,
      categoryId: musicCategory.id,
      contentCID: 'bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku', // Empty content placeholder
      metadata: { type: 'artist' },
    });

    if (response.success && response.id) {
      selectedArtistId.value = response.id;
      handleChangeMetadataField('artistId', response.id);
      // Refetch to include the new artist
      await artistsQuery.refetch();
      artistSearchText.value = '';
    }
  } catch (error: any) {
    emit('update:error', `Failed to create artist: ${error.message}`);
  }
};

// Handle season change - check if we need to create a new season structure
const handleSeasonChange = async () => {
  if (!selectedSeriesId.value || !seasonNumber.value) return;

  console.log('Handling season change:', { seriesId: selectedSeriesId.value, seasonNumber: seasonNumber.value });

  // Check if season structure already exists
  const seasonName = `Season ${seasonNumber.value}`;
  const existingSeason = structuresQuery.data.value?.find(
    (s: any) => {
      if (s.type !== 'season' || s.parentId !== selectedSeriesId.value) return false;

      // Check by metadata seasonNumber first, then by name
      if (s.metadata) {
        try {
          const meta = typeof s.metadata === 'string' ? JSON.parse(s.metadata) : s.metadata;
          if (meta.seasonNumber === seasonNumber.value) return true;
        } catch (e) {
          // Invalid metadata, fall through to name check
        }
      }

      return s.name === seasonName;
    }
  );

  if (existingSeason) {
    console.log('Found existing season:', existingSeason);
    selectedSeasonId.value = existingSeason.id;
  } else {
    console.log('Creating new season:', seasonName);
    try {
      // Create new season structure
      const response = await addStructureMutation.mutateAsync({
        name: seasonName,
        type: 'season',
        parentId: selectedSeriesId.value,
        order: seasonNumber.value,
        itemIds: [],
        metadata: JSON.stringify({ seasonNumber: seasonNumber.value }),
      });

      console.log('Season creation response:', response);

      if (response.success) {
        const newSeasonId = response.id || response.hash;
        if (newSeasonId) {
          selectedSeasonId.value = newSeasonId;
          console.log('Season created with ID:', newSeasonId);
          await structuresQuery.refetch();
        } else {
          console.error('No ID returned from season creation');
        }
      } else {
        console.error('Season creation failed:', response);
      }
    } catch (error) {
      console.error('Failed to create season:', error);
    }
  }
};

const handleOnSubmit = async () => {
  if (!readyToSave.value) return;

  // Clone the data to avoid readonly proxy issues
  const data = {
    ...readyToSave.value,
    metadata: { ...readyToSave.value.metadata },
  };

  // If this is a TV episode, ensure we have the proper structure hierarchy
  if (isTVCategory.value && selectedSeriesId.value) {
    console.log('Processing TV episode submission:', {
      seriesId: selectedSeriesId.value,
      seasonNumber: seasonNumber.value,
      episodeNumber: episodeNumber.value
    });

    // Ensure season structure exists
    await handleSeasonChange();

    // Add episode metadata
    if (!data.metadata) data.metadata = {};
    data.metadata.seasonNumber = seasonNumber.value;
    data.metadata.episodeNumber = episodeNumber.value;
    data.metadata.seriesId = selectedSeriesId.value;
    data.metadata.seasonId = selectedSeasonId.value;

    console.log('Episode metadata set:', data.metadata);
  }

  // For music releases, ensure artistId is in metadata
  if (isMusicCategory.value && selectedArtistId.value) {
    if (!data.metadata) data.metadata = {};
    data.metadata.artistId = selectedArtistId.value;
  }

  // Add license to metadata if selected
  console.log('[ReleaseForm] License state:', {
    selectedLicenseType: selectedLicenseType.value,
    licenseVersion: licenseVersion.value,
    licenseJurisdiction: licenseJurisdiction.value,
    licenseAttribution: licenseAttribution.value,
    customLicenseUrl: customLicenseUrl.value,
  });

  if (selectedLicenseType.value) {
    if (!data.metadata) data.metadata = {};

    if (selectedLicenseType.value === 'custom') {
      // Custom URL license
      data.metadata.license = JSON.stringify({
        type: 'custom',
        url: customLicenseUrl.value,
        ...(licenseAttribution.value ? { attribution: licenseAttribution.value } : {}),
      });
    } else {
      // CC license with version and optional jurisdiction
      data.metadata.license = JSON.stringify({
        type: selectedLicenseType.value,
        version: licenseVersion.value,
        ...(licenseJurisdiction.value ? { jurisdiction: licenseJurisdiction.value } : {}),
        ...(licenseAttribution.value ? { attribution: licenseAttribution.value } : {}),
      });
    }
    console.log('[ReleaseForm] License to save:', data.metadata.license);
  } else {
    // Clear license if none selected
    if (data.metadata?.license) {
      delete data.metadata.license;
    }
  }

  // Add quality tiers to metadata if any exist
  if (Object.keys(qualityTiers.value).length > 0) {
    if (!data.metadata) data.metadata = {};
    data.metadata.qualityTiers = JSON.stringify(qualityTiers.value);
    console.log('[ReleaseForm] Quality tiers to save:', data.metadata.qualityTiers);
  } else {
    // Clear quality tiers if none
    if (data.metadata?.qualityTiers) {
      delete data.metadata.qualityTiers;
    }
  }

  console.log('[ReleaseForm] Final metadata:', data.metadata);

  if (props.mode === 'edit' && data.id) {
    const response = await editReleaseMutation.mutateAsync({
      id: data.id,
      name: data.name!,
      categoryId: data.categoryId!,
      contentCID: data.contentCID!,
      thumbnailCID: data.thumbnailCID,
      metadata: data.metadata,
      siteAddress: data.siteAddress!,
      postedBy: data.postedBy as any,
    });

    // If TV episode and successful, add to season's itemIds
    // Use the original episode ID (data.id) not response.id since we're editing
    if (isTVCategory.value && selectedSeasonId.value && response.success) {
      await updateSeasonWithEpisode(data.id);
    }
  } else {
    const response = await addReleaseMutation.mutateAsync({
      name: data.name!,
      categoryId: data.categoryId!,
      contentCID: data.contentCID!,
      thumbnailCID: data.thumbnailCID,
      metadata: data.metadata,
    });

    // If TV episode and successful, add to season's itemIds
    if (isTVCategory.value && selectedSeasonId.value && response.success) {
      await updateSeasonWithEpisode(response.id!);
    }
  }
};

// Add episode to season's itemIds
const updateSeasonWithEpisode = async (episodeId: string) => {
  if (!selectedSeasonId.value) return;

  // Fetch current season structure
  const seasons = await structuresQuery.refetch();
  const season = seasons.data.value?.find((s: any) => s.id === selectedSeasonId.value);

  if (season) {
    // Check if episode is already in itemIds
    const currentItemIds = season.itemIds || [];
    if (!currentItemIds.includes(episodeId)) {
      console.log('Adding episode to season itemIds:', { seasonId: season.id, episodeId });
      // Update season with new episode
      await editStructureMutation.mutateAsync({
        ...season,
        itemIds: [...currentItemIds, episodeId],
      });
      console.log('Episode added to season successfully');
    } else {
      console.log('Episode already in season itemIds');
    }
  } else {
    console.error('Season not found:', selectedSeasonId.value);
  }
};

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

const formatFieldLabel = (fieldName: string): string => {
  return fieldLabelMap[fieldName] || fieldName;
};

const clearForm = () => {
  releaseItem.value = {
    id: '',
    name: '',
    contentCID: '',
    categoryId: '',
    metadata: {},
    siteAddress: '',
  };
  // Clear license fields
  selectedLicenseType.value = '';
  licenseVersion.value = '4.0';
  licenseJurisdiction.value = '';
  licenseAttribution.value = '';
  customLicenseUrl.value = '';
  // Clear artist field
  selectedArtistId.value = '';
  artistSearchText.value = '';
  // Clear TV fields
  selectedSeriesId.value = '';
  seriesSearchText.value = '';
  seasonNumber.value = 1;
  episodeNumber.value = 1;
  selectedSeasonId.value = '';

  formRef.value?.resetValidation();
  formRef.value?.reset();
};
</script>
