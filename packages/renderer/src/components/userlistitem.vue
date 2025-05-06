<template>
    <v-list-item class="d-flex align-center gap-4">
        <v-badge dot :color="getStatusColor(lastActive)" location="bottom right" offset-x="4" offset-y="4">
            <v-avatar size="60">
                <v-icon size="40">mdi-account</v-icon>
            </v-avatar>
        </v-badge>
        <span class="text-subtitle-1 font-weight-medium" style="transform: translateY(-10px);display:inline-block;">
            {{ Names }}
        </span>
        <div :style="{ color: getStatusColor(lastActive), marginLeft: '70px', fontSize: '12px', marginTop: '-10px' }">
            {{ getStatusText(lastActive) }}
        </div>
    </v-list-item>
</template>
<script setup lang="ts">
import { useOrbiter } from '/@/plugins/orbiter/utils';
import { suivre as follow } from '@constl/vue';
import { computed, ref, watchEffect } from 'vue';
import { selectTranslation } from '/@/utils';

const Names = computed(() => {
    return selectTranslation(names.value) || 'Anonymous';
})
const props = defineProps<{ userid: string; lastActive: number | undefined }>();
const { orbiter } = useOrbiter();
const names = follow(orbiter.listenForNameChange.bind(orbiter),{ accountId: computed(()=>props.userid) });
function getStatusColor(lastActive: number | undefined) {
    if (lastActive === undefined)
        return 'white'
    const now = Date.now()
    const diff = (now - lastActive) / 60000
    if (diff <= 1) return 'lime';
    else if (diff <= 5) return 'yellow';
    else return 'white';
}
function getStatusText(lastActive: number | undefined) {
    if (lastActive === undefined)
        return 'inactive'
    const now = Date.now();
    const diff = (now - lastActive) / 60000;
    if (diff <= 1) return 'Active just now';
    else if (diff <= 5) return `Active ${Math.floor(diff)} min ago`;
    else return 'Last active';
}
</script>
