# usage (from test_output/): bash ../tools/ui_audit.sh <out>  -> runs every autotest phase (4 at a time) into test_output/<out>/p<N>/ with one sheet.jpg per phase; review the sheets after UI changes. HB_EXE=<target dir> picks another build.
out=${1:-ui_audit}; rm -rf $out; mkdir -p $out
one() { d=$out/p$1; mkdir -p $d; (cd $d && HOLLOWBROOK_AUTOTEST=. HOLLOWBROOK_AUTOTEST_PHASE=$1 HOLLOWBROOK_DEV_TIME=2026-11-16T11:00:00 timeout 300 ../../../${HB_EXE:-target/release}/hollowbrook.exe > run.log 2>&1; echo "exit $?" >> run.log); }
n=0; for p in 0 1 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 40 41 42 43 44 45 46 47 48; do one $p & n=$((n+1)); [ $((n%4)) = 0 ] && wait; done; wait
python - $out <<'PY'
from PIL import Image
import glob,sys,os
for d in sorted(glob.glob(sys.argv[1]+'/p*')):
    fs=sorted(glob.glob(d+'/**/*.png',recursive=True))
    if not fs: continue
    ims=[Image.open(f).convert('RGB') for f in fs]; W,H=480,270; c=3; r=(len(ims)+c-1)//c
    g=Image.new('RGB',(W*c,H*r))
    for i,im in enumerate(ims): g.paste(im.resize((W,H)),((i%c)*W,(i//c)*H))
    g.save(d+'/sheet.jpg',quality=80)
print('sheets ok')
PY
grep -l "panick\|exit [1-9]" $out/p*/run.log
