import axios from 'axios'

export default {
  index () { return axios.get('/api/devices') },
  patch (id, payload) { return axios.patch('/api/devices/' + id, payload) }
}
