local Player = {}

function Player:new(data)
    data.civ = registry.civs[data.civID]
    if data.civ ~= nil then print("received invalid civ " .. data.civID .. "!") end

    setmetatable(data, self)
    self.__index =self
    return data
end

return Player
