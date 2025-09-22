import axios from "axios";
import {OauthToken} from "../helpers/auth.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

export class Link {
  linkAddress
  linkedAt
  active

  constructor(linkAddress, linkedAt, active) {
    this.linkAddress = linkAddress
    this.linkedAt = linkedAt
    this.active = active
  }

  toJson() {
    return {
      'link_address': this.linkAddress,
      'linked_at': this.linkedAt,
      'active': this.active
    }
  }
}

export class LinkGuild {
  guildId
  name
  icon
  enabled

  constructor(guildId, name, icon, enabled) {
    this.guildId = guildId
    this.name = name
    this.icon = icon
    this.enabled = enabled
  }

  toJson() {
    return {
      'guild_id': this.guildId,
      'name': this.name,
      'icon': this.icon,
      'enabled': this.enabled
    }
  }
}

export class User {
  /**
   * @property {string}
   */
  userId
  /**
   * @property {string}
   */
  globalName
  /**
   * @property {string}
   */
  avatar
  /**
   * @property {Link[]}
   */
  links
  /**
   * @property {LinkGuild[]}
   */
  linkGuilds
  /**
   * @property {OauthToken}
   */
  token

  constructor(userId, globalName, avatar, links, linkGuilds, token) {
    this.userId = userId
    this.globalName = globalName
    this.avatar = avatar
    this.links = links
    this.linkGuilds = linkGuilds
    this.token = token
  }

  static async loadMe(token) {
    let r = await axios.post(`${VITE_KB_API_URL}/users/@me`, {}, {
      headers: {
        'Authorization': 'Discord ' + token.accessToken
      }
    })
    return new User(
      r.data['user_id'],
      r.data['global_name'],
      r.data['avatar'],
      r.data['links'].map(l => new Link(['link_address'], l['linked_at'], l['active'])),
      r.data['link_guilds'].map(lg => new LinkGuild(lg['guild_id'],  lg['name'], lg['icon'], lg['enabled'])),
      token
    )
  }

  static async loadMeCache(token) {
    let cacheUser = await User.loadMe(token)
    User.saveCache(cacheUser)
    return cacheUser
  }

  static loadCache() {
    let cacheUser = localStorage.getItem('user');
    if (localStorage.getItem('user') === null) return null
    let user = Object.assign(new User, JSON.parse(cacheUser))
    user.token = user.token ? Object.assign(new OauthToken, user.token) : user.token
    user.links = user.links.map(l => Object.assign(new Link, l))
    user.linkGuilds = user.linkGuilds.map(lg => Object.assign(new LinkGuild, lg))
    return user
  }

  static saveCache(user) {
    localStorage.setItem('user', JSON.stringify(user))
  }

  static clearCache() {
    localStorage.removeItem('user')
  }

  toJson() {
    return {
      'user_id': this.userId,
      'global_name': this.globalName,
      'avatar': this.avatar,
      'links': this.links.map(l => l.toJson()),
      'link_guilds': this.linkGuilds.map(lg => lg.toJson())
    }
  }

  async save() {
    await axios.put(`${VITE_KB_API_URL}/users/@me`, this.toJson(), {
      headers: {
        'Authorization': 'Discord ' + this.token.accessToken
      }
    });
    User.saveCache(this)
  }

  logout() {
    this.token = null
    User.saveCache(this)
  }

}

export function isUserLoggedIn(user) {
  return user !== undefined && user !== null && user.token !== null && user.token.isValid
}
