#!/usr/bin/env python3
"""Analisi del CSV di PHONESTRA_AVSYNC (notes/av-sync-tests.md).

Marcatore video: un salto di luminosità media di almeno SALTO punti in un
fotogramma (lampo, o il cerchio che torna pieno nel video di prova di 30
minuti). Marcatore audio: il primo campione sopra la soglia dopo almeno
SILENZIO_MS di silenzio (bip). Ogni salto si abbina al bip che continua lo scarto
precedente; scarto = bip − salto (positivo: audio in ritardo sul video).

Uso: avsync-analizza.py <file.csv> [--salto 4] [--minuti 1]
"""
import argparse
import statistics as st

p = argparse.ArgumentParser()
p.add_argument("csv")
p.add_argument("--salto", type=float, default=4.0, help="salto minimo di luminosità (0-255)")
p.add_argument("--minuti", type=float, default=1.0, help="ampiezza delle righe del riassunto")
a = p.parse_args()

SILENZIO_MS = 200
FREQUENZA = 48000

video, audio, margini = [], [], []
for riga in open(a.csv):
    c = riga.strip().split(",")
    if c[0] == "V":
        video.append((float(c[1]), float(c[2])))
    elif c[0] == "A":
        audio.append((float(c[1]), float(c[2]), int(c[3]), int(c[4])))
    elif c[0] == "M":
        margini.append((float(c[1]), int(c[2])))

# Salti di luminosità (fronte di salita, al massimo uno ogni 500 ms).
lampi = []
for (t0, l0), (t1, l1) in zip(video, video[1:]):
    if l1 - l0 >= a.salto and (not lampi or t1 - lampi[-1] > 500):
        lampi.append(t1)

# Bip: primo campione sonoro dopo un silenzio.
bip, ultimo_suono = [], -1e9
for t, rms, inizio, n in audio:
    if inizio >= 0:
        istante = t + inizio * 1000 / FREQUENZA
        if istante - ultimo_suono > SILENZIO_MS:
            bip.append(istante)
        ultimo_suono = t + n * 1000 / FREQUENZA

# Ogni salto si abbina al bip che dà lo scarto più vicino al precedente:
# così uno scarto oltre mezzo periodo (500 ms) non salta al bip sbagliato.
coppie = []
precedente = 0.0
j = 0
for t in lampi:
    while j < len(bip) and bip[j] < t - 1500:
        j += 1
    candidati = [b - t for b in bip[j:j + 4] if abs(b - t) <= 1500]
    if candidati:
        scarto = min(candidati, key=lambda s: abs(s - precedente))
        if abs(scarto - precedente) <= 300 or not coppie:
            coppie.append((t, scarto))
            precedente = scarto

def margine_a(t):
    m = None
    for tm, v in margini:
        if tm > t:
            break
        m = v
    return m

print(f"fotogrammi {len(video)}, salti {len(lampi)}, bip {len(bip)}, coppie {len(coppie)}")
if not coppie:
    raise SystemExit
inizio = coppie[0][0]
print(f"{'min':>5} {'n':>4} {'media':>7} {'dev':>6} {'min':>7} {'max':>7} {'margine':>8}")
passo = a.minuti * 60000
k = 0
while k < len(coppie):
    blocco = [s for t, s in coppie[k:] if t < coppie[k][0] - (coppie[k][0] - inizio) % passo + passo]
    t0 = coppie[k][0]
    dev = st.pstdev(blocco) if len(blocco) > 1 else 0
    print(f"{(t0 - inizio) / 60000:5.1f} {len(blocco):4d} {st.mean(blocco):+7.1f} {dev:6.1f} {min(blocco):+7.1f} {max(blocco):+7.1f} {margine_a(t0)!s:>8}")
    k += len(blocco)
tutti = [s for _, s in coppie]
print(f"tutto: media {st.mean(tutti):+.1f} ms, dev {st.pstdev(tutti):.1f}, min {min(tutti):+.1f}, max {max(tutti):+.1f}")
