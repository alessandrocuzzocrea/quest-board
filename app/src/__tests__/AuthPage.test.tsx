import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import AuthPage from '../AuthPage'

vi.mock('../api', () => ({
  API: {
    login: vi.fn(),
    register: vi.fn(),
  },
}))

import { API } from '../api'

describe('AuthPage', () => {
  const onAuth = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders login form by default', () => {
    render(<AuthPage onAuth={onAuth} />)
    expect(screen.getByText('quest-board')).toBeInTheDocument()
    expect(screen.getAllByText(/Login/).length).toBeGreaterThanOrEqual(1)
    expect(screen.getByText('Register')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Login' })).toBeInTheDocument()
    expect(screen.getByText(/Default admin login/)).toBeInTheDocument()
  })

  it('switches to register tab on click', async () => {
    const user = userEvent.setup()
    render(<AuthPage onAuth={onAuth} />)
    await user.click(screen.getByText('Register'))
    expect(screen.getByText('Display Name (optional)')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Register' })).toBeInTheDocument()
    expect(screen.queryByText(/Default admin login/)).not.toBeInTheDocument()
  })

  it('calls API.login and onAuth on form submit', async () => {
    const user = userEvent.setup()
    const mockedLogin = vi.mocked(API.login)

    render(<AuthPage onAuth={onAuth} />)

    const inputs = screen.getAllByRole('textbox')
    await user.type(inputs[0], 'testuser')

    const passField = document.querySelector('input[type="password"]')!
    await user.type(passField, 'secret')

    await user.click(screen.getByRole('button', { name: 'Login' }))

    expect(mockedLogin).toHaveBeenCalledWith('testuser', 'secret')
    expect(onAuth).toHaveBeenCalled()
  })

  it('calls API.register with optional name', async () => {
    const user = userEvent.setup()
    const mockedRegister = vi.mocked(API.register)

    render(<AuthPage onAuth={onAuth} />)
    await user.click(screen.getByText('Register'))

    const textInputs = screen.getAllByRole('textbox')
    await user.type(textInputs[0], 'newuser')
    await user.type(textInputs[1], 'New User')

    const passField = document.querySelector('input[type="password"]')!
    await user.type(passField, 'p4ss')

    await user.click(screen.getByRole('button', { name: 'Register' }))

    expect(mockedRegister).toHaveBeenCalledWith('newuser', 'p4ss', 'New User')
    expect(onAuth).toHaveBeenCalled()
  })

  it('shows error message on failed login', async () => {
    const user = userEvent.setup()
    vi.mocked(API.login).mockRejectedValueOnce(new Error('invalid credentials'))

    render(<AuthPage onAuth={onAuth} />)
    const inputs = screen.getAllByRole('textbox')
    await user.type(inputs[0], 'bad')

    const passField = document.querySelector('input[type="password"]')!
    await user.type(passField, 'creds')

    await user.click(screen.getByRole('button', { name: 'Login' }))

    expect(await screen.findByText('invalid credentials')).toBeInTheDocument()
    expect(onAuth).not.toHaveBeenCalled()
  })

  it('clears error on tab switch', async () => {
    const user = userEvent.setup()
    vi.mocked(API.login).mockRejectedValueOnce(new Error('bad login'))

    render(<AuthPage onAuth={onAuth} />)
    const inputs = screen.getAllByRole('textbox')
    await user.type(inputs[0], 'bad')

    const passField = document.querySelector('input[type="password"]')!
    await user.type(passField, 'creds')

    await user.click(screen.getByRole('button', { name: 'Login' }))
    expect(await screen.findByText('bad login')).toBeInTheDocument()

    await user.click(screen.getByText('Register'))
    expect(screen.queryByText('bad login')).not.toBeInTheDocument()
  })
})
