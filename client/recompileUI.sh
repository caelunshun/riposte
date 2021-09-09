#!/bin/bash

rm src/generated.rs
echo "use crate::ui::flashing_button::FlashingButton; use crate::ui::turn_indicator::TurnIndicatorCircle;" > src/generated.rs
find ui -name "*.yml" | xargs -I{} duit-codegen {} --append -o src/generated.rs
