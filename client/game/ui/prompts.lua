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

-- Asks the user what to build in a city.
local CityBuildPrompt = {}

function CityBuildPrompt:new(game, city)
    local possibleTasks = game.client:getPossibleBuildTasks(city)
    local o = { game = game, city = city, possibleTasks = possibleTasks }
    setmetatable(o, self)
    self.__index = self
    return o
end

function CityBuildPrompt:build()
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

        title = "You have constructed a %name in %city. What would you like to work on next?"
    else
        vars = {}
        title = "What would you like to work on in %city?"
    end
    vars.city = self.city.name

    local titleText = Text:new("@size{16}{" .. title .. "}", vars)
    table.insert(titleText.classes, "highlightedText")
    root:addFixedChild(titleText)

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
            lines = {
                "%name",
                "%cost @icon{hammer}"
            }
            tooltipVars = {
                name = building.name,
                cost = building.cost,
            }
        end

        local tooltipString = ""
        for _, line in ipairs(lines) do
            tooltipString = tooltipString .. line .. "\n"
        end
        local tooltipText = Text:new(tooltipString, tooltipVars)
        table.insert(tooltipText.classes, "tooltipText")
        local tooltipContainer = Container:new(Padding:new(tooltipText, 10))
        table.insert(tooltipContainer.classes, "tooltipContainer")
        local tooltip = Tooltip:new(wrapper, tooltipContainer)

        root:addFixedChild(tooltip)
    end

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(300, 500)
    ui:createWindow("cityBuildPrompt", Vector(cv:getWidth() - size.x - 10, 50), size, container)
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

    for _, tech in ipairs(self.possibleTechs) do
        local entry = Text:new("@size{16}{%bullet %name    (%duration)}", {
            bullet = "•",
            name = tech.name,
            duration = self.game.thePlayer:estimateResearchTurns(tech),
        })
        table.insert(entry.classes, "hoverableText")

        local wrapper = Clickable:new(entry, function()
            self.game.client:setResearch(tech)
            ui:deleteWindow("researchPrompt")
            self.finished = true
        end)

        root:addFixedChild(wrapper)
    end

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(400, 400)
    ui:createWindow("researchPrompt", Vector(cv:getWidth() / 2 - size.x / 2, cv:getHeight() / 2 - size.y / 2), size, container)
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

return {
    CityBuildPrompt = CityBuildPrompt,
    ResearchPrompt = ResearchPrompt,
    PromptQueue = PromptQueue,
}
