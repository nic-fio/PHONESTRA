/*
 * Sostituto di libcairo-script-interpreter-2.dll per Phonestra su Windows.
 *
 * L'interprete vero di cairo usa liblzo2 (GPL-2.0-or-later). GTK lo chiama
 * solo per leggere i nodi «cairo script» dei file di debug del disegno
 * (gsk/gskrendernodeparser.c). Questo sostituto non crea mai una superficie:
 * GTK risponde «Cairo script did not create a surface», come un GTK compilato
 * senza l'interprete. Nient'altro di Phonestra ne dipende.
 *
 * Copyright (c) 2026 Nicola Fiorillo. Licenza: MIT.
 */
typedef int cairo_status_t;
typedef struct _csi csi_t;
typedef struct _csi_hooks csi_hooks_t;

#define CAIRO_STATUS_SUCCESS 0
#define CAIRO_STATUS_INVALID_STATUS 26 /* stesso valore di cairo.h */

static char interprete_finto;

__declspec(dllexport) csi_t *cairo_script_interpreter_create(void)
{
    return (csi_t *)&interprete_finto;
}

__declspec(dllexport) cairo_status_t cairo_script_interpreter_install_hooks(csi_t *ctx, const csi_hooks_t *hooks)
{
    (void)ctx; (void)hooks;
    return CAIRO_STATUS_SUCCESS;
}

__declspec(dllexport) cairo_status_t cairo_script_interpreter_feed_string(csi_t *ctx, const char *line, int len)
{
    (void)ctx; (void)line; (void)len;
    return CAIRO_STATUS_INVALID_STATUS;
}

__declspec(dllexport) cairo_status_t cairo_script_interpreter_destroy(csi_t *ctx)
{
    (void)ctx;
    return CAIRO_STATUS_INVALID_STATUS;
}
