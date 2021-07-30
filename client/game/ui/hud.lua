-- Renders the current HUD, whether that is
-- the main HUD or the city HUD.
local HUD = {}

local dume = require("dume")

local MainHud = require("game/ui/main_hud")
local CityHud = require("game/ui/city_hud")

local doubleClickThreshold = 0.5

function HUD:new(game)
    local o = {
        game = game,
        mainHud = MainHud:new(game),
        lastClickTime = nil,
    }
    setmetatable(o, self)
    self.__index = self
    o:setCurrent(o.mainHud)
    return o
end

function HUD:setCurrent(hud)
    if self.currentHud ~= nil then
        self.currentHud:close()
        self.currentHud.active = false
        hud:rebuildWindows()
    end

    hud.active = true

    self.currentHud = hud
end

function HUD:render(cv, time, dt)
    self.currentHud:render(cv, time, dt)
end

function HUD:handleEvent(event)
    self:checkForDoubleClicks(event)
    self.currentHud:handleEvent(event)

    if self.currentHud.closed then
        self:setCurrent(self.mainHud)
    end
end

function HUD:checkForDoubleClicks(event)
    if event.type == dume.EventType.MouseClick and event.mouse == dume.Mouse.Left
        and event.action == dume.Action.Press then
        if self.lastClickTime ~= nil and time - self.lastClickTime < doubleClickThreshold then
            self:onDoubleClick(event.pos)
        end

        self.lastClickTime = time
    end
end

function HUD:onDoubleClick(pos)
    if self.currentHud ~= self.mainHud then return end

    -- Attempt to open the city HUD.
    local clickedTilePos = self.game.view:getTilePosForScreenOffset(pos)
    local city = self.game:getCityAtPos(clickedTilePos)
    if city ~= nil then
        local cityHud = CityHud:new(self.game, city)
        self:setCurrent(cityHud)
        self.game.eventBus:trigger("cityHudOpened")
    end
end

return HUD
