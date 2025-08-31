import axios from "axios";
import {formatInternalRedirect, redirectTo} from "./redirect.js";

const KB_API_URL = import.meta.env.VITE_KB_API_URL;

export class OauthFlow {
  clientId
  redirectPath
  token

  save() {
    localStorage.setItem('authFlow', JSON.stringify(this))
  }

  static load() {
    let deserialized = JSON.parse(localStorage.getItem('authFlow'))
    if (deserialized === undefined || deserialized === null) {
      return null
    } else {
      return Object.assign(new this.constructor, deserialized)
    }
  }

  async authorize() {
    console.error("Not implemented")
  }

  async callback() {
    console.error("Not implemented")
  }
}

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

export class AuthorizationFlowPKCE extends OauthFlow {
  authorizeUrl
  authorizeAdditionalParams
  tokenUrl
  codeChallenge
  codeVerifier
  scope

  constructor(clientId, authorizeUrl, authorizeAdditionalParams, redirectPath, tokenUrl, scope) {
    super()
    this.clientId = clientId
    this.authorizeUrl = authorizeUrl
    this.authorizeAdditionalParams = authorizeAdditionalParams
    this.redirectPath = redirectPath
    this.tokenUrl = tokenUrl
    this.scope = scope
  }

  async generateCodeChallenge() {
    this.codeVerifier = Array(43 + 1)
      .join()
      .replace(/(.|$)/g, (match) => ((match.length ? Math.random() : '').toString(36).charAt(2 + (match.length ? Math.floor(Math.random() * 4) : 0))));

    let hash = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(this.codeVerifier))

    this.codeChallenge = btoa(String.fromCharCode(...new Uint8Array(hash)))
      .replace(/=/g, '') // Remove padding characters
      .replace(/\+/g, '-') // Replace + with -
      .replace(/\//g, '_'); // Replace / with _
  }

  async authorize() {
    await this.generateCodeChallenge()
    this.save()
    redirectTo(`${this.authorizeUrl}?response_type=code&client_id=${this.clientId}&code_challenge=${this.codeChallenge}&code_challenge_method=S256&scope=${this.scope.replace(' ', '+')}&redirect_uri=${encodeURIComponent(formatInternalRedirect(this.redirectPath))}${this.authorizeAdditionalParams}`);
  }

  async callback() {
    let urlParams = new URLSearchParams(window.location.search);
    try {
      let res = await axios.post(this.tokenUrl, {
          grant_type: 'authorization_code',
          code: urlParams.get('code'),
          redirect_uri: formatInternalRedirect(this.redirectPath),
          scope: this.scope,
          code_verifier: this.codeVerifier,
          client_id: this.clientId
        },
        {
          headers: {'Content-Type': 'application/x-www-form-urlencoded'},
        })
      this.token = new OauthToken()
      this.token.accessToken = res.data['access_token']
      this.token.tokenType = res.data['token_type']
      this.token.expiresIn = res.data['expires_in']
      this.token.refreshToken = res.data['refresh_token']
      this.token.scope = res.data['scope']
      console.log(this.token)
    } catch (err) {
      console.error("Error when getting token", err)
      this.token = "ERROR"

    }
  }

  save() {
    localStorage.setItem('authFlow', JSON.stringify(this))
  }

  static load() {
    let deserialized = JSON.parse(localStorage.getItem('authFlow'))
    if (deserialized === undefined || deserialized === null) {
      return null
    } else {
      return Object.assign(new AuthorizationFlowPKCE, deserialized)
    }
  }
}

export class ImplicitFlow extends OauthFlow {
  authorizeUrl
  scope

  constructor(clientId, authorizeUrl, redirectPath, scope = undefined) {
    super();
    this.clientId = clientId
    this.authorizeUrl = authorizeUrl
    this.redirectPath = redirectPath
    if (scope === undefined) {
      scope = 'openid email'
    }
    this.scope = scope
  }

  async authorize() {
    this.save()
    window.location.href = `${this.authorizeUrl}?response_type=token&client_id=${this.clientId}&scope=${encodeURIComponent(this.scope)}&redirect_uri=${encodeURIComponent(formatInternalRedirect(this.redirectPath))}`;
  }

  async callback() {
    let urlParams = new URLSearchParams(window.location.hash.substring(1));
    this.token = new OauthToken()
    this.token.accessToken = urlParams.get('access_token')
    this.token.tokenType = urlParams.get('token_type')
    this.token.expiresIn = urlParams.get('expires_in')
    this.token.refreshToken = urlParams.get('refresh_token')
    this.token.scope = urlParams.get('scope')
  }

  save() {
    localStorage.setItem('authFlow', JSON.stringify(this))
  }

  static load() {
    let deserialized = JSON.parse(localStorage.getItem('authFlow'))
    if (deserialized === undefined || deserialized === null) {
      return null
    } else {
      return Object.assign(new ImplicitFlow, deserialized)
    }
  }
}