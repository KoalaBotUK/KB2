import axios from "axios";
import {getUser} from "../stores/auth";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL
const user = getUser()

export async function linkEmail(organization, code, overwrite=false) {
  await axios.post(`${VITE_KB_API_URL}/verify/email/link`, {
      'organization': organization,
      'code': code,
      'overwrite': overwrite
    },
    {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    }
  )
}