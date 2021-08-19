-- Prompts for city build tasks and research.
-- NB: prompts need to be constructed on coroutines
-- as they may request information from the server.

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
local Divider = require("widget/divider")

local style = require("ui/style")

-- Asks the user what to build in a city.
local CityBuildPrompt = {}

function CityBuildPrompt:new(game, city)
    local possibleTasks = game.client:getPossibleBuildTasks(city)
    local o = { game = game, city = city, possibleTasks = possibleTasks }
    setmetatable(o, self)
    self.__index = self

    return o
end

local function createTooltip(lines, vars, child)
    local tooltipString = ""
    for _, line in ipairs(lines) do
        tooltipString = tooltipString .. line .. "\n"
    end
    local tooltipText = Text:new(tooltipString, vars)
    table.insert(tooltipText.classes, "tooltipText")
    local tooltipContainer = Container:new(Padding:new(tooltipText, 10))
    table.insert(tooltipContainer.classes, "tooltipContainer")
    local tooltip = Tooltip:new(child, tooltipContainer)
    return tooltip
end

function CityBuildPrompt:build()
    self.game.view:animateToTilePos(self.city.pos, 0.5)

    local root = Flex:column(10)

    local title, vars
    if self.city.previousBuildTask ~= nil then
        local verb
        if self.city.previousBuildTask.kind.unit ~= nil then
            verb = "trained"
            vars = { name = registry.unitKinds[self.city.previousBuildTask.kind.unit.unitKindID].name }
        else
            verb = "constructed"
            vars = { name = self.city.previousBuildTask.kind.building.buildingName }
        end

        title = "You have " .. verb .. " a @color{%highlight}{%name} in %city. What would you like to work on next?"
    else
        vars = {}
        title = "What would you like to work on in %city?"
    end
    vars.city = self.city.name
    vars.highlight = dumeColorToString(style.default.highlightedText.defaultTextStyle.color)

    local titleText = Text:new("@size{16}{" .. title .. "}", vars)
    root:addFixedChild(titleText)

    root:addFixedChild(Divider:new(1))

    for _, possibleTask in ipairs(self.possibleTasks) do
        local name
        if possibleTask.kind.building ~= nil then
            name = possibleTask.kind.building.buildingName
        else
            name = registry.unitKinds[possibleTask.kind.unit.unitKindID].name
        end
        local entry = Text:new("@size{16}{%bullet %name    (%duration)}", {
            bullet = "•",
            name = name,
            duration = tostring(self.city:estimateTurnsToBuild(possibleTask)),
        })
        table.insert(entry.classes, "hoverableText")

        local wrapper = Clickable:new(entry, function()
            self.game.client:setCityBuildTask(self.city, possibleTask.kind)
            self.finished = true
            ui:deleteWindow("cityBuildPrompt")
        end)

        local lines
        local tooltipVars
        if possibleTask.kind.unit ~= nil then
            local unitKind = registry.unitKinds[possibleTask.kind.unit.unitKindID]

            tooltipVars = {
                name = unitKind.name,
                cost = unitKind.cost,
                movement = unitKind.movement,
                strength = unitKind.strength,
                bullet = "•",
                percent = "%",
                carryUnitsCapacity = unitKind.carryUnitsCapacity,
                category = unitKind.category,
            }

            lines = {
                "%name (%category units)",
                "%cost @icon{hammer}",
                "Movement: %movement",
                "Strength: %strength"
            }

            for _, capability in ipairs(unitKind.capabilities or {}) do
                if capability == "found_city" then
                    lines[#lines + 1] = "%bullet Can found a city"
                elseif capability == "do_work" then
                    lines[#lines + 1] = "%bullet Can improve terrain"
                elseif capability == "carry_units" then
                    lines[#lines + 1] = "%bullet Can ferry units (capacity: %carryUnitsCapacity)"
                end
            end

            for _, bonus in ipairs(unitKind.combatBonuses or {}) do
                local line = "%bullet +" .. tostring(bonus.bonusPercent) .. "%percent "
                if bonus.onlyOnDefense then
                    line = line .. "defense "
                elseif bonus.onlyOnAttack then
                    line = line .. "attack "
                end

                if bonus.type == "againstUnit" then
                    line = line .. " against " .. registry.unitKinds[bonus.unit].name
                elseif bonus.type == "againstUnitCategory" then
                    line = line .. " against " .. bonus.category .. " units"
                else
                    line = line .. " when in city"
                end

                lines[#lines + 1] = line
            end
        else
            local building = registry.buildings[possibleTask.kind.building.buildingName]
            lines = getBuildingTooltip(building)
            tooltipVars = {
                percent = "%",
                bullet = "•"
            }
        end

        local tooltip = createTooltip(lines, tooltipVars, wrapper)
        root:addFixedChild(tooltip)
    end

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(300, 500)
    ui:createWindow("cityBuildPrompt", function(screenSize)
        return {
            pos = Vector(screenSize.x - size.x - 10, 120),
            size = size,
        }
    end, container, 5)
end

-- Asks the user what to research.
local ResearchPrompt = {}

function ResearchPrompt:new(game)
    local possibleTechNames = game.client:getPossibleTechs()
    local possibleTechs = {}
    for _, techName in ipairs(possibleTechNames) do
        possibleTechs[#possibleTechs + 1] = registry.techs[techName] or error("received invalid tech " .. techName)
    end
    local o = {
        game = game,
        possibleTechs = possibleTechs,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function ResearchPrompt:build()
    local root = Flex:column(10)

    local title = Text:new("@size{16}{What would you like to research next?}")
    table.insert(title.classes, "highlightedText")
    root:addFixedChild(title)

    root:addFixedChild(Divider:new(1))

    for _, tech in ipairs(self.possibleTechs) do
        local entry = Text:new("@size{16}{%bullet %name    (%duration)}", {
            bullet = "•",
            name = tech.name,
            duration = maybeInfinity(self.game.thePlayer:estimateResearchTurns(tech)),
        })
        table.insert(entry.classes, "hoverableText")

        local wrapper = Clickable:new(entry, function()
            self.game.client:setResearch(tech)
            ui:deleteWindow("researchPrompt")
            self.finished = true
        end)

        -- Tooltip
        local vars = {
            cost = tostring(tech.cost),
            name = tech.name,
            bullet = "•"
        }

        local lines = {
            "%name",
            "%cost @icon{beaker}",
        }

        -- TODO/PERF: we could precompute this data instead of searching
        -- the entire registry.

        for _, unit in pairs(registry.unitKinds) do
            if type(unit) == "table" then
                for _, t in ipairs(unit.techs) do
                    if t == tech.name then
                        lines[#lines + 1] = "%bullet Can train " .. article(unit.name) .. " " .. unit.name
                    end
                end
            end
        end

        for _, building in pairs(registry.buildings) do
            if type(building) == "table" then
                for _, t in ipairs(building.techs) do
                    if t == tech.name then
                        lines[#lines + 1] = "%bullet Can build " .. article(building.name) .. " " .. building.name
                    end
                end
            end
        end

        for _, improvement in ipairs(tech.unlocksImprovements or {}) do
            lines[#lines + 1] = "%bullet Can build " .. article(improvement) .. " " .. improvement
        end

        for _, resource in pairs(registry.resources) do
            if type(resource) == "table" then
                if resource.revealedBy == tech.name then
                    lines[#lines + 1] = "%bullet Reveals " .. resource.name
                end
            end
        end

        local leadsTo = {}
        for _, t in pairs(registry.techs) do
            if type(t) == "table" then
                for _, prerequisite in ipairs(t.prerequisites or {}) do
                    if prerequisite == tech.name then
                        leadsTo[#leadsTo + 1] = t.name
                        break
                    end
                end
            end
        end

        local leadsToLine = "%bullet Leads to "
        for i, s in ipairs(leadsTo) do
            leadsToLine = leadsToLine .. s
            if i ~= #leadsTo then
                leadsToLine = leadsToLine .. ", "
            end
        end
        if #leadsTo > 0 then
            lines[#lines + 1] = leadsToLine
        end

        local tooltip = createTooltip(lines, vars, wrapper)
        root:addFixedChild(tooltip)
    end

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(400, 400)
    ui:createWindow("researchPrompt", function(screenSize)
        return {
            pos = Vector(screenSize.x / 2 - size.x / 2, screenSize.y / 2 - size.y / 2 - 100),
            size = size,
        }
    end, container, 5)
end

-- A queue of prompts to display at the start
-- of the turn.
local PromptQueue = {}

function PromptQueue:new()
    local o = { prompts = {} }
    setmetatable(o, self)
    self.__index = self
    return o
end

function PromptQueue:push(prompt)
    self.prompts[#self.prompts + 1] = prompt
    if #self.prompts == 1 then
        prompt:build()
    end
end

function PromptQueue:pop()
    table.remove(self.prompts, 1)
    if #self.prompts > 0 then
        self.prompts[1]:build()
    end
end

function PromptQueue:tick()
    while #self.prompts > 0 and self.prompts[1].finished do
        self:pop()
    end
end

-- nullable
function PromptQueue:getCurrentPrompt()
    return self.prompts[1]
end

-- Returns the indefinite article to use before the
-- given noun: either `a` or `an` depending on whether
-- the noun starts with a vowel.
function article(noun)
    local firstChar = string.sub(noun, 1, 1)

    local vowels = {
        a = true,
        e = true,
        i = true,
        o = true,
        u = true,
    }

    if vowels[string.lower(firstChar)] then
        return "an"
    else
        return "a"
    end
end

return {
    CityBuildPrompt = CityBuildPrompt,
    ResearchPrompt = ResearchPrompt,
    PromptQueue = PromptQueue,
}
