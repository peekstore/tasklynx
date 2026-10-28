#!/usr/bin/env python3
"""Generate placeholder screenshots for Tasklynx docs using PIL."""
from PIL import Image, ImageDraw, ImageFont
import os

os.makedirs('/home/rkriad585/Projects/tasklynx/Screenshots', exist_ok=True)

def make_screenshot(name, title, lines):
    img = Image.new('RGB', (1280, 720), color='#070809')
    draw = ImageDraw.Draw(img)
    # Header
    draw.rectangle([(0,0),(1280,60)], fill='#101216')
    draw.text((20,18), '● TASKLYNX', fill='#ff453a')
    draw.text((200,18), title, fill='#f5f5f5')
    # Content
    y = 90
    for line in lines:
        draw.text((40,y), line, fill='#f5f5f5')
        y += 28
    img.save(f'/home/rkriad585/Projects/tasklynx/Screenshots/{name}.png')
    print(f'Created {name}.png')

make_screenshot('home', 'dashboard', [
    'LOCAL / DEVELOPER WORKSPACE',
    '',
    '[TODO]        Fix login bug        high   website',
    '[DOING]       Add API endpoint     normal backend',
    '[DONE]        Setup CI             low    devops',
])

make_screenshot('board', 'kanban board', [
    'TODO',
    '  - Fix login bug (high, website)',
    '',
    'DOING',
    '  - Add API endpoint (normal, backend)',
    '',
    'DONE',
    '  - Setup CI (low, devops)',
])

make_screenshot('web', 'web dashboard', [
    'Tasklynx Web Dashboard',
    'Responsive terminal-style UI at http://127.0.0.1:8787',
])
