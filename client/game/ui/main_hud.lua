local Hud = {}

local dume = require("dume")
local Vector = require("brinevector")

local Flex = require("widget/flex")
local Text = require("widget/text")
local Button = require("widget/button")
local Container = require("widget/container")
local Tooltip = require("widget/tooltip")

function Hud:new(game)
    local o = {
        selectedUnits = {},
        stagedPath = nil,
        game = game,
    }
    game.eventBus:registerHandler("unitUpdated", function(unit) o:onUnitUpdated(unit) end)
    game.eventBus:registerHandler("globalDataUpdated", function() o:rebuildBottomBar()  end)
    setmetatable(o, self)
    self.__index = self
    o:rebuildBottomBar()
    return o
end

function Hud:handleEvent(event)
    if event.type == dume.EventType.MouseClick then
        local clickedPos = self.game.view:getTilePosForScreenOffset(event.pos)

        if event.action == dume.Action.Press
                and event.mouse == dume.Mouse.Left then
            -- Attempt to select a unit.
            local unit = self.game:getUnitsAtPos(clickedPos)[1]
            if unit ~= nil then
                self.selectedUnits = { unit }
                return true
            else
                local deselected = #self.selectedUnits ~= 0
                self.selectedUnits = {}
                return deselected
            end
        end
    end

    return false
end

local spinningColor = dume.rgb(255, 255, 255, 200)

function Hud:render(cv, time)
    self.game.view:applyZoom(cv)
    for _, unit in ipairs(self.selectedUnits) do
        -- Paint spinning white dashes
        local radius = 50
        local center = self.game.view:getScreenOffsetForTilePos(unit.pos) + Vector(radius, radius)

        cv:beginPath()
        local numDashes = 16
        local angleOffset = time * 2 * math.pi / 10
        for i=1,numDashes do
            i = i - 1
            local arcLength = 2 * math.pi / numDashes
            local arcStart = angleOffset + i * arcLength
            local arcEnd = angleOffset + (i + 1) * arcLength - 0.1

            cv:moveTo(Vector(center.x + radius * math.cos(arcStart), center.y + radius * math.sin(arcStart)))
            cv:arc(center, radius, arcStart, arcEnd)
            cv:moveTo(Vector(center.x + radius * math.cos(arcEnd + 0.3), center.y + radius * math.sin(arcEnd + 0.3)))
        end

        cv:solidColor(spinningColor)
        cv:strokeWidth(4)
        cv:stroke()
    end
    cv:resetTransform()
end

function Hud:rebuildBottomBar()
    local root = Flex:row()

    local turnText = Text:new("@size{20}{Turn %turn\n%era Era}", {turn=tostring(self.game.turn),era=self.game.era})
    root:addFixedChild(turnText)

    local nextTurn = Button:new(Text:new("Next Turn"), function()
        print("Next turn!")
    end)
    root:addFixedChild(nextTurn)

    local container = Container:new(root)
    container.fillParent = true

    local height = 100
    local size = Vector(self.game.view.size.x, height)
    ui:createWindow("hudBottomBar", Vector(0, self.game.view.size.y - height), size, container)
end

function Hud:onUnitUpdated(unit)
    if unit == self.selectedUnits[1] then self:rebuildBottomBar() end
end

return Hud
