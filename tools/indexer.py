#!/usr/bin/env python3
"""Bioconda recipe indexer for oxo-call.

Parses bioconda-recipes meta.yaml files to build:
1. Tool alias map (entry_points → package name)
2. Tool metadata index (version, dependencies, homepage)
3. SQLite database for fast lookups

Usage:
    python tools/indexer.py --recipes-dir /path/to/bioconda-recipes/recipes/
    python tools/indexer.py --update  # incremental update

Output:
    tools/bioconda_index.db  — SQLite database
    tools/alias_map.json     — binary name → package name mapping
"""

import os
import sys
import json
import sqlite3
import re
import argparse
from pathlib import Path
from collections import defaultdict

# Jinja2-less YAML parsing for bioconda meta.yaml
# bioconda uses Jinja2 templates: {% set name = "humann" %}, {{ name }}, {{ version }}
# We need to resolve these before YAML parsing

JINJA_SET_RE = re.compile(r'\{%\s*set\s+(\w+)\s*=\s*"([^"]*)"\s*%\}')
JINJA_VAR_RE = re.compile(r'\{\{\s*(\w+)(?:\|lower)?\s*\}\}')

def preprocess_meta(content: str) -> str:
    """Resolve Jinja2 variables in meta.yaml content."""
    # Extract {% set ... %} variables
    vars = {}
    for m in JINJA_SET_RE.finditer(content):
        vars[m.group(1)] = m.group(2)

    # Replace {{ var }} references
    def replace_var(m):
        name = m.group(1)
        if name == 'name':
            return vars.get('name', 'UNKNOWN')
        if name == 'version':
            return vars.get('version', '0.0.0')
        return vars.get(name, m.group(0))

    result = JINJA_VAR_RE.sub(replace_var, content)
    return result


def parse_meta_yaml(filepath: str) -> dict | None:
    """Parse a bioconda meta.yaml file, resolving Jinja2 templates."""
    try:
        with open(filepath) as f:
            content = f.read()
    except Exception:
        return None

    # Resolve Jinja2
    content = preprocess_meta(content)

    # Simple key-value extraction (avoid full YAML parser dependency)
    info = {
        'package_name': '',
        'version': '',
        'homepage': '',
        'summary': '',
        'entry_points': [],
        'run_deps': [],
        'source_url': '',
    }

    # Extract package name
    m = re.search(r'^\s*name:\s*"([^"]+)"', content, re.MULTILINE)
    if not m:
        m = re.search(r"^\s*name:\s*'([^']+)'", content, re.MULTILINE)
    if not m:
        m = re.search(r'^\s*name:\s*(\S+)', content, re.MULTILINE)
    if m:
        info['package_name'] = m.group(1)

    # Extract version
    m = re.search(r'^\s*version:\s*"([^"]+)"', content, re.MULTILINE)
    if not m:
        m = re.search(r"^\s*version:\s*'([^']+)'", content, re.MULTILINE)
    if not m:
        m = re.search(r'^\s*version:\s*(\S+)', content, re.MULTILINE)
    if m:
        info['version'] = m.group(1)

    # Extract homepage/about
    m = re.search(r'^\s*home:\s*"?([^"\n]+)"?', content, re.MULTILINE)
    if m:
        info['homepage'] = m.group(1).strip()

    m = re.search(r'^\s*summary:\s*"([^"]+)"', content, re.MULTILINE)
    if not m:
        m = re.search(r"^\s*summary:\s*'([^']+)'", content, re.MULTILINE)
    if m:
        info['summary'] = m.group(1)

    # Extract entry_points (binary names)
    ep_section = False
    for line in content.split('\n'):
        if 'entry_points:' in line:
            ep_section = True
            continue
        if ep_section:
            if line.strip().startswith('-') and '=' in line:
                # - humann = humann.humann:main
                ep = line.split('=')[0].replace('-', '').strip()
                info['entry_points'].append(ep)
            elif line.strip() and not line.startswith(' ') and not line.startswith('-'):
                ep_section = False

    # Extract run dependencies
    run_section = False
    for line in content.split('\n'):
        if re.match(r'^\s*run:\s*$', line):
            run_section = True
            continue
        if run_section:
            dep = line.strip().lstrip('-').strip()
            if dep and not dep.startswith('#'):
                info['run_deps'].append(dep)
            if line.strip() and not line.startswith(' ') and not line.startswith('-'):
                run_section = False

    # Extract source URL
    m = re.search(r'^\s*url:\s*"([^"]+)"', content, re.MULTILINE)
    if not m:
        m = re.search(r"^\s*url:\s*'([^']+)'", content, re.MULTILINE)
    if m:
        info['source_url'] = m.group(1)

    return info if info['package_name'] else None


def build_index(recipes_dir: str, db_path: str, alias_path: str):
    """Build SQLite index and alias map from bioconda recipes."""
    recipes_path = Path(recipes_dir)
    if not recipes_path.exists():
        print(f"Error: recipes directory not found: {recipes_dir}")
        return

    # Find all meta.yaml files
    meta_files = list(recipes_path.glob("*/meta.yaml"))
    print(f"Found {len(meta_files)} recipes")

    # Setup SQLite
    conn = sqlite3.connect(db_path)
    conn.execute("""
        CREATE TABLE IF NOT EXISTS packages (
            name TEXT PRIMARY KEY,
            version TEXT,
            homepage TEXT,
            summary TEXT,
            source_url TEXT,
            entry_points TEXT,  -- JSON array
            run_deps TEXT,       -- JSON array
            indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)
    conn.execute("CREATE INDEX IF NOT EXISTS idx_pkg_name ON packages(name)")

    # Build alias map
    alias_map = {}  # binary_name → package_name

    inserted = 0
    for mf in meta_files:
        info = parse_meta_yaml(str(mf))
        if not info or not info['package_name']:
            continue

        name = info['package_name']
        conn.execute("""
            INSERT OR REPLACE INTO packages
            (name, version, homepage, summary, source_url, entry_points, run_deps)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        """, (
            name, info['version'], info['homepage'], info['summary'],
            info['source_url'],
            json.dumps(info['entry_points']),
            json.dumps(info['run_deps'])
        ))
        inserted += 1

        # Map entry points to package name
        for ep in info['entry_points']:
            alias_map[ep] = name
        # Also map the package name itself
        alias_map[name] = name

    conn.commit()
    conn.close()

    # Write alias map
    with open(alias_path, 'w') as f:
        json.dump(alias_map, f, indent=2)

    print(f"Indexed {inserted} packages")
    print(f"Alias map: {len(alias_map)} entries (binary names → package names)")

    # Show some statistics
    multi_binary = sum(1 for info in [parse_meta_yaml(str(mf)) for mf in meta_files]
                       if info and len(info['entry_points']) > 3)
    print(f"Packages with 4+ entry_points: {multi_binary}")

    # Show overlap with oxo-call tools
    oxo_tools = set()
    with open('docs/bench/reference_commands.csv') as f:
        for line in f:
            oxo_tools.add(line.split(',')[0])

    covered = oxo_tools & set(alias_map.keys())
    print(f"oxo-call tools in bioconda: {len(covered)}/{len(oxo_tools)}")
    missing = oxo_tools - set(alias_map.keys())
    if missing:
        print(f"  Missing: {sorted(missing)[:20]}...")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Bioconda recipe indexer for oxo-call')
    parser.add_argument('--recipes-dir', default='/data/home/wsx/Projects/bioconda-recipes/recipes',
                        help='Path to bioconda-recipes/recipes/')
    parser.add_argument('--db', default='tools/bioconda_index.db',
                        help='Output SQLite database path')
    parser.add_argument('--alias-map', default='tools/alias_map.json',
                        help='Output alias map JSON path')
    args = parser.parse_args()

    build_index(args.recipes_dir, args.db, args.alias_map)
