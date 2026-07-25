import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import NavBar from '../NavBar'

// Mock the useAuth hook from App
vi.mock('../App', () => ({
  useAuth: vi.fn(() => ({
    user: { id: '1', username: 'alice', role: 'user' },
    logout: vi.fn(),
  })),
}))

import * as AppModule from '../App'

describe('NavBar', () => {
  it('renders logo, nav links, username, and logout button', () => {
    render(
      <MemoryRouter>
        <NavBar />
      </MemoryRouter>,
    )
    expect(screen.getByText('quest-board')).toBeInTheDocument()
    expect(screen.getByText('Boards')).toBeInTheDocument()
    expect(screen.getByText('Settings')).toBeInTheDocument()
    expect(screen.getByText('alice')).toBeInTheDocument()
    expect(screen.getByText('Logout')).toBeInTheDocument()
  })

  it('highlights the Boards link when on /boards', () => {
    render(
      <MemoryRouter initialEntries={['/boards']}>
        <NavBar />
      </MemoryRouter>,
    )
    const boardsLink = screen.getByText('Boards')
    expect(boardsLink).toHaveStyle({ color: 'var(--accent)' })
  })

  it('highlights the Settings link when on /settings', () => {
    render(
      <MemoryRouter initialEntries={['/settings']}>
        <NavBar />
      </MemoryRouter>,
    )
    const settingsLink = screen.getByText('Settings')
    expect(settingsLink).toHaveStyle({ color: 'var(--accent)' })
  })

  it('does not highlight any link on other pages', () => {
    render(
      <MemoryRouter initialEntries={['/login']}>
        <NavBar />
      </MemoryRouter>,
    )
    expect(screen.getByText('Boards')).not.toHaveStyle({ color: 'var(--accent)' })
    expect(screen.getByText('Settings')).not.toHaveStyle({ color: 'var(--accent)' })
  })

  it('calls logout on button click', async () => {
    const mockLogout = vi.fn()
    vi.mocked(AppModule.useAuth).mockReturnValueOnce({
      user: { id: '1', username: 'bob', role: 'user' },
      logout: mockLogout,
    })

    const user = userEvent.setup()
    render(
      <MemoryRouter>
        <NavBar />
      </MemoryRouter>,
    )
    await user.click(screen.getByText('Logout'))
    expect(mockLogout).toHaveBeenCalled()
  })
})
