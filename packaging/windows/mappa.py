#!/usr/bin/env python3
# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
# mappa.py DB CARTELLA USCITA.json: il pacchetto MSYS2 UCRT64 di ogni file
# della cartella, dal database dei file (ucrt64.files.tar.zst estratto in DB);
# stampa pacchetto, versione, licenza e numero di file.
import os,sys,json
db,root=sys.argv[1],sys.argv[2]
own={}
for d in os.listdir(db):
    fl=os.path.join(db,d,'files')
    if not os.path.exists(fl): continue
    desc=open(os.path.join(db,d,'desc')).read().split('\n')
    def field(n):
        i=desc.index('%'+n+'%'); return desc[i+1]
    name,ver=field('NAME'),field('VERSION')
    lic=field('LICENSE') if '%LICENSE%' in desc else ''
    base=field('BASE') if '%BASE%' in desc else name
    for line in open(fl).read().split('\n')[1:]:
        if line.startswith('ucrt64/') and not line.endswith('/'):
            own.setdefault(line[len('ucrt64/'):].lower(),[]).append((name,ver,lic,base))
res={};unk=[]
for dp,dn,fn in os.walk(root):
    for f in fn:
        rel=os.path.relpath(os.path.join(dp,f),root).replace('\\','/')
        cands=[rel.lower(),'bin/'+rel.lower()]
        hit=None
        for c in cands:
            if c in own: hit=own[c][0];break
        if hit: res.setdefault(hit,[]).append(rel)
        else: unk.append(rel)
json.dump({'pkgs':{f'{k[0]}|{k[1]}|{k[2]}|{k[3]}':v for k,v in res.items()},'unknown':unk},open(sys.argv[3],'w'),indent=1)
for k,v in sorted(res.items()): print(f'{k[0]:45} {k[1]:22} {k[2][:40]:40} {len(v)}')
print('UNKNOWN',len(unk),unk[:60])
