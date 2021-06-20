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

local Player = require("game/player")

function Game:new()
    local o = {}
    setmetatable(o, self)
    self.__index = self
    return o
end

function Game:getTile(tilePos)
    return self.tiles[tilePos.x + tilePos.y * self.mapWidth + 1]
end

function Game:updatePlayer(playerdata)
    self.players[playerdata.id] = Player:new(playerdata)
end

function Game:setTurn(turn)
    self.turn = turn
end

function Game:setEra(era)
    self.era = era
end

return Game
