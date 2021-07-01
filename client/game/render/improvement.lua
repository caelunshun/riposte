local dume = require("dume")
local Vector = require("brinevector")

local function renderBasicImprovement(cv, improvementID)
    local aspectRatio = 640 / 512
    local size = Vector(30, 30 * aspectRatio)
    cv:drawSprite("icon/" .. improvementID, Vector(50, 15) - (size / 2), size.x)
end

local adjacentOffsets = {
    Vector(1, 0),
    Vector(1, 1),
    Vector(0, 1),
    Vector(-1, 1),
    Vector(-1, 0),
    Vector(-1, -1),
    Vector(0, -1),
    Vector(1, -1),
}

local function tileHasImprovement(tile, improvement)
    for _, i in ipairs(tile.improvements) do
        if i.id == improvement then return true end
    end
    return false
end

local function renderRoad(cv, tilePos, game)
    cv:strokeWidth(5)
    cv:solidColor(dume.rgb(80, 80, 80))

    -- Roads connect to other roads/cities on adjacent tiles (both straight and diagonal)
    local numConnections = 0
    for i=1,#adjacentOffsets do
        local offset = adjacentOffsets[i]
        local adjacentTilePos = tilePos + offset
        local adjacentTile = game:getTile(adjacentTilePos)

        if tileHasImprovement(adjacentTile, "Road") or game:getCityAtPos(adjacentTilePos) ~= nil then
            numConnections = numConnections + 1

            cv:beginPath()
            cv:moveTo(Vector(50, 50))
            cv:lineTo(offset * 100 + Vector(50, 50))
            cv:stroke()
        end
    end

    if numConnections == 0 then
        -- We need to render something - add a circle.
        cv:beginPath()
        cv:circle(Vector(50, 50), 20)
        cv:stroke()
    end
end

-- Renders an improvement.
return function(cv, improvementName, tilePos, game)
    if improvementName == "Cottage" then
        renderBasicImprovement(cv, "cottage")
    elseif improvementName == "Mine" then
        renderBasicImprovement(cv, "mine")
    elseif improvementName == "Farm" then
        renderBasicImprovement(cv, "farm")
    elseif improvementName == "Pasture" then
        renderBasicImprovement(cv, "pasture")
    elseif improvementName == "Road" then
        renderRoad(cv, tilePos, game)
    else
        error("unknown improvement " .. improvementName)
    end
end
