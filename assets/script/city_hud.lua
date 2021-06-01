-- The city HUD. Allows updating worked tiles, viewing building
-- and yield information, etc.

local buildingsSize = Vec2.new(200, 600)
local infoBarSize = Vec2.new(440, 150)

local CityHUD = {}
local BuildingsWindow = {}
local InfoBarWindow = {}
local WorkedTilesWindow = {}

local currentCityHUD = nil

function CityHUD.new(self, city)
    local o = { city = city }

    o.buildingsWindow = BuildingsWindow:new(city)
    o.infoBarWindow = InfoBarWindow:new(city)
    o.workedTilesWindow = WorkedTilesWindow:new(city)

    hud:openWindow(o.buildingsWindow)
    hud:openWindow(o.infoBarWindow)
    hud:openWindow(o.workedTilesWindow)

    -- take UI control
    hud:takeFullControl(true)

    setmetatable(o, self)
    self.__index = self

    currentCityHUD = o

    return o
end

function CityHUD.close(self)
    self.buildingsWindow.close = true
    self.infoBarWindow.close = true
    self.workedTilesWindow.close = true

    -- concede UI control
    hud:takeFullControl(false)

    currentCityHUD = nil
end

function BuildingsWindow.new(self, city)
    local o = { city = city, close = false }

    setmetatable(o, self)
    self.__index = self

    return o
end

function BuildingsWindow.paint(self, ui)
    ui:beginWindow("cityBuildings", 0, 100, buildingsSize.x, buildingsSize.y)

    ui:layoutDynamic(50, 1)

    ui:label("Buildings")

    local buildings = self.city:getBuildings()
    for i = 1, #buildings do
        local building = buildings[i]
        ui:label(string.format("* %s", building.name))
    end

    ui:endWindow()
end

function BuildingsWindow.shouldClose(self)
    return self.close
end

local function paintProgressBar(cv, posX, posY, progress, projectedProgress, progressCol, projectedProgressCol, text)
    local length = 400
    local endX = posX + length

    if projectedProgress > 1 then projectedProgress = 1 end
    if progress > 1 then progress = 1 end

    cv:beginPath()
    cv:rect(posX, posY, 400, 20)
    cv:fillColor({80, 80, 80})
    cv:fill()
    cv:strokeColor({0, 0, 0})
    cv:strokeWidth(0.5)
    cv:stroke()

    cv:beginPath()
    cv:rect(posX, posY, length * progress, 20)
    cv:fillColor(progressCol)
    cv:fill()

    cv:beginPath()
    cv:rect(posX + 400 * progress, posY, length * (projectedProgress - progress), 20)
    cv:fillColor(projectedProgressCol)
    cv:fill()

    cv:fontSize(12)
    cv:fillColor({255, 255, 255})
    cv:textFormat(TextBaseline.Middle, TextAlign.Center)
    cv:text((endX + posX) / 2, posY + 10, text)
end

function InfoBarWindow.new(self, city)
    local o = { city = city, close = false }

    setmetatable(o, self)
    self.__index = self

    return o
end

-- Uses the canvas directly instead of Nuklear
-- for more control.
function InfoBarWindow.paint(self, ui, cv)
    local windowSize = game:getCursor():getWindowSize()
    local posX = (windowSize.x - infoBarSize.x) / 2
    local posY = 10;

    local endX = posX + infoBarSize.x
    local endY = posY + infoBarSize.y

    local centerX = (posX + endX) / 2
    local centerY = (posY + endY) / 2

    local padding = 20

    -- Background
    cv:beginPath()
    cv:rect(posX, posY, infoBarSize.x, infoBarSize.y)
    cv:fillColor({50, 50, 50})
    cv:fill()

    -- Title
    cv:fontSize(15)
    cv:textFormat(TextBaseline.Top, TextAlign.Center)
    cv:fillColor({255, 255, 255})
    cv:text(centerX, posY + 5, self.city:getName())

    cv:fontSize(12)
    cv:text(centerX, posY + 25, string.format("Population: %d", self.city:getPopulation()))
    if self.city:isCapital() then
        cv:text(centerX, posY + 40, "Capital")
    end

    -- Population progress bar
    local storedFood = self.city:getStoredFood()
    local neededFood = self.city:getFoodNeededForGrowth()
    local foodSurplus = self.city:computeYield().food - self.city:getConsumedFood()
    local progress = storedFood / neededFood
    local projectedProgress = (storedFood + foodSurplus) / neededFood

    local projectedProgressCol = nil
    local text = nil
    if foodSurplus > 0 then
        local projectedTurns = math.ceil((neededFood - storedFood) / foodSurplus)
        text = string.format("Growing (%d turn", projectedTurns)
        if projectedTurns ~= 1 then text = text .. "s" end
        text = text .. ")"
        projectedProgressCol = {185, 112, 0}
    elseif foodSurplus < 0 then
        text = "STARVATION!"
        projectedProgressCol = {209, 65, 36}
    else
        projectedProgressCol = {0, 0, 0}
        text = "Stagnant"
    end
    paintProgressBar(cv, posX + padding, posY + padding + 40, progress, projectedProgress, {237, 155, 51}, projectedProgressCol, text)

    -- Production progress bar
    local task = self.city:getBuildTask()
    if task ~= nil then
        local turnsLeft = self.city:estimateTurnsForCompletion(task)
        local progress = task:getProgress() / task:getCost()
        local projectedProgress = (task:getProgress() + self.city:computeYield().hammers) / task:getCost()
        local text = string.format("%s (%d turn", task:getName(), turnsLeft)
        if turnsLeft ~= 1 then text = text .. "s" end
        text = text .. ")"
        paintProgressBar(cv, posX + padding, posY + padding + 80, progress, projectedProgress, progressColor, projectedProgressColor, text)
    end
end

function InfoBarWindow.shouldClose(self)
    return self.close
end

function WorkedTilesWindow.new(self, city)
    local o = { close = false, city = city }

    setmetatable(o, self)
    self.__index = self

    return o
end

function WorkedTilesWindow.paint(self, ui, cv)
    cv:applyZoom()
    local workedTiles = self.city:getWorkedTiles()
    for i = 1, #workedTiles do
        local tilePos = workedTiles[i]

        local offset = game:getScreenOffset(tilePos)

        cv:beginPath()
        cv:circle(offset.x + 50, offset.y + 50, 50)
        cv:strokeColor({255, 255, 255})
        cv:strokeWidth(2)
        cv:stroke()
    end
    cv:removeZoom()
end

function WorkedTilesWindow.shouldClose(self)
    return self.close
end

function WorkedTilesWindow.toggleManualWork(self, pos)
    if not self.city:canWorkTile(pos) then
        return
    end

    if pos == self.city:getPos() then
        return
    end

    -- Determine whether to disable or enable
    local manualWorked = self.city:getManualWorkedTiles()
    local alreadyWorked = false
    for i = 0, #manualWorked do
        if manualWorked[i] == pos then
            alreadyWorked = true
            break
        end
    end


    if alreadyWorked then
        self.city:removeManualWorkedTile(pos)
    else
        self.city:addManualWorkedTile(pos)
    end
    self.city:updateWorkedTiles()
end

registerDoubleClickHandler(function(pos)
    if currentCityHUD ~= nil then return end

    local city = game:getCityAtLocation(pos)
    if city ~= nil then
        if city:getOwner() ~= game:getThePlayer() then return end
        currentCityHUD = CityHUD:new(city)
    end
end)

engine:registerEventHandler("onKeyPressed", function(key)
    if key == Key.Escape and currentCityHUD ~= nil then
        currentCityHUD:close()
    end
end)

engine:registerEventHandler("onPosClicked", function(pos)
    -- Toggle worked tiles if needed.
    if currentCityHUD == nil then return end

    currentCityHUD.workedTilesWindow:toggleManualWork(pos)
end)
