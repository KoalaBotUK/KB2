import axios from "axios";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL


export class RoleMeta {
  constructor(id, name, permissions, color) {
    this.id = id;
    this.name = name;
    this.permissions = permissions;
    this.color = color;
  }

  static fromJson(json) {
    return new RoleMeta(json['id'], json['name'], json['permissions'], json['color']);
  }
}

export class ChannelMeta {
  constructor(id, name, type) {
    this.id = id;
    this.name = name;
    this.type = type
  }

  static fromJson(json) {
    return new ChannelMeta(json['id'], json['name'], json['type']);
  }
}

export class PartialGuildMeta {
  constructor(id, name, icon, isAdmin) {
    this.id = id;
    this.name = name;
    this.icon = icon;
    this.isAdmin = isAdmin;
  }

  static fromJson(json) {
    return new PartialGuildMeta(json['id'], json['name'], json['icon'], json['is_admin']);
  }

  static async fetchAll(token) {
    let r = await axios.get(`${VITE_KB_API_URL}/meta/guilds`,
      { headers: { 'Authorization': 'Discord ' + token } });
    return r.data.map(PartialGuildMeta.fromJson);
  }
}


export class GuildMeta extends PartialGuildMeta {
  constructor(id, name, icon, isAdmin, roles, channels) {
    super(id, name, icon, isAdmin);
    this.roles = roles;
    this.channels = channels;
  }

  static fromJson(json) {
    return new GuildMeta(json['id'], json['name'], json['icon'], json['is_admin'], json['roles'].map(RoleMeta.fromJson), json['channels'].map(ChannelMeta.fromJson));
  }

  static async fetch(id, token) {
    let r = await axios.get(`${VITE_KB_API_URL}/meta/guilds/${id}`,
      { headers: { 'Authorization': 'Discord ' + token } });
    return GuildMeta.fromJson(r.data);
  }
}