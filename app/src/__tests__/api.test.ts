import { describe, it, expect, beforeEach, vi } from 'vitest'

// Mock fetch globally
const mockFetch = vi.fn()
globalThis.fetch = mockFetch

// Import after mock so the module reads the mocked fetch at module level
// (api.ts uses fetch directly, not an import — it's a global)
import { API } from '../api'

function mockResponse(status: number, body: unknown) {
  return Promise.resolve({
    ok: status >= 200 && status < 300,
    status,
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(typeof body === 'string' ? body : JSON.stringify(body)),
  } as Response)
}

describe('API client', () => {
  beforeEach(() => {
    mockFetch.mockReset()
  })

  describe('auth', () => {
    it('login sends POST to /auth/login', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { user: { id: '1', username: 'alice' } }))
      const res = await API.login('alice', 'secret')
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/auth/login',
        expect.objectContaining({ method: 'POST', body: JSON.stringify({ username: 'alice', password: 'secret' }) }),
      )
      expect(res).toEqual({ user: { id: '1', username: 'alice' } })
    })

    it('register sends POST to /auth/register', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { user: { id: '2', username: 'bob' } }))
      const res = await API.register('bob', 'p4ss')
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/auth/register',
        expect.objectContaining({ method: 'POST', body: JSON.stringify({ username: 'bob', password: 'p4ss' }) }),
      )
      expect(res).toEqual({ user: { id: '2', username: 'bob' } })
    })

    it('register sends optional name', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { user: { id: '3', username: 'carol' } }))
      await API.register('carol', 'p4ss', 'Carol')
      const body = JSON.parse((mockFetch.mock.calls[0][1] as RequestInit).body as string)
      expect(body).toEqual({ username: 'carol', password: 'p4ss', name: 'Carol' })
    })

    it('logout sends POST to /auth/logout', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, {}))
      await API.logout()
      expect(mockFetch).toHaveBeenCalledWith('/api/v1/auth/logout', expect.objectContaining({ method: 'POST' }))
    })

    it('me sends GET to /auth/me', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { id: '1', username: 'alice' }))
      const res = await API.me()
      expect(mockFetch).toHaveBeenCalledWith('/api/v1/auth/me', expect.objectContaining({ method: 'GET' }))
      expect(res).toEqual({ id: '1', username: 'alice' })
    })

    it('changePassword sends PUT to /auth/me/password', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { ok: true }))
      await API.changePassword('old', 'new')
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/auth/me/password',
        expect.objectContaining({
          method: 'PUT',
          body: JSON.stringify({ current_password: 'old', new_password: 'new' }),
        }),
      )
    })
  })

  describe('boards', () => {
    it('listBoards sends GET to /boards', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, [{ id: 'b1', name: 'Board 1' }]))
      const res = await API.listBoards()
      expect(mockFetch).toHaveBeenCalledWith('/api/v1/boards', expect.objectContaining({ method: 'GET' }))
      expect(res).toEqual([{ id: 'b1', name: 'Board 1' }])
    })

    it('getBoardBySlug sends GET to /boards/by-slug/{slug}', async () => {
      const data = { board: { id: 'b1' }, lists: [] }
      mockFetch.mockResolvedValueOnce(mockResponse(200, data))
      const res = await API.getBoardBySlug('my-board')
      expect(mockFetch).toHaveBeenCalledWith('/api/v1/boards/by-slug/my-board', expect.objectContaining({ method: 'GET' }))
      expect(res).toEqual(data)
    })

    it('createBoard sends POST to /boards', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { id: 'b2', name: 'New' }))
      await API.createBoard({ name: 'New' })
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/boards',
        expect.objectContaining({ method: 'POST', body: JSON.stringify({ name: 'New' }) }),
      )
    })

    it('updateBoard sends PUT to /boards/{id}', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, {}))
      await API.updateBoard('b1', { name: 'Renamed' })
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/boards/b1',
        expect.objectContaining({ method: 'PUT', body: JSON.stringify({ name: 'Renamed' }) }),
      )
    })

    it('deleteBoard sends DELETE to /boards/{id}', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, {}))
      await API.deleteBoard('b1')
      expect(mockFetch).toHaveBeenCalledWith('/api/v1/boards/b1', expect.objectContaining({ method: 'DELETE' }))
    })
  })

  describe('cards', () => {
    it('createCard sends POST to /cards', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, {}))
      await API.createCard({ list_id: 'l1', name: 'Card' })
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/cards',
        expect.objectContaining({ method: 'POST', body: JSON.stringify({ list_id: 'l1', name: 'Card' }) }),
      )
    })

    it('getCard sends GET to /cards/{id}', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { id: 'c1', name: 'Card' }))
      const res = await API.getCard('c1')
      expect(mockFetch).toHaveBeenCalledWith('/api/v1/cards/c1', expect.objectContaining({ method: 'GET' }))
      expect(res).toEqual({ id: 'c1', name: 'Card' })
    })

    it('moveCard sends PUT to /cards/{id}/move', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, {}))
      await API.moveCard('c1', { list_id: 'l2', position: 500 })
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/cards/c1/move',
        expect.objectContaining({ method: 'PUT', body: JSON.stringify({ list_id: 'l2', position: 500 }) }),
      )
    })

    it('addCardLabel sends POST to /cards/{id}/labels', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, {}))
      await API.addCardLabel('c1', { label_id: 'lab1' })
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/cards/c1/labels',
        expect.objectContaining({ method: 'POST', body: JSON.stringify({ label_id: 'lab1' }) }),
      )
    })
  })

  describe('search', () => {
    it('search sends GET to /search with query param', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { cards: [], boards: [] }))
      await API.search('hello')
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/search?q=hello',
        expect.objectContaining({ method: 'GET' }),
      )
    })

    it('search encodes special characters', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { cards: [], boards: [] }))
      await API.search('foo & bar')
      expect(mockFetch).toHaveBeenCalledWith(
        '/api/v1/search?q=foo%20%26%20bar',
        expect.anything(),
      )
    })
  })

  describe('api keys', () => {
    it('listApiKeys sends GET', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, [{ id: 'k1', name: 'Key 1' }]))
      const res = await API.listApiKeys()
      expect(mockFetch).toHaveBeenCalledWith('/api/v1/api-keys', expect.objectContaining({ method: 'GET' }))
      expect(Array.isArray(res)).toBe(true)
    })

    it('createApiKey sends POST', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, { id: 'k1', key: 'sk-xxx' }))
      const res = await API.createApiKey({ name: 'My Key' })
      expect(res.key).toBe('sk-xxx')
    })

    it('deleteApiKey sends DELETE', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, {}))
      await API.deleteApiKey('k1')
      expect(mockFetch).toHaveBeenCalledWith('/api/v1/api-keys/k1', expect.objectContaining({ method: 'DELETE' }))
    })
  })

  describe('error handling', () => {
    it('throws on non-ok response with JSON error', async () => {
      mockFetch.mockResolvedValueOnce(
        Promise.resolve({
          ok: false,
          status: 401,
          json: () => Promise.resolve({ error: 'bad credentials' }),
          text: () => Promise.resolve(JSON.stringify({ error: 'bad credentials' })),
        } as Response),
      )
      await expect(API.login('x', 'y')).rejects.toThrow('bad credentials')
    })

    it('throws with raw text on non-JSON error', async () => {
      mockFetch.mockResolvedValueOnce(
        Promise.resolve({
          ok: false,
          status: 500,
          json: () => Promise.reject(new Error('not json')),
          text: () => Promise.resolve('Internal Server Error'),
        } as Response),
      )
      await expect(API.me()).rejects.toThrow('Internal Server Error')
    })
  })

  describe('credentials', () => {
    it('sends credentials: include on every request', async () => {
      mockFetch.mockResolvedValueOnce(mockResponse(200, {}))
      await API.me()
      expect((mockFetch.mock.calls[0][1] as RequestInit).credentials).toBe('include')
    })
  })
})
