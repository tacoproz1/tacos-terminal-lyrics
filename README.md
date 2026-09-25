# tacos-terminal-lyrics (Rust)

Letras sincronizadas en tu terminal, en letras grandes estilo bloque —
detecta la canción sola, busca la letra sola, y usa los colores de tu
propia terminal. Sin Python, sin pywal/matugen, sin pasos previos.

```
  ██   ██ ███████ ██      ██       ██████
  ██   ██ ██      ██      ██      ██    ██
  ███████ █████   ██      ██      ██    ██
  ██   ██ ██      ██      ██      ██    ██
  ██   ██ ███████ ███████ ███████  ██████
```

Fork en Rust de [tacoproz1/tacos-terminal-lyrics](https://github.com/tacoproz1/tacos-terminal-lyrics),
simplificado a un único binario:

- **Sin lrc-fetch / lrc-processor** — el visualizador busca la letra él solo
  en [lrclib.net](https://lrclib.net) con la metadata del player (artista,
  título, álbum y duración), con reintentos progresivos (título limpio →
  título original → búsqueda por query). Lo que baja se cachea en
  `~/.cache/tacos-lyrics/`.
- **Solo MPRIS** — cualquier reproductor que exponga MPRIS sirve:
  navegador (Firefox/Chrome al reproducir música), mpv, ytmgo, spotify,
  vlc, etc. No hay que configurar nada: se elige automáticamente el
  player activo (reproduciendo > pausado > con pista), y si cambias de
  player o de canción se detecta solo. Los players rotos del bus se
  saltan sin romper el escaneo.
- **ytmgo sin MPRIS también funciona** — ytmgo reproduce con un mpv que
  no expone MPRIS, así que hay un backend dedicado que lee la cola de
  su base de datos (`~/.local/share/ytmgo/ytmgo.db`), la posición del
  socket IPC de mpv (`/run/user/<uid>/ytmgo-mpv-*.sock`) y hasta
  reutiliza su propio cache de letras sincronizadas antes de ir a
  lrclib.
- **Colores que siguen tu tema** — por defecto las letras usan slots de
  la paleta de tu terminal, así heredan tu colorscheme (y sus cambios
  en vivo) sin configurar nada. Si quieres colores exactos o que una
  herramienta de theming (noctalia, matugen...) los genere, hay un
  archivo de tema simple que se recarga con SIGUSR1 — sin pywal ni
  dependencias: ver "Colores y temas" abajo.
- **Karaoke palabra a palabra** — el corte de color avanza palabra por
  palabra dentro de la línea (distribución uniforme, como el modo wlrc
  del original, pero calculado al vuelo: no hay que preprocesar nada).
  Pausa → la línea entera se atenúa; seek → salta a la línea correcta.

## Uso

```bash
tacos-lyrics            # todo automático
tacos-lyrics --compact   # fuente de 3 filas en vez de 5
tacos-lyrics --no-karaoke
tacos-lyrics --no-header
tacos-lyrics --refresh 100
```

Teclas: `q` o Ctrl-C para salir.

## Instalar

```bash
cargo install --git https://github.com/mikuri12/tacos-terminal-lyrics-rust
# o compilar del source:
git clone https://github.com/mikuri12/tacos-terminal-lyrics-rust
cd tacos-terminal-lyrics-rust && cargo build --release
```

Requisitos en tiempo de ejecución: ninguno más que el binario y un
player MPRIS (cualquier navegador moderno o reproductor de Linux ya lo
exponen). Conexión a internet solo la primera vez que escuchas cada
canción; luego sale del cache local.

## Cómo funciona

1. Escanea el bus de sesión buscando `org.mpris.MediaPlayer2.*` y elige
   el player más relevante (reproduciendo > pausado > con pista).
2. Con la metadata de la pista, consulta `lrclib.net/api/get`; si no
   hay resultado prueba `/api/search` con el título limpio (sin
   "(Official Video)", "feat...", etc.), luego con el original, y por
   último búsqueda por query. Solo acepta letras *sincronizadas* — las
   planas no sirven para un visualizador con timing.
3. Parsea el LRC (soporta varias marcas de tiempo por línea y tags
   `[ti:]`/`[ar:]`), lo cachea por "artista - título", y renderiza la
   línea activa con la posición real del player consultada en cada
   frame — por eso pausar y saltar funcionan sin lógica extra.
4. Los colores salen de un archivo de tema simple (`~/.config/tacos-lyrics/
   theme.txt`, o `--theme FILE`), con fallback a la paleta de la terminal;
   ver abajo.

## Colores y temas

Sin configurar nada, las letras usan slots de la paleta de tu terminal
(`38;5;N` / `39`), así siguen tu colorscheme vivo — cuando la terminal
cambia de tema, las letras cambian con ella.

Para fijar colores exactos hay un archivo de tema plano, `key = value`:

```
# ~/.config/tacos-lyrics/theme.txt
sung   = #cba6f7      # palabras ya cantadas (karaoke)
unsung = palette:8    # palabras pendientes
paused = dim          # la línea entera en pausa
header = default      # la línea "artista - título"
```

Valores: `#rrggbb`, `#rgb`, `palette:N` (slot 0-255 de tu terminal),
`default` (primer plano normal), `dim` (atenúa el primer plano).

El visualizador recarga el archivo al recibir `SIGUSR1`
(`kill -USR1 <pid>`), así las herramientas de theming pueden recolorear
la sesión en marcha. Hay dos plantillas en `templates/`:

- **Noctalia v5** — copia `templates/noctalia-theme.txt` a
  `~/.config/noctalia/templates/tacos-lyrics-theme.txt`,
  `templates/tacos-lyrics-reload.sh` a
  `~/.config/noctalia/templates/` (dale `chmod +x`), y añade a
  `~/.config/noctalia/templates.toml`:

  ```toml
  [theme.templates.user.tacos-lyrics]
  input_path  = "~/.config/noctalia/templates/tacos-lyrics-theme.txt"
  output_path = "~/.config/tacos-lyrics/theme.txt"
  post_hook   = "bash ~/.config/noctalia/templates/tacos-lyrics-reload.sh"
  ```

  Con esto, cada vez que cambies el colorscheme o el wallpaper en
  Noctalia, el visualizador corriendo se recolorea solo. La plantilla
  usa el color **primario** del scheme para las palabras cantadas y
  `dim` para el resto, así nunca se funde con el fondo de la terminal
  (los tonos derivados de `surface`/`on_surface` pueden ser casi
  idénticos al fondo). Verificado en vivo: Evie → Catppuccin con el
  visualizador en marcha.

- **matugen** (u otras herramientas) — cualquier plantilla que genere
  ese mismo formato de 4 líneas sirve; con matugen sería un template
  `.txt` con `{{colors.primary.default.hex}}` etc. y un post_hook que
  haga `pkill -USR1 tacos-lyrics`.

## Diferencias con el original

| Original (Python) | Este fork (Rust) |
|---|---|
| 3 herramientas (`lrc-fetch`, `lrc-processor`, `lrc-vis`) | 1 solo binario (`tacos-lyrics`) |
| Pre-proceso de la biblioteca en 2 pasos | Cero pasos: busca y renderiza on-the-fly |
| Formato wlrc intermedio | LRC estándar + karaoke calculado al vuelo |
| `playerctl` + python-pyyaml/mutagen/... | Sin dependencias de runtime |
| Config YAML + fuentes custom JSON | Flags de línea de comandos, 2 fuentes incluidas |
| Colores no implementados | Paleta de la terminal por defecto + archivo de tema + SIGUSR1 |

La fuente de letras grandes (5 filas) y la compacta (3 filas) son las
mismas del proyecto original.

## Licencia

MIT, como el proyecto original.
