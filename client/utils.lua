local dume = require("dume")

function generatePastelColor(seed)
    math.randomseed(seed)
    local r = math.random(0, 255)
    local g = math.random(0, 255)
    local b = math.random(0, 255)


    return dume.rgb(r, g, b)
end

-- Converts a number to a string, rounded to
-- one decimal place. Unlike string.format, does
-- not pad with zeroes.
function tostringRounded(x)
    local s = string.format("%.1f", x)

    -- remove ending zeroes
    while string.sub(s, #s, #s) == "0" or string.sub(s, #s, #s) == "." do
        s = string.sub(s, 1, #s - 1)
    end

    return s
end

function mergeTextLines(lines)
    local text = ""
    for i, line in ipairs(lines) do
        if line ~= nil then
            text = text .. line
        end
        if i ~= #lines then
            text = text .. "\n"
        end
    end

    if #text == 0 then return nil
    else return text end
end

function getResourceDescription(resource)
    local s = resource.name

    if resource.healthBonus ~= nil and resource.healthBonus ~= 0 then
        s = s .. ", +" .. tostring(resource.healthBonus) .. "@icon{health}"
    end

    if resource.happyBonus ~= nil and resource.happyBonus ~= 0 then
        s = s .. ", +" .. tostring(resource.happyBonus) .. "@icon{happy}"
    end

    return s
end
