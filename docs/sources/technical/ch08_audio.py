from build import arrow, box, c, fig, flow, p, path, rif, table, text, warn

FILI = fig(
    box(20, 40, 150, 60, "Reading thread", "priority −19", "navy")
    + box(200, 40, 150, 60, "Encoding thread", "AAC 192 kbit/s", "blue")
    + box(380, 40, 150, 60, "Queue", "256 packets, ~5 s", "amber")
    + box(560, 40, 150, 60, "Sending thread", "writes to the socket", "blue")
    + box(740, 40, 140, 60, "Audio channel", "to the PC", "dark")
    + arrow(172, 70, 198, 70) + arrow(352, 70, 378, 70) + arrow(532, 70, 558, 70) + arrow(712, 70, 738, 70)
    + path([(95, 102), (95, 170), (455, 170), (455, 104)], "#475569", True) + text(275, 190, "PCM: no encoding", 11)
    + box(620, 140, 260, 54, "Watch thread", "reads from the socket: sees the close", "light")
    + arrow(810, 102, 810, 138, "#475569", True),
    900, 206, "«FIG» — The four audio threads on the phone (CanaleAudio.java)")

S1 = p("The phone's audio plays from the PC's speakers, without interruptions and in sync with the video. The recipe came "
       "from measurements: loopback capture, AAC, timestamps from the sample count and a precise startup order.",
       lead=True) + \
    table(["Choice", "Why"], [
        ["Loopback capture: " + c("AudioPolicy") + " with " + c("ROUTE_FLAG_LOOP_BACK") + " on the sound usages "
         "(media, games, assistant, navigation, system sounds…: " + c("Audio.USI") + ")",
         "Meanwhile the phone stays silent; once the policy is removed, it plays again by itself. No setting to restore."],
        ["AAC-LC 192 kbit/s, 48 kHz stereo (Android's software encoder)", "As clean as PCM (" + c("notes/connection-tests.md") + " §42) and "
         "much lighter on Wi-Fi. PCM as a test fallback."],
        ["Timestamps from the sample count (samples × 10⁶ / 48000)", "Regular: the encoder's output time "
         "comes in bursts (1–3 ms and 30–40 ms instead of 21)."],
        ["Reading at priority −19, and reading, encoding and sending on separate threads", "If Wi-Fi or the encoder "
         "slow down, reading does not stop and no samples are lost."],
    ], "«TAB» — The audio recipe") + \
    p("On the phone (" + c("CanaleAudio.java") + ", which uses the classes of the measuring tool " + c("Audio.java")
      + ") there are four threads: reading (" + c("audio-lettura") + "), encoding (" + c("audio-codifica") + "), sending (" + c("audio-spedizione")
      + "; queue of 256 packets, about 5 s, which drops the oldest ones, counted in " + c("persi") + ") and the watch thread ("
      + c("audio-sentinella") + "), which reads from the socket only to notice the close. Each thread catches its "
      "own errors: an audio problem closes the channel with an " + c("errore") + " line, never the service. Only "
      "one audio channel at a time: a new one stops the old one and waits (at most 3 s) for it to have removed its "
      "policy.") + FILI

S2 = p("The audio channel goes only from the phone to the PC: " + c("orario u64 BE · lunghezza u32 BE · dati") + ". The PC "
       "sends nothing; closing the channel stops the capture.", lead=True) + \
    table(["Flag in the timestamp", "Content"], [
        ["bit 61", "UTF-8 text of the form " + c("chiave=valore …") + ". The first packet is always " + c("inizio")
         + " (" + c("formato=aac frequenza=48000 canali=2 bitrate=192000 sorgente=loopback buffer_ms=… "
         "registrazione=istanza|statica") + ") or " + c("errore …") + "; then " + c("lettura tid=… nice=…")
         + ", " + c("misura") + " (one per second), " + c("avviso") + ", " + c("errore") + "."],
        ["bit 62", "Codec configuration (" + c("AudioSpecificConfig") + ", 2 bytes " + c("11 90")
         + "), before any data."],
        ["none", "Data: an AAC frame of 1024 samples (21.333 ms), or 1024 PCM samples."],
    ], "«TAB» — The packets of the audio channel")

S3 = p("On the PC three pieces give each packet its timestamp and decide when to play it; then GStreamer decodes "
       "and plays it.", lead=True) + flow([("Stream", "audio channel", "navy"), ("Timestamps", "regular timestamps", "blue"),
           ("Playback margin", "when to play", "blue"), ("appsrc", "raw AAC, codec_data", "blue"),
           ("avdec_aac", "audioconvert, resample", "blue"), ("autoaudiosink", "PC speakers", "light")],
          "«FIG» — Audio from the channel to the speakers; a copy of the packets goes to the recording", width=960) + \
    table(["Piece", "What it does"], [
        [c("Durate") + " (durations)", "The duration of each packet from the sample count (21,333 or 21,334 µs, with no accumulated error)."],
        [c("Orari") + " (timestamps)", "Keeps the timestamps regular and realigns only beyond a 60 ms deviation."],
        [c("Margine") + " (playback margin)", "Decides when to play each packet: phone timestamp + an offset fixed from the first "
         "packet. It starts at 80 ms; a late packet (less than 10 ms before now) moves everything later "
         "(a moment of silence, then no gaps) and widens the margin by 40 ms, up to 300. A phone more than 200 ms "
         "ahead beyond the margin triggers a realignment."],
        [c("Margine::scendi"), "After 10 s of calm it reduces the margin by skipping a “silence” packet (less than 40 % "
         "of the average bytes). With AAC the packet size is almost constant and it never triggers ("
         + rif("Appendix C — Known issues") + ")."],
    ], "«TAB» — The pieces of playback") + \
    p(c("audio_nostro::riproduci(apritore)") + " is the function the connection calls; it ends if the channel "
      "closes and, when cancelled, closes it. " + c("PHONESTRA_AUDIO_CODEC=pcm") + " (or " + c("raw") + ") uses PCM "
      "instead of AAC.")

S4 = p("Audio capture does not start together with the connection: it waits for the drawer's mirror. It is a rule found "
       "through measurements.", lead=True) + warn("audio capture starts after the mirror of the drawer's screen, and 5 s after the mirror is open ("
          + c("ATTESA_SPECCHIO") + " 10 s at most, " + c("ASSESTAMENTO") + " 5 s, in " + c("collegamento.rs")
          + "). If the mirror is recreated, the capture restarts after 300 ms. With the capture started before the "
          "connection's initial sessions had started, the Facebook reels player in the windows starved "
          "and the audio had micro-interruptions; verified with alternating tests (" + c("notes/connection-tests.md") + " §48–49). Android's internal "
          "mechanism is not yet understood: do not change this order without redoing those tests.",
          "A rule not to lose.") + \
    p("The drawer signals the opening of the mirror with " + c("Collegamento::specchio_aperto()") + ", which increments "
      "a " + c("watch") + " counter followed by the audio task. The task only looks at the mirrors opened "
      "after the current service started: after a reconnection it waits for the new mirror (prove §53).")

S5 = p("The capture removes itself in every way the service can end, even the most abrupt.",
       lead=True) + table(["How it ends", "Who removes the audio policy"], [
    ["The PC closes the channel, or a new one is opened", c("CanaleAudio") + ": stops the recorder and calls "
     + c("unregisterAudioPolicy")],
    ["The service exits with " + c("System.exit"), "A shutdown hook (" + c("audio-fine") + ")"],
    ["The service dies suddenly (" + c("kill -9") + ")", "Android: the policy is tied with " + c("linkToDeath")
     + " to our process"],
], "«TAB» — Who removes the capture") + \
    p("The guardian has no action for the audio: no shell command removes another process's policy, and none is "
      "needed. When Phonestra closes, before detaching the audio, the connection pauses the media that are "
      "playing, otherwise they would resume from the phone's speaker (" + rif("Shutdown") + ").")

S6 = p("The “Record the screen” button in an app's window writes an MP4: the H.264 video as it is (" + c("h264parse ! mp4mux")
       + ") and the AAC audio as it is, without re-encoding. " + c("audio_nostro::ascolta()") + " gives a copy of the "
       "packets (" + c("broadcast") + "), " + c("caps_registrazione()") + " the caps with the " + c("codec_data")
       + " of the current audio. It starts from the first keyframe; with no AAC audio in progress the file has no "
       "audio.", lead=True) + \
    p("During the recording the app's display does not change size and is not recreated for orientation (the window "
      "scales the image), and the button shows the elapsed time (" + c("● m:ss") + "). The file goes to "
      + c("<XDG Videos>/Phonestra/<app> YYYY-MM-DD HH.MM.SS.mp4") + ".")

CHAPTER = ("Audio", [
    ("The recipe", S1),
    ("Audio packets", S2),
    ("Playback on the PC", S3),
    ("The startup order", S4),
    ("Who removes the capture", S5),
    ("Recording", S6),
])
