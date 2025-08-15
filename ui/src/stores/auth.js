import axios from "axios";

const DISCORD_CLIENT_ID = import.meta.env.VITE_DISCORD_CLIENT_ID;

export class OauthToken {
  accessToken
  tokenType
  expiresIn
  refreshToken
  scope
  date

  get isValid() {
    if (this.date === undefined) {
      return true
    }
    return this.date + this.expiresIn * 1000 > Date.now()
  }
}

export class DiscordUser {
  id
  username
  globalName
  avatar
  verified
  email
  token

  static async fromToken(token) {
    const user = new DiscordUser()
    user.token = token
    await user.fetchUser()
    return user
  }

  async fetchUser() {
    if (this.token === undefined || !this.token.isValid) {
      console.log("Cannot fetch user, token is not valid")
      return
    }

    let r = await axios.get('https://discord.com/api/v10/users/@me', {
      headers: {
        'Authorization': 'Bearer ' + this.token.accessToken
      }
    })
    this.id = r.data['id']
    this.username = r.data['username']
    this.globalName = r.data['global_name']
    this.avatar = r.data['avatar']
    this.verified = r.data['verified']
    this.email = r.data['email']
  }

  get avatarUrl() {
    return `https://cdn.discordapp.com/avatars/${this.id}/${this.avatar}.webp`
  }
}

export function setUser(user) {
  localStorage.setItem('discordUser', JSON.stringify(user))
}

export function getUser() {
  let deserialized = JSON.parse(localStorage.getItem('discordUser'))
  if (deserialized === undefined ||deserialized === null) {
    return null
  } else {
    let user = Object.assign(new DiscordUser, deserialized)
    user.token = Object.assign(new OauthToken, user.token)

    if (user.token === undefined || !user.token.isValid) {
      console.log(user.token)
      console.log(user.token.isValid)
      return null
    } else {
      return user
    }
  }
}




