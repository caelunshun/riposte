package.path = "client/?.lua;external/dume/ui/?.lua;external/lunajson/src/?.lua;external/lua-protobuf/?.lua"
package.cpath = "cmake-build-release/lib/lib?.so;cmake-build-debug/lib/lib?.so"
jit.on()

local buildMainMenu = require("ui/main_menu")
local uiStyle = require("ui/style")
local Game = require("game/game")
local Client = require("game/client")
local Renderer = require("game/renderer")

local dume = require("dume")
local Vector = require("brinevector")

local json = require("lunajson")

ui = dume.UI:new(cv, uiStyle)
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
            civ.color = dume.rgb(civ.color[1], civ.color[2], civ.color[3])
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

local game = nil
local client = nil

local cursorPos = Vector(0, 0)

function enterGame(bridge)
    -- Clear the UI to get rid of the menu.
    for windowName, _ in pairs(ui.windows) do ui:deleteWindow(windowName) end
    game = Game:new()
    client = Client:new(game, bridge)
end

local time = 0

function render(dt)
    cursorPos = Vector(_G.cursorPos.x, _G.cursorPos.y)

    time = time + dt

    callSafe(function()
        if client ~= nil then
            client:handleReceivedPackets()
        end

        if game ~= nil then
            game.view:tick(dt, cursorPos)
            Renderer:render(cv, game)
            game.hud:render(cv, time)
        end

        ui:render()
        cv:drawSprite("icon/cursor", cursorPos, 25)
    end)
end

function handleEvent(event)
    callSafe(function()
        -- convert tables to Vector
        if event.pos ~= nil then
            event.pos = Vector(event.pos.x, event.pos.y)
        end
        if event.offset ~= nil then
            event.offset = Vector(event.offset.x, event.offset.y)
        end

        ui:handleEvent(event)

        if game ~= nil then
            game.hud:handleEvent(event)
            game.view:handleEvent(event)
            game:handleEvent(event)
        end
    end)
end

function resize(newSize)
    callSafe(function()
        ui:resize(Vector(cv:getWidth(), cv:getHeight()), newSize)
    end)
end

function callSafe(f)
    local status, err = pcall(f)
    if not status then print("LUA ERROR: " .. err) end
end

