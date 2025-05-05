import axios from "axios";
import {getUser} from "../stores/auth.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL
const user = getUser()

export async function linkEmail(organization, token, overwrite=false) {
  await axios.post(`${VITE_KB_API_URL}/verify/email/link`, {
      'organization': organization,
      'token': token,
      'overwrite': overwrite
    },
    {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    }
  )
}