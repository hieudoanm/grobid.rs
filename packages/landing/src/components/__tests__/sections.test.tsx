import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Hero from '../Hero';
import Features from '../Features';
import DemoSection from '../DemoSection';
import Pipeline from '../Pipeline';
import InstallSection from '../InstallSection';
import Footer from '../Footer';

describe('Hero', () => {
  it('renders headline, badges and stats', () => {
    render(<Hero />);
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      /PDFs into clean Markdown/
    );
    expect(screen.getByText('Local-first')).toBeInTheDocument();
    expect(screen.getByText('pdf → md')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Get started' })).toHaveAttribute(
      'href',
      '#install'
    );
  });
});

describe('Features', () => {
  it('renders all feature cards', () => {
    render(<Features />);
    expect(screen.getByText('Local-first & private')).toBeInTheDocument();
    expect(screen.getByText('One command')).toBeInTheDocument();
    expect(screen.getByText('Rich metadata')).toBeInTheDocument();
    expect(screen.getByText('Resumable')).toBeInTheDocument();
    expect(screen.getByText('Structure preserved')).toBeInTheDocument();
    expect(screen.getByText('Auto-starts GROBID')).toBeInTheDocument();
  });
});

describe('DemoSection', () => {
  it('renders terminal examples', () => {
    render(<DemoSection />);
    expect(screen.getByText('Convert a folder of PDFs')).toBeInTheDocument();
    expect(screen.getByText('Custom data directory')).toBeInTheDocument();
    expect(screen.getByText('Version')).toBeInTheDocument();
    expect(screen.getByText('grobid')).toBeInTheDocument();
  });
});

describe('Pipeline', () => {
  it('renders the three pipeline steps', () => {
    render(<Pipeline />);
    expect(screen.getByText('Drop in PDFs')).toBeInTheDocument();
    expect(screen.getByText('Run grobid')).toBeInTheDocument();
    expect(screen.getByText('Get Markdown')).toBeInTheDocument();
  });
});

describe('InstallSection', () => {
  it('renders prerequisites and install commands', () => {
    render(<InstallSection />);
    expect(screen.getByText('Docker')).toBeInTheDocument();
    expect(
      screen.getByText(/releases\/download\/latest\/grobid/)
    ).toBeInTheDocument();
    expect(
      screen.getByRole('link', { name: 'View on GitHub' })
    ).toHaveAttribute('href', 'https://github.com/hieudoanm/grobid.rs');
  });
});

describe('Footer', () => {
  it('renders footer columns and license link', () => {
    render(<Footer />);
    expect(screen.getByText('Product')).toBeInTheDocument();
    expect(screen.getByText('Resources')).toBeInTheDocument();
    expect(screen.getByText('Legal')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'MIT License' })).toHaveAttribute(
      'href',
      'https://github.com/hieudoanm/grobid.rs/blob/master/LICENSE'
    );
  });
});
