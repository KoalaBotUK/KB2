import axios from "axios";
import {User} from "../stores/user.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

export async function linkEmail(organization, token, overwrite=false) {
  let user = User.loadCache()
  await axios.post(`${VITE_KB_API_URL}/users/${user.userId}/links`, {
      'origin': organization,
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

export async function putLinkGuild(guildId){
  let user = User.loadCache()
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
  let user = User.loadCache()
  return await axios.delete(`${VITE_KB_API_URL}/users/${user.userId}/link_guilds/${guildId}`,
    {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    }
  )
}