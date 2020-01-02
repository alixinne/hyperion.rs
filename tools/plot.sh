#!/bin/bash

cd hyperionc-udp
gnuplot -e "filename='nearest.csv'" -e "set term png" plot.gpi >nearest.png
gnuplot -e "filename='linear.csv'" -e "set term png" plot.gpi >linear.png
kitty +kitten icat nearest.png
echo
kitty +kitten icat linear.png
