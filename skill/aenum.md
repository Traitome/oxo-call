---
name: aenum
category: Python Library (Bioinformatics Utilities)
description: A Python library that extends the built-in enum module with advanced enumeration features including extendable enumerations, method support, and flexible member manipulation for bioinformatics data models and workflow states.
tags: [python, enum, enumeration, bioinformatics, data-model, state-machine, scripting]
author: AI-generated
source_url: https://pypi.org/project/aenum/
---

## Concepts

- **Extended Enumeration Types**: `aenum` provides `Enum`, `IntEnum`, `Flag`, and `IntFlag` classes that support adding methods and properties directly to enumeration classes, enabling rich data models for bioinformatics pipeline states (e.g., `FASTQ_STATUS = Enum('FASTQ_STATUS', 'UNKNOWN UNPAIRED PAIRED QUALITY_FILTERED')`).

- **Member Extension with `.extend()`**: The library allows adding new members to existing enumerations via the `.extend()` method, which is useful when integrating external annotation sources—for example, adding new variant classification categories after a database update without redefining the entire enum.

- **Multi-flag Support with Flags**: Unlike standard enums, `aenum.Flag` supports combining multiple enum values using bitwise operators, enabling elegant representation of multi-state bioinformatic conditions (e.g., `SAMPLE_FLAGS = Flag('SAMPLE_FLAGS', 'DNA_RNA QC_PASSED BAM_AVAILABLE VCF_GENERATED')` where a sample can have multiple flags simultaneously).

## Pitfalls

- **Confusion with Standard Library Import**: Importing from the wrong module (e.g., accidentally using `from enum import Enum` instead of `from aenum import Enum`) will silently fail to provide extended features like `.extend()` or method support, causing runtime `AttributeError` when attempting to use them.

- **Pickling Enum Instances Across Processes**: Enumerations defined with `aenum` that include custom methods may fail to pickle correctly when passed between Python processes (e.g., via multiprocessing in a workflow manager), resulting in `pickle.PicklingError` or corrupted enum representations.

- **Mixin Order with IntEnum and Properties**: Defining an `IntEnum` that also inherits from other mixins (e.g., `class Base(IntEnum, CompareHelper)`) can cause metaclass conflicts or unexpected integer coercion, leading to incorrect comparisons or hash values in variant annotation lookups.

## Examples

### Creating a bioinformatics sample status enum
**Args:** `import aenum` `class SampleStatus(aenum.Enum):` `    UNKNOWN = 0` `    QUEUED = 1` `    PROCESSING = 2` `    COMPLETED = 3` `    FAILED = 4`
**Explanation:** Defines a fixed enumeration of sample processing states that can be used in workflow management systems to track bioinformatics analysis pipeline progress.

### Extending an enum with newvariant classifications
**Args:** `import aenum` `class VariantType(aenum.Enum):` `    SNP = 1` `    DEL = 2` `    INS = 3` `VariantType.extend(['MNV = 4', 'STRUCTURAL = 5', 'COPY_NUMBER = 6'])`
**Explanation:** Dynamically adds new variant type categories after initial definition, useful when updating classification schemas to accommodate novel variant types discovered in ongoing research.

### Using Flag for multi-state quality control
**Args:** `import aenum` `class QCFlag(aenum.Flag):` `    PASS = auto()` `    WARN = auto()` `    FAIL = auto()` `combined = QCFlag.PASS | QCFlag.WARN`
**Explanation:** Creates a flag-based enum where multiple QC states can be combined, allowing samples to simultaneously pass initial checks while generating warnings without creating mutually exclusive enum values.

### Adding methods to an enum for annotation lookup
**Args:** `import aenum` `class Annotation(aenum.Enum):` `    GENE = 1` `    VARIANT = 2` `    REGION = 3` `    def description(self):` `        return {Annotation.GENE: 'Genic region', Annotation.VARIANT: 'Variant position', Annotation.REGION: 'Genomic interval'}[self]`
**Explanation:** Enables direct method calls on enum members for retrieving associated metadata, streamlining annotation pipeline code by embedding descriptive strings directly in the enum.

### Converting string values to enum members safely
**Args:** `import aenum` `class AnalysisTool(aenum.Enum):` `    BOWTIE = auto()` `    BOWTIE2 = auto()` `    STAR = auto()` `    try:` `        tool = AnalysisTool(value)` `    except ValueError:` `        tool = None`
**Explanation:** Demonstrates safe conversion of external input strings to enum members with exception handling, preventing crashes when parsing bioinformatics tool names from configuration files that may contain invalid entries.

### Using IntEnum for numeric thresholds in quality filtering
**Args:** `import aenum` `class QScoreThreshold(aenum.IntEnum):` `    LOW = 20` `    MEDIUM = 30` `    HIGH = 40` `    REQUIRED = 60` `filtered = read.qual >= QScoreThreshold.MEDIUM`
**Explanation:** Provides integer-backed enumerations that support direct numeric comparisons, allowing straightforward implementation of quality score filtering logic using named threshold constants instead of magic numbers.