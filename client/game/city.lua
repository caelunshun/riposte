local City = {}

local dume = require("dume")
local Vector = require("brinevector")
local style = require("ui/style")

function City:new()
    local o = {}
    setmetatable(o, self)
    self.__index = self
    return o
end

-- Updates the city with data received from the server
-- in an UpdateCity packet.
function City:updateData(data, game)
    if self.buildTask ~= nil and data.buildTask == nil then
        self.previousBuildTask = self.buildTask
    end

    if data.buildTask == nil then
        self.buildTask = nil
    end

    local previousOwner = self.owner

    for k, v in pairs(data) do
        self[k] = v
    end

    self.owner = game.players[self.ownerID]
    if self.owner == nil then print("city '" .. self.name .. "' has invalid owner!") end

    if self.owner ~= previousOwner then
        self.previousBuildTask = nil
    end

    self.buildings = {}
    for _, buildingName in ipairs(data.buildingNames) do
        local building = registry.buildings[buildingName]
        if building == nil then print("received invalid building '" .. buildingName .. "'!") end
        table.insert(self.buildings, building)
    end

    self.populationText = cv:parseTextMarkup("@size{14}{@color{rgb(0,0,0)}{%pop}}", style.default.text.defaultTextStyle, {pop=tostring(self.population)})
    self.populationParagraph = cv:createParagraph(self.populationText, {
        alignH = dume.Align.Center,
        alignV = dume.Align.Start,
        baseline = dume.Baseline.Middle,
        lineBreaks = false,
        maxDimensions = Vector(20, math.huge)
    })

    self.cityNameText = cv:parseTextMarkup("@size{10}{@color{rgb(255,255,255)}{%name}}", style.default.text.defaultTextStyle, {name=self.name})
    self.cityNameParagraph = cv:createParagraph(self.cityNameText, {
        alignH = dume.Align.Center,
        alignV = dume.Align.Start,
        baseline = dume.Baseline.Middle,
        lineBreaks = false,
        maxDimensions = Vector(100, math.huge)
    })

    if self.buildTask ~= nil and self.buildTask.kind.building ~= nil then
        local c = string.sub(self.buildTask.kind.building.buildingName, 1, 1)
        local text = cv:parseTextMarkup("@bold{@size{18}{@color{rgb(0, 0, 0)}{%c}}}", style.default.text.defaultTextStyle, {c=c})
        self.buildTaskParagraph = cv:createParagraph(text, {
            alignH = dume.Align.Center,
            alignV = dume.Align.Center,
            baseline = dume.Baseline.Top,
            lineBreaks = false,
            maxDimensions = Vector(20, 20)
        })
    end
end

function City:estimateTurnsToBuild(buildTask)
    return math.ceil(
            (buildTask.cost - buildTask.progress)
            / self.yield.hammers
    )
end

function City:isTileWorked(tilePos)
    for _, workedTile in ipairs(self.workedTiles) do
        if workedTile.x == tilePos.x and workedTile.y == tilePos.y then
            return true
        end
    end
    return false
end

local function drawFivePointStar(cv, center, outerRadius, innerRadius)
    local angleStep = math.pi * 2 / 5

    for i=1,5 do
        local outerTheta = angleStep * (i - 1) - math.pi / 2
        local innerTheta = angleStep * (i - 1 / 2) - math.pi / 2

        local outerPos = Vector(math.cos(outerTheta), math.sin(outerTheta)) * outerRadius + center
        local innerPos = Vector(math.cos(innerTheta), math.sin(innerTheta)) * innerRadius + center

        if i == 1 then
            cv:moveTo(outerPos)
        else
            cv:lineTo(outerPos)
        end
        cv:lineTo(innerPos)
    end

    -- close the path
    cv:lineTo(center - Vector(0, outerRadius))
end

local numHouses = 3
local housePositions = {
    Vector(20, 25),
    Vector(50, 25),
    Vector(25, 30),
}
local houseScales = {
    25,
    25,
    55,
}

function City:renderHouses(cv)
    for i=1,numHouses do
        local housePos = housePositions[i]
        local houseScale = houseScales[i]

        cv:drawSprite("icon/house", housePos, houseScale / 1.424)
    end
end

local bubbleColorA = dume.rgb(61, 61, 62, 180)
local bubbleColorB = dume.rgb(40, 40, 41, 180)
local populationCircleColor = dume.rgb(182, 207, 174)
local black = dume.rgb(0, 0, 0)
local buildCircleColor = dume.rgb(244, 195, 204)

local function renderProgressBar(cv, pos, size, progress, projectedProgress, progressColor, projectedProgressColor)
    projectedProgress = math.clamp(projectedProgress, 0, 1)

    cv:beginPath()
    cv:rect(pos, Vector(size.x * progress, size.y))
    cv:solidColor(progressColor)
    cv:fill()

    cv:beginPath()
    cv:rect(pos + Vector(size.x * progress, 0), Vector(size.x * (projectedProgress - progress), size.y))
    cv:solidColor(projectedProgressColor)
    cv:fill()
end

function City:renderBubble(cv)
    -- Rounded rectangle (bubble background)
    local bubbleWidth = 100
    local bubbleHeight = 20
    cv:beginPath()
    cv:roundedRect(Vector(0, 10), Vector(bubbleWidth, bubbleHeight), 5)
    cv:linearGradient(Vector(0, 10), Vector(0, 30), bubbleColorA, bubbleColorB)
    cv:fill()

    -- Production progress bar
    if self.buildTask ~= nil then
        local progress = self.buildTask.progress / self.buildTask.cost
        local projectedProgress = (self.buildTask.progress + self.yield.hammers) / self.buildTask.cost
        renderProgressBar(cv, Vector(0, 20), Vector(bubbleWidth, bubbleHeight / 2), progress, projectedProgress,
            style.default.productionProgressBar.progressColor, style.default.productionProgressBar.positivePredictedProgressColor)
    end

    -- Population growth population bar
    local progress = self.storedFood / self.foodNeededForGrowth
    local projectedProgress = (self.storedFood + self.yield.food - self.consumedFood) / self.foodNeededForGrowth
    renderProgressBar(cv, Vector(0, 10), Vector(bubbleWidth, bubbleHeight / 2), progress, projectedProgress,
        style.default.populationProgressBar.progressColor, style.default.populationProgressBar.positivePredictedProgressColor)

    -- Left circle, or five-point start if this is the capital
    local radius = 10
    local center = Vector(radius - 5, radius + 10)

    cv:beginPath()
    if self.isCapital then
        drawFivePointStar(cv, center, 18, 8)
    else
        cv:circle(center, radius)
    end
    cv:solidColor(populationCircleColor)
    cv:fill()
    cv:solidColor(black)
    cv:strokeWidth(1.5)
    cv:stroke()

    -- Right circle
    cv:beginPath()
    cv:circle(Vector(-radius + 5 + bubbleWidth, radius + 10), radius)
    cv:solidColor(buildCircleColor)
    cv:fill()
    cv:solidColor(black)
    cv:strokeWidth(1.5)
    cv:stroke()

    -- Left circle text (population)
    cv:drawParagraph(self.populationParagraph, Vector(-5, 20))

    -- Right circle overlay, depending on the build task:
    -- * unit - unit head icon
    -- * building - first letter of building name (TODO: we should have icons for these)
    if self.buildTask ~= nil then
        if self.buildTask.kind.unit ~= nil then
            cv:drawSprite("icon/unit_head/" .. self.buildTask.kind.unit.unitKindID, Vector(-radius * 2 + 5 + bubbleWidth, 10), radius * 2)
        else
            cv:drawParagraph(self.buildTaskParagraph, Vector(-radius * 2 + 5 + bubbleWidth, 10))
        end
    end

    -- City name
    cv:drawParagraph(self.cityNameParagraph, Vector(0, 20))
end

function City:render(cv)
    self:renderHouses(cv)
    self:renderBubble(cv)
end

return City
