import type { Metadata, Viewport } from 'next';
import { Geist, Geist_Mono } from 'next/font/google';
import { assetPath } from '@/lib/site';
import './globals.css';

const geistSans = Geist({
  variable: '--font-geist-sans',
  subsets: ['latin'],
});

const geistMono = Geist_Mono({
  variable: '--font-geist-mono',
  subsets: ['latin'],
});

const TITLE = 'grobid.rs — PDF to Markdown for LLMs';
const DESCRIPTION =
  'Convert PDF articles to clean, structured Markdown via GROBID TEI XML — local-first, resumable, with metadata in YAML frontmatter.';

export const metadata: Metadata = {
  metadataBase: new URL('https://hieudoanm.github.io'),
  title: TITLE,
  description: DESCRIPTION,
  openGraph: {
    title: TITLE,
    description: DESCRIPTION,
    url: 'https://hieudoanm.github.io/grobid.rs',
    siteName: 'grobid.rs',
    type: 'website',
    locale: 'en_US',
    images: [
      {
        url: assetPath('/svg/og-image.png'),
        width: 1280,
        height: 640,
        alt: 'grobid.rs — PDF to Markdown for LLMs',
      },
    ],
  },
  twitter: {
    card: 'summary_large_image',
    title: TITLE,
    description: DESCRIPTION,
    images: [assetPath('/svg/og-image.png')],
  },
  icons: {
    icon: assetPath('/favicon.ico'),
    shortcut: assetPath('/favicon.ico'),
    apple: assetPath('/svg/logo.svg'),
  },
};

export const viewport: Viewport = {
  themeColor: [
    { media: '(prefers-color-scheme: light)', color: '#f7f7f4' },
    { media: '(prefers-color-scheme: dark)', color: '#16161a' },
  ],
};

const themeInitScript = `(function(){try{var t=localStorage.getItem('grobid-theme');if(t!=='grobid'&&t!=='grobid-dark'){t=window.matchMedia('(prefers-color-scheme: dark)').matches?'grobid-dark':'grobid'}document.documentElement.setAttribute('data-theme',t)}catch(e){document.documentElement.setAttribute('data-theme','grobid')}})();`;

const RootLayout = ({ children }: { children: React.ReactNode }) => {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
      suppressHydrationWarning>
      <body className="bg-base-100 text-base-content flex min-h-full flex-col">
        <script dangerouslySetInnerHTML={{ __html: themeInitScript }} />
        {children}
      </body>
    </html>
  );
};

export default RootLayout;
