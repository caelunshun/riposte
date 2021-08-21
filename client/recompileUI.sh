#!/bin/bash

rm src/generated.rs
find ui -name "*.yml" | xargs -I{} duit-codegen {} --append -o src/generated.rs
