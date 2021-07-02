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
local ProgressBar = require("widget/progress_bar")

local style = require("ui/style")

local TaskSequence = require("task_sequence")
local SelectionGroups = require("game/ui/selection")

local prompts = require("game/ui/prompts")
local DiplomacyDialogue = require("game/ui/dialogue")

local buildUnitActions = require("game/ui/unit_actions")

local BottomControlWindow = {}
local UnitDisplayWindow = {}
local TurnIndicatorWindow = {}
local ScoreWindow = {}
local UnitStackWindow = {}
local ResearchBar = {}
local EconomyWindow = {}

function Hud:new(game)
    local o = {
        selectedUnits = {},
        selectedStack = nil,
        stagedPath = nil,
        stagedPathTarget = nil,
        game = game,
        selectionGroups = SelectionGroups:new(game),
        readyForNextTurn = false,
        tasks = TaskSequence:new(),
        timeSinceLastSelect = nil,
        promptQueue = prompts.PromptQueue:new(),
    }

    game.eventBus:registerHandler("globalDataUpdated", function()
        if o.active then
            o:rebuildWindows()
        end
    end)

    game.eventBus:registerHandler("cityUpdated", function(city)
        if o.active then
            o:onCityUpdated(city)
        end
    end)

    game.eventBus:registerHandler("thePlayerUpdated", function()
        if o.active then
            o:onThePlayerUpdated()
            o:rebuildWindows()
        end
    end)

    game.eventBus:registerHandler("turnChanged", function()
        o.readyForNextTurn = false
        o.timeSinceLastSelect = nil
    end)

    game.eventBus:registerHandler("unitDeleted", function(unit)
        for i, u in ipairs(o.selectedUnits) do
            if u == unit then
                table.remove(o.selectedUnits, i)
                game.eventBus:trigger("selectedUnitsUpdated", nil)
                break
            end
        end

        if #o.selectedUnits == 0 then
            o.selectedStack = nil
        end
    end)

    game.eventBus:registerHandler("unitUpdated", function(unit)
        for _, u in ipairs(o.selectedUnits) do
            if u == unit then
                o.selectedStack = game:getStackAtPos(u.pos)
            end
        end
    end)

    o.windows = {
        BottomControlWindow:new(game, o),
        UnitDisplayWindow:new(game, o),
        TurnIndicatorWindow:new(game, o),
        ScoreWindow:new(game, o),
        UnitStackWindow:new(game, o),
        ResearchBar:new(game, o),
        EconomyWindow:new(game, o),
    }

    setmetatable(o, self)
    self.__index = self
    return o
end

function Hud:rebuildWindows()
    for _, window in ipairs(self.windows) do
        window:rebuild()
    end
end

function Hud:close()
    for _, window in ipairs(self.windows) do
        window:close()
    end
end

function Hud:doUnitActions(action)
    for _, unit in ipairs(self.selectedUnits) do
        self.game.client:doUnitAction(unit, action)
    end

    self:clearSelection()
end

function Hud:handleKeyInput(event)
    if event.action ~= dume.Action.Release then return end

    if event.key == dume.Key.F then
        self:doUnitActions("Fortify")
    elseif event.key == dume.Key.Space then
        self:doUnitActions("SkipTurn")
    elseif event.key == dume.Key.H then
        self:doUnitActions("FortifyUntilHealed")
    end
end

function Hud:handleEvent(event)
    if self.promptQueue:getCurrentPrompt() ~= nil then return end

    if event.type == dume.EventType.Key and event.action == dume.Action.Press
            and event.key == dume.Key.Return and self.readyForNextTurn then
        self.game.client:endTurn()
        self.readyForNextTurn = false
    end

    if event.type == dume.EventType.Key then
        self:handleKeyInput(event)
    end

    if event.pos == nil then return end
    local clickedPos = self.game.view:getTilePosForScreenOffset(event.pos)

    local shouldComputePath = false

    if event.type == dume.EventType.MouseClick then
        if event.action == dume.Action.Press
                and event.mouse == dume.Mouse.Left
                and not (self.selectedStack ~= nil and self.selectedStack.pos == clickedPos and #self.selectedUnits > 0) then
            -- Attempt to select a unit at the top of the stack.
            local stack = self.game:getStackAtPos(clickedPos)
            local unit = stack.units[1]
            if unit ~= nil then
                self:selectUnitGroup(self.selectionGroups:popGroup(self.selectionGroups:getUnitGroup(unit), clickedPos))
                return true
            elseif #self.selectedUnits > 0 then
                self.selectedStack = nil
                local result = self:clearSelection()
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
                    self:moveSelectionAlongStagedPath()
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
        self:computePath(self.selectedStack.pos, clickedPos)
    end

    return false
end

-- Enqueues a task that computes a path between the given points.
function Hud:computePath(from, to)
    self.stagedPathTarget = to

    if from == to then
        self.stagedPath = { positions = {from.x, from.y} }
        return
    end

    local unitKindID = self.selectedUnits[1].kind.id

    self.tasks:enqueue(coroutine.create(function()
        local path = self.game.client:requestComputePath(from, to, unitKindID)
        self.stagedPath = path
        self.hasStagedPath = true
    end))
end

function Hud:moveGroupAlongPath(units, path, successCallback, failureCallback)
    self.tasks:enqueue(coroutine.create(function()
        local success = self.game.client:moveUnitsAlongPath(units, path)
        if success then
            -- Consume the part of the path that we followed.
            while #path.positions >= 2 and Vector(path.positions[1], path.positions[2]) ~= units[1].pos do
                table.remove(path.positions, 1)
                table.remove(path.positions, 1)
            end

            units.followingPath = path

            if successCallback ~= nil then successCallback() end
        else
            if failureCallback ~= nil then failureCallback() end
        end
    end))
end

-- Enqueues a task that moves the current selection along the currently staged path.
-- After the task finishes, the selection is cleared if moving the units was successful.
function Hud:moveSelectionAlongStagedPath()
    local units = self.selectedUnits
    self:moveGroupAlongPath(units, self.stagedPath, function()
        local hasMovementLeft = true
        for _, unit in ipairs(units) do
            if unit.movementLeft < 0.1 then
                hasMovementLeft = false
                break
            end
        end

        if not hasMovementLeft then
            self:clearSelection()
        else
            self.selectedStack = self.game:getStackAtPos(units[1].pos)
        end
    end)
end

function Hud:selectUnitGroup(group)
    self.tasks:enqueue(coroutine.create(function()
        if group == nil then return end

        for _, unit in ipairs(group) do
            if unit.owner ~= self.game.thePlayer then return end
            if self.game:isUnitAlive(unit) then
                if self.selectedStack ~= nil and self.selectedStack.pos ~= unit.pos then
                    self:clearSelectionNow()
                end

                if self.selectedStack == nil or #self.selectedUnits == 0 then
                    self.selectedStack = self.game:getStackAtPos(unit.pos)
                end

                -- don't duplicate selected units
                for i=1,#self.selectedUnits do
                    if self.selectedUnits[i] == unit then break end
                end

                self.selectedUnits[#self.selectedUnits + 1] = unit
                unit.isSelected = true
            end
        end

        self.game.eventBus:trigger("selectedUnitsUpdated", nil)
    end))
end

function Hud:selectUnit(unit)
    self:selectUnitGroup({unit})
end

function Hud:deselectUnit(unit)
    self.tasks:enqueue(coroutine.create(function()
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
    end))
end

function Hud:clearSelectionNow()
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

function Hud:clearSelection()
    self.tasks:enqueue(coroutine.create(function()
        self:clearSelectionNow()
    end))
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

function Hud:renderNextTurnPrompt(cv, time)
    local size = Vector(cv:getWidth(), cv:getHeight())

    -- animate alpha
    local alpha = math.floor((math.cos(time * math.pi) + 1) / 2 * 255)
    local text = cv:parseTextMarkup("@color{%color}{@size{18}{Press <ENTER> to end turn....}}", style.default.text.defaultTextStyle, {
        color = dumeColorToString(dume.rgb(255, 255, 255, alpha)),
    })
    local paragraph = cv:createParagraph(text, {
        alignH = dume.Align.Center,
        alignV = dume.Align.Start,
        baseline = dume.Baseline.Middle,
        lineBreaks = false,
        maxDimensions = size,
    })

    cv:drawParagraph(paragraph, Vector(0, cv:getHeight() - 150))
end

-- Automatically selects the next available unit group.
function Hud:doAutoSelect()
    if self.promptQueue:getCurrentPrompt() ~= nil then return end

    self.tasks:enqueue(coroutine.create(function()
        local group = self.selectionGroups:popNextGroup()

        if group == nil and #self.selectedUnits == 0 then
            self.readyForNextTurn = true
            return
        else
            self.readyForNextTurn = false
        end

        if group == nil then return end

        if group.followingPath ~= nil and #group.followingPath.positions > 0 then
            self:moveGroupAlongPath(group, group.followingPath, function()
                self.selectionGroups:createGroup(group)
            end, function()
                self:selectUnitGroup(group)
            end)
        else
            self:selectUnitGroup(group)
        end
    end))
end

function Hud:render(cv, time, dt)
    self.promptQueue:tick()

    if #self.selectedUnits > 0 then
        self.timeSinceLastSelect = nil
    else
        self.timeSinceLastSelect = (self.timeSinceLastSelect or 0) + dt
    end

    if self.timeSinceLastSelect ~= nil and self.timeSinceLastSelect > 0.5 then
        self:doAutoSelect()
    end

    self.tasks:tick()

    self.game.view:applyZoom(cv)

    self:renderStagedPath(cv)
    self:renderSelectionCircle(cv, time)

    cv:resetTransform()

    if self.readyForNextTurn then
        self:renderNextTurnPrompt(cv, time)
    end
end

function Hud:onCityUpdated(city)
    if city.owner == self.game.thePlayer and city.buildTask == nil then
        -- Prompt the user to set the new build task.
        local co = coroutine.create(function()
            self.promptQueue:push(prompts.CityBuildPrompt:new(self.game, city))
        end)
        callSafe(co)
    end
end

function Hud:onThePlayerUpdated()
    if self.game.thePlayer.researchingTech == nil and self.game.turn ~= 0 then
        local co = coroutine.create(function()
            self.promptQueue:push(prompts.ResearchPrompt:new(self.game))
        end)
        callSafe(co)
    end
end

local unitDisplayWindowWidth = 200
local turnIndicatorWindowWidth = 250

function BottomControlWindow:new(game, hud)
    local o = { game = game, hud = hud }
    setmetatable(o, self)
    self.__index = self
    game.eventBus:registerHandler("selectedUnitsUpdated", function()
        if hud.active then
            o:rebuild()
        end
    end)
    game.eventBus:registerHandler("unitUpdated", function(unit)
        if hud.active then
            o:rebuild()
        end
    end)
    return o
end

function BottomControlWindow:rebuild()
    local root = Flex:row()
    root:setCrossAlign(dume.Align.Center)

    buildUnitActions(root, self.hud.selectedUnits, self.game, self.hud)

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(cv:getWidth() - unitDisplayWindowWidth - turnIndicatorWindowWidth, 100)
    ui:createWindow("bottomControls", Vector(unitDisplayWindowWidth, cv:getHeight() - size.y), size, container, true, true)
end

function BottomControlWindow:close()
    ui:deleteWindow("bottomControls")
end

function UnitDisplayWindow:new(game, hud)
    local o = { game = game, hud = hud }
    game.eventBus:registerHandler("selectedUnitsUpdated", function()
        if hud.active then
            o:rebuild()
        end
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

        -- Display current worker task, if any
        local workerCap = unit:getCapability("worker")
        if workerCap ~= nil and workerCap.currentTask ~= nil then
            root:addFixedChild(Text:new("%presentParticiple (%turnsLeft)", {
                presentParticiple = workerCap.currentTask.presentParticiple,
                turnsLeft = tostring(workerCap.currentTask.turnsLeft),
            }))
        end
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
    ui:createWindow("unitDisplay", Vector(0, cv:getHeight() - size.y), size, container, false, false)
end

function UnitDisplayWindow:close()
    ui:deleteWindow("unitDisplay")
end

function TurnIndicatorWindow:new(game, hud)
    local o = { game = game, hud = hud }
    setmetatable(o, self)
    self.__index = self
    return o
end

function TurnIndicatorWindow:rebuild()
    local root = Flex:row()

    local flag = Image:new("icon/flag/" .. self.game.thePlayer.civ.id, turnIndicatorWindowWidth - 50)
    root:addFixedChild(flag)

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(turnIndicatorWindowWidth, 150)
    ui:createWindow("turnIndicator", Vector(cv:getWidth() - size.x, cv:getHeight() - size.y), size, container, false, true)
end

function TurnIndicatorWindow:close()
    ui:deleteWindow("turnIndicator")
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

        local war = ""
        if self.game.thePlayer:isAtWarWith(player) then
            war = " (WAR)"
        end

        local text = Text:new("%score:    %bracketA@color{%col}{%playerName}%bracketB@color{rgb(207,69,32)}{%war}", {
            score = tostring(player.score),
            col = dumeColorToString(player.civ.color),
            playerName = player.username,
            bracketA = bracketA,
            bracketB = bracketB,
            war =  war,
        })

        local wrapper = Clickable:new(text, function()
            -- open diplomacy dialogue with the player
            if player ~= self.game.thePlayer then
                self.hud.promptQueue:push(DiplomacyDialogue:new(self.game, player))
            end
        end)

        scores:addFixedChild(wrapper)
    end

    local container = Container:new(Padding:new(scores, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(250, 225)
    ui:createWindow("scores", Vector(cv:getWidth() - size.x, cv:getHeight() - size.y - 150), size, container, false, true)
end

function ScoreWindow:close()
    ui:deleteWindow("scores")
end

local UnitStatus = {
    Green = dume.rgb(68, 194, 113),
    Yellow = dume.rgb(254, 221, 0),
    Red = dume.rgb(231, 60, 62),
    Gray = dume.rgb(180, 180, 180),
}

local function getUnitStatus(unit)
    if unit.isFortified then
        return UnitStatus.Gray
    elseif unit.movementLeft < 0.1 then
        return UnitStatus.Red
    elseif unit.movementLeft < unit.kind.movement then
        return UnitStatus.Yellow
    else
        return UnitStatus.Green
    end
end

-- Widget that draws a circle whose color indicates a unit's status.
local UnitStatusIndicator = {}

function UnitStatusIndicator:new(status)
    local o = { params = {status = status}, fillParent = true }
    setmetatable(o, self)
    self.__index = self
    return o
end

function UnitStatusIndicator:paint(cv)
    cv:beginPath()
    cv:rect(Vector(-5, -5), Vector(10, 10))
    cv:solidColor(self.params.status)
    cv:fill()

    -- border
    cv:strokeWidth(0.8)
    cv:solidColor(dume.rgb(0, 0, 0))
    cv:stroke()
end

function UnitStackWindow:new(game, hud)
    local o = { game = game, hud = hud }
    setmetatable(o, self)
    self.__index = self

    game.eventBus:registerHandler("selectedUnitsUpdated", function()
        if hud.active then
            o:rebuild()
        end
    end)
    game.eventBus:registerHandler("unitUpdated", function()
        if hud.active then
            o:rebuild()
        end
    end)

    return o
end

function UnitStackWindow:rebuild()
    local root = Flex:row()

    root:setCrossAlign(dume.Align.End)

    if self.hud.selectedStack ~= nil and #self.hud.selectedUnits > 0 then
        for _, unit in ipairs(self.hud.selectedStack.units) do
            if unit.owner == self.game.thePlayer then
                local status = getUnitStatus(unit)
                local indicator = UnitStatusIndicator:new(status)
                local image = Image:new("icon/unit_head/" .. unit.kind.id, 35, indicator)

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
                        self.hud:selectUnitGroup(affectedUnits)
                    end
                end)

                root:addFixedChild(clickable)
            end
        end
    end

    local size = Vector(cv:getWidth() - unitDisplayWindowWidth - turnIndicatorWindowWidth - 200, 100)
    ui:createWindow("unitStack", Vector(unitDisplayWindowWidth + 100, cv:getHeight() - 120 - size.y), size, root, false, true)
end

function UnitStackWindow:close()
    ui:deleteWindow("unitStack")
end

function ResearchBar:new(game, hud)
    local o = {
        game = game,
        hud = hud,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function ResearchBar:rebuild()
    local size = Vector(400, 30)

    local progress = 0
    local predictedProgress = 0

    local text = "Research: None"

    local research = self.game.thePlayer.researchingTech
    if research ~= nil then
        local tech = registry.techs[research.techID]
        local cost = tech.cost
        progress = research.progress / cost
        predictedProgress = (research.progress + self.game.thePlayer.beakerRevenue) / cost

        local turns = self.game.thePlayer:estimateResearchTurns(tech, research.progress)
        text = "Research: " .. research.techID .. " (" .. maybeInfinity(turns) .. ")"
    end

    local child = Text:new("@size{15}{" .. text .. "}", {}, {
        alignH = dume.Align.Center,
        alignV = dume.Align.Center,
    })

    local root = ProgressBar:new(size, function()
        return progress
    end, function()
        return predictedProgress
    end, child)
    table.insert(root.classes, "researchProgressBar")

    ui:createWindow("researchBar", Vector(cv:getWidth() / 2 - size.x / 2, 1), size, root)
end

function ResearchBar:close()
    ui:deleteWindow("researchBar")
end

function EconomyWindow:new(game, hud)
    local o = {
        game = game,
        hud = hud,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function EconomyWindow:rebuild()
    local root = Flex:column(10)

    local positiveColor = dume.rgb(68, 194, 113)
    local negativeColor = dume.rgb(231, 60, 62)

    -- Gold
    local delta, deltaColor
    local netGold = self.game.thePlayer.netGold
    if netGold == nil then return end
    if netGold >= 0 then
        delta = string.format("+%d", netGold)
        deltaColor = positiveColor
    else
        delta = tostring(netGold)
        deltaColor = negativeColor
    end
    local goldText = Text:new("@icon{gold}: %gold @color{%deltaColor}{(%delta / turn)}", {
        gold = self.game.thePlayer.gold,
        delta = delta,
        deltaColor = dumeColorToString(deltaColor),
    })
    root:addFixedChild(goldText)

    local expenses = Text:new("@color{%negativeColor}{Expenses: } %expenses", {
        expenses = self.game.thePlayer.expenses,
        negativeColor = dumeColorToString(negativeColor),
    })
    root:addFixedChild(expenses)

    local revenue = Text:new("@color{%positiveColor}{Revenue: } %revenue", {
        revenue = self.game.thePlayer.baseRevenue,
        positiveColor = dumeColorToString(positiveColor),
    })
    root:addFixedChild(revenue)

    -- Research slider
    local slider = Flex:row()

    local percentText = Text:new("@icon{beaker}: %beakerPercent%percent", {
        beakerPercent = self.game.thePlayer.beakerPercent,
        percent = "%",
    })
    slider:addFixedChild(percentText)

    local plus = Button:new(Text:new("+", {}, {baseline=dume.Baseline.Middle}), function()
        self.game.client:setEconomySettings(self.game.thePlayer.beakerPercent + 10)
    end)
    table.insert(plus.classes, "sliderButton")
    slider:addFixedChild(plus)
    local minus = Button:new(Text:new("-", {}, {baseline=dume.Baseline.Middle}), function()
        self.game.client:setEconomySettings(self.game.thePlayer.beakerPercent - 10)
    end)
    table.insert(minus.classes, "sliderButton")
    slider:addFixedChild(minus)

    local outputText = Text:new("(+%beakerRevenue / turn)", {
        beakerRevenue = self.game.thePlayer.beakerRevenue,
    })
    slider:addFixedChild(outputText)

    root:addFixedChild(slider)

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(275, 150)
    ui:createWindow("economy", Vector(0, 0), size, container, false, false)
end

function EconomyWindow:close()
    ui:deleteWindow("economy")
end

function dumeColorToString(color)
    return string.format("rgba(%d,%d,%d,%d)", color[1], color[2], color[3], color[4])
end

return Hud
