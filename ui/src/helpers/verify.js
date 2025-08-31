import axios from "axios";
import {User} from "../stores/user.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL
const user = User.loadCache()

export async function linkEmail(organization, token, overwrite=false) {
  await axios.post(`${VITE_KB_API_URL}/users/${user.userId}/links`, {
      'origin': organization,
      'token': token
    },
    {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    }
  )
}