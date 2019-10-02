import axios from 'axios'

export default {
  save () { return axios.post('/api/config/save') }
}
