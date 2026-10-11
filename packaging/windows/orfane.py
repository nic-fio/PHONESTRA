#!/usr/bin/env python3
# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
# orfane.py CARTELLA: le DLL della cartella che phonestra.exe, i plugin di
# GStreamer e i loader di gdk-pixbuf non usano (né direttamente né per via
# di altre DLL), una per riga: si possono togliere.
import subprocess,os,sys,re
root=sys.argv[1]
roots=['phonestra.exe']+[os.path.join(d,f) for d in ['lib/gstreamer-1.0','lib/gdk-pixbuf-2.0/2.10.0/loaders'] for f in os.listdir(os.path.join(root,d)) if f.endswith('.dll')]
top={f.lower():f for f in os.listdir(root) if f.lower().endswith('.dll')}
seen=set();todo=list(roots)
while todo:
    f=todo.pop()
    out=subprocess.run(['objdump','-p',os.path.join(root,f)],capture_output=True,text=True).stdout
    for d in re.findall(r'DLL Name: (\S+)',out):
        d=d.lower()
        if d in top and d not in seen: seen.add(d); todo.append(top[d])
print('\n'.join(sorted(top[k] for k in top if k not in seen)))
