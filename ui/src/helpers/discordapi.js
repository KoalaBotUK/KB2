import axios from "axios";
import {getUser} from "../stores/auth.js";

let user = getUser();

export async function getUserAdminGuilds() {
  let guilds = (await axios.get('https://discord.com/api/v10/users/@me/guilds', {
    headers: {
      'Authorization': 'Bearer ' + user.token.accessToken
    }
  })).data
  //convert to map of id to obj
  return guilds.filter(g => (Number(g.permissions) & (1 << 3)) === (1 << 3))
    .reduce((acc, g) => {
      acc[g.id] = g;
      return acc;
    }, {});
}