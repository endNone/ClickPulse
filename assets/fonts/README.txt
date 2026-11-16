Bundled fonts: local WOFF2 subsets

Inter: https://github.com/rsms/inter
Noto Sans Mono: https://github.com/google/fonts/tree/main/ofl/notosansmono

The bundled files are modified subsets, renamed internally with a ClickPulse prefix.
They retain Latin and Latin Extended characters, punctuation, currencies, arrows,
and math symbols. Variable weights remain available. Reproduce them with
scripts/subset_fonts.py and fonttools[woff], using the original TTF files from
v0.3.0/assets/fonts. The script checks the original file hashes.

Each font is distributed under SIL Open Font License 1.1.
Copyright notices and complete licenses remain in the *-OFL.txt files.
Fonts load locally on demand; no font is downloaded or installed at runtime.
Chinese, Japanese and Korean use regional system fonts. Ubuntu installations
need fonts-noto-cjk; see docs/install-linux.md.
