"""Overview pages for a ui_audit run: every screenshot, small, labelled, 36 per page.
usage (from test_output/): python ../tools/audit_pages.py <audit dir>  -> <audit dir>/page0.jpg, page1.jpg, ..."""
import sys, glob, os
from PIL import Image, ImageDraw
root = sys.argv[1]
fs = sorted(glob.glob(os.path.join(root, 'p*', '**', '*.png'), recursive=True))
W, H, C = 320, 180, 6
for k in range(0, len(fs), 36):
    pg = fs[k:k + 36]
    g = Image.new('RGB', (W * C, H * ((len(pg) + C - 1) // C)))
    d = ImageDraw.Draw(g)
    for i, f in enumerate(pg):
        x, y = (i % C) * W, (i // C) * H
        g.paste(Image.open(f).convert('RGB').resize((W, H)), (x, y))
        d.rectangle([x, y + H - 14, x + W, y + H], fill=(0, 0, 0))
        d.text((x + 4, y + H - 13), os.path.relpath(f, root).replace(os.sep, '/'), fill=(255, 255, 0))
    g.save(os.path.join(root, f'page{k // 36}.jpg'), quality=82)
print(f"{len(fs)} screenshots, {(len(fs) + 35) // 36} pages")
