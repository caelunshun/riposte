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

local dume = require("dume")
local Vector = require("brinevector")

local EventBus = require("game/event")
local Hud = require("game/ui/main_hud")
local View = require("game/view")
local City = require("game/city")
local Player = require("game/player")
local Unit = require("game/unit")
local Stack = require("game/stack")

function Game:new()
    local o = {
        players = {},
        view = View:new(),

        units = {},
        stacksByPos = {},

        cities = {},
        citiesByPos = {},

        cheatMode = false,

        eventBus = EventBus:new(),
    }

    setmetatable(o, self)
    self.__index = self

    o.hud = Hud:new(o)

    return o
end

function Game:getTile(tilePos)
    return self.tiles[tilePos.x + tilePos.y * self.mapWidth + 1]
end

function Game:getVisibility(pos)
    return self.visibility[pos.x + pos.y * self.mapWidth + 1]
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
        local stack = self:getStackAtPos(data.pos)
        if stack ~= nil then
            stack:removeUnit(existingUnit)
        end
    end

    self.units[data.id] = Unit:new(data, self)

    local stackIndex = data.pos.x + data.pos.y * self.mapWidth
    local stack = self.stacksByPos[stackIndex]
    if stack == nil then self.stacksByPos[stackIndex] = Stack:new(data.pos); stack = self.stacksByPos[stackIndex] end
    stack:addUnit(self.units[data.id])

    if data.ownerID == 0 then
        self.view.center = Vector(data.pos.x * 100 + 50, data.pos.y * 100 + 50)
    end

    if existingUnit == nil then
        self.eventBus:trigger("unitCreated", self.units[data.id])
    end
    self.eventBus:trigger("unitUpdated", self.units[data.id])
end

function Game:isUnitAlive(unit)
    return self.units[unit.id] == unit
end

function Game:getStackAtPos(pos)
    return self.stacksByPos[pos.x + pos.y * self.mapWidth] or Stack:empty()
end

function Game:addCity(data)
    local city = self.cities[data.id] or City:new()
    self.cities[data.id] = city
    city:updateData(data, self)

    self.citiesByPos[city.pos.x + city.pos.y * self.mapWidth] = city

    self.eventBus:trigger("cityUpdated", city)
end

function Game:getCityAtPos(pos)
    return self.citiesByPos[pos.x + pos.y * self.mapWidth]
end

function Game:handleEvent(event)
    if event.type == dume.EventType.Key and event.key == dume.Key.L
        and event.action == dume.Action.Press then
        self.cheatMode = not self.cheatMode
    end
end

return Game
