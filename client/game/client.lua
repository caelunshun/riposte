-- Handles packets.
local Client = {}

local pb = require("pb")
local protoc = require("protoc")

local protoFile = io.open("proto/riposte.proto")
local protoData = protoFile:read("*all")
protoFile:close()
protoc:load(protoData)

function Client:new(game, bridge)
    local o = { game = game, bridge = bridge }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Client:handleReceivedPackets()
    local packet = self.bridge:pollReceivedPacket()
    while packet ~= nil do
        local decodedPacket = pb.decode("AnyServer", packet)
        for k, _ in pairs(decodedPacket) do print("Received packet: " .. k) end
        self:handlePacket(decodedPacket)
        packet = self.bridge:pollReceivedPacket()
    end
end

function Client:handlePacket(packet)
    -- Packet is an AnyServer message (a union)
    if packet.updateGlobalData ~= nil then
        self:handleUpdateGlobalData(packet.updateGlobalData)
    elseif packet.updateMap ~= nil then
        self:handleUpdateMap(packet.updateMap)
    elseif packet.updateUnit ~= nil then
        self:handleUpdateUnit(packet.updateUnit)
    end
end

function Client:handleUpdateGlobalData(packet)
    for _, playerdata in ipairs(packet.players) do
        self.game:updatePlayer(playerdata)
    end

    self.game:setTurn(packet.turn)
    self.game:setEra(packet.era)
end

function Client:handleUpdateMap(packet)
    self.game.tiles = packet.tiles
    self.game.mapWidth = packet.width
    self.game.mapHeight = packet.height
    self.game.visibility = packet.visibility
end

function Client:handleUpdateUnit(packet)
    self.game:addUnit(packet)
end

return Client
