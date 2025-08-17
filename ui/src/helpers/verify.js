import axios from "axios";
import {getUser} from "../stores/auth.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL
const user = getUser()

export async function linkEmail(organization, token, overwrite=false) {
  await axios.post(`${VITE_KB_API_URL}/users/${user.id}/links`, {
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