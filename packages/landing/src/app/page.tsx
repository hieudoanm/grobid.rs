import type { Metadata, NextPage } from 'next';
import type { FC } from 'react';
import CTA from '@/components/molecules/CTA';
import Features, { type Feature } from '@/components/molecules/Features';
import Hero from '@/components/molecules/Hero';
import SectionHead from '@/components/molecules/SectionHead';
import Terminal from '@/components/molecules/Terminal';
import {
  LuFileText,
  LuFile,
  LuLayoutGrid,
  LuLock,
  LuRefreshCw,
  LuSquareTerminal,
} from 'react-icons/lu';

export const metadata: Metadata = {
  alternates: { canonical: '/' },
};

const GITHUB = 'https://github.com/hieudoanm/grobid.rs';

const FEATURES: Feature[] = [
  {
    icon: LuLock,
    title: 'Local-first & private',
    body: 'GROBID runs in a Docker container on your machine. Your PDFs never leave your environment — safe for unpublished research and sensitive documents.',
  },
  {
    icon: LuSquareTerminal,
    title: 'One command',
    body: 'Drop PDFs into a pdf/ directory and run grobid. A single Rust binary orchestrates the whole pdf/ → tei/ → md/ pipeline.',
  },
  {
    icon: LuFileText,
    title: 'Rich metadata',
    body: 'Title, authors, year, journal, volume/issue/pages, and DOI are extracted into YAML frontmatter — ready for search, citation, and RAG pipelines.',
  },
  {
    icon: LuRefreshCw,
    title: 'Resumable',
    body: 'Already-converted TEI XML and Markdown files are skipped, so interrupted or repeated runs pick up exactly where they left off.',
  },
  {
    icon: LuLayoutGrid,
    title: 'Structure preserved',
    body: 'Headings, paragraphs, lists, tables, formulas, figures, and the reference list are rendered faithfully — no flat wall of text.',
  },
  {
    icon: LuFile,
    title: 'Auto-starts GROBID',
    body: 'If the GROBID service is not running, the CLI launches the official Docker image on port 8070 and waits until it is ready.',
  },
];

const HERO_LINES = [
  '$ grobid',
  '[INFO] Converting PDFs to Markdown via GROBID',
  '[INFO] [1/2] PDF → TEI: paper.pdf',
  '[INFO]   wrote paper.md',
  '[INFO] Conversion completed',
];

const DEMO_TERMINALS: { caption: string; lines: string[]; wide?: boolean }[] = [
  {
    caption: 'Convert a folder of PDFs',
    wide: true,
    lines: [
      '$ grobid',
      '[INFO] Converting PDFs to Markdown via GROBID',
      '[INFO] Processing 2 file(s) with 2 concurrent request(s)',
      '[INFO] [1/2] PDF → TEI: paper.pdf',
      '[INFO]   wrote paper.tei.xml',
      '[INFO] [1/2] TEI → MD: paper.tei.xml',
      '[INFO]   wrote paper.md',
      '[INFO] Conversion completed',
    ],
  },
  {
    caption: 'Custom data directory',
    lines: [
      '$ GROBID_DATA_DIR=~/papers grobid',
      '[INFO] PDF dir: /Users/you/papers/pdf',
      '[INFO] MD dir: /Users/you/papers/md',
    ],
  },
  {
    caption: 'Version',
    lines: ['$ grobid --version', 'grobid 0.0.1 (build 2026.11.07.23.10.04)'],
  },
];

const STEPS = [
  {
    title: 'Drop in PDFs',
    body: 'Put your PDFs into a pdf/ directory — or point GROBID_DATA_DIR anywhere else. The CLI auto-creates pdf/, tei/, and md/ on first run.',
    snippet: 'pdf/  ←  your papers',
  },
  {
    title: 'Run grobid',
    body: 'The CLI starts the GROBID Docker container if needed, then calls processFulltextDocument for each PDF and writes the TEI XML to tei/.',
    snippet: 'grobid   # starts GROBID on :8070 if required',
  },
  {
    title: 'Get Markdown',
    body: 'TEI is rendered to clean Markdown — headings, lists, tables, formulas, figures, and references — with metadata in YAML frontmatter.',
    snippet: 'md/  →  paper.md',
  },
];

const INSTALL_LINES = [
  '$ curl -fsSL https://raw.githubusercontent.com/hieudoanm/grobid.rs/master/scripts/install.sh | bash',
  'Installed to ~/.local/bin/grobid',
  '$ mkdir -p pdf && cp paper.pdf pdf/',
  '$ grobid',
  'md/paper.md',
];

const StepCard: FC<{ step: (typeof STEPS)[number]; index: number }> = ({
  step,
  index,
}) => (
  <article className="card border-base-300 bg-base-100 rounded-2xl border p-6">
    <span className="text-accent mb-2 block font-mono text-sm font-semibold">
      {String(index + 1).padStart(2, '0')}
    </span>
    <h3 className="text-base-content m-0 mb-2 text-lg font-bold">
      {step.title}
    </h3>
    <p className="text-base-content/60 text-sm">{step.body}</p>
    <code className="border-base-300 bg-base-200 text-base-content/60 mt-4 block overflow-x-auto rounded-lg border px-3 py-2 font-mono text-xs">
      {step.snippet}
    </code>
  </article>
);

const HomePage: NextPage = () => (
  <main>
    <Hero
      title="Turn research PDFs into clean Markdown for LLMs."
      description="grobid.rs converts PDFs to structured Markdown via GROBID TEI XML — entirely on your machine. Metadata lands in YAML frontmatter, sections and references stay intact, and nothing leaves your environment."
      primary={{ label: 'Get started', href: '#install' }}
      secondary={{ label: 'See it in action', href: '#demo' }}
      stats={[
        { value: 'pdf → md', label: 'one command' },
        { value: '0', label: 'cloud calls' },
        { value: 'resumable', label: 'skips finished files' },
      ]}>
      <Terminal caption="pdf/ → md/ in one command" lines={HERO_LINES} />
    </Hero>

    <Features
      title="Features"
      description="A focused pipeline that turns messy PDFs into LLM-ready Markdown — with honest TEI XML as the intermediate format."
      features={FEATURES}
    />

    <section id="demo" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="See it in action"
          description="One command converts every PDF in pdf/ — TEI XML lands in tei/ and Markdown with frontmatter lands in md/."
        />
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {DEMO_TERMINALS.map((t) => (
            <Terminal
              key={t.caption}
              caption={t.caption}
              lines={t.lines}
              className={t.wide ? 'lg:col-span-2' : ''}
            />
          ))}
        </div>
      </div>
    </section>

    <section id="how-it-works" className="py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="How the CLI works"
          description="A three-step local pipeline: PDFs in, structured Markdown out."
        />
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {STEPS.map((step, i) => (
            <StepCard key={step.title} step={step} index={i} />
          ))}
        </div>
      </div>
    </section>

    <section id="install" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-3xl px-5">
        <SectionHead
          title="Get started"
          description="Grab the prebuilt binary from GitHub Releases, or build from source with Cargo. Docker is only needed for the GROBID service, which the CLI can start for you. Requires Docker; Rust 1.85+ for source builds."
        />
        <Terminal caption="Install" lines={INSTALL_LINES} />
        <p className="text-base-content/60 mt-6 text-center text-sm">
          Or build from source:{' '}
          <code className="bg-base-300 rounded px-1.5 py-0.5 font-mono text-xs">
            ./scripts/install-local.sh
          </code>{' '}
          — see the{' '}
          <a
            href={`${GITHUB}/blob/master/README.md`}
            target="_blank"
            rel="noreferrer"
            className="text-primary font-semibold">
            README
          </a>{' '}
          for details.
        </p>
      </div>
    </section>

    <CTA
      title="Ready to convert?"
      description="One command turns your pdf/ directory into LLM-ready Markdown."
      primary={{ label: 'View on GitHub', href: GITHUB, external: true }}
      secondary={{
        label: 'Read the README',
        href: `${GITHUB}#readme`,
        external: true,
      }}
    />
  </main>
);

export default HomePage;
