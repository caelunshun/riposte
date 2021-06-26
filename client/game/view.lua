-- The position and zoom level on the map.
--- @class View
--- @field center table the center of the screen in world space
--- @field zoomFactor number the zoom factor (higher=closer)
--- @field size table the size of the window in logical pixels
local View = {}

local Vector = require("brinevector")

function View:new()
    local o = {
        center = Vector(0, 0),
        zoomFactor = 1,
        size = Vector(cv:getWidth(), cv:getHeight()),
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function View:resize(newSize)
    self.size = Vector(newSize.x, newSize.y)
end

function View:getOrigin()
    return self.center - self.size / 2
end

function View:getScreenOffsetForTilePos(tilePos)
    return (tilePos * 100 - self:getOrigin()) * 0.99
end

function View:getTilePosForScreenOffset(screenOffset)
    local centered = screenOffset - self.size / 2
    centered = centered / self.zoomFactor
    local translated = centered + self.center
    local scaled = translated / 100
    return Vector(math.floor(scaled.x), math.floor(scaled.y))
end

return View
