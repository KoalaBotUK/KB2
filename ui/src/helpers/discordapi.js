import axios from "axios";
import {getUser} from "../stores/auth.js";

let user = getUser();

export async function getUserAdminGuilds() {
  return (await axios.get('https://discord.com/api/v10/users/@me/guilds', {
    headers: {
      'Authorization': 'Bearer ' + user.token.accessToken
    }
  })).data.filter(guild => guild.permissions && (BigInt(guild.permissions) & BigInt(0x0000000000000008)) === BigInt(0x0000000000000008));
}

export async function getUserAdminGuildsAsMap() {
  let guilds = await getUserAdminGuilds();
  return Object.values(guilds).reduce((acc, guild) => {
    acc.set(guild.id, guild);
    return acc;
  }, new Map());
}