# CLI Clap
cli-about = Comic book, Manga and Comic compressor to CBZ
cli-help-input = Path to the single file or directory containing the archives to process
cli-help-lang = Interface language (e.g., "fr", "en"). Default is "fr"
cli-help-quality = Quality level for WebP encoding (1 to 100). A value of 90 offers an excellent weight/quality ratio
cli-help-height = Target height in pixels for resizing pages
cli-help-dim = Maximum safety dimension used if ratio cannot be determined
cli-help-mode = Output file management mode (suffix, rename, replace)
cli-help-threads = Maximum number of threads (0 for auto)
cli-help-glob = Use a Glob search pattern to filter files (e.g., "**/Batman*.cbr")
cli-help-savings = Minimum weight savings percentage required to validate original file replacement
cli-help-log = Path to the log file
cli-help-verbose = Display more information in the console during execution
cli-help-skip = Disables image compression: only converts the container to CBZ format
cli-help-force = Forces image re-encoding and resizing even if they are already in WebP format

# UI Labels for progress bar
cli-label-files = files
cli-label-elapsed = Elapsed time
cli-label-remaining = Remaining

# UI Messages & Logs
msg-log-start = 🚀 Starting RustyLibraryShrinker
msg-no-files-found = No files found.
msg-start-processing = RustyLibraryShrinker: { $count } file(s) to process
msg-image-skipped = IMAGE SKIPPED
msg-reason = Reason
msg-reason-decode = Failed to decode image (file might be corrupted)
msg-log-no-gain = insufficient gain
msg-processing-complete = Processing complete!

# Final report & Statistics
msg-detailed-results = DETAILED RESULTS PER FILE
msg-skipped-no-gain = no gain
msg-global-summary = GLOBAL SUMMARY
msg-summary-total-files = Total files
msg-optimized = Optimized
msg-not-optimized = Not optimized
msg-failed = Failed
msg-summary-img-optimized = Images optimized
msg-summary-img-skipped = Images skipped
msg-original-size = Original size
msg-final-size = Final size
msg-total-gain = Total gain