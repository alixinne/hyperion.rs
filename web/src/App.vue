<template>
  <div id="app">
    <div class="has-background-dark">
      <div class="container">
        <b-navbar type="is-dark">
          <template slot="brand">
            <b-navbar-item tag="router-link" :to="{ name: 'Dashboard' }">
              <server-header/>
            </b-navbar-item>
          </template>

          <template slot="start">
            <b-navbar-item tag="router-link" :to="{ name: 'Devices' }">
              Devices
            </b-navbar-item>
          </template>

          <template slot="end">
            <b-navbar-item>
              <save-config-button/>
            </b-navbar-item>
          </template>
        </b-navbar>
      </div>
    </div>

    <section v-if="$store.state.errored" class="hero is-danger">
      <div class="hero-body">
        <div class="container">
          <h1 class="title">An error occurred</h1>
          <h2 class="subtitle">{{ $store.state.errored }}</h2>
        </div>
      </div>
    </section>

    <router-view v-else/>

    <b-loading :active.sync="$store.state.loading"></b-loading>
  </div>
</template>

<script>
import SaveConfigButton from './components/SaveConfigButton'
import ServerHeader from './components/ServerHeader'

export default {
  name: 'App',
  components: {
    SaveConfigButton,
    ServerHeader
  }
}
</script>

<style lang="scss">
// Import Bulma's core
@import "~bulma/sass/utilities/_all";

// Set your colors
$primary: #ff9100;
$primary-invert: findColorInvert($primary);

// Setup $colors to use as bulma classes
$colors: (
    "white": ($white, $black),
    "black": ($black, $white),
    "light": ($light, $light-invert),
    "dark": ($dark, $dark-invert),
    "primary": ($primary, $primary-invert),
    "info": ($info, $info-invert),
    "success": ($success, $success-invert),
    "warning": ($warning, $warning-invert),
    "danger": ($danger, $danger-invert),
);

// Links
$link: $primary;
$link-invert: $primary-invert;
$link-focus-border: $primary;

// Less padding for sections
$section-padding: 1.5rem 1.5rem;

// Import Bulma and Buefy styles
@import "~bulma";
@import "~buefy/src/scss/buefy";
</style>
