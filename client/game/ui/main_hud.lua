local Hud = {}

local dume = require("dume")
local Vector = require("brinevector")

local Flex = require("widget/flex")
local Text = require("widget/text")
local Button = require("widget/button")
local Container = require("widget/container")
local Tooltip = require("widget/tooltip")
local Image = require("widget/image")
local Padding = require("widget/padding")
local Clickable = require("widget/clickable")

local style = require("ui/style")

local SelectionGroups = require("game/ui/selection")

local BottomControlWindow = {}
local UnitDisplayWindow = {}
local TurnIndicatorWindow = {}
local ScoreWindow = {}
local UnitStackWindow = {}

function Hud:new(game)
    local o = {
        selectedUnits = {},
        selectedStack = nil,
        stagedPath = nil,
        stagedPathTarget = nil,
        game = game,
        selectionGroups = SelectionGroups:new(game),
    }

    o.windows = {
        BottomControlWindow:new(game, o),
        UnitDisplayWindow:new(game, o),
        TurnIndicatorWindow:new(game, o),
        ScoreWindow:new(game, o),
        UnitStackWindow:new(game, o),
    }

    game.eventBus:registerHandler("globalDataUpdated", function()
        o:rebuildWindows()
    end)

    setmetatable(o, self)
    self.__index = self
    return o
end

function Hud:rebuildWindows()
    for _, window in ipairs(self.windows) do
        window:rebuild()
    end
end

function Hud:handleEvent(event)
    if event.pos == nil then return end
    local clickedPos = self.game.view:getTilePosForScreenOffset(event.pos)

    local shouldComputePath = false

    if event.type == dume.EventType.MouseClick then
        if event.action == dume.Action.Press
                and event.mouse == dume.Mouse.Left then
            -- Attempt to select a unit at the top of the stack.
            local stack = self.game:getStackAtPos(clickedPos)
            local unit = stack.units[1]
            if unit ~= nil then
                self:selectUnitGroup(self.selectionGroups:popGroup(self.selectionGroups:getUnitGroup(unit), clickedPos))
                return true
            elseif #self.selectedUnits > 0 then
                self.selectedStack = nil
                local result = self:clearSelection()
                self:selectUnitGroup(self.selectionGroups:popNextGroup())
                return result
            end
        elseif event.mouse == dume.Mouse.Right then
            if event.action == dume.Action.Press then
                -- If we have a selected stack, compute the path.
                if self.selectedStack ~= nil and #self.selectedUnits > 0 then
                    shouldComputePath = true
                end
            elseif event.action == dume.Action.Release then
                if self.stagedPath ~= nil and self.hasStagedPath then
                    for _, unit in ipairs(self.selectedUnits) do
                        unit:moveTo(Vector(self.stagedPath.positions[3], self.stagedPath.positions[4]))
                    end
                    self:clearSelection()
                end

                self.stagedPath = nil
                self.hasStagedPath = false
            end
        end
    elseif event.type == dume.EventType.CursorMove and self.hasStagedPath
        and clickedPos ~= self.stagedPathTarget then
            shouldComputePath = true
    end

    if shouldComputePath and #self.selectedUnits ~= 0 then
        local from = self.selectedStack.pos
        local to = clickedPos
        local unitKindID = self.selectedUnits[1].kind.id
        local callback = function(path)
            if path ~= nil then
                self.stagedPath = path
                self.hasStagedPath = true
            end
        end
        self.game.client:requestComputePath(callback, from, to, unitKindID)
        self.stagedPathTarget = to
    end

    return false
end

function Hud:selectUnitGroup(group)
    if group == nil then return end

    for _, unit in ipairs(group) do
        if unit.owner ~= self.game.thePlayer then return end
        if self.selectedStack ~= nil and self.selectedStack.pos ~= unit.pos then
            self:clearSelection()
        end

        if self.selectedStack == nil or #self.selectedUnits == 0 then
            self.selectedStack = self.game:getStackAtPos(unit.pos)
        end

        -- don't duplicate selected units
        for i=1,#self.selectedUnits do
            if self.selectedUnits[i] == unit then return end
        end

        self.selectedUnits[#self.selectedUnits + 1] = unit
        unit.isSelected = true
    end

    self.game.eventBus:trigger("selectedUnitsUpdated", nil)
end

function Hud:selectUnit(unit)
    self:selectUnitGroup({unit})
end

function Hud:deselectUnit(unit)
    unit.isSelected = false
    for i, u in ipairs(self.selectedUnits) do
        if unit == u then
            table.remove(self.selectedUnits, i)
            self.game.eventBus:trigger("selectedUnitsUpdated", nil)
            break
        end
    end

    if #self.selectedUnits == 0 then
        self.stagedPath = nil
        self.hasStagedPath = false
    end

    self.selectionGroups:createGroup({unit})
end

function Hud:clearSelection()
    self.selectionGroups:createGroup(self.selectedUnits)

    local didDeselect = #self.selectedUnits > 0
    for _, unit in ipairs(self.selectedUnits) do
        unit.isSelected = false
    end
    self.selectedUnits = {}
    if didDeselect then
        self.game.eventBus:trigger("selectedUnitsUpdated", nil)
    end
    return didDeselect
end

local white = dume.rgb(255, 255, 255)
local spinningColor = dume.rgb(255, 255, 255, 200)

function Hud:renderStagedPath(cv)
    if not self.hasStagedPath then return end
    if self.stagedPath == nil then return end

    cv:beginPath()
    local first = true
    for i=1,#self.stagedPath.positions,2 do
        local x = self.stagedPath.positions[i]
        local y = self.stagedPath.positions[i + 1]

        local dst = self.game.view:getScreenOffsetForTilePos(Vector(x, y)) + Vector(50, 50)
        if first then
            cv:moveTo(dst)
            first = false
        else
            cv:lineTo(dst)
        end
    end

    cv:strokeWidth(5)
    cv:solidColor(white)
    cv:stroke()
end

function Hud:renderSelectionCircle(cv, time)
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
end

function Hud:render(cv, time)
    self.game.view:applyZoom(cv)
    self:renderStagedPath(cv)
    self:renderSelectionCircle(cv, time)
    cv:resetTransform()
end

local unitDisplayWindowWidth = 200
local turnIndicatorWindowWidth = 200

function BottomControlWindow:new(game, hud)
    local o = { game = game, hud = hud }
    setmetatable(o, self)
    self.__index = self
    return o
end

function BottomControlWindow:rebuild()
    local root = Flex:row()

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(cv:getWidth() - unitDisplayWindowWidth - turnIndicatorWindowWidth, 100)
    ui:createWindow("bottomControls", Vector(unitDisplayWindowWidth, cv:getHeight() - size.y), size, container)
end

function UnitDisplayWindow:new(game, hud)
    local o = { game = game, hud = hud }
    game.eventBus:registerHandler("selectedUnitsUpdated", function()
        o:rebuild()
    end)
    setmetatable(o, self)
    self.__index = self
    return o
end

function UnitDisplayWindow:rebuild()
    local root = Flex:column(10)

    local units = self.hud.selectedUnits
    if #units == 1 then
        -- Single unit; display specific information
        local unit = units[1]
        local header = Text:new("@size{20}{%unitName}", {unitName=unit.kind.name})
        table.insert(header.classes, "highlightedText")
        root:addFixedChild(header)
        root:addFixedChild(Text:new("Strength: %strength", {strength=tostring(unit.strength)}))
        root:addFixedChild(Text:new("Movement: %movement", {movement=tostring(unit.movementLeft)}))
    elseif #units ~= 0 then
        -- Multiple units; display generic info (# of each unit kind in the stack)
        local header = Text:new("@size{20}{Unit Stack (%count)}", {count=tostring(#units)})
        table.insert(header.classes, "highlightedText")
        root:addFixedChild(header)

        local unitKindCounts = {}
        for _, unit in ipairs(units) do
            unitKindCounts[unit.kind.id] = (unitKindCounts[unit.kind.id] or 0) + 1
        end

        for kindID, count in pairs(unitKindCounts) do
            local kind = registry.unitKinds[kindID]
            local text = Text:new("%bullet %kind (%count)", {
                kind = kind.name,
                count = tostring(count),
                bullet = "•",
            })
            root:addFixedChild(text)
        end
    end

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(unitDisplayWindowWidth, 150)
    ui:createWindow("unitDisplay", Vector(0, cv:getHeight() - size.y), size, container)
end

function TurnIndicatorWindow:new(game, hud)
    local o = { game = game, hud = hud }
    setmetatable(o, self)
    self.__index = self
    return o
end

function TurnIndicatorWindow:rebuild()
    local root = Flex:row()

    local flag = Image:new("icon/flag/" .. self.game.thePlayer.civ.id, turnIndicatorWindowWidth - 40)
    root:addFixedChild(flag)

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(turnIndicatorWindowWidth, 150)
    ui:createWindow("turnIndicator", Vector(cv:getWidth() - size.x, cv:getHeight() - size.y), size, container)
end

function ScoreWindow:new(game, hud)
    local o = { game = game, hud = hud }
    setmetatable(o, self)
    self.__index = self
    return o
end

function ScoreWindow:rebuild()
    local scores = Flex:column(6)

    local players = {}
    for _, player in pairs(self.game.players) do
        players[#players + 1] = player
    end
    table.sort(players, function(a, b) return a.score > b.score end)

    for _, player in ipairs(players) do
        local bracketA = ""
        local bracketB = ""

        if player == self.game.thePlayer then
            bracketA = "["
            bracketB = "]"
        end

        local text = Text:new("%score:    %bracketA@color{%col}{%playerName}%bracketB", {
            score = tostring(player.score),
            col = dumeColorToString(player.civ.color),
            playerName = player.username,
            bracketA = bracketA,
            bracketB = bracketB,
        })
        scores:addFixedChild(text)
    end

    local container = Container:new(Padding:new(scores, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(200, 175)
    ui:createWindow("scores", Vector(cv:getWidth() - size.x, cv:getHeight() - size.y - 150), size, container)
end

function UnitStackWindow:new(game, hud)
    local o = { game = game, hud = hud }
    setmetatable(o, self)
    self.__index = self

    game.eventBus:registerHandler("selectedUnitsUpdated", function()
        o:rebuild()
    end)

    return o
end

function UnitStackWindow:rebuild()
    local root = Flex:row()

    root:setCrossAlign(dume.Align.End)

    if self.hud.selectedStack ~= nil then
        for _, unit in ipairs(self.hud.selectedStack.units) do
            if unit.owner == self.game.thePlayer then
                local image = Image:new("icon/unit_head/" .. unit.kind.id, 35)

                local container = Container:new(image)
                table.insert(container.classes, "unitHeadContainer")
                if unit.isSelected then table.insert(container.classes, "unitHeadContainerSelected") end

                local clickable = Clickable:new(container, function(mods)
                    if not mods.shift then
                        self.hud:clearSelection()
                    end

                    local affectedUnits = {}
                    if mods.alt then
                        -- Affect all units in the stack.
                        affectedUnits = self.hud.selectedStack.units
                    elseif mods.control then
                        -- Affect all units in the stack with the same kind as the clicked one.
                        for _, u in ipairs(self.hud.selectedStack.units) do
                            if unit.kind == u.kind then
                                affectedUnits[#affectedUnits + 1] = u
                            end
                        end
                    else
                        -- Affect only the clicked unit.
                        affectedUnits[1] = unit
                    end

                    if unit.isSelected then
                        for _, u in ipairs(affectedUnits) do
                            self.hud:deselectUnit(u)
                        end
                    else
                        for _, u in ipairs(affectedUnits) do
                            self.hud:selectUnit(u)
                        end
                    end
                end)

                root:addFixedChild(clickable)
            end
        end
    end

    local size = Vector(cv:getWidth() - unitDisplayWindowWidth - turnIndicatorWindowWidth - 200, 100)
    ui:createWindow("unitStack", Vector(unitDisplayWindowWidth + 100, cv:getHeight() - 120 - size.y), size, root)
end

function dumeColorToString(color)
    return string.format("rgba(%d,%d,%d,%d)", color[1], color[2], color[3], color[4])
end

return Hud
