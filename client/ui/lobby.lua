-- The multiplayer lobby.
local Lobby = {}

local json = require("lunajson")

local port = 19836

local function checkError(conn)
    if conn:getError() ~= nil then
        error(conn:getError())
    end
end

function Lobby:new()
    local conn = NetworkConnection:new("127.0.0.1", port)
    checkError(conn)
    conn:sendMessage(json.encode({
        createGame = {
            info = {
                numHumanPlayers = 1,
                neededHumanPlayers = 2,
                totalPlayers = 7,
            }
        }
    }))
    checkError(conn)
end

return function(ui)
    Lobby:new()
end
