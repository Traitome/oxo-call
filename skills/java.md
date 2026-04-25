---
name: java
category: programming
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
- Alternative GCs: `-XX:+UseParallelGC` for throughput, `-XX:+UseZGC` for ultra-low latency (Java 11+), `-XX:+UseShenandoahGC` for low pause times.
- JVM temporary directory: `-Djava.io.tmpdir=/path/to/tmpdir` redirects temp files; important on HPC where `/tmp` is small.
- Classpath (`-cp` or `-classpath`): colon-separated (Linux/macOS) or semicolon-separated (Windows) list of JAR files and directories.
- Run a JAR file: `java -jar tool.jar [args]`; the JAR must contain a `Main-Class` manifest attribute.
- Many bioinformatics tools wrap Java in shell scripts (e.g. `gatk`, `picard`, `trimmomatic`); check wrapper scripts when debugging memory issues.
- Multiple Java versions: managed via `update-alternatives` (Linux), `module load java/<version>` (HPC), or SDKMAN (`sdk use java <version>`).
- conda environments often install OpenJDK: `conda install -c conda-forge openjdk=17`; activates a self-contained JVM.
- Java 17 LTS is required by GATK 4.4+; Java 11 is required by many older bioinformatics tools.
- GC tuning: `-XX:MaxGCPauseMillis=200` sets target pause time for G1GC; `-XX:ParallelGCThreads=N` controls GC thread count.
- System properties: `-D<name>=<value>` sets JVM system properties; `-XshowSettings:all` displays all JVM settings.

## Pitfalls
- forgetting `-Xmx` causes the JVM to use a default heap (often 25% of RAM or 256 MB), leading to `OutOfMemoryError` in GATK and Picard on large datasets.
- `-Xmx` must be set lower than the total available RAM; leave at least 2–4 GB for the OS and other processes.
- Mismatched Java version: GATK 4 requires Java 8 or 17; using Java 11 or 21 may cause subtle errors or outright failure.
- On HPC, avoid using the system Java if the version is too old; load the correct module with `module load java/17` before running pipelines.
- `-Djava.io.tmpdir` must point to a directory with enough free space; GATK creates large temporary BAM files during variant calling.
- Thread count: GATK `--native-pair-hmm-threads` and `-nct` (older) control CPU usage; not the same as JVM thread settings.
- JAR files are not self-updating; always download the latest JAR version when upgrading tools.
- Setting `-Xms` = `-Xmx` in production avoids runtime heap resizing overhead but uses more memory upfront.
- ZGC and ShenandoahGC require Java 11+; they provide ultra-low pause times but may have different performance characteristics than G1GC.
- `-XX:+UseG1GC` with `-XX:MaxGCPauseMillis=200` helps reduce GC pauses for interactive applications.
- Java module system (Java 9+): use `--add-modules` or `--module-path` for modular JARs; most bioinformatics tools still use classpath.
- Container awareness: `-XX:+UseContainerSupport` (default in Java 10+) respects Docker memory limits; critical for containerized workflows.

## Examples

### check installed Java version
**Args:** `-version`
**Explanation:** java command; -version prints JVM version, vendor, and runtime path

### run a JAR-based tool with increased heap memory
**Args:** `-Xmx16g -jar picard.jar SortSam I=input.bam O=sorted.bam SORT_ORDER=coordinate`
**Explanation:** java command; -Xmx16g allocates 16 GB heap; -jar picard.jar runs JAR; SortSam Picard tool; I=input.bam input BAM; O=sorted.bam output BAM; SORT_ORDER=coordinate

### run GATK with custom tmp directory and GC settings
**Args:** `-Xmx8g -XX:+UseG1GC -Djava.io.tmpdir=/scratch/tmp -jar gatk.jar HaplotypeCaller -R ref.fa -I input.bam -O out.vcf`
**Explanation:** java command; -Xmx8g allocates 8 GB heap; -XX:+UseG1GC G1 garbage collector; -Djava.io.tmpdir=/scratch/tmp temp directory; -jar gatk.jar runs JAR; HaplotypeCaller GATK tool; -R ref.fa reference; -I input.bam input BAM; -O out.vcf output VCF

### run FastQC via its JAR directly
**Args:** `-Xmx2g -jar fastqc.jar --threads 4 sample.fastq.gz`
**Explanation:** java command; -Xmx2g allocates 2 GB heap; -jar fastqc.jar runs JAR; --threads 4 parallel threads; sample.fastq.gz input FASTQ

### show all system properties and JVM settings
**Args:** `-XshowSettings:all -version`
**Explanation:** java command; -XshowSettings:all displays all JVM settings; -version prints Java version

### list available JVM garbage collectors and tuning flags
**Args:** `-XX:+PrintFlagsFinal -version`
**Explanation:** java command; -XX:+PrintFlagsFinal prints all JVM flags; -version prints Java version

### run Trimmomatic via its JAR
**Args:** `-Xmx4g -jar trimmomatic.jar PE -threads 8 R1.fastq.gz R2.fastq.gz R1_trimmed.fastq.gz R1_unpaired.fastq.gz R2_trimmed.fastq.gz R2_unpaired.fastq.gz ILLUMINACLIP:adapters.fa:2:30:10`
**Explanation:** java command; -Xmx4g allocates 4 GB heap; -jar trimmomatic.jar runs JAR; PE paired-end mode; -threads 8 parallel threads; R1.fastq.gz R2.fastq.gz inputs; R1_trimmed.fastq.gz R1_unpaired.fastq.gz R2_trimmed.fastq.gz R2_unpaired.fastq.gz outputs; ILLUMINACLIP:adapters.fa:2:30:10 adapter trimming

### check available JVM memory settings
**Args:** `-XX:+PrintFlagsFinal -version 2>&1`
**Explanation:** java command; -XX:+PrintFlagsFinal prints all JVM flags; -version prints Java version; 2>&1 redirects stderr to stdout

### run a JAR with a custom classpath
**Args:** `-cp /path/to/lib1.jar:/path/to/lib2.jar com.example.MainClass arg1 arg2`
**Explanation:** java command; -cp sets classpath; /path/to/lib1.jar:/path/to/lib2.jar colon-separated JARs; com.example.MainClass fully qualified class name; arg1 arg2 arguments

### run Java with ZGC for ultra-low latency
**Args:** `-Xmx32g -XX:+UseZGC -jar gatk.jar HaplotypeCaller -R ref.fa -I input.bam -O out.vcf`
**Explanation:** java command; -Xmx32g allocates 32 GB heap; -XX:+UseZGC Z garbage collector; -jar gatk.jar runs JAR; HaplotypeCaller GATK tool; -R ref.fa reference; -I input.bam input BAM; -O out.vcf output VCF

### set equal initial and maximum heap for production
**Args:** `-Xms16g -Xmx16g -XX:+UseG1GC -jar picard.jar MarkDuplicates I=input.bam O=marked.bam M=metrics.txt`
**Explanation:** java command; -Xms16g initial heap; -Xmx16g maximum heap; -XX:+UseG1GC G1 garbage collector; -jar picard.jar runs JAR; MarkDuplicates Picard tool; I=input.bam input BAM; O=marked.bam output BAM; M=metrics.txt metrics file

### run Java with GC logging enabled
**Args:** `-Xmx8g -Xlog:gc*:file=gc.log:time,uptime,level,tags -jar tool.jar`
**Explanation:** java command; -Xmx8g allocates 8 GB heap; -Xlog:gc*:file=gc.log:time,uptime,level,tags GC logging; -jar tool.jar runs JAR

### run Java in a container with memory limits
**Args:** `-XX:+UseContainerSupport -Xmx4g -jar tool.jar`
**Explanation:** java command; -XX:+UseContainerSupport respects Docker memory limits; -Xmx4g allocates 4 GB heap; -jar tool.jar runs JAR

### run Java with system property for configuration
**Args:** `-Xmx8g -Dconfig.file=/path/to/config.properties -jar tool.jar`
**Explanation:** java command; -Xmx8g allocates 8 GB heap; -Dconfig.file=/path/to/config.properties system property; -jar tool.jar runs JAR

### run Java with parallel GC for throughput
**Args:** `-Xmx16g -XX:+UseParallelGC -XX:ParallelGCThreads=8 -jar tool.jar`
**Explanation:** java command; -Xmx16g allocates 16 GB heap; -XX:+UseParallelGC parallel garbage collector; -XX:ParallelGCThreads=8 GC threads; -jar tool.jar runs JAR
