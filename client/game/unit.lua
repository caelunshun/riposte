local Unit = {}

local dume = require("dume")
local style = require("ui/style")
local Vector = require("brinevector")

function Unit:new(game)
    local o = { game = game }
    setmetatable(o, self)
    self.__index = self
    return o
end

-- Updates the unit with data received in an UpdateUnit packet.
function Unit:updateData(data, game)
    local newPos = Vector(data.pos.x, data.pos.y)
    if newPos ~= self.pos and self.pos ~= nil then
        self.moveStartTime = time
        self.previousPos = self.pos
    end

    for k, v in pairs(data) do
        self[k] = v
    end

    self.pos = newPos

    self.kind = registry.unitKinds[self.kindID]
    if self.kind == nil then print("received invalid unit kind " .. self.kindID .. "!") end

    self.nameText = cv:parseTextMarkup("@bold{@size{14}{@color{rgb(0,0,0)}{%name}}}", style.default.text.defaultTextStyle, {name=self.kind.name})
    self.nameParagraph = cv:createParagraph(self.nameText, {
        alignH = dume.Align.Center,
        alignV = dume.Align.Start,
        lineBreaks = false,
        maxDimensions = Vector(100, 100),
        baseline = dume.Baseline.Top,
    })

    self.owner = game.players[self.ownerID]
    if self.owner == nil then error("invalid unit owner ID") end
end

function Unit:getOwner(game)
    return game.players[self.ownerID]
end

function Unit:render(cv, game)
    -- Movement interpolation
    local translation = Vector(0, 0)
    if self.moveStartTime ~= nil then
        -- integral of cosine velocity function for interpolation
        local timeSinceMove = time - self.moveStartTime
        local f = 1
        local vel = 1500
        local pos = 0
        if timeSinceMove <= f then
            pos = vel * -math.cos(timeSinceMove * f * math.pi) + vel
        else
            pos = (vel * -math.cos(f * f / 2 * math.pi) + vel) + vel * (timeSinceMove - f)
        end

        local posA = Vector(0, 0)
        local posB = (self.pos - self.previousPos) % Vector(100, 100)
        local dist = (posB - posA).length

        pos = math.clamp(pos, 0, dist)

        local ray = (posA - posB).normalized
        translation = -(posB + (ray * pos))
    end

    cv:translate(translation)

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

    cv:translate(-translation)
end

-- Attempts to move the unit.
--
-- Note that the position update is delayed until
-- we receive a response packet from the server.
function Unit:moveTo(newPos)
    self.game.client:moveUnit(self, newPos)
end

return Unit
