#!/bin/bash

glslc src/shaders/game.comp -o target/game.spv
cp src/web/index.html target/index.html