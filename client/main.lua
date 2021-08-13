package.path = "client/?.lua;external/dume/ui/?.lua;external/lunajson/src/?.lua;external/lua-protobuf/?.lua;external/lua-profiler/src/?.lua"
package.cpath = "cmake-build-release/lib/lib?.so;cmake-build-debug/lib/lib?.so;cmake-build-relwithdebinfo/lib/lib?.so;cmake-build-relwithdebinfo/lib/lib?.dylib;cmake-build-release/lib/lib?.dylib;cmake-build-debug/lib/lib?.dylib"
jit.on()

local profiler = require("profiler")
local enableProfiling = false

if enableProfiling then
    profiler.start()
    profiler.configuration({fW = 50, fnW = 50})
end

local buildMainMenu = require("ui/main_menu")
local uiStyle = require("ui/style")
local Game = require("game/game")
local Client = require("game/client")
local Renderer = require("game/renderer")

local Scheduler = require("scheduler")
local Networking = require("networking")
local MenuMusicPlayer = require("menu_music")

scheduler = Scheduler:new()
networking = Networking:new()

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

local lobby = nil

local menuMusic = MenuMusicPlayer:new()

local game = nil
local client = nil

local cursorPos = Vector(0, 0)

function enterLobby(l)
    lobby = l
end

function enterGame(bridge, isMultiplayer)
    lobby.client = nil
    lobby = nil

    -- Clear the UI to get rid of the menu.
    for windowName, _ in pairs(ui.windows) do ui:deleteWindow(windowName) end
    game = Game:new()
    client = Client:new(game, bridge)
    game.client = client

    menuMusic:close()
    menuMusic = nil

    if not isMultiplayer then
        client:adminStartGame()
    end
end

time = 0

local profileWritten = false

function render(dt)
    cursorPos = Vector(_G.cursorPos.x, _G.cursorPos.y)

    time = time + dt

    if enableProfiling and time > 20 and not profileWritten then
        profiler.stop()
        profiler.report("profile_data.txt")
        profileWritten = true
        print("[lua] Wrote profile data")
    end

    callSafe(function()
        networking:tick()
        scheduler:tick(time)

        if client ~= nil then
            client:handleReceivedPackets()
        end

        if game ~= nil then
            game.view:tick(dt, cursorPos)
            game.musicPlayer:tick()
            Renderer:render(cv, game)
            game.hud:render(cv, time, dt)

            if game:hasCombatEvent() then
                game:getCombatEvent():tick()
            end
        end

        if lobby ~= nil then
            lobby:tick(dt)
        end

        if menuMusic ~= nil then
            menuMusic:tick()
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

        local uiCapturedEvent = ui:handleEvent(event)

        if game ~= nil and not uiCapturedEvent then
            game.hud:handleEvent(event)
            game.view:handleEvent(event)
            game:handleEvent(event)
        end
    end)
end

function resize(oldSize, newSize)
    callSafe(function()
        ui:resize(oldSize, newSize)
    end)
end

function callSafe(f, arg)
    local status, err
    assert(f ~= nil)
    if type(f) == "function" then
        status, err = xpcall(f, debug.traceback)
    elseif type(f) == "thread" then
        status, err = coroutine.resume(f, arg)
    end
    if not status then print("LUA ERROR: " .. err) end
end
