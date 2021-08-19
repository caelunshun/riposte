-- A stack of one or more units on the same tile.
-- Units are sorted by their strength (greatest to least).
---@class Stack
---@field units table
---@field pos table
local Stack = {}

function Stack:new(pos)
    local o = { pos = pos, units = {} }
    setmetatable(o, self)
    self.__index = self
    return o
end

local theEmptyStack = Stack:new(nil)
function Stack:empty()
    return theEmptyStack
end

function Stack:addUnit(unit)
    assert(unit.pos == self.pos, "unit added to a stack at the wrong position")

    -- Insert at correct sorted position
    local pos = #self.units + 1
    for i=1,#self.units do
        if self.units[i].strength <= unit.strength then
            pos = i
            break
        end
    end
    table.insert(self.units, pos, unit)
end

function Stack:removeUnit(unit)
    for i, u in ipairs(self.units) do
        if unit == u then
            table.remove(self.units, i)
            return
        end
    end
end

return Stack
