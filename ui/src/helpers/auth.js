import axios from "axios";
import {formatInternalRedirect, redirectTo} from "./redirect.js";

export class OauthFlow {
  clientId
  redirectPath
  token

  save() {
    localStorage.setItem('authFlow', JSON.stringify(this))
  }

  static load() {
    const raw = localStorage.getItem('authFlow')
    if (raw === null) {
      return null
    }
    return Object.assign(new this(), JSON.parse(raw))
  }

  async authorize() {
    console.error("Not implemented")
  }

  async callback() {
    console.error("Not implemented")
  }
}

export class OauthToken {
  _accessToken
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

  set accessToken(v) {
    this._accessToken = v
  }

  get accessToken() {
    return this.isValid ? this._accessToken : null
  }
}

export class TokenExchangeError extends Error {
  constructor(reason, status) {
    super('OAuth token exchange failed')
    this.reason = reason
    this.status = status
  }
}

export function requestFailureReason(error) {
  if (error?.code === 'ECONNABORTED' || error?.code === 'ETIMEDOUT') return 'timeout'
  if (error?.code === 'ERR_NETWORK') return 'network_error'
  if (error?.response) return 'http_error'
  return 'unexpected_error'
}

const tokenErrorCodes = new Set([
  'invalid_grant', 'invalid_request', 'invalid_client', 'invalid_scope',
  'unauthorized_client', 'interaction_required', 'temporarily_unavailable',
])

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
    // Generate a cryptographically random code_verifier per RFC 7636 (43-128 chars,
    // unreserved character set). 32 random bytes base64url-encoded yields 43 chars.
    let randomBytes = crypto.getRandomValues(new Uint8Array(32))
    this.codeVerifier = btoa(String.fromCharCode(...randomBytes))
      .replace(/=/g, '') // Remove padding characters
      .replace(/\+/g, '-') // Replace + with -
      .replace(/\//g, '_'); // Replace / with _

    let hash = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(this.codeVerifier))

    this.codeChallenge = btoa(String.fromCharCode(...new Uint8Array(hash)))
      .replace(/=/g, '') // Remove padding characters
      .replace(/\+/g, '-') // Replace + with -
      .replace(/\//g, '_'); // Replace / with _
  }

  async authorize() {
    await this.generateCodeChallenge()
    this.save()
    redirectTo(`${this.authorizeUrl}?response_type=code&client_id=${this.clientId}&code_challenge=${this.codeChallenge}&code_challenge_method=S256&scope=${encodeURIComponent(this.scope)}&redirect_uri=${encodeURIComponent(formatInternalRedirect(this.redirectPath))}${this.authorizeAdditionalParams}`);
  }

  async callback() {
    const code = new URLSearchParams(window.location.search).get('code')
    if (!code) throw new TokenExchangeError('missing_code')
    if (!this.codeVerifier) throw new TokenExchangeError('missing_verifier')

    let res
    try {
      res = await axios.post(this.tokenUrl, {
          grant_type: 'authorization_code',
          code,
          redirect_uri: formatInternalRedirect(this.redirectPath),
          scope: this.scope,
          code_verifier: this.codeVerifier,
          client_id: this.clientId
        },
        {
          headers: {'Content-Type': 'application/x-www-form-urlencoded'},
          timeout: 15_000,
        })
    } catch (err) {
      const providerError = err?.response?.data?.error
      const reason = tokenErrorCodes.has(providerError) ? providerError : requestFailureReason(err)
      throw new TokenExchangeError(reason, err?.response?.status)
    }

    this.token = new OauthToken()
    this.token.accessToken = res.data?.access_token
    this.token.tokenType = res.data?.token_type
    this.token.expiresIn = res.data?.expires_in
    this.token.refreshToken = res.data?.refresh_token
    this.token.scope = res.data?.scope
    this.token.date = Date.now()
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
    this.token.date = Date.now()
  }
}
