import {DscCurrentGuild} from "../stores/discord.js";

export async function getCurrentUserGuildMetadata(token) {
  let userGuilds = await DscCurrentGuild.fetchCurrentUserGuilds(token);
  return userGuilds.reduce((acc, guild) => acc.set(guild.id, guild), new Map());
}

export function toAdminCurrentUserGuilds(userGuildMetaMap) {
  let adminGuilds = new Map();
  for (let guild of userGuildMetaMap.values()) {
    if (isGuildAdmin(guild)) {
      adminGuilds.set(guild.id, guild);
    }
  }
  return adminGuilds;
}
export function isGuildAdmin(userGuild) {
  return userGuild.owner || (userGuild.permissions & 0x8) === 0x8;
}

