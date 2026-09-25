#!/usr/bin/env bash
# post_hook for the noctalia tacos-lyrics theme template.
# Recolors every running tacos-lyrics instance after noctalia regenerates
# the theme file, so the visualizer follows the colorscheme live.
for pid in $(pgrep -x tacos-lyrics); do
    kill -USR1 "$pid" 2>/dev/null
done
