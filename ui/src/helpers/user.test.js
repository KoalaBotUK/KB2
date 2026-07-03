import { describe, it, expect, vi, beforeEach } from 'vitest'
import axios from 'axios'

vi.mock('axios')

// linkEmail reads the cached user (id + access token) at import time via
// User.loadCache(). Stub the store module so the test doesn't depend on
// localStorage/jsdom - we only care about what linkEmail sends to axios.
vi.mock('../stores/user.js', () => ({
  User: {
    loadCache: () => ({
      userId: 'test-user-id',
      token: { accessToken: 'test-access-token' }
    })
  }
}))

const { linkEmail } = await import('./user.js')

describe('linkEmail', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('sends overwrite: true in the POST body when overwrite=true', async () => {
    axios.post.mockResolvedValue({ data: {} })

    await linkEmail('some-organization', 'some-token', true)

    expect(axios.post).toHaveBeenCalledTimes(1)
    const [, body] = axios.post.mock.calls[0]
    // Regression check for issue #47: the overwrite param was previously
    // silently dropped from the request body.
    expect(body).toHaveProperty('overwrite', true)
    expect(body).toMatchObject({
      origin: 'some-organization',
      token: 'some-token',
      overwrite: true
    })
  })

  it('defaults overwrite to false when the argument is omitted', async () => {
    axios.post.mockResolvedValue({ data: {} })

    await linkEmail('some-organization', 'some-token')

    const [, body] = axios.post.mock.calls[0]
    expect(body).toHaveProperty('overwrite', false)
  })
})
