#!/bin/bash

set -e
set -x

ONE_GB=1073741824
TWO_GB=2147483648

# Copying GC, old scheduler, 1 GiB max hp
cargo run -- --gc-strategy copying --scheduler old --max-hp-for-gc $ONE_GB > copying_old_1gb
gnuplot plot.gnuplot > copying_old_1gb.svg

# Copying GC, old scheduler, 2 GiB max hp
cargo run -- --gc-strategy copying --scheduler old --max-hp-for-gc $TWO_GB > copying_old_2gb
gnuplot plot.gnuplot > copying_old_2gb.svg

# Copying GC, new scheduler
cargo run -- --gc-strategy copying --scheduler new > copying_old_new
gnuplot plot.gnuplot > copying_new.svg

# Compacting GC, old scheduler, 1 GiB max hp
cargo run -- --gc-strategy mark-compact --scheduler old --max-hp-for-gc $ONE_GB > compacting_old_1gb
gnuplot plot.gnuplot > compacting_old_1gb.svg

# Compacting GC, old scheduler, 2 GiB max hp
cargo run -- --gc-strategy mark-compact --scheduler old --max-hp-for-gc $TWO_GB > compacting_old_2gb
gnuplot plot.gnuplot > compacting_old_2gb.svg

# Compacting GC, new scheduler
cargo run -- --gc-strategy mark-compact --scheduler new > compacting_old_new
gnuplot plot.gnuplot > compacting_new.svg