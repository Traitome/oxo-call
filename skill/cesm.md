---
name: cesm
category: climate-modeling
description: The Community Earth System Model (CESM) is a fully-coupled global climate model used for simulating Earth's climate system, including atmosphere, ocean, land, and ice components. CESM is typically run through a case-building system where users configure model components, compile the code, and execute simulations for climate research.
tags: [climate, earth-system-model, coupled-model, atmosphere, ocean, land-ice, simulation, ncar]
author: AI-generated
source_url: https://www.cesm.ucar.edu/
---

## Concepts

- CESM uses a case-building workflow with `create_new_case`, `configure`, `build`, and `run` stages. Each simulation starts by creating a case with specific component settings (e.g., resolution, physics packages, forcing data).
- The model consists of multiple components: CAM (atmosphere), POP (ocean), CLM (land), CICE (ice), and a coupler that exchanges data between components. Components must be compatible in coupled configurations.
- Input data for CESM includes forcing datasets (atmospheric, oceanic, solar), initial conditions, and boundary conditions. These are typically provided through the CESM input data repository and specified in the case namelist.
- CESM output includes history files (monthly/daily averages), restart files (for job continuation), and log files. Output format is typically NetCDF for history files.
- The model uses a namelist system for configuration, with separate namelists for each component (e.g., `atm_in`, `ocn_in`, `lnd_in`) and the coupler (`cpl_in`).

## Pitfalls

- Attempting to run CESM without first successfully building the executable results in errors about missing executables or library linking failures. Always verify the build completes before submitting the run job.
- Mismatched component resolutions or incompatible physics packages cause model crashes or unrealistic results during runtime. Verify all components use compatible namelist settings.
- Insufficient walltime or memory allocation in the job script leads to the job being killed by the scheduler. Estimate resource needs based on resolution and component count.
- Missing or incorrect input data paths cause immediate startup failures. Ensure the `CESM_INPUT` environment variable points to the correct data directory.
- Using an outdated or unsupported CESM version for specific experiments leads to reproducibility issues and potential bugs. Always document the exact version used.

## Examples

### Create a new CESM case with specified resolution and components

**Args:** `--case /path/to/my_case --compset F2000climo --resolution 1.9x2.5_gx1v6`
**Explanation:** Creates a new case directory with the F2000climo component set (historical forcing) at 1.9x2.5 atmospheric and 1x1 oceanic resolution.

### Configure the case with specific model components

**Args:** `--mach cheyenne --ncdata /path/to/initdata.nc`
**Explanation:** Configures the case for the cheyenne machine architecture and specifies the initial conditions file for the model run.

### Build the CESM executable

**Args:** `--clean all`
**Explanation:** Cleans previous build artifacts and compiles the model with the configured component settings.

### Submit a CESM run job to the scheduler

**Args:** `--prereq /path/to/build_complete`
**Explanation:** Submits the run job after confirming the build stage completed successfully.

### Run the model for multiple segments with restart capabilities

**Args:** `--resubmit 5`
**Explanation:** Configures the case to automatically resubmit and run for 5 segments, using restart files to continue from previous stops.

### Set up a case with specific atmospheric physics configuration

**Args:** `--ninst 4`
**Explanation:** Configures the case to run with 4 instances of the atmospheric component for ensemble simulation purposes.

### Modify the model namelist for output frequency

**Args:** `--namelist write_nl=atm,' fincl2 = \"TREFHTMN:TREFHTMX\" '`
**Explanation:** Modifies the atmospheric namelist to include additional variables in the history output file.