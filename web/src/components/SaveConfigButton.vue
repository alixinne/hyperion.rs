<template>
  <div>
    <b-button :disabled="processing"
              class="is-success"
              label="Save"
              v-on:click="saveConfig"></b-button>
  </div>
</template>

<script>
import api from '@/api'

export default {
  data () {
    return {
      processing: false
    }
  },
  methods: {
    async saveConfig () {
      this.processing = true

      try {
        await api.config.save()
        this.$buefy.toast.open({
          message: 'Config saved successfully!',
          type: 'is-success'
        })
      } catch (error) {
        // Error display handled by axios interceptor
      }

      this.processing = false
    }
  }
}
</script>

<style scoped>
</style>
