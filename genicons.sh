#!/bin/bash
set -e

# Install rsvg-convert on MacOS with:
# $ brew install librsvg

cp koso.svg frontend/static/koso.svg
cp koso.svg frontend/src/lib/components/ui/koso-logo/koso.svg

cat koso.svg | sed 's|<g |<g transform="translate(100 100) scale(.8 .8)" |g' > icon.svg
rsvg-convert -h 180 -b white icon.svg > frontend/static/apple-touch-icon-180x180.png
rsvg-convert -h 512 -b white icon.svg > frontend/static/maskable-icon-512x512.png
rsvg-convert -h 512 icon.svg > frontend/static/pwa-512x512.png
rsvg-convert -h 192 icon.svg > frontend/static/pwa-192x192.png
rsvg-convert -h 64 icon.svg > frontend/static/pwa-64x64.png
rm icon.svg
