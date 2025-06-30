<template>
  <div class="profile-page pa-5">
    <v-container>
      <v-row justify="center" align="start" class="gap-4">
        <v-col cols="12" md="7">
          <h2 class="text-h5 font-weight-bold">Your Profile</h2>

          <div class="pic-section d-flex flex-column align-center pa-4">
            <v-avatar size="150" class="mb-2">
              <img :src="orbiterprofilepic || placeholder" class="profile-pic" alt="Profile" />
            </v-avatar>
            <v-btn icon class="camera-icon-btn" @click="triggerFileInput">
              <v-icon color="white">mdi-camera</v-icon>
            </v-btn>
            <input ref="fileInput" type="file" accept="image/*" class="d-none" @change="handleFileChange" />
          </div>

          <v-text-field v-model="username" label="Full Name" outlined dense />
          <v-textarea v-model="bio" label="Bio" outlined rows="3" dense />
          <v-btn color="primary" class="mt-4" @click="saveProfile">Save</v-btn>

        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, watchEffect } from 'vue'
import { useOrbiter } from '/@/plugins/orbiter/utils';
import { suivre as follow } from '@constl/vue';

const { orbiter } = useOrbiter();
const username = ref('')
const saveProfile = async () => {
  await orbiter.changeName({
    name: username.value,
    language: 'english',
  })
  savebio()
}
const apiUsername = follow(orbiter.listenForNameChange.bind(orbiter))
watchEffect(() => {
  if (apiUsername.value && apiUsername.value)
    username.value = apiUsername.value.english;
})
const bio = ref('')
const savebio = async () => {
  await orbiter.changeBio({
    name: bio.value,
    language: 'english',
  })
}

const apiBio = follow(orbiter.listenFornBioChange.bind(orbiter))
watchEffect(() => {
  if (apiBio.value && apiBio.value) {
    bio.value = apiBio.value.english;
  }
})

const photo = follow(orbiter.listenForProfilePhotoChange.bind(orbiter));
const orbiterprofilepic = computed(() => {
  if (photo.value)
    return URL.createObjectURL(new Blob([photo.value]))
  else { return undefined }
})
const profilePicUrl = ref<string | null>(null)
const profilePic = ref<File | null>(null)
const placeholder = '/@/assets/undraw/undraw_profile_pic_re_iwgo.svg'

const fileInput = ref<HTMLInputElement | null>(null)
const triggerFileInput = () => {
  fileInput.value?.click()
}

const handleFileChange = (event: Event) => {
  const target = event.target as HTMLInputElement
  if (target.files && target.files[0]) {
    profilePic.value = target.files[0]
    profilePicUrl.value = URL.createObjectURL(target.files[0])
  }
}
const savePhoto = async () => {
  if (!profilePic.value) return
  const file = profilePic.value
  const arrayBuffer = await file.arrayBuffer()
  const uint8Array = new Uint8Array(arrayBuffer)

  await orbiter.changeProfilePhoto({
    image: {
      contenu: uint8Array,
      nomFichier: file.name,
    },

  })
}
watch(profilePic, savePhoto)
</script>

<style scoped>
.profile-page {
  background-color: #141212;
  color: white;
  min-height: 30vh;
  text-align: center;
}

.pic-section {
  border-radius: 8px;
  width: 100%;
  position: relative;
}

.profile-pic {
  width: 160px;
  height: 160px;
  border-radius: 50%;
  object-fit: cover;
  border: 6px solid #9c27b0;
}

.camera-icon-btn {
  margin-top: -30px;
  /* background-color: #9c27b0; */
  color: white;
  border-radius: 50%;
  width: 40px;
  height: 40px;
  margin-left: 15%;
}

.d-none {
  display: none;
}

.info-card {
  background: rgb(60, 59, 59);
}
</style>