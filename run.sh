#!/bin/bash

set -e
set -x

ONE_GB=1073741824
TWO_GB=2147483648

# Copying GC, old scheduler, 1 GiB max hp
cargo run -- --gc-strategy copying --scheduler old --max-hp-for-gc $ONE_GB > copying_old_1gb
gnuplot plot.gnuplot > copying_old_1gb.png

# Copying GC, old scheduler, 2 GiB max hp
cargo run -- --gc-strategy copying --scheduler old --max-hp-for-gc $TWO_GB > copying_old_2gb
gnuplot plot.gnuplot > copying_old_2gb.png

# Copying GC, new scheduler
cargo run -- --gc-strategy copying --scheduler new > copying_new
gnuplot plot.gnuplot > copying_new.png

# Compacting GC, old scheduler, 1 GiB max hp
cargo run -- --gc-strategy mark-compact --scheduler old --max-hp-for-gc $ONE_GB > compacting_old_1gb
gnuplot plot.gnuplot > compacting_old_1gb.png

# Compacting GC, old scheduler, 2 GiB max hp
cargo run -- --gc-strategy mark-compact --scheduler old --max-hp-for-gc $TWO_GB > compacting_old_2gb
gnuplot plot.gnuplot > compacting_old_2gb.png

# Compacting GC, new scheduler
cargo run -- --gc-strategy mark-compact --scheduler new > compacting_new
gnuplot plot.gnuplot > compacting_new.png
