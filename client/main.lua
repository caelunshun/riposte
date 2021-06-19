package.path = "client/?.lua;external/dume/ui/?.lua;external/lunajson/src/?.lua"

local buildMainMenu = require("ui/main_menu")
local uiStyle = require("ui/style")

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

function render()
    ui:render()
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
    print(newSize.x, newSize.y)
    ui:resize(Vector(cv:getWidth(), cv:getHeight()), newSize)
end

