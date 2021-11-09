set terminal png notransparent rounded giant font "JetBrains Mono" 24 \
  size 1200,960 

set xtics nomirror
set ytics nomirror

set style line 80 lt 0 lc rgb "#808080"

set border 3 back ls 80 

set style line 81 lt 0 lc rgb "#808080" lw 0.5

set grid xtics
set grid ytics
set grid mxtics
set grid mytics

set grid back ls 81

set style line 1 lt 1 lc rgb "#A00000" lw 2 pt 7 ps 1.5
set style line 2 lt 1 lc rgb "#00A000" lw 2 pt 11 ps 1.5
set style line 3 lt 1 lc rgb "#5060D0" lw 2 pt 9 ps 1.5
set style line 4 lt 1 lc rgb "#0000A0" lw 2 pt 8 ps 1.5
set style line 5 lt 1 lc rgb "#D0D000" lw 2 pt 13 ps 1.5
set style line 6 lt 1 lc rgb "#00D0D0" lw 2 pt 12 ps 1.5
set style line 7 lt 1 lc rgb "#B200B2" lw 2 pt 5 ps 1.5

set datafile separator ','

set xlabel "call"
set ylabel "bytes"

set xrange [0:83000]

plot "hp.csv" using 0:1 with linespoints title "HP (canister allocs)", \
     "high_water.csv" using 0:1 with linespoints title "Wasm memory (runtime allocs)"
