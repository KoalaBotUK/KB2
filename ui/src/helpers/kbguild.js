import axios from "axios";
import {getUser} from "../stores/auth.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

let user = getUser();

export async function getGuilds() {
  let resp = await axios.get(`${VITE_KB_API_URL}/guilds`, {
    headers: {
      'Authorization': 'Discord ' + user.token.accessToken
    }
  });
  return resp.data;
}

export async function getGuild(guild_id) {
  return (await axios.get(`${VITE_KB_API_URL}/guilds/${guild_id}`, {
    headers: {
      'Authorization': 'Discord ' + user.token.accessToken
    }
  })).data;
}