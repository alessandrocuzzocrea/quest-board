import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { useState, useEffect, createContext, useContext, type ReactNode } from 'react'
import { API } from './api'
import type { UserResponse } from './types'
import AuthPage from './AuthPage'
import BoardsList from './BoardsList'
import BoardPage from './BoardPage'
import SettingsPage from './SettingsPage'

interface AuthCtx {
  user: UserResponse | null
  loading: boolean
  refresh: () => Promise<void>
  logout: () => Promise<void>
}

const AuthContext = createContext<AuthCtx>({ user: null, loading: true, refresh: async () => {}, logout: async () => {} })
export const useAuth = () => useContext(AuthContext)

function ProtectedRoute({ children }: { children: ReactNode }) {
  const { user, loading } = useAuth()
  if (loading) return <div className="page" style={{ textAlign: 'center', paddingTop: '40vh' }}><div className="spinner" /></div>
  if (!user) return <Navigate to="/login" replace />
  return <>{children}</>
}

export default function App() {
  const [user, setUser] = useState<UserResponse | null>(null)
  const [loading, setLoading] = useState(true)

  const refresh = async () => {
    try {
      const u = await API.me()
      setUser(u)
    } catch {
      setUser(null)
    }
    setLoading(false)
  }

  useEffect(() => { refresh() }, [])

  const logout = async () => {
    await API.logout()
    setUser(null)
    window.location.href = '/login'
  }

  return (
    <AuthContext.Provider value={{ user, loading, refresh, logout }}>
      <BrowserRouter>
        <Routes>
          <Route path="/login" element={user ? <Navigate to="/boards" replace /> : <AuthPage onAuth={refresh} />} />
          <Route path="/boards" element={<ProtectedRoute><BoardsList /></ProtectedRoute>} />
          <Route path="/board/:slug/:name?" element={<ProtectedRoute><BoardPage /></ProtectedRoute>} />
          <Route path="/settings" element={<ProtectedRoute><SettingsPage /></ProtectedRoute>} />
          <Route path="/" element={<Navigate to="/boards" replace />} />
          <Route path="*" element={<Navigate to="/boards" replace />} />
        </Routes>
      </BrowserRouter>
    </AuthContext.Provider>
  )
}
