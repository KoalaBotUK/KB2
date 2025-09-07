import {GuildMeta} from "../stores/meta.js";
import {isGuildAdmin} from "./discord.js";


export async function fetchGuildMetaMap(token) {
  return (await GuildMeta.fetchAll(token)).reduce((acc, guildMeta) => {
    guildMeta.roleMeta = guildMeta.roles.reduce((acc, role) => acc.set(role.id, role), new Map());
    return acc.set(guildMeta.id, guildMeta);
  }, new Map());
}

export function filterByAdmin(guildMetaMap, currentUserGuildMetaMap) {
  let adminGuilds = new Map();
  for (let guildMeta of guildMetaMap.values()) {
    if (isGuildAdmin(currentUserGuildMetaMap.get(guildMeta.id))) {
      adminGuilds.set(guildMeta.id, guildMeta);
    }
  }
  return adminGuilds
}