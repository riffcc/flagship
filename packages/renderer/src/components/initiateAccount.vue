<template>
  <v-dialog
    v-model="dialog"
    max-width="800"
  >
    <template #activator="{props}">
      <slot
        name="activator"
        v-bind="{props}"
      ></slot>
    </template>
    <v-card>
      <v-card-title class="d-flex">
        Welcome!
        <v-spacer />
        <v-btn icon>
          <v-icon @click="() => (dialog = false)">mdi-close</v-icon>
        </v-btn>
      </v-card-title>
      <v-card-text>
        <p class="mb-4">What language is your name in?</p>
        <v-combobox
          v-model="userNameLanguage"
          :items="
            knownLanguageCodes.map(code => {
              return {title: nuchabäl.rubiChabäl({runuk: code}) || code, value: code};
            })
          "
          label="Language"
          hint="You can write any language name if your language is not in the suggestions list."
          variant="outlined"
        >
        </v-combobox>
        <p class="mb-4">What is your name?</p>
        <v-text-field
          v-model="userName"
          label="Your name"
          hint="It doesn't have to be your name. (Just be sure it isn't really somebody else's.)"
          variant="outlined"
        ></v-text-field>
      </v-card-text>
      <v-card-actions>
        <v-btn
          color="primary"
          block
          :disabled="!userNameLanguage || !userName"
          :loading="savingName"
          @click="saveName"
        >
          Next
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import {Nuchabäl} from 'nuchabal';
import {computed, ref} from 'vue';
import {useOrbiter} from '/@/plugins/peerbit/utils';

const {orbiter} = useOrbiter();
const nuchabäl = new Nuchabäl({});

const dialog = ref<boolean>(false);

const knownLanguageCodes = computed(() => nuchabäl.konojelChabäl);
const userNameLanguage = ref('English');
const userName = ref<string>();

const savingName = ref(false);
const saveName = async () => {
  if (!userNameLanguage.value || !userName.value)
    throw new Error('Name or language not specified.');
  savingName.value = true;
  const code: string =
    typeof userNameLanguage.value === 'string'
      ? userNameLanguage.value
      : userNameLanguage.value['value'];
  await orbiter.changeName({name: userName.value, language: code});
  await orbiter.constellation.profil.initialiser();
  savingName.value = false;
};
</script>
