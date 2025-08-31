import axios from "axios";
import {User} from "./user.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

let user = User.loadCache();

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
      'verify': this.verify,
      'name': this.name,
      'icon': this.icon,
      'user_links': this.userLinks
    }
  }

  static async loadGuild(guildId) {
    let r = await axios.post(`${VITE_KB_API_URL}/guilds/${guildId}`, {}, {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    });

    return new Guild(guildId, r.data['verify'], r.data['name'], r.data['icon'], Object.assign(new Map, r.data['user_links']))
  }

  static async loadGuilds() {
    let r = await axios.post(`${VITE_KB_API_URL}/guilds`, {}, {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    });
    return r.data.map(g => new Guild(g['guild_id'], g['verify'], g['name'], g['icon']))
  }

  async save() {
    await axios.put(`${VITE_KB_API_URL}/guilds/${this.guildId}`, this.toJson(), {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    });
  }
}
