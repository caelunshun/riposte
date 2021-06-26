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
local UnitRenderer = {}

function UnitRenderer:renderTile(cv, tile, tilePos, game)
    local unit = game:getUnitsAtPos(tilePos)[1]
    if unit ~= nil then
        unit:render(cv, game)
    end
end

-- Responsible for rendering tiles on the map.
local TileRenderer = {
    renderers = {
        -- NB: order determines layering
        TerrainRenderer,
        GridOverlayRenderer,
        TreeRenderer,
        UnitRenderer,
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
    for x=0,game.mapWidth - 1 do
        for y=0,game.mapHeight - 1 do
            -- Simple 2D frustum cull.
            local tilePos = Vector(x, y)
            local pos = view:getScreenOffsetForTilePos(tilePos)

            local halfWindowSize = view.size / 2
            local centered = pos - halfWindowSize
            centered = centered * view.zoomFactor

            if not (centered.x < -halfWindowSize.x - (100 * view.zoomFactor)
                or centered.y < -halfWindowSize.y - (100 * view.zoomFactor)
                or centered.x > halfWindowSize.x
                or centered.y > halfWindowSize.y)
            then
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
            renderer:renderTile(cv, renderTiles[i], renderTilePos[i], game)
            cv:translate(-pos)
        end
    end
    cv:resetTransform()
end

return TileRenderer
