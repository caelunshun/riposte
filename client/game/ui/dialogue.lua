-- The diplomacy dialogue.
local DiplomacyDialogue = {}

local dume = require("dume")
local Vector = require("brinevector")

local Flex = require("widget/flex")
local Text = require("widget/text")
local Button = require("widget/button")
local Container = require("widget/container")
local Image = require("widget/image")
local Padding = require("widget/padding")
local Clickable = require("widget/clickable")
local Spacer = require("widget/spacer")
local Divider = require("widget/divider")

local UIUtils = require("ui/utils")

local Status = {
    Furious = 0,
    Annoyed = 1,
    Cautious = 2,
    Pleased = 3,
    Friendly = 4,
}

local function getGreeting(greetingPlayer, greetedPlayer)
    local r = {}

    local leader = greetingPlayer.leader
    -- local status = greetingPlayer:getStatusWith(greetedPlayer)
    local status = Status.Furious -- TODO

    if status >= Status.Pleased then
        r[#r + 1] = "What can we do for you?"
        r[#r + 1] = "How can we help?"
        r[#r + 1] = "What would you like from us?"

        if status == Status.Friendly then
            r[#r + 1] = string.format("Good evening, %s.", greetedPlayer.username)
        end
    end

    if status >= Status.Cautious then
        r[#r + 1] = "Yes?"
        r[#r + 1] = "What would you like?"

        if leader.aggressive >= 5 then
            r[#r + 1] = "I live by the sword."
        end
    end

    if status <= Status.Annoyed then
        if leader.aggressive >= 5 then
            r[#r + 1] = "I would beat you, but I'd infect my hands."
        end

        r[#r + 1] = "What do you want?"
        r[#r + 1] = "What do you want now?"
        r[#r + 1] = "What now?"

        if leader.paranoia >= 6 then
            r[#r + 1] = "Stop wasting my time."
        end

        if status == Status.Furious then
            r[#r + 1] = "The sight of you infects my eyes."
            r[#r + 1] = "Get out or we'll shoot you."
        end
    end

    math.randomseed(os.time())
    local index = math.random(1, #r)
    return r[index]
end

local function getWarDeclaration(greetingPlayer, greetedPlayer)
    local r = {}

    r[#r + 1] = "Our troops will be conducting some training exercises. On your land."
    r[#r + 1] = "I've had enough of you, " .. greetedPlayer.username .. "."
    r[#r + 1] = "Prepare to die."
    r[#r + 1] = "Say goodbye to your cities, " .. greetedPlayer.username .. "."

    if greetingPlayer.leader.aggressive >= 6 then
        r[#r + 1] = "You are but an ant to us. We will stampede you on the path to domination."
    end

    math.randomseed(os.time())
    local index = math.random(1, #r)
    return r[index] .. string.format(" (%s declares war!)", greetingPlayer.username)
end

-- Current "page" of a diplomacy dialogue.
local State = {
    Main = 0,
    DeclareWar = 1,
}

DiplomacyDialogue.State = State

function DiplomacyDialogue:new(game, withPlayer, initialState)
    local o = {
        game = game,
        withPlayer = withPlayer,
    }
    setmetatable(o, self)
    self.__index = self
    o:setState(initialState or State.Main)
    return o
end

function DiplomacyDialogue:setState(newState)
    self.state = newState
    self.opponentMessage = self:getOpponentMessage()
    self.options = self:getOptions()
    -- rebuild
    self:build()
end

-- Gets the opponent's current message (e.g. a greeeting.)
function DiplomacyDialogue:getOpponentMessage()
    if self.state == State.Main then
        return getGreeting(self.withPlayer, self.game.thePlayer)
    elseif self.state == State.DeclareWar then
        return getWarDeclaration(self.withPlayer, self.game.thePlayer)
    end
end

-- Gets the list of options the player may choose from.
-- Each option is a table with fields `text` and `onselect`.
function DiplomacyDialogue:getOptions()
    if self.state == State.Main then return self:getMainOptions()
    elseif self.state == State.DeclareWar then return self:getWarOptions()
    end
end

function DiplomacyDialogue:getMainOptions()
    local options = {
        {
            text = "Your head would look good on the end of a pole.",
            onselect = function()
                self:declareWar()
            end
        },
        {
            text = "Farewell.",
            onselect = function()
                self:close()
            end
        }
    }

    if self.game.thePlayer:isAtWarWith(self.withPlayer) then
        -- can't declare war again! (TODO:
        table.remove(options, 1)
    end

    return options
end

function DiplomacyDialogue:getWarOptions()
    return {
        {
            text = "So be it.",
            onselect = function()
                self:close()
            end
        }
    }
end

function DiplomacyDialogue:declareWar()
    UIUtils.openConfirmationPrompt(
            "Are you sure you want to declare war on " .. self.withPlayer.username .. "?",
            "Yes, WAR!",
            "No, reconsider...",
            function()
                self.game.client:declareWarOn(self.withPlayer)
                self:close()
            end
    )
end

function DiplomacyDialogue:close()
    self.finished = true
    ui:deleteWindow("diplomacyDialogue")
end

function DiplomacyDialogue:build()
    local root = Flex:column(20)
    root:setCrossAlign(dume.Align.Center)

    root:addFixedChild(Text:new("%opponentName of the %opponentCivName", {
        opponentName = self.withPlayer.username,
        opponentCivName = self.withPlayer.civ.name,
    }))

    root:addFixedChild(Image:new("icon/flag/" .. self.withPlayer.civ.id, 250))

    root:addFixedChild(Text:new("@size{20}{> " .. self.opponentMessage .. "}"))

    root:addFixedChild(Divider:new(2))

    root:addFixedChild(Spacer:new(dume.Axis.Vertical, 50))

    for _, option in ipairs(self.options) do
        local text = Text:new(option.text)
        table.insert(text.classes, "hoverableText")

        local container = Container:new(Padding:new(text, 15))

        local wrapper = Clickable:new(container, function()
            option.onselect()
        end)

        root:addFixedChild(wrapper)
    end

    local container = Container:new(Padding:new(root, 20))
    container.fillParent = true
    table.insert(container.classes, "windowContainer")

    local size = Vector(600, 750)
    ui:createWindow("diplomacyDialogue", UIUtils.centerWindow(size), container)
end

return DiplomacyDialogue
