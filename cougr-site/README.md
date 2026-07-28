# Cougr Site

This repository builds the documentation site for [Cougr](https://github.com/salazarsebas/Cougr). 

**Wait!** The content on this site is not hosted here. The actual markdown files live in the main `salazarsebas/Cougr` repository to ensure docs are versioned alongside the code they describe. This repository merely acts as the presentation layer.

## How it works

At build time, the `sync.py` script clones the `Cougr` repo, copies the documentation files into the correct `src/` subdirectories according to our Information Architecture (IA), and rewrites relative markdown links to ensure they work in mdBook. 

Then, GitHub Actions builds the `mdbook` and deploys it to GitHub Pages. The action runs automatically every night and on manual dispatch.

## Local Development

If you want to test the site structure or styling locally:

1. Install mdBook:
   ```bash
   cargo install mdbook
   ```
2. Run the sync script to pull down the latest content from the core repo:
   ```bash
   python3 sync.py
   ```
3. Build and serve the site locally:
   ```bash
   mdbook serve --open
   ```

## Contributing to Content

If you want to edit a page (like a tutorial, pattern guide, or the architecture docs), please open a pull request against the main `salazarsebas/Cougr` repository. Once merged there, this site will sync it automatically.
