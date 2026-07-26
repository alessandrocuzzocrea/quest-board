import { useState, type FormEvent } from 'react'
import { API } from './api'

export default function AuthPage({ onAuth }: { onAuth: () => Promise<void> }) {
  const [tab, setTab] = useState<'login' | 'register'>('login')
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [name, setName] = useState('')
  const [error, setError] = useState('')

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError('')
    try {
      if (tab === 'login') {
        await API.login(username, password)
      } else {
        await API.register(username, password, name || undefined)
      }
      await onAuth()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'An error occurred')
    }
  }

  const switchTab = (t: 'login' | 'register') => {
    setTab(t)
    setError('')
  }

  return (
    <div className="auth-page">
      <div className="auth-box">
        <h1>quest-board</h1>
        <div className="auth-tabs">
          <div className={`auth-tab ${tab === 'login' ? 'active' : ''}`} onClick={() => switchTab('login')}>Login</div>
          <div className={`auth-tab ${tab === 'register' ? 'active' : ''}`} onClick={() => switchTab('register')}>Register</div>
        </div>

        {error && <div className="alert alert-error">{error}</div>}

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Username</label>
            <input type="text" className="input" value={username} onChange={e => setUsername(e.target.value)} required />
          </div>
          {tab === 'register' && (
            <div className="form-group">
              <label>Display Name (optional)</label>
              <input type="text" className="input" value={name} onChange={e => setName(e.target.value)} />
            </div>
          )}
          <div className="form-group">
            <label>Password</label>
            <input type="password" className="input" value={password} onChange={e => setPassword(e.target.value)} required />
          </div>
          <button type="submit" className="btn btn-primary" style={{ width: '100%' }}>
            {tab === 'login' ? 'Login' : 'Register'}
          </button>
        </form>

        {tab === 'login' && (
          <p className="text-sm" style={{ marginTop: 12 }}>
            Default admin login: <code>admin</code> / <code>admin</code>
          </p>
        )}
      </div>
    </div>
  )
}
