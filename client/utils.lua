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

local function getBonusLine(effect, expectedType, icon)
    if effect.type ~= expectedType then return end

    return string.format("%%bullet +%d @icon{%s}", effect.amount, icon)
end

local function getBonusPercentLine(effect, expectedType, icon)
    if effect.type ~= expectedType then return end

    return string.format("%%bullet +%d%%percent @icon{%s}", effect.amount, icon)
end

function getBuildingTooltip(building)
    local lines = {}

    lines[#lines + 1] = building.name
    lines[#lines + 1] = string.format("%d @icon{hammer}", building.cost)

    if building.onlyForCivs ~= nil then
        local civ = registry.civs[building.onlyForCivs[1]]
        local replaces = registry.buildings[building.replaces]
        lines[#lines + 1] = "Unique Building for " .. civ.name .. " (Replaces " .. replaces.name .. ")"
    end

    for _, effect in ipairs(building.effects or {}) do
        lines[#lines + 1] = getBonusLine(effect, "bonusHammers", "hammer")
        lines[#lines + 1] = getBonusLine(effect, "bonusBeakers", "beaker")
        lines[#lines + 1] = getBonusLine(effect, "bonusCommerce", "coin")
        lines[#lines + 1] = getBonusLine(effect, "bonusFood", "bread")
        lines[#lines + 1] = getBonusLine(effect, "bonusGold", "gold")

        lines[#lines + 1] = getBonusPercentLine(effect, "bonusHammerPercent", "hammer")
        lines[#lines + 1] = getBonusPercentLine(effect, "bonusBeakerPercent", "beaker")
        lines[#lines + 1] = getBonusPercentLine(effect, "bonusCommercePercent", "coin")
        lines[#lines + 1] = getBonusPercentLine(effect, "bonusFoodPercent", "bread")
        lines[#lines + 1] = getBonusPercentLine(effect, "bonusGoldPercent", "gold")

        if effect.type == "oceanFoodBonus" then
            lines[#lines + 1] = string.format("%%bullet +%d @icon{bread} on ocean tiles", effect.amount)
        end

        if effect.type == "granaryFoodStore" then
            lines[#lines + 1] = "%bullet Retains 50%percent of stored food after growth"
        end

        if effect.type == "defenseBonusPercent" then
            lines[#lines + 1] = string.format("%%bullet +%d%%percent city defense", effect.amount)
        end

        if effect.type == "minusMaintenancePercent" then
            lines[#lines + 1] = string.format("%%bullet -%d%%percent city maintenance costs", effect.amount)
        end

        if effect.type == "happiness" then
            lines[#lines + 1] = string.format("%%bullet +%d @icon{happy}", effect.amount)
        end
        if effect.type == "health" then
            lines[#lines + 1] = string.format("%%bullet +%d @icon{healthy}", effect.amount)
        end
    end

    return lines
end
