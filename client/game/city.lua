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
    for k, v in pairs(data) do
        self[k] = v
    end

    self.owner = game.players[self.ownerID]

    self.buildings = {}
    for _, buildingName in ipairs(data.buildingNames) do
        local building = registry.buildings[buildingName]
        if building == nil then print("received invalid building '" .. buildingName .. "'!") end
        table.insert(self.buildings, building)
    end

    self.populationText = cv:parseTextMarkup("@size{12}{@color{rgb(0,0,0)}{%pop}}", style.defaultTextStyle, {pop=tostring(self.population)})
    self.populationParagraph = cv:createParagraph(self.populationText, {
        alignH = dume.Align.Center,
        alignV = dume.Align.Start,
        baseline = dume.Baseline.Middle,
        lineBreaks = false,
        maxDimensions = Vector(20, math.huge)
    })

    self.cityNameText = cv:parseTextMarkup("@size{10}{@color{rgb(255,255,255)}{%name}}", style.defaultTextStyle, {name=self.name})
    self.cityNameParagraph = cv:createParagraph(self.cityNameText, {
        alignH = dume.Align.Center,
        alignV = dume.Align.Start,
        baseline = dume.Baseline.Middle,
        lineBreaks = false,
        maxDimensions = Vector(100, math.huge)
    })
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

function City:renderBubble(cv)
    -- Rounded rectangle (bubble background)
    local bubbleWidth = 100
    local bubbleHeight = 20
    cv:beginPath()
    cv:roundedRect(Vector(0, 10), Vector(bubbleWidth, bubbleHeight), 5)
    cv:linearGradient(Vector(0, 10), Vector(0, 30), bubbleColorA, bubbleColorB)
    cv:fill()

    -- Left circle
    local radius = 10
    cv:beginPath()
    cv:circle(Vector(radius - 5, radius + 10), radius)
    cv:solidColor(populationCircleColor)
    cv:fill()
    cv:solidColor(black)
    cv:strokeWidth(1.5)
    cv:stroke()

    -- Right circle
    cv:beginPath()
    cv:circle(Vector(radius - 5 + bubbleWidth, radius + 10), radius)
    cv:solidColor(buildCircleColor)
    cv:fill()
    cv:solidColor(black)
    cv:strokeWidth(1.5)
    cv:stroke()

    -- Left circle text (population)
    cv:drawParagraph(self.populationParagraph, Vector(-5, 20))

    -- TODO right circle text

    -- City name
    cv:drawParagraph(self.cityNameParagraph, Vector(0, 20))
end

function City:render(cv)
    self:renderHouses(cv)
    self:renderBubble(cv)
end

return City
