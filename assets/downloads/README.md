# Release download artwork

Four local SVG badges distinguish Windows Setup/Portable x64 and macOS DMG Apple Silicon/Intel x64 downloads. The segmented layout, monitor icon and colored architecture labels are original ClickPulse artwork under GPL-3.0-or-later.

`scripts/release.py` links each badge to the corresponding attachment under the version tag. Image URLs also use that tag, so an older release keeps its own artwork. The release guides show the same layout with links to installation instructions until public release downloads exist.

Do not add a platform badge unless the release workflow actually builds and verifies its matching package. The images do not depend on a remote badge generator or downloaded fonts.
