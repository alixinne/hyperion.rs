import Vue from 'vue'
import 'es6-promise/auto'
import Vuex from 'vuex'

import server from './modules/server'

Vue.use(Vuex)

const state = {
}

const mutations = {
  loadStart (state) {
  },
  loadSuccess (state) {
  },
  loadError (state, error) {
  }
}

export default new Vuex.Store({
  state,
  mutations,
  modules: {
    server
  }
})
