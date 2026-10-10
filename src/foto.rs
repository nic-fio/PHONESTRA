// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Immagini delle finestre per le prove dell'interfaccia: con
//! `PHONESTRA_FOTO=<cartella>` ogni finestra si salva in `<cartella>/<nome>.png`
//! pochi secondi dopo l'apertura e poi ogni 10 s (sovrascrivendo).

use adw::prelude::*;

pub fn prendi(finestra: &impl IsA<gtk::Widget>, nome: &str) {
    let Some(cartella) = std::env::var_os("PHONESTRA_FOTO") else {
        return;
    };
    let percorso = std::path::PathBuf::from(cartella).join(format!("{nome}.png"));
    let debole = finestra.as_ref().downgrade();
    gtk::glib::timeout_add_seconds_local(6, move || {
        let Some(w) = debole.upgrade() else {
            return gtk::glib::ControlFlow::Break;
        };
        if let Err(e) = salva(&w, &percorso) {
            eprintln!("[foto] {}: {e}", percorso.display());
        }
        gtk::glib::ControlFlow::Continue
    });
}

fn salva(w: &gtk::Widget, percorso: &std::path::Path) -> Result<(), String> {
    let (l, a) = (w.width() as f64, w.height() as f64);
    if l < 1.0 || a < 1.0 {
        return Err("finestra non ancora disegnata".into());
    }
    let dipinto = gtk::WidgetPaintable::new(Some(w));
    let istantanea = gtk::Snapshot::new();
    dipinto.snapshot(&istantanea, l, a);
    let nodo = istantanea.to_node().ok_or("niente da disegnare")?;
    let disegnatore = w.native().ok_or("finestra senza superficie")?.renderer().ok_or("senza renderer")?;
    let texture = disegnatore.render_texture(nodo, None);
    texture.save_to_png(percorso).map_err(|e| e.to_string())
}
