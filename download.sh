#!/bin/bash

download_dir="downloads"

# Downloading required fonts
mkdir -p $download_dir/fonts/roboto
if [ ! -f "$download_dir/fonts/roboto/Roboto-Medium.ttf" ]; then 
    wget -nv -O $download_dir/fonts/roboto.zip https://fonts.google.com/download?family=Roboto
    unzip -q -o $download_dir/fonts/roboto.zip -d $download_dir/fonts/roboto
    rm $download_dir/fonts/roboto.zip
fi


# Download the pre-defined brotli dictionary
mkdir -p $download_dir/brotli
if [ ! -f "$download_dir/brotli/dictionary" ]; then
    wget -nv -O $download_dir/brotli/dictionary https://gist.githubusercontent.com/duskwuff/8a75e1b5e5a06d768336c8c7c370f0f3/raw/0a469443ceca5c1e8a2ffed48c6ecc1570750b05/dictionary.bin
fi