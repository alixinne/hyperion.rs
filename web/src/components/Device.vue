<template>
  <div class="panel">
    <div class="panel-heading">
      <div class="level is-mobile">
        <div class="level-left">
          <div class="level-item">
            <b-field v-bind:type="$v.deviceName.$error ? 'is-danger' : ''">
              <b-input placeholder="Device name"
                       v-model="$v.deviceName.$model"
                       v-on:input="updateDeviceName">
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
      <div class="columns device-field-columns is-vcentered">
        <div class="column device-field-label has-text-right-tablet is-one-fifth-tablet"><strong>Endpoint</strong></div>
        <div class="column">{{ device.endpoint }}</div>
      </div>
    </div>
    <div class="panel-block">
      <div class="columns device-field-columns is-vcentered">
        <div class="column device-field-label has-text-right-tablet is-one-fifth-tablet"><strong>Frequency</strong></div>
        <div class="column">
          <b-field>
            <b-slider :min="1" :max="120" ticks v-model="device.frequency"></b-slider>
          </b-field>
        </div>
      </div>
    </div>
    <div class="panel-block">
      <div class="columns device-field-columns is-vcentered">
        <div class="column device-field-label has-text-right-tablet is-one-fifth-tablet"><strong>Filter</strong></div>
        <div class="column">{{ device.filter }}</div>
      </div>
    </div>
    <div class="panel-block">
      <div class="columns device-field-columns is-vcentered">
        <div class="column device-field-label has-text-right-tablet is-one-fifth-tablet"><strong>Idle settings</strong></div>
        <div class="column">{{ device.idle }}</div>
      </div>
    </div>
    <div class="panel-block">
      <div class="columns device-field-columns is-vcentered">
        <div class="column device-field-label has-text-right-tablet is-one-fifth-tablet"><strong>Color format</strong></div>
        <div class="column">{{ device.format }}</div>
      </div>
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
  methods: {
    startBusy () {
      this.busy = true
    },
    endBusy (message, isError) {
      this.busy = false
      if (message) {
        this.$buefy.toast.open({
          message,
          type: isError ? 'is-danger' : 'is-success'
        })
      }
    },
    updateDeviceName: _.debounce(function (name) {
      this.$v.$touch()
      if (this.$v.deviceName.$anyError) {
        return
      }

      this.startBusy()
      this.$store.dispatch('devices/patch', { id: this.id, patch: { name } }).finally(() => {
        this.endBusy(this.device.name + ' renamed to ' + name + ' successfully!', false)
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

<style scoped lang="scss">
.device-field-columns {
  padding-top: 0;
  padding-bottom: 0;
  width: 100%;
}

.device-field-columns > .device-field-label {
}

.device-field-columns > .column {
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
}
</style>
