// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

package phonestra;

import android.media.MediaCodecInfo;
import android.media.MediaCodecList;
import android.util.Range;

import java.io.PrintStream;
import java.util.Arrays;

/**
 * I codificatori audio e video del telefono ({@code MediaCodecList},
 * {@code REGULAR_CODECS}), per la prova A1 dell'audio e la 14 del video
 * (notes/study/audio.md, video.md): una riga per codificatore e tipo.
 *
 * <p>{@code nome \t tipo \t hardware|software|? \t fornitore|android \t alias
 * \t dettagli}; i dettagli sono {@code chiave=valore} separati da spazi
 * (profili come numeri di {@code MediaCodecInfo.CodecProfileLevel}, per AAC
 * anche col nome).
 */
final class Codificatori {
    private Codificatori() {
    }

    static void stampa() throws Exception {
        PrintStream uscita = new PrintStream(System.out, false, "UTF-8");
        for (MediaCodecInfo info : new MediaCodecList(MediaCodecList.REGULAR_CODECS).getCodecInfos()) {
            if (!info.isEncoder()) {
                continue;
            }
            String realizzazione = info.isHardwareAccelerated() ? "hardware" : info.isSoftwareOnly() ? "software" : "?";
            String alias = info.isAlias() ? "alias di " + info.getCanonicalName() : "-";
            for (String tipo : info.getSupportedTypes()) {
                if (!tipo.startsWith("audio/") && !tipo.startsWith("video/")) {
                    continue;
                }
                String dettagli;
                try {
                    dettagli = dettagli(info.getCapabilitiesForType(tipo), tipo);
                } catch (RuntimeException e) {
                    dettagli = "errore=" + e;
                }
                uscita.print(info.getName() + "\t" + tipo + "\t" + realizzazione + "\t"
                        + (info.isVendor() ? "fornitore" : "android") + "\t" + alias + "\t" + dettagli + "\n");
            }
        }
        uscita.flush();
    }

    private static String dettagli(MediaCodecInfo.CodecCapabilities c, String tipo) {
        StringBuilder s = new StringBuilder();
        s.append("istanze=").append(c.getMaxSupportedInstances());
        MediaCodecInfo.AudioCapabilities a = c.getAudioCapabilities();
        if (a != null) {
            s.append(" bitrate=").append(intervallo(a.getBitrateRange()))
                    .append(" canali=").append(a.getMaxInputChannelCount());
            int[] frequenze = a.getSupportedSampleRates();
            if (frequenze != null) {
                s.append(" frequenze=").append(Arrays.toString(frequenze).replace(" ", ""));
            } else {
                StringBuilder f = new StringBuilder();
                for (Range<Integer> r : a.getSupportedSampleRateRanges()) {
                    f.append(f.length() > 0 ? "," : "").append(intervallo(r));
                }
                s.append(" frequenze=").append(f);
            }
        }
        MediaCodecInfo.VideoCapabilities v = c.getVideoCapabilities();
        if (v != null) {
            s.append(" allineamento=").append(v.getWidthAlignment()).append('x').append(v.getHeightAlignment())
                    .append(" larghezze=").append(intervallo(v.getSupportedWidths()))
                    .append(" altezze=").append(intervallo(v.getSupportedHeights()))
                    .append(" fps=").append(intervallo(v.getSupportedFrameRates()))
                    .append(" bitrate=").append(intervallo(v.getBitrateRange()));
        }
        MediaCodecInfo.EncoderCapabilities e = c.getEncoderCapabilities();
        if (e != null) {
            s.append(" modi=");
            String[] nomi = {"CQ", "VBR", "CBR", "CBR_FD"};
            int[] modi = {
                MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CQ,
                MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_VBR,
                MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CBR,
                MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CBR_FD,
            };
            boolean primo = true;
            for (int i = 0; i < modi.length; i++) {
                if (e.isBitrateModeSupported(modi[i])) {
                    s.append(primo ? "" : ",").append(nomi[i]);
                    primo = false;
                }
            }
        }
        if (c.profileLevels != null && c.profileLevels.length > 0) {
            s.append(" profili=");
            for (int i = 0; i < c.profileLevels.length; i++) {
                MediaCodecInfo.CodecProfileLevel p = c.profileLevels[i];
                s.append(i > 0 ? "," : "").append(p.profile);
                if (tipo.equals("audio/mp4a-latm")) {
                    s.append(nomeAac(p.profile));
                } else {
                    s.append('/').append(p.level);
                }
            }
        }
        return s.toString();
    }

    /** I profili AAC che interessano (AACObjectLC, HE, HE v2, LD, ELD). */
    private static String nomeAac(int profilo) {
        switch (profilo) {
            case 2:
                return "(LC)";
            case 5:
                return "(HE)";
            case 29:
                return "(HEv2)";
            case 23:
                return "(LD)";
            case 39:
                return "(ELD)";
            default:
                return "";
        }
    }

    private static String intervallo(Range<Integer> r) {
        return r == null ? "?" : r.getLower() + "-" + r.getUpper();
    }
}
