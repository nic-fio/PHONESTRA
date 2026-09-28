#!/usr/bin/env python3
"""Cerca in rete i telefoni Android con il Debug wireless attivo.

Manda una domanda mDNS per il servizio _adb-tls-connect._tcp con il bit QU
(risposta diretta a noi, non in multicast) e stampa nome, indirizzo e porta.
E' la prova che ha trovato il telefono quando `adb mdns services` non vedeva
niente (26 set 2026). Solo libreria standard.

Uso:  python3 mdns-cerca-telefono.py [secondi]
"""
import socket
import struct
import sys
import time

SERVIZIO = '_adb-tls-connect._tcp.local'


def codifica(nome):
    return b''.join(bytes([len(p)]) + p.encode() for p in nome.split('.') if p) + b'\0'


def leggi_nome(d, o):
    parti, salto = [], None
    while True:
        l = d[o]
        if l == 0:
            o += 1
            break
        if l & 0xC0 == 0xC0:  # puntatore di compressione
            if salto is None:
                salto = o + 2
            o = ((l & 0x3F) << 8) | d[o + 1]
            continue
        parti.append(d[o + 1:o + 1 + l].decode(errors='replace'))
        o += 1 + l
    return '.'.join(parti), (salto if salto is not None else o)


def cerca(durata):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.bind(('', 0))
    s.settimeout(1)
    # tipo 12 = PTR, classe 0x8001 = IN con bit QU
    domanda = struct.pack('>6H', 0, 0, 1, 0, 0, 0) + codifica(SERVIZIO) + struct.pack('>HH', 12, 0x8001)
    s.sendto(domanda, ('224.0.0.251', 5353))
    trovati = {}
    fine = time.time() + durata
    while time.time() < fine:
        try:
            d, _ = s.recvfrom(9000)
        except socket.timeout:
            continue
        _, _, qd, an, ns, ar = struct.unpack('>6H', d[:12])
        o = 12
        for _ in range(qd):
            _, o = leggi_nome(d, o)
            o += 4
        porte, indirizzi, bersagli = {}, {}, {}
        for _ in range(an + ns + ar):
            nome, o = leggi_nome(d, o)
            tipo, _, _, lung = struct.unpack('>HHIH', d[o:o + 10])
            o += 10
            dati = d[o:o + lung]
            if tipo == 33:  # SRV
                porte[nome] = struct.unpack('>H', dati[4:6])[0]
                bersagli[nome] = leggi_nome(d, o + 6)[0]
            elif tipo == 1:  # A
                indirizzi[nome] = socket.inet_ntoa(dati)
            o += lung
        for istanza, porta in porte.items():
            ip = indirizzi.get(bersagli[istanza], '?')
            # l'istanza e' adb-<numero di serie>-<suffisso>
            seriale = istanza.split('.')[0].split('-')[1] if istanza.startswith('adb-') else '?'
            trovati[istanza] = (seriale, ip, porta)
    return trovati


if __name__ == '__main__':
    risultati = cerca(float(sys.argv[1]) if len(sys.argv) > 1 else 4)
    if not risultati:
        print('Nessun telefono trovato (Debug wireless spento, altra rete o mDNS bloccato).')
    for istanza, (seriale, ip, porta) in risultati.items():
        print(f'{seriale}\t{ip}:{porta}\t{istanza}')
