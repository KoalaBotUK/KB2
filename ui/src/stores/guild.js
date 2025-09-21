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

export class VoteOption {
  emoji
  label
  users

  constructor(emoji, label, users) {
    this.emoji = emoji
    this.label = label
    this.users = users
  }

  toJson() {
    return {
      'emoji': this.emoji,
      'label': this.label,
      'users': this.users
    }
  }

  static fromJson(json) {
    let voteOption = new VoteOption()
    voteOption.emoji = json['emoji']
    voteOption.label = json['label']
    voteOption.users = json['users']
    return voteOption
  }
}

export class VoteVote {
  messageId
  title
  description
  options
  channelId
  closeAt
  open
  roleList
  roleListType

  // constructor(message_id, title, description, options, channel_id, close_at, open, role_list, role_list_type) {
  //   this.message_id = message_id
  //   this.title = title
  //   this.description = description
  //   this.options = options
  //   this.channel_id = channel_id
  //   this.close_at = close_at
  //   this.open = open
  //   this.role_list = role_list
  //   this.role_list_type = role_list_type
  // }

  toJson() {
    return {
      'message_id': this.messageId,
      'title': this.title,
      'description': this.description,
      'options': this.options,
      'channel_id': this.channelId,
      'close_at': this.closeAt,
      'open': this.open,
      'role_list': this.roleList,
      'role_list_type': this.roleListType
    }
  }

  static fromJson(json) {
    let vote = new VoteVote()
    vote.messageId = json['message_id']
    vote.title = json['title']
    vote.description = json['description']
    vote.options = json['options'].map(VoteOption.fromJson) //
    vote.channelId = json['channel_id']
    vote.closeAt = json['close_at'] //
    vote.open = json['open']
    vote.roleList = json['role_list']
    vote.roleListType = json['role_list_type'] //
    return vote
  }
}


export class Vote {
  votes

  constructor(roles) {
    this.roles = roles
  }

  toJson() {
    return {
      'roles': this.roles.map(r => r.toJson()),
    }
  }

  static fromJson(json) {
    let vote = new Vote()
    vote.votes = json['votes'].map(VoteVote.fromJson)
    return vote
  }
}

export class Guild {
  guildId
  verify
  vote
  name
  icon
  userLinks

  constructor(guildId, verify, vote, name, icon, userLinks) {
    this.guildId = guildId
    this.verify = verify
    this.vote = vote
    this.name = name
    this.icon = icon
    this.userLinks = userLinks
  }

  toJson() {
    return {
      'guild_id': this.guildId,
      'verify': this.verify.toJson(),
      'vote': this.vote.toJson(),
      'name': this.name,
      'icon': this.icon,
    }
  }

  static fromJson(json) {
    let guild = new Guild()
    guild.guildId = json['guild_id']
    guild.verify = Verify.fromJson(json['verify'])
    guild.vote = Vote.fromJson(json['vote'])
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
