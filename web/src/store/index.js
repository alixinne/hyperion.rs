import Vue from 'vue'
import 'es6-promise/auto'
import Vuex from 'vuex'

import server from './modules/server'

Vue.use(Vuex)

const state = {
  loading: true,
  errored: null
}

const mutations = {
  loadStart (state) {
    state.errored = null
    state.loading = true
  },
  loadSuccess (state) {
    state.errored = null
    state.loading = false
  },
  loadError (state, error) {
    state.errored = error
    state.loading = false
  }
}

export default new Vuex.Store({
  state,
  mutations,
  modules: {
    server
  }
})
