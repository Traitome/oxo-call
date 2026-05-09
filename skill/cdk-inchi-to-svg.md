---
name: cdk-inchi-to-svg
category: cheminformatics
description: A command-line tool that converts InChI (International Chemical Identifier) strings into SVG (Scalable Vector Graphics) molecular structure diagrams using the Chemistry Development Kit (CDK) rendering engine. Supports customizable display options for atoms, bonds, and overall layout.
tags:
  - chemistry
  - InChI
  - SVG
  - molecular-structure
  - CDK
  - visualization
  - cheminformatics
author: AI-generated
source_url: https://github.com/cdk/cdk
---

## Concepts

- **InChI Input Format**: The tool accepts standard InChI strings (e.g., `InChI=1S/C6H6/c1-2-3-4-5-6-1/h1-6H`) including layer-1 (connected skeleton) and layer-2 (isotopic, proton, charge, metal atom) information. Malformed or non-standard InChI strings cause parsing failures, so validate input before processing.

- **SVG Output Generation**: The tool renders 2D molecular structure diagrams with configurable atom symbol fonts, bond widths, and stereochemical indicators. The SVG output is vector-based, meaning it scales without quality loss, making it suitable for publications and web integration.

- **CDK Rendering Model**: CDK uses a coordinate-free representation for structure generation, then applies layout algorithms (e.g.,ACD/Lab MolFile conventions) to compute atom positions. If the InChI lacks explicit coordinates, CDK infers a plausible 2D layout; if coordinates are embedded (InChI auxiliary data), those take precedence.

- **Atom and Bond Styling Options**: The tool exposes flags for controlling background color, canvas dimensions, atom label font size, bond length, and highlight options for specific atoms or fragments. These options affect only the visual representation, not the chemical interpretation.

## Pitfalls

- **Invalid or Truncated InChI Strings**: Providing an incomplete InChI (missing the version layer `InChI=1S/` prefix) causes the parser to reject the input entirely, resulting in a blank SVG or error message. Always verify InChI strings include the required version indicator before passing them to the tool.

- **Oversized Canvas without Scaling**: Setting extremely large canvas dimensions (e.g., `--width 10000 --height 10000`) without adjusting bond scale factors produces microscopic molecular features, rendering the output impractical for viewing. Balance canvas size with the `--bond-length` option to maintain readable proportions.

- **Ignoring Stereochemical Flag Conflicts**: When an InChI contains stereochemical layers that the CDK layout algorithm cannot fully represent (e.g., allene axial chirality), the tool generates a flat projection and may silently omit stereobond indicators. Always inspect the output for completeness when working with chiral compounds.

- **Assuming Standard Bond Orders for Metallic Compounds**: For InChI strings representing organometallic complexes, CDK's default bond assignment may produce chemically implausible connectivity, especially for metal-ligand bonds with fractional orders. The SVG will render these bonds but they may not reflect the actual bonding situation in the molecule.

- **Output File Overwrite without Confirmation**: By default, the tool writes output to the specified SVG path without prompting, silently overwriting any existing file. Always backup or rename existing output files before re-running the tool with the same destination path.

## Examples

### Convert a simple benzene InChI to SVG with default settings
**Args:** `--inchi "InChI=1S/C6H6/c1-2-3-4-5-6-1/h1-6H" --output benzene.svg`
**Explanation:** This converts the benzene InChI string to an SVG using default canvas size (400x400), default bond lengths, and white background.

### Set custom canvas dimensions for a large molecule
**Args:** `--inchi "InChI=1S/C8H10/c1-2-3-4-5-6-7-8-1/h1-8H" --output xylene.svg --width 800 --height 600`
**Explanation:** Specifying explicit width and height ensures the entire molecular structure fits within an 800x600 pixel canvas without clipping.

### Customize atom label font size for presentation-ready output
**Args:** `--inchi "InChI=1S/H2O/h1H2/p+1" --output water.svg --atom-font-size 16`
**Explanation:** Setting a larger atom font size (16pt) improves readability when the SVG is used directly in presentations or documents without further scaling.

### Change background color to black for dark-themed documents
**Args:** `--inchi "InChI=1S/C2H6O/c1-2-3-4-5-6-1/h3-4H2" --output ethanol.svg --background "#1a1a1a" --foreground "#ffffff"`
**Explanation:** Specifying a dark background with white foreground creates an SVG suitable for dark-themed websites or slide decks.

### Highlight specific atoms using a highlight color
**Args:** `--inchi "InChI=1S/C3H9N/c1-2-4-3-5-1/h4-5H2,1H3" --output propylamine.svg --highlight "1,2" --highlight-color "#ff0000"`
**Explanation:** The highlight flags draw a red outline around atoms at indices 1 and 2, useful for emphasizing functional groups or reactive sites in educational diagrams.

### Adjust bond length to control molecular density in the output
**Args:** `--inchi "InChI=1S/C4H8/c1-2-3-4-1/h1-4H" --output butene.svg --bond-length 2.5`
**Explanation:** Setting a shorter bond length (2.5 units) compresses the structure, making it suitable for fitting multiple molecules in a small canvas area.

### Write SVG to stdout for piping into other tools
**Args:** `--inchi "InChI=1S/CH4/h1H4" --output -`
**Explanation:** Using a dash as the output path directs the SVG content to standard output, enabling pipeline processing with tools like `svg2png` or text editors that accept piped input.

### Enable explicit hydrogen display on all carbon atoms
**Args:** `--inchi "InChI=1S/C2H4/c1-2-1/h1-2H" --output ethene.svg --show-all-hydrogens`
**Explanation:** The `--show-all-hydrogens` flag forces rendering of hydrogen atoms attached to carbon, which are often omitted by default for clarity in larger molecules.

### Specify explicit coordinates via auxiliary data InChI
**Args:** `--inchi "InChI=1S/C3H8/c1-2-3-1/h1-3H/m0-0-1/r1-2-3/h1-2H" --output propane.svg`
**Explanation:** When auxiliary coordinate data is embedded in the InChI string, the tool prioritizes those coordinates over computed layouts, preserving a specific structural depiction.