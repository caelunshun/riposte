local InfoTextWindow = {}

local Vector = require("brinevector")

local dume = require("dume")

local style = require("ui/style")
local getTileInfoText = require("game/ui/tile_info")

local width = 200
local padding = 10

function InfoTextWindow:new(game)
    local o = {
        game = game,
        text = "",
        paragraph = nil,
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
            if tile ~= nil and (self.game.cheatMode or self.game:getVisibility(tilePos) ~= "Hidden") then
                self:setText(getTileInfoText(tile, tilePos, self.game))
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
    if #self.text == 0 then
        self.paragraph = nil
        return
    end

    local text = cv:parseTextMarkup(self.text, style.default.lightText.defaultTextStyle, {
        percent = "%",
    })
    self.paragraph = cv:createParagraph(text, {
        alignH = dume.Align.Start,
        alignV = dume.Align.Start,
        baseline = dume.Baseline.Top,
        lineBreaks = true,
        maxDimensions = Vector(width - padding * 2, math.huge)
    })
end

function InfoTextWindow:render(cv)
    if self.paragraph == nil then return end

    local textHeight = cv:getParagraphHeight(self.paragraph)

    local totalHeight = textHeight + padding * 2

    cv:beginPath()
    cv:rect(Vector(0, cv:getHeight() - totalHeight - 150), Vector(width, totalHeight))
    cv:solidColor(dume.rgb(50, 50, 50, 180))
    cv:fill()

    cv:drawParagraph(self.paragraph, Vector(padding, cv:getHeight() - 150 - totalHeight + padding))
end

function InfoTextWindow:close()

end

return InfoTextWindow
