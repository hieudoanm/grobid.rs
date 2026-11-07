import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Home from '../page';

describe('Home page', () => {
  it('renders all landing sections', () => {
    render(<Home />);
    expect(
      screen.getByRole('heading', {
        name: /Turn research PDFs into clean Markdown/i,
      })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'Features' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'See it in action' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'How the CLI works' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'Get started' })
    ).toBeInTheDocument();
  });

  it('links to the GitHub repository', () => {
    render(<Home />);
    const links = screen.getAllByRole('link', { name: 'GitHub' });
    expect(links.length).toBeGreaterThan(0);
    expect(links[0]).toHaveAttribute(
      'href',
      'https://github.com/hieudoanm/grobid.rs'
    );
  });
});
