#!/usr/bin/env python3
"""
Filter benchmark data to keep only tools that:
1. Have a corresponding skill file
2. Are accessible locally (via which command)
"""
import csv
import subprocess
from pathlib import Path

# Paths
PROJECT_ROOT = Path("/data/home/wsx/Projects/oxo/oxo-call")
BENCH_DIR = PROJECT_ROOT / "docs/bench"
SKILLS_DIR = PROJECT_ROOT / "skills"

# Get list of tools with skills
print("=== Step 1: Checking available skills ===")
skill_tools = set()
for skill_file in SKILLS_DIR.glob("*.md"):
    tool_name = skill_file.stem
    skill_tools.add(tool_name)

print(f"Found {len(skill_tools)} tools with skills")

# Special case: skill name -> actual command name mapping
COMMAND_ALIASES = {
    # R language
    "r": ["R", "Rscript"],
    # STAR aligner
    "star": ["STAR"],
    # SRA toolkit
    "sra-tools": ["fastq-dump", "prefetch", "fasterq-dump"],
    # Subread package
    "featurecounts": ["featureCounts"],
    # SnpEff
    "snpeff": ["snpEff", "snpSift"],
    # hap.py benchmark
    "hap_py": ["hap.py"],
    # BBMap suite
    "bbtools": ["bbmap", "bbmerge", "bbduk"],
    # IQ-TREE
    "iqtree2": ["iqtree", "iqtree2"],
    # Strelka2
    "strelka2": ["configureStrelkaGermlineWorkflow", "configureStrelkaSomaticWorkflow", "strelka"],
    # VarScan2
    "varscan2": ["varscan", "varscan2"],
    # BLAST
    "blast": ["blastn", "makeblastdb", "blastp", "blastx"],
    # FastANI
    "fastani": ["fastANI"],
    # FastQ Screen
    "fastq-screen": ["fastq_screen"],
    # HMMER
    "hmmer": ["hmmscan", "hmmsearch", "hmmbuild"],
    # MethylDackel
    "methyldackel": ["MethylDackel"],
    # MMseqs2
    "mmseqs2": ["mmseqs"],
    # NanoComp/NanoPlot/NanoStat
    "nanocomp": ["NanoComp"],
    "nanoplot": ["NanoPlot"],
    "nanostat": ["NanoStat"],
    # PacBio CCS
    "pbccs": ["ccs"],
    # RepeatMasker
    "repeatmasker": ["RepeatMasker"],
    # RSEM
    "rsem": ["rsem-calculate-expression", "rsem-prepare-reference"],
    # SURVIVOR
    "survivor": ["SURVIVOR"],
    # Trinity
    "trinity": ["Trinity"],
    # CrossMap
    "crossmap": ["CrossMap.py", "crossmap"],
    # DeepVariant
    "deepvariant": ["run_deepvariant", "deepvariant"],
    # EggNOG-mapper
    "eggnog-mapper": ["emapper.py", "emapper"],
    # Manta
    "manta": ["configManta.py", "runWorkflow.py"],
    # RSeQC
    "rseqc": ["read_distribution.py", "geneBody_coverage.py", "inner_distance.py"],
}

# Check which tools are accessible locally
print("\n=== Step 2: Checking locally accessible tools ===")
accessible_tools = set()
not_accessible = []

for tool in sorted(skill_tools):
    # Get list of commands to check for this tool
    commands_to_check = COMMAND_ALIASES.get(tool, [tool])
    
    found = False
    for cmd in commands_to_check:
        try:
            result = subprocess.run(
                ["which", cmd],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode == 0:
                accessible_tools.add(tool)
                print(f"✓ {tool} (via {cmd})")
                found = True
                break
        except Exception:
            pass
    
    if not found:
        not_accessible.append(tool)
        print(f"✗ {tool} (not found)")

print(f"\nAccessible tools: {len(accessible_tools)}/{len(skill_tools)}")

if not_accessible:
    print(f"\n=== Tools with skill but NOT accessible locally ({len(not_accessible)}) ===")
    for tool in sorted(not_accessible):
        print(f"  - {tool}")

# Filter reference_commands.csv
print(f"\n=== Step 3: Filtering reference_commands.csv ===")
input_file = BENCH_DIR / "reference_commands.csv"
output_file = BENCH_DIR / "reference_commands.csv.tmp"

filtered_count = 0
total_count = 0
removed_tools = set()

with open(input_file, 'r') as f_in, open(output_file, 'w', newline='') as f_out:
    reader = csv.DictReader(f_in)
    fieldnames = reader.fieldnames
    writer = csv.DictWriter(f_out, fieldnames=fieldnames)
    writer.writeheader()
    
    for row in reader:
        total_count += 1
        if row['tool'] in accessible_tools:
            writer.writerow(row)
            filtered_count += 1
        else:
            removed_tools.add(row['tool'])

# Replace original file
import shutil
shutil.move(str(output_file), str(input_file))

print(f"Kept {filtered_count}/{total_count} reference commands")
if removed_tools:
    print(f"Removed tools: {sorted(removed_tools)}")

# Filter usage_descriptions.csv
print(f"\n=== Step 4: Filtering usage_descriptions.csv ===")
input_file = BENCH_DIR / "usage_descriptions.csv"
output_file = BENCH_DIR / "usage_descriptions.csv.tmp"

filtered_count = 0
total_count = 0

with open(input_file, 'r') as f_in, open(output_file, 'w', newline='') as f_out:
    reader = csv.DictReader(f_in)
    fieldnames = reader.fieldnames
    writer = csv.DictWriter(f_out, fieldnames=fieldnames)
    writer.writeheader()
    
    for row in reader:
        total_count += 1
        if row['tool'] in accessible_tools:
            writer.writerow(row)
            filtered_count += 1

# Replace original file
shutil.move(str(output_file), str(input_file))

print(f"Kept {filtered_count}/{total_count} usage descriptions")

# Print final summary
print(f"\n=== Final Summary ===")
print(f"Total tools with skills: {len(skill_tools)}")
print(f"Locally accessible tools: {len(accessible_tools)}")
print(f"Removed tools: {len(not_accessible)}")
print(f"\n✅ Benchmark data filtered successfully!")
