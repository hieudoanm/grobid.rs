import type { FC } from 'react';

const STEPS = [
  {
    step: '01',
    title: 'Drop in PDFs',
    description:
      'Put your PDFs into a pdf/ directory — or point GROBID_DATA_DIR anywhere else. The CLI auto-creates pdf/, tei/, and md/ on first run.',
    command: 'pdf/  ←  your papers',
  },
  {
    step: '02',
    title: 'Run grobid',
    description:
      'The CLI starts the GROBID Docker container if needed, then calls processFulltextDocument for each PDF and writes the TEI XML to tei/.',
    command: 'grobid   # starts GROBID on :8070 if required',
  },
  {
    step: '03',
    title: 'Get Markdown',
    description:
      'TEI is rendered to clean Markdown — headings, lists, tables, formulas, figures, and references — with metadata in YAML frontmatter.',
    command: 'md/  →  paper.md',
  },
];

const Pipeline: FC = () => {
  return (
    <section id="how-it-works" className="py-20">
      <div className="mx-auto max-w-6xl px-4">
        <div className="text-center">
          <h2 className="text-3xl font-bold">How the CLI works</h2>
          <p className="text-base-content/70 mt-4">
            A three-step local pipeline: PDFs in, structured Markdown out.
          </p>
        </div>
        <div className="mt-12 grid gap-6 md:grid-cols-3">
          {STEPS.map((item) => (
            <div key={item.step} className="card bg-base-100 shadow-sm">
              <div className="card-body">
                <div className="flex items-center gap-3">
                  <span className="badge badge-primary badge-lg">
                    {item.step}
                  </span>
                  <h3 className="card-title">{item.title}</h3>
                </div>
                <p className="text-base-content/70">{item.description}</p>
                <code className="rounded-box bg-base-300 mt-2 block px-3 py-2 font-mono text-sm">
                  {item.command}
                </code>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Pipeline;
