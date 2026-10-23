# @grobid/landing

Static landing page for grobid.rs — plain HTML + CSS, no build step.

## Structure

```
index.html    # single-page site (nav, hero, features, demo, pipeline, install, footer)
styles.css    # all styles; light page, dark terminals
svg/          # logo.svg + og-image.png
favicon.ico
```

Asset paths are relative, so the same files work locally and under the
GitHub Pages `/grobid.rs/` base path. Open `index.html` directly or serve the
directory:

```bash
python3 -m http.server 8000 -d packages/landing
```

## Deployment

CI deploys this directory to GitHub Pages on pushes to `master` — there is no
build or test step.
