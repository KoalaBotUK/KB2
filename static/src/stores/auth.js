import axios from "axios";

const DISCORD_CLIENT_ID = import.meta.env.VITE_DISCORD_CLIENT_ID;
const DISCORD_CLIENT_SECRET = import.meta.env.VITE_DISCORD_CLIENT_SECRET;

class OauthToken {
  accessToken
  tokenType
  expiresIn
  refreshToken
  scope
  date

  get isValid() {
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

  static async fromAuthorizationCode(code) {
    const user = new DiscordUser()
    await user.fetchToken(code)
    await user.fetchUser()
    return user
  }

  async fetchToken(code) {
    this.token = new OauthToken()
    this.token.date = Date.now()
    let r = await axios.post('https://discord.com/api/oauth2/token', {
        grant_type: 'authorization_code',
        code: code,
        redirect_uri: 'http://localhost:3000/verify/discord/callback',
        scope: 'identify email'
      },
      {
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        auth: { username: DISCORD_CLIENT_ID, password: DISCORD_CLIENT_SECRET }}

    )
    this.token.accessToken = r.data['access_token']
    this.token.tokenType = r.data['token_type']
    this.token.expiresIn = r.data['expires_in']
    this.token.refreshToken = r.data['refresh_token']
    this.token.scope = r.data['scope']
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




