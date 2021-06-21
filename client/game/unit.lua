local Unit = {}

local dume = require("dume")
local style = require("ui/style")
local Vector = require("brinevector")

function Unit:new(data)
    data.kind = registry.unitKinds[data.kindID]
    if data.kind == nil then print("received invalid unit kind " .. data.kindID .. "!") end

    data.nameText = cv:parseTextMarkup("@bold{@size{14}{@color{rgb(0,0,0)}{%name}}}", style.defaultTextStyle, {name=data.kind.name})
    data.nameParagraph = cv:createParagraph(data.nameText, {
        alignH = dume.Align.Center,
        alignV = dume.Align.Start,
        lineBreaks = false,
        maxDimensions = Vector(100, 100),
        baseline = dume.Baseline.Top,
    })

    data.pos = Vector(data.pos.x, data.pos.y)

    setmetatable(data, self)
    self.__index = self
    return data
end

function Unit:getOwner(game)
    return game.players[self.ownerID]
end

function Unit:render(cv, game)
    -- Unit icon
    local spriteID = "texture/unit/" .. self.kind.id
    local size = 60
    cv:drawSprite(spriteID, Vector(50 - size / 2, 50 - size / 2), size)

    -- Unit name
    cv:drawParagraph(self.nameParagraph, Vector(0, 80))

    -- Unit nationality rectangle
    local owner = self:getOwner(game)
    cv:beginPath()
    cv:rect(Vector(70, 35), Vector(20, 30))
    cv:solidColor(owner.civ.color)
    cv:fill()
end

return Unit
