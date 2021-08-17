local InfoTextWindow = {}

local Vector = require("brinevector")

local dume = require("dume")
local Flex = require("widget/flex")
local Text = require("widget/text")
local Container = require("widget/container")
local Padding = require("widget/padding")

local getTileInfoText = require("game/ui/tile_info")

local size = Vector(200, 150)

function InfoTextWindow:new(game)
    local o = {
        game = game,
        text = "",
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function InfoTextWindow:handleEvent(event)
    if self.game.mapWidth == nil then return end

    if event.type == dume.EventType.CursorMove then
        local pos = event.pos
        local tilePos = self.game.view:getTilePosForScreenOffset(pos)

        if tilePos ~= self.hoveredTilePos then
            self.hoveredTilePos = tilePos

            local tile = self.game:getTile(tilePos)
            if tile ~= nil and self.game:getVisibility(tilePos) ~= "Hidden" then
                self:setText(getTileInfoText(tile, self.game))
            else
                self:setText("")
            end
        end
    end
end

function InfoTextWindow:setText(text)
    self.text = text
    self:rebuild()
end

function InfoTextWindow:rebuild()
    if self.text == "" then
        self:close()
        return
    end

    local text = Text:new(self.text, {
        percent = "%"
    })
    table.insert(text.classes, "lightText")
    local root = Container:new(Padding:new(text, 10))
    table.insert(root.classes, "lightContainer")
    root.fillParent = true
    ui:createWindow("infoText", function(screenSize)
        return {
            pos = Vector(0, screenSize.y - size.y - 150),
            size = size,
        }
    end, root, 3)
end

function InfoTextWindow:close()
    ui:deleteWindow("infoText")
end

return InfoTextWindow
