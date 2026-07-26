import { useState, useEffect } from 'react'
import { API } from './api'
import NavBar from './NavBar'
import { useAuth } from './App'

interface ApiKey {
  id: string
  name: string
  prefix?: string
  last_used_at: string | null
}

export default function SettingsPage() {
  const { user } = useAuth()
  const [currentPw, setCurrentPw] = useState('')
  const [newPw, setNewPw] = useState('')
  const [keys, setKeys] = useState<ApiKey[]>([])
  const [newKeyName, setNewKeyName] = useState('')
  const [msg, setMsg] = useState<{ text: string; type: string } | null>(null)

  const loadKeys = async () => {
    try {
      const k = await API.listApiKeys()
      setKeys(k as unknown as ApiKey[])
    } catch { /* ignore */ }
  }

  useEffect(() => { loadKeys() }, [])

  const changePassword = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await API.changePassword(currentPw, newPw)
      setMsg({ text: 'Password changed', type: 'success' })
      setCurrentPw('')
      setNewPw('')
    } catch (err: unknown) {
      setMsg({ text: err instanceof Error ? err.message : 'Failed to change password', type: 'error' })
    }
  }

  const createKey = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      const res = await API.createApiKey({ name: newKeyName })
      setMsg({ text: `Key created: ${res.key} (copy it now, it won't be shown again)`, type: 'success' })
      setNewKeyName('')
      loadKeys()
    } catch (err: unknown) {
      setMsg({ text: err instanceof Error ? err.message : 'Failed to create key', type: 'error' })
    }
  }

  const deleteKey = async (id: string) => {
    if (!confirm('Delete this API key?')) return
    try {
      await API.deleteApiKey(id)
      loadKeys()
    } catch (err: unknown) {
      setMsg({ text: err instanceof Error ? err.message : 'Failed to delete key', type: 'error' })
    }
  }

  return (
    <>
      <NavBar />
      <div className="page" style={{ maxWidth: 640 }}>
        <h1 className="mb-16">Settings</h1>

        {msg && <div className={`alert alert-${msg.type}`}>{msg.text}</div>}

        <div className="settings-section">
          <h3>Account</h3>
          <p>Logged in as <strong>{user?.username}</strong></p>
        </div>

        <div className="settings-section">
          <h3>Change Password</h3>
          <form onSubmit={changePassword}>
            <div className="form-group">
              <label>Current Password</label>
              <input type="password" className="input" value={currentPw} onChange={e => setCurrentPw(e.target.value)} required />
            </div>
            <div className="form-group">
              <label>New Password</label>
              <input type="password" className="input" value={newPw} onChange={e => setNewPw(e.target.value)} required />
            </div>
            <button className="btn btn-primary">Change Password</button>
          </form>
        </div>

        <div className="settings-section">
          <h3>API Keys</h3>
          <div className="mb-16">
            {keys.length === 0 ? (
              <p className="text-sm">No API keys yet.</p>
            ) : (
              keys.map(k => (
                <div key={k.id} className="api-key-row">
                  <span>{k.name}</span>
                  <code className="key-preview" style={{ fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>
                    {k.prefix || ''}...
                  </code>
                  <span className="text-sm">
                    {k.last_used_at ? `Last used ${new Date(k.last_used_at).toLocaleDateString()}` : 'Never used'}
                  </span>
                  <button onClick={() => deleteKey(k.id)} className="btn btn-sm btn-danger">Delete</button>
                </div>
              ))
            )}
          </div>
          <form onSubmit={createKey} className="flex gap-8">
            <input type="text" className="input" placeholder="Key name" value={newKeyName}
              onChange={e => setNewKeyName(e.target.value)} required style={{ flex: 1 }} />
            <button className="btn btn-primary">Create Key</button>
          </form>
        </div>
      </div>
    </>
  )
}
