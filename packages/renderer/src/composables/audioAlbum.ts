import {ref, watch} from 'vue';

export type AudioTrack = {
  index: number;
  cid: string;
  title: string;
  album: string;
  artist?: string;
  duration?: string;
  size?: string;
};

const albumFiles = ref<AudioTrack[]>([]);
const activeTrack = ref<AudioTrack>();
const repeat = ref(false);
const shuffle = ref(false);

const toggleRepeat = () => (repeat.value ? (repeat.value = false) : (repeat.value = true));
const toggleShuffle = () => (shuffle.value ? (shuffle.value = false) : (shuffle.value = true));

const handlePlay = (index: number) => {
  activeTrack.value = {
    index,
    cid: albumFiles.value[index].cid,
    title: albumFiles.value[index].title,
    album: albumFiles.value[index].album,
    artist: albumFiles.value[index].artist,
    duration: albumFiles.value[index].duration,
  };
};

const handlePrevious = () => {
  if (activeTrack.value && activeTrack.value.index > 0) {
    handlePlay(activeTrack.value.index - 1);
  }
};

const handleNext = () => {
  if (activeTrack.value) {
    if (shuffle.value) {
      const randomIndex = Math.floor(Math.random() * albumFiles.value.length);
      handlePlay(randomIndex !== activeTrack.value.index ? randomIndex : randomIndex + 1);
    } else {
      if (activeTrack.value.index !== albumFiles.value.length - 1) {
        handlePlay(activeTrack.value.index + 1);
      } else {
        if (repeat.value) {
          handlePlay(0);
        } else {
          activeTrack.value = undefined;
        }
      }
    }
  }
};

const handleOnClose = () => (activeTrack.value = undefined);

watch(activeTrack, (t) => {
  if ('mediaSession' in navigator) {
    if (navigator.mediaSession.metadata && t) {
      navigator.mediaSession.metadata.title = t.title;
      navigator.mediaSession.metadata.album = t.album;
      if (t.artist) {
        navigator.mediaSession.metadata.artist = t.artist;
      }
    }
  }
});

export const useAudioAlbum = () => {
  return {
    albumFiles,
    activeTrack,
    repeat,
    shuffle,
    handlePlay,
    handlePrevious,
    handleNext,
    handleOnClose,
    toggleRepeat,
    toggleShuffle,
  };
};
