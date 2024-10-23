#!/bin/bash

# Install rsvg-convert on MacOS with:
# $ brew install librsvg

rsvg-convert -h 180 -b white static/icon.svg > static/apple-touch-icon-180x180.png
rsvg-convert -h 512 -b white static/icon.svg > static/maskable-icon-512x512.png
rsvg-convert -h 512 static/icon.svg > static/pwa-512x512.png
rsvg-convert -h 192 static/icon.svg > static/pwa-192x192.png
rsvg-convert -h 64 static/icon.svg > static/pwa-64x64.png
