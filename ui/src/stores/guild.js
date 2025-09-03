import axios from "axios";
import {User} from "./user.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

let user = User.loadCache();

export class VerifyRole {
  roleId
  roleName
  pattern
  members

  constructor(roleId, name, pattern, members) {
    this.roleId = roleId
    this.roleName = name
    this.pattern = pattern
    this.members = members
  }

  toJson() {
    return {
      'role_id': this.roleId,
      'role_name': this.roleName,
      'pattern': this.pattern,
      'members': this.members
    }
  }

  static fromJson(json) {
    let verifyRole = new VerifyRole()
    verifyRole.roleId = json['role_id']
    verifyRole.roleName = json['role_name']
    verifyRole.pattern = json['pattern']
    verifyRole.members = json['members']
    return verifyRole
  }
}

export class Verify {
  roles
  userLinks

  constructor(roles, userLinks) {
    this.roles = roles
    this.userLinks = userLinks
  }

  toJson() {
    return {
      'roles': this.roles.map(r => r.toJson()),
      'user_links': this.userLinks
    }
  }

  static fromJson(json) {
    let verify = new Verify()
    verify.roles = json['roles'].map(VerifyRole.fromJson)
    verify.userLinks = Object.assign(new Map, json['user_links'])
    return verify
  }
}

export class Guild {
  guildId
  verify
  name
  icon
  userLinks

  constructor(guildId, verify, name, icon, userLinks) {
    this.guildId = guildId
    this.verify = verify
    this.name = name
    this.icon = icon
    this.userLinks = userLinks
  }

  toJson() {
    return {
      'guild_id': this.guildId,
      'verify': this.verify.toJson(),
      'name': this.name,
      'icon': this.icon,
    }
  }

  static fromJson(json) {
    let guild = new Guild()
    guild.guildId = json['guild_id']
    guild.verify = Verify.fromJson(json['verify'])
    guild.name = json['name']
    guild.icon = json['icon']
    return guild
  }

  static async loadGuild(guildId) {
    let r = await axios.post(`${VITE_KB_API_URL}/guilds/${guildId}`, {}, {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    });

    return Guild.fromJson(r.data)
  }

  static async loadGuilds() {
    let r = await axios.post(`${VITE_KB_API_URL}/guilds`, {}, {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    });
    return r.data.map(g => Guild.fromJson(g))
  }

  async save() {
    await axios.put(`${VITE_KB_API_URL}/guilds/${this.guildId}`, this.toJson(), {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    });
  }
}
