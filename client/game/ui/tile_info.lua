local function getHeader(tile)
    local header = tile.terrain
    if tile.hilled then
        header = header .. ", Hills"
    end
    if tile.forested then
        header = header .. ", Forest"
    end
    return header
end

local function getDefenseBonus(tile)
    local bonus = 0
    if tile.hilled then bonus = bonus + 25 end
    if tile.forested then bonus = bonus + 50 end
    if bonus > 0 then
        return "Defense bonus: +" .. tostring(bonus) .. "%percent"
    else
        return nil
    end
end

local function getYieldDescription(yield)
    local parts = {}
    if yield.food ~= nil and yield.food > 0 then
        parts[#parts + 1] = tostring(yield.food) .. "@icon{bread}"
    end
    if yield.hammers ~= nil and yield.hammers > 0 then
        parts[#parts + 1] = tostring(yield.hammers) .. "@icon{hammer}"
    end
    if yield.commerce ~= nil and yield.commerce > 0 then
        parts[#parts + 1] = tostring(yield.commerce) .. "@icon{coin}"
    end

    local s = ""
    for i, part in ipairs(parts) do
        s = s .. part
        if i ~= #parts then
            s = s .. ", "
        end
    end

    return s
end

local function getYield(tile)
    return getYieldDescription(tile.yield)
end

local function getResource(game, tile)
    if tile.resourceID ~= nil and #tile.resourceID > 0 then
        local resource = registry.resources[tile.resourceID]

        if not game.thePlayer:isTechUnlocked(resource.revealedBy) then
            return
        end

        local s = getResourceDescription(resource)

        local hasImprovement = false
        for _, improvement in ipairs(tile.improvements) do
            if improvement.id == resource.improvement then
                hasImprovement = true
                break
            end
        end

        s = s .. ", " .. getYieldDescription(resource.improvedBonus)

        if not hasImprovement then
            s = s .. " (@color{rgb(200,30,60)}{Requires " .. resource.improvement .. "})"
        end

        return s
    else
        return nil
    end
end

local function getImprovement(tile)
    local s = ""
    for i, improvement in ipairs(tile.improvements) do
        s = s .. improvement.id
        if i ~= #tile.improvements then
            s = s .. "\n"
        end
    end
    if #s == 0 then
        return nil
    else
        return s
    end
end

-- Gets info text (in Dume markup format) for the given tile.
return function(tile, game)
    local lines = {}

    lines[1] = getHeader(tile)
    lines[#lines + 1] = getDefenseBonus(tile)
    lines[#lines + 1] = getImprovement(tile)
    lines[#lines + 1] = getYield(tile)
    lines[#lines + 1] = getResource(game, tile)

    return mergeTextLines(lines)
end