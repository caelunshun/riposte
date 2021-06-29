-- Maintains a selection group for each unit.
--
-- When the player selects a unit, we automatically select
-- all units in the same group.
--
-- By default, each unit belongs to its own group. But if the player
-- moves multiple units at once, they're merged into the same group.
local SelectionGroups = {}

function SelectionGroups:new(game)
    local o = {
        groups = {},
        unitToGroup = {},
    }
    setmetatable(o, self)
    self.__index = self

    game.eventBus:registerHandler("unitCreated", function(unit)
        if unit.owner == game.thePlayer then
            o:createGroup({unit})
        end
    end)

    return o
end

-- Creates a new group containing the given units.
--
-- Any units already in a group are removed from their current group.
function SelectionGroups:createGroup(units)
    if #units == 0 then return end

    self.groups[#self.groups + 1] = units

    for _, unit in ipairs(units) do
        local previousGroup = self.unitToGroup[unit.id]
        if previousGroup ~= nil then
            removeFromTable(previousGroup, unit)
        end

        self.unitToGroup[unit.id] = units
    end
end

-- Pops the next group to be selected.
--
-- If there is no available selection, then we return nil.
function SelectionGroups:popNextGroup()
    if #self.groups == 0 then return nil end
    local group = self.groups[1]

    if #group == 0 then
        table.remove(self.groups, 1)
        return self:popNextGroup()
    end

    return self:popGroup(group)
end

-- Gets the group for the given unit.
function SelectionGroups:getUnitGroup(unit)
    return self.unitToGroup[unit.id]
end

function SelectionGroups:popGroup(group, expectedPos)
    local removed = false
    for i, g in ipairs(self.groups) do
        if g == group then
            table.remove(self.groups, i)
            removed = true
            break
        end
    end

    for _, unit in ipairs(group) do
        if self.unitToGroup[unit.id] == group then
            self.unitToGroup[unit.id] = nil
        end
    end

    self:splitGroup(group, expectedPos)
    if #group == 0 then return self:popNextGroup() end
    return group
end

-- Removes units from the given group
-- whose positions are not the same as the group's
-- position.
function SelectionGroups:splitGroup(group, expectedPos)
    local pos = expectedPos or group[1].pos
    local toRemove = {}
    for i, unit in ipairs(group) do
        if unit.pos ~= pos then
            toRemove[#toRemove + 1] = i
        end
    end

    local newGroup = {}
    for j, i in ipairs(toRemove) do
        table.insert(newGroup, table.remove(group, i - (j - 1)))
    end

    self:createGroup(newGroup)
end

function removeFromTable(t, x)
    for i=1,#t do
        if t[i] == x then
            table.remove(t, i)
            return
        end
    end
end

return SelectionGroups
