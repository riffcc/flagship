<template>
    <div class="profile-page pa-5">
        <v-container>
            <v-row justify="center" align="start" class="gap-4">

                <v-col cols="12" md="7">
                    <h2 class="text-h5 font-weight-bold">Your Profile</h2>

                    <div class="pic-section d-flex flex-column align-center pa-4 margin-left=">
                        <v-avatar size="150" class="mb-4">
                            <img :src="profilePic || placeholder" class="profile-pic" alt="Profile" />
                        </v-avatar>
                        <v-file-input v-model="profilePic" accept="image/*" capture="environment"
                            prepend-icon="mdi-camera" label="Take or Upload Photo" class="my-2 custom-file-input" />
                    </div>
                    <v-text-field v-model="name" label="Full Name" outlined dense />
                    <v-textarea v-model="bio" label="Bio" outlined rows="3" dense />
                    <v-btn color="primary" class="mt-4" @click="saveProfile">Save</v-btn>
                </v-col>
            </v-row>
        </v-container>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useOrbiter } from '/@/plugins/orbiter/utils';
import { suivre as follow } from '@constl/vue';
const { orbiter } = useOrbiter();
const accountName = follow(orbiter.listenForNameChange.bind(orbiter));

const name = ref('')
const bio = ref('')
const profilePic = ref()
const placeholder = import('/@/assets/undraw/undraw_profile_pic_re_iwgo.svg')
const saveProfile = () => {
    orbiter.changeName({ name: name.value, language: 'english' })
}
</script>

<style scoped>
.profile-page {
    background-color: #141212;
    color: white;
    min-height: 30vh;
    text-align: center;

}

.form-section,
.pic-section {
    border-radius: 8px;
    width: 100%;
}

.profile-pic {
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center;
}

.custom-file-input {
    background-color: transparent !important;
    border: none !important;
    box-shadow: none !important;
    width: 200px;
}

.info-card {
    background: rgb(60, 59, 59)
}
</style>
