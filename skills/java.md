---
name: java
category: runtime
description: Java runtime and compiler; executes JAR-based bioinformatics tools (GATK, Picard, FastQC, Trimmomatic) and manages JVM memory
tags: [java, jvm, jar, gatk, picard, fastqc, trimmomatic, xmx, classpath, openjdk]
author: oxo-call built-in
source_url: "https://docs.oracle.com/en/java/index.html"
---

## Concepts
- `JAVA_HOME` env var points to the JDK/JRE root (e.g. `/usr/lib/jvm/java-17-openjdk-amd64`); `$JAVA_HOME/bin/java` is the executable.
- Check active Java: `java -version`; check JAVA_HOME: `echo $JAVA_HOME`; find all installed JDKs: `update-java-alternatives -l` (Debian/Ubuntu).
- On macOS, `$(/usr/libexec/java_home)` prints the current JAVA_HOME; `-v 17` selects version 17.
- Memory allocation: `-Xmx<N>g` sets maximum heap (e.g. `-Xmx8g` for 8 GB); `-Xms<N>g` sets initial heap; critical for GATK HaplotypeCaller and other memory-hungry tools.
- JVM garbage collector: `-XX:+UseG1GC` (G1 GC) is recommended for GATK 4 and larger heap sizes (>4 GB); default for Java 9+.
- JVM temporary directory: `-Djava.io.tmpdir=/path/to/tmpdir` redirects temp files; important on HPC where `/tmp` is small.
- Classpath (`-cp` or `-classpath`): colon-separated (Linux/macOS) or semicolon-separated (Windows) list of JAR files and directories.
- Run a JAR file: `java -jar tool.jar [args]`; the JAR must contain a `Main-Class` manifest attribute.
- Many bioinformatics tools wrap Java in shell scripts (e.g. `gatk`, `picard`, `trimmomatic`); check wrapper scripts when debugging memory issues.
- Multiple Java versions: managed via `update-alternatives` (Linux), `module load java/<version>` (HPC), or SDKMAN (`sdk use java <version>`).
- conda environments often install OpenJDK: `conda install -c conda-forge openjdk=17`; activates a self-contained JVM.
- Java 17 LTS is required by GATK 4.4+; Java 11 is required by many older bioinformatics tools.

## Pitfalls
- forgetting `-Xmx` causes the JVM to use a default heap (often 25% of RAM or 256 MB), leading to `OutOfMemoryError` in GATK and Picard on large datasets.
- `-Xmx` must be set lower than the total available RAM; leave at least 2–4 GB for the OS and other processes.
- Mismatched Java version: GATK 4 requires Java 8 or 17; using Java 11 or 21 may cause subtle errors or outright failure.
- On HPC, avoid using the system Java if the version is too old; load the correct module with `module load java/17` before running pipelines.
- `-Djava.io.tmpdir` must point to a directory with enough free space; GATK creates large temporary BAM files during variant calling.
- Thread count: GATK `--native-pair-hmm-threads` and `-nct` (older) control CPU usage; not the same as JVM thread settings.
- JAR files are not self-updating; always download the latest JAR version when upgrading tools.

## Examples

### check installed Java version
**Args:** `-version`
**Explanation:** prints the JVM version, vendor, and runtime path; use this before running tools that require a specific Java version

### run a JAR-based tool with increased heap memory
**Args:** `-Xmx16g -jar picard.jar SortSam I=input.bam O=sorted.bam SORT_ORDER=coordinate`
**Explanation:** -Xmx16g allocates up to 16 GB heap; -jar runs the Picard JAR; SortSam is the Picard tool name; crucial for large BAM files

### run GATK with custom tmp directory and GC settings
**Args:** `-Xmx8g -XX:+UseG1GC -Djava.io.tmpdir=/scratch/tmp -jar gatk.jar HaplotypeCaller -R ref.fa -I input.bam -O out.vcf`
**Explanation:** G1GC is recommended for large heaps; tmpdir on scratch avoids filling /tmp; HaplotypeCaller for germline variant calling

### run FastQC via its JAR directly
**Args:** `-Xmx2g -jar fastqc.jar --threads 4 sample.fastq.gz`
**Explanation:** FastQC is Java-based; -Xmx2g is sufficient for most samples; --threads parallelises per-file QC

### show all system properties and JVM settings
**Args:** `-XshowSettings:all -version`
**Explanation:** prints all JVM settings (memory, locale, properties) followed by the Java version; useful for auditing JAVA_HOME, heap defaults, and system properties

### list available JVM garbage collectors and tuning flags
**Args:** `-XX:+PrintFlagsFinal -version`
**Explanation:** prints all JVM flags with current values; grep output for MaxHeapSize, InitialHeapSize, or UseG1GC to inspect defaults

### run Trimmomatic via its JAR
**Args:** `-Xmx4g -jar trimmomatic.jar PE -threads 8 R1.fastq.gz R2.fastq.gz R1_trimmed.fastq.gz R1_unpaired.fastq.gz R2_trimmed.fastq.gz R2_unpaired.fastq.gz ILLUMINACLIP:adapters.fa:2:30:10`
**Explanation:** -Xmx4g for trimming; PE mode for paired-end; ILLUMINACLIP removes Illumina adapters; 8 threads for parallelism

### check available JVM memory settings
**Args:** `-XX:+PrintFlagsFinal -version 2>&1`
**Explanation:** prints all JVM flags with their current values; grep for MaxHeapSize or InitialHeapSize to see actual memory settings

### run a JAR with a custom classpath
**Args:** `-cp /path/to/lib1.jar:/path/to/lib2.jar com.example.MainClass arg1 arg2`
**Explanation:** -cp sets the classpath with colon-separated JARs (semicolons on Windows); fully qualified class name after the classpath; arguments follow
