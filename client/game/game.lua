-- Represents the entire game state.
--
-- Avoid using globals to contain game state. Store everything
-- in this table.
--
--- @class Game
--- @field view View
--- @field mapWidth number
--- @field mapHeight number
local Game = {}

function Game:new()
    local o = {}
    setmetatable(o, self)
    self.__index = self
    return o
end

function Game:getTile(tilePos)
    return self.tiles[tilePos.x + tilePos.y * self.mapWidth + 1]
end

return Game
