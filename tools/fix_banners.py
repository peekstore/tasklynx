#!/usr/bin/env python3
import os

outdir = '/home/rkriad585/Projects/tasklynx/Tasklynx-logos'

letters = {
    'T': ('M12 15 L88 15 M50 15 L50 185', 100),
    'A': ('M12 185 L56 15 L100 185 M30 115 L82 115', 112),
    'S': ('M88 15 L12 15 L12 95 L88 95 L88 185 L12 185', 100),
    'K': ('M12 185 L12 15 M12 85 L102 15 M12 125 L102 185', 114),
    'L': ('M12 15 L12 185 L88 185', 100),
    'Y': ('M12 15 L56 100 L100 15 M56 100 L56 185', 112),
    'N': ('M12 185 L12 15 L100 185 L100 15', 112),
    'X': ('M12 15 L88 185 M88 15 L12 185', 100),
}

name = 'TASKLYNX'
stroke_w = 16
gap = 32

def branch_svg(stroke_color):
    return f'''    <g id="git-branch" transform="translate(0,10)">
      <line x1="40" y1="20" x2="40" y2="90" stroke="{stroke_color}" stroke-width="{stroke_w}" stroke-linecap="square"/>
      <line x1="40" y1="55" x2="100" y2="55" stroke="{stroke_color}" stroke-width="{stroke_w}" stroke-linecap="square"/>
      <circle cx="40" cy="20" r="14" fill="none" stroke="{stroke_color}" stroke-width="{stroke_w}"/>
      <circle cx="40" cy="90" r="14" fill="none" stroke="{stroke_color}" stroke-width="{stroke_w}"/>
      <circle cx="100" cy="55" r="14" fill="none" stroke="{stroke_color}" stroke-width="{stroke_w}"/>
      <circle cx="40" cy="55" r="7" fill="{stroke_color}"/>
    </g>'''

# Calculate dimensions
branch_w = 120
branch_h = 110
word_w = 0
for c in name:
    if c in letters:
        word_w += letters[c][1] + gap
word_w -= gap
total_w = branch_w + 60 + word_w
# Center in 1280x640
scale = 0.85
scaled_w = total_w * scale
scaled_h = 220 * scale
start_x = (1280 - scaled_w) / 2
start_y = (640 - scaled_h) / 2

for mode, stroke_color, bg_color in [('dark', '#f5f5f5', '#070809'), ('light', '#111111', '#ffffff')]:
    svg_parts = []
    svg_parts.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1280 640" width="1280" height="640">')
    svg_parts.append(f'  <rect width="100%" height="100%" fill="{bg_color}"/>')
    svg_parts.append(f'  <g transform="translate({start_x:.1f},{start_y:.1f}) scale({scale})">')
    svg_parts.append(branch_svg(stroke_color))
    x = branch_w + 60
    for c in name:
        if c in letters:
            d, w = letters[c]
            svg_parts.append(f'    <g transform="translate({x},5)">')
            svg_parts.append(f'      <path d="{d}" fill="none" stroke="{stroke_color}" stroke-width="{stroke_w}" stroke-linecap="square" stroke-linejoin="miter"/>')
            svg_parts.append(f'    </g>')
            x += w + gap
    svg_parts.append(f'  </g>')
    svg_parts.append('</svg>')
    with open(f'{outdir}/banner-on-{mode}-mode-1280x640.svg', 'w') as f:
        f.write('\n'.join(svg_parts))

print('fixed')
