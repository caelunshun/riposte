-- Handles packets.
local Client = {}

local pb = require("pb")
local protoc = require("protoc")

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
        self:handlePacket(decodedPacket)

        packet = self.bridge:pollReceivedPacket()
    end
end

function Client:handlePacket(packet)
    -- Packet is an AnyServer message (a union)

end

return Client
