<template>
  <div>
    <section class="hero">
      <div class="hero-body">
        <div class="container">
          <div class="title">Devices</div>
          <div class="subtitle">Manage this instance's devices settings.</div>
        </div>
      </div>
    </section>

    <section class="section">
      <div class="container">
        <device v-for="(item, index) in devices"
                v-bind:device="item"
                v-bind:id="index"
                v-bind:key="index"></device>
      </div>
    </section>
  </div>
</template>

<script>
import { mapState } from 'vuex'
import Device from '@/components/Device'

export default {
  computed: mapState({
    devices: state => state.devices.info
  }),
  components: {
    Device
  },
  mounted () {
    this.$store.dispatch('devices/load')
  }
}
</script>

<style scoped>
</style>
