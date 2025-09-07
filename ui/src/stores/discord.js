import axios from "axios";

export class DscCurrentGuild {
  constructor(id, name, icon, owner, permissions) {
    this.id = id;
    this.name = name;
    this.icon = icon;
    this.owner = owner;
    this.permissions = permissions;
  }

  static fromJson(json) {
    return new DscCurrentGuild(json['id'], json['name'], json['icon'], json['owner'], json['permissions']);
  }

  static async fetch(guildId, token) {
    let r = await axios.get(`https://discord.com/api/v10/guilds/${guildId}`,
      { headers: { 'Authorization': 'Bearer ' + token } });
    return DscCurrentGuild.fromJson(r.data)
  }

  static async fetchCurrentUserGuilds(token) {
    let r = await axios.get(`https://discord.com/api/v10/users/@me/guilds`,
      { headers: { 'Authorization': 'Bearer ' + token } });

    return r.data.map(DscCurrentGuild.fromJson)
  }
}
