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
local HUD = require("game/ui/hud")
local View = require("game/view")
local City = require("game/city")
local Player = require("game/player")
local Unit = require("game/unit")
local Stack = require("game/stack")
local CombatEvent = require("game/combat")

local MusicPlayer = require("game/music")

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

    o.hud = HUD:new(o)
    o.musicPlayer = MusicPlayer:new(o)

    return o
end

function Game:getTile(tilePos)
    return self.tiles[tilePos.x + tilePos.y * self.mapWidth + 1]
end

function Game:getVisibility(pos)
    return self.visibility[pos.x + pos.y * self.mapWidth + 1]
end

function Game:updatePlayer(playerdata)
    local player = self.players[playerdata.id] or Player:new(playerdata)
    self.players[player.id] = player
    player:updateData(playerdata)
end

function Game:setTurn(turn)
    local oldTurn = self.turn
    self.turn = turn

    if oldTurn ~= turn then
        self.eventBus:trigger("turnChanged")
    end
end

function Game:setEra(era)
    local oldEra = self.era
    self.era = era

    if era ~= oldEra then
        self.eventBus:trigger("eraChanged")
    end
end

function Game:addUnit(data)
    local existingUnit = self.units[data.id]
    if existingUnit ~= nil then
        local stack = self:getStackAtPos(existingUnit.pos)
        if stack ~= nil then
            stack:removeUnit(existingUnit)
        end
    end

    local unit = existingUnit or Unit:new(self)
    unit:updateData(data, self)
    self.units[unit.id] = unit

    local stackIndex = unit.pos.x + unit.pos.y * self.mapWidth
    local stack = self.stacksByPos[stackIndex]
    if stack == nil then self.stacksByPos[stackIndex] = Stack:new(unit.pos); stack = self.stacksByPos[stackIndex] end
    stack:addUnit(unit)

    if existingUnit == nil then
        self.eventBus:trigger("unitCreated", unit)

        if unit.ownerID == self.thePlayer.id then
            self.view.center = Vector(unit.pos.x * 100 + 50, unit.pos.y * 100 + 50)
        end
    end
    self.eventBus:trigger("unitUpdated", unit)
end

function Game:deleteUnit(id)
    local unit = self.units[id]
    if unit == nil then return end
    self:getStackAtPos(unit.pos):removeUnit(unit)
    self.units[id] = nil
    self.eventBus:trigger("unitDeleted", unit)
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

function Game:updateThePlayer(packet)
    self.thePlayer:updateData(packet)
    self.eventBus:trigger("thePlayerUpdated")
end

function Game:hasCombatEvent()
    return self.currentCombatEvent ~= nil
end

function Game:getCombatEvent()
    return self.currentCombatEvent
end

function Game:clearCombatEvent()
    self.currentCombatEvent = nil
end

function Game:startCombatEvent(packet)
    self.currentCombatEvent = CombatEvent:new(self, packet)
end

return Game
