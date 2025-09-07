import axios from "axios";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL


export class RoleMeta {
  constructor(id, name, permissions) {
    this.id = id;
    this.name = name;
    this.permissions = permissions;
  }

  static fromJson(json) {
    return new RoleMeta(json['id'], json['name'], json['permissions']);
  }
}

export class GuildMeta {
  constructor(id, name, icon, roles) {
    this.id = id;
    this.name = name;
    this.icon = icon;
    this.roles = roles;
  }

  static fromJson(json) {
    return new GuildMeta(json['id'], json['name'], json['icon'], json['roles'].map(RoleMeta.fromJson));
  }

  static async fetchAll(token) {
    let r = await axios.get(`${VITE_KB_API_URL}/meta/guilds`,
      { headers: { 'Authorization': 'Discord ' + token } });
    return r.data.map(GuildMeta.fromJson);
  }
}