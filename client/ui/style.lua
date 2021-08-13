-- The stylesheet passed to the UI.

local dume = require("dume")
local Vector = require("brinevector")

local researchProgressBar = {
    backgroundColor = dume.rgb(100, 100, 100, 150),
    borderColor = dume.rgb(0, 0, 0),
    borderRadius = 0,
    borderWidth = 1,
    progressColor = dume.rgb(108, 198, 74),
    positivePredictedProgressColor = dume.rgb(74, 119, 41),
    negativePredictedProgressColor = dume.rgb(207, 69, 32)
}

local populationProgressBar = {
    backgroundColor = dume.rgb(0, 0, 0),
    borderColor = dume.rgb(30, 30, 30),
    borderRadius = 0,
    borderWidth = 1,
    progressColor = dume.rgb(237, 155, 51),
    positivePredictedProgressColor = dume.rgb(185, 112, 0),
    negativePredictedProgressColor = dume.rgb(209, 65, 36)
}

local productionProgressBar = {
    backgroundColor = dume.rgb(0, 0, 0),
    borderColor = dume.rgb(30, 30, 30),
    borderRadius = 0,
    borderWidth = 1,
    progressColor = dume.rgb(72, 159, 223, 160),
    positivePredictedProgressColor = dume.rgb(141, 200, 232, 160),
    negativePredictedProgressColor = dume.rgb(209, 65, 36)
}

return {
    default = {
        text = {
            defaultTextStyle = {
                family = "Merriweather",
                size = 14,
                weight = dume.FontWeight.Normal,
                style = dume.FontStyle.Normal,
                color = dume.rgb(255, 255, 255),
            },
        },
        tooltipText = {
            defaultTextStyle = {
                family = "Merriweather",
                size = 15,
                weight = dume.FontWeight.Normal,
                style = dume.FontStyle.Normal,
                color = dume.rgb(255, 255, 255),
            }
        },
        windowContainer = {
            backgroundColor = dume.rgb(45, 45, 45, 254),
            borderWidth = 1,
            borderColor = dume.rgb(120, 120, 120),
            borderRadius = 0,
        },
        container = {
            backgroundColor = dume.rgb(50, 50, 50, 128),
            borderWidth = 1,
            borderColor = dume.rgb(65, 65, 65),
            borderRadius = 0,
        },
        tooltipContainer = {
            backgroundColor = dume.rgb(50, 50, 50, 254),
            borderWidth = 1,
            borderColor = dume.rgb(120, 120, 120),
            borderRadius = 0,
            minSize = Vector(300, 150),
        },
        smallTooltipContainer = {
            minSize = Vector(300, 100),
        },
        unitHeadContainer = {
            backgroundColor = dume.rgb(80, 80, 80),
            borderColor = dume.rgb(0, 0, 0),
        },
        unitHeadContainerSelected = {
            borderColor = dume.rgb(255, 205, 0),
            borderWidth = 2,
        },
        researchProgressBar = researchProgressBar,
        populationProgressBar = populationProgressBar,
        productionProgressBar = productionProgressBar,
        scrollable = {
            barColor = dume.rgb(60, 60, 60),
            hoveredBarColor = dume.rgb(70, 70, 70),
            grabbedBarColor = dume.rgb(80, 80, 80),
        },
        unitActionButton = {
            minSize = Vector(150, 50),
        },
        confirmationButton = {
            minSize = Vector(100, 20),
        },
        sliderButton = {
            minSize = Vector(20, 20),
        },
        highlightedText = {
            defaultTextStyle = {
                family = "Merriweather",
                size = 14,
                weight = dume.FontWeight.Normal,
                style = dume.FontStyle.Normal,
                color = dume.rgb(255, 191, 63),
            }
        },
        divider = {
            color = dume.rgb(180, 180, 180),
        },
        table = {
            cellBorderWidth = 2,
            cellBorderColor = dume.rgb(100, 100, 100),
            backgroundColor = dume.rgb(100, 100, 100, 100),
        }
    },
    hovered = {
        hoverableText = {
            defaultTextStyle = {
                family = "Merriweather",
                size = 14,
                weight = dume.FontWeight.Normal,
                style = dume.FontStyle.Normal,
                color = dume.rgb(255, 191, 63),
            }
        },
        button = {
            backgroundColor = dume.rgb(80, 80, 80, 128),
            borderWidth = 1,
            borderColor = dume.rgb(90, 90, 90),
            borderRadius = 0,
        },
    },
    pressed = {
        button = {
            backgroundColor = dume.rgb(90, 90, 90, 128),
            borderWidth = 1,
            borderColor = dume.rgb(110, 110, 110),
            borderRadius = 0,
        }
    },
}
