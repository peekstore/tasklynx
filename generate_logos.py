#!/usr/bin/env python3
import os

outdir = '/home/rkriad585/Projects/tasklynx/tasklynx-logos'
os.makedirs(outdir, exist_ok=True)

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
stroke_w = 14
gap = 28

total_w = 0
for c in name:
    if c in letters:
        total_w += letters[c][1] + gap
total_w -= gap

def branch_svg(stroke_color):
    return f'''  <g id="git-branch" transform="translate(60,30)">
    <line x1="40" y1="20" x2="40" y2="80" stroke="{stroke_color}" stroke-width="{stroke_w}" stroke-linecap="square"/>
    <line x1="40" y1="50" x2="90" y2="50" stroke="{stroke_color}" stroke-width="{stroke_w}" stroke-linecap="square"/>
    <circle cx="40" cy="20" r="12" fill="none" stroke="{stroke_color}" stroke-width="{stroke_w}"/>
    <circle cx="40" cy="80" r="12" fill="none" stroke="{stroke_color}" stroke-width="{stroke_w}"/>
    <circle cx="90" cy="50" r="12" fill="none" stroke="{stroke_color}" stroke-width="{stroke_w}"/>
    <circle cx="40" cy="50" r="6" fill="{stroke_color}"/>
  </g>'''

def make_wordmark_svg(stroke_color, bg_color=None, include_icon=True):
    svg_parts = []
    svg_parts.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w + 280} {220}" width="{total_w + 280}" height="220">')
    if bg_color:
        svg_parts.append(f'  <rect width="100%" height="100%" fill="{bg_color}"/>')
    if include_icon:
        svg_parts.append(branch_svg(stroke_color))
    x = 240 if include_icon else 0
    for c in name:
        if c in letters:
            d, w = letters[c]
            svg_parts.append(f'  <g transform="translate({x},10)">')
            svg_parts.append(f'    <path d="{d}" fill="none" stroke="{stroke_color}" stroke-width="{stroke_w}" stroke-linecap="square" stroke-linejoin="miter"/>')
            svg_parts.append(f'  </g>')
            x += w + gap
    svg_parts.append('</svg>')
    return '\n'.join(svg_parts)

with open(f'{outdir}/banner-on-dark-mode.svg', 'w') as f:
    f.write(make_wordmark_svg('#f5f5f5', None, True))
with open(f'{outdir}/banner-on-light-mode.svg', 'w') as f:
    f.write(make_wordmark_svg('#111111', None, True))

icon_dark = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" width="200" height="200">
  <g transform="translate(40,30)">
    <line x1="40" y1="20" x2="40" y2="80" stroke="#f5f5f5" stroke-width="14" stroke-linecap="square"/>
    <line x1="40" y1="50" x2="90" y2="50" stroke="#f5f5f5" stroke-width="14" stroke-linecap="square"/>
    <circle cx="40" cy="20" r="12" fill="none" stroke="#f5f5f5" stroke-width="14"/>
    <circle cx="40" cy="80" r="12" fill="none" stroke="#f5f5f5" stroke-width="14"/>
    <circle cx="90" cy="50" r="12" fill="none" stroke="#f5f5f5" stroke-width="14"/>
    <circle cx="40" cy="50" r="6" fill="#f5f5f5"/>
  </g>
</svg>'''
icon_light = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" width="200" height="200">
  <g transform="translate(40,30)">
    <line x1="40" y1="20" x2="40" y2="80" stroke="#111111" stroke-width="14" stroke-linecap="square"/>
    <line x1="40" y1="50" x2="90" y2="50" stroke="#111111" stroke-width="14" stroke-linecap="square"/>
    <circle cx="40" cy="20" r="12" fill="none" stroke="#111111" stroke-width="14"/>
    <circle cx="40" cy="80" r="12" fill="none" stroke="#111111" stroke-width="14"/>
    <circle cx="90" cy="50" r="12" fill="none" stroke="#111111" stroke-width="14"/>
    <circle cx="40" cy="50" r="6" fill="#111111"/>
  </g>
</svg>'''

with open(f'{outdir}/icon-on-dark-mode.svg', 'w') as f:
    f.write(icon_dark)
with open(f'{outdir}/icon-on-light-mode.svg', 'w') as f:
    f.write(icon_light)
with open(f'{outdir}/favicon-on-dark-mode.svg', 'w') as f:
    f.write(icon_dark.replace('#f5f5f5', '#ff453a'))
with open(f'{outdir}/favicon-on-light-mode.svg', 'w') as f:
    f.write(icon_light.replace('#111111', '#ff453a'))

with open(f'{outdir}/banner-on-dark-mode-1280x640.svg', 'w') as f:
    svg_parts = []
    svg_parts.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1280 640" width="1280" height="640">')
    svg_parts.append(f'  <rect width="100%" height="100%" fill="#070809"/>')
    svg_parts.append(f'  <g transform="translate(200,220) scale(0.9)">')
    svg_parts.append(branch_svg('#f5f5f5'))
    x = 240
    for c in name:
        if c in letters:
            d, w = letters[c]
            svg_parts.append(f'    <g transform="translate({x},10)">')
            svg_parts.append(f'      <path d="{d}" fill="none" stroke="#f5f5f5" stroke-width="{stroke_w}" stroke-linecap="square" stroke-linejoin="miter"/>')
            svg_parts.append(f'    </g>')
            x += w + gap
    svg_parts.append(f'  </g>')
    svg_parts.append('</svg>')
    f.write('\n'.join(svg_parts))

with open(f'{outdir}/banner-on-light-mode-1280x640.svg', 'w') as f:
    svg_parts = []
    svg_parts.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1280 640" width="1280" height="640">')
    svg_parts.append(f'  <rect width="100%" height="100%" fill="#ffffff"/>')
    svg_parts.append(f'  <g transform="translate(200,220) scale(0.9)">')
    svg_parts.append(branch_svg('#111111'))
    x = 240
    for c in name:
        if c in letters:
            d, w = letters[c]
            svg_parts.append(f'    <g transform="translate({x},10)">')
            svg_parts.append(f'      <path d="{d}" fill="none" stroke="#111111" stroke-width="{stroke_w}" stroke-linecap="square" stroke-linejoin="miter"/>')
            svg_parts.append(f'    </g>')
            x += w + gap
    svg_parts.append(f'  </g>')
    svg_parts.append('</svg>')
    f.write('\n'.join(svg_parts))

print('done')
