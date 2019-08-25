<template>
  <div class="panel">
    <div class="panel-heading">
      <div class="level is-mobile">
        <div class="level-left">
          <div class="level-item">
            <b-field v-bind:type="$v.deviceName.$error ? 'is-danger' : ''">
              <b-input placeholder="Device name"
                       v-model="$v.deviceName.$model">
              </b-input>
            </b-field>
          </div>
        </div>
        <div class="level-right">
          <div class="level-item">
            <b-switch v-bind:disabled="busy" v-model="deviceEnabled"></b-switch>
          </div>
        </div>
      </div>
    </div>
    <div class="panel-block">
      <p>{{ device.endpoint }}</p>
    </div>
  </div>
</template>

<script>
import _ from 'lodash'
import { required, minLength } from 'vuelidate/lib/validators'

export default {
  data: function () {
    return { busy: false, deviceName: null }
  },
  props: {
    id: Number,
    device: {
      type: Object
    }
  },
  validations: {
    deviceName: {
      required,
      minLength: minLength(4)
    }
  },
  mounted () {
    this.deviceName = this.device.name
  },
  watch: {
    deviceName: _.debounce(function (name) {
      if (this.$v.$anyError) {
        return
      }

      this.busy = true
      this.$store.dispatch('devices/patch', { id: this.id, patch: { name } }).finally(() => {
        this.busy = false
      })
    }, 500)
  },
  computed: {
    deviceEnabled: {
      get () {
        return this.device.enabled
      },
      set (enabled) {
        this.busy = true
        this.$store.dispatch('devices/patch', { id: this.id, patch: { enabled } }).finally(() => {
          this.busy = false
        })
      }
    }
  }
}
</script>

<style scoped>
</style>
