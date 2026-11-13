# @grobid/landing

Static landing page for grobid.rs — plain HTML + CSS, no build step.

## Structure

```text
index.html    # single-page site (nav, hero, features, demo, pipeline, install, footer)
404.html      # GitHub Pages 404 page
styles.css    # all styles; light page, dark terminals
logo.svg      # brand mark (nav, 404)
og-image.png  # Open Graph image (1280x640)
favicon.ico
robots.txt    # allow all + sitemap reference
sitemap.xml   # single-URL sitemap for the Pages site
humans.txt    # project/team credits
```

Asset paths are relative, so the same files work locally and under the GitHub
Pages `/grobid.rs/` base path. Open `index.html` directly or serve the
directory:

```bash
python3 -m http.server 8000 -d packages/landing
```

## Deployment

CI deploys this directory to GitHub Pages on pushes to `master` — there is no
build or test step.
