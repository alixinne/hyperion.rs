<template>
  <span>
    <span>hyperion.rs&nbsp;</span>
    <span v-if="info">
      {{ info.version }}@{{ info.hostname }}
    </span>
  </span>
</template>

<script>
import { mapState } from 'vuex'

export default {
  computed: mapState('server', [ 'info' ]),
  mounted () {
    this.$store.dispatch('server/load')
  }
}
</script>

<style scoped>
</style>
