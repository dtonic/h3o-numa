#!/bin/bash

# Cell count table for each resolution
declare -a CELL_COUNTS=(
    122                    # res 0
    842                    # res 1
    5882                   # res 2
    41162                  # res 3
    288122                 # res 4
    2016842                # res 5
    14117882               # res 6
    98825162               # res 7
    691776122              # res 8
    4842432842             # res 9
    33897029882            # res 10
    237279209162           # res 11
    1660954464122          # res 12
    11626681248842         # res 13
    81386768741882         # res 14
    569707381193162        # res 15
)

# Check if resolution argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <resolution>"
    exit 1
fi

res=$1

# Check if resolution is valid
if [ "$res" -lt 0 ] || [ "$res" -ge "${#CELL_COUNTS[@]}" ]; then
    echo "Error: Resolution ${res} is out of range (0-$((${#CELL_COUNTS[@]}-1)))"
    exit 1
fi

# Get cell count from array
cell_count=${CELL_COUNTS[$res]}

# Calculate bytes (u64 = 8 bytes)
bytes=$((cell_count * 8))
human_bytes=$(numfmt --to=iec-i --suffix=B $bytes)

echo "-- Metadata --"
echo "Resolution: ${res}"
echo "Cell count: ${cell_count}"
echo "Bytes: ${bytes} (${human_bytes})"
echo "Option: cargo run -r --bin gen_index_stdout --features tools -- ${res} --raw | zstd -T0 -22 --zstd=overlapLog=9 --size-hint=${bytes} --ultra --long=31 -o res${res}_cells.zst"
echo "----------------"

# Run the command
cargo run -r --bin gen_index_stdout --features tools -- ${res} --raw | \
zstd -T0 -22 --zstd=overlapLog=9 --size-hint=${bytes} --ultra --long=31 -o res${res}_cells.zst
