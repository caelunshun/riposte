-- The stylesheet passed to the UI.

local dume = require("dume")

local researchProgressBar = {
    backgroundColor = dume.rgb(0, 0, 0),
    borderColor = dume.rgb(30, 30, 30),
    borderRadius = 0,
    borderWidth = 1,
    progressColor = dume.rgb(108, 198, 74),
    positivePredictedProgressColor = dume.rgb(74, 119, 41),
    negativePredictedProgressColor = dume.rgb(207, 69, 32)
}

return {
    defaultTextStyle = {
        family = "Merriweather",
        size = 12,
        weight = dume.FontWeight.Normal,
        style = dume.FontStyle.Normal,
        color = dume.rgb(255, 255, 255),
    },
    windowContainer = {
        backgroundColor = dume.rgb(45, 45, 45, 192),
        borderWidth = 1,
        borderColor = dume.rgb(65, 65, 65),
        borderRadius = 0,
    },
    backgroundColor = dume.rgb(50, 50, 50, 128),
    borderWidth = 1,
    borderColor = dume.rgb(65, 65, 65),
    borderRadius = 0,
    hovered = {
        backgroundColor = dume.rgb(40, 40, 40),
    },
    pressed = {
        backgroundColor = dume.rgb(35, 35, 35),
        borderColor = dume.rgb(190, 77, 0),
    },
    researchProgressBar = researchProgressBar,
    scrollable = {
        barColor = dume.rgb(60, 60, 60),
        hoveredBarColor = dume.rgb(70, 70, 70),
        grabbedBarColor = dume.rgb(80, 80, 80),
    }
}
