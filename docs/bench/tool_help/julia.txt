
    julia [switches] -- [programfile] [args...]

Switches (a '*' marks the default value, if applicable; settings marked '($)' may trigger package
precompilation):

 Option                                        Description
 ---------------------------------------------------------------------------------------------------
 -v, --version                                 Display version information
 -h, --help                                    Print command-line options (this message)
 --help-hidden                                 Print uncommon options not shown by `-h`

 --project[={<dir>|@temp|@.|@script[<rel>]}]   Set <dir> as the active project/environment.
                                               Or, create a temporary environment with `@temp`
                                               The default @. option will search through parent
                                               directories until a Project.toml or JuliaProject.toml
                                               file is found. @script is similar, but searches up from
                                               the programfile or a path relative to programfile.
 -J, --sysimage <file>                         Start up with the given system image file
 -H, --home <dir>                              Set location of `julia` executable
 --startup-file={yes*|no}                      Load `JULIA_DEPOT_PATH/config/startup.jl`; 
                                               if `JULIA_DEPOT_PATH` environment variable is unset,
                                               load `~/.julia/config/startup.jl`
 --handle-signals={yes*|no}                    Enable or disable Julia's default signal handlers
 --sysimage-native-code={yes*|no}              Use native code from system image if available
 --compiled-modules={yes*|no|existing|strict}  Enable or disable incremental precompilation of
                                               modules. The `existing` option allows use of existing
                                               compiled modules that were previously precompiled,
                                               but disallows creation of new precompile files.
                                               The `strict` option is similar, but will error if no
                                               precompile file is found.
 --pkgimages={yes*|no|existing}                Enable or disable usage of native code caching in the
                                               form of pkgimages. The `existing` option allows use
                                               of existing pkgimages but disallows creation of new
                                               ones ($)

 -e, --eval <expr>                             Evaluate <expr>
 -E, --print <expr>                            Evaluate <expr> and display the result
 -m, --module <Package> [args]                 Run entry point of `Package` (`@main` function) with
                                               `args'.
 -L, --load <file>                             Load <file> immediately on all processors

 -t, --threads {auto|N[,auto|M]}               Enable N[+M] threads; N threads are assigned to the
                                               `default` threadpool, and if M is specified, M
                                               threads are assigned to the `interactive`
                                               threadpool; `auto` tries to infer a useful
                                               default number of threads to use but the exact
                                               behavior might change in the future. Currently sets
                                               N to the number of CPUs assigned to this Julia
                                               process based on the OS-specific affinity assignment
                                               interface if supported (Linux and Windows) or to the
                                               number of CPU threads if not supported (MacOS) or if
                                               process affinity is not configured, and sets M to 1.
 --gcthreads=N[,M]                             Use N threads for the mark phase of GC and M (0 or 1)
                                               threads for the concurrent sweeping phase of GC.
                                               N is set to the number of compute threads and
                                               M is set to 0 if unspecified.
 -p, --procs {N|auto}                          Integer value N launches N additional local worker
                                               processes `auto` launches as many workers as the
                                               number of local CPU threads (logical cores).
 --machine-file <file>                         Run processes on hosts listed in <file>

 -i, --interactive                             Interactive mode; REPL runs and
                                               `isinteractive()` is true.
 -q, --quiet                                   Quiet startup: no banner, suppress REPL warnings
 --banner={yes|no|short|auto*}                 Enable or disable startup banner
 --color={yes|no|auto*}                        Enable or disable color text
 --history-file={yes*|no}                      Load or save history

 --depwarn={yes|no*|error}                     Enable or disable syntax and method deprecation
                                               warnings (`error` turns warnings into errors)
 --warn-overwrite={yes|no*}                    Enable or disable method overwrite warnings
 --warn-scope={yes*|no}                        Enable or disable warning for ambiguous top-level
                                               scope

 -C, --cpu-target <target>                     Limit usage of CPU features up to <target>; set to
                                               `help` to see the available options
 -O, --optimize={0|1|2*|3}                     Set the optimization level (level 3 if `-O` is used
                                               without a level) ($)
 --min-optlevel={0*|1|2|3}                     Set a lower bound on the optimization level
 -g, --debug-info=[{0|1*|2}]                   Set the level of debug info generation (level 2 if
                                               `-g` is used without a level) ($)
 --inline={yes*|no}                            Control whether inlining is permitted, including
                                               overriding @inline declarations
 --check-bounds={yes|no|auto*}                 Emit bounds checks always, never, or respect
                                               @inbounds declarations ($)
 --math-mode={ieee|user*}                      Always follow `ieee` floating point semantics or
                                               respect `@fastmath` declarations

 --code-coverage[={none*|user|all}]            Count executions of source lines (omitting setting is
                                               equivalent to `user`)
 --code-coverage=@<path>                       Count executions but only in files that fall under
                                               the given file path/directory. The `@` prefix is
                                               required to select this option. A `@` with no path
                                               will track the current directory.
 --code-coverage=tracefile.info                Append coverage information to the LCOV tracefile
                                               (filename supports format tokens)
 --track-allocation[={none*|user|all}]         Count bytes allocated by each source line (omitting
                                               setting is equivalent to `user`)
 --track-allocation=@<path>                    Count bytes but only in files that fall under the
                                               given file path/directory. The `@` prefix is required
                                               to select this option. A `@` with no path will track
                                               the current directory.
 --bug-report=KIND                             Launch a bug report session. It can be used to start
                                               a REPL, run a script, or evaluate expressions. It
                                               first tries to use BugReporting.jl installed in
                                               current environment and fallbacks to the latest
                                               compatible BugReporting.jl if not. For more
                                               information, see --bug-report=help.

 --heap-size-hint=<size>[<unit>]               Forces garbage collection if memory usage is higher
                                               than the given value. The value may be specified as a
                                               number of bytes, optionally in units of: B, K (kibibytes),
                                               M (mebibytes), G (gibibytes), T (tebibytes), or % (percentage
                                               of physical memory).

