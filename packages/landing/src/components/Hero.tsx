import type { FC } from 'react';

const STATS = [
  { value: 'pdf → md', label: 'one command' },
  { value: '0', label: 'cloud calls' },
  { value: 'resumable', label: 'skips finished files' },
];

const Hero: FC = () => {
  return (
    <section className="hero bg-base-200 py-16 sm:py-24">
      <div className="hero-content flex-col items-center text-center">
        <div className="mb-4 flex flex-wrap justify-center gap-2">
          <span className="badge badge-primary">Local-first</span>
          <span className="badge badge-primary">GROBID in Docker</span>
          <span className="badge badge-primary">MIT licensed</span>
        </div>
        <h1 className="max-w-3xl text-4xl font-bold sm:text-5xl">
          Turn research PDFs into clean Markdown for LLMs.
        </h1>
        <p className="text-base-content/70 mt-6 max-w-2xl text-lg">
          grobid.rs converts PDFs to structured Markdown via GROBID TEI XML —
          entirely on your machine. Metadata lands in YAML frontmatter, sections
          and references stay intact, and nothing leaves your environment.
        </p>
        <div className="mt-8 flex flex-wrap justify-center gap-4">
          <a href="#install" className="btn btn-primary">
            Get started
          </a>
          <a href="#demo" className="btn btn-ghost">
            See it in action
          </a>
        </div>
        <div className="mt-12 grid grid-cols-3 gap-8">
          {STATS.map((stat) => (
            <div key={stat.label}>
              <div className="text-3xl font-bold">{stat.value}</div>
              <div className="text-base-content/60 text-sm">{stat.label}</div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Hero;
