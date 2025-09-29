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

export async function putLinkGuild(guildId){
  return await axios.put(`${VITE_KB_API_URL}/users/${user.userId}/link_guilds/${guildId}`, {
    },
    {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    }
  )
}

export async function deleteLinkGuild(guildId){
  return await axios.delete(`${VITE_KB_API_URL}/users/${user.userId}/link_guilds/${guildId}`,
    {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    }
  )
}