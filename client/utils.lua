local dume = require("dume")

function generatePastelColor(seed)
    math.randomseed(seed)
    local r = math.random(0, 255)
    local g = math.random(0, 255)
    local b = math.random(0, 255)


    return dume.rgb(r, g, b)
end
