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

local Vector = require("brinevector")

local Hud = require("game/ui/main_hud")
local View = require("game/view")
local Player = require("game/player")
local Unit = require("game/unit")

function Game:new()
    local o = {
        players = {},
        view = View:new(),

        units = {},
        unitsByPos = {},
    }

    setmetatable(o, self)
    self.__index = self

    o.hud = Hud:new(o)

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

function Game:addUnit(data)
    local existingUnit = self.units[data.id]
    if existingUnit ~= nil then
        local byPos = self.unitsByPos[existingUnit.pos.x + existingUnit.pos.y * self.mapWidth] or {}
        for i, unit in ipairs(byPos) do
            if unit == existingUnit then table.remove(byPos, i); break end
        end
    end

    self.units[data.id] = Unit:new(data)

    local byPosIndex = data.pos.x + data.pos.y * self.mapWidth
    local byPos = self.unitsByPos[byPosIndex]
    if byPos == nil then self.unitsByPos[byPosIndex] = {}; byPos = self.unitsByPos[byPosIndex] end
    table.insert(byPos, self.units[data.id])

    if data.ownerID == 0 then
        self.view.center = Vector(data.pos.x * 100 + 50, data.pos.y * 100 + 50)
    end
end

function Game:isUnitAlive(unit)
    return self.units[unit.id] == unit
end

function Game:getUnitsAtPos(pos)
    return self.unitsByPos[pos.x + pos.y * self.mapWidth] or {}
end

return Game
