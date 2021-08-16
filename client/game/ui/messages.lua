-- Keeps track of and displays game messages.
local Messages = {}

local Vector = require("brinevector")
local dume = require("dume")
local style = require("ui/style")

local messageColorBad = "rgb(220, 68, 5)"
local messageColorGood = "rgb(67, 176, 42)"

local messageDisplayTime = 6
local width = 500
local padding = 5
local spacing = 5

function Messages:new()
    local o = {
        messages = {},
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Messages:onCombatFinished(game, combat)
    local winner
    if combat.attackerWon then winner = combat.attacker
    else winner = combat.defender
    end

    local ourUnit, enemyUnit
    if combat.defender.owner == game.thePlayer then ourUnit = combat.defender; enemyUnit = combat.attacker
    else ourUnit = combat.attacker; enemyUnit = combat.defender
    end

    local ourUnitName = ourUnit.kind.name
    local enemyUnitName = enemyUnit.kind.name
    local article = article(enemyUnit.owner.civ.adjective)
    local enemy = enemyUnit.owner.civ.adjective

    local action
    if ourUnit == combat.attacker then action = "attacking"
    else action = "defending against"
    end

    if winner == ourUnit then
        self:push("@color{%color}{Your %ourUnit has defeated %article %enemy %enemyUnit!}", {
            color = messageColorGood,
            ourUnit = ourUnitName,
            article = article,
            enemy = enemy,
            enemyUnit = enemyUnitName,
        })
        playSound("sound/event/combat_victory", 0.4)
    else
        self:push("@color{%color}{Your %ourUnit has died while %action %article %enemy %enemyUnit!}", {
            color = messageColorBad,
            ourUnit = ourUnitName,
            article = article,
            enemy = enemy,
            enemyUnit = enemyUnitName,
            action = action,
        })
        playSound("sound/event/combat_defeat", 0.4)
    end
end

function Messages:onCityCaptured(event)
    self:push("@color{%color}{%city has been captured by the %capturer!}", {
        color = messageColorBad,
        city = event.city.name,
        capturer = event.capturer.civ.name,
    })
end

function Messages:onWarDeclared(event)
    self:push("@color{%color}{@bold{%declarer has declared war on %declared!}}", {
        color = messageColorBad,
        declarer = event.declarer.username,
        declared = event.declared.username,
    })
end

function Messages:onUnitUnderAttack(unit)
    self:push("@color{%color}{Your %unit is under attack!}", {
        color = messageColorBad,
        unit = unit.kind.name,
    })
end

function Messages:push(messageMarkup, vars)
    local text = cv:parseTextMarkup(messageMarkup, style.default.text.defaultTextStyle, vars)
    local paragraph = cv:createParagraph(text, {
        alignH = dume.Align.Center,
        alignV = dume.Align.Start,
        baseline = dume.Baseline.Top,
        lineBreaks = false,
        maxDimensions = Vector(width - padding * 2, math.huge),
    })
    self.messages[#self.messages + 1] = {
        paragraph = paragraph,
        timeLeft = messageDisplayTime,
    }
end

function Messages:tick(dt)
    if #self.messages == 0 then return end

    self.messages[1].timeLeft = self.messages[1].timeLeft - dt
    if self.messages[1].timeLeft <= 0 then
        table.remove(self.messages, 1)
    end
end

function Messages:render(cv)
    if #self.messages == 0 then return end

    local height = 0
    for _, msg in ipairs(self.messages) do
        height = height + cv:getParagraphHeight(msg.paragraph) + spacing
    end

    local offset = Vector(cv:getWidth() / 2 - width / 2, 60)

    -- Background rectangle
    cv:beginPath()
    cv:rect(offset, Vector(width, height + padding * 2))
    cv:solidColor(dume.rgb(0, 0, 0, 240))
    cv:fill()

    -- Text
    local cursor = padding
    for _, msg in ipairs(self.messages) do
        cv:drawParagraph(msg.paragraph, Vector(offset.x + padding, offset.y + cursor))
        cursor = cursor + cv:getParagraphHeight(msg.paragraph) + spacing
    end
end

function Messages:registerHandlers(game)
    game.eventBus:registerHandler("warDeclared", function(event)
        self:onWarDeclared(event)
    end)
    game.eventBus:registerHandler("cityCaptured", function(event)
        self:onCityCaptured(event)
    end)
end

return Messages
