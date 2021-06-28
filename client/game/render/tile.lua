-- Renders base terrain (grassland, desert, etc.)
local TerrainRenderer = {}

local dume = require("dume")
local Vector = require("brinevector")

function TerrainRenderer:renderTile(cv, tile)
    local spriteName = nil
    local terrain = tile.terrain
    if terrain == "Grassland" then
        spriteName = "texture/tile/grassland"
    elseif terrain == "Plains" then
        spriteName = "texture/tile/plains"
    elseif terrain == "Ocean" then
        spriteName = "texture/tile/ocean"
    elseif terrain == "Desert" then
        spriteName = "texture/tile/desert"
    end

    if tile.hilled then spriteName = spriteName .. "/hill" end

    cv:drawSprite(spriteName, Vector(0, 0), 100)
end

-- Renders an overlay grid to show tile boundaries.
local GridOverlayRenderer = {}

local gridColor = dume.rgb(80, 80, 80, 200)
function GridOverlayRenderer:renderTile(cv)
    cv:beginPath()
    cv:rect(Vector(0, 0), Vector(100, 100))
    cv:strokeWidth(0.5)
    cv:solidColor(gridColor)
    cv:stroke()
end

-- Renders trees over terrain
local TreeRenderer = {
    treeSpriteSize = Vector(0, 0)
}

cv:getSpriteSize("icon/tree", TreeRenderer.treeSpriteSize)

function TreeRenderer:renderTile(cv, tile, tilePos, game)
    if not tile.forested then return end

    local seed = tilePos.x + tilePos.y * game.mapWidth
    math.randomseed(seed)

    local numTrees = math.random(10, 20)
    for _=0,numTrees do
        local scaleX = (math.random() + 1) * 25
        local scaleY = scaleX * self.treeSpriteSize.y / self.treeSpriteSize.x
        local treePos =  Vector(math.random() * 100, math.random() * 100)
        treePos = treePos - Vector(scaleX, scaleY) / 2
        cv:drawSprite("icon/tree", treePos, scaleX)
    end
end

-- Renders units.
local UnitRenderer = {
    allowFog = false
}

function UnitRenderer:renderTile(cv, tile, tilePos, game)
    -- Render the unit at the top of the stack,
    -- which corresponds to the strongest unit.
    local unit = game:getStackAtPos(tilePos).units[1]
    if unit ~= nil then
        unit:render(cv, game)
    end
end

-- Renders cities.
local CityRenderer = {}

function CityRenderer:renderTile(cv, tile, tilePos, game)
    local city = game:getCityAtPos(tilePos)
    if city ~= nil then
        city:render(cv)
    end
end

-- Renders icons to indicate tile yields.
local YieldRenderer = {}

function YieldRenderer:renderTile(cv, tile)
    local scale = 15
    if tile.isWorked then scale = 25 end

    local icons = {}
    local cursor = 0
    local spacing = 6
    local bigSpacing = 20

    for i=1,tile.yield.food do
        icons[#icons + 1] = { pos = cursor, sprite = "icon/bread" }
        cursor = cursor + spacing
    end
    if tile.yield.hammers ~= 0 then cursor = cursor + bigSpacing end
    for i=1,tile.yield.hammers do
        icons[#icons + 1] = { pos = cursor, sprite = "icon/hammer" }
        cursor = cursor + spacing
    end
    if tile.yield.commerce ~= 0 then cursor = cursor + bigSpacing end
    for i=1,tile.yield.commerce do
        icons[#icons + 1] = { pos = cursor, sprite = "icon/coin" }
        cursor = cursor + spacing
    end

    local length = 0
    if #icons > 0 then length = icons[#icons].pos + scale end

    for i=1,#icons do
        local icon = icons[i]
        local posX = icon.pos + (50 - length / 2)
        cv:drawSprite(icon.sprite, Vector(posX, 50 - scale / 2), scale)
    end
end

-- Adds a fog layer on top of tiles that are fogged.
local FogRenderer = {}
local fogColor = dume.rgb(50, 50, 50, 150)

function FogRenderer:renderTile(cv, tile, tilePos, game, visibility)
    if visibility == "Fogged" then
        cv:beginPath()
        cv:rect(Vector(0, 0), Vector(100, 100))
        cv:solidColor(fogColor)
        cv:fill()
    end
end

-- Responsible for rendering tiles on the map.
local TileRenderer = {
    renderers = {
        -- NB: order determines layering
        TerrainRenderer,
        GridOverlayRenderer,
        TreeRenderer,
        CityRenderer,
        YieldRenderer,
        UnitRenderer,
        FogRenderer,
    }
}

function TileRenderer:render(cv, game)
    if game.tiles == nil or game.mapWidth == nil or game.mapHeight == nil then return end

    local view = game.view

    local renderTiles = {}
    local renderPos = {}
    local renderTilePos = {}
    local count = 1

    -- Render all tiles visible on the map.
    local firstVisibleTilePos = view:getTilePosForScreenOffset(Vector(0, 0))
    local lastVisibleTilePos = view:getTilePosForScreenOffset(view.size)

    firstVisibleTilePos.x = math.clamp(firstVisibleTilePos.x, 0, game.mapWidth - 1)
    firstVisibleTilePos.y = math.clamp(firstVisibleTilePos.y, 0, game.mapHeight - 1)
    lastVisibleTilePos.x = math.clamp(lastVisibleTilePos.x, 0, game.mapWidth - 1)
    lastVisibleTilePos.y = math.clamp(lastVisibleTilePos.y, 0, game.mapHeight - 1)

    for x=firstVisibleTilePos.x,lastVisibleTilePos.x do
        for y=firstVisibleTilePos.y,lastVisibleTilePos.y do
            local tilePos = Vector(x, y)

            if game.cheatMode or game:getVisibility(tilePos) ~= "Hidden" then
                local pos = view:getScreenOffsetForTilePos(tilePos)

                local tile = game:getTile(tilePos)
                renderTiles[count] = tile
                renderPos[count] = pos * view.zoomFactor
                renderTilePos[count] = tilePos
                count = count + 1
            end
        end
    end

    game.view:applyZoom(cv)
    for _, renderer in ipairs(self.renderers) do
        for i=1,count-1 do
            local pos = renderPos[i]
            cv:translate(pos)
            local visibility = game:getVisibility(renderTilePos[i])
            if renderer.allowFog == nil or (renderer.allowFog or visibility ~= "Fogged") then
                renderer:renderTile(cv, renderTiles[i], renderTilePos[i], game, visibility)
            end
            cv:translate(-pos)
        end
    end
    cv:resetTransform()
end

return TileRenderer
