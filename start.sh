#!/bin/bash

# -------------------------------------
# Sample Interval (in microseconds). In general, the lower the value, the less the risk of crashing the system, but the higher the CPU usage. Recommended value is 50000.
# Interval vzorkování (v mikrosekundách). Obecně platí, že čím nižší hodnota, tím menší riziko pádu systému, ale vyšší využití CPU. Doporučená hodnota je 50000.
# -------------------------------------
sample_interval=50000

# -------------------------------------
# Manual Points / Manuální body
# You can specify the points for the manual strategy here in the object format.
# The object keys are frequencies and values are arrays of manual point objects.
# -------------------------------------
core_0_manual_points='{"3500": [{"point": 0, "value": 5}, {"point": 10, "value": 10}, {"point": 20, "value": 15}], "3000": [{"point": 0, "value": 4}, {"point": 10, "value": 8}, {"point": 20, "value": 12}], "2500": [{"point": 0, "value": 3}, {"point": 10, "value": 6}, {"point": 20, "value": 9}], "2000": [{"point": 0, "value": 2}, {"point": 10, "value": 4}, {"point": 20, "value": 6}], "1500": [{"point": 0, "value": 1}, {"point": 10, "value": 2}, {"point": 20, "value": 3}], "1000": [{"point": 0, "value": 0}, {"point": 10, "value": 1}, {"point": 20, "value": 2}], "500":  [{"point": 0, "value": 0}, {"point": 10, "value": 0}, {"point": 20, "value": 0}]}'
core_1_manual_points='{"3500": [{"point": 0, "value": 6}, {"point": 10, "value": 11}, {"point": 20, "value": 16}], "3000": [{"point": 0, "value": 5}, {"point": 10, "value": 9}, {"point": 20, "value": 13}], "2500": [{"point": 0, "value": 4}, {"point": 10, "value": 7}, {"point": 20, "value": 10}], "2000": [{"point": 0, "value": 3}, {"point": 10, "value": 5}, {"point": 20, "value": 7}], "1500": [{"point": 0, "value": 2}, {"point": 10, "value": 3}, {"point": 20, "value": 4}], "1000": [{"point": 0, "value": 1}, {"point": 10, "value": 2}, {"point": 20, "value": 3}], "500":  [{"point": 0, "value": 0}, {"point": 10, "value": 0}, {"point": 20, "value": 0}]}'
core_2_manual_points='{"3500": [{"point": 0, "value": 7}, {"point": 10, "value": 12}, {"point": 20, "value": 17}], "3000": [{"point": 0, "value": 6}, {"point": 10, "value": 10}, {"point": 20, "value": 14}], "2500": [{"point": 0, "value": 5}, {"point": 10, "value": 8}, {"point": 20, "value": 11}], "2000": [{"point": 0, "value": 4}, {"point": 10, "value": 6}, {"point": 20, "value": 8}], "1500": [{"point": 0, "value": 3}, {"point": 10, "value": 4}, {"point": 20, "value": 5}], "1000": [{"point": 0, "value": 2}, {"point": 10, "value": 3}, {"point": 20, "value": 4}], "500":  [{"point": 0, "value": 1}, {"point": 10, "value": 1}, {"point": 20, "value": 1}]}'
core_3_manual_points='{"3500": [{"point": 0, "value": 8}, {"point": 10, "value": 13}, {"point": 20, "value": 18}], "3000": [{"point": 0, "value": 7}, {"point": 10, "value": 11}, {"point": 20, "value": 15}], "2500": [{"point": 0, "value": 6}, {"point": 10, "value": 9}, {"point": 20, "value": 12}], "2000": [{"point": 0, "value": 5}, {"point": 10, "value": 7}, {"point": 20, "value": 9}], "1500": [{"point": 0, "value": 4}, {"point": 10, "value": 5}, {"point": 20, "value": 6}], "1000": [{"point": 0, "value": 3}, {"point": 10, "value": 4}, {"point": 20, "value": 5}], "500":  [{"point": 0, "value": 2}, {"point": 10, "value": 2}, {"point": 20, "value": 2}]}'

# -------------------------------------
# Check if libryzenadj.so is in /usr/lib or standard system paths
# -------------------------------------
if ! ldconfig -p | grep -q libryzenadj.so; then
    export LD_LIBRARY_PATH=./lib/:$LD_LIBRARY_PATH
fi

# Run the application / Spuštění aplikace
# Make sure you're refering to the correct path of the application. /
# Ujistěte se, že se odkazujete na správnou cestu aplikace.
./gymdeck2 "$sample_interval" \
    "$core_0_manual_points" "$core_1_manual_points" "$core_2_manual_points" "$core_3_manual_points"
