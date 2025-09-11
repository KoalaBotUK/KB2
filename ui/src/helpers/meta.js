import {GuildMeta, PartialGuildMeta} from "../stores/meta.js";
import {isGuildAdmin} from "./discord.js";


export async function fetchGuildMetaMap(token) {
  return (await PartialGuildMeta.fetchAll(token)).reduce((acc, guildMeta) => {
    // guildMeta.roleMeta = guildMeta.roles.reduce((acc, role) => acc.set(role.id, role), new Map());
    return acc.set(guildMeta.id, guildMeta);
  }, new Map());
}

export function filterByAdmin(guildMetaMap) {
  return guildMetaMap.values().filter(g => g.isAdmin).reduce((acc, guildMeta) => acc.set(guildMeta.id, guildMeta), new Map());
}
