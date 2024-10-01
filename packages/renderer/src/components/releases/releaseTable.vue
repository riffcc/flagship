<template>
  <v-sheet
    height="360px"
    class="d-flex flex-column"
  >
    <p class="text-h6 text-center mb-2">{{ title }}</p>
    <v-divider></v-divider>
    <v-table
      v-if="pins.length > 0"
      class="my-auto py-4"
    >
      <tbody>
        <tr
          v-for="pin in pins"
          :key="pin.node.id"
        >
          <td>
            <v-sheet
              width="42px"
              height="42px"
              class="d-flex"
            >
              <v-image
                v-if="pin.node.piece.details.thumbnailCid"
                cover
                :src="`https://${IPFS_GATEWAY}/ipfs/${pin.node.piece.details.thumbnailCid}`"
              ></v-image>
              <v-icon
                icon="fas fa-image"
                size="x-large"
                class="ma-auto"
              ></v-icon>
            </v-sheet>
          </td>
          <td>{{ pin.node.piece.name }}</td>
          <td>{{ pin.node.category.name }}</td>
          <td>
            {{ `${pin.node.piece.contentCid.substring(0,4)}...${pin.node.piece.contentCid.substring(pin.node.piece.contentCid.length - 4)}` }}
          </td>
          <td>
            <PinActions :pin="pin.node" />
          </td>
        </tr>
      </tbody>
    </v-table>
    <p
      v-else
      class="text-center ma-auto"
    >
      No pins found.
    </p>
  </v-sheet>
</template>

<script setup>
import { PinActions } from '@components';

defineProps({
  title: String,
  pins: Array,
});
import { IPFS_GATEWAY } from '@/config/constants';
</script>
