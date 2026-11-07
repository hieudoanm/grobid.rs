import type { FC } from 'react';
import TerminalWindow, { type TerminalLine } from './TerminalWindow';

const EXAMPLES: { title: string; lines: TerminalLine[] }[] = [
  {
    title: 'Convert a folder of PDFs',
    lines: [
      { prefix: '$', code: 'grobid' },
      {
        code: '[INFO] Converting PDFs to Markdown via GROBID',
        output: true,
      },
      { code: '[INFO] [1/2] PDF → TEI: paper.pdf', output: true },
      { code: '[INFO] wrote paper.tei.xml', output: true },
      { code: '[INFO] [1/2] TEI → MD: paper.tei.xml', output: true },
      { code: '[INFO] wrote paper.md', output: true },
      { code: '[INFO] Conversion completed', output: true },
    ],
  },
  {
    title: 'Custom data directory',
    lines: [
      {
        prefix: '$',
        code: 'GROBID_DATA_DIR=~/papers grobid',
      },
      {
        code: '[INFO] PDF dir: /Users/you/papers/pdf',
        output: true,
      },
      {
        code: '[INFO] MD dir: /Users/you/papers/md',
        output: true,
      },
    ],
  },
  {
    title: 'Version',
    lines: [
      { prefix: '$', code: 'grobid --version' },
      { code: 'grobid 2026.11.07.21.33.09', output: true },
    ],
  },
];

const DemoSection: FC = () => {
  return (
    <section id="demo" className="bg-base-200 py-20">
      <div className="mx-auto max-w-6xl px-4">
        <div className="text-center">
          <h2 className="text-3xl font-bold">See it in action</h2>
          <p className="text-base-content/70 mt-4">
            One command converts every PDF in pdf/ — TEI XML lands in tei/ and
            Markdown with frontmatter lands in md/.
          </p>
        </div>
        <div className="mt-12 grid gap-6 lg:grid-cols-3">
          {EXAMPLES.map((example) => (
            <TerminalWindow
              key={example.title}
              title={example.title}
              lines={example.lines}
            />
          ))}
        </div>
      </div>
    </section>
  );
};

export default DemoSection;
