#!/usr/bin/env python3
# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
# zippa.py CARTELLA ZIP: lo zip portatile, con le barre «/» nei percorsi, i
# file in ordine e date fisse (lo stesso contenuto dà lo stesso zip).
import os,sys,zipfile
src,dst=sys.argv[1],sys.argv[2]
base=os.path.basename(src.rstrip('/'))
files=[]
for dp,dn,fn in os.walk(src):
    dn.sort()
    for f in sorted(fn): files.append(os.path.join(dp,f))
with zipfile.ZipFile(dst,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as z:
    for p in files:
        arc=base+'/'+os.path.relpath(p,src).replace(os.sep,'/')
        zi=zipfile.ZipInfo(arc,date_time=(2026,10,11,12,0,0))
        zi.compress_type=zipfile.ZIP_DEFLATED
        zi.external_attr=0o644<<16
        with open(p,'rb') as fh: z.writestr(zi,fh.read(),compresslevel=9)
print(len(files),'file')
