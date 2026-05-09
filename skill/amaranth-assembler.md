---
name: amaranth-assembler
category: HDL Simulation and Build
description: A command-line tool from the Amaranth HDL ecosystem used to assemble and build hardware designs into output formats such as RTL code, simulation wrappers, or bitstream intermediates. It drives the broader build pipeline by invoking synthesis and simulation steps based on a declarative platform description.
tags:
  - amaranth
  - hdl
  - fpga
  - digital-design
  - build
  - assembler
  - rtl
  - verilog
author: AI-generated
source_url: https://github.com/amaranth-hdl/amaranth
---

## Concepts

- **Platform-driven build model.** The assembler does not infer the target from the design alone; it requires a *platform object* that declares resource constraints (clock frequencies, I/O standards, pin assignments). Without a platform, the assembler defaults to a portable simulation backend, which may not reflect real hardware timing.

- **Output format selection controls downstream tools.** The `-t` / `--type` flag selects the output format (e.g., `verilog`, `systemverilog`, `rtlil`, `firrtl`). Choosing an incompatible type for the target frontend (e.g., emitting SystemVerilog for a tool that only accepts Verilog-2005) causes silent mismatches in synthesis tools that may surface only as counter-intuitive optimization results.

- **Top-level module resolution is name-based.** The top module is identified by its Python dotted-name notation (e.g., `mydesign.cpu.CPUCore`), not by a file path. The assembler must be able to import this name from the Python path, so the design package must be installed or the PYTHONPATH must include the parent directory.

- **The `amaranth-asm` intermediate representation.** Internally, the tool constructs an abstract intermediate representation (amaranth-asm IR) that captures module hierarchies, clock domains, and memory maps before emitting target RTL. This IR is replayable via `amaranth-asm dump`, which is useful for debugging mismatches between the Python source and the generated netlist.

- **Simulation launch via the `--run` alias.** Passing `--run` to a simulation-targeted build invokes the configured simulator (defaulting to verilator if present). The flag wires directly into the subprocess call and does not return an on-disk artifact unless `--output` is also specified.

## Pitfalls

- **Forgetting to set the top-level clock frequency on the platform.** If the platform omits the `clk_freq` attribute, the assembler emits a default timebase (often 1 Hz), causing simulation timebases to be wildly wrong and any PLL or delay-chain calculation in the RTL to be nonsensical. The design will simulate correctly but fail timing in hardware.

- **Specifying a top module name that cannot be imported.** Using `top=design` when `design.py` is not on the Python path produces a confusing `ModuleNotFoundError` rather than a clear "design not on PYTHONPATH" diagnostic. This is especially easy to miss when switching working directories.

- **Passing `--output` with a format that the downstream toolchain does not support.** Emitting `rtlil` is useful for certain formal verification flows, but if the downstream `yosys` invocation uses incompatible parameters, the assembler will exit with code 0 yet produce a syntactically empty file. Always verify the output file is non-empty after a build.

- **Confusing `amaranth-asm` (the assembler) with `amaranth-sim` (the simulator).** The assembler produces static output artifacts; it does not run cycles. Attempting to inject waveforms or VCD dump flags via the assembler will be silently ignored, and the user will report "the simulation never starts" when in fact no simulation was launched.

- **Using positional arguments where a named flag is required.** The tool is strict about flag ordering. Placing the output path before the platform specifier triggers a parse error that references the flag rather than the argument position, leading the user to inspect the wrong part of the command line.

## Examples

### Build a Verilog output from a design using the default simulation platform
**Args:** `-t verilog -p default -o build/design.v design:TopModule`
**Explanation:** The `-p default` platform selects the built-in simulation adapter which has no I/O constraints, making this suitable for RTL verification before committing to a board-specific platform.

### Assemble a SystemVerilog target for an ASIC flow with an explicit top module
**Args:** `-t systemverilog -p asapdkgate45 -o out/crypto_core.sv mylib.cores.AESRound --no-indirect-wires`
**Explanation:** The ASAPDK 45nm library platform enables standard-cell-specific cells, and `--no-indirect-wires` ensures the emitted SystemVerilog does not use tri1/tri0 resolution, which some ASIC simulators reject.

### Generate an RTLIL dump for formal verification with Yosys
**Args:** `-t rtlil -o rtlil_dump.il design:ControlFSM --preserve-attributes`
**Explanation:** Emitting RTLIL preserves attribute annotations (such as `(* keep *)`) that are lost in Verilog translation, which is required for `hierarchy -check` in Yosys to validate the module boundary correctly.

### Run a simulation end-to-end via the assembler with verilator
**Args:** `--run -t verilator -p ice40hx8k -o sim_build/blink.blob blink:Blinker --trace-format vcd --opt-level 2`
**Explanation:** The `--run` flag instructs the assembler to invoke verilator after emitting Verilog, and `--trace-format vcd` produces a waveform dump for gtkwave inspection of the clock-domain crossings.

### Build with a custom platform module installed in a local package
**Args:** `-t verilog -p myplatforms.DEVBOARD_V1 -o build/top.v mydesign:DEVEL_TOP --parallel`
**Explanation:** Pointing `-p` to a dotted name inside a locally installed package (`myplatforms`) enables board-specific I/O standards and PLL settings while `--parallel` accelerates multi-module synthesis by distributing sub-module emission across available CPU cores.