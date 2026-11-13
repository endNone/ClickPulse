# Release download artwork

Five local SVG badges distinguish Windows Portable x64, macOS DMG Apple Silicon/Intel x64 and Ubuntu ZIP x64/ARM64 downloads. The segmented layout, monitor icon and colored architecture labels are original ClickPulse artwork under GPL-3.0-or-later.

`scripts/release.py` links each badge to the corresponding attachment under the version tag. Image URLs use that tag, or an explicit immutable `artwork_revision` for a presentation-only correction, so an older release keeps its own artwork. All badges are 28 pixels tall, with `align="middle"` to center them vertically beside the platform text in GitHub Markdown. The release guides show the same layout with links to installation instructions until public release downloads exist.

Do not add a platform badge unless the release workflow actually builds and verifies its matching package. The images do not depend on a remote badge generator or downloaded fonts.
