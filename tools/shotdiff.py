"""Compare two screenshot trees (baseline vs new). Prints changed images and writes before|after|diff strips.
Exit code 1 if any image changed beyond the threshold or is missing."""
import sys, os, glob
from PIL import Image, ImageChops
base, new, out = sys.argv[1:4]
THRESH = 4.0  # mean absolute difference (0-255) treated as a real change; tuned above animation noise
os.makedirs(out, exist_ok=True)
bad = 0
for b in sorted(glob.glob(os.path.join(base, '**', '*.png'), recursive=True)):
    rel = os.path.relpath(b, base); n = os.path.join(new, rel)
    if not os.path.exists(n):
        print(f"MISSING  {rel}"); bad += 1; continue
    A = Image.open(b).convert('RGB'); B = Image.open(n).convert('RGB').resize(A.size)
    D = ImageChops.difference(A, B); g = D.convert('L')
    mean = sum(i * c for i, c in enumerate(g.histogram())) / (A.size[0] * A.size[1])
    if mean > THRESH:
        bad += 1
        w, h = A.size; s = Image.new('RGB', (w * 3, h)); s.paste(A, (0, 0)); s.paste(B, (w, 0)); s.paste(D.point(lambda v: min(255, v * 4)), (w * 2, 0))
        s.resize((w * 3 // 2, h // 2)).save(os.path.join(out, rel.replace(os.sep, '_')))
        print(f"CHANGED  {rel}  (difference {mean:.1f})")
print(f"shots: {bad} changed or missing" if bad else "shots: OK, nothing changed noticeably")
sys.exit(1 if bad else 0)
