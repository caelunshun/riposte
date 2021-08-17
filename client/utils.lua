local dume = require("dume")

function generatePastelColor(seed)
    math.randomseed(seed)
    local r = math.random(0, 255)
    local g = math.random(0, 255)
    local b = math.random(0, 255)


    return dume.rgb(r, g, b)
end

function mergeTextLines(lines)
    local text = ""
    for _, line in ipairs(lines) do
        if line ~= nil then
            text = text .. line .. "\n"
        end
    end
    return text
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
