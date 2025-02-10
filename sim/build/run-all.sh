#!/bin/sh

work="/home/jovyan/work"
experiments=''

# Collect experiments' paths
for e in "${work}"/* ; do
    experiments="$experiments${e##*/} "
done
printf "\nFound experiments: ${experiments}\n"

# Convert noteboooks to python scripts
for e in $experiments ; do
    jupyter nbconvert "${work}/${e}/${e}.ipynb" --to script
done

# Run experiments
for e in $experiments ; do
    printf "\n=> Running %s.py\n" "$e" 
    python3 "${work}/${e}/${e}.py"
done