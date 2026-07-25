import { Link, useLocation } from 'react-router-dom'
import { useAuth } from './App'

export default function NavBar() {
  const { user, logout } = useAuth()
  const loc = useLocation()

  const isActive = (path: string) => loc.pathname.startsWith(path) ? { color: 'var(--accent)' as const } : undefined

  return (
    <div className="topbar">
      <Link to="/boards" className="logo">quest-board</Link>
      <nav>
        <Link to="/boards" style={isActive('/boards')}>Boards</Link>
        <Link to="/settings" style={isActive('/settings')}>Settings</Link>
      </nav>
      <span className="user-name">{user?.username}</span>
      <button className="btn btn-sm" onClick={logout}>Logout</button>
    </div>
  )
}
