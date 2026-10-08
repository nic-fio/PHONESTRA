# What's new in Phonestra

Every version of Phonestra, newest first. The current version is always on the
[download page](./#get); older versions are not offered any more.

## 1.3.0 — October 2026

- **No GPL libraries in the AppImage.** GTK no longer brings libjbig and
  liblzo2 with it: libtiff is built without JBIG and the cairo script
  interpreter, used only by GTK's debugging tools, is left out. Nothing changes
  in how Phonestra works.

## 1.2.0 — October 2026

- **No more FFmpeg.** The phone now sends its sound as **Opus** (128 kbit/s),
  decoded on the PC by libopus. The video is decoded by the graphics card
  (VA-API on Intel and AMD, NVDEC on NVIDIA) and, on PCs without one, by
  **OpenH264**. The AppImage is half the size (55 MB instead of 102).
- After a Wi-Fi hiccup the audio playback margin can now shrink again during
  silent passages, so sound and picture return to a shorter delay.
- Screen recordings keep the phone's sound as it is, now in Opus.

## 1.1.1 — October 2026

- The sound stays **in sync with the video even after many minutes**: when the
  PC's sound card and the phone drift apart, Phonestra keeps correcting the
  timing instead of letting the sound slowly fall out of step.

## 1.1.0 — October 2026

- **Audio and video in sync.** While a player is playing on the phone
  (YouTube, Facebook reels, music apps…), every frame is shown at the moment
  its sound comes out of the speakers. Before this version the sound arrived
  250–530 ms after the image; now about 150 ms, steady.
- When nothing is playing, frames are shown as soon as they arrive, so
  browsing and typing stay immediate.

## 1.0.0 — October 2026

The first version published on this site.

- **Italian and English** interface, chosen automatically or in Preferences.
- Phones are called “Phone” or “Tablet” with their model, in the language of
  the interface.
- New licence: Phonestra is **freeware**, free to use at home and at work
  ([licence](licence.html)).
- libusb is now linked dynamically, like the other libraries of the AppImage.

## Release candidates — September 2026

Versions 1.0.0-rc.1 to rc.8 were published for testing only.

- **rc.8**: “Receive files…”, to copy photos, videos and documents from the
  phone to the PC.
- **rc.4 to rc.7**: the phone's screen stays off while you use it from the PC
  and turns on for calls; the user's screen timeout is preserved; no more
  micro-interruptions in reels with the screen off.
- **rc.3**: official logo in the About window and in the AppImage icon.
- **rc.1 and rc.2**: Phonestra works entirely with its own component on the
  phone, without scrcpy.
