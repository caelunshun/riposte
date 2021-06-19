package.path = "client/?.lua;external/dume/ui/?.lua;external/lunajson/src/?.lua"

local buildMainMenu = require("ui/main_menu")
local uiStyle = require("ui/style")
local Game = require("game/game")
local Renderer = require("game/renderer")
local View = require("game/view")

local dume = require("dume")
local Vector = require("brinevector")

local json = require("lunajson")

local ui = dume.UI:new(cv)
ui.style = uiStyle
buildMainMenu(ui)

-- The registry stores data loaded from JSON assets,
-- including unit kinds, civilizations, techs, etc.
registry = {
    unitKinds = {
        add = function(self, unitKind)
            self[unitKind.id] = unitKind
        end
    },
    civs = {
        add = function(self, civ)
            self[civ.id] = civ
        end
    },
    techs = {
        add = function(self, tech)
            self[tech.name] = tech
        end
    },
    resources = {
        add = function(self, resource)
            self[resource.id] = resource
        end
    },
    buildings = {
        add = function(self, building)
            self[building.name] = building
        end
    },
}

function loadDataFile(id, type, jsonData)
    print("[lua] Loading '" .. id .. "' into registry '" .. type .. "'")
    local data = json.decode(jsonData)
    local registryEntry = registry[type]
    if registryEntry == nil then error("invalid registry type " .. type) end
    registryEntry:add(data)
end

-- TEMP for testing.
local game = Game:new()
game.tiles = {}
game.view = View:new()
game.mapWidth = 64
game.mapHeight = 64
for x=1,64 do
    for y=1,64 do
        game.tiles[#game.tiles+1] = {
            terrain = "Plains",
            forested = (x % 4 == 1),
            hilled = (y % 3 == 1),
        }
        if x % 5 == 2 then game.tiles[#game.tiles].terrain = "Grassland" end
    end
end

function render(dt)
    game.view.center = Vector(game.view.center.x + 100 * dt, game.view.center.y + 100 * dt)
    Renderer:render(cv, game)
    -- ui:render()
end

function handleEvent(event)
    -- convert tables to Vector
    if event.pos ~= nil then
        event.pos = Vector(event.pos.x, event.pos.y)
    end
    if event.offset ~= nil then
        event.offset = Vector(event.offset.x, event.offset.y)
    end

    ui:handleEvent(event)
end

function resize(newSize)
    ui:resize(Vector(cv:getWidth(), cv:getHeight()), newSize)
end

