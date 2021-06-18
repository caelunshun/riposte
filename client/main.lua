package.path = "client/?.lua;external/dume/ui/?.lua"

local buildMainMenu = require("ui/main_menu")
local uiStyle = require("ui/style")

local dume = require("dume")
local Vector = require("brinevector")

local ui = dume.UI:new(cv)
ui.style = uiStyle
buildMainMenu(ui)

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
