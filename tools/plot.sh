#!/bin/bash

gnuplot -e "filename='nearest.csv'" -e "set term png" tools/plot.gpi >nearest.png
gnuplot -e "filename='linear.csv'" -e "set term png" tools/plot.gpi >linear.png
kitty +kitten icat nearest.png
echo
kitty +kitten icat linear.png
