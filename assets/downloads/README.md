# Release download artwork

Release tables use small platform icons beside platform names, followed by underlined text download links. The icons are 18 pixels square and use `align="absmiddle"` for vertical alignment in GitHub Markdown. GitHub Feed may omit the icons; platform names and package links remain ordinary text.

`scripts/release.py` pins icons to the version tag or an explicit immutable `artwork_revision`. Historical Releases can reference newer artwork without changing their tags or packages. Only include platforms with published packages.

## Platform icons

- `platform-windows.svg`: original four-pane mark, GPL-3.0-or-later.
- `platform-apple.svg`: [Simple Icons Apple](https://github.com/simple-icons/simple-icons/blob/17cd4ee07612198a1d8777a060d4100423762a8b/icons/apple.svg), recolored dark gray.
- `platform-ubuntu.svg`: [Simple Icons Ubuntu](https://github.com/simple-icons/simple-icons/blob/17cd4ee07612198a1d8777a060d4100423762a8b/icons/ubuntu.svg), recolored orange.

The Simple Icons artwork is distributed under [CC0-1.0](https://github.com/simple-icons/simple-icons/blob/17cd4ee07612198a1d8777a060d4100423762a8b/LICENSE.md). Platform names and logos identify compatibility; their trademarks belong to their respective owners.

## Earlier badges

The five segmented download badges remain available for older links. Their monitor icon and colored architecture labels are original ClickPulse artwork under GPL-3.0-or-later. They do not depend on a remote badge generator or downloaded fonts.
