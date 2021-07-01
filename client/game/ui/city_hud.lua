local dume = require("dume")
local Vector = require("brinevector")

local Flex = require("widget/flex")
local Container = require("widget/container")
local Text = require("widget/text")
local ProgressBar = require("widget/progress_bar")
local Spacer = require("widget/spacer")
local Divider = require("widget/divider")
local Padding = require("widget/padding")

local UiUtils = require("ui/utils")

local leftWindowSize = Vector(200, 600)
local topWindowSize = Vector(440, 150)

-- GUI opened when a city is double-clicked.
local CityHud = {}

local LeftWindow = {}
local TopWindow = {}
local WorkedTilesHud = {}

function CityHud:new(game, city)
    local o = {
        game = game,
        city = city,
    }
    setmetatable(o, self)
    self.__index = self

    o.windows = {
        LeftWindow:new(game, o),
        TopWindow:new(game, o),
    }

    o.workedTileHud = WorkedTilesHud:new(game, o)

    local handlerID = game.eventBus:registerHandler("cityUpdated", function(c)
        if c == o.city then
            o:rebuildWindows()
        end
    end)
    o.handlerID = handlerID

    return o
end

function CityHud:rebuildWindows()
    for _, window in ipairs(self.windows) do
        window:rebuild()
    end
end

function CityHud:close()
    for _, window in ipairs(self.windows) do
        window:close()
    end
end

function LeftWindow:new(game, cityHud)
    local o = {
        game = game,
        cityHud = cityHud,
        city = cityHud.city,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function LeftWindow:rebuild()
    local root = Flex:column()

    root:addFixedChild(Text:new("@size{18}{Buildings}"))
    root:addFixedChild(Divider:new(1))
    root:addFixedChild(Spacer:new(dume.Axis.Vertical, 50))

    for _, building in ipairs(self.city.buildings) do
        root:addFixedChild(
                Container:new(
                        Padding:new(
                                Text:new(
                                    "%bullet %buildingName",
                                        {
                                            bullet = "•",
                                            buildingName = building.name,
                                        }
                                ),
                                10
                        )
                )
        )
    end

    ui:createWindow("cityHudLeft", Vector(0, 100), leftWindowSize, UiUtils.createWindowContainer(root))
end

function LeftWindow:close()
    ui:deleteWindow("cityHudLeft")
end

function TopWindow:new(game, cityHud)
    local o = {
        game = game,
        cityHud = cityHud,
        city = cityHud.city,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function TopWindow:rebuild()
    local root = Flex:column()
    root:setCrossAlign(dume.Align.Center)

    -- Title
    root:addFixedChild(Text:new("@size{18}{%cityName}", {cityName = self.city.name}))
    root:addFixedChild(Text:new("Population: %population", {population = tostring(self.city.population)}))
    if self.city.isCapital then
        root:addFixedChild(Text:new("Capital"))
    end

    -- Population growth progress bar
    local progressBarSize = Vector(topWindowSize.x - 50, 20)
    do
        local progress = self.city.storedFood / self.city.foodNeededForGrowth
        local foodSurplus = self.city.yield.food - self.city.consumedFood
        local predictedProgress = (self.city.storedFood + foodSurplus) / self.city.foodNeededForGrowth

        local populationSubtitle
        if foodSurplus > 0 then
            local projectedTurns = math.ceil((self.city.foodNeededForGrowth - self.city.storedFood) / foodSurplus)
            populationSubtitle = "Growing (" .. projectedTurns .. " " .. maybePlural("turn", projectedTurns) .. ")"
        elseif foodSurplus < 0 then
            populationSubtitle = "STARVATION!"
        else
            populationSubtitle = "Stagnant"
        end

        local bar = ProgressBar:new(
                progressBarSize,
                function() return progress  end,
                function() return predictedProgress  end,
                Text:new(populationSubtitle, {}, {alignH = dume.Align.Center})
        )
        table.insert(bar.classes, "populationProgressBar")
        root:addFixedChild(bar)
    end

    -- Production progress bar
    local task = self.city.buildTask
    if task ~= nil then
        local progress = task.progress / task.cost
        local predictedProgress = (task.progress + self.city.yield.hammers) / task.cost

        local name
        if task.kind.unit ~= nil then
            name = registry.unitKinds[task.kind.unit.unitKindID].name
        else
            name = task.kind.building.buildingName
        end

        local turns = self.city:estimateTurnsToBuild(task)
        local productionSubtitle = string.format("%s (%d %s)", name, turns, maybePlural("turn", turns))

        local bar = ProgressBar:new(
                progressBarSize,
                function() return progress  end,
                function() return predictedProgress  end,
                Text:new(productionSubtitle, {}, {alignH = dume.Align.Center})
        )
        table.insert(bar.classes, "productionProgressBar")
        root:addFixedChild(bar)
    end

    ui:createWindow("cityHudTop", Vector(cv:getWidth() / 2 - topWindowSize.x / 2, 10), topWindowSize, UiUtils.createWindowContainer(root))
end

function TopWindow:close()
    ui:deleteWindow("cityHudTop")
end

function WorkedTilesHud:new(game, cityHud)
    local o = {
        game = game,
        cityHud = cityHud,
        city = cityHud.city,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function WorkedTilesHud:render(cv)
    self.game.view:applyZoom(cv)
    for _, workedTilePos in ipairs(self.city.workedTiles) do
        local pos = self.game.view:getScreenOffsetForTilePos(workedTilePos) * self.game.view.zoomFactor
        cv:translate(pos)

        cv:beginPath()
        cv:circle(Vector(50, 50), 50)
        cv:strokeWidth(2)
        cv:solidColor(dume.rgb(255, 255, 255))
        cv:stroke()

        cv:translate(-pos)
    end
    cv:resetTransform()
end

function WorkedTilesHud:handleEvent(event)
    -- Toggle worked tiles.
    if event.type == dume.EventType.MouseClick and event.mouse == dume.Mouse.Left
        and event.action == dume.Action.Press then
        local clickedTilePos = self.game.view:getTilePosForScreenOffset(event.pos)

        if clickedTilePos ~= self.city.pos then
            self.game.client:setTileWorkedManually(self.city, clickedTilePos, not self.city:isTileWorked(clickedTilePos))
        end
    end
end

function CityHud:handleEvent(event)
    if event.type == dume.EventType.Key and event.action == dume.Action.Press then
        if event.key == dume.Key.Escape then
            self:close()
        end
    end

    self.workedTileHud:handleEvent(event)
end

function CityHud:render(cv, time)
    self.workedTileHud:render(cv, time)
end

function CityHud:close()
    for _, window in ipairs(self.windows) do
        window:close()
    end
    self.closed = true
    self.game.eventBus:deregisterHandler(self.handlerID)
end

return CityHud
