-- Renders base terrain (grassland, desert, etc.)
local TerrainRenderer = {}

local Vector = require("brinevector")

function TerrainRenderer:renderTile(cv, tile, pos)
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

    if terrain.hilled then spriteName = spriteName .. "_hill" end

    cv:drawSprite(spriteName, pos, 100)
end

-- Renders trees over terrain
local TreeRenderer = {
    treeSpriteSize = Vector(0, 0)
}

cv:getSpriteSize("icon/tree", TreeRenderer.treeSpriteSize)

function TreeRenderer:renderTile(cv, tile, pos, tilePos, game)
    if not tile.forested then return end

    local seed = tilePos.x + tilePos.y * game.mapWidth
    math.randomseed(seed)

    local numTrees = math.random(10, 20)
    for _=0,numTrees do
        local scaleX = (math.random() + 1) * 25
        local scaleY = scaleX * self.treeSpriteSize.y / self.treeSpriteSize.x
        local treePos = pos + Vector(math.random() * 100, math.random() * 100)
        treePos = treePos - Vector(scaleX, scaleY) / 2
        cv:drawSprite("icon/tree", treePos, scaleX)
    end
end

-- Responsible for rendering tiles on the map.
local TileRenderer = {
    renderers = {
        -- NB: order determines layering
        TerrainRenderer,
        TreeRenderer,
    }
}

function TileRenderer:render(cv, game)
    local view = game.view

    local renderers = self.renderers

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
                for i=1,#renderers do
                    renderers[i]:renderTile(cv, tile, pos, tilePos, game)
                end
            end
        end
    end
end

return TileRenderer
