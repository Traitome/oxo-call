---
name: bgreat
category:基因组读数处理工具
description: 用于高通量测序数据的高效基因组读数处理和分析的CLI工具，支持多种输入格式和过滤器。
tags: [bioinformatics, genomics, NGS, sequence-analysis, read-processing]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bgreat
---

## Concepts

- **输入格式**: 支持FASTQ、FASTA和BAM/SAM格式的原始测序读数，可通过标准输入或文件指定读取数据。
- **数据模型**: 将每条读数表示为包含序列、碱基质量分数和可选元数据的对象，支持FASTA的简单序列和FASTQ的质量信息。
- **过滤参数**: 通过质量阈值（-q/-Q）、长度范围（--minlen/--maxlen）和序列模式匹配（--match/--reject）筛选读数。
- **输出控制**: 默认输出到标准输出，支持压缩输出（-z）和多种格式转换（-f fastq|fasta|sam）。

## Pitfalls

- **缺少输入文件时读取标准输入**: 如果未指定输入文件，bgreat 会尝试从标准输入读取，导致脚本挂起等待输入而不报错。
- **质量分数编码不匹配**: 默认为Phred+33质量编码，对使用Phred+64（旧版Illumina）的旧数据会导致质量分数解析错误，需要使用 --phred64 标志。
- **输出覆盖原文件**: 使用重定向（>）输出到输入文件所在的同一路径会清空原文件内容，导致数据丢失，应该先输出到临时文件再移动。
- **正则表达式匹配模式错误**: 使用 --match 参数时未转义特殊字符（如 .、*、?）会导致意外匹配，句点会匹配任意字符而非字面句点。

## Examples

### 过滤低质量读数并保存到新文件
**Args:** -q 20 input.fastq -o filtered.fastq
**Explanation:** 质量分数低于20的读数被移除，只保留高质量序列到指定输出文件。

### 从FASTA文件提取特定序列模式
**Args:** --match "^ATGC" input.fasta -o motif_matches.fasta
**Explanation:** 使用正则表达式匹配以ATGC开头的序列并输出，常用于提取特定基因区域。

### 转换FASTQ为FASTA格式
**Args:** -f fasta input.fastq -o output.fasta
**Explanation:** 将带质量分数的FASTQ转换为只含序列的FASTA格式用于不需要质量信息的下游分析。

### 压缩输出到gzipped文件
**Args:** -z input.fastq -o output.fastq.gz
**Explanation:** 输出自动压缩为gzip格式减少存储空间，适合大规模数据集的归档。

### 过滤最短读数后进行质量裁剪
**Args:** --minlen 50 -q 25 input.fastq | bgreat -q 25
**Explanation:** 两步处理：先过滤短于50bp的读数，再对保留的读数裁剪低质量末端，注意管道连接的累积效果。

### 使用Phred+64编码处理旧 Illumina 数据
**Args:** --phred64 input.fastq -o clean_output.fastq
**Explanation:** 指定使用Phred+64质量编码正确解析旧版Illumina测序的质量分数，避免质量问题。

### 统计输入文件中的读数数量
**Args:** input.fastq | wc -l
**Explanation:** 通过管道将bgreat输出传递给wc统计读数行数，用于快速检查数据集规模。