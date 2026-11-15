"""Rebuild the checked-in WOFF2 subsets from the original SIL OFL fonts.

Requires fonttools[woff]. Pass a folder containing the original TTFs from
v0.3.0/assets/fonts; checked hashes prevent a silent upstream/font change.
"""
import argparse
import hashlib
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

SOURCES = {
    'Inter': '29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031',
    'NotoSansMono': '2cb2adb378a8f574213e23df697050b83c54c27df465a2015552740b2769a081',
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('original_fonts', type=Path)
    parser.add_argument('--output', type=Path, default=Path(__file__).resolve().parent.parent/'assets/fonts')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    for name, digest in SOURCES.items():
        source = args.original_fonts/(name+'.ttf')
        if hashlib.sha256(source.read_bytes()).hexdigest() != digest:
            parser.error('Unexpected original font: '+str(source))
        font = TTFont(source, recalcTimestamp=False)
        options = subset.Options()
        options.recalc_timestamp = False
        options.name_IDs = ['*']
        options.name_legacy = True
        options.name_languages = ['*']
        worker = subset.Subsetter(options=options)
        # Latin/Latin Extended, punctuation, currencies, arrows and math symbols.
        worker.populate(unicodes=list(range(0x20, 0x250))+list(range(0x2000, 0x2300)))
        worker.subset(font)
        family = 'ClickPulse '+name+' Subset'
        for record in font['name'].names:
            if record.nameID in (1, 3, 4, 6, 16):
                value = family.replace(' ', '') if record.nameID == 6 else family
                record.string = value.encode(record.getEncoding())
        font.flavor = 'woff2'
        target = args.output/(name+'.woff2')
        font.save(target)
        print(f'{target.name}: {target.stat().st_size} bytes')


if __name__ == '__main__':
    main()
