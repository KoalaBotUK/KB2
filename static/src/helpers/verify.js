import axios from "axios";
import {getUser} from "../stores/auth";

const user = getUser()

export async function linkEmail(organization, token, overwrite=false) {
  await axios.post('http://localhost:8000/verify/email/link', {
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