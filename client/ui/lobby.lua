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

local json = require("lunajson")

local ip = "127.0.0.1"
local port = 19836

local function checkError(conn)
    if conn:getError() ~= nil then
        error(conn:getError())
    end
end

function Lobby:new()
    local o = {}
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
    checkError(conn)

    local response = conn:recvMessage()
    checkError(conn)
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
