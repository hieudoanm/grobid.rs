import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import Navbar from '../Navbar';
import ThemeToggle, {
  THEME_STORAGE_KEY,
  LIGHT_THEME,
  DARK_THEME,
} from '../ThemeToggle';
import TerminalWindow from '../TerminalWindow';

describe('Navbar', () => {
  it('renders brand and navigation links', () => {
    render(<Navbar />);
    expect(screen.getByText('grobid.rs')).toBeInTheDocument();
    expect(
      screen.getAllByRole('link', { name: 'Features' })[0]
    ).toHaveAttribute('href', '#features');
    expect(screen.getAllByRole('link', { name: 'Install' })[0]).toHaveAttribute(
      'href',
      '#install'
    );
  });
});

describe('ThemeToggle', () => {
  it('toggles the theme and persists it', () => {
    render(<ThemeToggle />);
    const button = screen.getByRole('button', {
      name: 'Toggle color theme',
    });
    fireEvent.click(button);
    expect(document.documentElement.getAttribute('data-theme')).toBe(
      DARK_THEME
    );
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe(DARK_THEME);
    fireEvent.click(button);
    expect(document.documentElement.getAttribute('data-theme')).toBe(
      LIGHT_THEME
    );
  });
});

describe('TerminalWindow', () => {
  it('renders command and output lines', () => {
    render(
      <TerminalWindow
        title="zsh"
        lines={[
          { prefix: '$', code: 'grobid' },
          { code: 'done', output: true },
        ]}
      />
    );
    expect(screen.getByText('zsh')).toBeInTheDocument();
    expect(screen.getByText('grobid')).toBeInTheDocument();
    expect(screen.getByText('done')).toBeInTheDocument();
  });
});
