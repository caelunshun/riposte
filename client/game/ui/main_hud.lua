local Hud = {}

local dume = require("dume")
local Vector = require("brinevector")

function Hud:new(game)
    local o = {
        selectedUnits = {},
        stagedPath = nil,
        game = game,
    }
    setmetatable(o, self)
    self.__index = self
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

            cv:arc(center, radius, arcStart, arcEnd)
            cv:moveTo(Vector(center.x + radius * math.cos(arcEnd + 0.3), center.y + radius * math.sin(arcEnd + 0.3)))
        end

        cv:solidColor(spinningColor)
        cv:strokeWidth(4)
        cv:stroke()
    end
    cv:resetTransform()
end

return Hud
