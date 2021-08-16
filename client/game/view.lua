-- The position and zoom level on the map.
--- @class View
--- @field center table the center of the screen in world space
--- @field zoomFactor number the zoom factor (higher=closer)
--- @field size table the size of the window in logical pixels
local View = {}

local dume = require("dume")
local Vector = require("brinevector")

local MoveDir = {
    Right = 0x01,
    Left = 0x02,
    Down = 0x04,
    Up = 0x10,
}

local minZoomFactor = 0.2
local maxZoomFactor = 8
local zoomSensitivity = 0.05

function View:new()
    local o = {
        center = Vector(0, 0),
        zoomFactor = 1,
        size = Vector(cv:getWidth(), cv:getHeight()),
        moveDir = 0,

        moveTime = Vector(0, 0),
        centerVelocity = Vector(0, 0),

        currentAnimation = nil,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function View:animateTo(target, overTime)
    self.currentAnimation = {
        from = self.center,
        to = target,
        totalTime = overTime,
        startTime = time,
    }
end

function View:animateToTilePos(tilePos, overTime)
    self:animateTo(Vector(tilePos.x * 100 + 50, tilePos.y * 100 + 50), overTime)
end

function View:resize(newSize)
    self.size = Vector(newSize.x, newSize.y)
end

function View:getOrigin()
    return self.center - self.size / 2
end

function View:getScreenOffsetForTilePos(tilePos)
    -- NB: we could use vector operations here instead of
    -- computing each component separately. However, for some
    -- reason this method runs an order of magnitude faster.
    return Vector(
            (tilePos.x * 100 - self.center.x + self.size.x / 2) * 0.99,
            (tilePos.y * 100 - self.center.y + self.size.y / 2) * 0.99
    )
end

function View:getTilePosForScreenOffset(screenOffset)
    local centered = screenOffset - self.size / 2
    centered = centered / self.zoomFactor
    local translated = centered + self.center
    local scaled = translated / 100
    return Vector(math.floor(scaled.x), math.floor(scaled.y))
end

local function sampleVelocityCurve(time)
    local cutoff = 1
    local max = 300
    if time >= cutoff then return max end
    return -(max / 2) * math.cos(time / (0.1 * math.pi)) + max / 2
end

function View:animateCenter()
    local anim = self.currentAnimation
    local from = anim.from
    local to = anim.to

    local time = (time - anim.startTime) / anim.totalTime

    if time > 1 then
        self.currentAnimation = nil
    end

    time = math.clamp(time, 0, 1)

    -- Smooth interpolation
    local x = (to.x - from.x) * -(math.cos(math.pi * time)) / 2 + (to.x - from.x) / 2
    local y = (to.y - from.y) * -(math.cos(math.pi * time)) / 2 + (to.y - from.y) / 2

    self.center = Vector(x, y) + from
end

function View:tick(dt, cursorPos)
    local threshold = 2

    if self.currentAnimation ~= nil then
        self:animateCenter()
    end

    self.moveDir = 0

    if math.abs(cursorPos.x - self.size.x) <= threshold then
        self.moveDir = bit.bor(self.moveDir, MoveDir.Right)
    elseif math.abs(cursorPos.x) <= threshold then
        self.moveDir = bit.bor(self.moveDir, MoveDir.Left)
    end

    if math.abs(cursorPos.y - self.size.y) <= threshold then
        self.moveDir = bit.bor(self.moveDir, MoveDir.Down)
    elseif math.abs(cursorPos.y) <= threshold then
        self.moveDir = bit.bor(self.moveDir, MoveDir.Up)
    end

    if bit.band(self.moveDir, MoveDir.Left) == 0 and bit.band(self.moveDir, MoveDir.Right) == 0 then
        self.centerVelocity.x = self.centerVelocity.x * (0.02 ^ dt)
        self.moveTime.x = 0
    end

    if bit.band(self.moveDir, MoveDir.Up) == 0 and bit.band(self.moveDir, MoveDir.Down) == 0 then
        self.centerVelocity.y = self.centerVelocity.y * (0.02 ^ dt)
        self.moveTime.y = 0
    end

    local speedX = sampleVelocityCurve(self.moveTime.x)
    local speedY = sampleVelocityCurve(self.moveTime.y)

    if bit.band(self.moveDir, MoveDir.Left) ~= 0 then
        self.centerVelocity.x = -speedX
    elseif bit.band(self.moveDir, MoveDir.Right) ~= 0 then
        self.centerVelocity.x = speedX
    end

    if bit.band(self.moveDir, MoveDir.Down) ~= 0 then
        self.centerVelocity.y = speedY
    elseif bit.band(self.moveDir, MoveDir.Up) ~= 0 then
        self.centerVelocity.y = -speedY
    end

    self.moveTime = self.moveTime + Vector(dt, dt)
    self.center = self.center + (self.centerVelocity * 1 / self.zoomFactor) * dt
end

function View:handleEvent(event)
    if event.type == dume.EventType.Scroll then
        self.zoomFactor = self.zoomFactor + event.offset.y * zoomSensitivity
        self.zoomFactor = math.clamp(self.zoomFactor, minZoomFactor, maxZoomFactor)
    end
end

-- Applies the zoom transformation to the canvas.
--
-- Make sure to call cv:resetTransform() to negate the
-- transformation after rendering.
function View:applyZoom(cv)
    local newDim = self.size / self.zoomFactor
    local diff = self.size - newDim
    cv:translate(-diff / 2 * self.zoomFactor)
    cv:scale(self.zoomFactor)
end

return View
