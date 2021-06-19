local Renderer = {}

local TileRenderer = require("game/render/tile")

Renderer.subrenderers = {
    -- NB: order determines layering
    TileRenderer
}

function Renderer:render(cv, game)
    for _, subrenderer in ipairs(self.subrenderers) do
        subrenderer:render(cv, game)
    end
end

return Renderer
