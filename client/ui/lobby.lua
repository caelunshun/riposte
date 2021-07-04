-- The multiplayer lobby.
local Lobby = {}

local dume = require("dume")
local Vector = require("brinevector")

local Image = require("widget/image")
local Container = require("widget/container")
local Navigator = require("widget/navigator")
local Center = require("widget/center")
local Flex = require("widget/flex")
local Clickable = require("widget/clickable")
local Text = require("widget/text")
local Padding = require("widget/padding")
local Button = require("widget/button")

local UiUtils = require("ui/utils")

local json = require("lunajson")

local ip = "127.0.0.1"
local port = 19836

local function showError(message)
    local root = Flex:column()
    root:addFixedChild(Text:new("@size{20}{%message}", {message=message}))
    root:addFixedChild(Button:new(Text:new("OK"), function()
        ui:deleteWindow("error")
    end))
    root:setCrossAlign(dume.Align.Center)
    local size = Vector(300, 100)
    local container = UiUtils.createWindowContainer(root)
    ui:createWindow("errorDialogue", Vector(cv:getWidth() / 2 - size.x / 2, cv:getHeight() / 2 - size.y / 2), size, container)
end

local function checkError(conn)
    if conn:getError() ~= nil then
        showError(conn:getError())
        return true
    end
    return false
end

function Lobby:new()
    local o = {
        availableGames = {}
    }
    setmetatable(o, self)
    self.__index = self

    o:updateAvailableGames()

    return o
end

function Lobby:updateAvailableGames()
    local conn = NetworkConnection:new(ip, port)
    checkError(conn)
    conn:sendMessage(json.encode({
        type = "requestGameList"
    }))
    if checkError(conn) then return end

    local response = conn:recvMessage()
    if checkError(conn) then return end
    print(response)
    self.availableGames = json.decode(response)
end

function Lobby:createGame()
    local conn = NetworkConnection:new(ip, port)
    checkError(conn)
    conn:sendMessage(json.encode({
        type = "createGame",
        info = {
            numHumanPlayers = 1,
            neededHumanPlayers = 2,
            totalPlayers = 7,
        }
    }))
    checkError(conn)
end

function Lobby:buildRootWidget()
    local root = Flex:column()
    root:setMainAlign(dume.Align.Center)
    root:setCrossAlign(dume.Align.Center)

    root:addFixedChild(Text:new("@size{20}{Multiplayer Games}"))

    root:addFixedChild(Text:new("Available: " .. #self.availableGames))

    root:addFixedChild(Button:new(Text:new("@size{18}{Create Game}"), function()
        self:createGame()
        self:updateAvailableGames()
    end))

    return root
end

return Lobby
