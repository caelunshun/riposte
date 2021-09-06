#!/bin/bash

rm src/generated.rs
echo "use crate::ui::flashing_button::FlashingButton;" > src/generated.rs
find ui -name "*.yml" | xargs -I{} duit-codegen {} --append -o src/generated.rs
