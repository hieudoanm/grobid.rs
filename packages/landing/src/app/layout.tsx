import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import type { FC, ReactNode } from 'react';
import Footer from '@/components/molecules/Footer';
import Navbar from '@/components/molecules/Navbar';
import './globals.css';

const inter = Inter({ subsets: ['latin'] });

const GITHUB = 'https://github.com/hieudoanm/grobid.rs';
const LOGO =
  process.env.NODE_ENV === 'production' ? '/grobid.rs/logo.svg' : '/logo.svg';

const NAV_LINKS = [
  { label: 'Features', href: '#features' },
  { label: 'Demo', href: '#demo' },
  { label: 'How it works', href: '#how-it-works' },
  { label: 'Install', href: '#install' },
];

const DESCRIPTION =
  'Convert PDF articles to clean, structured Markdown via GROBID TEI XML — local-first, resumable, with metadata in YAML frontmatter.';

export const metadata: Metadata = {
  metadataBase: new URL('https://hieudoanm.github.io/grobid.rs/'),
  title: {
    default: 'grobid.rs — PDF to Markdown for LLMs',
    template: '%s — grobid.rs',
  },
  description: DESCRIPTION,
  openGraph: {
    siteName: 'grobid.rs',
    type: 'website',
    title: 'grobid.rs — PDF to Markdown for LLMs',
    description: DESCRIPTION,
    url: 'https://hieudoanm.github.io/grobid.rs/',
    images: ['/og.png'],
  },
  twitter: { card: 'summary_large_image', images: ['/og.png'] },
  icons: {
    icon: [
      { url: '/favicon.ico', sizes: 'any' },
      { url: '/favicon.svg', type: 'image/svg+xml' },
    ],
  },
};

interface RootLayoutProps {
  children: ReactNode;
}

const RootLayout: FC<RootLayoutProps> = ({ children }) => (
  <html
    lang="en"
    data-theme="grobidrs"
    className={`scroll-smooth antialiased motion-reduce:scroll-auto ${inter.className}`}>
    <body className="bg-base-100 text-base-content m-0 leading-relaxed">
      <Navbar
        name="grobid.rs"
        logoSrc={LOGO}
        links={NAV_LINKS}
        githubUrl={GITHUB}
      />
      {children}
      <Footer
        name="grobid.rs"
        tagline="Local PDF → Markdown via GROBID · MIT licensed"
        githubUrl={GITHUB}
      />
    </body>
  </html>
);

export default RootLayout;
